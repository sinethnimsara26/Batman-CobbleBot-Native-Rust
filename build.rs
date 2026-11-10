//! Embed the source-derived Windows application icon in the native game EXE.
//! Entirely build-time: no icon decoder or external tool runs during gameplay.
//! The original SWF-derived artwork is generated before Cargo by the Windows
//! asset pipeline; missing icon is a hard error, not a silent default icon.
use std::{env, fs, path::{Path, PathBuf}, process::Command};

fn find_rc() -> PathBuf {
    if let Some(value) = env::var_os("RC") {
        return PathBuf::from(value);
    }
    // When VS Developer Command Prompt is active, rc.exe is on PATH.
    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return PathBuf::from("rc.exe");
    }
    // GitHub windows-latest has the SDK but need not expose rc.exe on PATH.
    if let Some(root) = env::var_os("ProgramFiles(x86)") {
        let bin = PathBuf::from(root).join("Windows Kits/10/bin");
        if let Ok(dirs) = fs::read_dir(bin) {
            let mut candidates = Vec::new();
            for entry in dirs.flatten() {
                let exe = entry.path().join("x64/rc.exe");
                if exe.is_file() {
                    candidates.push(exe);
                }
            }
            candidates.sort();
            if let Some(rc) = candidates.pop() {
                return rc;
            }
        }
    }
    panic!("Windows SDK rc.exe unavailable; install Windows SDK or set RC");
}

fn compile_icon(out: &Path, icon: &Path) {
    let rc_file = out.join("batman_icon.rc");
    let res_file = out.join("batman_icon.res");
    let icon_path = icon.canonicalize().expect("cannot canonicalize ICO");
    let path_for_rc = icon_path.to_string_lossy().replace('\\', "/");
    fs::write(&rc_file, format!("101 ICON \"{}\"\n", path_for_rc))
        .expect("cannot write build-time resource script");
    let result = Command::new(find_rc())
        .arg("/nologo")
        .arg(format!("/fo{}", res_file.display()))
        .arg(&rc_file)
        .output()
        .expect("failed to launch Windows resource compiler");
    if !result.status.success() || !res_file.is_file() {
        panic!("rc.exe failed ({}): {} {}",
               result.status,
               String::from_utf8_lossy(&result.stdout),
               String::from_utf8_lossy(&result.stderr));
    }
    println!("cargo:rustc-link-arg-bin=batman_cobblebot_native={}", res_file.display());
}

fn main() {
    println!("cargo:rerun-if-env-changed=RC");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        panic!("Windows icon build requires the supported MSVC target");
    }
    let icon = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join(".generated/windows-icon-art/batman-cobblebot.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    if !icon.is_file() {
        panic!("Missing source-derived Windows ICO; regenerate original assets and run build_windows_ico.py before cargo build");
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    compile_icon(&out, &icon);
}
