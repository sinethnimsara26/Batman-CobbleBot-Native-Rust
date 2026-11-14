use png::{ColorType, Decoder, Transformations};
use std::io::Cursor;

pub struct CollisionMask {
    pub origin_x: i32,
    pub origin_y: i32,
    pub w: usize,
    pub h: usize,
    words: Box<[u64]>,
}

impl CollisionMask {
    pub fn level1a() -> Self {
        decode_mask(
            include_bytes!("../reverse-engineering/generated/collision_masks/level1a_ground.png"),
            -5539,
            -1929,
        )
    }

    /// B07: original root20/level #783 named ground #782, in LEVEL-local
    /// pixels. The baked mask comes from the SHA-locked FWS at build time.
    /// This constructor is dormant: Game::new() still selects only Level1A.
    #[allow(dead_code)]
    pub fn level1b() -> Self {
        decode_mask(
            include_bytes!("../reverse-engineering/generated/collision_masks/level1b_ground.png"),
            -11196,
            -886,
        )
    }

    /// B14: original root21/level #1095 named ground #1094 (Penguin arena),
    /// in LEVEL-local pixels. The baked mask comes from the SHA-locked FWS at build time.
    #[allow(dead_code)]
    pub fn level1c() -> Self {
        decode_mask(
            include_bytes!("../reverse-engineering/generated/collision_masks/level1c_penguin_ground.png"),
            -1731,
            -549,
        )
    }

    /// B20: original root23/level #1317 named ground #1316 (Labs 2A),
    /// in LEVEL-local pixels. The baked mask comes from the SHA-locked FWS at build time.
    pub fn level2a() -> Self {
        decode_mask(
            include_bytes!("../reverse-engineering/generated/collision_masks/level2a_ground.png"),
            -1542,
            -2307,
        )
    }

    /// B27: original root25/level #1485 named ground #1484 (Labs 2B),
    /// in LEVEL-local pixels. The baked mask comes from the SHA-locked FWS at build time.
    pub fn level2b() -> Self {
        decode_mask(
            include_bytes!("../reverse-engineering/generated/collision_masks/level2b_ground.png"),
            -1542,
            -2307,
        )
    }

    /// Returns the runtime collision mask for the given gameplay stage.
    pub fn for_stage(stage_id: crate::stage::StageId) -> Self {
        match stage_id {
            crate::stage::StageId::Level1A => Self::level1a(),
            crate::stage::StageId::Level1B => Self::level1b(),
            crate::stage::StageId::Level1C => Self::level1c(),
            crate::stage::StageId::Level2A => Self::level2a(),
            crate::stage::StageId::Level2B => Self::level2b(),
        }
    }

    #[inline]
    pub fn solid(&self, x: f32, y: f32) -> bool {
        let px = x.floor() as i32 - self.origin_x;
        let py = y.floor() as i32 - self.origin_y;
        if px >= 0 && py >= 0 && (px as usize) < self.w && (py as usize) < self.h {
            let idx = (py as usize) * self.w + (px as usize);
            (self.words[idx >> 6] & (1u64 << (idx & 63))) != 0
        } else {
            false
        }
    }

    #[inline]
    pub fn solid_level_pos(&self, pos: crate::stage::LevelPos) -> bool {
        self.solid(pos.x, pos.y)
    }

    pub fn check_ground(&self, x: f32, y: &mut f32, height: f32, dy: &mut f32, _loop_count: u8) -> bool {
        if !self.solid(x, *y + height) {
            return false;
        }
        *y -= 1.0;
        *dy = 0.0;
        for _ in 0..15 {
            if self.solid(x, *y + height) {
                *y -= 1.0;
            } else {
                break;
            }
        }
        true
    }

    pub fn check_ceiling(&self, x: f32, y: &mut f32, height: f32, dy: &mut f32, _loop_count: u8) -> bool {
        if !self.solid(x, *y - height) {
            return false;
        }
        *y += 1.0;
        *dy = 0.0;
        for _ in 0..15 {
            if self.solid(x, *y - height) {
                *y += 1.0;
            } else {
                break;
            }
        }
        true
    }

    #[inline]
    pub fn check_wall_clear(&self, x: f32, y: f32, height: f32, dx: f32) -> bool {
        !self.solid(x + dx, y + height * 0.5)
    }

    #[cfg(test)]
    pub fn memory_usage_bytes(&self) -> usize {
        self.words.len() * std::mem::size_of::<u64>()
    }
}

fn decode_mask(bytes: &[u8], origin_x: i32, origin_y: i32) -> CollisionMask {
    let mut dec = Decoder::new(Cursor::new(bytes));
    dec.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0u8; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).unwrap();
    let data = &buf[..info.buffer_size()];
    let w = info.width as usize;
    let h = info.height as usize;
    let total_pixels = w * h;
    let word_count = (total_pixels + 63) / 64;
    let mut words = vec![0u64; word_count];

    let mut set_bit = |idx: usize, is_solid: bool| {
        if is_solid {
            words[idx >> 6] |= 1u64 << (idx & 63);
        }
    };

    match info.color_type {
        ColorType::Grayscale => {
            for (idx, &v) in data[..total_pixels].iter().enumerate() {
                set_bit(idx, v >= 128);
            }
        }
        ColorType::GrayscaleAlpha => {
            for (idx, p) in data.chunks_exact(2).take(total_pixels).enumerate() {
                set_bit(idx, p[0] >= 128 && p[1] >= 8);
            }
        }
        ColorType::Rgb => {
            for (idx, p) in data.chunks_exact(3).take(total_pixels).enumerate() {
                set_bit(idx, (p[0] | p[1] | p[2]) != 0);
            }
        }
        ColorType::Rgba => {
            for (idx, p) in data.chunks_exact(4).take(total_pixels).enumerate() {
                set_bit(idx, (p[0] | p[1] | p[2]) != 0 && p[3] >= 8);
            }
        }
        ColorType::Indexed => unreachable!(),
    }
    CollisionMask {
        origin_x,
        origin_y,
        w,
        h,
        words: words.into_boxed_slice(),
    }
}

#[cfg(test)]
mod bank_street_source_probes {
    use super::CollisionMask;
    use crate::stage::StageId;

    #[test]
    fn bank_ground_dimensions_and_spawn_space_match_source() {
        let ground = CollisionMask::level1b();
        assert_eq!(
            (ground.origin_x, ground.origin_y, ground.w, ground.h),
            (-11196, -886, 12102, 1413)
        );
        let stage = StageId::Level1B.descriptor();
        assert_eq!(stage.ground_symbol, 782);
        assert_eq!(stage.source_player_direction, -1);
        assert_eq!(stage.player_spawn_game_local, (528.15, -18.75));
        let (px, py) = stage.player_spawn_level_local();
        let spawn_pos = stage.player_spawn_level_pos();
        assert_eq!((spawn_pos.x, spawn_pos.y), (px, py));
        assert_eq!(ground.solid_level_pos(spawn_pos), ground.solid(px, py));
        assert!((px + 506.5).abs() < 0.001 && (py + 258.0).abs() < 0.001);
        assert!(!ground.solid(-11197.0, 0.0));
        assert!(!ground.solid(906.0, 0.0));
        assert!(!ground.solid(0.0, -887.0));
        assert!(!ground.solid(0.0, 527.0));
    }

    /// Use the original baked *geometry*, not arbitrary replacement slabs.
    /// Find a thick, exposed floor surface near source player/street positions.
    /// Probes run with 75px feet, 50px ceiling and the native 16px push-out.
    #[test]
    fn bank_street_walk_jump_fall_and_wall_probes_use_original_ground() {
        let ground = CollisionMask::level1b();
        let mut site = None;
        for x in [-506.0, -400.0, -600.0, -1000.0, -3000.0, -6000.0] {
            for y in -700..490 {
                let fy = y as f32;
                if ground.solid(x, fy)
                    && ground.solid(x, fy + 12.0)
                    && !ground.solid(x, fy - 1.0)
                    && !ground.solid(x, fy - 90.0)
                {
                    site = Some((x, fy));
                    break;
                }
            }
            if site.is_some() {
                break;
            }
        }
        let (x, floor) = site.expect("source #782 needs an exposed thick Bank Street floor");
        let mut y = floor - 75.0 + 2.0;
        let mut dy = 8.0;
        assert!(
            ground.check_ground(x, &mut y, 75.0, &mut dy, 0),
            "2px foot penetration must be resolved"
        );
        assert!(
            !ground.solid(x, y + 75.0),
            "feet must not remain inside source ground"
        );
        assert_eq!(dy, 0.0);

        // Original player logic tests wall at half-height before x movement.
        // If a source wall blocks a step, the point probe MUST prevent it;
        // if clear, advancing five pixels may be followed by a ledge fall.
        let mut walk_x = x;
        for _ in 0..12 {
            let can_move = ground.check_wall_clear(walk_x, y, 75.0, -5.0);
            if can_move {
                walk_x -= 5.0;
            }
            assert!(can_move || !ground.check_wall_clear(walk_x, y, 75.0, -5.0));
            assert!(
                !ground.solid(walk_x, y + 37.5),
                "native midpoint wall probe leaked into a source wall"
            );
        }

        // A vertical 25Hz jump / gravity / landing probe on the original
        // source mask, intentionally NOT a playable stage or AI test.
        let mut pos_y = y;
        let mut velocity = -42.0;
        let mut rose = false;
        let mut landed = false;
        for tick in 0..40 {
            if !ground.check_ceiling(x, &mut pos_y, 50.0, &mut velocity, 0) {
                pos_y += velocity;
            }
            if pos_y < y - 2.0 {
                rose = true;
            }
            if ground.check_ground(x, &mut pos_y, 75.0, &mut velocity, 0) {
                if tick > 0 {
                    landed = true;
                    break;
                }
            } else {
                velocity += 5.0;
            }
        }
        assert!(rose, "jump must lift off original Bank Street floor");
        assert!(landed, "original source ground should catch the falling probe");
    }

    #[test]
    fn collision_mask_bitpack_memory_savings_and_queries() {
        let g1a = CollisionMask::level1a();
        let g1b = CollisionMask::level1b();
        let raw_pixels_1a = g1a.w * g1a.h;
        let raw_pixels_1b = g1b.w * g1b.h;
        let mem_1a = g1a.memory_usage_bytes();
        let mem_1b = g1b.memory_usage_bytes();

        // 8x bit-pack ratio: words buffer uses 1 bit per pixel (ceil(N/64)*8 bytes)
        assert!(
            mem_1a <= (raw_pixels_1a / 8) + 8,
            "1A memory {mem_1a} must be bit-packed compared to {raw_pixels_1a} bytes"
        );
        assert!(
            mem_1b <= (raw_pixels_1b / 8) + 8,
            "1B memory {mem_1b} must be bit-packed compared to {raw_pixels_1b} bytes"
        );
        assert!(
            mem_1a < raw_pixels_1a / 7,
            "1A bit-packing must achieve at least 7x reduction"
        );
        assert!(
            mem_1b < raw_pixels_1b / 7,
            "1B bit-packing must achieve at least 7x reduction"
        );

        // Out-of-bounds queries return false without panicking
        assert!(!g1a.solid(g1a.origin_x as f32 - 10.0, g1a.origin_y as f32));
        assert!(!g1a.solid(g1a.origin_x as f32 + g1a.w as f32 + 10.0, g1a.origin_y as f32));
        assert!(!g1a.solid(g1a.origin_x as f32, g1a.origin_y as f32 - 10.0));
        assert!(!g1a.solid(g1a.origin_x as f32, g1a.origin_y as f32 + g1a.h as f32 + 10.0));
    }

    #[test]
    fn check_ground_and_ceiling_iterative_loop_bounds() {
        let ground = CollisionMask::level1b();
        // Non-colliding probe on known air coordinate returns false and does not mutate y or dy
        let mut y = 0.0;
        let mut dy = 5.0;
        assert!(!ground.check_ground(-11197.0, &mut y, 0.0, &mut dy, 0));
        assert_eq!(y, 0.0);
        assert_eq!(dy, 5.0);

        assert!(!ground.check_ceiling(-11197.0, &mut y, 0.0, &mut dy, 0));
        assert_eq!(y, 0.0);
        assert_eq!(dy, 5.0);
    }
}

#[cfg(test)]
mod penguin_arena_source_probes {
    use super::CollisionMask;
    use crate::stage::StageId;

    #[test]
    fn penguin_ground_dimensions_and_spawn_space_match_source() {
        let ground = CollisionMask::level1c();
        assert_eq!((ground.origin_x, ground.origin_y), (-1731, -549));
        assert_eq!((ground.w, ground.h), (2813, 926));
        let stage = StageId::Level1C.descriptor();
        let spawn = stage.player_spawn_level_pos();
        assert!((spawn.x - (-1363.6)).abs() < 0.05 && (spawn.y - 42.0).abs() < 0.05);
        assert!(
            !ground.solid_level_pos(spawn),
            "level1c spawn origin should be air above arena floor"
        );
        // Floor is at level_y = 124 (game_y = 363.25), player height = 75 -> resting origin y ~ 49.
        assert!(
            ground.solid(spawn.x, 124.0),
            "level1c ground floor must be solid directly under player spawn"
        );
        assert!(
            !ground.solid(spawn.x, 123.0),
            "level1c arena air immediately above floor at level_y = 123"
        );
    }

    #[test]
    fn penguin_arena_floor_and_boundary_walls_match_source_bounds() {
        let ground = CollisionMask::level1c();
        // Arena floor at level_y = 124 spans open air between left and right walls (~gx 0..2300).
        for gx in [122.0_f32, 500.0, 1035.65, 1500.0, 2000.0] {
            let lx = gx - 1485.65;
            assert!(!ground.solid(lx, 0.0), "arena interior air at gx={gx}");
            assert!(ground.solid(lx, 124.0), "arena floor solid at gx={gx}");
        }
        // Enclosed arena walls at gx < 0 and gx > 2300 (chest level_y = 0)
        assert!(ground.solid(-1500.0, 0.0), "left arena wall at lx=-1500");
        assert!(ground.solid(850.0, 0.0), "right arena wall at lx=850");
        // Bit-packed memory savings check (~325 KB)
        assert!(ground.memory_usage_bytes() <= (ground.w * ground.h) / 7);
    }
}

#[cfg(test)]
mod labs2a_source_probes {
    use super::CollisionMask;
    use crate::stage::StageId;

    #[test]
    fn labs2a_ground_dimensions_and_spawn_shaft_match_source() {
        let ground = CollisionMask::level2a();
        assert_eq!((ground.origin_x, ground.origin_y), (-1542, -2307));
        assert_eq!((ground.w, ground.h), (8781, 10060));
        let stage = StageId::Level2A.descriptor();
        let spawn = stage.player_spawn_level_pos();
        assert!((spawn.x - (-982.90)).abs() < 0.05 && (spawn.y - (-122.85)).abs() < 0.05);
        assert!(
            !ground.solid_level_pos(spawn),
            "level2a spawn (227.2, 115.4 game-local) must be open inside the entry elevator shaft"
        );
        // Entry shaft walls enclose the spawn shaft horizontally; lower shaft floor sits at game_y ~ 2648.
        let left_wall_lx = 160.0 - stage.level_origin_game_local.x;
        let right_wall_lx = 510.0 - stage.level_origin_game_local.x;
        assert!(ground.solid(left_wall_lx, spawn.y), "left entry shaft wall solid");
        assert!(ground.solid(right_wall_lx, spawn.y), "right entry shaft wall solid");
        assert!(ground.solid(spawn.x, 2410.0), "shaft base floor solid at level_y=2410");
        assert!(!ground.solid(spawn.x, 2409.0), "shaft interior open immediately above base floor");
        assert!(ground.memory_usage_bytes() <= (ground.w * ground.h) / 7);
    }
}

#[cfg(test)]
mod labs2b_source_probes {
    use super::CollisionMask;
    use crate::stage::StageId;

    #[test]
    fn labs2b_ground_dimensions_and_spawn_room_match_source() {
        let ground = CollisionMask::level2b();
        assert_eq!((ground.origin_x, ground.origin_y), (-1542, -2307));
        assert_eq!((ground.w, ground.h), (8781, 10060));
        let stage = StageId::Level2B.descriptor();
        let spawn = stage.player_spawn_level_pos();
        assert!((spawn.x - 3203.60).abs() < 0.05 && (spawn.y - 6616.05).abs() < 0.05);
        assert!(
            !ground.solid_level_pos(spawn),
            "level2b spawn (4413.7, 6854.3 game-local) must be open inside the TV control room"
        );
        let left_wall_lx = 4093.0 - stage.level_origin_game_local.x;
        let room_mid_ly = 7050.0 - stage.level_origin_game_local.y;
        assert!(ground.solid(left_wall_lx, room_mid_ly), "left TV control room wall solid");
        let spawn_floor_ly = 7154.3 - stage.level_origin_game_local.y;
        assert!(ground.solid(spawn.x, spawn_floor_ly), "TV control room floor solid at gy=7154.3");
        assert!(ground.memory_usage_bytes() <= (ground.w * ground.h) / 7);
    }
}
