//! Batches B14-B15: Level 1C Penguin (`boss` / `#1250`) source-backed metadata, pickup layout,
//! 99-frame outer timeline, 162-frame true nested child timelines, and attack FX geometry.
//!
//! Dormant metadata extracted verbatim from canonical `cobblebot.swf` (`root` frame 21,
//! `#1251` `game`, `#1250` `boss`, `#1103..#1249` child clips, and `penguinAI`). Live gameplay
//! and Level 1C switching remain gated for later batches (B16-B18).

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
/// Source `root` frame 21 ActionScript assigns `_root.stength = 5` (original spelling).
pub const PENGUIN_STRENGTH: i32 = 5;
pub const PENGUIN_ENEMY_WIDTH: f32 = 50.0;
pub const PENGUIN_ENEMY_HEIGHT: f32 = 50.0;
pub const PENGUIN_KILL_SCORE: i32 = 500;
pub const PENGUIN_ARENA_MIN_X: f32 = 0.0;
pub const PENGUIN_ARENA_MAX_X: f32 = 2300.0;

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
}
