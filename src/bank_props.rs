//! B08: exact original root20 Bank Street game-level prop display placements.
//!
//! Source: SHA-verified canonical original FWS, zero-based root frame 20,
//! game symbol #1075, named_game_instances in level-instances.json.
//! Source-visible prop IDs #785 (4 cans), #790 (4 trash), #793 (1 mailbox).
//! All are ABOVE the game-level street sprite #783 at depth 1 and BELOW the
//! actor #691 at game-level depth 1127. Level-local ground #782 is inside
//! street #783 and is NOT the same display-list depth namespace.
//!
//! This immutable model is deliberately DORMANT. No visual asset is loaded,
//! no stage transition occurs, and no physical/scripted collision is assumed.
//! AF08: canonical source verified root20 assigns bgTouch to can_* and bgHit
//! to trash_*/mailbox_*, and their complete parent source clip lengths
//! (7/11/3), with frame scripts at first and terminal zero-based frames.
//! Their live Flash hitTest outcomes and native playback remain UNVERIFIED.
#![allow(dead_code)]
use crate::stage::{GameDepth, GamePos, LevelPos, StageId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PropKind { Can, Trash, Mailbox }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceMatrix {
    pub sx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BankStreetProp {
    pub name: &'static str,
    pub symbol_id: u16,
    /// Original GAME (#1075) display-list depth; not a level-local depth.
    pub game_depth: GameDepth,
    pub kind: PropKind,
    /// Original Flash GAME-local placement transform, not root pixels.
    pub source_game_matrix: SourceMatrix,
}

/// AF08: immutable original parent MovieClip timelines and AVM1 onEnterFrame
/// handler identities. The two action frame indices use ZERO-BASED source
/// sprite frames (distinct from zero-based ROOT scene frame 20).
/// This is source metadata only; it does not schedule native frame advance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourcePropHandler { BgTouch, BgHit }

impl SourcePropHandler {
    pub const fn original_avm1_name(self) -> &'static str {
        match self {
            Self::BgTouch => "bgTouch",
            Self::BgHit => "bgHit",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourcePropTimeline {
    pub total_frames: u8,
    pub action_frames_zero_based: [u8; 2],
    pub handler: SourcePropHandler,
}

impl PropKind {
    /// Original SHA-verified symbols #785/#790/#793 and original root20
    /// name-prefix dispatch; NOT an implementation of Flash hitTest or damage.
    pub const fn source_timeline(self) -> SourcePropTimeline {
        match self {
            Self::Can => SourcePropTimeline {
                total_frames: 7, action_frames_zero_based: [0, 6],
                handler: SourcePropHandler::BgTouch,
            },
            Self::Trash => SourcePropTimeline {
                total_frames: 11, action_frames_zero_based: [0, 10],
                handler: SourcePropHandler::BgHit,
            },
            Self::Mailbox => SourcePropTimeline {
                total_frames: 3, action_frames_zero_based: [0, 2],
                handler: SourcePropHandler::BgHit,
            },
        }
    }
}

impl BankStreetProp {
    pub fn origin_game_pos(self) -> GamePos {
        GamePos::new(self.source_game_matrix.tx, self.source_game_matrix.ty)
    }

    pub fn origin_level_pos(self) -> LevelPos {
        StageId::Level1B.descriptor().to_level_pos(self.origin_game_pos())
    }

    /// Derived solely from the known original named level placement
    /// (1034.65,239.25) in GAME-local coordinates.
    pub fn origin_level_local(self) -> (f32,f32) {
        let p = self.origin_level_pos();
        (p.x, p.y)
    }
}

/// Original frame-0 game display-list order; root frame index zero-based.
pub const BANK_STREET_PROPS: [BankStreetProp;9] = [
    BankStreetProp { name: "can_03", symbol_id: 785, game_depth: GameDepth(1077), kind: PropKind::Can, source_game_matrix: SourceMatrix { sx: 1_f32, sy: 1_f32, tx: -3391.85_f32, ty: 379.55_f32 } },
    BankStreetProp { name: "can_02", symbol_id: 785, game_depth: GameDepth(1079), kind: PropKind::Can, source_game_matrix: SourceMatrix { sx: 1_f32, sy: 1_f32, tx: -1600.8_f32, ty: 379.55_f32 } },
    BankStreetProp { name: "can_01", symbol_id: 785, game_depth: GameDepth(1081), kind: PropKind::Can, source_game_matrix: SourceMatrix { sx: 1_f32, sy: 1_f32, tx: -376.95_f32, ty: 379.55_f32 } },
    BankStreetProp { name: "can_04", symbol_id: 785, game_depth: GameDepth(1083), kind: PropKind::Can, source_game_matrix: SourceMatrix { sx: 1_f32, sy: 1_f32, tx: -5231.3_f32, ty: 379.55_f32 } },
    BankStreetProp { name: "trash_01", symbol_id: 790, game_depth: GameDepth(1085), kind: PropKind::Trash, source_game_matrix: SourceMatrix { sx: 1.1087646484375_f32, sy: 1.1087646484375_f32, tx: 243.95_f32, ty: 252.3_f32 } },
    BankStreetProp { name: "trash_02", symbol_id: 790, game_depth: GameDepth(1088), kind: PropKind::Trash, source_game_matrix: SourceMatrix { sx: 1.1087646484375_f32, sy: 1.1087646484375_f32, tx: -1184.6_f32, ty: 252.3_f32 } },
    BankStreetProp { name: "trash_03", symbol_id: 790, game_depth: GameDepth(1091), kind: PropKind::Trash, source_game_matrix: SourceMatrix { sx: 1.1087646484375_f32, sy: 1.1087646484375_f32, tx: -2726.1_f32, ty: 252.3_f32 } },
    BankStreetProp { name: "trash_04", symbol_id: 790, game_depth: GameDepth(1094), kind: PropKind::Trash, source_game_matrix: SourceMatrix { sx: 1.1087646484375_f32, sy: 1.1087646484375_f32, tx: -4677_f32, ty: 252.3_f32 } },
    BankStreetProp { name: "mailbox_01", symbol_id: 793, game_depth: GameDepth(1097), kind: PropKind::Mailbox, source_game_matrix: SourceMatrix { sx: 1.1978607177734375_f32, sy: 1.1978607177734375_f32, tx: -4155.8_f32, ty: 246.45_f32 } },
];

#[cfg(test)]
mod tests {
    use super::{BANK_STREET_PROPS,PropKind,SourcePropHandler};
    use crate::stage::StageId;

    #[test]
    fn all_nine_source_props_have_unique_names_and_original_authored_depth() {
        let mut names=std::collections::BTreeSet::new();
        let mut depths=std::collections::BTreeSet::new();
        let mut counts=[0,0,0];
        for p in BANK_STREET_PROPS {
            assert!(names.insert(p.name));
            assert!(depths.insert(p.game_depth));
            assert!(p.game_depth>1 && p.game_depth<1127);
            assert!(p.source_game_matrix.sx>0.0 && p.source_game_matrix.sy>0.0);
            match p.kind {
                PropKind::Can => { assert_eq!(p.symbol_id,785); counts[0]+=1; }
                PropKind::Trash => { assert_eq!(p.symbol_id,790); counts[1]+=1; }
                PropKind::Mailbox => { assert_eq!(p.symbol_id,793); counts[2]+=1; }
            }
        }
        assert_eq!(counts,[4,4,1]);
        assert_eq!(depths.iter().copied().map(u16::from).collect::<Vec<_>>(),
                   [1077,1079,1081,1083,1085,1088,1091,1094,1097]);
    }

    #[test]
    fn game_local_prop_matrices_convert_to_level_local_without_rounding_depth() {
        let d=StageId::Level1B.descriptor();
        assert_eq!((d.root_frame_zero_based,d.game_symbol,d.level_symbol,d.ground_symbol),
                   (20,1075,783,782));
        let can_01=BANK_STREET_PROPS.iter().find(|p|p.name=="can_01").unwrap();
        assert_eq!((can_01.source_game_matrix.tx,can_01.source_game_matrix.ty),(-376.95,379.55));
        let pos_level = can_01.origin_level_pos();
        assert_eq!(can_01.origin_level_local(), (pos_level.x, pos_level.y));
        let (x,y)=can_01.origin_level_local();
        assert!((x+1411.60).abs()<0.002 && (y-140.30).abs()<0.002);
        let mailbox=BANK_STREET_PROPS.iter().find(|p|p.name=="mailbox_01").unwrap();
        assert_eq!((mailbox.source_game_matrix.sx,mailbox.source_game_matrix.sy),
                   (1.1978607177734375,1.1978607177734375));
        assert_eq!(mailbox.game_depth,1097);
        let mb_level = mailbox.origin_level_pos();
        assert_eq!(mailbox.origin_level_local(), (mb_level.x, mb_level.y));
        let (mx,my)=mailbox.origin_level_local();
        assert!((mx+5190.45).abs()<0.002 && (my-7.20).abs()<0.002);
    }

    #[test]
    fn original_bank_prop_timelines_include_terminal_frame_actions() {
        let expected = [
            (PropKind::Can, 7, [0, 6]),
            (PropKind::Trash, 11, [0, 10]),
            (PropKind::Mailbox, 3, [0, 2]),
        ];
        for (kind, frames, actions) in expected {
            let source = kind.source_timeline();
            assert_eq!(source.total_frames, frames);
            assert_eq!(source.action_frames_zero_based, actions);
            assert!(source.action_frames_zero_based[1] < source.total_frames);
        }
        for prop in BANK_STREET_PROPS {
            let source=prop.kind.source_timeline();
            assert!(source.total_frames > 1);
            assert_eq!(source.action_frames_zero_based[0], 0);
            assert_eq!(source.action_frames_zero_based[1], source.total_frames - 1);
        }
    }

    #[test]
    fn source_root20_binds_touch_for_cans_and_hit_for_trash_and_mailbox() {
        for prop in BANK_STREET_PROPS {
            let handler = prop.kind.source_timeline().handler;
            let expected = match prop.kind {
                PropKind::Can => SourcePropHandler::BgTouch,
                PropKind::Trash | PropKind::Mailbox => SourcePropHandler::BgHit,
            };
            assert_eq!(handler, expected, "source callback changed: {}", prop.name);
            assert_eq!(handler.original_avm1_name(), match expected {
                SourcePropHandler::BgTouch => "bgTouch",
                SourcePropHandler::BgHit => "bgHit",
            });
        }
        // Source bgHit references hitTest + playerAttacking + metal1; the
        // runtime does NOT yet call it or model a native hit/damage response.
    }

    #[test]
    fn current_player_remains_level1a_and_props_are_not_game_collision() {
        let game=crate::game::Game::new();
        assert_eq!(game.active_stage(),StageId::Level1A);
        assert!(BANK_STREET_PROPS.iter().all(|p|p.symbol_id!=782 && p.symbol_id!=153));
        // There is intentionally no physics/AI/visible object path to these
        // metadata records: callbacks are now source-verified, but scheduling
        // them in a playable native Level1B is still a separate future gate.
    }
}
