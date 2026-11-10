#!/usr/bin/env python3
"""Batch08: test a genuinely RELOCATED standalone EXE and cross-check CI gates.

The EXE is copied under a unique LOCALAPPDATA staging directory: no installed
Python, original SWF, external asset archive or browser is used by the game.
CI Python here is QA automation only and is not shipped with the game.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from windows_icon_resource_probe import inspect_resource, inspect_shell_and_window


def json_at(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise AssertionError(f"Invalid JSON report: {path}")
    return data


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_reports(root: Path) -> dict:
    gui = json_at(root / "windows-gui-pilot/pilot-report.json")
    fs = json_at(root / "windows-fullscreen-qa/fs-qa-03-report.json")
    audio = json_at(root / "windows-intro-audio-qa/intro-audio-qa-05-report.json")
    icon = json_at(root / "windows-icon-resource-qa/windows-icon-resource-qa.json")
    bench = json_at(root / "render-benchmark.json")
    scroll = json_at(root / "scenery-scroll-benchmark.json")
    if gui.get("errors"):
        raise AssertionError(f"Existing real Win32 GUI errors: {gui['errors']}")
    if gui.get("qa_navigation", {}).get("hud_test_status") != "THREE_OS_F8_CAPTURES_WITH_CHANGED_HUD":
        raise AssertionError("F8 HUD fixture coverage missing")
    if fs.get("status") != "PASS_NATIVE_FULLSCREEN_TITLE_INTRO_RESTORE_MAXIMIZED" or fs.get("errors"):
        raise AssertionError(f"Fullscreen F/Esc gate not passed: {fs.get('status')}")
    if icon.get("pe_resource",{}).get("status") != "PASS_NATIVE_PE_RESOURCE":
        raise AssertionError("Embedded PE icon report missing")
    if icon.get("shell_and_native_window",{}).get("status") != "PASS_WINDOWS_SHELL_AND_NATIVE_GAME_ICON":
        raise AssertionError("Shell/window icon QA report missing")
    sound_state = audio.get("status")
    accepted = ("PASS_REAL_WIN32_FOCUS_MCI_PAUSE_POSITION_RESUME",
                "BLOCKED_HOSTED_RUNNER_NO_AUDIO_ENDPOINT")
    if sound_state not in accepted or audio.get("errors"):
        raise AssertionError(f"Existing audio/focus gate failed: {sound_state}")
    for data in (bench,scroll):
        if not 0 <= data.get("p95_ms",float("inf")) < 40:
            raise AssertionError("Existing <40ms rendering performance cap violated")
    return {
        "legacy_gui_errors":0, "f8_hud":"THREE_OS_F8_CAPTURES_WITH_CHANGED_HUD",
        "fullscreen":fs["status"], "source_build_pe_icon":icon["pe_resource"]["status"],
        "source_build_shell_icon":icon["shell_and_native_window"]["status"],
        "intro_audio_focus":sound_state,
        "audio_hardware_verified":sound_state=="PASS_REAL_WIN32_FOCUS_MCI_PAUSE_POSITION_RESUME",
        "render_p95_ms":bench["p95_ms"],
        "scenery_p95_ms":scroll["p95_ms"],
        "runtime_assets_in_exe":True,
    }


def installed_copy_smoke(exe: Path, root: Path) -> dict:
    local = Path(os.environ.get("LOCALAPPDATA") or tempfile.gettempdir())
    with tempfile.TemporaryDirectory(prefix="BatmanCobbleBot-QA-",dir=local) as temp:
        stage=Path(temp)
        copied=stage / "batman_cobblebot_native.exe"
        shutil.copy2(exe,copied)
        if sha256(exe) != sha256(copied):
            raise AssertionError("Installed-copy SHA256 differs from CI binary")
        pe=inspect_resource(copied)
        shell=inspect_shell_and_window(copied)
        if pe["status"]!="PASS_NATIVE_PE_RESOURCE" or shell["status"]!="PASS_WINDOWS_SHELL_AND_NATIVE_GAME_ICON":
            raise AssertionError("Installed-copy icon/resource failed")
        captured=stage/"game-hq-smoke"
        completed=subprocess.run([str(copied),"--render-smoke-hq",str(captured)],
                                 cwd=stage,timeout=120,check=False)
        if completed.returncode != 0:
            raise AssertionError(f"Relocated standalone EXE smoke exit {completed.returncode}")
        reference=root/"visual-smoke-hq"
        produced=sorted(p.name for p in captured.glob("*.png"))
        expected=sorted(p.name for p in reference.glob("*.png"))
        if len(produced)!=20 or produced!=expected:
            raise AssertionError(f"Relocated EXE screenshot list mismatch: {len(produced)}")
        mismatches=[p for p in produced if sha256(captured/p)!=sha256(reference/p)]
        if mismatches:
            raise AssertionError(f"Relocated EXE pixel hashes differ: {mismatches}")
        return {
            "sha256":sha256(copied),
            "standalone":True,
            "game_staged_under":"LOCALAPPDATA temporary QA directory",
            "original_swf_not_staged":True,
            "icon_pe":pe["status"],
            "icon_windows_shell":shell["status"],
            "hq_smoke_hashes_identical":20,
            "child_exit":completed.returncode,
        }


def main() -> None:
    parser=argparse.ArgumentParser()
    parser.add_argument("--exe",type=Path,required=True)
    parser.add_argument("--generated",type=Path,required=True)
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    result={"batch":"FINAL-INTEGRATION-08","source":"actual built native Windows Rust EXE",
            "integration_gates":check_reports(args.generated),
            "relocated_installation":installed_copy_smoke(args.exe,args.generated),
            "result":"PASS_AUTOMATED_INTEGRATION_WITH_EXPLICIT_AUDIO_LIMIT",
            "manual_acceptance_remaining":["Player verification of icon in Windows taskbar/Explorer",
                "Original-Flash fidelity review", "Audio driver position/loopback on audio-equipped Windows",
                "Multi-monitor and mixed-DPI real-hardware testing"],
            "protected_scope":"No merge, no main or Level1B changes"}
    if result["integration_gates"]["audio_hardware_verified"]:
        result["result"]="PASS_AUTOMATED_INTEGRATION_WITH_NATIVE_MCI_POSITION"
    (args.out/"batch08-integration-report.json").write_text(
        json.dumps(result,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(result,indent=2))


if __name__=="__main__":
    main()
