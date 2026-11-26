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
    pt01_close_encounter_matrix();
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

// Expand only the diagnostic, not the gameplay rules. Initial close-range
// offsets and hurt/fall are controlled stress fixtures, not a human playthrough.
fn pt01_close_encounter_matrix() {
    for stage in [StageId::Level1B, StageId::Level2A] {
        let ground = CollisionMask::for_stage(stage);
        let f = fixture(stage, &ground);
        let mut valid_controls = 0;
        let mut cases = 0;
        for dir in [-1, 1] {
            for initial in [GoonState::Stand, GoonState::Hurt, GoonState::Fall] {
                for gap in [30.0_f32, 90.0] {
                    for damage_enabled in [false, true] {
                        cases += 1;
                        let mut game = setup(stage, f, dir, "run", initial, false);
                        let mut control = setup(stage, f, dir, "run", initial, true);
                        game.player.x = f.1 - gap * dir as f32;
                        control.player.x = game.player.x;
                        if damage_enabled {
                            game.player_invincible_ticks = 0;
                            control.player_invincible_ticks = 0;
                        }
                        let key = if dir < 0 { VK_LEFT } else { VK_RIGHT };
                        let mut input = InputState::default();
                        let mut ctrl_input = InputState::default();
                        input.set(key, true);
                        ctrl_input.set(key, true);
                        let mut crossing = false;
                        let mut player_crossing = false;
                        let mut ctrl_crossing = false;
                        let mut witness = false;
                        for tick in 0..12 {
                            let pre_x = game.player.x + game.level_origin_x();
                            let pre_y = game.player.y + game.level_origin_y();
                            let pre_ground = game.player.on_ground;
                            let pre_state = game.player.state;
                            let pre_enemy = &game.bank_goons.as_ref().unwrap().goons[f.0];
                            let (ex, ey, es, alive) = (pre_enemy.x, pre_enemy.y,
                                pre_enemy.state, pre_enemy.active && pre_enemy.health > 0);
                            let pre_health = game.health;
                            game.tick(&mut input, &ground);
                            control.tick(&mut ctrl_input, &ground);
                            if game.fadeout_tick.is_some() || control.fadeout_tick.is_some() {
                                break;
                            }
                            let post_x = game.player.x + game.level_origin_x();
                            let post_y = game.player.y + game.level_origin_y();
                            let enemy = &game.bank_goons.as_ref().unwrap().goons[f.0];
                            let relative_cross = (pre_x - ex) * dir as f32 <= 0.0
                                && (post_x - enemy.x) * dir as f32 > 0.0;
                            let crossed_old_center = (pre_x - ex) * dir as f32 <= 0.0
                                && (post_x - ex) * dir as f32 > 0.0;
                            crossing |= relative_cross;
                            player_crossing |= relative_cross && crossed_old_center;
                            ctrl_crossing |= (control.player.x - f.1) * dir as f32 > 15.0;
                            if !witness && relative_cross && alive {
                                witness = true;
                                println!("PT01_CLOSE_WITNESS stage={:?} initial={:?} gap={} damage={} dir={} tick={} player_x={:.2}->{:.2} player_y={:.2}->{:.2} enemy_x={:.2}->{:.2} enemy_y={:.2}->{:.2} player_crossed_old_center={} pre_ground={} post_ground={} player_state={:?}->{:?} enemy_state={:?}->{:?} health={}->{} dx={:.2}",
                                    stage, initial, gap, damage_enabled, dir, tick,
                                    pre_x, post_x, pre_y, post_y, ex, enemy.x, ey, enemy.y,
                                    crossed_old_center, pre_ground, game.player.on_ground,
                                    pre_state, game.player.state, es, enemy.state,
                                    pre_health, game.health, game.player.dx);
                            }
                        }
                        if ctrl_crossing { valid_controls += 1; }
                        // Regression for the 16 player-driven crossings witnessed
                        // in run 45849657061. This guards the current geometric
                        // approximation, not exact Flash shape hitTest parity.
                        if gap == 30.0 && matches!(initial, GoonState::Hurt | GoonState::Fall) {
                            assert!(ctrl_crossing, "PT01 close control did not cross: {:?} {:?} dir={} damage={}", stage, initial, dir, damage_enabled);
                            assert!(!player_crossing, "PT01 living hurt/fall goon crossed: {:?} {:?} dir={} damage={}", stage, initial, dir, damage_enabled);
                        }
                        println!("PT01_CLOSE_SUMMARY stage={:?} initial={:?} gap={} damage={} dir={} control_crossed={} relative_crossed={} player_crossed_old_center={} live_witness={} health={}",
                            stage, initial, gap, damage_enabled, dir, ctrl_crossing, crossing,
                            player_crossing, witness, game.health);
                    }
                }
            }
        }
        assert_eq!(cases, 24);
        assert!(valid_controls > 0, "PT01 close matrix: no reachable control on {:?}", stage);
    }
}
