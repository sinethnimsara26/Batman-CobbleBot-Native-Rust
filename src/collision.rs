use png::{ColorType, Decoder, Transformations};
use std::io::Cursor;

pub struct CollisionMask { pub origin_x:i32,pub origin_y:i32,pub w:usize,pub h:usize,solid:Vec<u8> }
impl CollisionMask {
    pub fn level1a()->Self{decode_mask(include_bytes!("../reverse-engineering/generated/collision_masks/level1a_ground.png"),-5539,-1929)}
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