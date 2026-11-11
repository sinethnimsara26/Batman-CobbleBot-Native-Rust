//! B09 — original Bank Street source goon timeline labels and spawn metadata.
//!
//! Original SHA-checked FWS root frame 20 ZERO-BASED, game #1075:
//! #880 (four named goons), #964 (six named goons); each sprite has 89
//! SOURCE timeline frames and the same nine source labels. A parent-frame
//! label span is NOT the AVM1 animation duration/AI policy in live gameplay.
//!
//! All metadata remains dormant. No source sprites, AI, audio, attacks,
//! enemy hitboxes, 1B transition, or runtime Flash/Python are activated.
#![allow(dead_code)]
use crate::stage::StageId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceGoonLabel {
    pub name: &'static str,
    pub first_frame_zero_based: u16,
    pub end_frame_exclusive: u16,
}

/// Original source labels from the two DefineSprite 89-frame timelines.
/// The stand label has nine authored parent frames; all later spans ten.
pub const GOON_LABELS: [SourceGoonLabel; 9] = [
    SourceGoonLabel { name: "stand", first_frame_zero_based: 0, end_frame_exclusive: 9 },
    SourceGoonLabel { name: "walk", first_frame_zero_based: 9, end_frame_exclusive: 19 },
    SourceGoonLabel { name: "retreat", first_frame_zero_based: 19, end_frame_exclusive: 29 },
    SourceGoonLabel { name: "attack", first_frame_zero_based: 29, end_frame_exclusive: 39 },
    SourceGoonLabel { name: "hurt", first_frame_zero_based: 39, end_frame_exclusive: 49 },
    SourceGoonLabel { name: "fall", first_frame_zero_based: 49, end_frame_exclusive: 59 },
    SourceGoonLabel { name: "die", first_frame_zero_based: 59, end_frame_exclusive: 69 },
    SourceGoonLabel { name: "idle", first_frame_zero_based: 69, end_frame_exclusive: 79 },
    SourceGoonLabel { name: "intro", first_frame_zero_based: 79, end_frame_exclusive: 89 },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoonSourceTimeline {
    pub source_symbol_id: u16,
    pub parent_frame_count: u16,
}

pub const GOON_TIMELINES: [GoonSourceTimeline; 2] = [
    GoonSourceTimeline { source_symbol_id: 880, parent_frame_count: 89 },
    GoonSourceTimeline { source_symbol_id: 964, parent_frame_count: 89 },
];

impl GoonSourceTimeline {
    /// Timeline lookup only. The original script controls actual looping,
    /// playback rates, state selection and individual child clip timing.
    pub fn source_label(self, frame_zero_based: u16) -> Option<SourceGoonLabel> {
        if frame_zero_based >= self.parent_frame_count {
            return None;
        }
        GOON_LABELS.iter().copied().find(|label|
            label.first_frame_zero_based <= frame_zero_based
            && frame_zero_based < label.end_frame_exclusive)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankGoonSpawn {
    pub name: &'static str,
    pub symbol_id: u16,
    /// Original GAME #1075 direct depth (not ground #782 nested depth).
    pub game_depth: u16,
    /// Original game-local placement translation, NOT world camera pixels.
    pub x_game: f32,
    pub y_game: f32,
}

impl BankGoonSpawn {
    pub fn level_local(self) -> (f32, f32) {
        let level = StageId::Level1B.descriptor().level_origin_game_local;
        (self.x_game - level.0, self.y_game - level.1)
    }
}

/// Ten ORIGINAL game #1075 frame-0 goon instances in authored depth order.
pub const BANK_GOON_SPAWNS: [BankGoonSpawn; 10] = [
    BankGoonSpawn { name: "goon1", symbol_id: 880, game_depth: 1105, x_game: -1086.15_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon2", symbol_id: 964, game_depth: 1107, x_game: -746.15_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon4", symbol_id: 880, game_depth: 1109, x_game: -2074.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon3", symbol_id: 964, game_depth: 1111, x_game: -1734.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon6", symbol_id: 964, game_depth: 1113, x_game: -3466.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon5", symbol_id: 964, game_depth: 1115, x_game: -3126.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon7", symbol_id: 964, game_depth: 1117, x_game: -5364.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon9", symbol_id: 880, game_depth: 1119, x_game: -5970.75_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon8", symbol_id: 880, game_depth: 1121, x_game: -4976.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon10", symbol_id: 964, game_depth: 1123, x_game: -6110.45_f32, y_game: 398.1_f32 },
];

#[cfg(test)]
mod tests {
    use super::{BANK_GOON_SPAWNS,GOON_LABELS,GOON_TIMELINES};
    use crate::stage::StageId;

    #[test]
    fn both_original_goon_families_cover_exactly_89_source_frames() {
        assert_eq!(GOON_TIMELINES.map(|x| x.source_symbol_id), [880, 964]);
        assert_eq!(GOON_TIMELINES.map(|x| x.parent_frame_count), [89, 89]);
        assert_eq!(GOON_LABELS.map(|x|x.name),
            ["stand","walk","retreat","attack","hurt","fall","die","idle","intro"]);
        for clip in GOON_TIMELINES {
            for frame in 0..89 {
                let label=clip.source_label(frame).expect("every original source frame has label");
                assert!(label.first_frame_zero_based<=frame && frame<label.end_frame_exclusive);
            }
            assert_eq!(clip.source_label(89),None);
            assert_eq!(clip.source_label(u16::MAX),None);
            for pair in GOON_LABELS.windows(2) {
                assert_eq!(pair[0].end_frame_exclusive,pair[1].first_frame_zero_based);
            }
            assert_eq!(GOON_LABELS[0].first_frame_zero_based,0);
            assert_eq!(GOON_LABELS[8].end_frame_exclusive,89);
        }
    }

    #[test]
    fn ten_named_original_goons_preserve_both_family_ids_and_game_depths() {
        assert_eq!(BANK_GOON_SPAWNS.map(|g|g.name),
            ["goon1","goon2","goon4","goon3","goon6","goon5",
             "goon7","goon9","goon8","goon10"]);
        assert_eq!(BANK_GOON_SPAWNS.map(|g|g.game_depth),
            [1105,1107,1109,1111,1113,1115,1117,1119,1121,1123]);
        let mut seen=std::collections::BTreeSet::new();
        let mut families=[0,0];
        for g in BANK_GOON_SPAWNS {
            assert!(seen.insert(g.name));
            assert_eq!(g.y_game,398.1_f32);
            assert!(g.game_depth>1097 && g.game_depth<1127);
            match g.symbol_id {
                880=>families[0]+=1,
                964=>families[1]+=1,
                other=>panic!("unexpected original goon symbol {other}"),
            }
        }
        assert_eq!(families,[4,6]);
        assert_eq!((BANK_GOON_SPAWNS[0].x_game,BANK_GOON_SPAWNS[9].x_game),
                   (-1086.15_f32,-6110.45_f32));
    }

    #[test]
    fn source_spawn_space_is_level_relative_but_game_remains_level1a() {
        let d=StageId::Level1B.descriptor();
        assert_eq!((d.root_frame_zero_based,d.game_symbol,d.level_symbol,d.ground_symbol),
                   (20,1075,783,782));
        let p=BANK_GOON_SPAWNS[0];
        let (x,y)=p.level_local();
        assert!((x+2120.80).abs()<0.002);
        assert!((y-158.85).abs()<0.002);
        let game=crate::game::Game::new();
        assert_eq!(game.active_stage(),StageId::Level1A);
    }
}
