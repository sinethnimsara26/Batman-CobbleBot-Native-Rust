#!/usr/bin/env python3
"""Batch07: inspect the REAL linked PE icon resource and Windows Shell icon.

Read-only test executable inspection; no fake window/shell success.
"""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import json
import struct
import subprocess
import time
from pathlib import Path

RT_GROUP_ICON = 14
RT_ICON = 3
ICON_RESOURCE_ID = 101
LOAD_LIBRARY_AS_DATAFILE = 0x2
SHGFI_ICON = 0x100
SHGFI_SMALLICON = 1
GCLP_HICON = -14


class SHFILEINFOW(ctypes.Structure):
    _fields_ = [
        ("hIcon", wintypes.HANDLE),
        ("iIcon", ctypes.c_int),
        ("dwAttributes", wintypes.DWORD),
        ("szDisplayName", wintypes.WCHAR * 260),
        ("szTypeName", wintypes.WCHAR * 80),
    ]


def inspect_resource(exe: Path) -> dict:
    k32 = ctypes.WinDLL("kernel32", use_last_error=True)
    k32.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
    k32.LoadLibraryExW.restype = wintypes.HMODULE
    k32.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
    k32.FindResourceW.restype = wintypes.HANDLE
    k32.SizeofResource.argtypes = [wintypes.HMODULE, wintypes.HANDLE]
    k32.SizeofResource.restype = wintypes.DWORD
    k32.LoadResource.argtypes = [wintypes.HMODULE, wintypes.HANDLE]
    k32.LoadResource.restype = wintypes.HANDLE
    k32.LockResource.argtypes = [wintypes.HANDLE]
    k32.LockResource.restype = ctypes.c_void_p
    k32.FreeLibrary.argtypes = [wintypes.HMODULE]
    hmod = k32.LoadLibraryExW(str(exe.resolve()), None, LOAD_LIBRARY_AS_DATAFILE)
    if not hmod:
        raise RuntimeError(f"Cannot load EXE resource view: WinError {ctypes.get_last_error()}")
    try:
        def resource(resource_id: int, kind: int) -> bytes:
            found = k32.FindResourceW(hmod, ctypes.c_void_p(resource_id), ctypes.c_void_p(kind))
            if not found:
                raise AssertionError(f"Missing PE resource {kind}/{resource_id}")
            length = k32.SizeofResource(hmod, found)
            loaded = k32.LoadResource(hmod, found)
            ptr = k32.LockResource(loaded)
            if not ptr or not length:
                raise AssertionError(f"PE resource {kind}/{resource_id} empty")
            return ctypes.string_at(ptr, length)
        group = resource(ICON_RESOURCE_ID, RT_GROUP_ICON)
        if len(group) < 6:
            raise AssertionError("GROUP_ICON truncated")
        reserved, kind, count = struct.unpack_from("<HHH", group)
        if (reserved, kind, count) != (0, 1, 7):
            raise AssertionError(f"Wrong GROUP_ICON header {(reserved,kind,count)}")
        expected = [16,24,32,48,64,128,256]
        entries = []
        for index, size in enumerate(expected):
            w, h, colors, reserved, planes, bpp, length, ident = struct.unpack_from(
                "<BBBBHHIH", group, 6 + 14 * index)
            if (w or 256, h or 256) != (size,size):
                raise AssertionError(f"PE icon entry {index} unexpected size {w}x{h}")
            payload = resource(ident, RT_ICON)
            if len(payload) != length or not payload.startswith(b"\\x89PNG\\r\\n\\x1a\\n"):
                raise AssertionError(f"Invalid RT_ICON bytes for {size}px")
            entries.append({"size":size,"id":ident,"byte_length":length,"png":True})
        return {"group_resource_id":ICON_RESOURCE_ID,"resource_type":"RT_GROUP_ICON",
                "entries":entries,"status":"PASS_NATIVE_PE_RESOURCE"}
    finally:
        k32.FreeLibrary(hmod)


def inspect_shell_and_window(exe: Path) -> dict:
    shell = ctypes.WinDLL("shell32",use_last_error=True)
    u32 = ctypes.WinDLL("user32",use_last_error=True)
    shell.SHGetFileInfoW.argtypes = [wintypes.LPCWSTR, wintypes.DWORD,
                                  ctypes.POINTER(SHFILEINFOW), wintypes.UINT, wintypes.UINT]
    shell.SHGetFileInfoW.restype = ctypes.c_size_t
    u32.DestroyIcon.argtypes = [wintypes.HANDLE]
    u32.FindWindowW.argtypes = [wintypes.LPCWSTR,wintypes.LPCWSTR]
    u32.FindWindowW.restype = wintypes.HWND
    u32.GetClassLongPtrW.argtypes = [wintypes.HWND,ctypes.c_int]
    u32.GetClassLongPtrW.restype = ctypes.c_size_t

    # SHGetFileInfoW asks the actual Windows Shell to load the file icon;
    # it is not a manually rendered PNG or a fake Explorer screenshot.
    shell_sizes = []
    for size, flag in ((32,0), (16,SHGFI_SMALLICON)):
        info = SHFILEINFOW()
        ok = shell.SHGetFileInfoW(str(exe.resolve()),0,ctypes.byref(info),
                                  ctypes.sizeof(info),SHGFI_ICON|flag)
        if not ok or not info.hIcon:
            raise AssertionError(f"Windows Shell failed to load {size}px EXE icon")
        shell_sizes.append(size)
        u32.DestroyIcon(info.hIcon)

    process = subprocess.Popen([str(exe.resolve())],cwd=str(exe.parent))
    class_icon = False
    try:
        deadline=time.monotonic()+12
        while time.monotonic()<deadline:
            if process.poll() is not None:
                raise AssertionError(f"Native game exited before icon test ({process.returncode})")
            hwnd=u32.FindWindowW("BatmanCobbleBotLevel1AFidelity",None)
            if hwnd:
                class_icon=bool(u32.GetClassLongPtrW(hwnd,GCLP_HICON))
                break
            time.sleep(0.25)
        if not class_icon:
            raise AssertionError("Normal native Win32 game window has no embedded class icon")
    finally:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
    return {"shell_extracted_icon_sizes":shell_sizes,"normal_game_window_class_icon":class_icon,
            "status":"PASS_WINDOWS_SHELL_AND_NATIVE_GAME_ICON"}


def main() -> None:
    parser=argparse.ArgumentParser()
    parser.add_argument("--exe",type=Path,required=True)
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    result={"exe":args.exe.name,"pe_resource":inspect_resource(args.exe),
            "shell_and_native_window":inspect_shell_and_window(args.exe),
            "claim_limit":"Shell API extracted real EXE icon and native window registered class icon; taskbar pin/cache and visual user aesthetic acceptance still need player observation."}
    (args.out/"windows-icon-resource-qa.json").write_text(
        json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print("ICON-EXE-07 PASS",json.dumps(result))


if __name__=="__main__":
    main()
