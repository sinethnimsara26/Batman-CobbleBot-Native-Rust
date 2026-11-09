#!/usr/bin/env python3
"""CI-only, bounded REAL Win32 GUI feasibility probe. Not part of Rust game runtime."""
from __future__ import annotations
import argparse, ctypes, hashlib, json, os, platform, subprocess, sys, time
from ctypes import wintypes
from pathlib import Path
from PIL import ImageGrab, ImageChops

TITLE = "The Batman - CobbleBot Caper - Level 1A Fidelity Slice"
class RECT(ctypes.Structure):
    _fields_ = [("left", wintypes.LONG),("top", wintypes.LONG),("right", wintypes.LONG),("bottom", wintypes.LONG)]
class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", wintypes.WORD),("wScan", wintypes.WORD),("dwFlags", wintypes.DWORD),("time", wintypes.DWORD),("dwExtraInfo", ctypes.c_void_p)]
class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx",wintypes.LONG),("dy",wintypes.LONG),("mouseData",wintypes.DWORD),("dwFlags",wintypes.DWORD),("time",wintypes.DWORD),("dwExtraInfo",ctypes.c_void_p)]
class INPUT_UNION(ctypes.Union):
    _fields_ = [("ki",KEYBDINPUT),("mi",MOUSEINPUT)]
class INPUT(ctypes.Structure):
    _fields_ = [("type",wintypes.DWORD),("data",INPUT_UNION)]

def find_window(user32, pid):
    matches=[]
    cb_type=ctypes.WINFUNCTYPE(wintypes.BOOL,wintypes.HWND,wintypes.LPARAM)
    def visit(hwnd,_):
        actual=wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd,ctypes.byref(actual))
        if actual.value==pid and user32.IsWindowVisible(hwnd):
            title=ctypes.create_unicode_buffer(256)
            user32.GetWindowTextW(hwnd,title,len(title))
            if TITLE in title.value: matches.append((hwnd,title.value))
        return True
    callback=cb_type(visit)
    user32.EnumWindows(callback,0)
    return matches[0] if matches else None

def send_enter(user32,up):
    event=INPUT()
    event.type=1 # INPUT_KEYBOARD
    event.data.ki.wVk=0x0d
    event.data.ki.dwFlags=2 if up else 0
    ctypes.set_last_error(0)
    sent=user32.SendInput(1,ctypes.byref(event),ctypes.sizeof(INPUT))
    return {"sent":int(sent),"last_error":ctypes.get_last_error()}

def screenshot(user32,hwnd,path,report):
    rect=RECT()
    if not user32.GetWindowRect(hwnd,ctypes.byref(rect)):
        raise RuntimeError("GetWindowRect failed")
    x,y=user32.GetSystemMetrics(76),user32.GetSystemMetrics(77)
    w,h=user32.GetSystemMetrics(78),user32.GetSystemMetrics(79)
    box=[max(x,rect.left),max(y,rect.top),min(x+w,rect.right),min(y+h,rect.bottom)]
    report["window_rect"]=[rect.left,rect.top,rect.right,rect.bottom]
    report["virtual_screen"]=[x,y,w,h]
    if w<=0 or h<=0 or box[2]<=box[0] or box[3]<=box[1]:
        raise RuntimeError("Window does not intersect usable virtual desktop")
    im=ImageGrab.grab(bbox=tuple(box),all_screens=True).convert("RGB")
    im.save(path)
    # Sample 10K pixels only; stable, cheap, explicit near-black heuristic.
    small=im.resize((100,100))
    dark=sum(1 for r,g,b in small.getdata() if r<20 and g<20 and b<20)/10000
    report["captures"].append({"file":path.name,"box":box,"size":list(im.size),
       "sha256":hashlib.sha256(path.read_bytes()).hexdigest(),
       "dark_fraction_sampled":dark,"type":"OS desktop ImageGrab, NOT Rust framebuffer"})
    return im

def probe(exe,out):
    out.mkdir(parents=True,exist_ok=True)
    result={"scenario":"WIN-GUI-PILOT-01","build_sha":os.environ.get("GITHUB_SHA"),
      "runner":platform.platform(),"classification":"UNVERIFIED","session":{},
      "process":{},"input":{"type":"SendInput Enter down/hold/up","attempted":False},
      "captures":[],"errors":[],
      "warning":"CI-only feasibility; screenshot differences do not prove intended game input or original Flash fidelity"}
    proc=None
    try:
        if sys.platform!="win32":
            result["classification"]="INFRASTRUCTURE_NOT_WINDOWS"; return result
        user32=ctypes.WinDLL("user32",use_last_error=True)
        kernel32=ctypes.WinDLL("kernel32",use_last_error=True)
        # ctypes defaults to 32-bit int return types; HWND is pointer-sized!
        user32.GetForegroundWindow.restype=wintypes.HWND
        user32.SetForegroundWindow.argtypes=[wintypes.HWND]
        user32.ShowWindow.argtypes=[wintypes.HWND,ctypes.c_int]
        user32.GetWindowRect.argtypes=[wintypes.HWND,ctypes.POINTER(RECT)]
        user32.IsWindowVisible.argtypes=[wintypes.HWND]
        user32.GetWindowTextW.argtypes=[wintypes.HWND,wintypes.LPWSTR,ctypes.c_int]
        user32.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
        user32.SendInput.argtypes=[wintypes.UINT,ctypes.POINTER(INPUT),ctypes.c_int]
        user32.SendInput.restype=wintypes.UINT
        user32.GetWindowThreadProcessId.argtypes=[wintypes.HWND,ctypes.POINTER(wintypes.DWORD)]
        user32.GetWindowThreadProcessId.restype=wintypes.DWORD
        result["session"]["input_struct_bytes"]=ctypes.sizeof(INPUT)
        sid=wintypes.DWORD()
        if kernel32.ProcessIdToSessionId(os.getpid(),ctypes.byref(sid)):
            result["session"]["id"]=sid.value
        result["session"]["screen"]=[user32.GetSystemMetrics(0),user32.GetSystemMetrics(1)]
        if not exe.is_file():
            result["classification"]="EXECUTABLE_MISSING"; return result
        proc=subprocess.Popen([str(exe.resolve())],cwd=str(Path.cwd()),
            stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        result["process"]["pid"]=proc.pid
        found=None
        for _ in range(32):
            if proc.poll() is not None:
                result["process"]["early_exit"]=proc.returncode
                result["classification"]="APP_EXITED_BEFORE_WINDOW_UNDETERMINED"; return result
            found=find_window(user32,proc.pid)
            if found: break
            time.sleep(0.25)
        if not found:
            result["classification"]="NO_VISIBLE_GAME_WINDOW_UNDETERMINED"; return result
        hwnd,title=found
        result["process"]["window_found"]=True
        result["process"]["window_title"]=title
        user32.ShowWindow(hwnd,5)
        result["input"]["foreground_request"]=bool(user32.SetForegroundWindow(hwnd))
        time.sleep(0.7)
        result["input"]["foreground_confirmed"]=user32.GetForegroundWindow()==hwnd
        before=after=None
        try: before=screenshot(user32,hwnd,out/"before_enter.png",result)
        except Exception as e: result["errors"].append("before_capture: "+str(e))
        if not result["input"]["foreground_confirmed"]:
            result["classification"]="WINDOW_FOUND_NOT_FOREGROUND_INPUT_SKIPPED"; return result
        result["input"]["attempted"]=True
        result["input"]["keydown"]=send_enter(user32,False)
        time.sleep(0.24) # > 25 Hz game frame
        result["input"]["keyup"]=send_enter(user32,True)
        time.sleep(0.55)
        try: after=screenshot(user32,hwnd,out/"after_enter.png",result)
        except Exception as e: result["errors"].append("after_capture: "+str(e))
        if before is not None and after is not None and before.size==after.size:
            result["input"]["screen_pixels_changed"]=ImageChops.difference(before,after).getbbox() is not None
        if not result["input"]["keydown"]["sent"] or not result["input"]["keyup"]["sent"]:
            result["classification"]="OS_INPUT_API_REJECTED_OR_BLOCKED"
        elif before is None or after is None:
            result["classification"]="INPUT_API_ACCEPTED_DESKTOP_CAPTURE_UNAVAILABLE"
        elif result["input"].get("screen_pixels_changed"):
            result["classification"]="WINDOW_CAPTURED_INPUT_API_ACCEPTED_VISUAL_CHANGE_NEEDS_REVIEW"
        else:
            result["classification"]="WINDOW_CAPTURED_INPUT_API_ACCEPTED_NO_VISIBLE_CHANGE"
        return result
    except Exception as exc:
        result["errors"].append(type(exc).__name__+": "+str(exc))
        result["classification"]="PILOT_EXCEPTION_NOT_GAME_PROOF"
        return result
    finally:
        if proc is not None:
            try: proc.terminate(); proc.wait(timeout=3)
            except Exception:
                try: proc.kill(); proc.wait(timeout=3)
                except Exception as e: result["errors"].append("process_cleanup: "+str(e))
        (out/"pilot-report.json").write_text(json.dumps(result,indent=2)+"\n",encoding="utf-8")
        print(json.dumps({"gui_pilot":result["classification"],
                          "screenshots":len(result["captures"]),
                          "errors":result["errors"]}))

if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--exe",type=Path,required=True)
    p.add_argument("--out",type=Path,required=True)
    a=p.parse_args()
    probe(a.exe,a.out)
