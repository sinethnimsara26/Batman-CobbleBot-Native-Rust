#!/usr/bin/env python3
"""FS-QA-03: true Win32 F/Esc fullscreen QA of the ordinary native Rust EXE.

CI-only Python harness; never shipped with, imported by, or required by the
playable game. Fails on real OS geometry / screenshot / input regressions.
Original Flash fullscreen equivalence and physical monitors remain unproven.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time
from ctypes import wintypes

from windows_gui_pilot import RECT, POINT, find_window, screenshot, send_key
from windows_title_startup_probe import near_black_top_fraction

VK_F = 0x46
VK_ESCAPE = 0x1B
VK_RETURN = 0x0D
GWL_STYLE = -16
WS_POPUP = 0x80000000
WS_OVERLAPPEDWINDOW = 0x00CF0000
SW_MAXIMIZE = 3
MONITOR_DEFAULTTONEAREST = 2


class MONITORINFO(ctypes.Structure):
    _fields_ = [
        ("cbSize", wintypes.DWORD),
        ("rcMonitor", RECT),
        ("rcWork", RECT),
        ("dwFlags", wintypes.DWORD),
    ]


class WINDOWPLACEMENT(ctypes.Structure):
    _fields_ = [
        ("length", wintypes.UINT),
        ("flags", wintypes.UINT),
        ("showCmd", wintypes.UINT),
        ("ptMinPosition", POINT),
        ("ptMaxPosition", POINT),
        ("rcNormalPosition", RECT),
    ]


def coords(rect):
    return [rect.left, rect.top, rect.right, rect.bottom]


def approx_same(a, b, tolerance=4):
    return len(a) == len(b) and all(abs(x - y) <= tolerance for x, y in zip(a, b))


def configure(user32):
    user32.GetForegroundWindow.restype = wintypes.HWND
    user32.SetForegroundWindow.argtypes = [wintypes.HWND]
    user32.SetForegroundWindow.restype = wintypes.BOOL
    user32.ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.ShowWindow.restype = wintypes.BOOL
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(RECT)]
    user32.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(RECT)]
    user32.ClientToScreen.argtypes = [wintypes.HWND, ctypes.POINTER(POINT)]
    user32.GetWindowPlacement.argtypes = [wintypes.HWND, ctypes.POINTER(WINDOWPLACEMENT)]
    user32.GetWindowPlacement.restype = wintypes.BOOL
    user32.GetWindowLongPtrW.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.GetWindowLongPtrW.restype = ctypes.c_ssize_t
    user32.MonitorFromWindow.argtypes = [wintypes.HWND, wintypes.DWORD]
    user32.MonitorFromWindow.restype = wintypes.HANDLE
    user32.GetMonitorInfoW.argtypes = [wintypes.HANDLE, ctypes.POINTER(MONITORINFO)]
    user32.GetMonitorInfoW.restype = wintypes.BOOL
    user32.SendInput.argtypes = [wintypes.UINT, ctypes.c_void_p, ctypes.c_int]
    user32.SendInput.restype = wintypes.UINT


def snapshot(user32, hwnd):
    wr, cr = RECT(), RECT()
    pl = WINDOWPLACEMENT()
    pl.length = ctypes.sizeof(pl)
    if not user32.GetWindowRect(hwnd, ctypes.byref(wr)):
        raise RuntimeError("GetWindowRect failed")
    if not user32.GetClientRect(hwnd, ctypes.byref(cr)):
        raise RuntimeError("GetClientRect failed")
    if not user32.GetWindowPlacement(hwnd, ctypes.byref(pl)):
        raise RuntimeError("GetWindowPlacement failed")
    pt = POINT(0, 0)
    if not user32.ClientToScreen(hwnd, ctypes.byref(pt)):
        raise RuntimeError("ClientToScreen failed")
    monitor = user32.MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)
    if not monitor:
        raise RuntimeError("MonitorFromWindow failed")
    mi = MONITORINFO()
    mi.cbSize = ctypes.sizeof(mi)
    if not user32.GetMonitorInfoW(monitor, ctypes.byref(mi)):
        raise RuntimeError("GetMonitorInfoW failed")
    cw, ch = cr.right - cr.left, cr.bottom - cr.top
    if cw <= 0 or ch <= 0:
        raise AssertionError("Empty game client rect")
    # Mirrors unchanged src/win32.rs::viewport_for_client (600x400).
    if cw * 400 <= ch * 600:
        dw, dh = cw, cw * 400 // 600
    else:
        dw, dh = ch * 600 // 400, ch
    x, y = (cw - dw) // 2, (ch - dh) // 2
    if abs(dw * 2 - dh * 3) > 2:
        raise AssertionError("3:2 logical viewport was distorted")
    return {
        "window_rect": coords(wr),
        "client_size": [cw, ch],
        "client_screen_origin": [pt.x, pt.y],
        "style": user32.GetWindowLongPtrW(hwnd, GWL_STYLE) & 0xffffffff,
        "show_cmd": pl.showCmd,
        "normal_rect": coords(pl.rcNormalPosition),
        "monitor_rect": coords(mi.rcMonitor),
        "monitor_work_rect": coords(mi.rcWork),
        "viewport": [x, y, dw, dh],
        "foreground": user32.GetForegroundWindow() == hwnd,
    }


def wait_snapshot(user32, hwnd, predicate, description):
    last = None
    for _ in range(32):
        last = snapshot(user32, hwnd)
        if predicate(last):
            return last
        time.sleep(0.075)
    raise AssertionError(description + " actual=" + json.dumps(last))


def press(user32, key, name, events, delay=0.18):
    down = send_key(user32, key, False)
    time.sleep(delay)  # > 4 native 25Hz engine ticks
    up = send_key(user32, key, True)
    events.append({"key": name, "keydown": down, "keyup": up})
    if down["sent"] != 1 or up["sent"] != 1:
        raise AssertionError("SendInput failed for " + name)


def capture(user32, hwnd, out, report, filename, state):
    img = screenshot(user32, hwnd, out / filename, report)
    # Crop via real client-to-screen origin, not window chrome or black bars.
    sx, sy = state["client_screen_origin"]
    vx, vy, dw, dh = state["viewport"]
    bbox = report["captures"][-1]["box"]
    left, top = sx + vx - bbox[0], sy + vy - bbox[1]
    stage = (left, top, left + dw, top + dh)
    if left < 0 or top < 0 or stage[2] > img.width or stage[3] > img.height:
        raise AssertionError("Full logical viewport outside OS desktop capture: " + str(stage))
    area = img.crop(stage)
    fraction = near_black_top_fraction(area)
    assert area.width == dw and area.height == dh
    measure = {"file": filename, "stage_crop": list(stage), "top_dark_fraction": fraction,
               "image_size": list(img.size), "viewport": state["viewport"]}
    # Verify REAL black letterbox strips in a full-monitor capture.
    # Sample strictly inside the bars, clear of clipping boundaries.
    if approx_same(state["window_rect"], state["monitor_rect"], 1):
        cw, ch = state["client_size"]
        samples = []
        if vx >= 8:
            for px, py in [(left - vx // 2, top + dh // 2),
                           (left + dw + vx // 2, top + dh // 2)]:
                samples.append(img.getpixel((px, py)))
        if vy >= 8:
            for px, py in [(left + dw // 2, top - vy // 2),
                           (left + dw // 2, top + dh + vy // 2)]:
                samples.append(img.getpixel((px, py)))
        if samples and any(max(pixel) > 25 for pixel in samples):
            raise AssertionError("Non-black OS letterbox pixels: " + str(samples))
        measure["letterbox_samples"] = samples
        measure["letterbox_classification"] = "BLACK_BARS_VERIFIED" if samples else "EXACT_3_TO_2_NO_BARS"
    report["visuals"].append(measure)
    return fraction


def validate_fullscreen(state, name):
    if state["style"] & WS_POPUP == 0:
        raise AssertionError(name + " did not acquire WS_POPUP")
    if state["style"] & WS_OVERLAPPEDWINDOW:
        raise AssertionError(name + " retained overlapped window chrome")
    if not approx_same(state["window_rect"], state["monitor_rect"], 1):
        raise AssertionError(name + " did not cover full monitor: " +
                             str(state["window_rect"]) + " vs " + str(state["monitor_rect"]))
    if not state["foreground"]:
        raise AssertionError(name + " lost foreground focus")


def validate_restored(state, saved, name, maximized=False):
    if state["style"] != saved["style"]:
        raise AssertionError(name + " did not restore original window style")
    if state["show_cmd"] != saved["show_cmd"]:
        raise AssertionError(name + " did not restore show/maximize state")
    if not approx_same(state["normal_rect"], saved["normal_rect"], 2):
        raise AssertionError(name + " did not restore WINDOWPLACEMENT normal rectangle")
    if not approx_same(state["window_rect"], saved["window_rect"], 5):
        raise AssertionError(name + " did not restore visual window bounds")
    if not state["foreground"]:
        raise AssertionError(name + " lost foreground focus")
    if maximized and state["show_cmd"] != SW_MAXIMIZE:
        raise AssertionError(name + " unexpectedly left maximized state")


def run(exe, out):
    out.mkdir(parents=True, exist_ok=True)
    report = {
        "scenario": "FS-QA-03",
        "build_sha": os.environ.get("GITHUB_SHA"),
        "launch": "ordinary native Rust Windows EXE, no QA CLI flag",
        "platform": platform.platform(),
        "status": "NOT_RUN",
        "snapshots": {}, "events": [], "visuals": [], "captures": [], "errors": [],
        "limits": ("Single available hosted Windows desktop only; no claim of actual "
                   "1920x1080 display, multi-monitor transitions, physical keyboard/audio, "
                   "taskbar visibility in all configurations, or original Flash equivalence."),
    }
    user32 = None
    proc = None
    try:
        if sys.platform != "win32":
            raise RuntimeError("Windows-only actual-EXE fullscreen test")
        if not exe.is_file():
            raise FileNotFoundError(str(exe))
        user32 = ctypes.WinDLL("user32", use_last_error=True)
        configure(user32)
        report["monitor_count"] = user32.GetSystemMetrics(80)  # SM_CMONITORS
        for k in (VK_F, VK_ESCAPE, VK_RETURN):
            send_key(user32, k, True)
        time.sleep(0.25)
        proc = subprocess.Popen([str(exe.resolve())], cwd=str(Path.cwd()),
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        report["pid"] = proc.pid
        found = None
        for _ in range(40):
            if proc.poll() is not None:
                raise RuntimeError("Game exited before window: " + str(proc.returncode))
            found = find_window(user32, proc.pid)
            if found:
                break
            time.sleep(0.25)
        if not found:
            raise RuntimeError("Game window not found")
        hwnd, title = found
        report["title"] = title
        user32.ShowWindow(hwnd, 5)
        user32.SetForegroundWindow(hwnd)
        time.sleep(0.65)  # leave initial F/Escape edge gates released
        normal = snapshot(user32, hwnd)
        if not normal["foreground"]:
            raise RuntimeError("Window was not foreground: SendInput would be ambiguous")
        if normal["style"] & WS_POPUP:
            raise AssertionError("Ordinary game unexpectedly starts fullscreen")
        report["snapshots"]["normal"] = normal
        capture(user32, hwnd, out, report, "fs_01_normal_title.png", normal)

        press(user32, VK_F, "F enter normal fullscreen", report["events"])
        full = wait_snapshot(user32, hwnd,
                             lambda s: bool(s["style"] & WS_POPUP),
                             "F fullscreen transition timed out")
        time.sleep(0.28)
        full = snapshot(user32, hwnd)
        validate_fullscreen(full, "normal-window F")
        report["snapshots"]["fullscreen_title"] = full
        title_dark = capture(user32, hwnd, out, report, "fs_02_fullscreen_title.png", full)
        if title_dark >= 0.55:
            raise AssertionError("Fullscreen title changed scenes or rendered mostly black")

        press(user32, VK_RETURN, "Enter start intro while fullscreen", report["events"])
        time.sleep(0.24)
        intro_full = snapshot(user32, hwnd)
        validate_fullscreen(intro_full, "fullscreen intro")
        report["snapshots"]["fullscreen_intro"] = intro_full
        dark = capture(user32, hwnd, out, report, "fs_03_fullscreen_intro.png", intro_full)
        if dark <= 0.75:
            raise AssertionError("Full screen Enter did not enter source-derived intro")

        press(user32, VK_ESCAPE, "Esc restore during intro", report["events"])
        restored = wait_snapshot(user32, hwnd,
                                 lambda s: not bool(s["style"] & WS_POPUP),
                                 "Escape failed to restore windowed style")
        time.sleep(0.22)
        restored = snapshot(user32, hwnd)
        validate_restored(restored, normal, "intro escape")
        report["snapshots"]["restored_intro"] = restored
        dark = capture(user32, hwnd, out, report, "fs_04_restored_intro.png", restored)
        if dark <= 0.75:
            raise AssertionError("Esc switched away from intro unexpectedly")
        # A wrongly propagated Esc starts a 41-tick fade (~1.64s), then
        # enters Level1A. Confirm the story remains alive past that interval.
        time.sleep(1.90)
        stayed = snapshot(user32, hwnd)
        validate_restored(stayed, normal, "post-Esc intro")
        report["snapshots"]["intro_after_skip_window"] = stayed
        dark = capture(user32, hwnd, out, report, "fs_05_intro_after_2s.png", stayed)
        if dark <= 0.75:
            raise AssertionError("Fullscreen Esc leaked into game intro skip")
        report["intro_esc_not_skipped"] = True

        # WINDOWPLACEMENT includes previously maximized state, not just x/y.
        user32.ShowWindow(hwnd, SW_MAXIMIZE)
        time.sleep(0.45)
        maximized = snapshot(user32, hwnd)
        if maximized["show_cmd"] != SW_MAXIMIZE:
            raise AssertionError("Could not enter maximized state for restore test")
        report["snapshots"]["maximized_before"] = maximized
        capture(user32, hwnd, out, report, "fs_06_maximized_before.png", maximized)
        press(user32, VK_F, "F enter from maximized", report["events"])
        full_max = wait_snapshot(user32, hwnd,
                                 lambda s: bool(s["style"] & WS_POPUP),
                                 "Maximized-window F transition timed out")
        time.sleep(0.25)
        full_max = snapshot(user32, hwnd)
        validate_fullscreen(full_max, "maximized-window F")
        report["snapshots"]["maximized_fullscreen"] = full_max
        capture(user32, hwnd, out, report, "fs_07_maximized_fullscreen.png", full_max)
        press(user32, VK_ESCAPE, "Esc restore previous maximized state", report["events"])
        restored_max = wait_snapshot(user32, hwnd,
                                     lambda s: not bool(s["style"] & WS_POPUP),
                                     "Maximized Escape failed to restore style")
        time.sleep(0.26)
        restored_max = snapshot(user32, hwnd)
        validate_restored(restored_max, maximized, "maximized restore", maximized=True)
        report["snapshots"]["maximized_restored"] = restored_max
        capture(user32, hwnd, out, report, "fs_08_maximized_restored.png", restored_max)
        report["status"] = "PASS_NATIVE_FULLSCREEN_TITLE_INTRO_RESTORE_MAXIMIZED"
    except Exception as exc:
        report["status"] = "FAIL"
        report["errors"].append(type(exc).__name__ + ": " + str(exc))
    finally:
        if user32:
            for key in (VK_F, VK_ESCAPE, VK_RETURN):
                try:
                    report["events"].append({"cleanup_keyup": key,
                                             "result": send_key(user32, key, True)})
                except Exception as exc:
                    report["errors"].append("key cleanup: " + str(exc))
        if proc is not None:
            try:
                proc.terminate()
                proc.wait(timeout=3)
            except Exception:
                try:
                    proc.kill()
                    proc.wait(timeout=3)
                except Exception as exc:
                    report["errors"].append("process cleanup: " + str(exc))
        target = out / "fs-qa-03-report.json"
        target.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"fs_qa": report["status"], "captures": len(report["captures"]),
                          "monitor_count": report.get("monitor_count"),
                          "errors": report["errors"]}, indent=2))
    if report["errors"] or not report["status"].startswith("PASS_"):
        raise SystemExit("FS-QA-03 failed; inspect " + str(target))


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--exe", required=True, type=Path)
    p.add_argument("--out", required=True, type=Path)
    a = p.parse_args()
    run(a.exe, a.out)
