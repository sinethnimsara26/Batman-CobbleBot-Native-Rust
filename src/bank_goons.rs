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
use crate::collision::CollisionMask;
use crate::stage::{GameDepth, GamePos, LevelPos, StageId};

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

/// Original depth-1 child movie clip within a labeled 89-frame parent.
/// Ranges index separately baked family frames, not 89-frame parent snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceGoonChildClip {
    pub name: &'static str,
    pub source_child_symbol_id: u16,
    pub first_frame_inclusive: u16,
    pub end_frame_exclusive: u16,
}
impl SourceGoonChildClip {
    pub const fn frame_count(self) -> u16 {
        self.end_frame_exclusive - self.first_frame_inclusive
    }
}

/// Original source child frames for family A; NOT game ticks or AI durations.
pub const GOON_A_CLIPS: [SourceGoonChildClip; 9] = [
    SourceGoonChildClip { name: "stand", source_child_symbol_id: 799, first_frame_inclusive: 0, end_frame_exclusive: 1 },
    SourceGoonChildClip { name: "walk", source_child_symbol_id: 824, first_frame_inclusive: 1, end_frame_exclusive: 11 },
    SourceGoonChildClip { name: "retreat", source_child_symbol_id: 834, first_frame_inclusive: 11, end_frame_exclusive: 21 },
    SourceGoonChildClip { name: "attack", source_child_symbol_id: 847, first_frame_inclusive: 21, end_frame_exclusive: 31 },
    SourceGoonChildClip { name: "hurt", source_child_symbol_id: 854, first_frame_inclusive: 31, end_frame_exclusive: 37 },
    SourceGoonChildClip { name: "fall", source_child_symbol_id: 873, first_frame_inclusive: 37, end_frame_exclusive: 45 },
    SourceGoonChildClip { name: "die", source_child_symbol_id: 879, first_frame_inclusive: 45, end_frame_exclusive: 53 },
    SourceGoonChildClip { name: "idle", source_child_symbol_id: 799, first_frame_inclusive: 53, end_frame_exclusive: 54 },
    SourceGoonChildClip { name: "intro", source_child_symbol_id: 799, first_frame_inclusive: 54, end_frame_exclusive: 55 },
];

/// Original source child frames for family B; NOT game ticks or AI durations.
pub const GOON_B_CLIPS: [SourceGoonChildClip; 9] = [
    SourceGoonChildClip { name: "stand", source_child_symbol_id: 883, first_frame_inclusive: 0, end_frame_exclusive: 1 },
    SourceGoonChildClip { name: "walk", source_child_symbol_id: 915, first_frame_inclusive: 1, end_frame_exclusive: 12 },
    SourceGoonChildClip { name: "retreat", source_child_symbol_id: 923, first_frame_inclusive: 12, end_frame_exclusive: 23 },
    SourceGoonChildClip { name: "attack", source_child_symbol_id: 939, first_frame_inclusive: 23, end_frame_exclusive: 33 },
    SourceGoonChildClip { name: "hurt", source_child_symbol_id: 944, first_frame_inclusive: 33, end_frame_exclusive: 39 },
    SourceGoonChildClip { name: "fall", source_child_symbol_id: 959, first_frame_inclusive: 39, end_frame_exclusive: 47 },
    SourceGoonChildClip { name: "die", source_child_symbol_id: 963, first_frame_inclusive: 47, end_frame_exclusive: 55 },
    SourceGoonChildClip { name: "idle", source_child_symbol_id: 883, first_frame_inclusive: 55, end_frame_exclusive: 56 },
    SourceGoonChildClip { name: "intro", source_child_symbol_id: 939, first_frame_inclusive: 56, end_frame_exclusive: 66 },
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
    /// True source child selected by this outer label, not a playback policy.
    pub fn source_child_clip(self, name: &str) -> Option<SourceGoonChildClip> {
        let clips: &[SourceGoonChildClip; 9] = match self.source_symbol_id {
            880 => &GOON_A_CLIPS,
            964 => &GOON_B_CLIPS,
            _ => return None,
        };
        clips.iter().copied().find(|clip| clip.name == name)
    }

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
    pub game_depth: GameDepth,
    /// Original game-local placement translation, NOT world camera pixels.
    pub x_game: f32,
    pub y_game: f32,
}

impl BankGoonSpawn {
    #[inline]
    pub fn game_pos(self) -> GamePos {
        GamePos::new(self.x_game, self.y_game)
    }

    #[inline]
    pub fn level_pos(self) -> LevelPos {
        StageId::Level1B.descriptor().to_level_pos(self.game_pos())
    }

    #[inline]
    pub fn level_local(self) -> (f32, f32) {
        let p = self.level_pos();
        (p.x, p.y)
    }
}

/// Ten ORIGINAL game #1075 frame-0 goon instances in authored depth order.
pub const BANK_GOON_SPAWNS: [BankGoonSpawn; 10] = [
    BankGoonSpawn { name: "goon1", symbol_id: 880, game_depth: GameDepth(1105), x_game: -1086.15_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon2", symbol_id: 964, game_depth: GameDepth(1107), x_game: -746.15_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon4", symbol_id: 880, game_depth: GameDepth(1109), x_game: -2074.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon3", symbol_id: 964, game_depth: GameDepth(1111), x_game: -1734.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon6", symbol_id: 964, game_depth: GameDepth(1113), x_game: -3466.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon5", symbol_id: 964, game_depth: GameDepth(1115), x_game: -3126.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon7", symbol_id: 964, game_depth: GameDepth(1117), x_game: -5364.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon9", symbol_id: 880, game_depth: GameDepth(1119), x_game: -5970.75_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon8", symbol_id: 880, game_depth: GameDepth(1121), x_game: -4976.85_f32, y_game: 398.1_f32 },
    BankGoonSpawn { name: "goon10", symbol_id: 964, game_depth: GameDepth(1123), x_game: -6110.45_f32, y_game: 398.1_f32 },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoonState {
    Idle,
    Stand,
    Walk,
    Retreat,
    Attack,
    Hurt,
    Fall,
    Die,
    Intro,
}

impl GoonState {
    pub const fn label_name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Stand => "stand",
            Self::Walk => "walk",
            Self::Retreat => "retreat",
            Self::Attack => "attack",
            Self::Hurt => "hurt",
            Self::Fall => "fall",
            Self::Die => "die",
            Self::Intro => "intro",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoonFamily {
    A, // DefineSprite #880
    B, // DefineSprite #964
}

impl GoonFamily {
    pub const fn symbol_id(self) -> u16 {
        match self {
            Self::A => 880,
            Self::B => 964,
        }
    }

    pub fn clips(self) -> &'static [SourceGoonChildClip; 9] {
        match self {
            Self::A => &GOON_A_CLIPS,
            Self::B => &GOON_B_CLIPS,
        }
    }

    pub fn child_clip(self, state: GoonState) -> SourceGoonChildClip {
        let name = state.label_name();
        let clips = self.clips();
        for clip in clips {
            if clip.name == name {
                return *clip;
            }
        }
        clips[0]
    }
}

/// Deterministic pseudo-random number generator for goon AI decisions.
/// Uses 64-bit XorShift64* and produces f32 in [0.0, 1.0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoonRng {
    pub state: u64,
}

impl GoonRng {
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

impl Default for GoonRng {
    fn default() -> Self {
        Self::new(0x1337_BCB1_0000)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoonCombatSound {
    Punch,
    Punch2,
    Kick,
    Metal(u8),
    Bam,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerCombatContext {
    pub x_game: f32,
    pub y_game: f32,
    pub dir: i32,
    pub is_attacking: bool,
    pub is_invincible: bool,
    pub state_name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GoonCombatEvent {
    Sound(GoonCombatSound),
    Score(u32),
    PlayerHurt { damage: i32 },
    Wave2Spawned { count: usize },
    StageComplete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BankGoon {
    pub name: &'static str,
    pub family: GoonFamily,
    pub game_depth: GameDepth,
    pub x: f32,
    pub y: f32,
    pub dir: i8, // -1: left, 1: right
    pub state: GoonState,
    pub frame: u16, // child frame within state's clip
    pub health: i32,
    pub max_health: i32,
    pub strength: i32,
    pub speed: f32,
    pub attack_count: f32,
    pub fall_count: u16,
    pub flash: u16,
    pub invincible: bool,
    pub visible: bool,
    pub active: bool,
    pub hit_count: u32,
    pub is_wave2: bool,
    pub spawn_index: usize,
}

impl BankGoon {
    pub fn from_spawn(spawn: BankGoonSpawn) -> Self {
        Self::from_spawn_with_index(spawn, 0)
    }

    pub fn from_spawn_with_index(spawn: BankGoonSpawn, spawn_index: usize) -> Self {
        let family = match spawn.symbol_id {
            880 => GoonFamily::A,
            964 => GoonFamily::B,
            _ => GoonFamily::A,
        };
        Self {
            name: spawn.name,
            family,
            game_depth: spawn.game_depth,
            x: spawn.x_game,
            y: spawn.y_game,
            dir: -1,
            state: GoonState::Idle,
            frame: 0,
            health: 32,
            max_health: 32,
            strength: 5,
            speed: 9.0,
            attack_count: 0.0,
            fall_count: 0,
            flash: 0,
            invincible: false,
            visible: true,
            active: true,
            hit_count: 0,
            is_wave2: false,
            spawn_index,
        }
    }

    #[inline]
    pub fn game_pos(&self) -> GamePos {
        GamePos::new(self.x, self.y)
    }

    #[inline]
    pub fn level_pos(&self) -> LevelPos {
        StageId::Level1B.descriptor().to_level_pos(self.game_pos())
    }

    pub fn set_state(&mut self, new_state: GoonState) {
        if self.state != new_state {
            self.state = new_state;
            self.frame = 0;
        }
    }

    /// Check if a flying batarang hits this goon.
    /// In Flash AVM1: checkBatarang(char) -> char.curBat.
    /// Goon bounding box: width ~50px (+-25px), height ~85px (y-75 to y+10).
    pub fn apply_batarang_hit(
        &mut self,
        bat_x: f32,
        bat_y: f32,
        events: &mut Vec<GoonCombatEvent>,
    ) -> bool {
        if !self.active || self.health <= 0 {
            return false;
        }
        if self.state == GoonState::Hurt || self.state == GoonState::Fall || self.state == GoonState::Die {
            return false;
        }
        let hit_x = (bat_x - self.x).abs() <= 25.0;
        let hit_y = bat_y >= self.y - 75.0 && bat_y <= self.y + 10.0;
        if hit_x && hit_y {
            self.health -= 5;
            events.push(GoonCombatEvent::Sound(GoonCombatSound::Kick));
            if self.health <= 0 {
                self.health = 0;
                self.set_state(GoonState::Die);
                self.hit_count = 0;
                events.push(GoonCombatEvent::Score(100));
                events.push(GoonCombatEvent::Sound(GoonCombatSound::Bam));
            } else {
                self.set_state(GoonState::Hurt);
            }
            return true;
        }
        false
    }

    /// Check if Batman attacks this goon.
    /// In Flash AVM1:
    /// - checkPlayer(char, dir, -120): close-quarters knockdown sweep -> 20 dmg, Fall, Punch2
    /// - checkPlayer(char, dir, -20): standard attack reach:
    ///   - lowkick -> 15 dmg, Fall, Kick
    ///   - highpunch -> 15 dmg, Fall, Punch2
    ///   - highkick -> 15 dmg, Fall, Kick
    ///   - punch (default) -> 10 dmg, Hurt, Punch, comboMeter / hitCount++
    pub fn apply_player_attack(
        &mut self,
        player: &PlayerCombatContext,
        events: &mut Vec<GoonCombatEvent>,
    ) -> bool {
        if !self.active || self.health <= 0 || !player.is_attacking {
            return false;
        }
        if self.state == GoonState::Hurt || self.state == GoonState::Fall || self.state == GoonState::Die {
            return false;
        }

        let dx = (self.x - player.x_game).abs();
        let dy = (self.y - player.y_game).abs();
        if dy > 50.0 {
            return false;
        }

        // Close-quarters knockdown (offset -120 in AVM1, dx <= 45.0)
        if dx <= 45.0 {
            self.health -= 20;
            events.push(GoonCombatEvent::Sound(GoonCombatSound::Punch2));
            self.invincible = true;
            self.fall_count = 0;
            if self.health <= 0 {
                self.health = 0;
                self.set_state(GoonState::Die);
                self.hit_count = 0;
                events.push(GoonCombatEvent::Score(100));
                events.push(GoonCombatEvent::Sound(GoonCombatSound::Bam));
            } else {
                self.set_state(GoonState::Fall);
            }
            return true;
        }

        // Standard attack reach (offset -20 in AVM1, dx <= 90.0)
        if dx <= 90.0 {
            let (dmg, sound, new_state) = match player.state_name {
                "lowkick" => (15, GoonCombatSound::Kick, GoonState::Fall),
                "highpunch" => (15, GoonCombatSound::Punch2, GoonState::Fall),
                "highkick" => (15, GoonCombatSound::Kick, GoonState::Fall),
                _ => (10, GoonCombatSound::Punch, GoonState::Hurt),
            };
            self.health -= dmg;
            events.push(GoonCombatEvent::Sound(sound));
            if new_state == GoonState::Hurt {
                self.hit_count += 1;
            } else {
                self.fall_count = 0;
            }
            if self.health <= 0 {
                self.health = 0;
                self.set_state(GoonState::Die);
                self.hit_count = 0;
                events.push(GoonCombatEvent::Score(100));
                events.push(GoonCombatEvent::Sound(GoonCombatSound::Bam));
            } else {
                self.set_state(new_state);
            }
            return true;
        }

        false
    }

    /// Full combat and noncombat tick based on original Flash AVM1 goonAI.
    pub fn tick_combat(
        &mut self,
        player: &PlayerCombatContext,
        collision: &CollisionMask,
        rng: &mut GoonRng,
        events: &mut Vec<GoonCombatEvent>,
    ) {
        if !self.active {
            return;
        }

        // Die state: flash counter and despawn removal
        if self.state == GoonState::Die {
            let clip = self.family.child_clip(GoonState::Die);
            // Lock to last frame, no parent wrapping or upright popping!
            self.frame = (self.frame + 1).min(clip.frame_count().saturating_sub(1));
            self.flash += 1;
            if self.flash > 50 {
                self.visible = (self.flash % 4) >= 2;
            }
            if self.flash > 100 {
                self.visible = false;
                self.active = false;
            }
            return;
        }

        // Hurt state: knockback slide (-6 * dir) and child animation
        if self.state == GoonState::Hurt {
            let kx = -6.0 * (self.dir as f32);
            if !check_goon_wall(collision, self.x, self.y, kx) {
                self.x += kx;
            }
            let clip = self.family.child_clip(GoonState::Hurt);
            if self.frame + 1 >= clip.frame_count() {
                self.set_state(GoonState::Stand);
            } else {
                self.frame += 1;
            }
            return;
        }

        // Fall state: knockback slide (-10 * dir) for up to 6 ticks, child animation
        if self.state == GoonState::Fall {
            if self.fall_count < 6 {
                let kx = -10.0 * (self.dir as f32);
                if !check_goon_wall(collision, self.x, self.y, kx) {
                    self.x += kx;
                }
                self.fall_count += 1;
            }
            let clip = self.family.child_clip(GoonState::Fall);
            if self.frame + 1 >= clip.frame_count() {
                self.set_state(GoonState::Stand);
            } else {
                self.frame += 1;
            }
            return;
        }

        // Attack state: strikes player and returns to Stand
        if self.state == GoonState::Attack {
            let clip = self.family.child_clip(GoonState::Attack);
            // Strike window at frames 4..6
            if self.frame >= 4 && self.frame <= 6 {
                let range = (self.x - player.x_game).abs();
                let rangey = (self.y - player.y_game).abs();
                if range <= 70.0 && rangey <= 40.0 && !player.is_invincible && player.state_name != "hurt" {
                    events.push(GoonCombatEvent::PlayerHurt { damage: self.strength });
                    events.push(GoonCombatEvent::Sound(GoonCombatSound::Metal(1)));
                }
            }
            if self.frame + 1 >= clip.frame_count() {
                self.set_state(GoonState::Stand);
            } else {
                self.frame += 1;
            }
            return;
        }

        // In Stand: check if goon decides to attack player
        if self.state == GoonState::Stand {
            let range = (self.x - player.x_game).abs();
            let rangey = (self.y - player.y_game).abs();
            if range < 70.0 && rangey < 40.0 && self.attack_count > 10.0 {
                self.set_state(GoonState::Attack);
                self.attack_count = 0.0;
                return;
            }
        }

        // Other states follow noncombat motion and checks
        self.tick_noncombat(player.x_game, player.y_game, collision, rng);
    }

    /// Step goon noncombat state machine based on exact Flash AVM1 goonAI.
    pub fn tick_noncombat(
        &mut self,
        player_x: f32,
        player_y: f32,
        collision: &CollisionMask,
        rng: &mut GoonRng,
    ) {
        if !self.active || self.health <= 0 {
            return;
        }

        let range = (self.x - player_x).abs();
        let rangey = (self.y - player_y).abs();

        // 1. Direction / facing adjustment (from AVM1 lines 40-78)
        if range > 180.0 && rangey < 400.0 {
            if self.x > player_x {
                self.dir = -1;
            } else if self.x < player_x {
                self.dir = 1;
            }
        } else if range < 50.0 && rangey < 400.0 {
            if self.state != GoonState::Attack
                && self.state != GoonState::Hurt
                && self.state != GoonState::Fall
            {
                self.set_state(GoonState::Retreat);
            }
        }

        // 2. State-specific behavior
        match self.state {
            GoonState::Idle => {
                // Flash AVM1 line 60: if range < 400 && rangey < 400 -> stand
                if range < 400.0 && rangey < 400.0 {
                    self.set_state(GoonState::Stand);
                }
            }
            GoonState::Intro => {
                let clip = self.family.child_clip(GoonState::Intro);
                if self.frame + 1 >= clip.frame_count() {
                    self.set_state(GoonState::Stand);
                } else {
                    self.frame += 1;
                }
            }
            GoonState::Stand => {
                self.attack_count += 1.0;
                let clip = self.family.child_clip(GoonState::Stand);
                self.frame = (self.frame + 1) % clip.frame_count().max(1);

                // Flash AVM1 lines 78-95:
                if range < 150.0 && self.attack_count < 8.0 {
                    let rand = rng.next_f32() - rng.next_f32();
                    if rand > 0.0 {
                        self.set_state(GoonState::Retreat);
                        self.attack_count = -rng.next_f32() * 10.0;
                        if rand > 0.5 {
                            self.dir = -self.dir;
                        }
                    } else {
                        self.set_state(GoonState::Walk);
                    }
                } else if self.attack_count > (rng.next_f32() * 20.0 + 25.0) {
                    if range > 300.0 {
                        self.set_state(GoonState::Walk);
                    }
                }
            }
            GoonState::Walk => {
                let dx = self.speed * (self.dir as f32);
                let wall = check_goon_wall(collision, self.x, self.y, dx);
                let edge = check_goon_edge(collision, self.x, self.y, 50.0 * (self.dir as f32));

                if !wall && edge {
                    self.x += dx;
                    self.attack_count += 1.0;
                    let clip = self.family.child_clip(GoonState::Walk);
                    self.frame = (self.frame + 1) % clip.frame_count().max(1);
                } else {
                    // Flash AVM1: stopped by wall or edge; turns around and returns to stand
                    self.dir = -self.dir;
                    self.set_state(GoonState::Stand);
                }
            }
            GoonState::Retreat => {
                let dx = self.speed * (self.dir as f32);
                let wall = check_goon_wall(collision, self.x, self.y, -dx);
                let edge = check_goon_edge(collision, self.x, self.y, -50.0 * (self.dir as f32));

                if !wall && edge {
                    self.x -= dx;
                    self.attack_count += 1.0;
                    let clip = self.family.child_clip(GoonState::Retreat);
                    self.frame = (self.frame + 1) % clip.frame_count().max(1);

                    // Flash AVM1 lines 111-120:
                    if self.attack_count > 5.0 && range > (rng.next_f32() * 100.0 + 250.0) {
                        let rand = rng.next_f32() - rng.next_f32();
                        if rand > 0.5 {
                            self.set_state(GoonState::Walk);
                        } else {
                            self.set_state(GoonState::Stand);
                        }
                    }
                } else {
                    // Flash AVM1: stopped by wall or edge in retreat
                    self.set_state(GoonState::Stand);
                }
            }
            _ => {}
        }
    }
}

/// Checks wall ahead of goon using Level 1B ground mask.
/// Flash AVM1 checkWall: hitTest at (goon.x + dx, goon.y - 25.0).
pub fn check_goon_wall(collision: &CollisionMask, x_game: f32, y_game: f32, dx: f32) -> bool {
    let stage = StageId::Level1B.descriptor();
    let test_pos = stage.to_level_pos(GamePos::new(x_game + dx, y_game - 25.0));
    collision.solid_level_pos(test_pos)
}

/// Checks ground ahead of goon using Level 1B ground mask.
/// Flash AVM1 checkEdge: hitTest at (goon.x + dx, goon.y + 4.0).
pub fn check_goon_edge(collision: &CollisionMask, x_game: f32, y_game: f32, dx: f32) -> bool {
    let stage = StageId::Level1B.descriptor();
    let test_pos = stage.to_level_pos(GamePos::new(x_game + dx, y_game + 4.0));
    collision.solid_level_pos(test_pos)
}

#[derive(Clone, Debug, PartialEq)]
pub struct BankGoonSquad {
    pub goons: [BankGoon; 10],
    pub rng: GoonRng,
    pub e_kills: usize,
    pub respawned: u32,
}

impl BankGoonSquad {
    pub fn new() -> Self {
        let mut idx = 0;
        let goons = BANK_GOON_SPAWNS.map(|spawn| {
            let g = BankGoon::from_spawn_with_index(spawn, idx);
            idx += 1;
            g
        });
        Self {
            goons,
            rng: GoonRng::new(0x1337_BCB1_0000),
            e_kills: 0,
            respawned: 0,
        }
    }

    #[inline]
    pub fn alive_count(&self) -> usize {
        self.goons.iter().filter(|g| g.active && g.health > 0).count()
    }

    #[inline]
    pub fn active_count(&self) -> usize {
        self.goons.iter().filter(|g| g.active).count()
    }

    #[inline]
    pub fn is_wave2_active(&self) -> bool {
        self.respawned > 0
    }

    #[inline]
    pub fn is_stage_clear(&self) -> bool {
        self.respawned > 0 && self.e_kills >= 8
    }

    pub fn tick_noncombat(
        &mut self,
        player_x: f32,
        player_y: f32,
        collision: &CollisionMask,
    ) {
        for goon in &mut self.goons {
            goon.tick_noncombat(player_x, player_y, collision, &mut self.rng);
        }
    }

    /// Full squad combat tick with batarangs, player melee attacks,
    /// goon attacks, and second wave activation (`respawn2`).
    pub fn tick_combat(
        &mut self,
        player: &PlayerCombatContext,
        batarangs: &[(f32, f32)],
        collision: &CollisionMask,
        events: &mut Vec<GoonCombatEvent>,
    ) {
        // 1. Check batarangs against alive goons
        for &(bat_x, bat_y) in batarangs {
            for goon in &mut self.goons {
                if goon.apply_batarang_hit(bat_x, bat_y, events) {
                    break;
                }
            }
        }

        // 2. Check player melee attacks against alive goons
        if player.is_attacking {
            for goon in &mut self.goons {
                if goon.apply_player_attack(player, events) {
                    break;
                }
            }
        }

        // 3. Tick all goons and detect despawn removal
        for goon in &mut self.goons {
            let was_active = goon.active;
            goon.tick_combat(player, collision, &mut self.rng, events);
            if was_active && !goon.active {
                self.e_kills += 1;
            }
        }

        // 4. Check wave 2 activation (Flash AVM1 respawn2)
        // If initial wave (10 goons) is defeated:
        if self.respawned == 0 && self.e_kills >= 10 {
            self.e_kills = 0;
            self.respawned = 1;

            // Flash AVM1 respawn2 loops i = 0..7 (8 enemies)
            for i in 0..8 {
                let goon = &mut self.goons[i];
                goon.active = true;
                goon.visible = true;
                goon.health = 50;
                goon.max_health = 50;
                goon.strength = 5;
                goon.speed = 9.0;
                goon.attack_count = 0.0;
                goon.fall_count = 0;
                goon.flash = 0;
                goon.hit_count = 0;
                goon.is_wave2 = true;

                // Alternate spawns to right and left of player with random offset (500..1100 px)
                let rand_offset = 500.0 + (self.rng.next_f32() * 600.0);
                if i % 2 == 0 {
                    goon.x = player.x_game + rand_offset;
                    goon.dir = -1;
                } else {
                    goon.x = player.x_game - rand_offset;
                    goon.dir = 1;
                }
                goon.set_state(GoonState::Walk);
            }
            events.push(GoonCombatEvent::Wave2Spawned { count: 8 });
        }

        // 5. Check stage exit trigger (Flash AVM1 respawn2 end)
        // When wave 2 (8 enemies) is defeated:
        if self.respawned > 0 && self.e_kills >= 8 {
            events.push(GoonCombatEvent::StageComplete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        check_goon_edge, check_goon_wall, BankGoon, BankGoonSquad, GoonCombatEvent,
        GoonCombatSound, GoonFamily, GoonRng, GoonState, PlayerCombatContext,
        BANK_GOON_SPAWNS, GOON_A_CLIPS, GOON_B_CLIPS, GOON_LABELS, GOON_TIMELINES,
    };
    use crate::collision::CollisionMask;
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
        assert_eq!(BANK_GOON_SPAWNS.map(|g|u16::from(g.game_depth)),
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
    fn original_nested_goon_child_clips_are_complete_and_never_parent_wrapped() {
        let a = [(799,1),(824,10),(834,10),(847,10),(854,6),(873,8),(879,8),(799,1),(799,1)];
        let b = [(883,1),(915,11),(923,11),(939,10),(944,6),(959,8),(963,8),(883,1),(939,10)];
        for (outer, clips, expected, total) in [
            (GOON_TIMELINES[0], &GOON_A_CLIPS, &a, 55),
            (GOON_TIMELINES[1], &GOON_B_CLIPS, &b, 66),
        ] {
            let mut cursor = 0;
            for (i, clip) in clips.iter().enumerate() {
                assert_eq!(clip.name, GOON_LABELS[i].name);
                assert_eq!(clip.first_frame_inclusive, cursor);
                assert_eq!((clip.source_child_symbol_id, clip.frame_count()), expected[i]);
                assert_eq!(outer.source_child_clip(clip.name), Some(*clip));
                cursor = clip.end_frame_exclusive;
            }
            assert_eq!(cursor, total);
            assert_eq!(outer.source_child_clip("unknown"), None);
        }
        // B's 11th walk/retreat frame must not be lost at parent-label end.
        assert_eq!(GOON_B_CLIPS[1].frame_count(), 11);
        assert_eq!(GOON_B_CLIPS[2].frame_count(), 11);
        // Hurt, fall, die stop at their child end; no modulo parent-wrap.
        for clips in [&GOON_A_CLIPS, &GOON_B_CLIPS] {
            assert_eq!([clips[4].frame_count(), clips[5].frame_count(), clips[6].frame_count()], [6,8,8]);
        }
    }

    #[test]
    fn source_spawn_space_is_level_relative_but_game_remains_level1a() {
        let d=StageId::Level1B.descriptor();
        assert_eq!((d.root_frame_zero_based,d.game_symbol,d.level_symbol,d.ground_symbol),
                   (20,1075,783,782));
        let p=BANK_GOON_SPAWNS[0];
        let p_level = p.level_pos();
        assert_eq!(p.level_local(), (p_level.x, p_level.y));
        let (x,y)=p.level_local();
        assert!((x+2120.80).abs()<0.002);
        assert!((y-158.85).abs()<0.002);
        let game=crate::game::Game::new();
        assert_eq!(game.active_stage(),StageId::Level1A);
    }

    #[test]
    fn goon_initialization_from_spawns_matches_spec() {
        let squad = BankGoonSquad::new();
        assert_eq!(squad.goons.len(), 10);
        let mut count_a = 0;
        let mut count_b = 0;
        for (i, goon) in squad.goons.iter().enumerate() {
            assert_eq!(goon.health, 32);
            assert_eq!(goon.max_health, 32);
            assert_eq!(goon.strength, 5);
            assert_eq!(goon.speed, 9.0);
            assert_eq!(goon.state, GoonState::Idle);
            assert_eq!(goon.frame, 0);
            assert_eq!(goon.dir, -1);
            assert!(goon.active);
            assert!(goon.visible);
            assert_eq!(goon.name, BANK_GOON_SPAWNS[i].name);
            assert_eq!(goon.x, BANK_GOON_SPAWNS[i].x_game);
            assert_eq!(goon.y, BANK_GOON_SPAWNS[i].y_game);
            match goon.family {
                GoonFamily::A => count_a += 1,
                GoonFamily::B => count_b += 1,
            }
        }
        assert_eq!((count_a, count_b), (4, 6));
    }

    #[test]
    fn goon_proximity_activation_and_facing() {
        let ground = CollisionMask::level1b();
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]); // x = -1086.15, y = 398.1
        let mut rng = GoonRng::new(42);

        // Far away: range = 500, rangey = 0 -> stays Idle
        goon.tick_noncombat(goon.x + 500.0, goon.y, &ground, &mut rng);
        assert_eq!(goon.state, GoonState::Idle);

        // Within 400px: range = 350 -> activates to Stand
        goon.tick_noncombat(goon.x + 350.0, goon.y, &ground, &mut rng);
        assert_eq!(goon.state, GoonState::Stand);

        // Facing: player is to the right (x > goon.x), range > 180 -> dir should be 1
        assert_eq!(goon.dir, 1);

        // Facing: player moves to the left (x < goon.x), range > 180 -> dir should be -1
        goon.tick_noncombat(goon.x - 300.0, goon.y, &ground, &mut rng);
        assert_eq!(goon.dir, -1);
    }

    #[test]
    fn goon_walk_and_retreat_motion_and_child_clips() {
        let ground = CollisionMask::level1b();
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        let mut rng = GoonRng::new(100);

        // Force to Walk rightwards
        goon.state = GoonState::Walk;
        goon.dir = 1;
        let start_x = goon.x;

        // One tick on clear ground
        goon.tick_noncombat(goon.x + 250.0, goon.y, &ground, &mut rng);
        assert_eq!(goon.x, start_x + 9.0);
        assert_eq!(goon.frame, 1);

        // Force to Retreat rightwards (moves in opposite direction: x -= speed * dir)
        goon.state = GoonState::Retreat;
        let retreat_start_x = goon.x;
        goon.tick_noncombat(goon.x + 250.0, goon.y, &ground, &mut rng);
        assert_eq!(goon.x, retreat_start_x - 9.0);
    }

    #[test]
    fn goon_ledge_edge_and_wall_turn_with_collision_mask() {
        let ground = CollisionMask::level1b();
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        let mut rng = GoonRng::new(200);

        // At goon spawn y=398.1 (level y=158.85), ground is solid below
        assert!(check_goon_edge(&ground, goon.x, goon.y, 0.0));
        // Wall check ahead on flat street is clear
        assert!(!check_goon_wall(&ground, goon.x, goon.y, 0.0));

        // Far off in the void past street boundary (x = 2500.0), edge check returns false
        assert!(!check_goon_edge(&ground, 2500.0, goon.y, 0.0));

        // When a goon in Walk state reaches near the right edge (x = 1930.0), ground ahead (1980.0) is missing
        goon.x = 1930.0;
        goon.state = GoonState::Walk;
        goon.dir = 1;
        goon.tick_noncombat(goon.x + 200.0, goon.y, &ground, &mut rng);
        // Did not advance forward, turned around to -1 and returned to Stand
        assert_eq!(goon.dir, -1);
        assert_eq!(goon.state, GoonState::Stand);
    }

    #[test]
    fn goon_deterministic_rng_reproducibility() {
        let ground = CollisionMask::level1b();
        let mut squad1 = BankGoonSquad::new();
        let mut squad2 = BankGoonSquad::new();

        for _ in 0..50 {
            squad1.tick_noncombat(-1000.0, 398.1, &ground);
            squad2.tick_noncombat(-1000.0, 398.1, &ground);
        }

        for (g1, g2) in squad1.goons.iter().zip(squad2.goons.iter()) {
            assert_eq!(g1.x, g2.x);
            assert_eq!(g1.y, g2.y);
            assert_eq!(g1.dir, g2.dir);
            assert_eq!(g1.state, g2.state);
            assert_eq!(g1.frame, g2.frame);
            assert_eq!(g1.attack_count, g2.attack_count);
        }
    }

    #[test]
    fn goon_takes_batarang_damage_and_falls_back_in_hurt() {
        let ground = CollisionMask::level1b();
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        let mut rng = GoonRng::new(300);
        let mut events = Vec::new();

        let initial_health = goon.health; // 32
        // Batarang flying straight at goon body
        let hit = goon.apply_batarang_hit(goon.x, goon.y - 30.0, &mut events);
        assert!(hit);
        assert_eq!(goon.health, initial_health - 5);
        assert_eq!(goon.state, GoonState::Hurt);
        assert_eq!(events, vec![GoonCombatEvent::Sound(GoonCombatSound::Kick)]);

        // During Hurt, goon slides back (-6 * dir; dir is -1 so moves right by 6)
        let pre_x = goon.x;
        let dummy_player = PlayerCombatContext {
            x_game: -500.0,
            y_game: 398.1,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };
        goon.tick_combat(&dummy_player, &ground, &mut rng, &mut events);
        assert_eq!(goon.x, pre_x + 6.0);
    }

    #[test]
    fn goon_takes_punches_and_kicks_with_exact_damage() {
        let ground = CollisionMask::level1b();
        let mut rng = GoonRng::new(400);

        // 1. Standard Punch: 10 damage, Hurt state, Sound::Punch
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        let mut events = Vec::new();
        let p_punch = PlayerCombatContext {
            x_game: goon.x + 60.0,
            y_game: goon.y,
            dir: -1,
            is_attacking: true,
            is_invincible: false,
            state_name: "punch",
        };
        assert!(goon.apply_player_attack(&p_punch, &mut events));
        assert_eq!(goon.health, 22); // 32 - 10
        assert_eq!(goon.state, GoonState::Hurt);
        assert_eq!(events, vec![GoonCombatEvent::Sound(GoonCombatSound::Punch)]);

        // 2. Low Kick: 15 damage, Fall state, Sound::Kick
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        events.clear();
        let p_lowkick = PlayerCombatContext {
            x_game: goon.x + 60.0,
            y_game: goon.y,
            dir: -1,
            is_attacking: true,
            is_invincible: false,
            state_name: "lowkick",
        };
        assert!(goon.apply_player_attack(&p_lowkick, &mut events));
        assert_eq!(goon.health, 17); // 32 - 15
        assert_eq!(goon.state, GoonState::Fall);
        assert_eq!(events, vec![GoonCombatEvent::Sound(GoonCombatSound::Kick)]);

        // 3. High Punch: 15 damage, Fall state, Sound::Punch2
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        events.clear();
        let p_highpunch = PlayerCombatContext {
            x_game: goon.x + 60.0,
            y_game: goon.y,
            dir: -1,
            is_attacking: true,
            is_invincible: false,
            state_name: "highpunch",
        };
        assert!(goon.apply_player_attack(&p_highpunch, &mut events));
        assert_eq!(goon.health, 17); // 32 - 15
        assert_eq!(goon.state, GoonState::Fall);
        assert_eq!(events, vec![GoonCombatEvent::Sound(GoonCombatSound::Punch2)]);

        // 4. Close-quarters Knockdown (dx <= 45.0): 20 damage, Fall state, Sound::Punch2
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        events.clear();
        let p_close = PlayerCombatContext {
            x_game: goon.x + 30.0, // within 45px!
            y_game: goon.y,
            dir: -1,
            is_attacking: true,
            is_invincible: false,
            state_name: "punch",
        };
        assert!(goon.apply_player_attack(&p_close, &mut events));
        assert_eq!(goon.health, 12); // 32 - 20
        assert_eq!(goon.state, GoonState::Fall);
        assert_eq!(events, vec![GoonCombatEvent::Sound(GoonCombatSound::Punch2)]);
    }

    #[test]
    fn goon_death_score_flashing_and_removal() {
        let ground = CollisionMask::level1b();
        let mut rng = GoonRng::new(500);
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        goon.health = 10;
        let mut events = Vec::new();

        let p_punch = PlayerCombatContext {
            x_game: goon.x + 60.0,
            y_game: goon.y,
            dir: -1,
            is_attacking: true,
            is_invincible: false,
            state_name: "punch",
        };

        // Taking fatal 10 damage triggers Die, Score(100), Sound(Bam)
        assert!(goon.apply_player_attack(&p_punch, &mut events));
        assert_eq!(goon.health, 0);
        assert_eq!(goon.state, GoonState::Die);
        assert_eq!(
            events,
            vec![
                GoonCombatEvent::Sound(GoonCombatSound::Punch),
                GoonCombatEvent::Score(100),
                GoonCombatEvent::Sound(GoonCombatSound::Bam)
            ]
        );

        let dummy_player = PlayerCombatContext {
            x_game: -500.0,
            y_game: 398.1,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };

        // Tick through flash sequence 1..101
        for _ in 0..60 {
            goon.tick_combat(&dummy_player, &ground, &mut rng, &mut events);
        }
        assert!(goon.active);
        assert_eq!(goon.flash, 60);

        // Advance to 101 ticks -> goon becomes inactive (despawned)
        for _ in 0..41 {
            goon.tick_combat(&dummy_player, &ground, &mut rng, &mut events);
        }
        assert!(!goon.active);
        assert!(!goon.visible);
    }

    #[test]
    fn goon_attack_damages_player() {
        let ground = CollisionMask::level1b();
        let mut rng = GoonRng::new(600);
        let mut goon = BankGoon::from_spawn(BANK_GOON_SPAWNS[0]);
        goon.state = GoonState::Attack;
        goon.frame = 3; // next tick will be frame 4 (strike frame!)
        let mut events = Vec::new();

        let player = PlayerCombatContext {
            x_game: goon.x + 40.0, // within 70px reach
            y_game: goon.y,
            dir: -1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };

        goon.tick_combat(&player, &ground, &mut rng, &mut events);
        assert_eq!(goon.frame, 4);
        assert_eq!(
            events,
            vec![
                GoonCombatEvent::PlayerHurt { damage: 5 },
                GoonCombatEvent::Sound(GoonCombatSound::Metal(1))
            ]
        );
    }

    #[test]
    fn respawn2_triggers_wave2_with_8_enemies_and_50_health() {
        let ground = CollisionMask::level1b();
        let mut squad = BankGoonSquad::new();
        let mut events = Vec::new();

        let player = PlayerCombatContext {
            x_game: -1000.0,
            y_game: 398.1,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };

        // Simulate killing all 10 wave 1 goons
        for goon in &mut squad.goons {
            goon.health = 0;
            goon.active = false;
        }
        squad.e_kills = 10;
        assert_eq!(squad.respawned, 0);

        // Next squad combat tick detects 10 kills and spawns Wave 2!
        squad.tick_combat(&player, &[], &ground, &mut events);
        assert_eq!(squad.respawned, 1);
        assert_eq!(squad.e_kills, 0);
        assert!(squad.is_wave2_active());
        assert_eq!(events, vec![GoonCombatEvent::Wave2Spawned { count: 8 }]);

        // First 8 goons are reactivated with 50 health in Walk state
        for i in 0..8 {
            let g = &squad.goons[i];
            assert!(g.active);
            assert!(g.visible);
            assert_eq!(g.health, 50);
            assert_eq!(g.max_health, 50);
            assert_eq!(g.speed, 9.0);
            assert_eq!(g.state, GoonState::Walk);
            assert!(g.is_wave2);
        }
    }

    #[test]
    fn wave2_completion_triggers_stage_exit() {
        let ground = CollisionMask::level1b();
        let mut squad = BankGoonSquad::new();
        let mut events = Vec::new();

        let player = PlayerCombatContext {
            x_game: -1000.0,
            y_game: 398.1,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };

        squad.respawned = 1; // Wave 2 is active
        squad.e_kills = 8;   // All 8 wave 2 enemies defeated

        squad.tick_combat(&player, &[], &ground, &mut events);
        assert!(squad.is_stage_clear());
        assert_eq!(events, vec![GoonCombatEvent::StageComplete]);
    }
}
