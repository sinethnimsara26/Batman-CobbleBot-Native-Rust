//! Diagnostic only: raster occupancy is not an exact AVM1 shape oracle.
use super::*;
use crate::assets::{SpriteFrame, batman_contact_qa_frames};
use crate::bank_goons::GoonState;

fn raster_hit(frame: &SpriteFrame, px: f32, py: f32, dir: i32, x: f32, y: f32) -> bool {
    let (w, h) = (frame.image.w as i32, frame.image.h as i32);
    let mirrored = dir < 0;
    let ax = if mirrored { w as f32 - frame.anchor_x } else { frame.anchor_x };
    let col = (x - px + ax).floor() as i32;
    let row = (y - py + frame.anchor_y).floor() as i32;
    if col < 0 || col >= w || row < 0 || row >= h { return false; }
    let src_col = if mirrored { w - 1 - col } else { col };
    frame.image.pixels[row as usize * w as usize + src_col as usize] >> 24 != 0
}

// Select an authored goon and a sampled clear corridor in the real mask.
fn fixture(stage: StageId, ground: &CollisionMask) -> (usize, f32, f32) {
    let mut g = Game::new();
    g.start_stage(stage);
    let (ox, oy) = (g.level_origin_x(), g.level_origin_y());
    for (idx, enemy) in g.bank_goons.as_ref().unwrap().goons.iter().enumerate() {
        let x = enemy.x - ox;
        for dy in -45..=45 {
            let floor = (enemy.y - oy).floor() + dy as f32;
            if (-150..=150).step_by(5).all(|dx| {
                let xx = x + dx as f32;
                ground.solid(xx, floor + 3.0)
                    && !ground.solid(xx, floor - 4.0)
                    && !ground.solid(xx, floor - 40.0)
                    && !ground.solid(xx, floor - 75.0)
                    && !ground.solid(xx, floor - 120.0)
            }) { return (idx, x, floor); }
        }
    }
    panic!("PT01: no clear authored goon corridor on {:?}", stage);
}

fn setup(stage: StageId, f: (usize, f32, f32), dir: i32,
    scenario: &str, initial: GoonState, control: bool) -> Game {
    let (idx, x, floor) = f;
    let mut g = Game::new();
    g.start_stage(stage);
    g.screen = AppScreen::Playing;
    g.player.x = x - dir as f32 * if scenario == "away" { 40.0 } else { 90.0 };
    g.player.y = floor - PLAYER_H;
    g.player.dx = 0.0;
    g.player.dy = 0.0;
    g.player.dir = if scenario == "away" { -dir } else { dir };
    g.player.state = PlayerState::Stand;
    g.player.state_tick = 0;
    g.player.jumping = false;
    g.player.on_ground = true;
    g.player.walk_count = if scenario == "run" { 12 } else { 0 };
    // Isolate body contact from damage and stun; not a playability claim.
    g.player_invincible_ticks = 200;
    g.bank_biker = None;
    g.labs_eagles.clear();
    for (i, enemy) in g.bank_goons.as_mut().unwrap().goons.iter_mut().enumerate() {
        enemy.active = !control && i == idx;
        enemy.visible = enemy.active;
        if i == idx {
            enemy.dir = -(dir as i8);
            enemy.set_state(initial);
            if initial == GoonState::Die { enemy.health = 0; }
            if scenario == "vertical_offset" { enemy.y += 420.0; }
        }
    }
    g
}

#[test]
fn pt01_held_input_contact_trace() {
    let frames = batman_contact_qa_frames();
    assert_eq!(frames.logical_pixel_scale, 1.0);
    for stage in [StageId::Level1B, StageId::Level2A] {
        let ground = CollisionMask::for_stage(stage);
        let f = fixture(stage, &ground);
        for dir in [-1, 1] {
            for (scenario, initial, ticks) in [
                ("walk", GoonState::Stand, 12),
                ("run", GoonState::Stand, 12),
                ("away", GoonState::Stand, 5),
                ("jump", GoonState::Stand, 12),
                ("vertical_offset", GoonState::Stand, 12),
                ("hurt", GoonState::Hurt, 12),
                ("fall", GoonState::Fall, 12),
                ("dead", GoonState::Die, 12),
            ] {
                let mut game = setup(stage, f, dir, scenario, initial, false);
                let mut control = setup(stage, f, dir, scenario, initial, true);
                let key = if game.player.dir < 0 { VK_LEFT } else { VK_RIGHT };
                let mut input = InputState::default();
                let mut ctrl_input = InputState::default();
                input.set(key, true);
                ctrl_input.set(key, true);
                if scenario == "jump" {
                    input.set(VK_SPACE, true);
                    ctrl_input.set(VK_SPACE, true);
                }
                let start_x = game.player.x;
                let mut crossed = false;
                let mut zero_dx = 0;
                let mut raster_hits = 0;
                for tick in 0..ticks {
                    let pre_x = game.player.x;
                    game.tick(&mut input, &ground);
                    control.tick(&mut ctrl_input, &ground);
                    input.set(VK_SPACE, false);
                    ctrl_input.set(VK_SPACE, false);
                    let enemy = &game.bank_goons.as_ref().unwrap().goons[f.0];
                    let px = game.player.x + game.level_origin_x();
                    let py = game.player.y + game.level_origin_y();
                    let frame_id = game.animation_frame();
                    let frame = &frames.frames[frame_id];
                    // Post-tick frame and matched control dx are approximations,
                    // NOT captured original/native callback state.
                    let probe_x = enemy.x + 50.0 * enemy.dir as f32 + control.player.dx;
                    let low = raster_hit(frame, px, py, game.player.dir, probe_x, enemy.y - 20.0);
                    let high = !low && raster_hit(frame, px, py, game.player.dir, probe_x, enemy.y - 120.0);
                    if low || high { raster_hits += 1; }
                    if game.player.dx == 0.0 && control.player.dx != 0.0 { zero_dx += 1; }
                    if (px - enemy.x) * dir as f32 > 0.0 { crossed = true; }
                    println!("PT01_TRACE stage={:?} scenario={} dir={} tick={} player=({:.2},{:.2}) step={:.2} dx={:.2} control_dx={:.2} control_x={:.2} frame={} enemy=({:.2},{:.2}) enemy_dir={} enemy_state={:?} low={} high={} on_ground={}",
                        stage, scenario, dir, tick, px, py, game.player.x - pre_x,
                        game.player.dx, control.player.dx, control.player.x + control.level_origin_x(),
                        frame_id, enemy.x, enemy.y, enemy.dir, enemy.state, low, high, game.player.on_ground);
                }
                println!("PT01_SUMMARY stage={:?} scenario={} dir={} fixture={:?} crossed={} zero_dx={} post_raster_hits={} travel={:.2} control_travel={:.2}",
                    stage, scenario, dir, f, crossed, zero_dx, raster_hits,
                    game.player.x - start_x, control.player.x - start_x);
                // Assert control reachability, not an invented contact rule.
                if matches!(scenario, "walk" | "run") {
                    assert!((control.player.x - f.1) * dir as f32 > 15.0,
                        "PT01 control did not cross authored goon x: {:?} {} {}", stage, scenario, dir);
                    assert!(control.fadeout_tick.is_none(), "control left fixture");
                }
            }
        }
    }
}

#[test]
fn pt01_raster_hit_respects_anchor_and_mirroring() {
    let frame = SpriteFrame {
        image: crate::assets::Image { w: 4, h: 2, pixels: vec![0, 0xff000000, 0, 0, 0, 0, 0, 0] },
        anchor_x: 1.0, anchor_y: 1.0,
    };
    assert!(raster_hit(&frame, 100.0, 200.0, 1, 100.0, 199.0));
    assert!(!raster_hit(&frame, 100.0, 200.0, 1, 99.0, 199.0));
    assert!(raster_hit(&frame, 100.0, 200.0, -1, 99.0, 199.0));
    assert!(!raster_hit(&frame, 100.0, 200.0, -1, 100.0, 199.0));
    assert!(!raster_hit(&frame, 100.0, 200.0, -1, 99.0, 200.0));
}
