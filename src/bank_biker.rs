//! Bank Biker (`biker1`) entity and AVM1 `bikerAI` state machine.
//!
//! # Reverse Engineering Provenance
//! - Flash Symbol: DefineSprite `#1005` (20 parent frames, 2 child states).
//!   - `drive`: Frame 0, PlaceObject2 child DefineSprite `#981` (2 frames, vibration).
//!   - `fall`: Frame 9, PlaceObject2 child DefineSprite `#1004` (7 frames, crash/flying off).
//! - Placement in parent GAME sprite `#1075`:
//!   - Instance name: `"biker1"`
//!   - Depth: `1129` (`GameDepth(1129)`)
//!   - Initial position: `(-570.25, 375.85)`
//!   - Scale matrix: `1.1585846` (twips 75929 / 65536)
//! - Flash AVM1 bytecode: `bikerAI` (frame 15, body offset 113959, 238 ops).
//!   - Speed: `24.0`, Direction: `dir = 1.0` (moves right).
//!   - Strength: `10` damage to player.
//!   - Loop-around: When `x > player.x + 1200.0`, wraps to `player.x - 1200.0`.
//!   - Engine audio cue: Emits `bike.start()` when `player.x - 750.0 < x && x < player.x - 700.0`.
//!   - Player hit: If player is attacking, biker enters `fall`, awards +100 score, plays `bam`.
//!     If player is not attacking, deals 10 damage to player, plays `kick`.
//!   - Batarang hit: Biker enters `fall`, awards +100 score.
//!   - Fall & Despawn: Advances clip `#1004` frames 0..6 (clamped), increments `flash`.
//!     Flashes when `flash > 50` (visible 2 out of 4 frames), despawns when `flash > 100`.

use crate::stage::{GameDepth, GamePos};

pub const BIKER_SPEED: f32 = 24.0;
pub const BIKER_STRENGTH: u8 = 10;
pub const BIKER_DEPTH: GameDepth = GameDepth(1129);
pub const BIKER_INITIAL_X: f32 = -570.25;
pub const BIKER_INITIAL_Y: f32 = 375.85;
pub const BIKER_SCALE: f32 = 1.1585846;

pub const BIKER_DRIVE_FRAMES: usize = 2; // DefineSprite #981
pub const BIKER_FALL_FRAMES: usize = 7;  // DefineSprite #1004

/// Biker animation and behavioral state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BikerState {
    Drive,
    Fall,
}

/// Sound events emitted by the biker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BikerSound {
    Bike,
    Kick,
    Bam,
}

/// Combat and interaction events emitted by the biker during a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BikerEvent {
    Sound(BikerSound),
    Score(u32),
    PlayerHurt { damage: u8 },
}

/// Bank street motorcycle enemy (`biker1`).
#[derive(Debug, Clone)]
pub struct BankBiker {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub dir: f32,
    pub strength: u8,
    pub state: BikerState,
    pub depth: GameDepth,
    pub scale: f32,
    pub frame: usize,
    pub flash: u32,
    pub visible: bool,
    pub bike_sfx_played: bool,
    pub active: bool,
    pub alive: bool,
}

impl Default for BankBiker {
    fn default() -> Self {
        Self::new()
    }
}

impl BankBiker {
    /// Constructs the biker with canonical Flash placement and attributes.
    pub fn new() -> Self {
        Self {
            x: BIKER_INITIAL_X,
            y: BIKER_INITIAL_Y,
            speed: BIKER_SPEED,
            dir: 1.0,
            strength: BIKER_STRENGTH,
            state: BikerState::Drive,
            depth: BIKER_DEPTH,
            scale: BIKER_SCALE,
            frame: 0,
            flash: 0,
            visible: true,
            bike_sfx_played: false,
            active: true,
            alive: true,
        }
    }

    /// Resets the biker to initial spawn condition.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Returns current GAME-local position.
    pub fn pos(&self) -> GamePos {
        GamePos::new(self.x, self.y)
    }

    /// Transitions to a new state and resets frame counters.
    pub fn set_state(&mut self, new_state: BikerState) {
        if self.state != new_state {
            self.state = new_state;
            self.frame = 0;
            if new_state == BikerState::Fall {
                self.alive = false;
                self.flash = 0;
            }
        }
    }

    /// Source `checkPlayer(this, dir, -40)` vertical alignment: biker `self.y`
    /// sits at `375.85` (wheels/floor) while `player_y` is Batman's waist
    /// (`323.25` when standing on the `398.25` street floor, hitbox `[player_y - 45, player_y + 75]`).
    #[inline]
    fn hits_player_y(&self, player_y: f32) -> bool {
        let probe_y = self.y - 40.0;
        probe_y >= player_y - 45.0 && probe_y <= player_y + 75.0
    }

    /// Tests and applies swept batarang segment collision `(prev_x, cur_x, y)`.
    pub fn apply_batarang_segment_hit(
        &mut self,
        bat_prev_x: f32,
        bat_x: f32,
        bat_y: f32,
        events: &mut Vec<BikerEvent>,
    ) -> bool {
        if !self.active || self.state == BikerState::Fall {
            return false;
        }

        let dy = bat_y - self.y;
        if crate::bank_goons::segment_overlaps_x(bat_prev_x, bat_x, self.x, 45.0)
            && dy >= -135.0
            && dy <= 20.0
        {
            self.set_state(BikerState::Fall);
            events.push(BikerEvent::Score(100));
            events.push(BikerEvent::Sound(BikerSound::Bam));
            return true;
        }

        false
    }

    /// Tests and applies batarang collision. Returns true if hit connected.
    pub fn apply_batarang_hit(
        &mut self,
        bat_x: f32,
        bat_y: f32,
        events: &mut Vec<BikerEvent>,
    ) -> bool {
        self.apply_batarang_segment_hit(bat_x, bat_x, bat_y, events)
    }

    /// Tests and applies Batman melee attack collision. Returns true if hit connected.
    pub fn apply_player_attack(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_attacking: bool,
        events: &mut Vec<BikerEvent>,
    ) -> bool {
        if !self.active || self.state == BikerState::Fall || !player_attacking {
            return false;
        }

        let dx = (player_x - self.x).abs();
        if dx <= 50.0 && self.hits_player_y(player_y) {
            self.set_state(BikerState::Fall);
            events.push(BikerEvent::Score(100));
            events.push(BikerEvent::Sound(BikerSound::Bam));
            return true;
        }

        false
    }

    /// Executes one tick of `bikerAI` with swept batarang segments `(prev_x, cur_x, y)`
    /// and returns the index of the batarang consumed on hit, if any.
    pub fn tick_swept(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_attacking: bool,
        player_hurt: bool,
        batarang_segments: &[(f32, f32, f32)],
        events: &mut Vec<BikerEvent>,
    ) -> Option<usize> {
        if !self.active {
            return None;
        }

        match self.state {
            BikerState::Drive => {
                // Advance engine vibration child animation (DefineSprite #981: 2 frames)
                self.frame = (self.frame + 1) % BIKER_DRIVE_FRAMES;

                // Move forward horizontally at 24.0 units/tick
                self.x += self.speed * self.dir;

                // Loop around when ahead of player: x > player.x + 1200
                if self.x > player_x + 1200.0 {
                    self.x = player_x - 1200.0;
                    self.visible = true;
                    self.flash = 0;
                    self.bike_sfx_played = false;
                } else if !self.bike_sfx_played
                    && self.x > player_x - 750.0
                    && self.x < player_x - 700.0
                {
                    // Audio cue trigger: approach sound when approaching from behind
                    self.bike_sfx_played = true;
                    events.push(BikerEvent::Sound(BikerSound::Bike));
                }

                // Original bikerAI evaluates player contact before checkBatarang
                // (RE-20261126-049/053). Keep native geometry and hurt gate.
                let dx = (player_x - self.x).abs();
                if dx <= 45.0 && self.hits_player_y(player_y) {
                    if player_attacking {
                        self.set_state(BikerState::Fall);
                        events.push(BikerEvent::Score(100));
                        events.push(BikerEvent::Sound(BikerSound::Bam));
                    } else if !player_hurt {
                        events.push(BikerEvent::Sound(BikerSound::Kick));
                        events.push(BikerEvent::PlayerHurt {
                            damage: self.strength,
                        });
                    }
                }

                // Preserve the existing Fall guard in apply_batarang_segment_hit:
                // a melee defeat cannot award another score or consume a shot.
                // A surviving Drive biker can still be defeated by a projectile
                // in the same callback, after any contact damage event.
                for (idx, &(prev_bx, bx, by)) in batarang_segments.iter().enumerate() {
                    if self.apply_batarang_segment_hit(prev_bx, bx, by, events) {
                        return Some(idx);
                    }
                }
            }
            BikerState::Fall => {
                // Advance crash child animation (DefineSprite #1004: 7 frames), clamped to last frame
                self.frame = (self.frame + 1).min(BIKER_FALL_FRAMES.saturating_sub(1));

                // Despawn flash counter
                self.flash += 1;
                if self.flash > 50 {
                    self.visible = (self.flash % 4) >= 2;
                }
                if self.flash > 100 {
                    self.visible = false;
                    self.active = false;
                }
            }
        }
        None
    }

    /// Executes one tick of `bikerAI` matching Flash AVM1 bytecode.
    pub fn tick(
        &mut self,
        player_x: f32,
        player_y: f32,
        player_attacking: bool,
        player_hurt: bool,
        batarangs: &[(f32, f32, bool)],
        events: &mut Vec<BikerEvent>,
    ) {
        let segments: Vec<(f32, f32, f32)> = batarangs
            .iter()
            .filter_map(|&(bx, by, active)| active.then_some((bx, bx, by)))
            .collect();
        let _ = self.tick_swept(player_x, player_y, player_attacking, player_hurt, &segments, events);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biker_initialization_and_metadata() {
        let biker = BankBiker::new();
        assert_eq!(biker.x, BIKER_INITIAL_X);
        assert_eq!(biker.y, BIKER_INITIAL_Y);
        assert_eq!(biker.speed, 24.0);
        assert_eq!(biker.dir, 1.0);
        assert_eq!(biker.strength, 10);
        assert_eq!(biker.state, BikerState::Drive);
        assert_eq!(biker.depth, GameDepth(1129));
        assert!((biker.scale - 1.1585846).abs() < 1e-6);
        assert_eq!(biker.frame, 0);
        assert_eq!(biker.flash, 0);
        assert!(biker.visible);
        assert!(biker.active);
        assert!(biker.alive);
    }

    #[test]
    fn biker_drive_movement_and_animation_wrap() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();

        // 1 tick: advances position by 24.0 and frame to 1
        biker.tick(0.0, 375.85, false, false, &[], &mut events);
        assert_eq!(biker.x, BIKER_INITIAL_X + 24.0);
        assert_eq!(biker.frame, 1);

        // 2nd tick: advances position by another 24.0 and wraps frame back to 0 (2-frame cycle)
        events.clear();
        biker.tick(0.0, 375.85, false, false, &[], &mut events);
        assert_eq!(biker.x, BIKER_INITIAL_X + 48.0);
        assert_eq!(biker.frame, 0);
    }

    #[test]
    fn biker_loops_around_ahead_of_player() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();
        let player_x = 0.0;

        // Position biker just beyond player.x + 1200.0
        biker.x = player_x + 1201.0;
        biker.bike_sfx_played = true;

        biker.tick(player_x, 375.85, false, false, &[], &mut events);
        // Wraps to player.x - 1200.0
        assert_eq!(biker.x, player_x - 1200.0);
        assert!(biker.visible);
        assert!(!biker.bike_sfx_played);
        assert_eq!(biker.state, BikerState::Drive);
    }

    #[test]
    fn biker_engine_audio_cue_trigger() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();
        let player_x = 0.0;

        // Outside audio range
        biker.x = -800.0;
        biker.tick(player_x, 375.85, false, false, &[], &mut events);
        assert!(events.is_empty());
        assert!(!biker.bike_sfx_played);

        // Inside audio range: player_x - 750 < x < player_x - 700 (-750 < (-740 + 24 = -716) < -700)
        biker.x = -740.0;
        events.clear();
        biker.tick(player_x, 375.85, false, false, &[], &mut events);
        assert_eq!(events, vec![BikerEvent::Sound(BikerSound::Bike)]);
        assert!(biker.bike_sfx_played);

        // Next tick inside range does not retrigger sound
        events.clear();
        biker.tick(player_x, 375.85, false, false, &[], &mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn biker_damages_player_when_not_attacking() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();
        let player_x = 0.0;
        let player_y = 375.85;

        // Place biker close to player
        biker.x = 10.0;
        biker.tick(player_x, player_y, false, false, &[], &mut events);

        assert_eq!(
            events,
            vec![
                BikerEvent::Sound(BikerSound::Kick),
                BikerEvent::PlayerHurt { damage: 10 }
            ]
        );
        // Biker stays in Drive state
        assert_eq!(biker.state, BikerState::Drive);
        assert!(biker.alive);
    }

    #[test]
    fn biker_defeated_by_batarang_or_player_attack() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();
        let player_x = 0.0;
        let player_y = 375.85;

        // Player attacks biker
        biker.x = 15.0;
        biker.tick(player_x, player_y, true, false, &[], &mut events);

        assert_eq!(
            events,
            vec![
                BikerEvent::Score(100),
                BikerEvent::Sound(BikerSound::Bam)
            ]
        );
        assert_eq!(biker.state, BikerState::Fall);
        assert!(!biker.alive);
        assert_eq!(biker.frame, 0);

        // Advance fall frames: clips to 6
        for _ in 0..10 {
            events.clear();
            biker.tick(player_x, player_y, false, false, &[], &mut events);
        }
        assert_eq!(biker.frame, 6); // Clamped to frame 6 (7-frame clip)

        // Advance flash to > 100 for despawn
        biker.flash = 100;
        events.clear();
        biker.tick(player_x, player_y, false, false, &[], &mut events);
        assert!(!biker.visible);
        assert!(!biker.active);
    }

    #[test]
    fn biker_defeated_by_batarang_projectile() {
        let mut biker = BankBiker::new();
        let mut events = Vec::new();

        biker.x = 500.0;
        biker.y = 375.85;

        // Batarang hits biker: bx = 510.0, by = 350.0
        let batarangs = [(510.0, 350.0, true)];
        biker.tick(0.0, 375.85, false, false, &batarangs, &mut events);

        assert_eq!(
            events,
            vec![
                BikerEvent::Score(100),
                BikerEvent::Sound(BikerSound::Bam)
            ]
        );
        assert_eq!(biker.state, BikerState::Fall);
        assert!(!biker.alive);
    }

    #[test]
    fn standing_batman_on_bank_street_floor_interacts_with_biker_melee_contact_and_batarang() {
        // Level 1B street floor sits at y_game = 398.25 -> standing Batman player.y_game = 323.25
        // and standing batarang shot sits at 303.25 (323.25 - 20.0).
        const STANDING_Y: f32 = 323.25;
        const STANDING_BATARANG_Y: f32 = 303.25;

        // 1. Standing contact damages Batman when not attacking
        let mut biker = BankBiker::new();
        let mut events = Vec::new();
        biker.x = 6.0; // advances +24 -> 30.0 (within 45px of player at 0.0)
        biker.tick(0.0, STANDING_Y, false, false, &[], &mut events);
        assert_eq!(
            events,
            vec![
                BikerEvent::Sound(BikerSound::Kick),
                BikerEvent::PlayerHurt { damage: 10 },
            ]
        );

        // 2. Standing melee defeats biker
        let mut biker2 = BankBiker::new();
        events.clear();
        biker2.x = 6.0;
        biker2.tick(0.0, STANDING_Y, true, false, &[], &mut events);
        assert_eq!(biker2.state, BikerState::Fall);
        assert!(events.contains(&BikerEvent::Score(100)));

        // 3. Standing batarang defeats biker
        let mut biker3 = BankBiker::new();
        events.clear();
        biker3.x = 476.0; // advances +24 -> 500.0
        biker3.tick(0.0, STANDING_Y, false, false, &[(505.0, STANDING_BATARANG_Y, true)], &mut events);
        assert_eq!(biker3.state, BikerState::Fall);
        assert!(events.contains(&BikerEvent::Score(100)));

        // 4. Jumping well above biker avoids contact
        let mut biker4 = BankBiker::new();
        events.clear();
        biker4.x = 6.0;
        biker4.tick(0.0, 220.0, false, false, &[], &mut events);
        assert!(events.is_empty());
        assert_eq!(biker4.state, BikerState::Drive);
    }
}
