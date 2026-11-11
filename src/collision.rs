use png::{ColorType, Decoder, Transformations};
use std::io::Cursor;

pub struct CollisionMask { pub origin_x:i32,pub origin_y:i32,pub w:usize,pub h:usize,solid:Vec<u8> }
impl CollisionMask {
    pub fn level1a()->Self{decode_mask(include_bytes!("../reverse-engineering/generated/collision_masks/level1a_ground.png"),-5539,-1929)}
    /// B07: original root20/level #783 named ground #782, in LEVEL-local
    /// pixels. The baked mask comes from the SHA-locked FWS at build time.
    /// This constructor is dormant: Game::new() still selects only Level1A.
    #[allow(dead_code)]
    pub fn level1b()->Self{decode_mask(include_bytes!("../reverse-engineering/generated/collision_masks/level1b_ground.png"),-11196,-886)}
    #[inline] pub fn solid(&self,x:f32,y:f32)->bool{
        let px=x.floor() as i32-self.origin_x; let py=y.floor() as i32-self.origin_y;
        px>=0&&py>=0&&(px as usize)<self.w&&(py as usize)<self.h&&self.solid[py as usize*self.w+px as usize]!=0
    }
    pub fn check_ground(&self,x:f32,y:&mut f32,height:f32,dy:&mut f32,loop_count:u8)->bool{
        if loop_count>=16||!self.solid(x,*y+height){return false;}
        *y-=1.0;*dy=0.0;
        if self.solid(x,*y+height){let _=self.check_ground(x,y,height,dy,loop_count+1);} true
    }
    pub fn check_ceiling(&self,x:f32,y:&mut f32,height:f32,dy:&mut f32,loop_count:u8)->bool{
        if loop_count>=16||!self.solid(x,*y-height){return false;}
        *y+=1.0;*dy=0.0;
        if self.solid(x,*y-height){let _=self.check_ceiling(x,y,height,dy,loop_count+1);} true
    }
    #[inline] pub fn check_wall_clear(&self,x:f32,y:f32,height:f32,dx:f32)->bool{!self.solid(x+dx,y+height*0.5)}
}
fn decode_mask(bytes:&[u8],origin_x:i32,origin_y:i32)->CollisionMask{
    let mut dec=Decoder::new(Cursor::new(bytes)); dec.set_transformations(Transformations::EXPAND|Transformations::STRIP_16);
    let mut r=dec.read_info().unwrap(); let mut buf=vec![0u8;r.output_buffer_size()]; let info=r.next_frame(&mut buf).unwrap();
    let data=&buf[..info.buffer_size()]; let mut solid=Vec::with_capacity(info.width as usize*info.height as usize);
    match info.color_type{
        ColorType::Grayscale=>solid.extend(data.iter().map(|&v|u8::from(v>=128))),
        ColorType::GrayscaleAlpha=>for p in data.chunks_exact(2){solid.push(u8::from(p[0]>=128&&p[1]>=8))},
        ColorType::Rgb=>for p in data.chunks_exact(3){solid.push(u8::from((p[0]|p[1]|p[2])!=0))},
        ColorType::Rgba=>for p in data.chunks_exact(4){solid.push(u8::from((p[0]|p[1]|p[2])!=0&&p[3]>=8))},
        ColorType::Indexed=>unreachable!(),
    }
    CollisionMask{origin_x,origin_y,w:info.width as usize,h:info.height as usize,solid}
}

#[cfg(test)]
mod bank_street_source_probes {
    use super::CollisionMask;
    use crate::stage::StageId;

    #[test]
    fn bank_ground_dimensions_and_spawn_space_match_source() {
        let ground=CollisionMask::level1b();
        assert_eq!((ground.origin_x,ground.origin_y,ground.w,ground.h),
                   (-11196,-886,12102,1413));
        let stage=StageId::Level1B.descriptor();
        assert_eq!(stage.ground_symbol,782);
        assert_eq!(stage.source_player_direction,-1);
        assert_eq!(stage.player_spawn_game_local,(528.15,-18.75));
        let (px,py)=stage.player_spawn_level_local();
        assert!((px+506.5).abs()<0.001 && (py+258.0).abs()<0.001);
        assert!(!ground.solid(-11197.0,0.0));
        assert!(!ground.solid(906.0,0.0));
        assert!(!ground.solid(0.0,-887.0));
        assert!(!ground.solid(0.0,527.0));
    }

    /// Use the original baked *geometry*, not arbitrary replacement slabs.
    /// Find a thick, exposed floor surface near source player/street positions.
    /// Probes run with 75px feet, 50px ceiling and the native 16px push-out.
    #[test]
    fn bank_street_walk_jump_fall_and_wall_probes_use_original_ground() {
        let ground=CollisionMask::level1b();
        let mut site=None;
        for x in [-506.0,-400.0,-600.0,-1000.0,-3000.0,-6000.0] {
            for y in -700..490 {
                let fy=y as f32;
                if ground.solid(x,fy)
                    && ground.solid(x,fy+12.0)
                    && !ground.solid(x,fy-1.0)
                    && !ground.solid(x,fy-90.0)
                {
                    site=Some((x,fy));
                    break;
                }
            }
            if site.is_some(){break;}
        }
        let (x,floor)=site.expect("source #782 needs an exposed thick Bank Street floor");
        let mut y=floor-75.0+2.0;
        let mut dy=8.0;
        assert!(ground.check_ground(x,&mut y,75.0,&mut dy,0),
                "2px foot penetration must be resolved");
        assert!(!ground.solid(x,y+75.0),"feet must not remain inside source ground");
        assert_eq!(dy,0.0);

        // Original player logic tests wall at half-height before x movement.
        // If a source wall blocks a step, the point probe MUST prevent it;
        // if clear, advancing five pixels may be followed by a ledge fall.
        let mut walk_x=x;
        for _ in 0..12 {
            let can_move=ground.check_wall_clear(walk_x,y,75.0,-5.0);
            if can_move {walk_x-=5.0;}
            assert!(can_move || !ground.check_wall_clear(walk_x,y,75.0,-5.0));
            assert!(!ground.solid(walk_x,y+37.5),
                    "native midpoint wall probe leaked into a source wall");
        }

        // A vertical 25Hz jump / gravity / landing probe on the original
        // source mask, intentionally NOT a playable stage or AI test.
        let mut pos_y=y;
        let mut velocity=-42.0;
        let mut rose=false;
        let mut landed=false;
        for tick in 0..40 {
            if !ground.check_ceiling(x,&mut pos_y,50.0,&mut velocity,0) {
                pos_y+=velocity;
            }
            if pos_y<y-2.0 {rose=true;}
            if ground.check_ground(x,&mut pos_y,75.0,&mut velocity,0) {
                if tick>0 {landed=true;break;}
            } else {
                velocity+=5.0;
            }
        }
        assert!(rose,"jump must lift off original Bank Street floor");
        assert!(landed,"original source ground should catch the falling probe");
    }
}
