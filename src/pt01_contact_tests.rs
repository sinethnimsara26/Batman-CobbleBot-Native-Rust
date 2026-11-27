//! Diagnostic only: raster occupancy is not an exact AVM1 shape oracle.
use super::*;
use crate::assets::{SpriteFrame, SpriteSet, batman_contact_qa_frames};
use crate::bank_goons::{BankGoon, GoonState, PlayerCombatContext};
use std::cell::RefCell;

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


#[derive(Clone, Copy)]
struct Phase {
    player: PlayerCombatContext,
    frame: usize,
    dx: f32,
}
struct EnemySample {
    name: &'static str,
    x: f32,
    y: f32,
    dir: i8,
    was_active: bool,
    before_state: GoonState,
    before_health: i32,
    after_state: GoonState,
    after_health: i32,
    after_active: bool,
    blocked_before: bool,
    blocks: bool,
    blocked_after: bool,
}
#[derive(Default)]
struct Capture {
    phase: Option<Phase>,
    enemies: Vec<EnemySample>,
}
thread_local! {
    // Armed for exactly one traced Game::tick; unrelated tests and matched
    // controls never collect rows. No production data or code path is added.
    static CAPTURE: RefCell<Option<Capture>> = RefCell::new(None);
}

pub(crate) fn record_phase(player: PlayerCombatContext, frame: usize, dx: f32) {
    CAPTURE.with(|slot| {
        if let Some(capture) = slot.borrow_mut().as_mut() {
            assert!(capture.phase.is_none(), "duplicate PT01 player phase");
            capture.phase = Some(Phase { player, frame, dx });
        }
    });
}

pub(crate) fn record_enemy(
    goon: &BankGoon, was_active: bool, before_state: GoonState, before_health: i32,
    blocked_before: bool, blocks: bool, blocked_after: bool,
) {
    CAPTURE.with(|slot| {
        if let Some(capture) = slot.borrow_mut().as_mut() {
            capture.enemies.push(EnemySample {
                name: goon.name, x: goon.x, y: goon.y, dir: goon.dir,
                was_active, before_state, before_health,
                after_state: goon.state, after_health: goon.health,
                after_active: goon.active, blocked_before, blocks, blocked_after,
            });
        }
    });
}

fn traced_tick(game: &mut Game, input: &mut InputState, ground: &CollisionMask,
    frames: &SpriteSet, scenario: &str, tick: usize, dir: i32) {
    CAPTURE.with(|slot| assert!(slot.replace(Some(Capture::default())).is_none()));
    game.tick(input, ground);
    let capture = CAPTURE.with(|slot| slot.replace(None).expect("armed PT01 capture"));
    let phase = capture.phase.expect("PT01 fixture did not reach enemy callback");
    // Current fixtures isolate one authored enemy; the ordered shadow loop
    // also preserves hypothetical zero-dx propagation if more are added.
    assert_eq!(capture.enemies.len(), 1, "PT01 fixture active enemy count");
    let frame = &frames.frames[phase.frame];
    let mut shadow_dx = phase.dx;
    let mut aggregate = false;
    for (order, enemy) in capture.enemies.iter().enumerate() {
        assert_eq!(enemy.blocked_before, aggregate, "PT01 native aggregate before");
        aggregate |= enemy.blocks;
        assert_eq!(enemy.blocked_after, aggregate, "PT01 native aggregate after");
        // This is a raster candidate at the native phase, not AVM1 shape
        // equivalence. Entry health is sampled after native bat/melee batching.
        let eligible = enemy.was_active && enemy.before_health > 0;
        let incoming = shadow_dx;
        let probe_x = enemy.x + 50.0 * enemy.dir as f32 + incoming;
        let low = eligible && raster_hit(frame, phase.player.x_game, phase.player.y_game,
            phase.player.dir, probe_x, enemy.y - 20.0);
        let high = eligible && !low && raster_hit(frame, phase.player.x_game, phase.player.y_game,
            phase.player.dir, probe_x, enemy.y - 120.0);
        if low || high { shadow_dx = 0.0; }
        println!("PT01_PHASE stage={:?} scenario={} dir={} tick={} order={} enemy={} player=({:.2},{:.2}) player_dir={} phase_frame={} post_frame={} requested_dx={:.2} shadow_in={:.2} shadow_out={:.2} native_before={} native_block={} native_after={} enemy_xy=({:.2},{:.2}) enemy_dir={} state={:?}->{:?} health={}->{} active={}->{} eligible={} probe_x={:.2} low={} high={}",
            game.active_stage(), scenario, dir, tick, order, enemy.name,
            phase.player.x_game, phase.player.y_game, phase.player.dir, phase.frame,
            game.animation_frame(), phase.dx, incoming, shadow_dx,
            enemy.blocked_before, enemy.blocks, enemy.blocked_after,
            enemy.x, enemy.y, enemy.dir, enemy.before_state, enemy.after_state,
            enemy.before_health, enemy.after_health, enemy.was_active, enemy.after_active,
            eligible, probe_x, low, high);
    }
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
                    traced_tick(&mut game, &mut input, &ground, &frames, scenario, tick, dir);
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
    pt01_close_encounter_matrix(&frames);
    pt02_biker_contact_matrix(&frames);
}

// PT02: exercise the real Game enemy/event callback with controlled phase
// poses and relative offsets. This is not a held-input or original Flash run.
fn pt02_biker_contact_matrix(frames: &SpriteSet) {
    use crate::bank_biker::{BikerState, BIKER_INITIAL_Y};
    let stage = StageId::Level1B;
    let ground = CollisionMask::for_stage(stage);
    let f = fixture(stage, &ground);
    let mut cases = 0;
    for state in [PlayerState::Stand, PlayerState::Run, PlayerState::Punch] {
        for dir in [-1, 1] {
            for invincible in [false, true] {
                for gap in [-90.0_f32, -50.0, -45.0, -25.0, 0.0, 25.0, 45.0, 50.0, 90.0] {
                    let mut game = Game::new();
                    game.start_stage(stage);
                    game.screen = AppScreen::Playing;
                    game.bank_goons = None;
                    game.labs_eagles.clear();
                    game.shots.clear();
                    game.player.x = f.1;
                    game.player.y = f.2 - PLAYER_H;
                    game.player.dir = dir;
                    game.player.state = state;
                    game.player.state_tick = 0;
                    game.player.dx = if state == PlayerState::Run { 19.0 * dir as f32 } else { 0.0 };
                    game.player.dy = 0.0;
                    game.player_attacking = state == PlayerState::Punch;
                    game.player_invincible_ticks = if invincible { 35 } else { 0 };
                    let px = game.player.x + game.level_origin_x();
                    let py = game.player.y + game.level_origin_y();
                    let dx = game.player.dx;
                    let frame_id = game.animation_frame();
                    let frame = &frames.frames[frame_id];
                    let mut biker = BankBiker::new();
                    // Keep original Y, speed, facing and strength; offset X to
                    // obtain the desired post-drive gap in a real street plane.
                    biker.x = px + gap - biker.speed * biker.dir;
                    let pre_x = biker.x;
                    game.bank_biker = Some(biker);
                    let before_health = game.health;
                    let before_score = game.score;
                    game.tick_stage_enemies(&ground);
                    let biker = game.bank_biker.as_ref().expect("PT02 biker fixture");
                    let probe_x = biker.x + 50.0 * biker.dir + dx;
                    let probe_y = biker.y - 40.0;
                    let shadow_hit = raster_hit(frame, px, py, dir, probe_x, probe_y);
                    let damage = before_health - game.health;
                    let score = game.score - before_score;
                    assert!((biker.x - pre_x - biker.speed * biker.dir).abs() < 0.01,
                        "PT02 fixture must reach ordinary drive branch");
                    assert_eq!(biker.y, BIKER_INITIAL_Y);
                    assert!(damage >= 0 && damage <= biker.strength as i32);
                    assert!(score == 0 || score == 100);
                    assert!(!(damage > 0 && score > 0), "PT02 isolated callback double outcome");
                    if invincible { assert_eq!(damage, 0, "existing invincibility gate"); }
                    println!("PT02_PHASE pose={:?} dir={} invincible={} gap={} player=({:.2},{:.2}) frame={} requested_dx={:.2} result_dx={:.2} biker_pre_x={:.2} biker_post=({:.2},{:.2}) probe=({:.2},{:.2}) shadow_hit={} damage={} score={} biker_state={:?} player_state={:?}",
                        state, dir, invincible, gap, px, py, frame_id, dx, game.player.dx,
                        pre_x, biker.x, biker.y, probe_x, probe_y, shadow_hit, damage, score,
                        biker.state, game.player.state);
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 108);
    // Isolated component priority witness: both contact and swept projectile
    // eligible at the same endpoint. Observe current order, do not assert
    // that projectile-first is the desired original behavior.
    for attacking in [false, true] {
        for hurt in [false, true] {
            let mut biker = BankBiker::new();
            biker.x = -biker.speed;
            let mut events = Vec::new();
            let consumed = biker.tick_swept(0.0, 323.25, attacking, hurt,
                &[(-50.0, 50.0, 303.25)], &mut events);
            assert_eq!(biker.state, BikerState::Fall);
            assert_eq!(events.iter().filter(|e| matches!(e, BikerEvent::Score(100))).count(), 1);
            println!("PT02_PRIORITY attacking={} hurt={} consumed={:?} events={:?}",
                attacking, hurt, consumed, events);
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

// Expand only the diagnostic, not the gameplay rules. Initial close-range
// offsets and hurt/fall are controlled stress fixtures, not a human playthrough.
fn pt01_close_encounter_matrix(frames: &SpriteSet) {
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
                        let scenario = format!("close_{:?}_{}_damage{}", initial, gap, damage_enabled);
                        for tick in 0..12 {
                            let pre_x = game.player.x + game.level_origin_x();
                            let pre_y = game.player.y + game.level_origin_y();
                            let pre_ground = game.player.on_ground;
                            let pre_state = game.player.state;
                            let pre_enemy = &game.bank_goons.as_ref().unwrap().goons[f.0];
                            let (ex, ey, es, alive) = (pre_enemy.x, pre_enemy.y,
                                pre_enemy.state, pre_enemy.active && pre_enemy.health > 0);
                            let pre_health = game.health;
                            traced_tick(&mut game, &mut input, &ground, frames, &scenario, tick, dir);
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
