use png::{ColorType, Decoder, Transformations};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use std::rc::Rc;

#[derive(Clone)]
pub struct Image { pub w: usize, pub h: usize, pub pixels: Vec<u32> }

pub struct SpriteFrame { pub image: Image, pub anchor_x: f32, pub anchor_y: f32 }

pub struct SpriteSet {
    pub frames: Vec<SpriteFrame>,
    pub logical_pixel_scale: f32,
}

struct LazyFrameMeta {
    w: usize,
    h: usize,
    anchor_x: f32,
    anchor_y: f32,
    offset: usize,
    len: usize,
}

pub struct LazySpriteFrame {
    pub image: Rc<Image>,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

pub struct LazySpriteSet {
    bytes: &'static [u8],
    frames: Vec<LazyFrameMeta>,
    pub logical_pixel_scale: f32,
    cache: RefCell<VecDeque<(usize,Rc<Image>)>>,
}

impl LazySpriteSet {
    pub fn len(&self)->usize { self.frames.len() }
    pub fn is_empty(&self)->bool { self.frames.is_empty() }

    pub fn frame(&self,index:usize)->LazySpriteFrame {
        let meta=&self.frames[index];
        let mut cache=self.cache.borrow_mut();
        let image=if let Some(pos)=cache.iter().position(|(i,_)|*i==index){
            let item=cache.remove(pos).unwrap();
            let image=Rc::clone(&item.1);
            cache.push_back(item);
            image
        }else{
            let image=Rc::new(decode_png(&self.bytes[meta.offset..meta.offset+meta.len]));
            assert_eq!((image.w,image.h),(meta.w,meta.h));
            cache.push_back((index,Rc::clone(&image)));
            if cache.len()>4{cache.pop_front();}
            image
        };
        LazySpriteFrame{image,anchor_x:meta.anchor_x,anchor_y:meta.anchor_y}
    }
}

impl std::ops::Deref for SpriteSet {
    type Target = [SpriteFrame];
    fn deref(&self) -> &Self::Target { &self.frames }
}

pub struct Assets {
    pub batman_frames: SpriteSet,
    pub batman_frames_hq: SpriteSet,
    pub root_sky: Image,
    pub root_moon: Image,
    pub city_background: Image,
    pub pickup_frames: SpriteSet,
    pub batarang_frames: SpriteSet,
    pub level_title_frames: SpriteSet,
    pub fadein_frames: SpriteSet,
    pub fadeout_frames: SpriteSet,
    pub tutorial_overlay_frames: SpriteSet,
    pub level_title_frames_hq: LazySpriteSet,
    pub fadein_frames_hq: LazySpriteSet,
    pub fadeout_frames_hq: LazySpriteSet,
    pub tutorial_overlay_frames_hq: LazySpriteSet,
    pub hud_base: Image,
    pub hud_digits_small: Image,
    pub hud_digits_large: Image,
    pub hud_shell_hq: SpriteSet,
    pub hud_digits_small_hq: Image,
    pub hud_digits_large_hq: Image,
    pub level1a_tiles: LevelTileSet,
    pub title_screen: Image,
    pub instructions_screen: Image,
}

impl Assets {
    pub fn load() -> Self {
        Self {
            batman_frames: decode_sprite_frames_expected(include_bytes!("../assets/batman_frames.bin"),1.0,"batman_frames"),
            batman_frames_hq: decode_sprite_frames_expected(include_bytes!("../assets/batman_frames_hq.bin"),3.0,"batman_frames_hq"),
            root_sky: decode_png(include_bytes!("../assets/root_bg_131.png")),
            root_moon: decode_png(include_bytes!("../assets/root_bg_134.png")),
            city_background: decode_png(include_bytes!("../assets/level1a_bg.png")),
            pickup_frames: decode_sprite_frames_expected(include_bytes!("../assets/pickup_frames.bin"),1.0,"pickup_frames"),
            batarang_frames: decode_sprite_frames_expected(include_bytes!("../assets/batarang_frames.bin"),1.0,"batarang_frames"),
            level_title_frames: decode_sprite_frames_expected(include_bytes!("../assets/level_title_frames.bin"),1.0,"level_title_frames"),
            fadein_frames: decode_sprite_frames_expected(include_bytes!("../assets/fadein_frames.bin"),1.0,"fadein_frames"),
            fadeout_frames: decode_sprite_frames_expected(include_bytes!("../assets/fadeout_frames.bin"),1.0,"fadeout_frames"),
            tutorial_overlay_frames: decode_sprite_frames_expected(include_bytes!("../assets/tutorial_overlay_frames.bin"),1.0,"tutorial_overlay_frames"),
            level_title_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/level_title_frames_hq.bin"),3.0,"level_title_frames_hq"),
            fadein_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadein_frames_hq.bin"),3.0,"fadein_frames_hq"),
            fadeout_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadeout_frames_hq.bin"),3.0,"fadeout_frames_hq"),
            tutorial_overlay_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/tutorial_overlay_frames_hq.bin"),3.0,"tutorial_overlay_frames_hq"),
            hud_base: decode_png(include_bytes!("../assets/hud_base.png")),
            hud_digits_small: decode_png(include_bytes!("../assets/hud_digits_small.png")),
            hud_digits_large: decode_png(include_bytes!("../assets/hud_digits_large.png")),
            hud_shell_hq: decode_sprite_frames_expected(include_bytes!("../assets/hud_shell_hq.bin"),3.0,"hud_shell_hq"),
            hud_digits_small_hq: decode_png(include_bytes!("../assets/hud_digits_small_hq.png")),
            hud_digits_large_hq: decode_png(include_bytes!("../assets/hud_digits_large_hq.png")),
            level1a_tiles: LevelTileSet::new(include_bytes!("../assets/level1a_tiles.bin")),
            title_screen: decode_png(include_bytes!("../assets/title_screen.png")),
            instructions_screen: decode_png(include_bytes!("../assets/instructions_screen.png")),
        }
    }
}

pub struct LevelTileSet {
    bytes: &'static [u8],
    index: HashMap<(i32,i32),(usize,usize)>,
    cache: RefCell<VecDeque<((i32,i32),Rc<Image>)>>,
}
impl LevelTileSet {
    fn new(bytes:&'static [u8])->Self{
        assert!(bytes.len()>=12 && &bytes[..8]==b"BCLVT001");
        let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let mut pos=12; let mut index=HashMap::with_capacity(count);
        for _ in 0..count {
            let x=i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
            let y=i32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
            let len=u32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap()) as usize;
            pos+=12; assert!(pos+len<=bytes.len()); index.insert((x,y),(pos,len)); pos+=len;
        }
        assert_eq!(pos,bytes.len());
        Self{bytes,index,cache:RefCell::new(VecDeque::new())}
    }
    pub fn tile(&self,x:i32,y:i32)->Option<Rc<Image>>{
        let mut cache=self.cache.borrow_mut();
        if let Some(i)=cache.iter().position(|(k,_)|*k==(x,y)){
            let item=cache.remove(i).unwrap(); let image=Rc::clone(&item.1); cache.push_back(item); return Some(image);
        }
        let (offset,len)=*self.index.get(&(x,y))?;
        let image=Rc::new(decode_png(&self.bytes[offset..offset+len]));
        cache.push_back(((x,y),Rc::clone(&image))); if cache.len()>12{cache.pop_front();}
        Some(image)
    }
}

fn decode_sprite_frames_lazy_expected(bytes:&'static [u8],expected_scale:f32,label:&str)->LazySpriteSet{
    let set=decode_sprite_frames_lazy(bytes);
    let delta=(set.logical_pixel_scale-expected_scale).abs();
    assert!(
        delta<=0.0001,
        "lazy sprite pack scale mismatch for {}: expected {}, got {}",
        label,expected_scale,set.logical_pixel_scale
    );
    set
}

fn decode_sprite_frames_lazy(bytes:&'static [u8])->LazySpriteSet{
    assert!(bytes.len()>=12,"lazy sprite pack too small");
    let magic=&bytes[..8];
    let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let (logical_pixel_scale,mut pos)=match magic{
        b"BCBFRM01"=>(1.0,12usize),
        b"BCBFRM02"=>{
            assert!(bytes.len()>=16,"BCBFRM02 header truncated");
            let scale=f32::from_le_bytes(bytes[12..16].try_into().unwrap());
            assert!(scale.is_finite() && scale>0.0,"invalid lazy BCBFRM02 logical pixel scale");
            (scale,16usize)
        }
        _=>panic!("unsupported lazy sprite pack format: {:?}",magic),
    };
    let mut frames=Vec::with_capacity(count);
    for _ in 0..count{
        assert!(pos+16<=bytes.len(),"lazy sprite frame header truncated");
        let w=u16::from_le_bytes(bytes[pos..pos+2].try_into().unwrap()) as usize;
        let h=u16::from_le_bytes(bytes[pos+2..pos+4].try_into().unwrap()) as usize;
        let anchor_x=f32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
        let anchor_y=f32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap());
        let len=u32::from_le_bytes(bytes[pos+12..pos+16].try_into().unwrap()) as usize;
        pos+=16;
        assert!(pos+len<=bytes.len(),"lazy sprite frame PNG payload truncated");
        frames.push(LazyFrameMeta{w,h,anchor_x,anchor_y,offset:pos,len});
        pos+=len;
    }
    assert!(!frames.is_empty());
    assert_eq!(pos,bytes.len(),"trailing bytes in lazy sprite pack");
    LazySpriteSet{
        bytes,
        frames,
        logical_pixel_scale,
        cache:RefCell::new(VecDeque::new()),
    }
}

fn decode_sprite_frames_expected(bytes:&[u8],expected_scale:f32,label:&str)->SpriteSet{
    let set=decode_sprite_frames(bytes);
    let delta=(set.logical_pixel_scale-expected_scale).abs();
    assert!(
        delta<=0.0001,
        "sprite pack scale mismatch for {}: expected {}, got {}",
        label,expected_scale,set.logical_pixel_scale
    );
    set
}

fn decode_sprite_frames(bytes:&[u8])->SpriteSet{
    assert!(bytes.len()>=12,"sprite pack too small");
    let magic=&bytes[..8];
    let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let (logical_pixel_scale,mut pos)=match magic{
        b"BCBFRM01"=>(1.0,12usize),
        b"BCBFRM02"=>{
            assert!(bytes.len()>=16,"BCBFRM02 header truncated");
            let scale=f32::from_le_bytes(bytes[12..16].try_into().unwrap());
            assert!(scale.is_finite() && scale>0.0,"invalid BCBFRM02 logical pixel scale");
            (scale,16usize)
        }
        _=>panic!("unsupported sprite pack format: {:?}",magic),
    };
    let mut out=Vec::with_capacity(count);
    for _ in 0..count{
        assert!(pos+16<=bytes.len(),"sprite frame header truncated");
        let w=u16::from_le_bytes(bytes[pos..pos+2].try_into().unwrap()) as usize;
        let h=u16::from_le_bytes(bytes[pos+2..pos+4].try_into().unwrap()) as usize;
        let ax=f32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
        let ay=f32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap());
        let len=u32::from_le_bytes(bytes[pos+12..pos+16].try_into().unwrap()) as usize;
        pos+=16;
        assert!(pos+len<=bytes.len(),"sprite frame PNG payload truncated");
        let image=decode_png(&bytes[pos..pos+len]);
        assert_eq!((image.w,image.h),(w,h));
        pos+=len;
        out.push(SpriteFrame{image,anchor_x:ax,anchor_y:ay});
    }
    assert!(!out.is_empty());
    assert_eq!(pos,bytes.len(),"trailing bytes in sprite pack");
    SpriteSet{frames:out,logical_pixel_scale}
}

pub fn decode_png(bytes:&[u8])->Image{
    let mut decoder=Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(Transformations::EXPAND|Transformations::STRIP_16);
    let mut reader=decoder.read_info().expect("PNG header");
    let mut buf=vec![0u8;reader.output_buffer_size()];
    let info=reader.next_frame(&mut buf).expect("PNG data");
    let data=&buf[..info.buffer_size()]; let mut out=Vec::with_capacity(info.width as usize*info.height as usize);
    match info.color_type{
        ColorType::Rgba=>for p in data.chunks_exact(4){out.push(((p[3]as u32)<<24)|((p[0]as u32)<<16)|((p[1]as u32)<<8)|p[2]as u32)},
        ColorType::Rgb=>for p in data.chunks_exact(3){out.push(0xff000000|((p[0]as u32)<<16)|((p[1]as u32)<<8)|p[2]as u32)},
        ColorType::Grayscale=>for &v in data{let v=v as u32;out.push(0xff000000|(v<<16)|(v<<8)|v)},
        ColorType::GrayscaleAlpha=>for p in data.chunks_exact(2){let v=p[0]as u32;out.push(((p[1]as u32)<<24)|(v<<16)|(v<<8)|v)},
        ColorType::Indexed=>unreachable!(),
    }
    Image{w:info.width as usize,h:info.height as usize,pixels:out}
}

#[cfg(test)]
mod tests{
    use super::{decode_sprite_frames,decode_sprite_frames_expected,decode_sprite_frames_lazy};

    #[test]
    fn legacy_v1_defaults_to_one(){
        let set=decode_sprite_frames(include_bytes!("../assets/batarang_frames.bin"));
        assert!((set.logical_pixel_scale-1.0).abs()<0.0001);
        assert!(!set.frames.is_empty());
    }

    #[test]
    fn lazy_v2_reports_embedded_scale(){
        let set=decode_sprite_frames_lazy(include_bytes!("../assets/scale_probe.bin"));
        assert!((set.logical_pixel_scale-3.0).abs()<0.0001);
        assert_eq!(set.len(),1);
        let frame=set.frame(0);
        assert!(frame.image.w>0 && frame.image.h>0);
    }

    #[test]
    fn v2_reports_embedded_scale(){
        let set=decode_sprite_frames(include_bytes!("../assets/scale_probe.bin"));
        assert!((set.logical_pixel_scale-3.0).abs()<0.0001);
        assert!(!set.frames.is_empty());
    }

    #[test]
    #[should_panic(expected="sprite pack scale mismatch for scale_probe: expected 1, got 3")]
    fn v2_scale_mismatch_panics(){
        let _=decode_sprite_frames_expected(
            include_bytes!("../assets/scale_probe.bin"),1.0,"scale_probe"
        );
    }
}
