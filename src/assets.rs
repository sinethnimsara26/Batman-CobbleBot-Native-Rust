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
            if cache.len()>2{cache.pop_front();}
            image
        };
        LazySpriteFrame{image,anchor_x:meta.anchor_x,anchor_y:meta.anchor_y}
    }
}

impl std::ops::Deref for SpriteSet {
    type Target = [SpriteFrame];
    fn deref(&self) -> &Self::Target { &self.frames }
}

pub struct CommonAssets {
    pub batman_frames: SpriteSet,
    pub batman_frames_hq: SpriteSet,
    pub root_sky: Image,
    pub root_moon: Image,
    pub root_moon_hq: SpriteSet,
    pub pickup_frames: SpriteSet,
    pub batarang_frames: SpriteSet,
    pub pickup_frames_hq: SpriteSet,
    pub batarang_frames_hq: SpriteSet,
    pub level_title_frames: LazySpriteSet,
    pub fadein_frames: LazySpriteSet,
    pub fadeout_frames: LazySpriteSet,
    pub tutorial_overlay_frames: LazySpriteSet,
    pub level_title_frames_hq: LazySpriteSet,
    pub fadein_frames_hq: LazySpriteSet,
    pub fadeout_frames_hq: LazySpriteSet,
    pub tutorial_overlay_frames_hq: LazySpriteSet,
    pub hud_base: Image,
    pub hud_digits_small: Image,
    pub hud_digits_large: Image,
    pub hud_shell_hq: SpriteSet,
    pub hud_digits_small_hq: Image,
    pub hud_digits_batarang_hq: Image,
    pub hud_digits_large_hq: Image,
    pub title_screen: Image,
    pub instructions_screen: Image,
    /// 180 original root-frame-18 story samples, loaded lazily.
    pub intro_frames: LazySpriteSet,
    /// 225 original root-frame-22 (`card:escape`, #1267) story samples, loaded lazily.
    pub escape_card_frames: LazySpriteSet,
    /// 55 Level 1B Family A (`#880`) goon child frames, loaded lazily.
    pub goon_a_frames: LazySpriteSet,
    /// 66 Level 1B Family B (`#964`) goon child frames, loaded lazily.
    pub goon_b_frames: LazySpriteSet,
    /// 9 Level 1B biker (`#1005`) child frames (`drive: 0..2`, `fall: 2..9`), loaded lazily.
    pub biker_frames: LazySpriteSet,
    /// 162 Level 1C Penguin (`#1250`) child frames across all 10 source labels, loaded lazily.
    pub penguin_frames: LazySpriteSet,
    /// 23 Labs 2A/2B animated door (`#1325` frames `0..=19`) and jamb (`#1394..=#1396` frames `20..=22`) sprites.
    pub door_frames: LazySpriteSet,
    /// 2 Labs 2A/2B elevator platform sprites (`#1328` -> `0`, `#1330` -> `1`).
    pub elevator_frames: LazySpriteSet,
    /// 69 Labs 2A/2B goon (`#1365`) child frames across all 9 source labels, loaded lazily.
    pub labs_goon_frames: LazySpriteSet,
    /// 41 Labs eagle (`#1393`) child frames across all 5 source labels, loaded lazily.
    pub eagle_frames: LazySpriteSet,
}

impl CommonAssets {
    pub fn load() -> Self {
        Self {
            batman_frames: decode_sprite_frames_expected(include_bytes!("../assets/batman_frames.bin"),1.0,"batman_frames"),
            batman_frames_hq: decode_sprite_frames_expected(include_bytes!("../assets/batman_frames_hq.bin"),3.0,"batman_frames_hq"),
            root_sky: decode_png(include_bytes!("../assets/root_bg_131.png")),
            root_moon: decode_png(include_bytes!("../assets/root_bg_134.png")),
            root_moon_hq: decode_sprite_frames_expected(include_bytes!("../assets/root_bg_134_hq.bin"),3.0,"root_moon_hq"),
            pickup_frames: decode_sprite_frames_expected(include_bytes!("../assets/pickup_frames.bin"),1.0,"pickup_frames"),
            batarang_frames: decode_sprite_frames_expected(include_bytes!("../assets/batarang_frames.bin"),1.0,"batarang_frames"),
            pickup_frames_hq: decode_sprite_frames_expected(include_bytes!("../assets/pickup_frames_hq.bin"),3.0,"pickup_frames_hq"),
            batarang_frames_hq: decode_sprite_frames_expected(include_bytes!("../assets/batarang_frames_hq.bin"),3.0,"batarang_frames_hq"),
            level_title_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/level_title_frames.bin"),1.0,"level_title_frames"),
            fadein_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadein_frames.bin"),1.0,"fadein_frames"),
            fadeout_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadeout_frames.bin"),1.0,"fadeout_frames"),
            tutorial_overlay_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/tutorial_overlay_frames.bin"),1.0,"tutorial_overlay_frames"),
            level_title_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/level_title_frames_hq.bin"),3.0,"level_title_frames_hq"),
            fadein_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadein_frames_hq.bin"),3.0,"fadein_frames_hq"),
            fadeout_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/fadeout_frames_hq.bin"),3.0,"fadeout_frames_hq"),
            tutorial_overlay_frames_hq: decode_sprite_frames_lazy_expected(include_bytes!("../assets/tutorial_overlay_frames_hq.bin"),3.0,"tutorial_overlay_frames_hq"),
            hud_base: decode_png(include_bytes!("../assets/hud_base.png")),
            hud_digits_small: decode_png(include_bytes!("../assets/hud_digits_small.png")),
            hud_digits_large: decode_png(include_bytes!("../assets/hud_digits_large.png")),
            hud_shell_hq: decode_sprite_frames_expected(include_bytes!("../assets/hud_shell_hq.bin"),3.0,"hud_shell_hq"),
            hud_digits_small_hq: decode_png(include_bytes!("../assets/hud_digits_small_hq.png")),
            hud_digits_batarang_hq: decode_png(include_bytes!("../assets/hud_digits_batarang_hq.png")),
            hud_digits_large_hq: decode_png(include_bytes!("../assets/hud_digits_large_hq.png")),
            title_screen: decode_png(include_bytes!("../assets/title_screen.png")),
            instructions_screen: decode_png(include_bytes!("../assets/instructions_screen.png")),
            intro_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/intro_frames.bin"),1.0,"intro_frames"),
            escape_card_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/escape_card_frames.bin"),1.0,"escape_card_frames"),
            goon_a_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/goon_a_frames.bin"),1.0,"goon_a_frames"),
            goon_b_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/goon_b_frames.bin"),1.0,"goon_b_frames"),
            biker_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/biker_frames.bin"),1.0,"biker_frames"),
            penguin_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/penguin_frames.bin"),1.0,"penguin_frames"),
            door_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/door_frames.bin"),1.0,"door_frames"),
            elevator_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/elevator_frames.bin"),1.0,"elevator_frames"),
            labs_goon_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/labs_goon_frames.bin"),1.0,"labs_goon_frames"),
            eagle_frames: decode_sprite_frames_lazy_expected(include_bytes!("../assets/eagle_frames.bin"),1.0,"eagle_frames"),
        }
    }
}

pub struct StageAssets {
    pub stage_id: crate::stage::StageId,
    pub background: Image,
    pub tiles: LevelTileSet,
    pub tiles_hq_base: LevelTileSet,
    pub detail_tiles_hq: LevelTileSet,
}

impl StageAssets {
    pub fn load_level1a() -> Self {
        Self {
            stage_id: crate::stage::StageId::Level1A,
            background: decode_png(include_bytes!("../assets/level1a_bg.png")),
            tiles: LevelTileSet::new_expected(include_bytes!("../assets/level1a_tiles.bin"),1.0,12,"level1a_tiles"),
            tiles_hq_base: LevelTileSet::new_expected(include_bytes!("../assets/level1a_tiles_hq_base.bin"),1.0,4,"level1a_tiles_hq_base"),
            detail_tiles_hq: LevelTileSet::new_expected(include_bytes!("../assets/level1a_detail_tiles_hq.bin"),3.0,4,"level1a_detail_tiles_hq"),
        }
    }

    pub fn load_level1b() -> Self {
        Self {
            stage_id: crate::stage::StageId::Level1B,
            background: decode_png(include_bytes!("../assets/root_bg_131.png")),
            tiles: LevelTileSet::new_expected(include_bytes!("../assets/level1b_tiles.bin"),1.0,12,"level1b_tiles"),
            tiles_hq_base: LevelTileSet::empty(1.0),
            detail_tiles_hq: LevelTileSet::empty(3.0),
        }
    }

    pub fn load_level1c() -> Self {
        Self {
            stage_id: crate::stage::StageId::Level1C,
            background: decode_png(include_bytes!("../assets/root_bg_131.png")),
            tiles: LevelTileSet::new_expected(include_bytes!("../assets/level1c_tiles.bin"),1.0,12,"level1c_tiles"),
            tiles_hq_base: LevelTileSet::empty(1.0),
            detail_tiles_hq: LevelTileSet::empty(3.0),
        }
    }

    pub fn load_level2a() -> Self {
        Self {
            stage_id: crate::stage::StageId::Level2A,
            background: decode_png(include_bytes!("../assets/root_bg_131.png")),
            tiles: LevelTileSet::new_expected(include_bytes!("../assets/level2a_tiles.bin"),1.0,12,"level2a_tiles"),
            tiles_hq_base: LevelTileSet::empty(1.0),
            detail_tiles_hq: LevelTileSet::empty(3.0),
        }
    }

    pub fn load(stage_id: crate::stage::StageId) -> Self {
        match stage_id {
            crate::stage::StageId::Level1A => Self::load_level1a(),
            crate::stage::StageId::Level1B => Self::load_level1b(),
            crate::stage::StageId::Level1C => Self::load_level1c(),
            crate::stage::StageId::Level2A => Self::load_level2a(),
            _ => panic!("StageAssets::load: stage {:?} assets not yet embedded", stage_id),
        }
    }
}

pub struct Assets {
    pub common: CommonAssets,
    pub stage: StageAssets,
    pub city_background: Image,
    #[allow(dead_code)]
    pub level1a_tiles: LevelTileSet,
    #[allow(dead_code)]
    pub level1a_tiles_hq_base: LevelTileSet,
    #[allow(dead_code)]
    pub level1a_detail_tiles_hq: LevelTileSet,
}

impl std::ops::Deref for Assets {
    type Target = CommonAssets;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl std::ops::DerefMut for Assets {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl Assets {
    pub fn load() -> Self {
        let common = CommonAssets::load();
        let stage = StageAssets::load_level1a();
        let city_background = stage.background.clone();
        let level1a_tiles = stage.tiles.clone();
        let level1a_tiles_hq_base = stage.tiles_hq_base.clone();
        let level1a_detail_tiles_hq = stage.detail_tiles_hq.clone();
        Self {
            common,
            stage,
            city_background,
            level1a_tiles,
            level1a_tiles_hq_base,
            level1a_detail_tiles_hq,
        }
    }

    pub fn load_stage(&mut self, stage_id: crate::stage::StageId) {
        let stage = StageAssets::load(stage_id);
        if stage_id == crate::stage::StageId::Level1A {
            self.city_background = stage.background.clone();
            self.level1a_tiles = stage.tiles.clone();
            self.level1a_tiles_hq_base = stage.tiles_hq_base.clone();
            self.level1a_detail_tiles_hq = stage.detail_tiles_hq.clone();
        }
        self.stage = stage;
    }
}

#[derive(Copy,Clone)]
struct LevelTileMeta {
    offset: usize,
    len: usize,
    crop_x: i32,
    crop_y: i32,
}

pub struct LevelTileFrame {
    pub image: Rc<Image>,
    pub crop_x: i32,
    pub crop_y: i32,
}

pub struct LevelTileSet {
    bytes: &'static [u8],
    index: HashMap<(i32,i32),LevelTileMeta>,
    cache: RefCell<VecDeque<((i32,i32),Rc<Image>)>>,
    cache_limit: usize,
    pub logical_pixel_scale: f32,
}
impl LevelTileSet {
    fn new_expected(bytes:&'static [u8],expected_scale:f32,cache_limit:usize,label:&str)->Self{
        assert!(cache_limit>0,"level tile cache limit must be positive");
        assert!(bytes.len()>=12,"level tile pack too small");
        let magic=&bytes[..8];
        let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let (logical_pixel_scale,mut pos)=match magic{
            b"BCLVT001"=>(1.0,12usize),
            b"BCLVT002"=>{
                assert!(bytes.len()>=16,"BCLVT002 header truncated");
                let scale=f32::from_le_bytes(bytes[12..16].try_into().unwrap());
                assert!(scale.is_finite()&&scale>0.0,"invalid BCLVT002 logical pixel scale");
                (scale,16usize)
            }
            _=>panic!("unsupported level tile pack format: {:?}",magic),
        };
        assert!(
            (logical_pixel_scale-expected_scale).abs()<=0.0001,
            "level tile pack scale mismatch for {}: expected {}, got {}",
            label,expected_scale,logical_pixel_scale
        );
        let mut index=HashMap::with_capacity(count);
        for _ in 0..count {
            if magic==b"BCLVT001"{
                assert!(pos+12<=bytes.len(),"BCLVT001 tile header truncated");
                let x=i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
                let y=i32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
                let len=u32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap()) as usize;
                pos+=12;
                assert!(pos+len<=bytes.len(),"BCLVT001 tile PNG truncated");
                index.insert((x,y),LevelTileMeta{offset:pos,len,crop_x:0,crop_y:0});
                pos+=len;
            }else{
                assert!(pos+20<=bytes.len(),"BCLVT002 tile header truncated");
                let x=i32::from_le_bytes(bytes[pos..pos+4].try_into().unwrap());
                let y=i32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
                let crop_x=i32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap());
                let crop_y=i32::from_le_bytes(bytes[pos+12..pos+16].try_into().unwrap());
                let len=u32::from_le_bytes(bytes[pos+16..pos+20].try_into().unwrap()) as usize;
                pos+=20;
                assert!(crop_x>=0&&crop_y>=0,"negative BCLVT002 crop offset");
                assert!(pos+len<=bytes.len(),"BCLVT002 tile PNG truncated");
                index.insert((x,y),LevelTileMeta{offset:pos,len,crop_x,crop_y});
                pos+=len;
            }
        }
        assert_eq!(pos,bytes.len(),"trailing bytes in level tile pack");
        Self{
            bytes,index,cache:RefCell::new(VecDeque::new()),
            cache_limit,logical_pixel_scale
        }
    }
    pub fn empty(logical_pixel_scale: f32) -> Self {
        Self {
            bytes: &[],
            index: HashMap::new(),
            cache: RefCell::new(VecDeque::new()),
            cache_limit: 1,
            logical_pixel_scale,
        }
    }
    pub fn tile(&self,x:i32,y:i32)->Option<LevelTileFrame>{
        let meta=*self.index.get(&(x,y))?;
        let mut cache=self.cache.borrow_mut();
        let image=if let Some(i)=cache.iter().position(|(k,_)|*k==(x,y)){
            let item=cache.remove(i).unwrap();
            let image=Rc::clone(&item.1);
            cache.push_back(item);
            image
        }else{
            let image=Rc::new(decode_png(&self.bytes[meta.offset..meta.offset+meta.len]));
            cache.push_back(((x,y),Rc::clone(&image)));
            if cache.len()>self.cache_limit{cache.pop_front();}
            image
        };
        Some(LevelTileFrame{image,crop_x:meta.crop_x,crop_y:meta.crop_y})
    }
}

impl Clone for LevelTileSet {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes,
            index: self.index.clone(),
            cache: RefCell::new(VecDeque::new()),
            cache_limit: self.cache_limit,
            logical_pixel_scale: self.logical_pixel_scale,
        }
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
        assert_eq!(set.len(),5);
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

    #[test]
    fn common_and_stage_assets_split_invariants(){
        let assets=super::Assets::load();
        assert_eq!(assets.stage.stage_id,crate::stage::StageId::Level1A);
        assert!((assets.stage.tiles.logical_pixel_scale-1.0).abs()<0.0001);
        assert!((assets.stage.tiles_hq_base.logical_pixel_scale-1.0).abs()<0.0001);
        assert!((assets.stage.detail_tiles_hq.logical_pixel_scale-3.0).abs()<0.0001);
        assert_eq!(assets.common.intro_frames.len(),180);
        assert_eq!(assets.intro_frames.len(),180);
        assert_eq!(assets.common.title_screen.w,600);
        assert_eq!(assets.title_screen.w,600);
    }

    #[test]
    fn stage_assets_load_and_swap_preserves_depth_and_scale(){
        let stage_1a=super::StageAssets::load(crate::stage::StageId::Level1A);
        assert_eq!(stage_1a.stage_id,crate::stage::StageId::Level1A);
        assert!((stage_1a.tiles.logical_pixel_scale-1.0).abs()<0.0001);
        assert!((stage_1a.detail_tiles_hq.logical_pixel_scale-3.0).abs()<0.0001);

        let mut assets=super::Assets::load();
        assets.load_stage(crate::stage::StageId::Level1A);
        assert_eq!(assets.stage.stage_id,crate::stage::StageId::Level1A);
    }

    #[test]
    fn assets_deref_transparent_access(){
        let assets=super::Assets::load();
        assert_eq!(assets.batman_frames.frames.len(),assets.common.batman_frames.frames.len());
        assert_eq!(assets.hud_shell_hq.frames.len(),assets.common.hud_shell_hq.frames.len());
        assert_eq!(assets.hud_shell_hq.frames.len(),1);
    }

    #[test]
    fn stage_assets_load_level1b_invariants(){
        let stage_1b=super::StageAssets::load(crate::stage::StageId::Level1B);
        assert_eq!(stage_1b.stage_id,crate::stage::StageId::Level1B);
        assert!((stage_1b.tiles.logical_pixel_scale-1.0).abs()<0.0001);
        assert_eq!(stage_1b.tiles.index.len(),75);
        assert!((stage_1b.tiles_hq_base.logical_pixel_scale-1.0).abs()<0.0001);
        assert_eq!(stage_1b.tiles_hq_base.index.len(),0);
        assert!((stage_1b.detail_tiles_hq.logical_pixel_scale-3.0).abs()<0.0001);
        assert_eq!(stage_1b.detail_tiles_hq.index.len(),0);
    }

    #[test]
    fn b19_stage_1c_and_enemy_sprite_packs_invariants(){
        let stage_1c=super::StageAssets::load(crate::stage::StageId::Level1C);
        assert_eq!(stage_1c.stage_id,crate::stage::StageId::Level1C);
        assert!((stage_1c.tiles.logical_pixel_scale-1.0).abs()<0.0001);
        assert_eq!(stage_1c.tiles.index.len(),60);
        assert_eq!(stage_1c.tiles_hq_base.index.len(),0);
        assert_eq!(stage_1c.detail_tiles_hq.index.len(),0);

        let mut assets=super::Assets::load();
        assert_eq!(assets.goon_a_frames.len(),55);
        assert_eq!(assets.goon_b_frames.len(),66);
        assert_eq!(assets.biker_frames.len(),9);
        assert_eq!(assets.penguin_frames.len(),162);
        assert_eq!(assets.door_frames.len(),23);
        assert_eq!(assets.elevator_frames.len(),2);
        assert_eq!(assets.labs_goon_frames.len(),69);
        assert_eq!(assets.eagle_frames.len(),41);
        assets.load_stage(crate::stage::StageId::Level1B);
        assert_eq!(assets.stage.stage_id,crate::stage::StageId::Level1B);
        assets.load_stage(crate::stage::StageId::Level1C);
        assert_eq!(assets.stage.stage_id,crate::stage::StageId::Level1C);
    }

    #[test]
    fn b20_labs2a_tile_pack_and_scenery_invariants(){
        let stage_2a=super::StageAssets::load(crate::stage::StageId::Level2A);
        assert_eq!(stage_2a.stage_id,crate::stage::StageId::Level2A);
        assert!((stage_2a.tiles.logical_pixel_scale-1.0).abs()<0.0001);
        assert_eq!(stage_2a.tiles.index.len(),177);
        assert_eq!(stage_2a.tiles_hq_base.index.len(),0);
        assert_eq!(stage_2a.detail_tiles_hq.index.len(),0);
        assert!(stage_2a.tiles.index.contains_key(&(0,0)),"rooftop shaft tile (0,0) must exist");
        assert!(stage_2a.tiles.index.contains_key(&(1800,2400)),"upper lab tile (1800,2400) must exist");
        assert!(stage_2a.tiles.index.contains_key(&(4800,3600)),"mid lab tile (4800,3600) must exist");
        assert!(stage_2a.tiles.index.contains_key(&(4200,6800)),"tv exit tile (4200,6800) must exist");

        let mut assets=super::Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        assert_eq!(assets.stage.stage_id,crate::stage::StageId::Level2A);
        assert_eq!(assets.stage.tiles.index.len(),177);
    }
}
