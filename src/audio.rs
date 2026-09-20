use crate::game::AudioEvent;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;

#[link(name="winmm")]
extern "system" {
    fn mciSendStringW(
        command: *const u16,
        return_string: *mut u16,
        return_length: u32,
        callback: *mut c_void,
    ) -> u32;
}

fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}

fn command(s:&str)->bool{
    unsafe{mciSendStringW(wide(s).as_ptr(),std::ptr::null_mut(),0,std::ptr::null_mut())==0}
}

pub struct Audio {
    dir:PathBuf,
    music:bool,
    jump:bool,
    item:bool,
}

impl Audio {
    pub fn new()->Self{
        let dir=std::env::temp_dir().join(format!("batman_cobblebot_native_{}",std::process::id()));
        let _=fs::create_dir_all(&dir);

        let music_path=dir.join("level1a.mp3");
        let jump_path=dir.join("jump.mp3");
        let item_path=dir.join("item.mp3");
        let music=fs::write(&music_path,include_bytes!("../assets/level1a_music.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_music",music_path.display()));
        let jump=fs::write(&jump_path,include_bytes!("../assets/jump.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_jump",jump_path.display()));
        let item=fs::write(&item_path,include_bytes!("../assets/item.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_item",item_path.display()));

        Self{dir,music,jump,item}
    }

    pub fn handle(&mut self,event:AudioEvent){
        match event{
            AudioEvent::MusicStart if self.music=>{
                let _=command("stop bcb_music");
                let _=command("seek bcb_music to start");
                let _=command("play bcb_music repeat");
            }
            AudioEvent::MusicStop if self.music=>{
                let _=command("stop bcb_music");
                let _=command("seek bcb_music to start");
            }
            AudioEvent::Jump if self.jump=>{
                let _=command("stop bcb_jump");
                let _=command("seek bcb_jump to start");
                let _=command("play bcb_jump");
            }
            AudioEvent::Item if self.item=>{
                let _=command("stop bcb_item");
                let _=command("seek bcb_item to start");
                let _=command("play bcb_item");
            }
            _=>{}
        }
    }
}

impl Drop for Audio{
    fn drop(&mut self){
        if self.music{let _=command("close bcb_music");}
        if self.jump{let _=command("close bcb_jump");}
        if self.item{let _=command("close bcb_item");}
        let _=fs::remove_dir_all(&self.dir);
    }
}
