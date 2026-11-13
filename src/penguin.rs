//! Batches B14-B17: Level 1C Penguin (`boss` / `#1250`) source-backed metadata, pickup layout,
//! 99-frame outer timeline, 162-frame true nested child timelines, attack FX geometry,
//! and complete `penguinAI` movement/combat state machine.
//!
//! Dormant runtime extracted verbatim from canonical `cobblebot.swf` (`root` frame 21/22,
//! `#1251` `game`, `#1250` `boss`, `#1103..#1249` child clips, `checkPlayer`/`checkBatarang`,
//! and `penguinAI`). Live stage switching & `card:escape` end sequence remain gated for B18.

#![allow(dead_code)]

use crate::collision::CollisionMask;
use crate::stage::{GamePos, StageId};

pub const PENGUIN_GAME_SYMBOL: u16 = 1251;
pub const PENGUIN_LEVEL_SYMBOL: u16 = 1095;
pub const PENGUIN_GROUND_SYMBOL: u16 = 1094;
pub const PENGUIN_BG_SYMBOL: u16 = 1078;
pub const PENGUIN_SYMBOL_ID: u16 = 1250;
pub const PENGUIN_PARENT_FRAMES: u16 = 99;
pub const PENGUIN_TOTAL_CHILD_FRAMES: u16 = 162;
pub const PENGUIN_TOTAL_BAKED_CHILD_FRAMES: u16 = PENGUIN_TOTAL_CHILD_FRAMES;

/// Common 1x raster dimensions and registration origin emitted by
/// `bake_player_states.py --symbol 1250 --scale 1 --output-scale 1`.
pub const PENGUIN_SPRITE_CANVAS_1X: (u32, u32) = (938, 410);
pub const PENGUIN_ANCHOR_IN_BITMAP_1X: (f32, f32) = (535.05466, 168.02714);
/// Exact 3x scaled canvas & anchor matching HQ3x source rasterization (`scale=6 -> output_scale=3`).
pub const PENGUIN_SPRITE_CANVAS_HQ3X: (u32, u32) = (2814, 1230);
pub const PENGUIN_ANCHOR_IN_BITMAP_HQ3X: (f32, f32) = (1605.164, 504.08142);

pub const PENGUIN_INITIAL_POS: GamePos = GamePos::new(1035.65, 316.2);
pub const PENGUIN_PLAYER_SPAWN_POS: GamePos = GamePos::new(122.05, 281.25);
pub const PENGUIN_INITIAL_SCALE_X: f32 = -0.752166748046875;
pub const PENGUIN_INITIAL_SCALE_Y: f32 = 0.7521820068359375;

pub const PENGUIN_HEALTH: i32 = 200;
pub const PENGUIN_SPEED: f32 = 6.0;
/// Source `root` Level 1C ActionScript assigns `boss.stength = 5` (original spelling typo).
pub const PENGUIN_STRENGTH: i32 = 5;
pub const PENGUIN_ENEMY_WIDTH: f32 = 50.0;
pub const PENGUIN_ENEMY_HEIGHT: f32 = 50.0;
pub const PENGUIN_KILL_SCORE: i32 = 500;
pub const PENGUIN_ARENA_MIN_X: f32 = 0.0;
pub const PENGUIN_ARENA_MAX_X: f32 = 2300.0;
pub const PENGUIN_DODGE_WALL_RIGHT_X: f32 = 2100.0;
pub const PENGUIN_DODGE_WALL_LEFT_X: f32 = 100.0;
pub const PENGUIN_DODGE_INITIAL_DY: f32 = -12.0;
pub const PENGUIN_DODGE_VX: f32 = 15.0;
pub const PENGUIN_DODGE_MAX_TICKS: u16 = 20;
pub const PENGUIN_GLOBAL_GRAVITY: f32 = 5.0;

/// Level 1C root camera & scroll setup (`_root.game.onEnterFrame = _global.cameraLogic`).
pub const PENGUIN_CAM_DELAY: f32 = 0.0;
pub const PENGUIN_CAM_HEIGHT: f32 = 270.0;
pub const PENGUIN_CAM_JUMP: f32 = 4.0;
pub const PENGUIN_CAM_MIN_X: f32 = 250.0;
pub const PENGUIN_CAM_MAX_X: f32 = 2150.0;
pub const PENGUIN_CAM_OFFSET: f32 = 0.0;
pub const PENGUIN_CAM_ZOOM: f32 = 100.0;
pub const PENGUIN_BG_SCROLL: f32 = 6.0;

pub const PENGUIN_PICKUP_SYMBOL: u16 = 695;
pub const PENGUIN_PICKUP_SCALE: f32 = 1.1306915283203125;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level1cPickupPlacement {
    pub instance_name: &'static str,
    pub symbol_id: u16,
    pub pos: GamePos,
}

pub const LEVEL1C_PICKUPS: [Level1cPickupPlacement; 4] = [
    Level1cPickupPlacement {
        instance_name: "item_br01",
        symbol_id: PENGUIN_PICKUP_SYMBOL,
        pos: GamePos::new(-647.1, 153.0),
    },
    Level1cPickupPlacement {
        instance_name: "item_br02",
        symbol_id: PENGUIN_PICKUP_SYMBOL,
        pos: GamePos::new(394.4, 201.8),
    },
    Level1cPickupPlacement {
        instance_name: "item_br03",
        symbol_id: PENGUIN_PICKUP_SYMBOL,
        pos: GamePos::new(722.65, 212.4),
    },
    Level1cPickupPlacement {
        instance_name: "item_br04",
        symbol_id: PENGUIN_PICKUP_SYMBOL,
        pos: GamePos::new(1239.65, 196.95),
    },
];

pub const LEVEL1C_PICKUP_POINTS: [(f32, f32); 4] = [
    (LEVEL1C_PICKUPS[0].pos.x, LEVEL1C_PICKUPS[0].pos.y),
    (LEVEL1C_PICKUPS[1].pos.x, LEVEL1C_PICKUPS[1].pos.y),
    (LEVEL1C_PICKUPS[2].pos.x, LEVEL1C_PICKUPS[2].pos.y),
    (LEVEL1C_PICKUPS[3].pos.x, LEVEL1C_PICKUPS[3].pos.y),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PenguinState {
    Stand,
    Walk,
    Jump,
    Dodge,
    Block,
    Attack,
    Attack2,
    Hurt,
    Fall,
    Die,
}

/// Source AVM1 terminal behavior executed at the end of a `#1250` child clip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PenguinChildAction {
    /// Child clip has no terminal stop action and loops continuously (`stand` #1103, `walk` #1117).
    Loop,
    /// Child clip executes AVM1 `Stop` on its final frame (`jump` #1135, `dodge` #1149, `die` #1249).
    Stop,
    /// Child clip sets `_parent.state = "stand"` on its final frame (`block` #1156, `attack2` #1205).
    SetParentStand,
    /// Child clip sets `_parent.state = "dodge"; _parent.dy = -18.0` on its final frame (`attack` #1172).
    TransitionToDodge { dy: f32 },
    /// Child clip runs `this.rand = random(10); if (this.rand > 3) _parent.state = "stand" else _parent.state = "attack"` (`hurt` #1207).
    RandomStandOrAttack {
        rand_modulus: u8,
        stand_if_greater_than: u8,
    },
    /// Child clip sets `_parent.fallCount = 0; _parent.dy = -20.0; _parent.state = "dodge"` on its final frame (`fall` #1224).
    RecoverToDodge { dy: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PenguinLabelSpan {
    pub state: PenguinState,
    pub label: &'static str,
    pub instance_name: Option<&'static str>,
    pub start_frame: u16,
    pub span_frames: u16,
    pub child_symbol_id: u16,
    pub child_frames: u16,
    /// Inclusive starting index inside the flat 162-frame baked child sheet (`0..162`).
    pub baked_start_frame: u16,
    /// Exclusive ending index inside the flat 162-frame baked child sheet (`0..=162`).
    pub baked_end_frame: u16,
    pub child_sx: f32,
    pub child_sy: f32,
    pub child_tx: f32,
    pub child_ty: f32,
    pub terminal_action: PenguinChildAction,
}

pub const PENGUIN_CHILD_CLIPS: [PenguinLabelSpan; 10] = [
    PenguinLabelSpan {
        state: PenguinState::Stand,
        label: "stand",
        instance_name: None,
        start_frame: 0,
        span_frames: 9,
        child_symbol_id: 1103,
        child_frames: 21,
        baked_start_frame: 0,
        baked_end_frame: 21,
        child_sx: 1.4289703,
        child_sy: 1.4289703,
        child_tx: 17.55,
        child_ty: -76.95,
        terminal_action: PenguinChildAction::Loop,
    },
    PenguinLabelSpan {
        state: PenguinState::Walk,
        label: "walk",
        instance_name: None,
        start_frame: 9,
        span_frames: 10,
        child_symbol_id: 1117,
        child_frames: 8,
        baked_start_frame: 21,
        baked_end_frame: 29,
        child_sx: 1.4557495,
        child_sy: 1.4557495,
        child_tx: 16.3,
        child_ty: -78.5,
        terminal_action: PenguinChildAction::Loop,
    },
    PenguinLabelSpan {
        state: PenguinState::Jump,
        label: "jump",
        instance_name: None,
        start_frame: 19,
        span_frames: 10,
        child_symbol_id: 1135,
        child_frames: 9,
        baked_start_frame: 29,
        baked_end_frame: 38,
        child_sx: 1.4657593,
        child_sy: 1.4657593,
        child_tx: 11.15,
        child_ty: -79.0,
        terminal_action: PenguinChildAction::Stop,
    },
    PenguinLabelSpan {
        state: PenguinState::Dodge,
        label: "dodge",
        instance_name: None,
        start_frame: 29,
        span_frames: 10,
        child_symbol_id: 1149,
        child_frames: 10,
        baked_start_frame: 38,
        baked_end_frame: 48,
        child_sx: 1.4657593,
        child_sy: 1.4657593,
        child_tx: 11.15,
        child_ty: -79.0,
        terminal_action: PenguinChildAction::Stop,
    },
    PenguinLabelSpan {
        state: PenguinState::Block,
        label: "block",
        instance_name: None,
        start_frame: 39,
        span_frames: 10,
        child_symbol_id: 1156,
        child_frames: 14,
        baked_start_frame: 48,
        baked_end_frame: 62,
        child_sx: 1.4729309,
        child_sy: 1.4729309,
        child_tx: -62.1,
        child_ty: -80.45,
        terminal_action: PenguinChildAction::SetParentStand,
    },
    PenguinLabelSpan {
        state: PenguinState::Attack,
        label: "attack",
        instance_name: None,
        start_frame: 49,
        span_frames: 10,
        child_symbol_id: 1172,
        child_frames: 10,
        baked_start_frame: 62,
        baked_end_frame: 72,
        child_sx: 1.4600067,
        child_sy: 1.4600067,
        child_tx: 15.8,
        child_ty: -78.9,
        terminal_action: PenguinChildAction::TransitionToDodge { dy: -18.0 },
    },
    PenguinLabelSpan {
        state: PenguinState::Attack2,
        label: "attack2",
        instance_name: Some("laser"),
        start_frame: 59,
        span_frames: 10,
        child_symbol_id: 1205,
        child_frames: 23,
        baked_start_frame: 72,
        baked_end_frame: 95,
        child_sx: 1.4437256,
        child_sy: 1.4437256,
        child_tx: 13.35,
        child_ty: -77.5,
        terminal_action: PenguinChildAction::SetParentStand,
    },
    PenguinLabelSpan {
        state: PenguinState::Hurt,
        label: "hurt",
        instance_name: None,
        start_frame: 69,
        span_frames: 9,
        child_symbol_id: 1207,
        child_frames: 6,
        baked_start_frame: 95,
        baked_end_frame: 101,
        child_sx: 1.4587097,
        child_sy: 1.4587097,
        child_tx: -60.05,
        child_ty: -79.5,
        terminal_action: PenguinChildAction::RandomStandOrAttack {
            rand_modulus: 10,
            stand_if_greater_than: 3,
        },
    },
    PenguinLabelSpan {
        state: PenguinState::Fall,
        label: "fall",
        instance_name: None,
        start_frame: 78,
        span_frames: 10,
        child_symbol_id: 1224,
        child_frames: 21,
        baked_start_frame: 101,
        baked_end_frame: 122,
        child_sx: 1.4470673,
        child_sy: 1.4470673,
        child_tx: -58.6,
        child_ty: -77.1,
        terminal_action: PenguinChildAction::RecoverToDodge { dy: -20.0 },
    },
    PenguinLabelSpan {
        state: PenguinState::Die,
        label: "die",
        instance_name: None,
        start_frame: 88,
        span_frames: 11,
        child_symbol_id: 1249,
        child_frames: 40,
        baked_start_frame: 122,
        baked_end_frame: 162,
        child_sx: 1.5249481,
        child_sy: 1.5249481,
        child_tx: 28.05,
        child_ty: -81.05,
        terminal_action: PenguinChildAction::Stop,
    },
];

pub const PENGUIN_LABELS: [PenguinLabelSpan; 10] = PENGUIN_CHILD_CLIPS;
pub const PENGUIN_LABEL_SPANS: [PenguinLabelSpan; 10] = PENGUIN_CHILD_CLIPS;

/// On `#1205` (`attack2` / `"laser"`) local frame 10:
/// `if (_root.playerState == "electro") { gotoAndPlay(_currentframe - 1); }`.
pub const PENGUIN_LASER_ELECTRO_HOLD_LOCAL_FRAME: u16 = 10;
pub const PENGUIN_LASER_HITBOX_SYMBOL_ID: u16 = 1192;
pub const PENGUIN_LASER_HITBOX_DEPTH: u16 = 10;
pub const PENGUIN_LASER_HITBOX_ACTIVE_LOCAL_START: u16 = 9;
pub const PENGUIN_LASER_HITBOX_ACTIVE_LOCAL_END: u16 = 10;
pub const PENGUIN_LASER_HITBOX_BAKED_FRAMES: (u16, u16) = (81, 82);

/// Source bounds & child transform of invisible `#1192` (`boss.laser.hitBox`) during `attack2`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PenguinLaserHitBoxSpec {
    pub symbol_id: u16,
    pub depth: u16,
    pub active_local_start: u16,
    pub active_local_end: u16,
    pub raw_bounds: (f32, f32, f32, f32),
    pub in_child_sx: f32,
    pub in_child_sy: f32,
    pub in_child_tx: f32,
    pub in_child_ty: f32,
    pub alpha_mult: u16,
}

pub const PENGUIN_LASER_HITBOX: PenguinLaserHitBoxSpec = PenguinLaserHitBoxSpec {
    symbol_id: PENGUIN_LASER_HITBOX_SYMBOL_ID,
    depth: PENGUIN_LASER_HITBOX_DEPTH,
    active_local_start: PENGUIN_LASER_HITBOX_ACTIVE_LOCAL_START,
    active_local_end: PENGUIN_LASER_HITBOX_ACTIVE_LOCAL_END,
    raw_bounds: (-88.995316, -37.82471, 197.39531, 37.82471),
    in_child_sx: 0.8508148,
    in_child_sy: 1.0,
    in_child_tx: -245.75,
    in_child_ty: 45.7,
    alpha_mult: 0,
};

impl PenguinLaserHitBoxSpec {
    /// Returns true on `#1205` local frames `9..=10` (removed by `RemoveObject2(depth=10)` on frame 11).
    pub const fn is_active_on_local_frame(&self, local_frame: u16) -> bool {
        local_frame >= self.active_local_start && local_frame <= self.active_local_end
    }

    /// Returns `(xmin, ymin, xmax, ymax)` in `#1205` child-local coordinate space.
    pub const fn child_local_aabb(&self) -> (f32, f32, f32, f32) {
        (
            self.raw_bounds.0 * self.in_child_sx + self.in_child_tx,
            self.raw_bounds.1 * self.in_child_sy + self.in_child_ty,
            self.raw_bounds.2 * self.in_child_sx + self.in_child_tx,
            self.raw_bounds.3 * self.in_child_sy + self.in_child_ty,
        )
    }

    /// Returns `(xmin, ymin, xmax, ymax)` in unflipped `#1250` boss-local space (facing `-x` left).
    pub const fn unflipped_boss_local_aabb(&self) -> (f32, f32, f32, f32) {
        let clip = &PENGUIN_CHILD_CLIPS[6]; // Attack2 (#1205)
        let local = self.child_local_aabb();
        (
            local.0 * clip.child_sx + clip.child_tx,
            local.1 * clip.child_sy + clip.child_ty,
            local.2 * clip.child_sx + clip.child_tx,
            local.3 * clip.child_sy + clip.child_ty,
        )
    }

    /// Returns `(xmin, ymin, xmax, ymax)` in `#1250` boss-local space oriented by `facing_sign`
    /// (`-1.0` = facing left / unflipped artwork, `+1.0` = facing right / horizontally mirrored).
    pub fn boss_local_aabb(&self, facing_sign: f32) -> (f32, f32, f32, f32) {
        let (xmin, ymin, xmax, ymax) = self.unflipped_boss_local_aabb();
        if facing_sign >= 0.0 {
            (-xmax, ymin, -xmin, ymax)
        } else {
            (xmin, ymin, xmax, ymax)
        }
    }
}

/// Source grandchild attack/deflect/flash/debris FX placement inside `#1250` child clips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PenguinNestedFxPlacement {
    pub state: PenguinState,
    pub parent_child_symbol: u16,
    pub local_frame: u16,
    pub depth: u16,
    pub fx_symbol_id: u16,
    pub instance_name: Option<&'static str>,
    pub description: &'static str,
}

pub const PENGUIN_NESTED_FX_PLACEMENTS: [PenguinNestedFxPlacement; 14] = [
    PenguinNestedFxPlacement {
        state: PenguinState::Block,
        parent_child_symbol: 1156,
        local_frame: 1,
        depth: 10,
        fx_symbol_id: 695,
        instance_name: Some("item_br03"),
        description: "Umbrella-deflected batarang FX",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Block,
        parent_child_symbol: 1156,
        local_frame: 9,
        depth: 1,
        fx_symbol_id: 1103,
        instance_name: None,
        description: "Block recovery stand pose",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 7,
        depth: 2,
        fx_symbol_id: 1184,
        instance_name: None,
        description: "Laser charge glow FX",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 7,
        depth: 4,
        fx_symbol_id: 1185,
        instance_name: None,
        description: "Laser tip charge spark",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 8,
        depth: 4,
        fx_symbol_id: 1187,
        instance_name: None,
        description: "Laser muzzle ignition flash",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 8,
        depth: 5,
        fx_symbol_id: 1189,
        instance_name: None,
        description: "Laser muzzle ring burst",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 9,
        depth: 4,
        fx_symbol_id: 1190,
        instance_name: None,
        description: "Laser beam active discharge",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Attack2,
        parent_child_symbol: 1205,
        local_frame: 9,
        depth: 10,
        fx_symbol_id: 1192,
        instance_name: Some("hitBox"),
        description: "Invisible alpha-0 laser hitBox (active frames 9..=10)",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Fall,
        parent_child_symbol: 1224,
        local_frame: 0,
        depth: 1,
        fx_symbol_id: 1103,
        instance_name: None,
        description: "White-flash cxform impact frame",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Die,
        parent_child_symbol: 1249,
        local_frame: 13,
        depth: 1,
        fx_symbol_id: 1240,
        instance_name: None,
        description: "Umbrella explosion debris piece 1",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Die,
        parent_child_symbol: 1249,
        local_frame: 13,
        depth: 3,
        fx_symbol_id: 1242,
        instance_name: None,
        description: "Umbrella explosion debris piece 2",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Die,
        parent_child_symbol: 1249,
        local_frame: 13,
        depth: 5,
        fx_symbol_id: 1244,
        instance_name: None,
        description: "Umbrella explosion debris piece 3",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Die,
        parent_child_symbol: 1249,
        local_frame: 13,
        depth: 7,
        fx_symbol_id: 1246,
        instance_name: None,
        description: "Umbrella explosion debris piece 4",
    },
    PenguinNestedFxPlacement {
        state: PenguinState::Die,
        parent_child_symbol: 1249,
        local_frame: 13,
        depth: 9,
        fx_symbol_id: 1248,
        instance_name: None,
        description: "Umbrella explosion debris piece 5",
    },
];

pub fn penguin_clip_for_state(state: PenguinState) -> &'static PenguinLabelSpan {
    PENGUIN_CHILD_CLIPS
        .iter()
        .find(|clip| clip.state == state)
        .expect("all PenguinState variants exist in PENGUIN_CHILD_CLIPS")
}

pub fn level1c_game_to_level(pos: GamePos) -> crate::stage::LevelPos {
    StageId::Level1C.descriptor().to_level_pos(pos)
}

/// Checks Level 1C ground/wall mask using original Flash AVM1 `checkWall(character, height, dx)`:
/// `level.ground.hitTest(char._x + game._x + dx, char._y + height / 2 + game._y, true)`.
pub fn check_penguin_wall(
    collision: &CollisionMask,
    x_game: f32,
    y_game: f32,
    height: f32,
    dx: f32,
) -> bool {
    let test_pos = level1c_game_to_level(GamePos::new(x_game + dx, y_game + height * 0.5));
    collision.solid_level_pos(test_pos)
}

/// Deterministic XorShift64* PRNG mirroring `GoonRng` for source `Math.random()` / `random(n)` parity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PenguinRng {
    pub state: u64,
}

impl PenguinRng {
    pub const fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        let x = self.state.wrapping_mul(0x2545F4914F6CDD1D);
        let v = (x >> 40) as u32;
        (v as f32) / 16777216.0_f32
    }
}

impl Default for PenguinRng {
    fn default() -> Self {
        Self::new(0x1337_BCB1_C0BB)
    }
}

/// Outcome returned when evaluating incoming batarang collision against Penguin (`penguinAI`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PenguinBatarangOutcome {
    None,
    Blocked,
    Hurt,
}

/// Sound effects triggered by Flash AVM1 `penguinAI`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PenguinSound {
    Punch,
    Punch2,
    Kick,
    Bam,
}

/// Batman hit reaction flavor produced by Penguin attacks (`hurt` vs `electro`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PenguinPlayerHurtKind {
    /// Close-range `attack`/`attack2` contact: sets `playerState = "hurt"` and `playerInvincible = true`.
    MeleeHurt,
    /// `laser.hitBox` (`#1192`) contact: sets `playerState = "electro"` without `playerInvincible`.
    LaserElectro,
}

/// Player snapshot passed into `PenguinBoss::tick_combat` matching root AVM1 globals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenguinPlayerContext<'a> {
    pub x_game: f32,
    pub y_game: f32,
    pub dx: f32,
    pub dir: i8,
    pub width: f32,
    pub attacking: bool,
    pub invincible: bool,
    pub state_name: &'a str,
}

/// Side-effect events emitted by `PenguinBoss::tick_combat`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PenguinCombatEvent {
    ZeroPlayerDx,
    HideBatarang { index: usize },
    Sound(PenguinSound),
    Score(i32),
    ComboHit { combo_count: u16 },
    PlayerHit { damage: i32, kind: PenguinPlayerHurtKind },
    DefeatedTrigger,
}

/// Source-faithful Level 1C Penguin boss state (`_root.game.boss`, `#1250`, `_global.penguinAI`).
#[derive(Clone, Debug, PartialEq)]
pub struct PenguinBoss {
    pub x: f32,
    pub y: f32,
    pub enemy_y: f32,
    pub dy: f32,
    pub dir: i8,
    pub speed: f32,
    pub health: i32,
    pub max_health: i32,
    /// Original root ActionScript initializes `boss.stength = 5` (missing `r`).
    pub stength: i32,
    pub enemy_width: f32,
    pub enemy_height: f32,
    pub jumping: bool,
    pub range: f32,
    pub attack_count: u8,
    pub fall_count: u16,
    pub state: PenguinState,
    pub child_frame: u16,
    pub done: bool,
    pub bammed: bool,
    pub active: bool,
}

impl Default for PenguinBoss {
    fn default() -> Self {
        Self::new()
    }
}

impl PenguinBoss {
    /// Initializes Penguin from exact Level 1C root timeline setup (`health = 200`,
    /// `speed = 6`, `stength = 5`, `dir = 1`, `state = Stand`, `enemy_y = 316.2`).
    pub fn new() -> Self {
        Self {
            x: PENGUIN_INITIAL_POS.x,
            y: PENGUIN_INITIAL_POS.y,
            enemy_y: PENGUIN_INITIAL_POS.y,
            dy: 0.0,
            dir: 1,
            speed: PENGUIN_SPEED,
            health: PENGUIN_HEALTH,
            max_health: PENGUIN_HEALTH,
            stength: PENGUIN_STRENGTH,
            enemy_width: PENGUIN_ENEMY_WIDTH,
            enemy_height: PENGUIN_ENEMY_HEIGHT,
            jumping: false,
            range: 0.0,
            attack_count: 0,
            fall_count: 0,
            state: PenguinState::Stand,
            child_frame: 0,
            done: false,
            bammed: false,
            active: true,
        }
    }

    /// Canonical melee/laser strength alias reading the source `stength` field.
    #[inline]
    pub const fn strength(&self) -> i32 {
        self.stength
    }

    #[inline]
    pub fn game_pos(&self) -> GamePos {
        GamePos::new(self.x, self.y)
    }

    #[inline]
    pub fn level_pos(&self) -> crate::stage::LevelPos {
        level1c_game_to_level(self.game_pos())
    }

    /// Current `_xscale = dir * Math.abs(_xscale)` applied every alive `penguinAI` frame.
    #[inline]
    pub fn facing_scale_x(&self) -> f32 {
        (self.dir as f32) * PENGUIN_INITIAL_SCALE_X.abs()
    }

    /// Sets boss state and rewinds `child_frame` to 0 when transitioning states.
    #[inline]
    pub fn set_state(&mut self, new_state: PenguinState) {
        if self.state != new_state {
            self.state = new_state;
            self.child_frame = 0;
        }
    }

    #[inline]
    pub fn face_x(&mut self, target_x: f32) {
        if self.x > target_x {
            self.dir = -1;
        } else if self.x < target_x {
            self.dir = 1;
        }
    }

    /// Advances nested child sprite frame according to `#1250` (`#1115..#1249`) child DoActions.
    pub fn step_child_frame(&mut self, player_state: &str, rng: &mut PenguinRng) {
        let clip = penguin_clip_for_state(self.state);
        if self.state == PenguinState::Attack2
            && self.child_frame == PENGUIN_LASER_ELECTRO_HOLD_LOCAL_FRAME
            && player_state == "electro"
        {
            self.child_frame = PENGUIN_LASER_ELECTRO_HOLD_LOCAL_FRAME.saturating_sub(1);
            return;
        }

        if self.child_frame + 1 < clip.child_frames {
            self.child_frame += 1;
            return;
        }

        match clip.terminal_action {
            PenguinChildAction::Loop => {
                self.child_frame = 0;
            }
            PenguinChildAction::Stop => {
                self.child_frame = clip.child_frames.saturating_sub(1);
            }
            PenguinChildAction::SetParentStand => {
                self.set_state(PenguinState::Stand);
            }
            PenguinChildAction::TransitionToDodge { dy }
            | PenguinChildAction::RecoverToDodge { dy } => {
                self.set_state(PenguinState::Dodge);
                self.dy = dy;
                self.fall_count = 0;
            }
            PenguinChildAction::RandomStandOrAttack {
                rand_modulus,
                stand_if_greater_than,
            } => {
                let roll = (rng.next_f32() * (rand_modulus as f32)) as u8;
                if roll > stand_if_greater_than {
                    self.set_state(PenguinState::Stand);
                } else {
                    self.set_state(PenguinState::Attack);
                    self.fall_count = 0;
                }
            }
        }
    }

    #[inline]
    fn batman_body_aabb(player_x: f32, player_y: f32, player_width: f32) -> (f32, f32, f32, f32) {
        let half_w = (player_width * 0.5).max(25.0) + 10.0;
        (player_x - half_w, player_y - 45.0, player_x + half_w, player_y + 75.0)
    }

    #[inline]
    fn aabb_overlaps(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
        a.0 <= b.2 && a.2 >= b.0 && a.1 <= b.3 && a.3 >= b.1
    }

    #[inline]
    fn point_in_aabb(px: f32, py: f32, aabb: (f32, f32, f32, f32)) -> bool {
        px >= aabb.0 && px <= aabb.2 && py >= aabb.1 && py <= aabb.3
    }

    /// World-space (`GamePos`) bounding box of Penguin's `#1192` `laser.hitBox` in current facing.
    #[inline]
    pub fn laser_game_aabb(&self) -> (f32, f32, f32, f32) {
        let local = PENGUIN_LASER_HITBOX.boss_local_aabb(self.dir as f32);
        let sx = PENGUIN_INITIAL_SCALE_X.abs();
        let sy = PENGUIN_INITIAL_SCALE_Y;
        (
            self.x + local.0 * sx,
            self.y + local.1 * sy,
            self.x + local.2 * sx,
            self.y + local.3 * sy,
        )
    }

    /// Evaluates active `#1205` `laser.hitBox` (`#1192`) overlap on local frames `9..=10`.
    #[inline]
    pub fn laser_hits_player(&self, player: &PenguinPlayerContext<'_>) -> bool {
        self.state == PenguinState::Attack2
            && PENGUIN_LASER_HITBOX.is_active_on_local_frame(self.child_frame)
            && Self::aabb_overlaps(
                self.laser_game_aabb(),
                Self::batman_body_aabb(player.x_game, player.y_game, player.width),
            )
    }

    /// Source `checkPlayer(this, dir, offset)`: tests `(this._x + playerWidth * dir + playerDx, this._y + offset)`
    /// against Batman's hit area.
    #[inline]
    pub fn check_player_point(&self, player: &PenguinPlayerContext<'_>, y_offset: f32) -> bool {
        let test_x = self.x + player.width * (self.dir as f32) + player.dx;
        let test_y = self.y + y_offset;
        Self::point_in_aabb(
            test_x,
            test_y,
            Self::batman_body_aabb(player.x_game, player.y_game, player.width),
        )
    }

    fn apply_kill_with_score(&mut self, score: i32, events: &mut Vec<PenguinCombatEvent>) {
        if self.health <= 0 {
            self.health = 0;
            self.set_state(PenguinState::Die);
            events.push(PenguinCombatEvent::Score(score));
        }
    }

    fn step_motion_and_range(
        &mut self,
        player_x: f32,
        player_dir: i8,
        collision: &CollisionMask,
    ) {
        // 1. Arena X-clamp boundary bounce (`0..2300` -> dodge back inside with dy = -12)
        if self.x > PENGUIN_ARENA_MAX_X {
            self.dir = 1;
            self.set_state(PenguinState::Dodge);
            self.dy = PENGUIN_DODGE_INITIAL_DY;
        } else if self.x < PENGUIN_ARENA_MIN_X {
            self.dir = -1;
            self.set_state(PenguinState::Dodge);
            self.dy = PENGUIN_DODGE_INITIAL_DY;
        }

        // 2. Smooth vertical settle toward `enemy_y` whenever not dodging
        if self.state != PenguinState::Dodge {
            let diff = self.y - self.enemy_y;
            if diff.abs() > 5.0 {
                self.y -= diff * 0.3;
            } else {
                self.y = self.enemy_y;
            }
        }

        // 3. Distance range calculation, facing, mid-range laser trigger, and close flee flip
        self.range = (self.x - player_x).abs();
        if self.range > 180.0 && self.range < 320.0 {
            if self.state != PenguinState::Dodge {
                self.face_x(player_x);
                if self.attack_count < 3
                    && !matches!(
                        self.state,
                        PenguinState::Attack2
                            | PenguinState::Hurt
                            | PenguinState::Fall
                            | PenguinState::Dodge
                            | PenguinState::Block
                    )
                {
                    self.set_state(PenguinState::Attack2);
                    self.attack_count += 1;
                }
            }
        } else if self.range < 180.0
            && ((self.dir == 1 && player_dir == 1 && self.x > player_x)
                || (self.dir == -1 && player_dir == -1 && self.x < player_x))
        {
            self.dir = -self.dir;
        }

        if self.range > 360.0 {
            self.face_x(player_x);
            self.attack_count = 0;
        }

        // 4. Per-state physics (`attack2` laser handled by combat branch)
        let dir_f = self.dir as f32;
        match self.state {
            PenguinState::Walk => {
                let dx = self.speed * dir_f;
                if !check_penguin_wall(collision, self.x, self.y, -50.0, dx) {
                    self.x += dx;
                }
            }
            PenguinState::Dodge => {
                if self.y <= self.enemy_y {
                    self.y += self.dy;
                    self.dy += PENGUIN_GLOBAL_GRAVITY;
                    if self.x > PENGUIN_DODGE_WALL_RIGHT_X {
                        self.dir = 1;
                        self.x -= PENGUIN_DODGE_VX * (self.dir as f32);
                    } else if self.x < PENGUIN_DODGE_WALL_LEFT_X {
                        self.dir = -1;
                        self.x -= PENGUIN_DODGE_VX * (self.dir as f32);
                    } else if !check_penguin_wall(
                        collision,
                        self.x,
                        self.y,
                        -50.0,
                        -self.speed * dir_f,
                    ) {
                        self.x -= PENGUIN_DODGE_VX * dir_f;
                    }
                } else {
                    self.dy = 0.0;
                    self.y = self.enemy_y;
                    self.set_state(PenguinState::Stand);
                }
                self.fall_count += 1;
                if self.fall_count >= PENGUIN_DODGE_MAX_TICKS {
                    self.dy = 0.0;
                    self.y = self.enemy_y;
                    self.set_state(PenguinState::Stand);
                }
            }
            PenguinState::Hurt => {
                if !check_penguin_wall(collision, self.x, self.y, -50.0, self.speed * dir_f) {
                    self.x -= 6.0 * dir_f;
                }
            }
            PenguinState::Fall => {
                if self.fall_count < 6 {
                    if !check_penguin_wall(collision, self.x, self.y, -50.0, self.speed * dir_f) {
                        self.x -= 6.0 * dir_f;
                    }
                    self.fall_count += 1;
                }
            }
            _ => {}
        }
    }

    fn maybe_enter_walk(&mut self, collision: &CollisionMask) {
        if !check_penguin_wall(collision, self.x, self.y, self.enemy_height, 0.0)
            && !matches!(
                self.state,
                PenguinState::Dodge
                    | PenguinState::Block
                    | PenguinState::Attack
                    | PenguinState::Attack2
                    | PenguinState::Hurt
                    | PenguinState::Fall
            )
        {
            self.set_state(PenguinState::Walk);
        }
    }

    fn evaluate_batarang_indexed(
        &mut self,
        player_x: f32,
        batarangs: &[(f32, f32)],
        rng: &mut PenguinRng,
    ) -> (PenguinBatarangOutcome, Option<usize>) {
        if self.health <= 0 {
            return (PenguinBatarangOutcome::None, None);
        }
        let hit_idx = batarangs.iter().position(|&(bx, by)| {
            (bx - self.x).abs() <= 25.0 && by >= self.y - 75.0 && by <= self.y + 10.0
        });
        let Some(idx) = hit_idx else {
            return (PenguinBatarangOutcome::None, None);
        };
        if matches!(
            self.state,
            PenguinState::Hurt | PenguinState::Fall | PenguinState::Block | PenguinState::Dodge
        ) {
            return (PenguinBatarangOutcome::None, None);
        }

        self.face_x(player_x);
        if rng.next_f32() * 5.0 > 2.0 {
            self.set_state(PenguinState::Block);
            (PenguinBatarangOutcome::Blocked, Some(idx))
        } else {
            (PenguinBatarangOutcome::Hurt, Some(idx))
        }
    }

    /// Evaluates batarang overlap during `penguinAI`: blocks if `rng * 5.0 > 2.0` (`60%`),
    /// otherwise returns `Hurt`.
    pub fn evaluate_batarang(
        &mut self,
        player_x: f32,
        batarangs: &[(f32, f32)],
        rng: &mut PenguinRng,
    ) -> PenguinBatarangOutcome {
        self.evaluate_batarang_indexed(player_x, batarangs, rng).0
    }

    /// Executes noncombat `penguinAI` boundary clamp, Y settle, range/facing logic,
    /// state physics (`Walk`, `Dodge`, `Hurt`, `Fall`), batarang block test, and approach `Walk`.
    pub fn tick_noncombat(
        &mut self,
        player_x: f32,
        player_dir: i8,
        player_state: &str,
        batarangs: &[(f32, f32)],
        collision: &CollisionMask,
        rng: &mut PenguinRng,
    ) -> PenguinBatarangOutcome {
        if !self.active || self.health <= 0 {
            return PenguinBatarangOutcome::None;
        }
        self.step_motion_and_range(player_x, player_dir, collision);
        let bat_outcome = self.evaluate_batarang(player_x, batarangs, rng);
        self.maybe_enter_walk(collision);
        self.step_child_frame(player_state, rng);
        bat_outcome
    }

    /// Executes full `penguinAI` tick including active `#1192` laser damage (`"electro"`),
    /// batarang `-5` HP & `+500` score, `checkPlayer(dir)` (`lowkick`/`punch`/`kick` & close attack),
    /// `checkPlayer(dir, -120)` (`-20` knockdown slam & `+100` score), approach `Walk`, and defeat trigger.
    pub fn tick_combat(
        &mut self,
        player: &PenguinPlayerContext<'_>,
        batarangs: &[(f32, f32)],
        collision: &CollisionMask,
        rng: &mut PenguinRng,
        events: &mut Vec<PenguinCombatEvent>,
    ) {
        if !self.active {
            return;
        }

        if self.health <= 0 {
            self.set_state(PenguinState::Die);
            if !self.bammed {
                self.bammed = true;
                events.push(PenguinCombatEvent::Sound(PenguinSound::Bam));
            }
            events.push(PenguinCombatEvent::DefeatedTrigger);
            self.step_child_frame(player.state_name, rng);
            return;
        }

        self.step_motion_and_range(player.x_game, player.dir, collision);

        // Active `attack2` laser hitbox (#1192) -> Batman "electro" (`done` de-bounces while overlapping)
        if self.state == PenguinState::Attack2 {
            if self.laser_hits_player(player) {
                if !self.done {
                    self.done = true;
                    if player.state_name != "capespin"
                        && player.state_name != "electro"
                        && !player.invincible
                    {
                        events.push(PenguinCombatEvent::PlayerHit {
                            damage: self.stength,
                            kind: PenguinPlayerHurtKind::LaserElectro,
                        });
                    }
                }
            } else {
                self.done = false;
            }
        }

        // Batarang check: 60% Block + hide batarang, 40% -5 HP + Hurt (+500 kill)
        let (bat_outcome, bat_idx) = self.evaluate_batarang_indexed(player.x_game, batarangs, rng);
        match bat_outcome {
            PenguinBatarangOutcome::Blocked => {
                if let Some(index) = bat_idx {
                    events.push(PenguinCombatEvent::HideBatarang { index });
                }
            }
            PenguinBatarangOutcome::Hurt => {
                self.health -= 5;
                self.set_state(PenguinState::Hurt);
                self.apply_kill_with_score(PENGUIN_KILL_SCORE, events);
            }
            PenguinBatarangOutcome::None => {}
        }

        // Combat player contact branches (`checkPlayer(dir)` / `checkPlayer(dir, -120)` / approach `Walk`)
        if self.health > 0 {
            if self.check_player_point(player, 0.0) {
                events.push(PenguinCombatEvent::ZeroPlayerDx);
                if player.attacking
                    && !matches!(
                        self.state,
                        PenguinState::Hurt | PenguinState::Fall | PenguinState::Attack
                    )
                {
                    if player.state_name == "lowkick" {
                        self.health -= 15;
                        self.set_state(PenguinState::Fall);
                        self.fall_count = 0;
                        events.push(PenguinCombatEvent::Sound(PenguinSound::Kick));
                        events.push(PenguinCombatEvent::ComboHit { combo_count: 50 });
                    } else if matches!(player.state_name, "punch" | "lowpunch" | "kick") {
                        self.health -= 10;
                        self.set_state(PenguinState::Hurt);
                        events.push(PenguinCombatEvent::Sound(PenguinSound::Punch));
                        events.push(PenguinCombatEvent::ComboHit { combo_count: 50 });
                    }
                    self.apply_kill_with_score(PENGUIN_KILL_SCORE, events);
                } else if matches!(self.state, PenguinState::Attack | PenguinState::Attack2)
                    && player.state_name != "hurt"
                    && player.state_name != "fall"
                    && !player.invincible
                {
                    events.push(PenguinCombatEvent::PlayerHit {
                        damage: self.stength,
                        kind: PenguinPlayerHurtKind::MeleeHurt,
                    });
                } else if !matches!(
                    self.state,
                    PenguinState::Dodge
                        | PenguinState::Attack
                        | PenguinState::Attack2
                        | PenguinState::Hurt
                        | PenguinState::Fall
                ) {
                    self.set_state(PenguinState::Attack);
                    self.fall_count = 0;
                }
            } else if self.check_player_point(player, -120.0) {
                events.push(PenguinCombatEvent::ZeroPlayerDx);
                if player.attacking
                    && !matches!(
                        self.state,
                        PenguinState::Hurt
                            | PenguinState::Fall
                            | PenguinState::Dodge
                            | PenguinState::Attack
                            | PenguinState::Attack2
                    )
                {
                    events.push(PenguinCombatEvent::Sound(PenguinSound::Punch2));
                    self.health -= 20;
                    self.set_state(PenguinState::Fall);
                    self.fall_count = 0;
                    // Source bytecode @127850 explicitly adds +100 (not +500) on `-120` overhead kill.
                    self.apply_kill_with_score(100, events);
                    events.push(PenguinCombatEvent::ComboHit { combo_count: 20 });
                }
            } else {
                self.maybe_enter_walk(collision);
            }
        }

        self.step_child_frame(player.state_name, rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn penguin_labels_and_child_clips_match_source_timeline() {
        let total_parent_span: u16 = PENGUIN_LABELS.iter().map(|s| s.span_frames).sum();
        let total_child_frames: u16 = PENGUIN_CHILD_CLIPS.iter().map(|s| s.child_frames).sum();
        assert_eq!(total_parent_span, PENGUIN_PARENT_FRAMES);
        assert_eq!(total_child_frames, PENGUIN_TOTAL_CHILD_FRAMES);

        let mut expected_start = 0u16;
        let mut expected_baked = 0u16;
        for span in PENGUIN_LABEL_SPANS {
            assert_eq!(span.start_frame, expected_start);
            assert_eq!(span.baked_start_frame, expected_baked);
            assert_eq!(span.baked_end_frame - span.baked_start_frame, span.child_frames);
            expected_start += span.span_frames;
            expected_baked += span.child_frames;
        }
        assert_eq!(expected_baked, PENGUIN_TOTAL_BAKED_CHILD_FRAMES);
        assert_eq!(
            penguin_clip_for_state(PenguinState::Attack2).instance_name,
            Some("laser")
        );
        assert_eq!(penguin_clip_for_state(PenguinState::Die).child_symbol_id, 1249);
    }

    #[test]
    fn penguin_child_terminal_actions_and_transforms_match_swf() {
        assert_eq!(
            penguin_clip_for_state(PenguinState::Stand).terminal_action,
            PenguinChildAction::Loop
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Walk).terminal_action,
            PenguinChildAction::Loop
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Jump).terminal_action,
            PenguinChildAction::Stop
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Dodge).terminal_action,
            PenguinChildAction::Stop
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Block).terminal_action,
            PenguinChildAction::SetParentStand
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Attack).terminal_action,
            PenguinChildAction::TransitionToDodge { dy: -18.0 }
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Attack2).terminal_action,
            PenguinChildAction::SetParentStand
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Hurt).terminal_action,
            PenguinChildAction::RandomStandOrAttack {
                rand_modulus: 10,
                stand_if_greater_than: 3,
            }
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Fall).terminal_action,
            PenguinChildAction::RecoverToDodge { dy: -20.0 }
        );
        assert_eq!(
            penguin_clip_for_state(PenguinState::Die).terminal_action,
            PenguinChildAction::Stop
        );
        assert_eq!(PENGUIN_SPRITE_CANVAS_1X, (938, 410));
        assert_eq!(PENGUIN_SPRITE_CANVAS_HQ3X, (2814, 1230));
        assert!((PENGUIN_ANCHOR_IN_BITMAP_1X.0 - 535.05466).abs() < 1e-4);
        assert!((PENGUIN_ANCHOR_IN_BITMAP_1X.1 - 168.02714).abs() < 1e-4);
    }

    #[test]
    fn penguin_laser_hitbox_bounds_and_active_window_match_source() {
        assert!(!PENGUIN_LASER_HITBOX.is_active_on_local_frame(8));
        assert!(PENGUIN_LASER_HITBOX.is_active_on_local_frame(9));
        assert!(PENGUIN_LASER_HITBOX.is_active_on_local_frame(10));
        assert!(!PENGUIN_LASER_HITBOX.is_active_on_local_frame(11));
        assert_eq!(PENGUIN_LASER_ELECTRO_HOLD_LOCAL_FRAME, 10);
        assert_eq!(PENGUIN_LASER_HITBOX_BAKED_FRAMES, (72 + 9, 72 + 10));

        let child_hb = PENGUIN_LASER_HITBOX.child_local_aabb();
        assert!((child_hb.0 - (-321.46854)).abs() < 0.01);
        assert!((child_hb.1 - 7.875289).abs() < 0.01);
        assert!((child_hb.2 - (-77.80315)).abs() < 0.01);
        assert!((child_hb.3 - 83.52471).abs() < 0.01);

        let boss_left = PENGUIN_LASER_HITBOX.boss_local_aabb(-1.0);
        assert!((boss_left.0 - (-450.76233)).abs() < 0.02);
        assert!((boss_left.1 - (-66.13024)).abs() < 0.02);
        assert!((boss_left.2 - (-98.97639)).abs() < 0.02);
        assert!((boss_left.3 - 43.08676).abs() < 0.02);

        let boss_right = PENGUIN_LASER_HITBOX.boss_local_aabb(1.0);
        assert!((boss_right.0 - 98.97639).abs() < 0.02);
        assert!((boss_right.2 - 450.76233).abs() < 0.02);
    }

    #[test]
    fn penguin_nested_fx_inventory_matches_swf_placements() {
        assert_eq!(PENGUIN_NESTED_FX_PLACEMENTS.len(), 14);
        let batarang = PENGUIN_NESTED_FX_PLACEMENTS
            .iter()
            .find(|p| p.fx_symbol_id == 695)
            .expect("block deflected batarang #695 present");
        assert_eq!(batarang.local_frame, 1);
        assert_eq!(batarang.instance_name, Some("item_br03"));

        let explosion_count = PENGUIN_NESTED_FX_PLACEMENTS
            .iter()
            .filter(|p| p.state == PenguinState::Die && p.local_frame == 13)
            .count();
        assert_eq!(explosion_count, 5);
    }

    #[test]
    fn penguin_stats_and_pickups_match_root21_source() {
        assert_eq!(PENGUIN_SYMBOL_ID, 1250);
        assert_eq!(PENGUIN_HEALTH, 200);
        assert_eq!(PENGUIN_SPEED, 6.0);
        assert_eq!(PENGUIN_STRENGTH, 5);
        assert_eq!(PENGUIN_KILL_SCORE, 500);
        assert_eq!(LEVEL1C_PICKUPS.len(), 4);
        assert_eq!(LEVEL1C_PICKUPS[0].instance_name, "item_br01");
        assert_eq!(LEVEL1C_PICKUPS[3].instance_name, "item_br04");
        assert!(PENGUIN_INITIAL_SCALE_X < 0.0 && PENGUIN_INITIAL_SCALE_Y > 0.0);
    }

    #[test]
    fn penguin_and_player_spawn_convert_cleanly_to_level1c_space() {
        let boss_lvl = level1c_game_to_level(PENGUIN_INITIAL_POS);
        let player_lvl = level1c_game_to_level(PENGUIN_PLAYER_SPAWN_POS);
        assert!((boss_lvl.x - (1035.65 - 1485.65)).abs() < 0.05);
        assert!((boss_lvl.y - (316.2 - 239.25)).abs() < 0.05);
        assert!((player_lvl.x - (122.05 - 1485.65)).abs() < 0.05);
        assert!((player_lvl.y - (281.25 - 239.25)).abs() < 0.05);
    }

    #[test]
    fn penguin_boss_init_defaults_preserve_200hp_speed6_camera_and_stength_typo() {
        let boss = PenguinBoss::new();
        assert_eq!(boss.health, 200);
        assert_eq!(boss.max_health, 200);
        assert_eq!(boss.speed, 6.0);
        assert_eq!(boss.stength, 5);
        assert_eq!(boss.strength(), 5);
        assert_eq!(boss.dir, 1);
        assert_eq!(boss.state, PenguinState::Stand);
        assert!((boss.enemy_y - 316.2).abs() < 1e-4);
        assert_eq!(boss.enemy_width, 50.0);
        assert_eq!(boss.enemy_height, 50.0);
        assert_eq!(boss.attack_count, 0);
        assert_eq!(boss.fall_count, 0);
        assert_eq!(PENGUIN_CAM_MIN_X, 250.0);
        assert_eq!(PENGUIN_CAM_MAX_X, 2150.0);
        assert_eq!(PENGUIN_CAM_HEIGHT, 270.0);
        assert_eq!(PENGUIN_CAM_JUMP, 4.0);
        assert_eq!(PENGUIN_GLOBAL_GRAVITY, 5.0);
        assert!(boss.facing_scale_x() > 0.0);
    }

    #[test]
    fn penguin_range_facing_attack2_quota_and_close_flee_flip() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0xBEEF_1C01);
        let mut boss = PenguinBoss::new();
        boss.attack_count = 3;

        // Far range (> 360) faces player and resets attack_count to 0, then approaches in Walk.
        boss.tick_noncombat(200.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, -1);
        assert_eq!(boss.attack_count, 0);
        assert_eq!(boss.state, PenguinState::Walk);
        assert!((boss.facing_scale_x() - (-PENGUIN_INITIAL_SCALE_X.abs())).abs() < 1e-5);

        // Mid-range (180 < range < 320) enters Attack2 up to attack_count < 3.
        for expected_count in 1..=3u8 {
            boss.set_state(PenguinState::Stand);
            boss.x = 1000.0;
            boss.tick_noncombat(750.0, 1, "stand", &[], &mask, &mut rng);
            assert_eq!(boss.state, PenguinState::Attack2);
            assert_eq!(boss.attack_count, expected_count);
            assert_eq!(boss.dir, -1);
        }

        // 4th entry in mid-range does NOT trigger Attack2 while attack_count == 3.
        boss.set_state(PenguinState::Stand);
        boss.x = 1000.0;
        boss.tick_noncombat(750.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.state, PenguinState::Walk);
        assert_eq!(boss.attack_count, 3);

        // Close range (< 180) with boss facing away in player's direction flips dir to face player.
        boss.set_state(PenguinState::Stand);
        boss.dir = 1;
        boss.x = 900.0;
        boss.tick_noncombat(800.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, -1);

        // Symmetric check for dir == -1, player_dir == -1, boss.x < player.x.
        boss.set_state(PenguinState::Stand);
        boss.dir = -1;
        boss.x = 700.0;
        boss.tick_noncombat(800.0, -1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, 1);
    }

    #[test]
    fn penguin_arena_boundary_bounce_and_dodge_parabolic_arc() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0xD0D6_E001);
        let mut boss = PenguinBoss::new();

        // Exceeding right arena bound (2300) triggers Dodge left (-15 * dir with dir = 1, dy = -12).
        boss.x = 2310.0;
        boss.tick_noncombat(200.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, 1);
        assert_eq!(boss.state, PenguinState::Dodge);
        assert!((boss.x - 2295.0).abs() < 1e-4);
        assert!((boss.y - (boss.enemy_y - 12.0)).abs() < 1e-4);
        assert!((boss.dy - (-7.0)).abs() < 1e-4);

        // Step parabolic arc to landing back past enemy_y -> returns to Stand (then approaches).
        for _ in 0..8 {
            if boss.state != PenguinState::Dodge {
                break;
            }
            boss.tick_noncombat(200.0, 1, "stand", &[], &mask, &mut rng);
        }
        assert!(boss.x < 2280.0);
        assert!((boss.y - boss.enemy_y).abs() < 1e-4);

        // Crossing below left arena bound (0.0) triggers Dodge right (-15 * dir with dir = -1).
        boss.set_state(PenguinState::Stand);
        boss.fall_count = 0;
        boss.x = -5.0;
        boss.tick_noncombat(1500.0, -1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, -1);
        assert_eq!(boss.state, PenguinState::Dodge);
        assert!((boss.x - 10.0).abs() < 1e-4);

        // Safety timer fall_count >= 20 terminates Dodge cleanly even if still airborne.
        boss.fall_count = 19;
        boss.y = boss.enemy_y - 40.0;
        boss.dy = -5.0;
        boss.tick_noncombat(1500.0, -1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dy, 0.0);
        assert!((boss.y - boss.enemy_y).abs() < 1e-4);
        assert_ne!(boss.state, PenguinState::Dodge);
    }

    #[test]
    fn penguin_dodge_inner_wall_guards_and_smooth_y_settle() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0x1111_2222);
        let mut boss = PenguinBoss::new();

        // Inner right wall clamp (> 2100) forces dir = 1 in Dodge.
        boss.set_state(PenguinState::Dodge);
        boss.dir = -1;
        boss.x = 2150.0;
        boss.dy = -10.0;
        boss.tick_noncombat(500.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, 1);
        assert!((boss.x - 2135.0).abs() < 1e-4);

        // Inner left wall clamp (< 100) forces dir = -1 in Dodge.
        boss.set_state(PenguinState::Dodge);
        boss.dir = 1;
        boss.fall_count = 0;
        boss.x = 80.0;
        boss.y = boss.enemy_y;
        boss.dy = -10.0;
        boss.tick_noncombat(500.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.dir, -1);
        assert!((boss.x - 95.0).abs() < 1e-4);

        // Outside Dodge, large vertical delta settles by 30% toward enemy_y per tick.
        boss.set_state(PenguinState::Stand);
        boss.x = 1000.0;
        boss.y = boss.enemy_y - 20.0;
        boss.tick_noncombat(200.0, 1, "stand", &[], &mask, &mut rng);
        assert!((boss.y - (boss.enemy_y - 14.0)).abs() < 1e-4);
    }

    #[test]
    fn penguin_batarang_rng_block_and_level1c_collision_wall_stop() {
        let mask = CollisionMask::level1c();
        let mut boss = PenguinBoss::new();

        // Search deterministic seeds exercising both sides of `rand * 5.0 > 2.0`.
        let mut saw_block = false;
        let mut saw_hurt = false;
        for seed in 1u64..64 {
            let mut trial = boss.clone();
            let mut rng = PenguinRng::new(seed);
            let outcome = trial.tick_noncombat(
                200.0,
                1,
                "stand",
                &[(trial.x + 10.0, trial.y - 40.0)],
                &mask,
                &mut rng,
            );
            match outcome {
                PenguinBatarangOutcome::Blocked => {
                    assert_eq!(trial.state, PenguinState::Block);
                    saw_block = true;
                }
                PenguinBatarangOutcome::Hurt => {
                    saw_hurt = true;
                }
                PenguinBatarangOutcome::None => {}
            }
            if saw_block && saw_hurt {
                break;
            }
        }
        assert!(saw_block && saw_hurt);

        // Open arena Walk moves at `speed = 6.0` px/tick toward player.
        let mut rng = PenguinRng::new(0xABCD_1234);
        boss.x = 1200.0;
        boss.dir = -1;
        boss.set_state(PenguinState::Walk);
        boss.tick_noncombat(400.0, 1, "stand", &[], &mask, &mut rng);
        assert_eq!(boss.state, PenguinState::Walk);
        assert!((boss.x - 1194.0).abs() < 1e-4);

        // Solid Level 1C perimeter wall checks (`checkWall`) hold outside 0..2300 while interior is open.
        assert!(!check_penguin_wall(&mask, 1200.0, boss.enemy_y, -50.0, -6.0));
        assert!(check_penguin_wall(&mask, 2557.0, boss.enemy_y, -50.0, 6.0));
        assert!(check_penguin_wall(&mask, -236.0, boss.enemy_y, -50.0, -6.0));
    }

    #[test]
    fn penguin_attack2_laser_hitbox_electro_done_and_capespin_immunity() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0x1A5E_2001);
        let mut events = Vec::new();
        let mut boss = PenguinBoss::new();
        boss.x = 1000.0;
        boss.dir = -1;
        boss.set_state(PenguinState::Attack2);
        boss.child_frame = 8; // local frame 8 -> before active window 9..=10

        let mut player = PenguinPlayerContext {
            x_game: 800.0, // inside boss -x laser (-339..-74)
            y_game: 290.0,
            dx: 0.0,
            dir: 1,
            width: 50.0,
            attacking: false,
            invincible: false,
            state_name: "stand",
        };

        // Frame 8: laser hitbox inactive -> done=false, advances to frame 9
        boss.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(events.iter().all(|e| !matches!(e, PenguinCombatEvent::PlayerHit { .. })));
        assert!(!boss.done);
        assert_eq!(boss.child_frame, 9);

        // Frame 9: laser hitbox #1192 active -> deals 5 damage + LaserElectro, sets done=true
        events.clear();
        boss.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert_eq!(
            events,
            vec![PenguinCombatEvent::PlayerHit {
                damage: 5,
                kind: PenguinPlayerHurtKind::LaserElectro,
            }]
        );
        assert!(boss.done);
        assert_eq!(boss.child_frame, 10);

        // Frame 10 while playerState == "electro": #1205 gotoAndPlay(_currentframe - 1) holds beam & done suppresses double hit
        player.state_name = "electro";
        events.clear();
        boss.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(events.is_empty());
        assert_eq!(boss.child_frame, 9);

        // Advance out of hold once player exits electro -> frame 11 deactivates hitbox & clears done
        player.state_name = "stand";
        boss.child_frame = 11;
        boss.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(!boss.done);

        // CapeSpin deflects active frame 9 beam without PlayerHit
        boss.child_frame = 9;
        player.state_name = "capespin";
        events.clear();
        boss.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(boss.done);
        assert!(events.is_empty());
    }

    #[test]
    fn penguin_batarang_block_vs_5_hp_hurt_and_500_kill_score() {
        let mask = CollisionMask::level1c();
        let mut events = Vec::new();
        let player = PenguinPlayerContext {
            x_game: 300.0,
            y_game: 290.0,
            dx: 0.0,
            dir: 1,
            width: 50.0,
            attacking: false,
            invincible: false,
            state_name: "stand",
        };

        // Find seeds triggering block vs unblocked 5 dmg hurt.
        let mut block_seed = None;
        let mut hurt_seed = None;
        for seed in 1u64..128 {
            let mut rng = PenguinRng::new(seed);
            let mut trial = PenguinBoss::new();
            trial.x = 1000.0;
            events.clear();
            trial.tick_combat(
                &player,
                &[(trial.x, trial.y - 30.0)],
                &mask,
                &mut rng,
                &mut events,
            );
            if trial.state == PenguinState::Block && block_seed.is_none() {
                assert!(events.contains(&PenguinCombatEvent::HideBatarang { index: 0 }));
                assert_eq!(trial.health, 200);
                block_seed = Some(seed);
            } else if trial.state == PenguinState::Hurt && hurt_seed.is_none() {
                assert_eq!(trial.health, 195);
                hurt_seed = Some(seed);
            }
            if block_seed.is_some() && hurt_seed.is_some() {
                break;
            }
        }
        assert!(block_seed.is_some() && hurt_seed.is_some());

        // Lethal batarang hit at 5 HP awards +500 score, enters Die, and triggers Bam once.
        let mut rng = PenguinRng::new(hurt_seed.unwrap());
        let mut dying = PenguinBoss::new();
        dying.x = 1000.0;
        dying.health = 5;
        events.clear();
        dying.tick_combat(
            &player,
            &[(dying.x, dying.y - 30.0)],
            &mask,
            &mut rng,
            &mut events,
        );
        assert_eq!(dying.health, 0);
        assert_eq!(dying.state, PenguinState::Die);
        assert!(events.contains(&PenguinCombatEvent::Score(500)));

        // Post-death tick triggers Bam once + DefeatedTrigger (`card:escape` hook for B18).
        events.clear();
        dying.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(events.contains(&PenguinCombatEvent::Sound(PenguinSound::Bam)));
        assert!(events.contains(&PenguinCombatEvent::DefeatedTrigger));
        events.clear();
        dying.tick_combat(&player, &[], &mask, &mut rng, &mut events);
        assert!(!events.contains(&PenguinCombatEvent::Sound(PenguinSound::Bam)));
    }

    #[test]
    fn penguin_melee_check_player_lowkick_punch_and_500_kill_branches() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0xC0B0_2002);
        let mut events = Vec::new();
        let mut boss = PenguinBoss::new();
        boss.x = 1000.0;
        boss.dir = -1;

        // Batman in range not attacking -> triggers Boss Attack (fall_count = 0), then MeleeHurt
        let passive_player = PenguinPlayerContext {
            x_game: 950.0, // within checkPlayer(dir=-1) around 1000 - 50 = 950
            y_game: 290.0,
            dx: 0.0,
            dir: 1,
            width: 50.0,
            attacking: false,
            invincible: false,
            state_name: "stand",
        };
        boss.tick_combat(&passive_player, &[], &mask, &mut rng, &mut events);
        assert_eq!(boss.state, PenguinState::Attack);
        assert_eq!(boss.fall_count, 0);
        assert!(events.contains(&PenguinCombatEvent::ZeroPlayerDx));

        // Next tick while Boss in Attack -> deals stength (5) MeleeHurt
        events.clear();
        boss.tick_combat(&passive_player, &[], &mask, &mut rng, &mut events);
        assert!(events.contains(&PenguinCombatEvent::PlayerHit {
            damage: 5,
            kind: PenguinPlayerHurtKind::MeleeHurt,
        }));

        // Batman lowkick from Stand -> 15 dmg, Fall, Kick SFX, combo 50
        boss.set_state(PenguinState::Stand);
        let lowkick_player = PenguinPlayerContext {
            attacking: true,
            state_name: "lowkick",
            ..passive_player
        };
        events.clear();
        boss.tick_combat(&lowkick_player, &[], &mask, &mut rng, &mut events);
        assert_eq!(boss.health, 185);
        assert_eq!(boss.state, PenguinState::Fall);
        assert!(events.contains(&PenguinCombatEvent::Sound(PenguinSound::Kick)));
        assert!(events.contains(&PenguinCombatEvent::ComboHit { combo_count: 50 }));

        // Batman punch from Stand at 10 HP -> 10 dmg lethal -> +500 score & Die
        boss.set_state(PenguinState::Stand);
        boss.health = 10;
        let punch_player = PenguinPlayerContext {
            attacking: true,
            state_name: "punch",
            ..passive_player
        };
        events.clear();
        boss.tick_combat(&punch_player, &[], &mask, &mut rng, &mut events);
        assert_eq!(boss.health, 0);
        assert_eq!(boss.state, PenguinState::Die);
        assert!(events.contains(&PenguinCombatEvent::Sound(PenguinSound::Punch)));
        assert!(events.contains(&PenguinCombatEvent::Score(500)));
    }

    #[test]
    fn penguin_overhead_check_player_minus120_slam_20_dmg_and_100_source_score() {
        let mask = CollisionMask::level1c();
        let mut rng = PenguinRng::new(0x0E12_0003);
        let mut events = Vec::new();
        let mut boss = PenguinBoss::new();
        boss.x = 1000.0;
        boss.dir = -1;
        boss.health = 40;

        // Aerial Batman at y = 200 sits above offset 0 (316.2 > 275) but hits offset -120 (196.2)
        let aerial_player = PenguinPlayerContext {
            x_game: 950.0,
            y_game: 200.0,
            dx: 0.0,
            dir: 1,
            width: 50.0,
            attacking: true,
            invincible: false,
            state_name: "jumpkick",
        };
        assert!(!boss.check_player_point(&aerial_player, 0.0));
        assert!(boss.check_player_point(&aerial_player, -120.0));

        boss.tick_combat(&aerial_player, &[], &mask, &mut rng, &mut events);
        assert_eq!(boss.health, 20);
        assert_eq!(boss.state, PenguinState::Fall);
        assert!(events.contains(&PenguinCombatEvent::Sound(PenguinSound::Punch2)));
        assert!(events.contains(&PenguinCombatEvent::ComboHit { combo_count: 20 }));

        // Lethal -120 slam from 20 HP emits exact source +100 score branch
        boss.set_state(PenguinState::Stand);
        events.clear();
        boss.tick_combat(&aerial_player, &[], &mask, &mut rng, &mut events);
        assert_eq!(boss.health, 0);
        assert_eq!(boss.state, PenguinState::Die);
        assert!(events.contains(&PenguinCombatEvent::Score(100)));
    }

    #[test]
    fn penguin_hurt_random_recovery_and_fall_recovery_to_dodge() {
        let mut boss = PenguinBoss::new();
        boss.set_state(PenguinState::Attack);
        boss.child_frame = 9;
        let mut rng = PenguinRng::new(0x5555_0001);
        boss.step_child_frame("stand", &mut rng);
        assert_eq!(boss.state, PenguinState::Dodge);
        assert_eq!(boss.dy, -18.0);

        boss.set_state(PenguinState::Fall);
        boss.child_frame = 20;
        boss.step_child_frame("stand", &mut rng);
        assert_eq!(boss.state, PenguinState::Dodge);
        assert_eq!(boss.dy, -20.0);

        let mut saw_stand = false;
        let mut saw_attack = false;
        for seed in 1u64..64 {
            let mut trial = PenguinBoss::new();
            trial.set_state(PenguinState::Hurt);
            trial.child_frame = 5;
            let mut trng = PenguinRng::new(seed);
            trial.step_child_frame("stand", &mut trng);
            if trial.state == PenguinState::Stand {
                saw_stand = true;
            } else if trial.state == PenguinState::Attack {
                saw_attack = true;
            }
            if saw_stand && saw_attack {
                break;
            }
        }
        assert!(saw_stand && saw_attack);
    }
}
