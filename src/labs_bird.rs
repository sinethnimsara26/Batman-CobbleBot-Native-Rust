//! Level 2A/Labs Eagle (`#1393`) and `_global.birdAI` state machine.
//!
//! Extracted from canonical `cobblebot.swf` (`_global.birdAI`, offset 142373,
//! and sprite `#1393` with 41 baked true child frames across 5 source states).
//! Note: in `Level2A` root frame 23, the original SWF init checks
//! `n.substring(0, 5) == "eagle_BAK"`, leaving `eagle1` (`ai_attached=false`)
//! animating idle in-place unless `_global.birdAI` is attached.

use crate::bank_goons::{GoonRng, PlayerCombatContext};
use crate::stage::LabsEnemyPlacement;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BirdState {
    Fly,
    Glide,
    Attack,
    Hurt,
    Die,
}

impl BirdState {
    /// `(start_frame_inclusive, frame_count, loops)` inside `eagle_frames.bin` (`41` frames).
    pub const fn clip_range(self) -> (usize, usize, bool) {
        match self {
            Self::Fly => (0, 10, true),     // #1377 (10f)
            Self::Glide => (10, 10, true),  // #1379 (10f)
            Self::Attack => (20, 12, false), // #1384 (12f, f11 -> fly)
            Self::Hurt => (32, 5, false),   // #1387 (5f, f4 -> fly)
            Self::Die => (37, 4, false),    // #1392 (4f, stop)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BirdEvent {
    Score(u32),
    PlayerHurt { damage: i32 },
    EagleSfx,
    KickSfx,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabsEagle {
    pub name: &'static str,
    pub x: f32,
    pub y: f32,
    pub dir: i8,
    pub state: BirdState,
    pub frame: usize,
    pub health: i32,
    pub speed: f32,
    pub turn_back: bool,
    pub fly_count: u32,
    pub attack_count: u32,
    pub die_count: u32,
    pub dy: f32,
    pub bird_sfx: bool,
    pub scale: f32,
    pub ai_attached: bool,
}

impl LabsEagle {
    /// Creates an eagle from a `Level2A` SWF placement (`ai_attached = false` due to `eagle_BAK`).
    pub fn from_labs2a(spawn: LabsEnemyPlacement, ai_attached: bool) -> Self {
        Self {
            name: spawn.name,
            x: spawn.x,
            y: spawn.y,
            dir: -1,
            state: BirdState::Fly,
            frame: 0,
            health: 30,
            speed: 5.0,
            turn_back: false,
            fly_count: 0,
            attack_count: 0,
            die_count: 0,
            dy: 0.0,
            bird_sfx: false,
            scale: spawn.sx,
            ai_attached,
        }
    }

    pub fn set_state(&mut self, next: BirdState) {
        if self.state != next {
            self.state = next;
            self.frame = 0;
        }
    }

    pub fn pack_frame_index(&self) -> usize {
        let (start, count, _) = self.state.clip_range();
        start + self.frame.min(count.saturating_sub(1))
    }

    /// Advances either inert clip animation (when `!ai_attached`) or exact `_global.birdAI`.
    pub fn tick(
        &mut self,
        player: &PlayerCombatContext,
        batarangs: &[(f32, f32)],
        rng: &mut GoonRng,
        events: &mut Vec<BirdEvent>,
    ) {
        let (_, count, loops) = self.state.clip_range();
        if loops {
            self.frame = (self.frame + 1) % count.max(1);
        } else if self.frame + 1 < count {
            self.frame += 1;
        } else if matches!(self.state, BirdState::Attack | BirdState::Hurt) {
            self.set_state(BirdState::Fly);
        }

        if !self.ai_attached {
            return;
        }

        let range = (self.x - player.x_game).abs();
        let dir_i = self.dir as i32;
        if range > 180.0 && range < 240.0 && self.state != BirdState::Die {
            if self.turn_back {
                self.turn_back = false;
                self.set_state(BirdState::Fly);
                if self.x > player.x_game {
                    self.dir = -1;
                } else if self.x < player.x_game {
                    self.dir = 1;
                }
            }
        } else if range < 180.0
            && player.is_attacking
            && ((dir_i == 1 && player.dir == 1 && self.x > player.x_game)
                || (dir_i == -1 && player.dir == -1 && self.x < player.x_game))
        {
            self.dir = -self.dir;
        } else if range > 500.0 {
            if self.x > player.x_game {
                self.dir = -1;
            } else if self.x < player.x_game {
                self.dir = 1;
            }
        } else if range > 320.0 && self.state != BirdState::Die {
            self.set_state(BirdState::Glide);
            self.attack_count = 0;
        }

        match self.state {
            BirdState::Fly => {
                let r_x = (rng.next_f32() * self.speed).floor();
                self.x += (r_x + self.speed) * (self.dir as f32);
                let r9 = (rng.next_f32() * 9.0).floor();
                let r6 = (rng.next_f32() * 6.0).floor();
                self.y -= (r9 - r6) + (self.y - player.y_game + 45.0) / 16.0;
                self.fly_count += 1;
                if self.fly_count > 10 {
                    self.attack_count = 0;
                }
                if self.fly_count > 30 {
                    self.set_state(BirdState::Glide);
                    self.fly_count = 0;
                }
            }
            BirdState::Glide => {
                let r_x = (rng.next_f32() * self.speed).floor();
                self.x += (r_x + self.speed * 2.0 + 5.0) * (self.dir as f32);
                let r3 = (rng.next_f32() * 3.0).floor();
                self.y -= (self.y - player.y_game + 65.0) / 16.0 + r3;
                self.fly_count += 1;
                if self.fly_count > 30 {
                    self.set_state(BirdState::Fly);
                    self.fly_count = 0;
                }
            }
            BirdState::Die => {
                self.y += self.dy;
                self.dy += 1.0;
                self.die_count += 1;
                if self.die_count > 200 {
                    self.x = player.x_game + 1000.0 * (player.dir as f32);
                    self.y = player.y_game - 1000.0;
                    self.health = 30;
                    self.set_state(BirdState::Glide);
                    self.fly_count = 0;
                    self.attack_count = 0;
                    self.bird_sfx = false;
                }
            }
            BirdState::Attack | BirdState::Hurt => {}
        }

        if (self.x < player.x_game + 700.0 || self.x > player.x_game - 700.0) && !self.bird_sfx {
            self.bird_sfx = true;
            events.push(BirdEvent::EagleSfx);
        }

        // AVM1 `checkPlayer(this, dir, 40)`
        let hits_player = (self.x + (self.dir as f32) * 25.0 - player.x_game).abs() <= 45.0
            && (self.y - player.y_game).abs() <= 40.0;
        if hits_player {
            if player.is_attacking && self.state != BirdState::Hurt && self.state != BirdState::Die {
                events.push(BirdEvent::KickSfx);
                self.take_damage(10, events);
            } else if player.state_name != "hurt"
                && !player.is_invincible
                && self.state != BirdState::Hurt
                && self.state != BirdState::Die
                && self.attack_count == 0
            {
                if self.state == BirdState::Fly {
                    self.set_state(BirdState::Attack);
                    self.fly_count = 0;
                    events.push(BirdEvent::KickSfx);
                    events.push(BirdEvent::PlayerHurt { damage: 10 });
                }
                self.attack_count += 1;
            }
        }

        // AVM1 `checkBatarang(this)`
        let hit_bat = batarangs
            .iter()
            .any(|&(bx, by)| (self.x - bx).abs() <= 40.0 && (self.y - by).abs() <= 45.0);
        if hit_bat && self.state != BirdState::Hurt && self.state != BirdState::Die {
            self.take_damage(10, events);
        }
    }

    fn take_damage(&mut self, amount: i32, events: &mut Vec<BirdEvent>) {
        self.health -= amount;
        if self.health > 0 {
            self.set_state(BirdState::Hurt);
            events.push(BirdEvent::KickSfx);
        } else {
            self.set_state(BirdState::Die);
            self.die_count = 0;
            self.dy = 3.0;
            events.push(BirdEvent::Score(100));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::labs2a;

    #[test]
    fn labs2a_eagle_spawns_inert_under_eagle_bak_and_bird_ai_tracks_hurt_die_respawn() {
        let mut inert = LabsEagle::from_labs2a(labs2a::EAGLES[0], false);
        assert_eq!(inert.x, 1388.0);
        assert_eq!(inert.y, -163.8);
        assert!(!inert.ai_attached);

        let mut rng = GoonRng::new(42);
        let mut events = Vec::new();
        let player = PlayerCombatContext {
            x_game: 1388.0,
            y_game: -163.8,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };
        inert.tick(&player, &[], &mut rng, &mut events);
        assert_eq!((inert.x, inert.y), (1388.0, -163.8));
        assert!(events.is_empty());

        // Active _global.birdAI test: seek/attack/batarang kill/respawn
        let mut active = LabsEagle::from_labs2a(labs2a::EAGLES[0], true);
        active.x = 1950.0;
        active.y = 200.0;
        let far_player = PlayerCombatContext {
            x_game: 1000.0,
            y_game: 200.0,
            dir: 1,
            is_attacking: false,
            is_invincible: false,
            state_name: "stand",
        };
        active.tick(&far_player, &[], &mut rng, &mut events);
        assert_eq!(active.dir, -1);
        assert!(events.contains(&BirdEvent::EagleSfx));

        // Batarang hits 3 times (30 HP -> die -> +100 score -> respawn after 200 ticks)
        events.clear();
        for _ in 0..3 {
            active.set_state(BirdState::Fly);
            active.tick(&far_player, &[(active.x, active.y)], &mut rng, &mut events);
        }
        assert_eq!(active.state, BirdState::Die);
        assert!(events.contains(&BirdEvent::Score(100)));

        for _ in 0..202 {
            active.tick(&far_player, &[], &mut rng, &mut events);
        }
        assert_eq!(active.state, BirdState::Glide);
        assert_eq!(active.health, 30);
        assert!((active.x - (far_player.x_game + 1000.0)).abs() < 150.0);
    }
}
