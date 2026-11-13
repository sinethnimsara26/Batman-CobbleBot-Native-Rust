//! Batch B14: Level 1C Penguin (`boss` / `#1250`) source-backed metadata and pickup layout.
//!
//! Dormant metadata extracted verbatim from canonical `cobblebot.swf` (`root` frame 21,
//! `#1251` `game`, `#1250` `boss`, and `penguinAI`). Live gameplay and Level 1C switching
//! remain gated for later batches.

use crate::stage::{GamePos, StageId};

pub const PENGUIN_GAME_SYMBOL: u16 = 1251;
pub const PENGUIN_LEVEL_SYMBOL: u16 = 1095;
pub const PENGUIN_GROUND_SYMBOL: u16 = 1094;
pub const PENGUIN_BG_SYMBOL: u16 = 1078;
pub const PENGUIN_SYMBOL_ID: u16 = 1250;
pub const PENGUIN_PARENT_FRAMES: u16 = 99;
pub const PENGUIN_TOTAL_CHILD_FRAMES: u16 = 162;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PenguinLabelSpan {
    pub state: PenguinState,
    pub label: &'static str,
    pub instance_name: Option<&'static str>,
    pub start_frame: u16,
    pub span_frames: u16,
    pub child_symbol_id: u16,
    pub child_frames: u16,
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
    },
    PenguinLabelSpan {
        state: PenguinState::Walk,
        label: "walk",
        instance_name: None,
        start_frame: 9,
        span_frames: 10,
        child_symbol_id: 1117,
        child_frames: 8,
    },
    PenguinLabelSpan {
        state: PenguinState::Jump,
        label: "jump",
        instance_name: None,
        start_frame: 19,
        span_frames: 10,
        child_symbol_id: 1135,
        child_frames: 9,
    },
    PenguinLabelSpan {
        state: PenguinState::Dodge,
        label: "dodge",
        instance_name: None,
        start_frame: 29,
        span_frames: 10,
        child_symbol_id: 1149,
        child_frames: 10,
    },
    PenguinLabelSpan {
        state: PenguinState::Block,
        label: "block",
        instance_name: None,
        start_frame: 39,
        span_frames: 10,
        child_symbol_id: 1156,
        child_frames: 14,
    },
    PenguinLabelSpan {
        state: PenguinState::Attack,
        label: "attack",
        instance_name: None,
        start_frame: 49,
        span_frames: 10,
        child_symbol_id: 1172,
        child_frames: 10,
    },
    PenguinLabelSpan {
        state: PenguinState::Attack2,
        label: "attack2",
        instance_name: Some("laser"),
        start_frame: 59,
        span_frames: 10,
        child_symbol_id: 1205,
        child_frames: 23,
    },
    PenguinLabelSpan {
        state: PenguinState::Hurt,
        label: "hurt",
        instance_name: None,
        start_frame: 69,
        span_frames: 9,
        child_symbol_id: 1207,
        child_frames: 6,
    },
    PenguinLabelSpan {
        state: PenguinState::Fall,
        label: "fall",
        instance_name: None,
        start_frame: 78,
        span_frames: 10,
        child_symbol_id: 1224,
        child_frames: 21,
    },
    PenguinLabelSpan {
        state: PenguinState::Die,
        label: "die",
        instance_name: None,
        start_frame: 88,
        span_frames: 11,
        child_symbol_id: 1249,
        child_frames: 40,
    },
];

pub const PENGUIN_LABELS: [PenguinLabelSpan; 10] = PENGUIN_CHILD_CLIPS;

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
        for span in PENGUIN_LABELS {
            assert_eq!(span.start_frame, expected_start);
            expected_start += span.span_frames;
        }
        assert_eq!(
            penguin_clip_for_state(PenguinState::Attack2).instance_name,
            Some("laser")
        );
        assert_eq!(penguin_clip_for_state(PenguinState::Die).child_symbol_id, 1249);
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
