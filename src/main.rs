#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod audio;
mod collision;
mod game;
mod headless;
mod render;
mod win32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|arg| arg == "--render-smoke") {
        let out = args.get(pos + 1).map(String::as_str).unwrap_or("visual-smoke");
        headless::render_smoke(out);
        return;
    }
    win32::run();
}
