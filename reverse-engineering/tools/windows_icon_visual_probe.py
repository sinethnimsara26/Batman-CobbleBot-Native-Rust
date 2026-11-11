#!/usr/bin/env python3
"""ICON-SHELL-PIXEL-QA-09: draw REAL Windows Shell HICONs to PNG evidence.

The prior resource test checked valid PE icons and nonzero Shell HICONs.
This one examines actual Shell-returned pixel content, not a fabricated
Explorer screenshot or a claim about the user's pinned taskbar icon cache.

Python/Pillow and ctypes run only in Windows CI; game runtime stays Rust.
"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import json
from pathlib import Path

from PIL import Image, ImageDraw

from windows_icon_resource_probe import SHFILEINFOW

SHGFI_ICON = 0x100
SHGFI_SMALLICON = 0x1
DIB_RGB_COLORS = 0
DI_NORMAL = 0x3


class BITMAPINFOHEADER(ctypes.Structure):
    _fields_ = [
        ("biSize", wintypes.DWORD),
        ("biWidth", ctypes.c_int32),
        ("biHeight", ctypes.c_int32),
        ("biPlanes", wintypes.WORD),
        ("biBitCount", wintypes.WORD),
        ("biCompression", wintypes.DWORD),
        ("biSizeImage", wintypes.DWORD),
        ("biXPelsPerMeter", ctypes.c_int32),
        ("biYPelsPerMeter", ctypes.c_int32),
        ("biClrUsed", wintypes.DWORD),
        ("biClrImportant", wintypes.DWORD),
    ]


class RGBQUAD(ctypes.Structure):
    _fields_ = [
        ("rgbBlue", ctypes.c_ubyte),
        ("rgbGreen", ctypes.c_ubyte),
        ("rgbRed", ctypes.c_ubyte),
        ("rgbReserved", ctypes.c_ubyte),
    ]


class BITMAPINFO(ctypes.Structure):
    _fields_ = [("bmiHeader", BITMAPINFOHEADER),
                ("bmiColors", RGBQUAD * 1)]


def is_gold(pixel: tuple[int, ...]) -> bool:
    red, green, blue = pixel[:3]
    return red > 145 and green > 82 and blue < 135 and red > green + 22 and green > blue + 12


def golden_ink(image: Image.Image) -> set[tuple[int, int]]:
    rgb = image.convert("RGB")
    return {(x, y) for y in range(rgb.height) for x in range(rgb.width)
            if is_gold(rgb.getpixel((x, y)))}


def draw_shell_icon(shell_hicon, size: int) -> Image.Image:
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    gdi = ctypes.WinDLL("gdi32", use_last_error=True)

    user32.GetDC.argtypes = [wintypes.HWND]
    user32.GetDC.restype = wintypes.HDC
    user32.ReleaseDC.argtypes = [wintypes.HWND, wintypes.HDC]
    user32.ReleaseDC.restype = ctypes.c_int
    user32.DrawIconEx.argtypes = [wintypes.HDC, ctypes.c_int, ctypes.c_int,
                                wintypes.HICON, ctypes.c_int, ctypes.c_int,
                                wintypes.UINT, wintypes.HBRUSH, wintypes.UINT]
    user32.DrawIconEx.restype = wintypes.BOOL

    gdi.CreateCompatibleDC.argtypes = [wintypes.HDC]
    gdi.CreateCompatibleDC.restype = wintypes.HDC
    gdi.CreateDIBSection.argtypes = [
        wintypes.HDC, ctypes.POINTER(BITMAPINFO), wintypes.UINT,
        ctypes.POINTER(ctypes.c_void_p), wintypes.HANDLE, wintypes.DWORD]
    gdi.CreateDIBSection.restype = wintypes.HANDLE
    gdi.SelectObject.argtypes = [wintypes.HDC, wintypes.HANDLE]
    gdi.SelectObject.restype = wintypes.HANDLE
    gdi.DeleteObject.argtypes = [wintypes.HANDLE]
    gdi.DeleteObject.restype = wintypes.BOOL
    gdi.DeleteDC.argtypes = [wintypes.HDC]
    gdi.DeleteDC.restype = wintypes.BOOL

    screen = user32.GetDC(None)
    if not screen:
        raise RuntimeError("GetDC returned no desktop HDC")
    memory = None
    bitmap = None
    previous = None
    try:
        memory = gdi.CreateCompatibleDC(screen)
        if not memory:
            raise RuntimeError("CreateCompatibleDC failed")
        header = BITMAPINFOHEADER()
        header.biSize = ctypes.sizeof(BITMAPINFOHEADER)
        header.biWidth = size
        header.biHeight = -size  # Top-down BGRA, no silent vertical flip.
        header.biPlanes = 1
        header.biBitCount = 32
        header.biCompression = 0
        header.biSizeImage = size * size * 4
        info = BITMAPINFO()
        info.bmiHeader = header
        pointer = ctypes.c_void_p()
        bitmap = gdi.CreateDIBSection(screen, ctypes.byref(info),
                                      DIB_RGB_COLORS, ctypes.byref(pointer),
                                      None, 0)
        if not bitmap or not pointer.value:
            raise RuntimeError("CreateDIBSection failed")
        previous = gdi.SelectObject(memory, bitmap)
        if not previous:
            raise RuntimeError("SelectObject failed")
        # Opaque dark backdrop makes the HICON's real antialiasing visible and
        # avoids interpreting uninitialised alpha bytes as visual content.
        pixel = bytes((32, 29, 27, 255))  # BGRA for RGB(27,29,32).
        ctypes.memmove(pointer, pixel * (size * size), size * size * 4)
        if not user32.DrawIconEx(memory, 0, 0, shell_hicon, size, size,
                                0, None, DI_NORMAL):
            raise RuntimeError(f"DrawIconEx failed: {ctypes.get_last_error()}")
        raw = ctypes.string_at(pointer, size * size * 4)
        return Image.frombytes("RGBA", (size, size), raw, "raw", "BGRA").convert("RGB")
    finally:
        if previous and memory:
            gdi.SelectObject(memory, previous)
        if bitmap:
            gdi.DeleteObject(bitmap)
        if memory:
            gdi.DeleteDC(memory)
        user32.ReleaseDC(None, screen)


def shell_icon_for_file(exe: Path, size: int) -> Image.Image:
    shell = ctypes.WinDLL("shell32", use_last_error=True)
    user32 = ctypes.WinDLL("user32", use_last_error=True)
    shell.SHGetFileInfoW.argtypes = [
        wintypes.LPCWSTR, wintypes.DWORD, ctypes.POINTER(SHFILEINFOW),
        wintypes.UINT, wintypes.UINT]
    shell.SHGetFileInfoW.restype = ctypes.c_size_t
    user32.DestroyIcon.argtypes = [wintypes.HICON]
    user32.DestroyIcon.restype = wintypes.BOOL
    info = SHFILEINFOW()
    flags = SHGFI_ICON | (SHGFI_SMALLICON if size == 16 else 0)
    ok = shell.SHGetFileInfoW(str(exe.resolve()), 0, ctypes.byref(info),
                              ctypes.sizeof(info), flags)
    if not ok or not info.hIcon:
        raise AssertionError(f"SHGetFileInfoW failed at {size}px")
    try:
        return draw_shell_icon(info.hIcon, size)
    finally:
        user32.DestroyIcon(info.hIcon)


def test_shell_colors(exe: Path, artwork: Path, out: Path) -> dict:
    cases = []
    evidence = []
    for size in (16, 32):
        source = Image.open(artwork / f"batman-cobblebot-{size}.png").convert("RGBA")
        if source.size != (size, size):
            raise AssertionError(f"Expected source PNG {size}x{size}")
        # The source PNG is a comparison reference, not an OS screenshot.
        ref = Image.new("RGBA", (size, size), (27, 29, 32, 255))
        ref.alpha_composite(source)
        ref_rgb = ref.convert("RGB")
        shell_rgb = shell_icon_for_file(exe, size)
        ref_file = out / f"artwork-reference-{size}.png"
        os_file = out / f"actual-windows-shell-hicon-{size}.png"
        ref_rgb.save(ref_file)
        shell_rgb.save(os_file)
        expected = golden_ink(ref_rgb)
        observed = golden_ink(shell_rgb)
        # One-pixel tolerance for Windows DPI rasterization/resampling. This
        # does not allow an unrelated generic file icon with no yellow bat.
        covered = sum(any((x + dx, y + dy) in observed
                          for dx in (-1, 0, 1) for dy in (-1, 0, 1))
                      for x, y in expected)
        min_gold = max(3, (len(expected) + 3) // 4)
        min_covered = max(3, (len(expected) + 2) // 3)
        passed = len(expected) >= 5 and len(observed) >= min_gold and covered >= min_covered
        cases.append({"size":size, "source_gold_pixels":len(expected),
                      "shell_gold_pixels":len(observed),
                      "source_gold_with_nearby_shell_ink":covered,
                      "required_shell_gold":min_gold,
                      "required_coverage":min_covered,
                      "status":"PASS_GOLD_BAT_IN_REAL_SHELL_HICON" if passed else "FAIL_SHELL_ICON_PIXEL_MISMATCH",
                      "shell_capture":os_file.name,
                      "source_comparison":ref_file.name})
        evidence.append((size,ref_rgb,shell_rgb))
    # Display sheet upscales only for review; actual shell PNGs remain
    # unmodified at the true small/large taskbar pixel sizes.
    sheet = Image.new("RGB", (584, 365), (228, 231, 237))
    d = ImageDraw.Draw(sheet)
    for i,(size,source,actual) in enumerate(evidence):
        cell = 128
        x = 14 + i * 288
        for j,(name,img) in enumerate((("PNG reference",source),("Windows Shell HICON",actual))):
            tile=img.resize((cell,cell),Image.Resampling.NEAREST)
            y=40 + j * 158
            sheet.paste(tile,(x,y))
            d.text((x,y-16),f"{size}px: {name}",fill=(22,26,34))
    sheet.save(out/"shell-icon-pixel-comparison.png")
    return {"cases":cases,
            "all_passed":all(c["status"].startswith("PASS_") for c in cases),
            "provenance":"Actual SHGetFileInfoW HICON drawn by Win32 DrawIconEx into top-down 32-bit DIB; no Explorer or taskbar screenshot claim.",
            "limits":"Hosted Windows Shell color/content verified; user pinned taskbar icon cache, actual shortcut and aesthetic remain separate."}


def main() -> None:
    parser=argparse.ArgumentParser()
    parser.add_argument("--exe",type=Path,required=True)
    parser.add_argument("--art",type=Path,required=True)
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    result=test_shell_colors(args.exe,args.art,args.out)
    (args.out/"shell-icon-pixel-report.json").write_text(
        json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(result,indent=2))
    if not result["all_passed"]:
        raise SystemExit("Actual Windows Shell icon pixels did not meet source-bat coverage thresholds")


if __name__ == "__main__":
    main()
