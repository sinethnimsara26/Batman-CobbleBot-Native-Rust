#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assets;
mod audio;
mod collision;
mod game;
mod headless;
mod render;
mod surface;
// B02 original-source metadata stays inactive until stage-transition work.
mod stage;
// B08: original Bank Street prop metadata is source-backed and inactive.
mod bank_props;
// B09: original source goon frame metadata remains unused by the live game.
mod bank_goons;
// B12: original Bank Street biker enemy metadata and AI.
mod bank_biker;
// B14: original Level 1C Penguin (`#1250`) dormant metadata and pickup layout.
mod penguin;
mod win32;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|arg| arg == "--render-smoke") {
        let out = args.get(pos + 1).map(String::as_str).unwrap_or("visual-smoke");
        headless::render_smoke(out);
        return;
    }
    if let Some(pos) = args.iter().position(|arg| arg == "--render-smoke-hq") {
        let out = args.get(pos + 1).map(String::as_str).unwrap_or("visual-smoke-hq");
        headless::render_smoke_hq(out);
        return;
    }
    if let Some(pos) = args.iter().position(|arg| arg == "--benchmark-render") {
        let iterations = args.get(pos + 1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
        let out = args.get(pos + 2).map(String::as_str).unwrap_or("render-benchmark.json");
        headless::benchmark_render(iterations,out);
        return;
    }
    if let Some(pos) = args.iter().position(|arg| arg == "--benchmark-scenery-scroll") {
        let out = args.get(pos + 1).map(String::as_str).unwrap_or("scenery-scroll-benchmark.json");
        headless::benchmark_scenery_scroll(out);
        return;
    }
    // Opt-in test/developer navigation, disabled for ordinary players.
    let qa_navigation = args.iter().any(|arg| arg == "--qa-navigation");
    win32::run(qa_navigation);
}
