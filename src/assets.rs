use png::{ColorType, Decoder, Transformations};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::io::Cursor;
use std::rc::Rc;

#[derive(Clone)]
pub struct Image { pub w: usize, pub h: usize, pub pixels: Vec<u32> }

pub struct SpriteFrame { pub image: Image, pub anchor_x: f32, pub anchor_y: f32 }

pub struct Assets {
    pub batman_frames: Vec<SpriteFrame>,
    pub city_background: Image,
    pub pickup_frames: Vec<SpriteFrame>,
    pub batarang_frames: Vec<SpriteFrame>,
    pub level_title_frames: Vec<SpriteFrame>,
    pub fadein_frames: Vec<SpriteFrame>,
    pub tutorial_overlay_frames: Vec<SpriteFrame>,
    pub tutorial_tostreet_text: Image,
    pub level1a_tiles: LevelTileSet,
    pub title_screen: Image,
    pub instructions_screen: Image,
}

impl Assets {
    pub fn load() -> Self {
        Self {
            batman_frames: decode_sprite_frames(include_bytes!("../assets/batman_frames.bin")),
            city_background: decode_png(include_bytes!("../assets/level1a_bg.png")),
            pickup_frames: decode_sprite_frames(include_bytes!("../assets/pickup_frames.bin")),
            batarang_frames: decode_sprite_frames(include_bytes!("../assets/batarang_frames.bin")),
            level_title_frames: decode_sprite_frames(include_bytes!("../assets/level_title_frames.bin")),
            fadein_frames: decode_sprite_frames(include_bytes!("../assets/fadein_frames.bin")),
            tutorial_overlay_frames: decode_sprite_frames(include_bytes!("../assets/tutorial_overlay_frames.bin")),
            tutorial_tostreet_text: decode_png(include_bytes!("../assets/tutorial_tostreet_text.png")),
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

fn decode_sprite_frames(bytes:&[u8])->Vec<SpriteFrame>{
    assert!(bytes.len()>=12 && &bytes[..8]==b"BCBFRM01");
    let count=u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut pos=12; let mut out=Vec::with_capacity(count);
    for _ in 0..count{
        let w=u16::from_le_bytes(bytes[pos..pos+2].try_into().unwrap()) as usize;
        let h=u16::from_le_bytes(bytes[pos+2..pos+4].try_into().unwrap()) as usize;
        let ax=f32::from_le_bytes(bytes[pos+4..pos+8].try_into().unwrap());
        let ay=f32::from_le_bytes(bytes[pos+8..pos+12].try_into().unwrap());
        let len=u32::from_le_bytes(bytes[pos+12..pos+16].try_into().unwrap()) as usize; pos+=16;
        let image=decode_png(&bytes[pos..pos+len]); assert_eq!((image.w,image.h),(w,h)); pos+=len;
        out.push(SpriteFrame{image,anchor_x:ax,anchor_y:ay});
    }
    assert!(!out.is_empty()); out
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