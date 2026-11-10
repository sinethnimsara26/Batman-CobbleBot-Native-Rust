#!/usr/bin/env python3
"""CI-only, bounded REAL Win32 GUI feasibility probe. Not part of Rust game runtime."""
from __future__ import annotations
import argparse, ctypes, hashlib, json, os, platform, re, subprocess, sys, time
from ctypes import wintypes
from pathlib import Path
from PIL import ImageGrab, ImageChops

TITLE = "The Batman - CobbleBot Caper - Level 1A Fidelity Slice"
class RECT(ctypes.Structure):
    _fields_ = [("left", wintypes.LONG),("top", wintypes.LONG),("right", wintypes.LONG),("bottom", wintypes.LONG)]
class POINT(ctypes.Structure):
    _fields_ = [("x", wintypes.LONG), ("y", wintypes.LONG)]

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

def send_key(user32, key, up):
    event=INPUT()
    event.type=1 # INPUT_KEYBOARD
    event.data.ki.wVk=key
    event.data.ki.dwFlags=2 if up else 0
    ctypes.set_last_error(0)
    sent=user32.SendInput(1,ctypes.byref(event),ctypes.sizeof(INPUT))
    return {"sent":int(sent),"last_error":ctypes.get_last_error()}

def send_enter(user32,up):
    return send_key(user32,0x0d,up)

def send_mouse_button(user32, release):
    event = INPUT()
    event.type = 0  # INPUT_MOUSE
    event.data.mi.dwFlags = 0x0004 if release else 0x0002  # LEFTUP / LEFTDOWN
    ctypes.set_last_error(0)
    count = user32.SendInput(1, ctypes.byref(event), ctypes.sizeof(INPUT))
    return {"sent": int(count), "last_error": ctypes.get_last_error()}


def source_skip_click_point(user32, hwnd):
    # The same original-SWF Button #129 rectangle generated for Rust.
    source_path = Path("assets/intro_button_bounds.rs")
    original = source_path.read_text(encoding="utf-8")
    matched = re.search(r"INTRO_SKIP_RECT\s*:\s*\(i32,\s*i32,\s*i32,\s*i32\)\s*=\s*\(([^)]+)\)", original)
    if not matched:
        raise ValueError("Missing source-generated SKIP bounds: " + str(source_path))
    left, top, right, bottom = [int(x.strip()) for x in matched.group(1).split(",")]
    if not (0 <= left < right <= 600 and 0 <= top < bottom <= 400):
        raise ValueError("Invalid source-derived SKIP rectangle")
    center_x, center_y = (left + right) // 2, (top + bottom) // 2
    client = RECT()
    if not user32.GetClientRect(hwnd, ctypes.byref(client)):
        raise RuntimeError("GetClientRect failed")
    cw, ch = client.right - client.left, client.bottom - client.top
    if cw <= 0 or ch <= 0:
        raise RuntimeError("Win32 game client area is empty")
    if cw * 400 <= ch * 600:
        vw, vh = cw, cw * 400 // 600
    else:
        vw, vh = ch * 600 // 400, ch
    vx, vy = (cw - vw) // 2, (ch - vh) // 2
    point = POINT(vx + center_x * vw // 600, vy + center_y * vh // 400)
    client_coords = [point.x, point.y]
    if not user32.ClientToScreen(hwnd, ctypes.byref(point)):
        raise RuntimeError("ClientToScreen failed")
    return {
        "source_generated_file": str(source_path),
        "source_stage_bounds": [left, top, right, bottom],
        "logical_stage": [600, 400],
        "logical_click_center": [center_x, center_y],
        "win32_client_size": [cw, ch],
        "letterboxed_viewport": [vx, vy, vw, vh],
        "client_click_position": client_coords,
        "screen_click_position": [point.x, point.y],
    }


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
    dark=sum(1 for r,g,b in (small.get_flattened_data() if hasattr(small, 'get_flattened_data') else small.getdata()) if r<20 and g<20 and b<20)/10000
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
        user32.GetClientRect.argtypes=[wintypes.HWND,ctypes.POINTER(RECT)]
        user32.ClientToScreen.argtypes=[wintypes.HWND,ctypes.POINTER(POINT)]
        user32.SetCursorPos.argtypes=[ctypes.c_int,ctypes.c_int]
        user32.GetCursorPos.argtypes=[ctypes.POINTER(POINT)]
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
        proc=subprocess.Popen([str(exe.resolve()),"--qa-navigation"],cwd=str(Path.cwd()),
            stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        result["process"]["pid"]=proc.pid
        # Native startup can exceed the former 8s GUI discovery budget on
        # a cold hosted Windows VM. The separate ordinary-EXE FS-QA probe
        # successfully found the SAME app after the preceding 8s pilot timed
        # out in CI #42836412666. Do not weaken downstream F8 assertions:
        # allow a bounded 30s discovery, record actual startup latency,
        # and preserve genuine exit/no-window as explicit failures.
        started_wait=time.monotonic()
        result["process"]["window_discovery_budget_seconds"]=30.0
        found=None
        for _ in range(120):
            if proc.poll() is not None:
                result["process"]["early_exit"]=proc.returncode
                result["process"]["window_discovery_elapsed_seconds"]=round(time.monotonic()-started_wait,3)
                result["classification"]="APP_EXITED_BEFORE_WINDOW_UNDETERMINED"; return result
            found=find_window(user32,proc.pid)
            if found: break
            time.sleep(0.25)
        result["process"]["window_discovery_elapsed_seconds"]=round(time.monotonic()-started_wait,3)
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
        # WIN-GUI-PILOT-02: genuine left-button press and release inside the
        # source-derived SKIP button, followed by actual desktop HUD captures.
        # Do not confuse this OS input with a simulated Rust game-state call.
        mouse = {"scenario": "real-win32-release-inside-source-skip",
                 "attempted": False, "classification": "NOT_ATTEMPTED"}
        result["mouse_skip"] = mouse
        try:
            if not result["input"]["keydown"]["sent"] or not result["input"]["keyup"]["sent"]:
                mouse["classification"] = "KEYBOARD_PHASE_NOT_PROVEN"
            elif user32.GetForegroundWindow() != hwnd:
                mouse["classification"] = "GAME_NOT_FOREGROUND"
            else:
                geometry = source_skip_click_point(user32, hwnd)
                mouse["geometry"] = geometry
                sx, sy = geometry["screen_click_position"]
                mouse["set_cursor_pos"] = bool(user32.SetCursorPos(sx, sy))
                cursor = POINT()
                mouse["cursor_verified"] = bool(user32.GetCursorPos(ctypes.byref(cursor))) and [cursor.x, cursor.y] == [sx, sy]
                if not mouse["set_cursor_pos"] or not mouse["cursor_verified"]:
                    mouse["classification"] = "CURSOR_POSITION_NOT_VERIFIED"
                else:
                    time.sleep(0.10)
                    mouse["attempted"] = True
                    mouse["press"] = send_mouse_button(user32, False)
                    time.sleep(0.12)
                    mouse["release"] = send_mouse_button(user32, True)
                    mouse["classification"] = ("MOUSE_API_ACCEPTED_VISUAL_REVIEW_REQUIRED"
                        if mouse["press"]["sent"] == 1 and mouse["release"]["sent"] == 1
                        else "MOUSE_API_INJECTION_FAILED")
                    # Native SKIP's 41 frames last about 1.64 seconds at 25 Hz.
                    # Capture early fade, later level title and clean gameplay
                    # HUD after 100-tick title timeline; timings are approximate.
                    time.sleep(0.22)
                    screenshot(user32,hwnd,out/"skip_fade_early.png",result)
                    time.sleep(2.45)
                    screenshot(user32,hwnd,out/"skip_level_title.png",result)
                    time.sleep(3.85)
                    screenshot(user32,hwnd,out/"skip_gameplay_hud.png",result)
                    # Separately prove the opt-in native Rust developer jump,
                    # NEVER count this as proof the mouse skip itself worked.
                    qa={"mode":"--qa-navigation","shortcut":"F7 direct gameplay",
                        "classification":"NOT_TESTED"}
                    result["qa_navigation"]=qa
                    if user32.GetForegroundWindow()==hwnd:
                        qa["keydown"]=send_key(user32,0x76,False)
                        time.sleep(0.16)
                        qa["keyup"]=send_key(user32,0x76,True)
                        time.sleep(0.28)
                        reference=screenshot(user32,hwnd,out/"qa_f7_direct_gameplay.png",result)
                        qa["classification"]="CAPTURED_NEEDS_VISUAL_INSPECTION"
                        # HUD-REAL-05C: three actual F8 OS keypresses. F8 works
                        # only with the explicit opt-in --qa-navigation flag.
                        cases=[
                            ("qa_f8_batarangs20_lives10_score1234.png",[20,10,1234]),
                            ("qa_f8_batarangs99_lives12_score123456.png",[99,12,123456]),
                            ("qa_f8_batarangs20_lives99_score2147483647.png",[20,99,2147483647]),
                        ]
                        qa["hud_fixtures"]=[]
                        for filename,values in cases:
                            down=send_key(user32,0x77,False)
                            time.sleep(0.16)
                            up=send_key(user32,0x77,True)
                            time.sleep(0.30)
                            img=screenshot(user32,hwnd,out/filename,result)
                            if down["sent"]!=1 or up["sent"]!=1:
                                raise RuntimeError("QA F8 SendInput rejected for "+filename)
                            if img.size!=reference.size:
                                raise RuntimeError("QA HUD capture dimensions changed")
                            # Check only for a changed region, not digit identity.
                            # Real human/asset inspection must verify the values.
                            region=(int(img.width*0.27),int(img.height*0.04),
                                    int(img.width*0.74),int(img.height*0.23))
                            changed=ImageChops.difference(
                                reference.crop(region),img.crop(region)).getbbox() is not None
                            qa["hud_fixtures"].append({
                                "file":filename,"expected_values":values,
                                "keydown":down,"keyup":up,
                                "hud_region":list(region),
                                "changed_from_prior":changed,
                                "classification":"OS_CAPTURE_NEEDS_NUMERIC_VISUAL_REVIEW",
                            })
                            if not changed:
                                raise RuntimeError("QA F8 HUD region unchanged: "+filename)
                            reference=img
                        qa["hud_test_status"]="THREE_OS_F8_CAPTURES_WITH_CHANGED_HUD"
                    else:
                        qa["classification"]="GAME_NOT_FOREGROUND"
                    mouse["recorded_images"] = [
                        "after_enter.png", "skip_fade_early.png",
                        "skip_level_title.png", "skip_gameplay_hud.png",
                        "qa_f7_direct_gameplay.png"
                    ]
                    mouse["qualification"] = (
                        "Screenshot differences alone do not prove the source 41-frame fade. "
                        "Inspect actual game scene and HQ gameplay HUD typography; "
                        "original Flash comparison and human acceptance remain open."
                    )
        except Exception as mouse_error:
            mouse["classification"] = "MOUSE_PILOT_EXCEPTION_NOT_GAME_PROOF"
            mouse["error"] = type(mouse_error).__name__ + ": " + str(mouse_error)
            result["errors"].append("mouse_skip_phase: " + mouse["error"])
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
