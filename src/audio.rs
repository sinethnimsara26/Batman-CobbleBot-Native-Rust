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
    fn mciGetErrorStringW(error:u32,buffer:*mut u16,length:u32)->i32;
}

fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}

fn command(s:&str)->bool{
    unsafe{mciSendStringW(wide(s).as_ptr(),std::ptr::null_mut(),0,std::ptr::null_mut())==0}
}

// QA-ONLY: the MCI alias is owned by this Rust process. External Python
// cannot truthfully query it; expose read-only status inside the game only
// when the opt-in BCB_FOCUS_QA_LOG telemetry path is present.
fn mci_status(query:&str)->(String,u32){
    let mut reply=[0u16;128];
    let status=unsafe{mciSendStringW(
        wide(query).as_ptr(),reply.as_mut_ptr(),reply.len() as u32,
        std::ptr::null_mut()
    )};
    let end=reply.iter().position(|&u|u==0).unwrap_or(reply.len());
    (String::from_utf16_lossy(&reply[..end]).replace('\t'," ").replace('\r'," ").replace('\n'," "),status)
}

// The paused soundtrack must follow the fixed-step Intro animation.
// Failed MCI operations may not falsely change playback state.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
enum IntroTransport {
    #[default]
    Stopped,
    Playing,
    PausedForFocus,
}
impl IntroTransport {
    fn focus_command(self,active:bool)->Option<&'static str>{
        match (self,active) {
            (Self::Playing,false)=>Some("pause bcb_intro"),
            (Self::PausedForFocus,true)=>Some("resume bcb_intro"),
            _=>None,
        }
    }
    fn accept_focus_result(&mut self,active:bool,success:bool){
        if !success{return;}
        match (*self,active) {
            (Self::Playing,false)=>*self=Self::PausedForFocus,
            (Self::PausedForFocus,true)=>*self=Self::Playing,
            _=>{},
        }
    }
}
// The looping Level 1A soundtrack must pause with the fixed-step game when
// the window loses focus. Stop takes precedence over any pending resume.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
enum MusicTransport {
    #[default]
    Stopped,
    Playing,
    PausedForFocus,
}
impl MusicTransport {
    fn focus_command(self,active:bool)->Option<&'static str>{
        match (self,active) {
            (Self::Playing,false)=>Some("pause bcb_music"),
            (Self::PausedForFocus,true)=>Some("resume bcb_music"),
            _=>None,
        }
    }
    fn accept_focus_result(&mut self,active:bool,success:bool){
        if !success{return;}
        match (*self,active){
            (Self::Playing,false)=>*self=Self::PausedForFocus,
            (Self::PausedForFocus,true)=>*self=Self::Playing,
            _=>{},
        }
    }
    fn accept_start_result(&mut self,success:bool){
        *self=if success{Self::Playing}else{Self::Stopped};
    }
    fn stop(&mut self){*self=Self::Stopped;}
}

pub struct Audio {
    dir:PathBuf,
    intro:bool,
    intro_open_error:u32,
    intro_file_written:bool,
    intro_transport:IntroTransport,
    music_transport:MusicTransport,
    music:bool,
    jump:bool,
    item:bool,
}

impl Audio {
    pub fn new()->Self{
        let dir=std::env::temp_dir().join(format!("batman_cobblebot_native_{}",std::process::id()));
        let _=fs::create_dir_all(&dir);

        let intro_path=dir.join("intro.mp3");
        let music_path=dir.join("level1a.mp3");
        let jump_path=dir.join("jump.mp3");
        let item_path=dir.join("item.mp3");
        let intro_file_written=fs::write(&intro_path,include_bytes!("../assets/intro_sound.mp3")).is_ok();
        let intro_open_error=if intro_file_written {
            let line=format!("open \"{}\" type mpegvideo alias bcb_intro",intro_path.display());
            unsafe{mciSendStringW(wide(&line).as_ptr(),std::ptr::null_mut(),0,std::ptr::null_mut())}
        }else{u32::MAX};
        let intro=intro_file_written && intro_open_error==0;
        let music=fs::write(&music_path,include_bytes!("../assets/level1a_music.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_music",music_path.display()));
        let jump=fs::write(&jump_path,include_bytes!("../assets/jump.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_jump",jump_path.display()));
        let item=fs::write(&item_path,include_bytes!("../assets/item.mp3")).is_ok()
            && command(&format!("open \"{}\" type mpegvideo alias bcb_item",item_path.display()));

        Self{dir,intro,intro_open_error,intro_file_written,intro_transport:IntroTransport::Stopped,music_transport:MusicTransport::Stopped,music,jump,item}
    }

    /// Win32 stops the 25Hz simulation off-focus; both continuing MCI
    /// soundtracks must follow. One-shot jump/item effects remain unchanged.
    /// Failed MCI commands never fabricate an updated transport state.
    pub fn on_focus_changed(&mut self,active:bool){
        if self.intro {
            if let Some(mci)=self.intro_transport.focus_command(active){
                let success=command(mci);
                self.intro_transport.accept_focus_result(active,success);
            }
        }
        if self.music {
            if let Some(mci)=self.music_transport.focus_command(active){
                let success=command(mci);
                self.music_transport.accept_focus_result(active,success);
            }
        }
    }

    /// Read the real Windows MCI driver's mode, position and time format.
    /// Called solely by the opt-in offline QA trace; never on a normal run.
    pub fn qa_intro_open_diagnostics(&self)->(u32,String,bool){
        let mut buf=[0u16;256];
        let ok=unsafe{mciGetErrorStringW(self.intro_open_error,buf.as_mut_ptr(),buf.len() as u32)}!=0;
        let end=buf.iter().position(|&u|u==0).unwrap_or(buf.len());
        let message=if ok{String::from_utf16_lossy(&buf[..end])}
          else{"mciGetErrorStringW unavailable for code".into()};
        (self.intro_open_error,message.replace('\t'," ").replace('\r'," ").replace('\n'," "),self.intro_file_written)
    }

    pub fn qa_intro_mci_status(&self)->(String,String,String,u32,u32,u32){
        if !self.intro {
            return ("not_open".into(),"".into(),"".into(),1,1,1);
        }
        let (mode,em)=mci_status("status bcb_intro mode");
        let (position,ep)=mci_status("status bcb_intro position");
        let (format,ef)=mci_status("status bcb_intro time format");
        (mode,position,format,em,ep,ef)
    }

    pub fn qa_intro_transport(&self)->&'static str{
        match self.intro_transport{
            IntroTransport::Stopped=>"stopped",
            IntroTransport::Playing=>"playing",
            IntroTransport::PausedForFocus=>"paused_for_focus",
        }
    }

    pub fn handle(&mut self,event:AudioEvent){
        match event{
            AudioEvent::IntroStart if self.intro=>{
                let _=command("stop bcb_intro");
                let _=command("seek bcb_intro to start");
                // Original SWF StartSound #130 occurs on intro root frame 18.
                self.intro_transport=if command("play bcb_intro"){
                    IntroTransport::Playing
                }else{
                    IntroTransport::Stopped
                };
            }
            AudioEvent::IntroStop if self.intro=>{
                // A subsequent focus gain must never restart a stopped intro.
                self.intro_transport=IntroTransport::Stopped;
                let _=command("stop bcb_intro");
                let _=command("seek bcb_intro to start");
            }
            AudioEvent::MusicStart if self.music=>{
                let _=command("stop bcb_music");
                let _=command("seek bcb_music to start");
                self.music_transport.accept_start_result(command("play bcb_music repeat"));
            }
            AudioEvent::MusicStop if self.music=>{
                // Prevent any later focus gain from reviving a stopped song,
                // even if the MCI stop/seek itself fails.
                self.music_transport.stop();
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

#[cfg(test)]
mod tests {
    use super::{IntroTransport,MusicTransport};
    #[test]
    fn music_focus_pauses_resumes_only_successful_started_track(){
        let mut state=MusicTransport::Stopped;
        assert_eq!(state.focus_command(false),None);
        state.accept_start_result(false); // no device / failed play
        assert_eq!(state,MusicTransport::Stopped);
        assert_eq!(state.focus_command(true),None);

        state.accept_start_result(true);
        assert_eq!(state,MusicTransport::Playing);
        assert_eq!(state.focus_command(true),None);
        assert_eq!(state.focus_command(false),Some("pause bcb_music"));
        state.accept_focus_result(false,false);
        assert_eq!(state,MusicTransport::Playing);
        state.accept_focus_result(false,true);
        assert_eq!(state,MusicTransport::PausedForFocus);
        assert_eq!(state.focus_command(false),None);
        assert_eq!(state.focus_command(true),Some("resume bcb_music"));
        state.accept_focus_result(true,false);
        assert_eq!(state,MusicTransport::PausedForFocus);
        state.accept_focus_result(true,true);
        assert_eq!(state,MusicTransport::Playing);
        assert_eq!(state.focus_command(true),None);
    }

    #[test]
    fn music_stop_during_focus_pause_never_restores_old_loop(){
        let mut state=MusicTransport::Stopped;
        state.accept_start_result(true);
        state.accept_focus_result(false,true);
        assert_eq!(state,MusicTransport::PausedForFocus);
        state.stop();
        assert_eq!(state.focus_command(true),None);
        assert_eq!(state,MusicTransport::Stopped);
        state.accept_start_result(false);
        assert_eq!(state.focus_command(true),None);
        state.accept_start_result(true); // new scene/music explicit restart
        assert_eq!(state.focus_command(false),Some("pause bcb_music"));
        state.stop();
        assert_eq!(state.focus_command(true),None);
    }

    #[test]
    fn focus_pause_resume_is_idempotent_and_preserves_media_position(){
        let mut state=IntroTransport::Stopped;
        assert_eq!(state.focus_command(false),None);
        state=IntroTransport::Playing;
        assert_eq!(state.focus_command(true),None);
        assert_eq!(state.focus_command(false),Some("pause bcb_intro"));
        state.accept_focus_result(false,true);
        assert_eq!(state,IntroTransport::PausedForFocus);
        assert_eq!(state.focus_command(false),None);
        assert_eq!(state.focus_command(true),Some("resume bcb_intro"));
        state.accept_focus_result(true,true);
        assert_eq!(state,IntroTransport::Playing);
        assert_eq!(state.focus_command(true),None);
    }

    #[test]
    fn failed_mci_calls_and_scene_stop_do_not_invent_audio_resume(){
        let mut state=IntroTransport::Playing;
        state.accept_focus_result(false,false);
        assert_eq!(state,IntroTransport::Playing);
        state.accept_focus_result(false,true);
        state.accept_focus_result(true,false);
        assert_eq!(state,IntroTransport::PausedForFocus);
        state=IntroTransport::Stopped;
        assert_eq!(state.focus_command(true),None);
    }
}

impl Drop for Audio{
    fn drop(&mut self){
        if self.intro{let _=command("close bcb_intro");}
        if self.music{let _=command("close bcb_music");}
        if self.jump{let _=command("close bcb_jump");}
        if self.item{let _=command("close bcb_item");}
        let _=fs::remove_dir_all(&self.dir);
    }
}
