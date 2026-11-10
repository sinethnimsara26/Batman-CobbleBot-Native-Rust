#!/usr/bin/env python3
"""INTRO-AUDIO-QA-05: real Windows foreground/focus and in-process MCI clock.

Standalone, CI-only, no shipped Python runtime. The native Rust executable
queries its own original MCI alias under explicit BCB_FOCUS_QA_LOG opt-in.
Checks position stability during real Win32 focus loss, resumed transport,
and frozen/advancing native 25Hz story frames. No speaker-loopback claim.
"""
from __future__ import annotations

import argparse
import csv
import ctypes
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

TITLE = "The Batman - CobbleBot Caper - Level 1A Fidelity Slice"
VK_RETURN = 0x0D
WS_OVERLAPPEDWINDOW = 0x00CF0000
WS_VISIBLE = 0x10000000
PM_REMOVE = 0x0001

WNDPROC = ctypes.WINFUNCTYPE(ctypes.c_ssize_t, wintypes.HWND,
                             wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM)


class WNDCLASSW(ctypes.Structure):
    _fields_ = [
        ("style", wintypes.UINT), ("lpfnWndProc", WNDPROC),
        ("cbClsExtra", ctypes.c_int), ("cbWndExtra", ctypes.c_int),
        ("hInstance", wintypes.HINSTANCE), ("hIcon", wintypes.HANDLE),
        ("hCursor", wintypes.HANDLE), ("hbrBackground", wintypes.HANDLE),
        ("lpszMenuName", wintypes.LPCWSTR), ("lpszClassName", wintypes.LPCWSTR),
    ]


class MSG(ctypes.Structure):
    _fields_ = [
        ("hwnd", wintypes.HWND), ("message", wintypes.UINT),
        ("wParam", wintypes.WPARAM), ("lParam", wintypes.LPARAM),
        ("time", wintypes.DWORD), ("pt", POINT),
        ("lPrivate", wintypes.DWORD),
    ]


def prepare_user32(user32, kernel32):
    user32.GetForegroundWindow.restype = wintypes.HWND
    user32.SetForegroundWindow.argtypes = [wintypes.HWND]
    user32.SetForegroundWindow.restype = wintypes.BOOL
    user32.ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
    user32.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.IsWindowVisible.argtypes = [wintypes.HWND]
    user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(RECT)]
    user32.SendInput.argtypes = [wintypes.UINT, ctypes.c_void_p, ctypes.c_int]
    user32.SendInput.restype = wintypes.UINT
    user32.DefWindowProcW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
    user32.DefWindowProcW.restype = ctypes.c_ssize_t
    user32.RegisterClassW.argtypes = [ctypes.POINTER(WNDCLASSW)]
    user32.RegisterClassW.restype = wintypes.ATOM
    user32.CreateWindowExW.argtypes = [
        wintypes.DWORD, wintypes.LPCWSTR, wintypes.LPCWSTR, wintypes.DWORD,
        ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int,
        wintypes.HWND, wintypes.HANDLE, wintypes.HINSTANCE, ctypes.c_void_p,
    ]
    user32.CreateWindowExW.restype = wintypes.HWND
    user32.DestroyWindow.argtypes = [wintypes.HWND]
    user32.PeekMessageW.argtypes = [
        ctypes.POINTER(MSG), wintypes.HWND, wintypes.UINT, wintypes.UINT, wintypes.UINT,
    ]
    user32.PeekMessageW.restype = wintypes.BOOL
    user32.TranslateMessage.argtypes = [ctypes.POINTER(MSG)]
    user32.DispatchMessageW.argtypes = [ctypes.POINTER(MSG)]
    kernel32.GetModuleHandleW.argtypes = [wintypes.LPCWSTR]
    kernel32.GetModuleHandleW.restype = wintypes.HINSTANCE


def pump(user32, seconds):
    end = time.monotonic() + seconds
    message = MSG()
    while time.monotonic() < end:
        for _ in range(100):
            if not user32.PeekMessageW(ctypes.byref(message), None, 0, 0, PM_REMOVE):
                break
            user32.TranslateMessage(ctypes.byref(message))
            user32.DispatchMessageW(ctypes.byref(message))
        time.sleep(0.01)


def build_focus_sink(user32, kernel32):
    class_name = "BCBAudioFocusQa" + str(os.getpid())

    @WNDPROC
    def callback(hwnd, msg, wp, lp):
        return user32.DefWindowProcW(hwnd, msg, wp, lp)

    wc = WNDCLASSW()
    wc.lpfnWndProc = callback
    wc.hInstance = kernel32.GetModuleHandleW(None)
    wc.lpszClassName = class_name
    if not user32.RegisterClassW(ctypes.byref(wc)):
        raise RuntimeError("Could not register test foreground window class")
    # Place at bottom of hosted desktop, away from the intro image's top ROI.
    hwnd = user32.CreateWindowExW(
        0, class_name, "Audio focus test - alternate foreground window",
        WS_VISIBLE | WS_OVERLAPPEDWINDOW, 8, 645, 320, 102,
        None, None, wc.hInstance, None,
    )
    if not hwnd:
        raise RuntimeError("Could not create real Win32 foreground test window")
    return hwnd, callback


def load_trace(path):
    if not path.is_file():
        return []
    with path.open(encoding="utf-8", newline="") as f:
        return list(csv.DictReader(f, delimiter="\t"))


def status(row, key, require_mci=True):
    """Real native intro tick/focus evidence independently of MCI hardware.

    An unavailable driver is NOT considered a passing audio test. We still
    finish the genuine OS-focus visual checks and keep the CI step red.
    """
    if row is None:
        raise AssertionError("Missing native game telemetry at " + key)
    if row.get("screen") != "Intro":
        raise AssertionError("Native window left Intro during focus experiment: " + str(row))
    for field in ("elapsed_ms", "intro_tick"):
        if not str(row.get(field, "")).isdigit():
            raise AssertionError("Malformed native focus/intro telemetry: " + str(row))
    pos = None
    if require_mci:
        for field in ("mode_error", "position_error"):
            if row.get(field) != "0":
                raise AssertionError("MCI driver rejected status " + field + ": " + str(row))
        try:
            pos = int(row["mci_position"])
        except (ValueError, KeyError) as exc:
            raise AssertionError("MCI playback position unavailable: " + str(row)) from exc
    return {"time_ms": int(row["elapsed_ms"]), "position": pos,
            "intro_tick": int(row["intro_tick"]), "mode": row.get("mci_mode", "").strip().lower(),
            "transport": row.get("transport"), "active": row.get("active") == "1",
            "reason": row.get("reason"), "format": row.get("mci_time_format"),
            "open_error": row.get("open_error"), "open_message": row.get("open_message"),
            "file_written": row.get("file_written")}


def last_intro(rows, active=None, reason=None):
    candidates = [
        r for r in rows
        if r.get("screen") == "Intro"
        and (active is None or (r.get("active") == "1") == active)
        and (reason is None or r.get("reason") == reason)
    ]
    return candidates[-1] if candidates else None


def require_foreground(user32, target, label, seconds=1.6):
    user32.ShowWindow(target, 5)
    user32.SetForegroundWindow(target)
    for _ in range(int(seconds / 0.04)):
        if user32.GetForegroundWindow() == target:
            return
        pump(user32, 0.04)
    raise AssertionError("Real Win32 foreground transition denied: " + label)


def capture(user32, hwnd, path, report, label):
    img = screenshot(user32, hwnd, path, report)
    report.setdefault("image_roles", []).append(label)
    return near_black_top_fraction(img)



def audio_services_preflight():
    """Report hosted-VM audio availability; attempt benign service starts.

    This is strictly CI-only, never used or needed by normal game users.
    A missing driver/device does NOT turn the strict real MCI test into a pass.
    """
    script = r'''
$ErrorActionPreference = 'Continue'
$result = [ordered]@{
    os = [System.Environment]::OSVersion.VersionString
    services = [ordered]@{}
    audio_endpoints = @()
    mci_mpegvideo = ''
    diagnostics = @()
}
foreach ($n in @('AudioEndpointBuilder', 'Audiosrv')) {
    try {
        $svc = Get-Service -Name $n -ErrorAction Stop
        $before = $svc.Status.ToString()
        if ($svc.Status -ne 'Running') {
            try {
                Start-Service -Name $n -ErrorAction Stop
                Start-Sleep -Milliseconds 250
            } catch {
                $result.diagnostics += ('Start-Service ' + $n + ': ' + $_.Exception.Message)
            }
        }
        $after = (Get-Service -Name $n -ErrorAction Stop).Status.ToString()
        $result.services[$n] = [ordered]@{
            before = $before
            after = $after
            start_type = $svc.StartType.ToString()
        }
    } catch {
        $result.services[$n] = [ordered]@{ error = $_.Exception.Message }
    }
}
try {
    $endpoints = @(Get-PnpDevice -Class AudioEndpoint -PresentOnly -ErrorAction Stop)
    $result.audio_endpoints = @($endpoints | ForEach-Object {
        [ordered]@{ name = $_.FriendlyName; status = $_.Status }
    })
} catch {
    $result.diagnostics += ('AudioEndpoint enumeration: ' + $_.Exception.Message)
}
try {
    $reg = Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MCI32' -ErrorAction Stop
    $result.mci_mpegvideo = [string]$reg.mpegvideo
} catch {
    $result.diagnostics += ('MCI32 registry: ' + $_.Exception.Message)
}
$result | ConvertTo-Json -Compress -Depth 5
'''
    try:
        result = subprocess.run(
            ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
            capture_output=True, text=True, timeout=35, check=False,
        )
        return {
            "exit_code": result.returncode,
            "values": json.loads(result.stdout.strip()) if result.stdout.strip().startswith("{") else None,
            "stdout_tail": result.stdout[-1000:],
            "stderr_tail": result.stderr[-2000:],
        }
    except Exception as exc:
        return {"error": type(exc).__name__ + ": " + str(exc)}


def run(exe, out):
    out.mkdir(parents=True, exist_ok=True)
    trace_path = out / "intro-audio-mci.tsv"
    report = {
        "scenario": "INTRO-AUDIO-QA-05",
        "platform": platform.platform(),
        "build_sha": os.environ.get("GITHUB_SHA"),
        "launch": "normal Rust Windows EXE; only read-only BCB_FOCUS_QA_LOG opt-in env",
        "status": "NOT_RUN",
        "captures": [],
        "events": {},
        "measurements": {},
        "errors": [],
        "limits": ("Win32 MCI status is a real native media-driver query inside "
                   "the game process; audio device speaker/loopback not captured, "
                   "taskbar volume flyout not specifically interacted with, "
                   "and original Flash runtime behavior not established."),
    }
    user32 = None
    proc = None
    sink = None
    callback = None
    try:
        if sys.platform != "win32":
            raise RuntimeError("This native OS focus/MCI test requires Windows")
        if not exe.is_file():
            raise FileNotFoundError(exe)
        user32 = ctypes.WinDLL("user32", use_last_error=True)
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        prepare_user32(user32, kernel32)
        report["audio_service_preflight"] = audio_services_preflight()
        print("Audio service/device preflight:", json.dumps(report["audio_service_preflight"], indent=2))
        # Neutralize leftover injected keyboard state from existing GUI tests.
        send_key(user32, VK_RETURN, True)
        env = dict(os.environ)
        env["BCB_FOCUS_QA_LOG"] = str(trace_path.resolve())
        proc = subprocess.Popen(
            [str(exe.resolve())], cwd=str(Path.cwd()), env=env,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        report["pid"] = proc.pid
        found = None
        for _ in range(120):
            if proc.poll() is not None:
                raise RuntimeError("Native game exited before visible HWND: " + str(proc.returncode))
            found = find_window(user32, proc.pid)
            if found:
                break
            time.sleep(0.25)
        if not found:
            raise AssertionError("Native Win32 game HWND not discovered within 30sec")
        hwnd, title = found
        report["window_title"] = title
        require_foreground(user32, hwnd, "game launch")
        pump(user32, 0.65)
        if not trace_path.is_file():
            raise AssertionError("Native Rust opt-in MCI trace file missing")
        before = capture(user32, hwnd, out / "qa05_01_title.png", report, "title")
        if before > 0.58:
            raise AssertionError("Unexpected mostly-black initial title, cannot test Intro")
        down, up = send_key(user32, VK_RETURN, False), None
        pump(user32, 0.19)
        up = send_key(user32, VK_RETURN, True)
        if down["sent"] != 1 or up["sent"] != 1:
            raise AssertionError("Native OS Enter SendInput failed")
        pump(user32, 0.5)
        dark = capture(user32, hwnd, out / "qa05_02_intro_active.png", report, "intro-before-focus")
        if dark < 0.73:
            raise AssertionError("Enter did not start real native intro")
        before_rows = load_trace(trace_path)
        before_row = last_intro(before_rows, active=True)
        if before_row is None:
            raise AssertionError("Real native Intro telemetry was not recorded")
        mci_available = before_row.get("open_error") == "0"
        report["measurements"]["mci_audio_initialized"] = mci_available
        before_status = status(before_row, "before", require_mci=mci_available)
        report["measurements"]["active_before"] = before_status
        if mci_available:
            if "play" not in before_status["mode"]:
                raise AssertionError("MCI driver is not playing original intro: " + str(before_status))
            if before_status["transport"] != "playing":
                raise AssertionError("Rust intro transport did not enter playing state")
        else:
            report["measurements"]["mci_initialization_blocker"] = {
                "open_error": before_status["open_error"],
                "open_message": before_status["open_message"],
                "file_written": before_status["file_written"],
                "audio_service_preflight": report.get("audio_service_preflight"),
            }
        # Exercise real foreground loss even on CI hosts without an MCI output
        # endpoint, providing independent OS/visual evidence in all cases.
        sink, callback = build_focus_sink(user32, kernel32)
        require_foreground(user32, sink, "focus sink")
        pump(user32, 0.38)
        paused_early_rows = load_trace(trace_path)
        paused_early = status(last_intro(paused_early_rows, active=False),
                              "paused_early", require_mci=mci_available)
        capture(user32, hwnd, out / "qa05_03_intro_defocused.png", report,
                "intro-while-other-window-foreground")
        pump(user32, 0.95)
        paused_late_rows = load_trace(trace_path)
        paused_late = status(last_intro(paused_late_rows, active=False),
                             "paused_late", require_mci=mci_available)
        report["measurements"]["paused_early"] = paused_early
        report["measurements"]["paused_late"] = paused_late
        report["events"]["focus_loss_count"] = sum(
            r["reason"] == "focus_loss" for r in paused_late_rows)
        if report["events"]["focus_loss_count"] < 1:
            raise AssertionError("No real game Win32 focus-loss message recorded")
        if paused_early["active"] or paused_late["active"]:
            raise AssertionError("Native game did not actually deactivate")
        if paused_early["intro_tick"] != paused_late["intro_tick"]:
            raise AssertionError("Original intro visuals continued while defocused")
        if paused_late["time_ms"] - paused_early["time_ms"] < 500:
            raise AssertionError("Insufficient real-focus-loss observation interval")
        report["measurements"]["visual_intro_paused_by_real_os_focus"] = True
        if mci_available:
            if (paused_early["transport"] != "paused_for_focus"
                    or paused_late["transport"] != "paused_for_focus"):
                raise AssertionError("Rust audio transport did not enter focus pause")
            if "pause" not in paused_early["mode"] or "pause" not in paused_late["mode"]:
                raise AssertionError("WinMM did not report paused mode")
            pos_change = paused_late["position"] - paused_early["position"]
            report["measurements"]["paused_position_change_units"] = pos_change
            if not (-50 <= pos_change <= 75):
                raise AssertionError("MCI position advanced during pause: " + str(pos_change))
        # Return focus to the same ordinary Rust game window.
        require_foreground(user32, hwnd, "game regain")
        pump(user32, 0.35)
        regain_rows = load_trace(trace_path)
        regained = status(last_intro(regain_rows, active=True),
                          "regained", require_mci=mci_available)
        pump(user32, 0.6)
        final_rows = load_trace(trace_path)
        final_status = status(last_intro(final_rows, active=True),
                              "after-resume", require_mci=mci_available)
        report["measurements"]["regained"] = regained
        report["measurements"]["after_resume"] = final_status
        report["events"]["focus_gain_count"] = sum(
            r["reason"] == "focus_gain" for r in final_rows)
        if report["events"]["focus_gain_count"] < 1:
            raise AssertionError("No real game Win32 focus-gain message recorded")
        if not regained["active"] or not final_status["active"]:
            raise AssertionError("Native game did not become active again")
        if final_status["intro_tick"] - regained["intro_tick"] < 7:
            raise AssertionError("25Hz intro visuals did not resume after Win32 focus gain")
        report["measurements"]["visual_intro_resumed_by_real_os_focus"] = True
        if mci_available:
            if "play" not in regained["mode"] or "play" not in final_status["mode"]:
                raise AssertionError("WinMM did not report resumed playback")
            if final_status["position"] - regained["position"] < 150:
                raise AssertionError("MCI position did not advance after focus regain")
        after_dark = capture(user32, hwnd, out / "qa05_04_intro_resumed.png",
                             report, "intro-after-focus")
        if after_dark < 0.70:
            raise AssertionError("The actual game left its intro unexpectedly")
        if not mci_available:
            report["status"] = "BLOCKED_HOSTED_RUNNER_MCI_INITIALIZATION"
            report["errors"].append(
                "Real OS focus/visual freeze/resume verified, but WinMM MCI audio "
                "initialization failed (error " + str(before_status["open_error"]) +
                ": " + str(before_status["open_message"]) +
                "); actual playback position remains UNVERIFIED")
        else:
            report["status"] = "PASS_REAL_WIN32_FOCUS_MCI_PAUSE_POSITION_RESUME"
    except Exception as exc:
        report["status"] = "FAIL"
        report["errors"].append(type(exc).__name__ + ": " + str(exc))
    finally:
        if user32 is not None:
            try:
                report["events"]["cleanup_enter_up"] = send_key(user32, VK_RETURN, True)
            except Exception as exc:
                report["errors"].append("enter cleanup: " + str(exc))
            if sink is not None:
                try:
                    user32.DestroyWindow(sink)
                except Exception as exc:
                    report["errors"].append("focus sink cleanup: " + str(exc))
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
        # Save any partial native MCI log even after the test fails.
        report["telemetry_rows"] = len(load_trace(trace_path))
        path = out / "intro-audio-qa-05-report.json"
        path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"audio_focus_qa": report["status"],
                          "telemetry_rows": report["telemetry_rows"],
                          "screenshots": len(report["captures"]),
                          "errors": report["errors"]}, indent=2))
    if report["errors"] or report["status"] != "PASS_REAL_WIN32_FOCUS_MCI_PAUSE_POSITION_RESUME":
        raise SystemExit("INTRO-AUDIO-QA-05 FAILED; see " + str(path))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    args = parser.parse_args()
    run(args.exe, args.out)
