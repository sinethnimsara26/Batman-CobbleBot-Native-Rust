//! B02: immutable, source-extracted stage metadata and an INACTIVE route.
//!
//! Original source: reverse-engineering/manifests/{level-spec,
//! level-instances,level-constants-raw,routing-evidence}.json, verified on the
//! canonical FWS in Windows B01 Actions run #42981445349.
//!
//! The native game still only renders and simulates Level 1A. This module
//! intentionally does NOT load assets, move the player, execute transitions,
//! change sound playback or reinitialize root progress. B03 and B04 own those
//! separate steps. Root progress (score, lives, Batarangs) is distinct from
//! stage-local camera, health, enemies, checkpoints, wave and timer state.
//! B04 creates only a frozen pending-stage handoff after Level1A's fade;
//! it does not load Level1B, start sound #1076 or alter stage identity.
//!
//! Metadata is dormant until those later implementation gates. Suppress only
//! this module's dead-code warnings so no fake runtime usage is introduced.
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum StageId {
    Level1A = 0,
    Level1B = 1,
    Level1C = 2,
    Level2A = 3,
    Level2B = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoryCard {
    Escape,
    Tv,
    Kabuki,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceScene {
    Stage(StageId),
    Card(StoryCard),
}

/// Root-scoped progress transferred between Flash scenes; health, camera,
/// checkpoints, enemies and animation remain stage-local and are NOT copied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RootProgress {
    pub score: i32,
    pub lives: i32,
    pub batarangs: i32,
}

/// B04: immutable destination intent after the source #34 fade. This is NOT
/// permission to load a partial B03 stage pack or play its unbaked soundtrack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingStageEntry {
    pub from: StageId,
    pub to: StageId,
    pub progress: RootProgress,
    /// Source root StartSound tags; the current 1A player uses MusicStop,
    /// while the new stage cue must remain deferred until assets are complete.
    pub outgoing_root_sound_id: u16,
    pub incoming_root_sound_id: u16,
}

impl PendingStageEntry {
    pub fn after_source_fade(from: StageId, progress: RootProgress) -> Option<Self> {
        let SourceScene::Stage(to) = from.next_source_scene() else { return None; };
        Some(Self {
            from, to, progress,
            outgoing_root_sound_id: from.descriptor().root_start_sound_id,
            incoming_root_sound_id: to.descriptor().root_start_sound_id,
        })
    }
}

/// Strongly typed coordinates in GAME-local space (child of GAME sprite #1075 / #696 / etc.).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GamePos {
    pub x: f32,
    pub y: f32,
}

impl GamePos {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Strongly typed coordinates in LEVEL-local space (relative to the stage's level sprite origin).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LevelPos {
    pub x: f32,
    pub y: f32,
}

impl LevelPos {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Strongly typed display list depth in GAME sprite space.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct GameDepth(pub u16);

/// Strongly typed display list depth inside a LEVEL sprite space.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LevelDepth(pub u16);

impl From<(f32, f32)> for GamePos {
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<GamePos> for (f32, f32) {
    #[inline]
    fn from(p: GamePos) -> Self {
        (p.x, p.y)
    }
}

impl PartialEq<(f32, f32)> for GamePos {
    #[inline]
    fn eq(&self, other: &(f32, f32)) -> bool {
        self.x == other.0 && self.y == other.1
    }
}

impl PartialEq<GamePos> for (f32, f32) {
    #[inline]
    fn eq(&self, other: &GamePos) -> bool {
        self.0 == other.x && self.1 == other.y
    }
}

impl From<(f32, f32)> for LevelPos {
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<LevelPos> for (f32, f32) {
    #[inline]
    fn from(p: LevelPos) -> Self {
        (p.x, p.y)
    }
}

impl PartialEq<(f32, f32)> for LevelPos {
    #[inline]
    fn eq(&self, other: &(f32, f32)) -> bool {
        self.x == other.0 && self.y == other.1
    }
}

impl PartialEq<LevelPos> for (f32, f32) {
    #[inline]
    fn eq(&self, other: &LevelPos) -> bool {
        self.0 == other.x && self.1 == other.y
    }
}

impl From<u16> for GameDepth {
    #[inline]
    fn from(d: u16) -> Self {
        Self(d)
    }
}

impl From<GameDepth> for u16 {
    #[inline]
    fn from(d: GameDepth) -> Self {
        d.0
    }
}

impl PartialEq<u16> for GameDepth {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl PartialEq<GameDepth> for u16 {
    #[inline]
    fn eq(&self, other: &GameDepth) -> bool {
        *self == other.0
    }
}

impl PartialOrd<u16> for GameDepth {
    #[inline]
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<GameDepth> for u16 {
    #[inline]
    fn partial_cmp(&self, other: &GameDepth) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl From<u16> for LevelDepth {
    #[inline]
    fn from(d: u16) -> Self {
        Self(d)
    }
}

impl From<LevelDepth> for u16 {
    #[inline]
    fn from(d: LevelDepth) -> Self {
        d.0
    }
}

impl PartialEq<u16> for LevelDepth {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

impl PartialEq<LevelDepth> for u16 {
    #[inline]
    fn eq(&self, other: &LevelDepth) -> bool {
        *self == other.0
    }
}

impl PartialOrd<u16> for LevelDepth {
    #[inline]
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<LevelDepth> for u16 {
    #[inline]
    fn partial_cmp(&self, other: &LevelDepth) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceCamera {
    pub height: f32,
    pub jump: f32,
    pub min_x: f32,
    pub max_x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageDescriptor {
    pub id: StageId,
    /// Zero-based ROOT timeline index from the verified FWS parser.
    pub root_frame_zero_based: u8,
    pub game_symbol: u16,
    pub level_symbol: u16,
    pub ground_symbol: u16,
    /// Game-local origin of the placed "level" sprite; not screen pixels.
    pub level_origin_game_local: GamePos,
    /// Player placement is relative to the "game" sprite (NOT the level).
    pub player_spawn_game_local: GamePos,
    pub source_player_direction: i8,
    pub camera: SourceCamera,
    /// Original root StartSound ID. A tag does not establish loop semantics.
    pub root_start_sound_id: u16,
}

impl SourceCamera {
    /// Exact cameraLogic() 10%-truncated horizontal correction, applied only
    /// within the original stage's strict game-local camera bounds.
    /// Left-facing Batman aims for x=475; right-facing for x=125.
    /// Kept separate from Level1A's active renderer until Level1B is complete.
    pub fn horizontal_step(self, player_game_x:f32, camera_x:f32, facing:i8)->f32 {
        if player_game_x<=self.min_x || player_game_x>=self.max_x {
            return camera_x;
        }
        let target=if facing>0 {125.0} else {475.0};
        let delta=player_game_x+camera_x-target;
        if delta.abs()>4.0 { camera_x-(delta/10.0).trunc() } else { camera_x }
    }

    /// Original cameraLogic() jumping target (game-local player position).
    pub fn jumping_y(self, player_game_y:f32)->f32 {
        (self.height-player_game_y)/self.jump
    }

    /// Original AVM1 cameraLogic() Y update, not the separate cameraLogic2().
    ///
    /// Verified original bytecode: jumping assigns (camHeight-player._y)/
    /// camJump; grounded checks abs(game._y+player._y-150)>1, then adds
    /// ToInteger(((camHeight-player._y)/camJump-game._y)/4).
    /// The grounded guard's literal 150 is SOURCE-AUTHENTIC even for Bank
    /// Street (camHeight=125); do not substitute self.height there.
    /// Returns (new game-camera Y, camera Y delta), NOT backdrop parallax.
    /// This helper is dormant: active Level1A Game::update_camera is intact.
    pub fn vertical_step(self, player_game_y:f32, camera_y:f32, jumping:bool)->(f32,f32) {
        let new_y=if jumping {
            self.jumping_y(player_game_y)
        } else if (camera_y+player_game_y-150.0).abs()>1.0 {
            camera_y+((self.jumping_y(player_game_y)-camera_y)/4.0).trunc()
        } else {
            camera_y
        };
        (new_y,new_y-camera_y)
    }

    #[inline]
    pub fn horizontal_step_pos(self, player_pos: GamePos, camera_x: f32, facing: i8) -> f32 {
        self.horizontal_step(player_pos.x, camera_x, facing)
    }

    #[inline]
    pub fn vertical_step_pos(self, player_pos: GamePos, camera_y: f32, jumping: bool) -> (f32, f32) {
        self.vertical_step(player_pos.y, camera_y, jumping)
    }
}

impl StageDescriptor {
    /// Convert a GAME-local position into LEVEL-local space.
    #[inline]
    pub const fn to_level_pos(&self, pos: GamePos) -> LevelPos {
        LevelPos {
            x: pos.x - self.level_origin_game_local.x,
            y: pos.y - self.level_origin_game_local.y,
        }
    }

    /// Convert a LEVEL-local position into GAME-local space.
    #[inline]
    pub const fn to_game_pos(&self, pos: LevelPos) -> GamePos {
        GamePos {
            x: pos.x + self.level_origin_game_local.x,
            y: pos.y + self.level_origin_game_local.y,
        }
    }

    /// Source game-local player placement converted to level-local coordinates.
    /// Does not mutate the running game and does not apply root screen camera.
    pub fn player_spawn_level_local(&self) -> (f32, f32) {
        let p = self.to_level_pos(self.player_spawn_game_local);
        (p.x, p.y)
    }

    /// Type-safe player spawn position in level-local space.
    pub const fn player_spawn_level_pos(&self) -> LevelPos {
        self.to_level_pos(self.player_spawn_game_local)
    }
}

pub const STAGES: [StageDescriptor; 5] = [
    StageDescriptor {
        id: StageId::Level1A,
        root_frame_zero_based: 19,
        game_symbol: 696,
        level_symbol: 152,
        ground_symbol: 151,
        level_origin_game_local: GamePos { x: 2467.65, y: 239.25 },
        player_spawn_game_local: GamePos { x: 284.95, y: -661.5 },
        source_player_direction: 1,
        camera: SourceCamera { height: 150.0, jump: 1.25, min_x: -1350.0, max_x: 7650.0 },
        root_start_sound_id: 751,
    },
    StageDescriptor {
        id: StageId::Level1B,
        root_frame_zero_based: 20,
        game_symbol: 1075,
        level_symbol: 783,
        ground_symbol: 782,
        level_origin_game_local: GamePos { x: 1034.65, y: 239.25 },
        player_spawn_game_local: GamePos { x: 528.15, y: -18.75 },
        source_player_direction: -1,
        camera: SourceCamera { height: 125.0, jump: 3.0, min_x: -6220.0, max_x: 600.0 },
        root_start_sound_id: 1076,
    },
    StageDescriptor {
        id: StageId::Level1C,
        root_frame_zero_based: 21,
        game_symbol: 1251,
        level_symbol: 1095,
        ground_symbol: 1094,
        level_origin_game_local: GamePos { x: 1485.65, y: 239.25 },
        player_spawn_game_local: GamePos { x: 122.05, y: 281.25 },
        source_player_direction: 1,
        camera: SourceCamera { height: 270.0, jump: 4.0, min_x: 250.0, max_x: 2150.0 },
        root_start_sound_id: 1252,
    },
    StageDescriptor {
        id: StageId::Level2A,
        root_frame_zero_based: 23,
        game_symbol: 1402,
        level_symbol: 1317,
        ground_symbol: 1316,
        level_origin_game_local: GamePos { x: 1210.1, y: 238.25 },
        player_spawn_game_local: GamePos { x: 227.2, y: 115.4 },
        source_player_direction: -1,
        camera: SourceCamera { height: 275.0, jump: 3.0, min_x: 200.0, max_x: 55252.0 },
        root_start_sound_id: 1409,
    },
    StageDescriptor {
        id: StageId::Level2B,
        root_frame_zero_based: 25,
        game_symbol: 1496,
        level_symbol: 1485,
        ground_symbol: 1484,
        level_origin_game_local: GamePos { x: 1210.1, y: 238.25 },
        player_spawn_game_local: GamePos { x: 4413.7, y: 6854.3 },
        source_player_direction: 1,
        camera: SourceCamera { height: 275.0, jump: 3.0, min_x: 250.0, max_x: 55252.0 },
        root_start_sound_id: 1504,
    },
];

impl StageId {
    pub const ALL: [Self; 5] = [
        Self::Level1A, Self::Level1B, Self::Level1C, Self::Level2A, Self::Level2B
    ];

    /// Strings used by the original gotoNext/gotoLast ActionScript routes.
    pub const fn source_label(self) -> &'static str {
        match self {
            Self::Level1A => "level1a",
            Self::Level1B => "level1b",
            Self::Level1C => "level1c",
            Self::Level2A => "level2a",
            Self::Level2B => "level2b",
        }
    }

    /// Source-extraction manifest uses level1c_penguin, unlike AVM1 level1c.
    pub const fn manifest_key(self) -> &'static str {
        match self {
            Self::Level1C => "level1c_penguin",
            _ => self.source_label(),
        }
    }

    pub fn from_source_label(label: &str) -> Option<Self> {
        match label {
            "level1a" => Some(Self::Level1A),
            "level1b" => Some(Self::Level1B),
            "level1c" => Some(Self::Level1C),
            "level2a" => Some(Self::Level2A),
            "level2b" => Some(Self::Level2B),
            _ => None,
        }
    }

    pub const fn from_source_root_frame(frame: u8) -> Option<Self> {
        match frame {
            19 => Some(Self::Level1A),
            20 => Some(Self::Level1B),
            21 => Some(Self::Level1C),
            23 => Some(Self::Level2A),
            25 => Some(Self::Level2B),
            _ => None, // 22/24/26 are story card frames, not gameplay.
        }
    }

    pub const fn descriptor(self) -> &'static StageDescriptor {
        &STAGES[self as usize]
    }

    /// Declarative ORIGINAL Flash progression, deliberately never invoked
    /// by Game::tick in B02. Completing a stage must not prematurely jump.
    pub const fn next_source_scene(self) -> SourceScene {
        match self {
            Self::Level1A => SourceScene::Stage(Self::Level1B),
            Self::Level1B => SourceScene::Stage(Self::Level1C),
            Self::Level1C => SourceScene::Card(StoryCard::Escape),
            Self::Level2A => SourceScene::Card(StoryCard::Tv),
            Self::Level2B => SourceScene::Card(StoryCard::Kabuki),
        }
    }
}

impl StoryCard {
    pub const fn root_frame_zero_based(self) -> u8 {
        match self {
            Self::Escape => 22,
            Self::Tv => 24,
            Self::Kabuki => 26,
        }
    }

    pub const fn sprite_id(self) -> u16 {
        match self {
            Self::Escape => 1267,
            Self::Tv => 1479,
            Self::Kabuki => 1569,
        }
    }

    pub const fn source_label(self) -> &'static str {
        match self {
            Self::Escape => "card:escape",
            Self::Tv => "card:tv",
            Self::Kabuki => "card:kabuki",
        }
    }

    /// Kabuki's original destination is Level3A, outside B02's stage set.
    pub const fn next_in_scope(self) -> Option<StageId> {
        match self {
            Self::Escape => Some(StageId::Level2A),
            Self::Tv => Some(StageId::Level2B),
            Self::Kabuki => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceScene, StageId, StoryCard, STAGES};
    use crate::game::{AppScreen, Game, LEVEL_X, LEVEL_Y};

    #[test]
    fn five_stages_have_stable_original_root_frames_and_symbols() {
        let expected = [
            (19, 696, 152, 151),
            (20, 1075, 783, 782),
            (21, 1251, 1095, 1094),
            (23, 1402, 1317, 1316),
            (25, 1496, 1485, 1484),
        ];
        for (index, id) in StageId::ALL.iter().enumerate() {
            let d = id.descriptor();
            let (frame, game, level, ground) = expected[index];
            assert_eq!(d.id, *id);
            assert_eq!((d.root_frame_zero_based, d.game_symbol, d.level_symbol, d.ground_symbol),
                       (frame, game, level, ground));
            assert_eq!(StageId::from_source_root_frame(frame), Some(*id));
            assert!(d.camera.min_x < d.camera.max_x);
            assert!(matches!(d.source_player_direction, -1 | 1));
        }
        assert_eq!(STAGES.len(), 5);
        for card_frame in [22, 24, 26] {
            assert_eq!(StageId::from_source_root_frame(card_frame), None);
        }
    }

    #[test]
    fn original_actionscript_labels_round_trip_without_renaming_penguin() {
        for id in StageId::ALL {
            assert_eq!(StageId::from_source_label(id.source_label()), Some(id));
        }
        assert_eq!(StageId::Level1C.source_label(), "level1c");
        assert_eq!(StageId::Level1C.manifest_key(), "level1c_penguin");
        assert_eq!(StageId::from_source_label("level1c_penguin"), None);
        assert_eq!(StageId::from_source_label("level3a"), None);
    }

    #[test]
    fn immutable_routes_preserve_important_intermediate_story_cards() {
        assert_eq!(StageId::Level1A.next_source_scene(), SourceScene::Stage(StageId::Level1B));
        assert_eq!(StageId::Level1B.next_source_scene(), SourceScene::Stage(StageId::Level1C));
        assert_eq!(StageId::Level1C.next_source_scene(), SourceScene::Card(StoryCard::Escape));
        assert_eq!(StoryCard::Escape.next_in_scope(), Some(StageId::Level2A));
        assert_eq!(StageId::Level2A.next_source_scene(), SourceScene::Card(StoryCard::Tv));
        assert_eq!(StoryCard::Tv.next_in_scope(), Some(StageId::Level2B));
        assert_eq!(StageId::Level2B.next_source_scene(), SourceScene::Card(StoryCard::Kabuki));
        assert_eq!(StoryCard::Kabuki.next_in_scope(), None);
        assert_eq!((StoryCard::Escape.root_frame_zero_based(), StoryCard::Escape.sprite_id()), (22,1267));
        assert_eq!((StoryCard::Tv.root_frame_zero_based(), StoryCard::Tv.sprite_id()), (24,1479));
        assert_eq!((StoryCard::Kabuki.root_frame_zero_based(), StoryCard::Kabuki.sprite_id()), (26,1569));
    }

    #[test]
    fn source_camera_and_music_cues_are_stage_specific_and_inert() {
        let expected = [
            (150.0, 1.25, -1350.0, 7650.0, 751),
            (125.0, 3.0, -6220.0, 600.0, 1076),
            (270.0, 4.0, 250.0, 2150.0, 1252),
            (275.0, 3.0, 200.0, 55252.0, 1409),
            (275.0, 3.0, 250.0, 55252.0, 1504),
        ];
        for (index, id) in StageId::ALL.iter().enumerate() {
            let d = id.descriptor();
            let (height, jump, min_x, max_x, cue) = expected[index];
            assert_eq!((d.camera.height, d.camera.jump, d.camera.min_x,
                        d.camera.max_x, d.root_start_sound_id),
                       (height, jump, min_x, max_x, cue));
        }
    }

    #[test]
    fn verified_source_player_positions_convert_from_game_to_level_space() {
        let expected_local = [
            (-2182.70, -900.75),
            (-506.50, -258.00),
            (-1363.60, 42.00),
            (-982.90, -122.85),
            (3203.60, 6616.05),
        ];
        for (id, (x, y)) in StageId::ALL.into_iter().zip(expected_local) {
            let d = id.descriptor();
            let (rx, ry) = d.player_spawn_level_local();
            assert!((rx - x).abs() < 0.002, "{id:?}: x={rx} != {x}");
            assert!((ry - y).abs() < 0.002, "{id:?}: y={ry} != {y}");
        }
    }

    #[test]
    fn bank_camera_boundaries_spawn_direction_and_jump_are_source_backed() {
        let stage=StageId::Level1B.descriptor();
        let camera=stage.camera;
        assert_eq!((camera.min_x,camera.max_x,camera.height,camera.jump),
                   (-6220.0,600.0,125.0,3.0));
        assert_eq!(stage.source_player_direction,-1);
        // 528.15 + 26.10 - left-facing target 475 = 79.25;
        // Flash truncates 79.25/10 to 7 before adjusting camera x.
        let stepped=camera.horizontal_step(528.15,26.10,stage.source_player_direction);
        assert!((stepped-19.10).abs()<0.0001);
        assert_eq!(camera.horizontal_step(-6220.0,26.10,-1),26.10);
        assert_eq!(camera.horizontal_step(600.0,26.10,-1),26.10);
        assert_eq!(camera.horizontal_step(528.15,-49.15,-1),-49.15);
        assert!((camera.jumping_y(-18.75)-(143.75/3.0)).abs()<0.0001);
        assert_eq!(StageId::Level1A.descriptor().camera.height,150.0);
    }

    #[test]
    fn original_camera_logic_vertical_target_and_truncated_grounded_return() {
        // SHA-verified B01 original AVM1 cameraLogic(), body offset 102517:
        // 103229..103265 jumping target and 103409..103473 grounded ease.
        let cases=[
            (StageId::Level1A,135.0_f32),
            (StageId::Level1B,47.9166667_f32),
            (StageId::Level1C,72.1875_f32),
        ];
        for (id,expected_target) in cases {
            let camera=id.descriptor().camera;
            let (jump_y,jump_delta)=camera.vertical_step(-18.75,26.1,true);
            assert!((jump_y-expected_target).abs()<0.0001,"{id:?} jump {jump_y}");
            assert!((jump_delta-(jump_y-26.1)).abs()<0.0001);
            let (ground_y,ground_delta)=camera.vertical_step(-18.75,26.1,false);
            let expected_ease=((expected_target-26.1)/4.0).trunc();
            assert!((ground_delta-expected_ease).abs()<0.0001,"{id:?} grounded delta {ground_delta}");
            assert!((ground_y-(26.1+expected_ease)).abs()<0.0001);
        }
        let bank=StageId::Level1B.descriptor().camera;
        // Flash ToInteger truncates toward zero; it does NOT floor negatives.
        let (y,delta)=bank.vertical_step(-18.75,100.0,false);
        assert_eq!(delta,-13.0);
        assert_eq!(y,87.0);
    }

    #[test]
    fn original_camera_logic_grounded_threshold_is_literal_150_not_cam_height() {
        // Original AVM1 Push(150) at absolute FWS offset 103368, followed
        // by Math.abs > 1; Bank Street instead initializes camHeight=125.
        let bank=StageId::Level1B.descriptor().camera;
        assert_eq!(bank.height,125.0);
        assert_eq!(bank.vertical_step(-18.75,168.75,false),(168.75,0.0));
        assert_eq!(bank.vertical_step(-18.75,169.75,false),(169.75,0.0));
        let (y,delta)=bank.vertical_step(-18.75,169.76,false);
        assert!(delta<0.0);
        assert!((y-(169.76+delta)).abs()<0.0001);
        assert!((bank.jumping_y(-18.75)-47.9166667).abs()<0.0001);
    }

    #[test]
    fn default_game_still_only_runs_level1a_with_unchanged_spawn() {
        let game = Game::new();
        assert_eq!(game.screen, AppScreen::Title);
        assert_eq!(game.active_stage(), StageId::Level1A);
        assert_eq!(game.intro_tick, 0);
        assert_eq!(game.ticks, 0);
        assert_eq!((game.score, game.lives, game.batarangs, game.health), (0, 5, 0, 100));
        assert_eq!((LEVEL_X, LEVEL_Y), StageId::Level1A.descriptor().level_origin_game_local);
        let (x, y) = StageId::Level1A.descriptor().player_spawn_level_local();
        assert!((game.player.x - x).abs() < 0.002);
        assert!((game.player.y - y).abs() < 0.002);
        assert!(game.shots.is_empty());
        assert!(!game.level1a_complete);
    }

    #[test]
    fn coordinate_and_depth_newtypes_preserve_round_trip_and_ordering() {
        let stage = StageId::Level1B.descriptor();
        let game_pos = stage.player_spawn_game_local;
        let level_pos = stage.to_level_pos(game_pos);
        assert_eq!(stage.player_spawn_level_pos(), level_pos);
        let back_to_game = stage.to_game_pos(level_pos);
        assert!((back_to_game.x - game_pos.x).abs() < 0.0001);
        assert!((back_to_game.y - game_pos.y).abs() < 0.0001);

        // Tuple conversion and PartialEq traits
        let tuple_gp: (f32, f32) = game_pos.into();
        assert_eq!(tuple_gp, (game_pos.x, game_pos.y));
        assert_eq!(game_pos, (game_pos.x, game_pos.y));
        assert_eq!((game_pos.x, game_pos.y), game_pos);

        let tuple_lp: (f32, f32) = level_pos.into();
        assert_eq!(tuple_lp, (level_pos.x, level_pos.y));
        assert_eq!(level_pos, (level_pos.x, level_pos.y));
        assert_eq!((level_pos.x, level_pos.y), level_pos);

        // Depth types and ordering
        let d_prop = GameDepth(1077);
        let d_actor = GameDepth(1127);
        assert!(d_prop < d_actor);
        assert_eq!(d_prop, 1077u16);
        assert_eq!(1077u16, d_prop);
        assert!(d_prop < 1127u16);
        assert!(1077u16 < d_actor);
        assert_eq!(u16::from(d_prop), 1077);

        let l_ground = LevelDepth(1);
        assert_eq!(l_ground, 1u16);
        assert_eq!(u16::from(l_ground), 1);
    }
}
