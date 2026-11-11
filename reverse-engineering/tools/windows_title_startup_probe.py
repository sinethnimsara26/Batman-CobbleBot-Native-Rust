#!/usr/bin/env python3
"""HUD-REAL-06A: real Win32 cold launch with Enter already physically held.

Uses OS SendInput, launches the unmodified default executable (NO QA flags),
captures genuine desktop screenshots, and distinguishes the colorful title
scene from the intro's black upper band. CI-only; not a shipped runtime.
"""
from __future__ import annotations
import argparse
import ctypes
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time
from ctypes import wintypes
from PIL import Image

from windows_gui_pilot import find_window, send_enter, screenshot

def near_black_top_fraction(image: Image.Image) -> float:
    # The original title art fills this zone with colorful Gotham art;
    # the story intro uses an opaque black upper panel. These two
    # first-party capture landmarks are intentionally very different.
    w,h=image.size
    area=image.crop((round(w*0.12),round(h*0.07),
                     round(w*0.88),round(h*0.27)))
    pixels=area.get_flattened_data() if hasattr(area,"get_flattened_data") else area.getdata()
    return sum(1 for r,g,b in pixels if max(r,g,b)<30)/(area.width*area.height)

def run(exe:Path,out:Path):
    out.mkdir(parents=True,exist_ok=True)
    report={"scenario":"WIN-TITLE-HELD-LAUNCH-06A",
            "build_sha":os.environ.get("GITHUB_SHA"),
            "platform":platform.platform(),
            "launch_mode":"normal user EXE, no --qa-navigation",
            "captures":[],"events":{},"top_dark_fraction":{},
            "classification":"NOT_TESTED","errors":[],
            "limits":"Visual threshold classifies tested title and intro scenes; not exact original Flash behavior."}
    user32=None
    proc=None
    enter_held=False
    try:
        if sys.platform!="win32":
            raise RuntimeError("Windows-only native startup test")
        if not exe.is_file():
            raise FileNotFoundError("Missing EXE: "+str(exe))
        user32=ctypes.WinDLL("user32",use_last_error=True)
        user32.GetForegroundWindow.restype=wintypes.HWND
        user32.SetForegroundWindow.argtypes=[wintypes.HWND]
        user32.SetForegroundWindow.restype=wintypes.BOOL
        user32.ShowWindow.argtypes=[wintypes.HWND,ctypes.c_int]
        user32.GetWindowRect.argtypes=[wintypes.HWND,ctypes.POINTER(
            __import__("windows_gui_pilot").RECT)]
        user32.IsWindowVisible.argtypes=[wintypes.HWND]
        user32.GetWindowTextW.argtypes=[wintypes.HWND,wintypes.LPWSTR,ctypes.c_int]
        user32.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
        user32.GetWindowThreadProcessId.restype=wintypes.DWORD
        user32.SendInput.argtypes=[
            wintypes.UINT,ctypes.POINTER(__import__("windows_gui_pilot").INPUT),ctypes.c_int]
        user32.SendInput.restype=wintypes.UINT
        # Release any leftover synthetic Enter before the controlled trial.
        send_enter(user32,True)
        time.sleep(0.12)
        report["events"]["prelaunch_enter_down"]=send_enter(user32,False)
        if report["events"]["prelaunch_enter_down"]["sent"]!=1:
            raise RuntimeError("OS rejected prelaunch Enter keydown")
        enter_held=True
        # The real game starts while Enter is already DOWN, as when launched
        # immediately after typing Start-Process <path> + Enter in PowerShell.
        proc=subprocess.Popen([str(exe.resolve())],cwd=str(Path.cwd()),
                stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        report["events"]["pid"]=proc.pid
        found=None
        for _ in range(32):
            if proc.poll() is not None:
                raise RuntimeError("Game exited before window, code "+str(proc.returncode))
            found=find_window(user32,proc.pid)
            if found: break
            time.sleep(0.25)
        if not found: raise RuntimeError("No native game window found")
        hwnd,title=found
        report["events"]["title"]=title
        user32.ShowWindow(hwnd,5)
        user32.SetForegroundWindow(hwnd)
        time.sleep(0.85)
        if user32.GetForegroundWindow()!=hwnd:
            raise RuntimeError("OS foreground focus not confirmed")
        held=screenshot(user32,hwnd,out/"06a_title_enter_held.png",report)
        report["top_dark_fraction"]["enter_held"]=near_black_top_fraction(held)
        time.sleep(0.55)
        held_later=screenshot(user32,hwnd,out/"06a_title_enter_held_later.png",report)
        report["top_dark_fraction"]["enter_held_later"]=near_black_top_fraction(held_later)
        report["events"]["enter_released"]=send_enter(user32,True)
        enter_held=False
        if report["events"]["enter_released"]["sent"]!=1:
            raise RuntimeError("OS rejected release of inherited Enter key")
        time.sleep(0.38)
        released=screenshot(user32,hwnd,out/"06a_title_after_release.png",report)
        report["top_dark_fraction"]["released"]=near_black_top_fraction(released)
        # The key must have been released before the subsequent fresh press.
        report["events"]["fresh_press_down"]=send_enter(user32,False)
        enter_held=True
        time.sleep(0.20)
        report["events"]["fresh_press_up"]=send_enter(user32,True)
        enter_held=False
        time.sleep(0.58)
        intro=screenshot(user32,hwnd,out/"06a_intro_after_fresh_enter.png",report)
        report["top_dark_fraction"]["fresh_enter_intro"]=near_black_top_fraction(intro)
        if report["events"]["fresh_press_down"]["sent"]!=1 or report["events"]["fresh_press_up"]["sent"]!=1:
            raise RuntimeError("OS rejected deliberate Enter down/up")
        dark=report["top_dark_fraction"]
        # Grounded by previous current-branch native screenshots:
        # title: ~0.13 dark in upper center; intro: 1.00 dark.
        for key in ("enter_held","enter_held_later","released"):
            if dark[key]>=0.50:
                raise AssertionError(f"Title changed before fresh Enter: {key}={dark[key]:.3f}")
        if dark["fresh_enter_intro"]<=0.75:
            raise AssertionError(f"Fresh Enter did not enter intro: {dark['fresh_enter_intro']:.3f}")
        report["classification"]="PASS_HELD_ENTER_IGNORED_FRESH_ENTER_ACCEPTED"
    except Exception as e:
        report["classification"]="FAILED"
        report["errors"].append(type(e).__name__+": "+str(e))
    finally:
        if user32 is not None and enter_held:
            try: report["events"]["final_cleanup_enter_up"]=send_enter(user32,True)
            except Exception as e: report["errors"].append("key_cleanup: "+str(e))
        if proc is not None:
            try: proc.terminate(); proc.wait(timeout=3)
            except Exception:
                try: proc.kill(); proc.wait(timeout=3)
                except Exception as e: report["errors"].append("process_cleanup: "+str(e))
        path=out/"title-startup-report.json"
        path.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
        print(json.dumps({"title_startup":report["classification"],
                          "top_dark_fraction":report["top_dark_fraction"],
                          "errors":report["errors"]},indent=2))
    if report["errors"] or report["classification"]!="PASS_HELD_ENTER_IGNORED_FRESH_ENTER_ACCEPTED":
        raise SystemExit("WIN-TITLE-HELD-LAUNCH-06A FAILED; see "+str(path))

if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--exe",required=True,type=Path)
    parser.add_argument("--out",required=True,type=Path)
    args=parser.parse_args()
    run(args.exe,args.out)
