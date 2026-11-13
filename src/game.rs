use crate::bank_biker::{BankBiker, BikerEvent};
use crate::bank_goons::{BankGoonSquad, GoonCombatEvent, PlayerCombatContext};
use crate::collision::CollisionMask;
use crate::penguin::{
    PenguinBoss, PenguinCombatEvent, PenguinPlayerContext, PenguinPlayerHurtKind, PenguinRng,
    PenguinState, LEVEL1C_PICKUP_POINTS,
};
use crate::stage::{labs2a, LabsCameraMode, LabsTriggerPlacement, PendingStageEntry, RootProgress, StageId, StoryCard};

pub const LOGICAL_W:usize=600;
pub const LOGICAL_H:usize=400;
pub const FIXED_STEP:f32=1.0/25.0;

pub const VK_LEFT:usize=37; pub const VK_UP:usize=38; pub const VK_RIGHT:usize=39; pub const VK_DOWN:usize=40;
pub const VK_SPACE:usize=32; pub const VK_ENTER:usize=13; pub const VK_ESCAPE:usize=27; pub const VK_I:usize=73;
pub const VK_A:usize=65; pub const VK_D:usize=68; pub const VK_S:usize=83;

const PLAYER_W:f32=50.0;
const PLAYER_H:f32=75.0;
const CEILING_HEIGHT:f32=50.0;
const PLAYER_SPEED:f32=19.0;
const INITIAL_WALK_SLOWDOWN:f32=6.0;
const JUMP_VELOCITY:f32=-42.0;
const GLOBAL_GRAVITY:f32=5.0;
const GLIDE_GRAVITY:f32=1.0;

pub const LEVEL_X:f32=2467.65;
pub const LEVEL_Y:f32=239.25;
const GAME_ROOT_X:f32=26.1;
const GAME_ROOT_Y:f32=15.95;
const BG_INITIAL_Y:f32=-13.1;
#[allow(dead_code)]
const CAM_HEIGHT:f32=150.0;
#[allow(dead_code)]
const CAM_JUMP:f32=1.25;
#[allow(dead_code)]
const CAM_MIN:f32=-1350.0;
#[allow(dead_code)]
const CAM_MAX:f32=7650.0;
const CAM_OFFSET:f32=0.0;

pub const PICKUPS:[(f32,f32);7]=[
    (-803.05,19.85),(873.3,-9.0),(1682.9,73.45),(2546.5,62.35),
    (3396.0,112.3),(4448.1,133.4),(5534.95,162.25)
];

pub const LEVEL1B_PICKUPS:[(f32,f32);4]=[
    (-2068.1,138.5),(-1026.6,187.3),(-234.35,197.9),(732.15,182.45)
];

const CHECKPOINT_1B_01:CheckpointRect=CheckpointRect{min_x:98.152,min_y:102.475,max_x:364.689,max_y:398.098};
const CHECKPOINT_1B_02:CheckpointRect=CheckpointRect{min_x:364.689,min_y:110.725,max_x:969.362,max_y:406.348};
const CHECKPOINT_1B_03:CheckpointRect=CheckpointRect{min_x:-8243.564,min_y:124.725,max_x:-6683.608,max_y:420.348};

const OVERLAY_GO_LEFT:usize=10;
const OVERLAY_FIGHT_BTNS:usize=75;

#[derive(Copy,Clone)]
struct CheckpointRect{min_x:f32,min_y:f32,max_x:f32,max_y:f32}
impl CheckpointRect{
    fn hits_player(self,x:f32,y:f32)->bool{
        let p_min_x=x-PLAYER_W*0.5;
        let p_max_x=x+PLAYER_W*0.5;
        let p_min_y=y;
        let p_max_y=y+PLAYER_H;
        p_max_x>=self.min_x&&p_min_x<=self.max_x&&p_max_y>=self.min_y&&p_min_y<=self.max_y
    }
}

// Exact world-space bounds of the original symbol-153 checkpoint instances.
// They are gameplay controls only and are intentionally excluded from scenery.
const CP_WRONG:CheckpointRect=CheckpointRect{min_x:-1551.231,min_y:-35.775,max_x:-1389.000,max_y:259.848};
const CP_GLIDE1:CheckpointRect=CheckpointRect{min_x:3603.937,min_y:-51.791,max_x:3863.156,max_y:341.848};
const CP_NEXT:CheckpointRect=CheckpointRect{min_x:7556.890,min_y:484.575,max_x:11288.258,max_y:780.198};
const CP_GLIDE3:CheckpointRect=CheckpointRect{min_x:6818.906,min_y:-22.791,max_x:7120.649,max_y:370.848};
const CP_JUMP1:CheckpointRect=CheckpointRect{min_x:-611.231,min_y:-133.575,max_x:-449.000,max_y:162.048};
const CP_JUMP2:CheckpointRect=CheckpointRect{min_x:527.019,min_y:-133.575,max_x:689.250,max_y:162.048};
const CP_WALK:CheckpointRect=CheckpointRect{min_x:128.644,min_y:-129.575,max_x:401.112,max_y:166.048};

// Exact bounds of level-local pit symbol 149 after its original placement
// matrix (sx=2.1762237549, sy=5.7515563965, tx=-392.1, ty=1025.6).
const PIT_1A:CheckpointRect=CheckpointRect{
    min_x:-5246.276,min_y:450.444,max_x:4461.967,max_y:1601.043
};

// Original overlay (symbol 737) label starts, converted to zero-based frames.
const OVERLAY_BLANK:usize=0;
const OVERLAY_GO_RIGHT:usize=1;
const OVERLAY_JUMP:usize=20;
const OVERLAY_WRONGWAY:usize=30;
const OVERLAY_GLIDE_DOWN:usize=39;
const OVERLAY_WALK:usize=58;
const OVERLAY_TO_STREET:usize=140;
const ESCAPE_CARD_TERMINAL_FRAME:u16=224;
const ESCAPE_CARD_SOUND_ID:u16=130;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum AppScreen{Title,Instructions,Intro,Playing,EscapeCard}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum AudioEvent{IntroStart,IntroStop,MusicStart,MusicStop,Jump,Item,DoorOpen}

/// Runtime state of a single Labs sliding door (`#1325`/`#1324`), matching AVM1
/// 0-based timeline frames `0..=19` (`Stop()` at `0` closed & `9` open; sound `#1323` on `1`).
#[derive(Copy,Clone,Debug,PartialEq,Eq,Default)]
pub struct LabsDoorState {
    pub frame: u8,
    pub playing: bool,
}

/// Runtime state of a single Labs elevator (`#1328`/`#1330`), matching AVM1 `_global.elevatorLogic`.
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct LabsElevatorState {
    pub y: f32,
    pub dy: f32,
    pub moving: bool,
    pub active: bool,
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum PlayerState{
    Stand,Walk,Run,Jump,Fall,Glide,Land,Duck,Up,
    Punch,HighPunch,LowPunch,JumpPunch,
    Kick,HighKick,LowKick,JumpKick,
    Batarang,DuckBatarang,JumpBatarang,
    GrapplingUp,Grappling,CapeSpin,Hurt2,Hurt,Electro,Die,
}
impl PlayerState{
    pub fn range(self)->(usize,usize){match self{
        // These are the true child-movie timelines under outer Batman sprite
        // 691, not the outer label spans. Total native player frames = 279.
        Self::Stand=>(0,9),Self::Walk=>(9,21),Self::Run=>(21,31),
        Self::Jump=>(31,35),Self::Fall=>(35,40),Self::Glide=>(40,42),
        Self::Land=>(42,44),Self::Duck=>(44,49),Self::Up=>(49,51),
        Self::Punch=>(51,62),Self::HighPunch=>(62,73),Self::LowPunch=>(73,82),
        Self::JumpPunch=>(82,91),Self::Kick=>(91,99),Self::HighKick=>(99,107),
        Self::LowKick=>(107,122),Self::JumpKick=>(122,132),
        Self::Batarang=>(132,141),Self::DuckBatarang=>(141,153),Self::JumpBatarang=>(153,160),
        Self::GrapplingUp=>(160,170),Self::Grappling=>(170,198),Self::CapeSpin=>(198,207),
        Self::Hurt2=>(207,224),Self::Hurt=>(224,235),Self::Electro=>(235,256),Self::Die=>(256,279),
    }}

    fn loops(self)->bool{matches!(self,Self::Stand|Self::Walk|Self::Run)}

    fn timeline_completion(self)->Option<PlayerState>{match self{
        // Recovered from DoAction blocks at the final frames of the child clips.
        Self::Land=>Some(Self::Run),
        Self::Punch|Self::HighPunch|Self::Kick|Self::HighKick|
        Self::Batarang|Self::CapeSpin|Self::Hurt2|Self::Hurt|Self::Electro=>Some(Self::Stand),
        Self::LowPunch|Self::LowKick|Self::DuckBatarang=>Some(Self::Duck),
        Self::JumpPunch|Self::JumpKick|Self::JumpBatarang=>Some(Self::Fall),
        _=>None,
    }}

    fn display_tick(self,tick:u16)->usize{
        let(s,e)=self.range();
        let count=e-s;
        if self.loops(){
            s+tick as usize%count
        }else if self==Self::Grappling{
            // The original grappling child explicitly Stop()s on local frame 10
            // until swing logic advances it.
            s+(tick as usize).min(10)
        }else{
            s+(tick as usize).min(count-1)
        }
    }

    fn melee_ground_lock(self)->bool{
        matches!(self,Self::Punch|Self::LowPunch|Self::HighPunch|Self::Kick|Self::LowKick|Self::HighKick)
    }
    fn airborne_attack(self)->bool{
        matches!(self,Self::JumpPunch|Self::JumpKick|Self::JumpBatarang)
    }
    fn blocks_new_attack(self)->bool{
        matches!(self,Self::Batarang|Self::JumpBatarang|Self::Grappling|Self::Glide)
    }
}

#[derive(Clone)]
pub struct InputState{held:[bool;256],pressed:[bool;256]}
impl Default for InputState{fn default()->Self{Self{held:[false;256],pressed:[false;256]}}}
impl InputState{
    pub fn set(&mut self,key:usize,down:bool){
        if key>=256{return;}
        if down&&!self.held[key]{self.pressed[key]=true;}
        self.held[key]=down;
    }
    pub fn held(&self,key:usize)->bool{key<256&&self.held[key]}
    pub fn pressed(&self,key:usize)->bool{key<256&&self.pressed[key]}
    pub fn clear_pressed(&mut self){self.pressed.fill(false)}
    pub fn release_all(&mut self){self.held.fill(false);self.pressed.fill(false)}
}

pub struct Player{
    pub x:f32,pub y:f32,
    pub dx:f32,pub dy:f32,
    pub dir:i32,
    pub state:PlayerState,
    pub state_tick:u16,
    pub jumping:bool,
    pub on_ground:bool,
    walk_count:u16,
    walk_slowdown:f32,
    run_left:i16,
    run_right:i16,
}

pub struct Shot{pub x:f32,pub y:f32,pub dir:i32}

pub struct Game{
    // B04: real runtime remains 1A; the next scene is only a pending intent.
    stage:StageId,
    pending_stage_entry:Option<PendingStageEntry>,
    pub screen:AppScreen,
    /// Original root frame 18 intro sequence, 0-based at 25 fps.
    pub intro_tick:u16,
    /// The 41 original frames of Flash fadeout sprite #34, started by SKIP.
    pub intro_fadeout_tick:Option<u16>,
    intro_sound_stopped:bool,
    pub player:Player,
    pub camera_x:f32,pub camera_y:f32,
    pub background_x:f32,pub background_y:f32,
    labs_camera_mode:LabsCameraMode,
    cam_height:f32,
    on_platform:bool,
    current_platform_x:f32,
    current_platform_idx:Option<usize>,
    gravity:f32,
    pub batarangs:i32,
    pub score:i32,
    pub lives:i32,
    pub health:i32,
    pub player_attacking:bool,
    pub overlay_frame:usize,
    overlay_playing:bool,
    walk_checkpoint_started_tick:Option<u64>,
    pub level1a_exit_reached:bool,
    pub fadeout_tick:Option<u16>,
    pub level1a_complete:bool,
    pub level1b_exit_reached:bool,
    pub level1b_complete:bool,
    pub level1c_exit_reached:bool,
    pub level1c_complete:bool,
    /// Original root frame 22 (`card:escape`) story sequence (`0..=224` at 25 fps).
    pub escape_card_tick:u16,
    /// Original 41-frame fadeout sprite `#34` started from `#1267` terminal frame or SKIP `#1268`.
    pub escape_card_fadeout_tick:Option<u16>,
    escape_card_sound_stopped:bool,
    pub escape_card_complete:bool,
    pub bank_goons:Option<BankGoonSquad>,
    pub bank_biker:Option<BankBiker>,
    pub penguin_boss:Option<PenguinBoss>,
    penguin_rng:PenguinRng,
    pub pickups:[bool;7],
    pub shots:Vec<Shot>,
    pub labs_doors:Vec<LabsDoorState>,
    pub labs_elevators:Vec<LabsElevatorState>,
    pub ticks:u64,
    audio_events:Vec<AudioEvent>,
    pit_done:bool,
    respawn_after_fade:bool,
    gameover_after_fade:bool,
}

impl Game{
    pub fn new()->Self{
        Self{
            stage:StageId::Level1A,
            pending_stage_entry:None,
            screen:AppScreen::Title,
            intro_tick:0,
            intro_fadeout_tick:None,
            intro_sound_stopped:false,
            // Runtime collision uses level-local coordinates. The original
            // player transform is game-local, so subtract the level placement.
            player:Player{
                x:284.95-LEVEL_X,y:-661.5-LEVEL_Y,
                dx:0.0,dy:0.0,dir:1,
                state:PlayerState::Fall,state_tick:0,
                jumping:true,on_ground:false,
                walk_count:0,walk_slowdown:INITIAL_WALK_SLOWDOWN,
                run_left:0,run_right:0,
            },
            camera_x:GAME_ROOT_X,camera_y:GAME_ROOT_Y,
            background_x:0.0,background_y:BG_INITIAL_Y,
            labs_camera_mode:LabsCameraMode::CameraLogic2,
            cam_height:CAM_HEIGHT,
            on_platform:false,
            current_platform_x:0.0,
            current_platform_idx:None,
            gravity:GLOBAL_GRAVITY,
            batarangs:0,score:0,lives:5,health:100,player_attacking:false,
            overlay_frame:OVERLAY_BLANK,overlay_playing:false,walk_checkpoint_started_tick:None,level1a_exit_reached:false,fadeout_tick:None,level1a_complete:false,
            level1b_exit_reached:false,level1b_complete:false,
            level1c_exit_reached:false,level1c_complete:false,
            escape_card_tick:0,escape_card_fadeout_tick:None,escape_card_sound_stopped:false,escape_card_complete:false,
            bank_goons:None,bank_biker:None,penguin_boss:None,penguin_rng:PenguinRng::new(0x1C00_1250),
            pickups:[false;7],shots:Vec::new(),labs_doors:Vec::new(),labs_elevators:Vec::new(),ticks:0,audio_events:Vec::new(),pit_done:false,respawn_after_fade:false,gameover_after_fade:false,
        }
    }

    #[inline]
    pub fn level_origin_x(&self)->f32{
        self.stage.descriptor().level_origin_game_local.x
    }

    #[inline]
    pub fn level_origin_y(&self)->f32{
        self.stage.descriptor().level_origin_game_local.y
    }

    pub fn start_stage(&mut self,stage:StageId){
        self.stage=stage;
        self.pending_stage_entry=None;
        let desc=stage.descriptor();
        let (spawn_x,spawn_y)=desc.player_spawn_level_local();
        self.player.x=spawn_x;
        self.player.y=spawn_y;
        self.player.dir=desc.source_player_direction as i32;
        self.player.dx=0.0;
        self.player.dy=if matches!(stage,StageId::Level2A|StageId::Level2B){3.0}else{0.0};
        self.player.state=PlayerState::Fall;
        self.player.state_tick=0;
        self.player.jumping=true;
        self.player.on_ground=false;
        self.player.walk_count=0;
        self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
        self.player.run_left=0;
        self.player.run_right=0;
        self.health=100;
        self.fadeout_tick=None;
        self.camera_x=GAME_ROOT_X;
        self.camera_y=GAME_ROOT_Y;
        self.background_x=0.0;
        self.background_y=BG_INITIAL_Y;
        self.labs_camera_mode=LabsCameraMode::CameraLogic2;
        self.cam_height=desc.camera.height;
        self.on_platform=false;
        self.current_platform_x=0.0;
        self.overlay_frame=OVERLAY_BLANK;
        self.overlay_playing=false;
        self.walk_checkpoint_started_tick=None;
        self.pit_done=false;
        self.respawn_after_fade=false;
        self.gameover_after_fade=false;
        self.level1a_exit_reached=false;
        self.level1a_complete=false;
        self.level1b_exit_reached=false;
        self.level1b_complete=false;
        self.level1c_exit_reached=false;
        self.level1c_complete=false;
        self.escape_card_complete=false;
        self.pickups=[false;7];

        match stage{
            StageId::Level1A=>{
                self.bank_goons=None;
                self.bank_biker=None;
                self.penguin_boss=None;
            },
            StageId::Level1B=>{
                self.bank_goons=Some(BankGoonSquad::new());
                self.bank_biker=Some(BankBiker::new());
                self.penguin_boss=None;
            },
            StageId::Level1C=>{
                self.bank_goons=None;
                self.bank_biker=None;
                self.penguin_boss=Some(PenguinBoss::new());
                self.penguin_rng=PenguinRng::new(0x1C00_1250);
            },
            _=>{
                self.bank_goons=None;
                self.bank_biker=None;
                self.penguin_boss=None;
            }
        }
        self.labs_doors=if stage==StageId::Level2A{
            vec![LabsDoorState::default();crate::stage::labs2a::DOORS.len()]
        }else{
            Vec::new()
        };
        self.labs_elevators=if stage==StageId::Level2A{
            crate::stage::labs2a::ELEVATORS
                .iter()
                .map(|e|LabsElevatorState{y:e.y,dy:0.0,moving:false,active:true})
                .collect()
        }else{
            Vec::new()
        };
        self.current_platform_idx=None;
        self.gravity=GLOBAL_GRAVITY;
        self.update_camera();
    }

    #[cfg_attr(not(test), allow(dead_code))]
    #[inline]
    pub fn labs_camera_mode(&self)->LabsCameraMode{self.labs_camera_mode}

    #[cfg_attr(not(test), allow(dead_code))]
    #[inline]
    pub fn cam_height(&self)->f32{self.cam_height}

    #[cfg_attr(not(test), allow(dead_code))]
    #[inline]
    pub fn set_platform_state(&mut self,on_platform:bool,current_platform_x:f32){
        self.on_platform=on_platform;
        self.current_platform_x=current_platform_x;
    }

    /// The only loaded stage is 1A until its full successor pack is ready.
    pub fn active_stage(&self)->StageId{self.stage}

    /// B04 readonly handoff; does not make Stage1B playable or start its audio.
    pub fn pending_stage_entry(&self)->Option<PendingStageEntry>{self.pending_stage_entry}

    /// Consumes an implemented playable stage handoff (`Level1B` or `Level1C`),
    /// resetting stage actors while preserving carried `RootProgress` and restarting music.
    pub fn consume_pending_stage_entry(&mut self)->Option<StageId>{
        let entry=self.pending_stage_entry?;
        if !matches!(entry.to,StageId::Level1B|StageId::Level1C) {
            return None;
        }
        self.start_stage(entry.to);
        self.score=entry.progress.score;
        self.lives=entry.progress.lives;
        self.batarangs=entry.progress.batarangs;
        self.shots.clear();
        self.ticks=0;
        self.screen=AppScreen::Playing;
        self.audio_events.push(AudioEvent::MusicStart);
        Some(entry.to)
    }

    pub fn animation_frame(&self)->usize{
        self.player.state.display_tick(self.player.state_tick)
    }

    pub fn take_audio_events(&mut self)->Vec<AudioEvent>{
        std::mem::take(&mut self.audio_events)
    }

    fn enter(&mut self,state:PlayerState){
        if self.player.state!=state{
            self.player.state=state;
            self.player.state_tick=0;
            if matches!(state, PlayerState::Hurt | PlayerState::Hurt2 | PlayerState::Die) {
                self.player_attacking = false;
            }
        }
    }

    fn stop_intro_sound_once(&mut self){
        if !self.intro_sound_stopped {
            self.intro_sound_stopped=true;
            self.audio_events.push(AudioEvent::IntroStop);
        }
    }

    fn advance_intro_story(&mut self){
        if self.intro_tick<179 {
            self.intro_tick+=1;
            // Source sprite #123 frame179 stops its audio independently.
            if self.intro_tick==179 { self.stop_intro_sound_once(); }
        }
    }

    fn begin_level1a(&mut self){
        self.screen=AppScreen::Playing;
        self.intro_tick=0;
        self.intro_fadeout_tick=None;
        self.stop_intro_sound_once();
        self.audio_events.push(AudioEvent::MusicStart);
        // Original level entry has playerJumping=true and snaps the camera.
        self.update_camera();
    }

    /// Developer-only F7 target: use the same level-entry and neutral 25 Hz
    /// simulation as a normal player who enters Level 1A and waits six seconds.
    /// Called only from the Win32 --qa-navigation path; never used by the
    /// original intro / SKIP transition or normal gameplay.
    pub fn prepare_level1a_for_qa(&mut self,ground:&CollisionMask){
        self.begin_level1a();
        let mut neutral=InputState::default();
        for _ in 0..150 {
            self.tick(&mut neutral,ground);
        }
    }

    /// Source button129's release-inside event starts the original fadeout.
    /// No keyboard or mouse path may bypass its 41-frame transition.
    pub fn skip_intro(&mut self)->bool{
        if self.screen!=AppScreen::Intro || self.intro_fadeout_tick.is_some(){return false;}
        self.intro_fadeout_tick=Some(0);
        true
    }

    fn stop_escape_card_sound_once(&mut self){
        if !self.escape_card_sound_stopped {
            self.escape_card_sound_stopped=true;
            self.audio_events.push(AudioEvent::IntroStop);
        }
    }

    fn advance_escape_card_story(&mut self){
        if self.escape_card_tick<ESCAPE_CARD_TERMINAL_FRAME {
            self.escape_card_tick+=1;
        }
    }

    /// Source button #1268's release-inside event sets `gotoNext="level2a"` and plays fadeout `#34`.
    pub fn skip_escape_card(&mut self)->bool{
        if self.screen!=AppScreen::EscapeCard
            || self.escape_card_fadeout_tick.is_some()
            || self.escape_card_complete
        {
            return false;
        }
        self.escape_card_fadeout_tick=Some(0);
        true
    }

    fn begin_escape_card(&mut self){
        self.screen=AppScreen::EscapeCard;
        self.escape_card_tick=0;
        self.escape_card_fadeout_tick=None;
        self.escape_card_sound_stopped=false;
        self.escape_card_complete=false;
        self.fadeout_tick=None;
        self.level1c_complete=true;
        // Root frame 22 executes `StartSound 130` (same looping track as frame 18).
        self.audio_events.push(AudioEvent::IntroStart);
    }

    fn finish_escape_card_to_level2a(&mut self){
        self.stop_escape_card_sound_once();
        self.escape_card_fadeout_tick=Some(40);
        self.escape_card_complete=true;
        self.pending_stage_entry=PendingStageEntry::after_story_card(
            StoryCard::Escape,
            StageId::Level1C,
            ESCAPE_CARD_SOUND_ID,
            RootProgress{score:self.score,lives:self.lives,batarangs:self.batarangs},
        );
    }

    pub fn tick(&mut self,input:&mut InputState,ground:&CollisionMask){
        match self.screen{
            AppScreen::Title=>{
                if input.pressed(VK_I){self.screen=AppScreen::Instructions}
                else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE){
                    self.screen=AppScreen::Intro;
                    self.intro_tick=0;
                    self.intro_fadeout_tick=None;
                    self.intro_sound_stopped=false;
                    // StartSound id 130 is on original SWF root frame 18.
                    self.audio_events.push(AudioEvent::IntroStart);
                }
                input.clear_pressed();
                return
            },
            AppScreen::Instructions=>{
                if input.pressed(VK_ESCAPE){self.screen=AppScreen::Title}
                else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE){
                    self.screen=AppScreen::Intro;
                    self.intro_tick=0;
                    self.intro_fadeout_tick=None;
                    self.intro_sound_stopped=false;
                    // StartSound id 130 is on original SWF root frame 18.
                    self.audio_events.push(AudioEvent::IntroStart);
                }
                input.clear_pressed();
                return
            },
            AppScreen::Intro=>{
                // Root frame18 is stopped; button129 release-inside starts
                // fadeout sprite34. Its frame39 StopSounds precedes the
                // frame40 dispatcher jump to level1a.
                if let Some(frame)=self.intro_fadeout_tick{
                    self.advance_intro_story();
                    if frame>=40 {
                        self.begin_level1a();
                    }else{
                        let next=frame+1;
                        self.intro_fadeout_tick=Some(next);
                        if next==39 { self.stop_intro_sound_once(); }
                    }
                }else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE)||
                         input.pressed(VK_ESCAPE) {
                    let _=self.skip_intro();
                }else{
                    // Source sprite123's final frame is almost entirely black.
                    // Do not trap the user there indefinitely: display the
                    // terminal frame for one tick, then enter Level 1A. This
                    // native auto-completion is a user-requested usability
                    // correction; the original scripted auto-route is not
                    // established by the recovered Stop action alone.
                    let was_at_end=self.intro_tick==179;
                    self.advance_intro_story();
                    if was_at_end { self.begin_level1a(); }
                }
                input.clear_pressed();
                return
            },
            AppScreen::EscapeCard=>{
                // Root frame 22 plays #1267 (`225` frames, `0..=224`), holding
                // `#1254` at stopped frame `161`. Frame `224` executes
                // `stop(); _root.gotoNext = "level2a"; _root.fadeout.play();`.
                if !self.escape_card_complete {
                    if let Some(frame)=self.escape_card_fadeout_tick{
                        self.advance_escape_card_story();
                        if frame>=40 {
                            self.finish_escape_card_to_level2a();
                        }else{
                            let next=frame+1;
                            self.escape_card_fadeout_tick=Some(next);
                            if next==39 { self.stop_escape_card_sound_once(); }
                        }
                    }else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE)||input.pressed(VK_ESCAPE){
                        let _=self.skip_escape_card();
                    }else{
                        self.advance_escape_card_story();
                        if self.escape_card_tick>=ESCAPE_CARD_TERMINAL_FRAME {
                            self.escape_card_fadeout_tick=Some(0);
                        }
                    }
                }
                input.clear_pressed();
                return
            },
            AppScreen::Playing=>{}
        }

        self.ticks+=1;
        if self.advance_level_exit(){input.clear_pressed();return;}
        if self.level1a_complete{input.clear_pressed();return;}
        if self.level1b_complete{input.clear_pressed();return;}
        if self.level1c_complete{input.clear_pressed();return;}
        self.advance_tutorial_overlay();
        let state_at_tick_start=self.player.state;

        // Child movie clips in the Flash file normally return action states to
        // stand/fall. Until each child clip's embedded ActionScript is mapped
        // individually, use each recovered child movie's true timeline and final-frame action as
        // the completion clock instead of letting locomotion overwrite it.
        self.finish_timeline_action_if_needed(input);

        // Flash playerLogic applies LAST frame's dx/dy before reading this
        // frame's keys. This one-frame ordering is important to the feel.
        self.apply_previous_motion_and_collision(ground);

        // playerLogic zeros horizontal velocity after collision, except for
        // grappling and the 3px high-attack lunge.
        self.prepare_motion_for_input(ground);

        // Match the original input order: look -> horizontal -> combat ->
        // gadget -> jump/glide.
        self.process_look(input);
        self.process_horizontal(input);
        self.process_combat(input);
        self.process_gadget(input);
        self.process_jump_and_glide(input);

        match self.stage {
            StageId::Level1B => self.tick_level1b_enemies(ground),
            StageId::Level1C => self.tick_level1c_boss(ground),
            _ => {}
        }

        self.collect_pickups();
        self.update_shots();
        self.update_pit_logic();
        self.update_tutorial_checkpoints();
        self.update_labs_elevators();
        self.update_labs_camera_triggers();
        self.update_labs_doors();
        self.update_camera();

        if self.player.state==state_at_tick_start{
            self.player.state_tick=self.player.state_tick.saturating_add(1);
        }
        // Sprite 262 contains StartSound 248 on local frame 1.
        if self.player.state==PlayerState::Jump&&self.player.state_tick==1{
            self.audio_events.push(AudioEvent::Jump);
        }
        input.clear_pressed();
    }

    fn finish_timeline_action_if_needed(&mut self,input:&InputState){
        let state=self.player.state;

        // Original punch child (332) has a frame-4 action:
        // if _root.key_x is no longer latched (S was released), Stop(),
        // return to stand, and clear playerAttacking. Holding S lets the
        // remaining punch frames play through to the normal frame-10 exit.
        if state==PlayerState::Punch
            && self.player.state_tick==4
            && !input.held(VK_S)
        {
            self.player_attacking=false;
            self.enter(PlayerState::Stand);
            return;
        }

        let Some(next)=state.timeline_completion() else{return;};
        let(s,e)=state.range();
        if (self.player.state_tick as usize)<(e-s).saturating_sub(1){return;}

        if matches!(state,
            PlayerState::Punch|PlayerState::HighPunch|PlayerState::LowPunch|PlayerState::JumpPunch|
            PlayerState::Kick|PlayerState::HighKick|PlayerState::LowKick|PlayerState::JumpKick)
        {
            self.player_attacking=false;
        }
        self.enter(next);
    }

    fn apply_previous_motion_and_collision(&mut self,ground:&CollisionMask){
        // Horizontal movement first, exactly as playerLogic.
        if ground.check_wall_clear(self.player.x,self.player.y,PLAYER_H,self.player.dx){
            self.player.x+=self.player.dx;
        }

        // Vertical movement uses a 50px ceiling probe in the original, while
        // floor and wall probes use playerHeight=75.
        if self.player.jumping{
            if !ground.check_ceiling(
                self.player.x,&mut self.player.y,CEILING_HEIGHT,&mut self.player.dy,0
            ){
                self.player.y+=self.player.dy;
            }
        }else if !self.on_platform{
            self.player.y+=self.player.dy;
        }

        let grounded=ground.check_ground(
            self.player.x,&mut self.player.y,PLAYER_H,&mut self.player.dy,0
        ) || self.on_platform;
        self.player.on_ground=grounded;

        if !grounded{
            self.player.dy+=self.gravity;

            // The original lets Batman drift off a ledge for several frames
            // before playerJumping flips once downward speed exceeds 24.
            if matches!(self.player.state,PlayerState::Run|PlayerState::Walk|PlayerState::Stand)
                && self.player.dy>24.0
            {
                self.player.jumping=true;
                self.enter(PlayerState::Fall);
            }

            // Crucial glide behavior: every descending frame first returns to
            // fall and restores gravity=5. Held Space later in the SAME tick
            // can re-enter glide, halve dy again, and set gravity=1.
            if self.player.jumping
                && self.player.state!=PlayerState::Grappling
                && self.player.dy>5.0
                && !self.player.state.airborne_attack()
            {
                self.enter(PlayerState::Fall);
                self.gravity=GLOBAL_GRAVITY;
            }
        }else if self.player.jumping{
            self.player.jumping=false;
            self.enter(PlayerState::Land);
            self.player.walk_count=12;
            self.player.dx=0.0;
            self.player.dy=0.0;
            self.gravity=GLOBAL_GRAVITY;
        }
    }

    fn prepare_motion_for_input(&mut self,ground:&CollisionMask){
        if self.player.state!=PlayerState::Grappling
            && self.player.state!=PlayerState::HighPunch
            && self.player.state!=PlayerState::HighKick
        {
            self.player.dx=0.0;
        }

        if matches!(self.player.state,PlayerState::HighPunch|PlayerState::HighKick){
            let lunge=self.player.dir as f32*3.0;
            if ground.check_wall_clear(self.player.x,self.player.y,PLAYER_H,lunge){
                self.player.dx=lunge;
            }
        }
    }

    fn process_look(&mut self,input:&InputState){
        if input.held(VK_UP){
            if self.player.state!=PlayerState::Batarang
                && self.player.state!=PlayerState::JumpBatarang
                && !self.player.jumping
                && self.player.state!=PlayerState::HighKick
                && self.player.state!=PlayerState::HighPunch
            {
                self.enter(PlayerState::Up);
            }
        }else if self.player.state==PlayerState::Up{
            self.enter(PlayerState::Stand);
        }

        if input.held(VK_DOWN){
            if !self.player.jumping
                && self.player.state!=PlayerState::Batarang
                && self.player.state!=PlayerState::JumpBatarang
                && self.player.state!=PlayerState::DuckBatarang
                && self.player.state!=PlayerState::LowKick
                && self.player.state!=PlayerState::LowPunch
            {
                self.enter(PlayerState::Duck);
            }
        }else if self.player.state==PlayerState::Duck{
            self.enter(PlayerState::Stand);
        }
    }

    fn process_horizontal(&mut self,input:&InputState){
        let left=input.held(VK_LEFT)&&!input.held(VK_RIGHT);
        let right=input.held(VK_RIGHT)&&!input.held(VK_LEFT);

        if left{
            if self.player.run_left>15&&self.player.run_left<20{
                self.player.walk_count=12;
            }
            self.player.run_left=20;
            self.horizontal_direction(-1);
        }else{
            self.player.run_left=(self.player.run_left-1).max(0);
        }

        if right{
            if self.player.run_right>15&&self.player.run_right<20{
                self.player.walk_count=12;
            }
            self.player.run_right=20;
            self.horizontal_direction(1);
        }else{
            self.player.run_right=(self.player.run_right-1).max(0);
        }

        if !left&&!right{
            self.player.walk_count=0;
            self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
            if !self.player.jumping&&matches!(self.player.state,PlayerState::Walk|PlayerState::Run){
                self.enter(PlayerState::Stand);
            }
        }
    }

    fn horizontal_direction(&mut self,dir:i32){
        // The original blocks ground melee locomotion but still permits
        // airborne punch/kick movement.
        if self.player.state.melee_ground_lock(){return;}

        if !self.player.jumping
            && self.player.state!=PlayerState::Batarang
            && self.player.state!=PlayerState::JumpBatarang
            && self.player.state!=PlayerState::Grappling
            && self.player.state!=PlayerState::Glide
        {
            if self.player.walk_count<8{
                self.enter(PlayerState::Walk);
                self.player.walk_count+=1;
            }else{
                self.enter(PlayerState::Run);
            }
        }

        let restricted=matches!(
            self.player.state,
            PlayerState::Batarang|PlayerState::JumpBatarang|PlayerState::Grappling
        );
        if !restricted||(restricted&&self.player.jumping){
            if self.player.state==PlayerState::Walk{
                self.player.dx=dir as f32*(PLAYER_SPEED-self.player.walk_slowdown);
                self.player.walk_slowdown-=0.5;
            }else{
                self.player.dx=dir as f32*PLAYER_SPEED;
                self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
            }
        }
        self.player.dir=dir;
    }

    fn process_combat(&mut self,input:&InputState){
        let both=input.pressed(VK_D)&&input.pressed(VK_S);
        if both&&!self.player.jumping&&!self.player.state.blocks_new_attack(){
            self.enter(PlayerState::CapeSpin);
            self.player.walk_count=0;
            self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
            return;
        }

        if input.pressed(VK_D){
            self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
            if !self.player.state.blocks_new_attack(){
                let state=if self.player.jumping{PlayerState::JumpKick}
                else if input.held(VK_UP){PlayerState::HighKick}
                else if input.held(VK_DOWN){PlayerState::LowKick}
                else{PlayerState::Kick};
                self.enter(state);
                self.player_attacking=true;
            }
            self.player.walk_count=0;
        }

        if input.pressed(VK_S){
            self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
            if !self.player.state.blocks_new_attack(){
                let state=if self.player.jumping{PlayerState::JumpPunch}
                else if input.held(VK_UP){PlayerState::HighPunch}
                else if input.held(VK_DOWN){PlayerState::LowPunch}
                else{PlayerState::Punch};
                self.enter(state);
                self.player_attacking=true;
            }
            self.player.walk_count=0;
        }
    }

    fn process_gadget(&mut self,input:&InputState){
        if !input.pressed(VK_A){return;}
        self.player.walk_slowdown=INITIAL_WALK_SLOWDOWN;
        if self.player.state.blocks_new_attack()||self.batarangs<=0{
            self.player.walk_count=0;
            return;
        }

        let game_x=self.player.x+self.level_origin_x();
        let game_y=self.player.y+self.level_origin_y();
        let (state,shot_y)=if self.player.jumping{
            (PlayerState::JumpBatarang,game_y-20.0+self.player.dy)
        }else if self.player.state==PlayerState::Duck{
            (PlayerState::DuckBatarang,game_y+20.0)
        }else{
            (PlayerState::Batarang,game_y-20.0)
        };

        self.shots.push(Shot{x:game_x,y:shot_y,dir:self.player.dir});
        self.batarangs-=1;
        self.enter(state);
        if !self.player.jumping{self.player.dx=0.0;}
        self.player.walk_count=0;
    }

    fn process_jump_and_glide(&mut self,input:&InputState){
        if input.pressed(VK_SPACE){
            if !self.player.jumping{
                self.player.jumping=true;
                self.enter(PlayerState::Jump);
                self.player.dy=JUMP_VELOCITY;
                self.gravity=GLOBAL_GRAVITY;
                self.player.walk_count=12;
            }
        }else if input.held(VK_SPACE)
            && self.player.jumping
            && self.player.state!=PlayerState::Glide
            && self.player.dy>5.0
        {
            self.enter(PlayerState::Glide);
            self.player.dy*=0.5;
            self.gravity=GLIDE_GRAVITY;
        }
    }

    fn advance_level_exit(&mut self)->bool{
        let Some(t)=self.fadeout_tick else{return false;};
        if self.level1c_exit_reached && !self.respawn_after_fade && !self.gameover_after_fade {
            let state_name = self.player_combat_state_name();
            if let Some(boss) = &mut self.penguin_boss {
                if boss.state == PenguinState::Die {
                    boss.step_child_frame(state_name, &mut self.penguin_rng);
                }
            }
        }
        if t<40{
            let next=t+1;
            self.fadeout_tick=Some(next);
            // SWF fadeout sprite #34 (zero-based frame 39) dispatches
            // StopSounds before root's destination-frame jump at frame 40.
            // Change only the completed exits: pit/continue paths retain
            // their established native behavior and regression contract.
            if next==39 && (self.level1a_exit_reached || self.level1b_exit_reached || self.level1c_exit_reached) {
                self.audio_events.push(AudioEvent::MusicStop);
            }
            return false;
        }

        self.fadeout_tick=Some(40);
        if self.respawn_after_fade{
            // pitLogic routes through root "continue" with gotoLast="start".
            // Restart the same stage while preserving root values.
            let stage=self.stage;
            let lives=self.lives;
            let score=self.score;
            let batarangs=self.batarangs;
            let events=std::mem::take(&mut self.audio_events);
            let mut fresh=Game::new();
            fresh.screen=AppScreen::Playing;
            if stage!=StageId::Level1A {
                fresh.start_stage(stage);
            }
            fresh.lives=lives;
            fresh.score=score;
            fresh.batarangs=batarangs;
            fresh.audio_events=events;
            fresh.update_camera();
            *self=fresh;
            return true;
        }
        if self.gameover_after_fade{
            self.audio_events.push(AudioEvent::MusicStop);
            self.screen=AppScreen::Title;
            self.fadeout_tick=None;
            self.gameover_after_fade=false;
            self.ticks=0;
            return true;
        }

        if self.level1a_exit_reached {
            // Original root19 sets gotoNext=level1b, then plays #34.
            self.pending_stage_entry=PendingStageEntry::after_source_fade(
                self.stage,
                RootProgress{score:self.score,lives:self.lives,batarangs:self.batarangs},
            );
            self.level1a_complete=true;
        }else if self.level1b_exit_reached {
            // Original root20 sets gotoNext=level1c, then plays #34.
            self.pending_stage_entry=PendingStageEntry::after_source_fade(
                self.stage,
                RootProgress{score:self.score,lives:self.lives,batarangs:self.batarangs},
            );
            self.level1b_complete=true;
        }else if self.level1c_exit_reached {
            // Original penguinAI sets gotoNext="card:escape", then plays #34.
            self.begin_escape_card();
        }else{
            // Other non-pit terminal fades preserve the old sound behavior.
            self.audio_events.push(AudioEvent::MusicStop);
            self.level1a_complete=true;
        }
        true
    }

    fn overlay_goto_and_play(&mut self,frame:usize){
        self.overlay_frame=frame;
        self.overlay_playing=true;
    }

    fn advance_tutorial_overlay(&mut self){
        if !self.overlay_playing{return;}
        // Frame scripts on the last frame of each original tutorial segment
        // jump back to label "blank". The game-over segment is not used in
        // Level 1A and is therefore deliberately left out of this list.
        if matches!(self.overlay_frame,9|19|29|38|47|65|74|83|92|101|139|148){
            self.overlay_frame=OVERLAY_BLANK;
            self.overlay_playing=false;
        }else{
            self.overlay_frame=(self.overlay_frame+1).min(148);
        }
    }

    fn level_title_finished(&self)->bool{
        // Native frame 99 is Flash _currentframe 100 and Stop()s there.
        self.ticks>=99
    }

    fn player_combat_state_name(&self) -> &'static str {
        match self.player.state {
            PlayerState::Punch => "punch",
            PlayerState::HighPunch => "highpunch",
            PlayerState::LowPunch => "lowpunch",
            PlayerState::JumpPunch => "jumppunch",
            PlayerState::Kick => "kick",
            PlayerState::HighKick => "highkick",
            PlayerState::LowKick => "lowkick",
            PlayerState::JumpKick => "jumpkick",
            PlayerState::CapeSpin => "capespin",
            PlayerState::Hurt | PlayerState::Hurt2 => "hurt",
            PlayerState::Electro => "electro",
            PlayerState::Fall => "fall",
            PlayerState::Die => "die",
            _ => "other",
        }
    }

    fn tick_level1b_enemies(&mut self, ground: &CollisionMask) {
        let player_x_game = self.player.x + self.level_origin_x();
        let player_y_game = self.player.y + self.level_origin_y();
        let is_hurt = matches!(self.player.state, PlayerState::Hurt | PlayerState::Hurt2) || self.fadeout_tick.is_some();
        let state_name = self.player_combat_state_name();
        let player_ctx = PlayerCombatContext {
            x_game: player_x_game,
            y_game: player_y_game,
            dir: self.player.dir,
            is_attacking: self.player_attacking,
            is_invincible: is_hurt || self.player.state == PlayerState::Die,
            state_name,
        };

        let mut goon_events = Vec::new();
        if let Some(squad) = &mut self.bank_goons {
            let batarangs: Vec<(f32, f32)> = self.shots.iter().map(|s| (s.x, s.y)).collect();
            squad.tick_combat(&player_ctx, &batarangs, ground, &mut goon_events);
        }

        let mut biker_events = Vec::new();
        if let Some(biker) = &mut self.bank_biker {
            let batarangs: Vec<(f32, f32, bool)> = self.shots.iter().map(|s| (s.x, s.y, true)).collect();
            biker.tick(player_x_game, player_y_game, self.player_attacking, is_hurt, &batarangs, &mut biker_events);
        }

        for ev in goon_events {
            match ev {
                GoonCombatEvent::PlayerHurt { damage } => {
                    self.apply_player_damage(damage);
                }
                GoonCombatEvent::Score(pts) => {
                    self.score += pts as i32;
                }
                GoonCombatEvent::StageComplete => {
                    if !self.level1b_exit_reached && self.fadeout_tick.is_none() {
                        self.level1b_exit_reached = true;
                        self.fadeout_tick = Some(0);
                        self.player.dx = 0.0;
                        self.player.dy = 0.0;
                    }
                }
                _ => {}
            }
        }

        for ev in biker_events {
            match ev {
                BikerEvent::PlayerHurt { damage } => {
                    self.apply_player_damage(damage as i32);
                }
                BikerEvent::Score(pts) => {
                    self.score += pts as i32;
                }
                _ => {}
            }
        }
    }

    fn tick_level1c_boss(&mut self, ground: &CollisionMask) {
        let player_x_game = self.player.x + self.level_origin_x();
        let player_y_game = self.player.y + self.level_origin_y();
        let is_hurt = matches!(self.player.state, PlayerState::Hurt | PlayerState::Hurt2) || self.fadeout_tick.is_some();
        let state_name = self.player_combat_state_name();
        let player_ctx = PenguinPlayerContext {
            x_game: player_x_game,
            y_game: player_y_game,
            dx: self.player.dx,
            dir: self.player.dir.clamp(-1, 1) as i8,
            width: PLAYER_W,
            attacking: self.player_attacking,
            invincible: is_hurt || self.player.state == PlayerState::Die,
            state_name,
        };

        let mut boss_events = Vec::new();
        if let Some(boss) = &mut self.penguin_boss {
            let batarangs: Vec<(f32, f32)> = self.shots.iter().map(|s| (s.x, s.y)).collect();
            boss.tick_combat(&player_ctx, &batarangs, ground, &mut self.penguin_rng, &mut boss_events);
        }

        let mut hidden_indices = Vec::new();
        for ev in boss_events {
            match ev {
                PenguinCombatEvent::ZeroPlayerDx => {
                    self.player.dx = 0.0;
                }
                PenguinCombatEvent::HideBatarang { index } => {
                    hidden_indices.push(index);
                }
                PenguinCombatEvent::Score(pts) => {
                    self.score += pts;
                }
                PenguinCombatEvent::PlayerHit { damage, kind } => {
                    let hurt_state = match kind {
                        PenguinPlayerHurtKind::MeleeHurt => PlayerState::Hurt,
                        PenguinPlayerHurtKind::LaserElectro => PlayerState::Electro,
                    };
                    self.apply_player_damage_with_state(damage, hurt_state);
                }
                PenguinCombatEvent::DefeatedTrigger => {
                    if !self.level1c_exit_reached && self.fadeout_tick.is_none() {
                        self.level1c_exit_reached = true;
                        self.fadeout_tick = Some(0);
                        self.player.dx = 0.0;
                        self.player.dy = 0.0;
                    }
                }
                PenguinCombatEvent::Sound(_) | PenguinCombatEvent::ComboHit { .. } => {}
            }
        }
        if !hidden_indices.is_empty() {
            hidden_indices.sort_unstable();
            hidden_indices.dedup();
            for index in hidden_indices.into_iter().rev() {
                if index < self.shots.len() {
                    self.shots.remove(index);
                }
            }
        }
    }

    fn apply_player_damage(&mut self, damage: i32) {
        self.apply_player_damage_with_state(damage, PlayerState::Hurt);
    }

    fn apply_player_damage_with_state(&mut self, damage: i32, hurt_state: PlayerState) {
        if self.fadeout_tick.is_some() || self.player.state == PlayerState::Die {
            return;
        }
        self.health = self.health.saturating_sub(damage);
        if self.health <= 0 {
            self.health = 0;
            self.enter(PlayerState::Die);
            if self.lives > 0 {
                self.lives -= 1;
                self.respawn_after_fade = true;
            } else {
                self.gameover_after_fade = true;
            }
            self.fadeout_tick = Some(0);
            self.player.dx = 0.0;
            self.player.dy = 0.0;
        } else {
            self.enter(hurt_state);
            self.player.walk_count = 0;
            self.player.walk_slowdown = INITIAL_WALK_SLOWDOWN;
        }
    }

    fn update_pit_logic(&mut self){
        let hit = match self.stage {
            StageId::Level1A => PIT_1A.hits_player(self.player.x, self.player.y),
            StageId::Level1B => (self.player.y + self.level_origin_y()) > 400.0,
            _ => false,
        };
        if !hit{
            self.pit_done=false;
            return;
        }
        if self.pit_done||self.fadeout_tick.is_some(){return;}

        // Exact pitLogic routing recovered from AVM1:
        // if (_root.lives > 0) { lives--; gotoNext="continue"; }
        // else { gotoNext="gameover"; }
        // gotoLast="start"; fadeout.play();
        self.pit_done=true;
        if self.lives>0{
            self.lives-=1;
            self.respawn_after_fade=true;
        }else{
            self.gameover_after_fade=true;
        }
        self.fadeout_tick=Some(0);
        self.player.dx=0.0;
        self.player.dy=0.0;
    }

    fn update_tutorial_checkpoints(&mut self){
        let x=self.player.x+self.level_origin_x();
        let y=self.player.y+self.level_origin_y();

        match self.stage {
            StageId::Level1A => {
                let title_done=self.level_title_finished();

                if title_done&&CP_WRONG.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_WRONGWAY);
                }

                if title_done&&CP_WALK.hits_player(x,y){
                    let started=*self.walk_checkpoint_started_tick.get_or_insert(self.ticks);
                    let elapsed_ms=self.ticks.saturating_sub(started)*40;
                    if elapsed_ms<3000{
                        self.overlay_goto_and_play(OVERLAY_GO_RIGHT);
                    }else if elapsed_ms>3000&&elapsed_ms<6000{
                        self.overlay_goto_and_play(OVERLAY_WALK);
                    }
                }

                if title_done&&(CP_JUMP1.hits_player(x,y)||CP_JUMP2.hits_player(x,y)){
                    self.overlay_goto_and_play(OVERLAY_JUMP);
                }

                if CP_GLIDE1.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_GLIDE_DOWN);
                }
                if CP_GLIDE3.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_TO_STREET);
                }

                if CP_NEXT.hits_player(x,y)&&self.fadeout_tick.is_none(){
                    // Source gotoNext="level1b" starts fadeout sprite #34. B04
                    // snapshots a pending entry only AFTER the complete 41 frames;
                    // renderer/collision still show the final 1A frame.
                    self.level1a_exit_reached=true;
                    self.fadeout_tick=Some(0);
                    self.player.dx=0.0;
                    self.player.dy=0.0;
                }
            }
            StageId::Level1B => {
                if CHECKPOINT_1B_01.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_FIGHT_BTNS);
                }
                if CHECKPOINT_1B_02.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_GO_LEFT);
                }
                if CHECKPOINT_1B_03.hits_player(x,y){
                    self.overlay_goto_and_play(OVERLAY_GO_RIGHT);
                }
            }
            _ => {}
        }
    }

    fn collect_pickups(&mut self){
        let px=self.player.x+self.level_origin_x();
        let py=self.player.y+self.level_origin_y();
        let pickups_slice: &[(f32,f32)] = match self.stage {
            StageId::Level1A => &PICKUPS,
            StageId::Level1B => &LEVEL1B_PICKUPS,
            StageId::Level1C => &LEVEL1C_PICKUP_POINTS,
            _ => &[],
        };
        for(i,&(x,y))in pickups_slice.iter().enumerate(){
            if i < self.pickups.len() && self.pickups[i]{continue;}
            // Native approximation of Flash movie-clip hitTest bounds. The
            // art itself is original; exact alpha hit testing can replace this
            // once the pickup registration box is extracted.
            if(px-x).abs()<45.0&&(py-y).abs()<55.0{
                self.pickups[i]=true;
                self.batarangs+=5;
                self.score+=5;
                self.audio_events.push(AudioEvent::Item);
            }
        }
    }

    fn update_shots(&mut self){
        let px=self.player.x+self.level_origin_x();
        for shot in &mut self.shots{shot.x+=shot.dir as f32*50.0;}
        self.shots.retain(|shot|(shot.x-px).abs()<500.0);
    }

    fn labs_camera_triggers(&self)->&'static [LabsTriggerPlacement]{
        match self.stage{
            StageId::Level2A=>&labs2a::CAMERAS,
            _=>&[],
        }
    }

    fn update_labs_elevators(&mut self){
        if self.stage!=StageId::Level2A||self.labs_elevators.is_empty(){return;}
        let placements=&crate::stage::labs2a::ELEVATORS;
        if self.labs_elevators.len()!=placements.len(){return;}

        let origin_x=self.level_origin_x();
        let origin_y=self.level_origin_y();
        for (idx,(state,&elev)) in self.labs_elevators.iter_mut().zip(placements.iter()).enumerate(){
            if !state.active{continue;}
            let player_x=self.player.x+origin_x;
            let player_y=self.player.y+origin_y;
            let hits=elev.hits_player_at(state.y,player_x,player_y);
            let limit=elev.limit_y as f32;

            // Exact AVM1 _global.elevatorLogic (root frame 15): mount, lockstep carry,
            // jump-off (-36 + dy/3), and limit reach with `delete this.onEnterFrame`.
            if !self.on_platform{
                if hits && self.player.dy>=0.0{
                    if (state.y-limit).abs()>f32::EPSILON{
                        state.moving=true;
                    }
                    if state.y<limit{
                        state.dy=12.0;
                        // Keep slow-gliding feet inside #1327 hitBox on mount tick before elevator descends.
                        self.player.y+=state.dy;
                    }else{
                        self.player.y-=32.0;
                        state.dy=-12.0;
                    }
                    self.current_platform_idx=Some(idx);
                    self.current_platform_x=elev.x;
                    self.on_platform=true;
                }
            }else if self.current_platform_idx==Some(idx){
                if hits{
                    self.on_platform=true;
                    self.current_platform_x=elev.x;
                    if !self.player.jumping{
                        self.player.dy=state.dy;
                        self.gravity=0.0;
                        let snap=if state.dy>0.0{20.0}else{40.0};
                        self.player.y=(state.y-PLAYER_H-snap)-origin_y;
                    }else{
                        self.on_platform=false;
                        self.current_platform_idx=None;
                        self.player.dy=-36.0+state.dy/3.0;
                        self.gravity=GLOBAL_GRAVITY;
                    }
                }else if (player_y-state.y).abs()>160.0 || !self.player.jumping{
                    self.on_platform=false;
                    self.current_platform_idx=None;
                    self.gravity=GLOBAL_GRAVITY;
                }
            }

            if (state.dy>0.0 && state.y>=limit) || (state.dy<0.0 && state.y<=limit){
                state.dy=0.0;
                state.y=limit;
                state.moving=false;
                state.active=false;
                if self.current_platform_idx==Some(idx){
                    self.on_platform=false;
                    self.current_platform_idx=None;
                    self.player.dy=0.0;
                    self.gravity=GLOBAL_GRAVITY;
                }
            }

            if (state.dy<0.0 && state.y>limit) || (state.dy>0.0 && state.y<limit){
                state.y+=state.dy;
            }
        }
    }

    fn update_labs_doors(&mut self){
        if self.stage!=StageId::Level2A||self.labs_doors.is_empty(){return;}
        let placements=&crate::stage::labs2a::DOORS;
        if self.labs_doors.len()!=placements.len(){return;}

        let player_x=self.player.x+self.level_origin_x();
        let player_y=self.player.y+self.level_origin_y();
        for (state,&door) in self.labs_doors.iter_mut().zip(placements.iter()){
            // AVM1 _global.doorOpen: overlapping #1325 opens closed doors (frame 0 -> "open");
            // leaving #1325 closes open doors (frame 9 -> "close"). While overlapping, the jump
            // past the close branch keeps the door open even if the player lingers inside.
            if door.hits_player(player_x,player_y){
                if state.frame==0 && !state.playing{
                    state.playing=true;
                }
            }else if state.frame==9 && !state.playing{
                state.playing=true;
            }

            if state.playing{
                state.frame=(state.frame+1)%20;
                if state.frame==1{
                    self.audio_events.push(AudioEvent::DoorOpen);
                }
                if state.frame==0 || state.frame==9{
                    state.playing=false;
                }
            }
        }
    }

    fn update_labs_camera_triggers(&mut self){
        // Original frame 23/25 camera_01..10 triggers switch game.onEnterFrame only
        // when grounded (!playerJumping) and overlapping the #153 trigger box.
        if self.player.jumping{return;}
        let triggers=self.labs_camera_triggers();
        if triggers.is_empty(){return;}

        let player_x=self.player.x+self.level_origin_x();
        let player_y=self.player.y+self.level_origin_y();
        let cam3_offset=if self.stage==StageId::Level2B{550.0}else{500.0};

        for &trigger in triggers{
            if trigger.hits_player(player_x,player_y){
                match trigger.camera_mode(){
                    LabsCameraMode::CameraLogic2=>{
                        self.labs_camera_mode=LabsCameraMode::CameraLogic2;
                        self.cam_height=self.stage.descriptor().camera.height;
                    },
                    LabsCameraMode::CameraLogic3=>{
                        self.labs_camera_mode=LabsCameraMode::CameraLogic3;
                        self.cam_height=self.camera_y-player_y+cam3_offset;
                    },
                }
            }
        }
    }

    fn apply_camera_horizontal_ease(&mut self,cam:crate::stage::SourceCamera,player_x:f32,target_x:f32){
        let player_position_x=player_x+self.camera_x;
        if player_x>cam.min_x&&player_x<cam.max_x{
            let delta=player_position_x-target_x;
            if delta.abs()>4.0{
                let ease=(delta/10.0).trunc();
                self.camera_x-=ease;
                self.background_x+=ease/2.0;
            }
        }
    }

    fn update_camera(&mut self){
        let cam=self.stage.descriptor().camera;
        let bg_width=2472.0_f32;
        let bg_diff=self.background_x+self.camera_x;
        if bg_diff>0.0{
            self.background_x-=bg_width/2.0;
        }else if bg_diff< -(bg_width/2.0){
            self.background_x+=bg_width/2.0;
        }

        let player_x=self.player.x+self.level_origin_x();
        let player_y=self.player.y+self.level_origin_y();

        if matches!(self.stage,StageId::Level2A|StageId::Level2B){
            match self.labs_camera_mode{
                LabsCameraMode::CameraLogic2=>{
                    let target_x=if self.on_platform{
                        300.0+(player_x-self.current_platform_x)
                    }else if self.player.dir>0{
                        125.0
                    }else{
                        475.0
                    };
                    self.apply_camera_horizontal_ease(cam,player_x,target_x);
                    self.camera_y=self.cam_height-player_y;
                },
                LabsCameraMode::CameraLogic3=>{
                    let target_x=if self.player.dir>0{150.0}else{450.0};
                    self.apply_camera_horizontal_ease(cam,player_x,target_x);
                    let target=(self.cam_height-player_y)/cam.jump;
                    if !self.player.jumping{
                        self.camera_y=target;
                    }else if (self.camera_y+player_y-150.0).abs()>1.0{
                        self.camera_y+=((target-self.camera_y)/4.0).trunc();
                    }
                },
            }
            return;
        }

        // Exact translation of the original cameraLogic() recovered from the
        // decompiled ActionScript for Bank stages (1A/1B/1C).
        let target_x=if self.player.dir>0{125.0}else{475.0};
        self.apply_camera_horizontal_ease(cam,player_x,target_x);

        if self.player.jumping{
            self.camera_y=(cam.height-player_y)/cam.jump;
            self.background_y=(player_y/2.0-cam.height/1.5)/cam.jump-CAM_OFFSET;
        }else if (self.camera_y+player_y-150.0).abs()>1.0{
            let target=(cam.height-player_y)/cam.jump;
            self.camera_y+=((target-self.camera_y)/4.0).trunc();
            self.background_y=((player_y/2.0-cam.height/1.5)/cam.jump).trunc()-CAM_OFFSET;
        }
    }
}


#[cfg(test)]
mod b04_pending_transition_tests {
    use super::{AppScreen,AudioEvent,Game,InputState,LEVEL_X,LEVEL_Y};
    use crate::collision::CollisionMask;
    use crate::stage::{PendingStageEntry,RootProgress,StageId};

    #[test]
    fn level1a_exit_has_exact_fade_stop_cue_and_frozen_root_handoff() {
        let ground=CollisionMask::level1a();
        let mut game=Game::new();
        let mut input=InputState::default();
        game.screen=AppScreen::Playing;
        game.score=1234;
        game.lives=3;
        game.batarangs=17;
        game.health=42; // Stage-local health MUST NOT travel with the root stats.
        game.player.x=7600.0-LEVEL_X;
        game.player.y=520.0-LEVEL_Y;
        game.tick(&mut input,&ground);
        assert_eq!(game.fadeout_tick,Some(0));
        assert!(game.level1a_exit_reached);
        assert_eq!(game.pending_stage_entry(),None);
        assert!(game.take_audio_events().is_empty());
        for i in 1..=38 {
            game.tick(&mut input,&ground);
            assert_eq!(game.fadeout_tick,Some(i));
            assert_eq!(game.pending_stage_entry(),None);
            assert!(game.take_audio_events().is_empty());
        }
        game.tick(&mut input,&ground); // sprite34 frame39 StopSounds
        assert_eq!(game.fadeout_tick,Some(39));
        assert_eq!(game.take_audio_events(),vec![AudioEvent::MusicStop]);
        game.tick(&mut input,&ground); // sprite34 frame40 still draws
        assert_eq!(game.fadeout_tick,Some(40));
        assert!(!game.level1a_complete);
        assert_eq!(game.pending_stage_entry(),None);
        assert!(game.take_audio_events().is_empty());
        game.tick(&mut input,&ground); // dispatcher latches next scene
        assert!(game.level1a_complete);
        assert_eq!(game.screen,AppScreen::Playing);
        assert_eq!(game.active_stage(),StageId::Level1A);
        assert_eq!(game.pending_stage_entry(),Some(PendingStageEntry {
            from:StageId::Level1A,
            to:StageId::Level1B,
            progress:RootProgress{score:1234,lives:3,batarangs:17},
            outgoing_root_sound_id:751,
            incoming_root_sound_id:1076,
        }));
        assert!(game.take_audio_events().is_empty()); // no premature 1B start
        for _ in 0..5 {game.tick(&mut input,&ground);}
        assert_eq!(game.active_stage(),StageId::Level1A);
        assert_eq!(game.fadeout_tick,Some(40));
        assert_eq!(game.health,42);
        assert!(game.take_audio_events().is_empty());
    }

    #[test]
    fn death_restart_does_not_schedule_a_new_stage_or_stop_1a_music() {
        let ground=CollisionMask::level1a();
        let mut game=Game::new();
        let mut input=InputState::default();
        game.screen=AppScreen::Playing;
        game.lives=2;
        game.score=77;
        game.batarangs=9;
        game.fadeout_tick=Some(38);
        game.respawn_after_fade=true;
        game.tick(&mut input,&ground);
        assert_eq!(game.fadeout_tick,Some(39));
        assert!(game.take_audio_events().is_empty());
        game.tick(&mut input,&ground);
        game.tick(&mut input,&ground);
        assert_eq!(game.fadeout_tick,None);
        assert_eq!(game.active_stage(),StageId::Level1A);
        assert_eq!(game.pending_stage_entry(),None);
        assert_eq!((game.score,game.lives,game.batarangs),(77,2,9));
        assert!(game.take_audio_events().is_empty());
    }
}

#[cfg(test)]
mod qa_navigation_tests {
    use super::{AppScreen,Game,InputState,LEVEL_X,LEVEL_Y,LOGICAL_H,VK_ENTER};
    use crate::collision::CollisionMask;

    #[test]
    fn qa_f7_matches_normal_level_entry_after_neutral_gameplay() {
        let ground=CollisionMask::level1a();
        let mut natural=Game::new();
        let mut input=InputState::default();

        // Follow the public title-to-intro and source-derived 41-frame
        // SKIP path rather than forcing the normal game's screen state.
        input.set(VK_ENTER,true);
        natural.tick(&mut input,&ground);
        input.set(VK_ENTER,false);
        assert_eq!(natural.screen,AppScreen::Intro);
        assert!(natural.skip_intro());
        for _ in 0..41 { natural.tick(&mut input,&ground); }
        assert_eq!(natural.screen,AppScreen::Playing);
        assert_eq!(natural.ticks,0);
        for _ in 0..150 { natural.tick(&mut input,&ground); }

        let mut qa=Game::new();
        qa.prepare_level1a_for_qa(&ground);
        assert_eq!(qa.screen,AppScreen::Playing);
        assert_eq!(qa.ticks,150);
        assert_eq!(qa.ticks,natural.ticks);
        assert_eq!(qa.player.state,natural.player.state);
        assert_eq!(qa.player.jumping,natural.player.jumping);
        assert_eq!(qa.player.on_ground,natural.player.on_ground);
        assert_eq!(qa.overlay_frame,natural.overlay_frame);
        assert!((qa.player.x-natural.player.x).abs()<0.001);
        assert!((qa.player.y-natural.player.y).abs()<0.001);
        assert!((qa.camera_x-natural.camera_x).abs()<0.001);
        assert!((qa.camera_y-natural.camera_y).abs()<0.001);

        // These are the same logical stage coordinates used by the legacy
        // and HQ Batman renderers, not guessed screen-space spawn offsets.
        let stage_y=qa.player.y+LEVEL_Y+qa.camera_y;
        let stage_x=qa.player.x+LEVEL_X+qa.camera_x;
        assert!(stage_y>=0.0 && stage_y<LOGICAL_H as f32,
                "QA actor origin offscreen: stage_y={stage_y}");
        assert!(stage_x>=0.0 && stage_x<super::LOGICAL_W as f32,
                "QA actor origin offscreen: stage_x={stage_x}");
    }
}

#[cfg(test)]
mod b13_level1b_stage_activation_tests {
    use super::{
        AppScreen, AudioEvent, Game, InputState, LEVEL1B_PICKUPS,
        OVERLAY_FIGHT_BTNS, OVERLAY_GO_LEFT, OVERLAY_GO_RIGHT,
    };
    use crate::collision::CollisionMask;
    use crate::stage::{PendingStageEntry, RootProgress, StageId};

    #[test]
    fn level1b_start_stage_initializes_state_and_enemies() {
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1B);

        assert_eq!(game.active_stage(), StageId::Level1B);
        assert_eq!(game.pending_stage_entry(), None);
        assert_eq!(game.player.dir, -1);
        assert!((game.player.x - (-506.5)).abs() < 0.001);
        assert!((game.player.y - (-258.0)).abs() < 0.001);
        assert_eq!(game.health, 100);
        assert!(game.bank_goons.is_some());
        assert!(game.bank_biker.is_some());
        assert!(!game.level1b_exit_reached);
        assert!(!game.level1b_complete);
    }

    #[test]
    fn level1b_camera_bounds_and_parameters() {
        let mut game = Game::new();
        game.start_stage(StageId::Level1B);
        let cam = game.stage.descriptor().camera;
        assert_eq!(cam.min_x, -6220.0);
        assert_eq!(cam.max_x, 600.0);
        assert_eq!(cam.height, 125.0);
        assert_eq!(cam.jump, 3.0);
    }

    #[test]
    fn level1b_collects_pickups() {
        let ground = CollisionMask::level1b();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1B);
        let mut input = InputState::default();

        // Position player at pickup 0: (-2068.1, 138.5) in game space
        let origin_x = game.level_origin_x();
        let origin_y = game.level_origin_y();
        game.player.x = LEVEL1B_PICKUPS[0].0 - origin_x;
        game.player.y = LEVEL1B_PICKUPS[0].1 - origin_y;

        let prev_batarangs = game.batarangs;
        let prev_score = game.score;
        game.tick(&mut input, &ground);

        assert!(game.pickups[0]);
        assert_eq!(game.batarangs, prev_batarangs + 5);
        assert_eq!(game.score, prev_score + 5);
        assert!(game.take_audio_events().contains(&AudioEvent::Item));
    }

    #[test]
    fn level1b_checkpoints_trigger_overlays() {
        let ground = CollisionMask::level1b();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1B);
        let mut input = InputState::default();
        let origin_x = game.level_origin_x();
        let origin_y = game.level_origin_y();

        // Checkpoint 01: min_x:98.152, min_y:102.475, max_x:364.689, max_y:398.098 -> OVERLAY_FIGHT_BTNS (75)
        game.player.x = 200.0 - origin_x;
        game.player.y = 200.0 - origin_y;
        game.tick(&mut input, &ground);
        assert_eq!(game.overlay_frame, OVERLAY_FIGHT_BTNS);

        // Checkpoint 02: min_x:364.689, min_y:110.725, max_x:969.362, max_y:406.348 -> OVERLAY_GO_LEFT (10)
        game.player.x = 500.0 - origin_x;
        game.player.y = 200.0 - origin_y;
        game.tick(&mut input, &ground);
        assert_eq!(game.overlay_frame, OVERLAY_GO_LEFT);

        // Checkpoint 03: min_x:-8243.564, min_y:124.725, max_x:-6683.608, max_y:420.348 -> OVERLAY_GO_RIGHT (1)
        game.player.x = -7000.0 - origin_x;
        game.player.y = 200.0 - origin_y;
        game.tick(&mut input, &ground);
        assert_eq!(game.overlay_frame, OVERLAY_GO_RIGHT);
    }

    #[test]
    fn level1b_pit_fall_triggers_fade_and_respawns_at_1b_start() {
        let ground = CollisionMask::level1b();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1B);
        let mut input = InputState::default();

        // Void fall: player y in game space > 400.0
        let origin_y = game.level_origin_y();
        game.player.y = 450.0 - origin_y;
        game.lives = 3;

        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, Some(0));
        assert_eq!(game.lives, 2);
        assert!(game.respawn_after_fade);

        // Advance through 40-frame fadeout
        for _ in 0..40 {
            game.tick(&mut input, &ground);
        }
        assert_eq!(game.fadeout_tick, Some(40));

        // Tick 41: advance_level_exit resets game to start of Level 1B
        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, None);
        assert_eq!(game.active_stage(), StageId::Level1B);
        assert_eq!(game.lives, 2);
        assert!((game.player.x - (-506.5)).abs() < 0.001);
        assert!((game.player.y - (-258.0)).abs() < 0.001);
        assert_eq!(game.health, 100);
    }

    #[test]
    fn level1b_stage_completion_transitions_to_level1c() {
        let ground = CollisionMask::level1b();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1B);
        let mut input = InputState::default();
        game.score = 5000;
        game.lives = 4;
        game.batarangs = 25;

        // Simulate squad completing wave 2
        if let Some(squad) = &mut game.bank_goons {
            squad.respawned = 1;
            squad.e_kills = 8;
        }

        // Next tick routes StageComplete from goon squad
        game.tick(&mut input, &ground);
        assert!(game.level1b_exit_reached);
        assert_eq!(game.fadeout_tick, Some(0));

        // Advance through fadeout to tick 39
        for i in 1..=38 {
            game.tick(&mut input, &ground);
            assert_eq!(game.fadeout_tick, Some(i));
            assert_eq!(game.pending_stage_entry(), None);
        }

        // Tick 39: MusicStop emitted
        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, Some(39));
        assert_eq!(game.take_audio_events(), vec![AudioEvent::MusicStop]);

        // Tick 40: terminal frame
        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, Some(40));

        // Tick 41: pending_stage_entry latched targeting Level 1C
        game.tick(&mut input, &ground);
        assert!(game.level1b_complete);
        assert_eq!(game.active_stage(), StageId::Level1B);
        assert_eq!(game.pending_stage_entry(), Some(PendingStageEntry {
            from: StageId::Level1B,
            to: StageId::Level1C,
            progress: RootProgress { score: 5000, lives: 4, batarangs: 25 },
            outgoing_root_sound_id: 1076,
            incoming_root_sound_id: 1252,
        }));
    }
}

#[cfg(test)]
mod b18_penguin_escape_card_tests {
    use super::{AppScreen, AudioEvent, Game, InputState, PlayerState, Shot, VK_SPACE};
    use crate::collision::CollisionMask;
    use crate::penguin::{
        PenguinBoss, PenguinState, LEVEL1C_PICKUP_POINTS, PENGUIN_KILL_SCORE,
    };
    use crate::stage::{PendingStageEntry, RootProgress, StageId};

    #[test]
    fn level1c_boss_defeat_enters_escape_card_with_preserved_progress_and_audio() {
        let ground = CollisionMask::level1c();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1C);
        game.score = 1230;
        game.lives = 4;
        game.batarangs = 12;

        // Collect one Level 1C pickup (+5 batarangs, +5 score, AudioEvent::Item).
        let (px, py) = LEVEL1C_PICKUP_POINTS[0];
        game.player.x = px - game.level_origin_x();
        game.player.y = py - game.level_origin_y();
        let mut input = InputState::default();
        game.tick(&mut input, &ground);
        assert!(game.pickups[0]);
        assert_eq!(game.batarangs, 17);
        assert_eq!(game.score, 1235);
        assert!(game.take_audio_events().contains(&AudioEvent::Item));

        // Put Penguin at 10 HP right in front of Batman and land a punch for +500 & defeat.
        let origin_x = game.level_origin_x();
        let origin_y = game.level_origin_y();
        let (boss_x, boss_y) = {
            let boss = game.penguin_boss.as_mut().expect("boss initialized in Level1C");
            boss.health = 10;
            boss.set_state(PenguinState::Stand);
            boss.dir = -1;
            (boss.x, boss.y)
        };
        game.player.jumping = false;
        game.player.on_ground = true;
        game.player.dx = 0.0;
        game.player.dy = 0.0;
        game.player.x = boss_x - 30.0 - origin_x;
        game.player.y = boss_y - origin_y;
        game.player.dir = 1;
        game.player.state = PlayerState::Punch;
        game.player.state_tick = 1;
        game.player_attacking = true;

        game.tick(&mut input, &ground);
        {
            let boss = game.penguin_boss.as_ref().unwrap();
            assert_eq!(boss.state, PenguinState::Die);
        }
        assert!(game.score >= 1235 + PENGUIN_KILL_SCORE);
        // On the subsequent frame, penguinAI's `health <= 0` branch sets
        // `gotoNext = "card:escape"` and plays `#34` fadeout (`DefeatedTrigger`).
        game.tick(&mut input, &ground);
        assert!(game.level1c_exit_reached);
        assert_eq!(game.fadeout_tick, Some(0));

        // 41-frame fadeout runs before entering card:escape; MusicStop fires on frame 39.
        for i in 1..=38 {
            game.tick(&mut input, &ground);
            assert_eq!(game.fadeout_tick, Some(i));
            assert_eq!(game.screen, AppScreen::Playing);
        }
        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, Some(39));
        assert_eq!(game.take_audio_events(), vec![AudioEvent::MusicStop]);

        game.tick(&mut input, &ground);
        assert_eq!(game.fadeout_tick, Some(40));
        assert_eq!(game.screen, AppScreen::Playing);

        let expected_score = game.score;
        game.tick(&mut input, &ground);
        assert!(game.level1c_complete);
        assert_eq!(game.screen, AppScreen::EscapeCard);
        assert_eq!(game.escape_card_tick, 0);
        assert_eq!(game.escape_card_fadeout_tick, None);
        assert_eq!(game.score, expected_score);
        assert_eq!(game.lives, 4);
        assert_eq!(game.batarangs, 17);
        assert_eq!(game.take_audio_events(), vec![AudioEvent::IntroStart]);
    }

    #[test]
    fn escape_card_plays_225_frame_timeline_never_loops_and_ends_at_level2a() {
        let ground = CollisionMask::level1c();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1C);
        game.score = 4321;
        game.lives = 3;
        game.batarangs = 9;
        game.level1c_exit_reached = true;
        game.fadeout_tick = Some(40);
        let mut input = InputState::default();
        game.tick(&mut input, &ground);
        assert_eq!(game.screen, AppScreen::EscapeCard);
        let _ = game.take_audio_events();

        // Advance #1267 from frame 0 through frame 224 (225 frames total).
        for tick in 1..224 {
            game.tick(&mut input, &ground);
            assert_eq!(game.escape_card_tick, tick);
            assert_eq!(game.escape_card_fadeout_tick, None);
            assert!(!game.escape_card_complete);
        }

        // Frame 224 triggers `#1267` DoAction: `stop(); _root.gotoNext = "level2a"; _root.fadeout.play();`.
        game.tick(&mut input, &ground);
        assert_eq!(game.escape_card_tick, 224);
        assert_eq!(game.escape_card_fadeout_tick, Some(0));

        // During the 41-frame fadeout, #1267 remains at frame 224 (never loops to 0).
        for f in 1..=38 {
            game.tick(&mut input, &ground);
            assert_eq!(game.escape_card_tick, 224);
            assert_eq!(game.escape_card_fadeout_tick, Some(f));
        }
        game.tick(&mut input, &ground);
        assert_eq!(game.escape_card_fadeout_tick, Some(39));
        assert_eq!(game.take_audio_events(), vec![AudioEvent::IntroStop]);

        game.tick(&mut input, &ground);
        assert_eq!(game.escape_card_fadeout_tick, Some(40));

        // Final fade tick latches end-to-2A with all progress intact and holds.
        game.tick(&mut input, &ground);
        assert!(game.escape_card_complete);
        assert_eq!(game.escape_card_tick, 224);
        assert_eq!(
            game.pending_stage_entry(),
            Some(PendingStageEntry {
                from: StageId::Level1C,
                to: StageId::Level2A,
                progress: RootProgress {
                    score: 4321,
                    lives: 3,
                    batarangs: 9,
                },
                outgoing_root_sound_id: 130,
                incoming_root_sound_id: 1409,
            })
        );

        // Extra ticks must never loop back to frame 0 or re-emit IntroStart.
        for _ in 0..10 {
            game.tick(&mut input, &ground);
            assert_eq!(game.screen, AppScreen::EscapeCard);
            assert_eq!(game.escape_card_tick, 224);
            assert!(game.take_audio_events().is_empty());
        }
    }

    #[test]
    fn escape_card_skip_requires_41_frame_fade_and_routes_laser_electro_plus_batarang_hide() {
        let ground = CollisionMask::level1c();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1C);
        let mut input = InputState::default();

        // Laser hitBox active local frame 9 -> Batman enters PlayerState::Electro (`damage = 5`).
        let boss = game.penguin_boss.as_mut().unwrap();
        boss.set_state(PenguinState::Attack2);
        boss.child_frame = 9;
        boss.dir = -1;
        let hb = boss.laser_game_aabb();
        game.player.x = (hb.0 + hb.2) * 0.5 - game.level_origin_x();
        game.player.y = (hb.1 + hb.3) * 0.5 - game.level_origin_y();
        game.tick(&mut input, &ground);
        assert_eq!(game.player.state, PlayerState::Electro);
        assert_eq!(game.health, 95);

        // Deflected batarang in Attack range removes shot from active projectiles.
        let mut boss2 = PenguinBoss::new();
        boss2.health = 200;
        game.penguin_boss = Some(boss2);
        let bx = game.penguin_boss.as_ref().unwrap().x;
        let by = game.penguin_boss.as_ref().unwrap().y;
        game.shots.push(Shot {
            x: bx + 5.0,
            y: by - 35.0,
            dir: -1,
        });
        // Seed until EvaluateBatarang returns Blocked (deterministic RNG trial).
        game.penguin_rng = super::PenguinRng::new(7);
        game.tick(&mut input, &ground);
        assert!(
            game.shots.is_empty() || game.penguin_boss.as_ref().unwrap().health < 200,
            "batarang either blocked & removed or damaged boss"
        );

        // Test EscapeCard skip (#1268 / Space): never bypasses the 41-frame fadeout.
        game.score = 777;
        game.lives = 2;
        game.batarangs = 3;
        game.level1c_exit_reached = true;
        game.fadeout_tick = Some(40);
        game.tick(&mut input, &ground);
        assert_eq!(game.screen, AppScreen::EscapeCard);
        let _ = game.take_audio_events();

        input.set(VK_SPACE, true);
        game.tick(&mut input, &ground);
        input.set(VK_SPACE, false);
        assert_eq!(game.escape_card_fadeout_tick, Some(0));
        assert!(!game.escape_card_complete);

        for _ in 0..41 {
            game.tick(&mut input, &ground);
        }
        assert!(game.escape_card_complete);
        assert_eq!(
            game.pending_stage_entry().map(|p| (p.to, p.progress.score)),
            Some((StageId::Level2A, 777))
        );
    }
}

#[cfg(test)]
mod b19_bank_milestone_tests {
    use super::{AppScreen, AudioEvent, Game, InputState, LOGICAL_H, LOGICAL_W, PICKUPS, VK_A, VK_ENTER, LEVEL_X, LEVEL_Y};
    use crate::assets::Assets;
    use crate::collision::CollisionMask;
    use crate::render::{self, HQ_H, HQ_W};
    use crate::stage::StageId;
    use crate::surface::RenderSurface;

    #[test]
    fn integrated_bank_chapter_1a_to_1b_to_1c_to_escape_card_and_level2a_handoff() {
        let mut ground = CollisionMask::for_stage(StageId::Level1A);
        let mut game = Game::new();
        let mut input = InputState::default();

        // Title -> Intro -> SKIP fadeout -> Level 1A
        input.set(VK_ENTER, true);
        game.tick(&mut input, &ground);
        input.set(VK_ENTER, false);
        assert_eq!(game.screen, AppScreen::Intro);
        assert!(game.skip_intro());
        for _ in 0..41 {
            game.tick(&mut input, &ground);
        }
        assert_eq!(game.screen, AppScreen::Playing);
        assert_eq!(game.active_stage(), StageId::Level1A);
        let _ = game.take_audio_events();

        // Collect one batarang pickup and trigger Level 1A exit
        game.player.x = PICKUPS[0].0 - LEVEL_X;
        game.player.y = PICKUPS[0].1 - LEVEL_Y;
        game.tick(&mut input, &ground);
        assert!(game.score >= 5);
        assert!(game.batarangs >= 5);
        game.lives = 4;

        game.level1a_exit_reached = true;
        game.fadeout_tick = Some(0);
        for _ in 0..41 {
            game.tick(&mut input, &ground);
        }
        let next = game.consume_pending_stage_entry();
        assert_eq!(next, Some(StageId::Level1B));
        ground = CollisionMask::for_stage(StageId::Level1B);
        assert_eq!(game.active_stage(), StageId::Level1B);
        assert!(game.bank_goons.is_some());
        assert!(game.bank_biker.is_some());
        assert_eq!((game.lives, game.batarangs), (4, 5));
        assert!(game.score >= 5);
        assert!(game.take_audio_events().contains(&AudioEvent::MusicStart));

        // Defeat Wave 1 + Wave 2 goons in Level 1B to trigger Level 1C handoff
        if let Some(squad) = &mut game.bank_goons {
            squad.e_kills = 10;
        }
        game.tick(&mut input, &ground);
        {
            let squad = game.bank_goons.as_ref().expect("1B squad active");
            assert_eq!(squad.respawned, 1);
            assert!(squad.active_count() >= 8);
        }
        if let Some(squad) = &mut game.bank_goons {
            squad.e_kills = 8;
        }
        game.tick(&mut input, &ground);
        assert!(game.level1b_exit_reached);
        for _ in 0..41 {
            game.tick(&mut input, &ground);
        }
        let next_1c = game.consume_pending_stage_entry();
        assert_eq!(next_1c, Some(StageId::Level1C));
        ground = CollisionMask::for_stage(StageId::Level1C);
        assert_eq!(game.active_stage(), StageId::Level1C);
        assert!(game.penguin_boss.is_some());
        assert!(game.bank_goons.is_none());
        assert!(game.bank_biker.is_none());
        assert_eq!((game.lives, game.batarangs), (4, 5));
        assert!(game.take_audio_events().contains(&AudioEvent::MusicStart));

        // Defeat Penguin boss -> EscapeCard -> Level2A pending handoff
        game.player.jumping = false;
        game.player.on_ground = true;
        game.player.dy = 0.0;
        let (bx, by) = {
            let boss = game.penguin_boss.as_mut().expect("1C boss");
            boss.health = 1;
            boss.state = crate::penguin::PenguinState::Stand;
            (boss.x, boss.y)
        };
        let d1c = StageId::Level1C.descriptor();
        game.player.x = bx - d1c.level_origin_game_local.x - 20.0;
        game.player.y = by - d1c.level_origin_game_local.y;
        game.player.dir = 1;
        game.player.state = crate::game::PlayerState::Punch;
        game.player.state_tick = 1;
        game.player_attacking = true;
        let _ = VK_A;
        for _ in 0..45 {
            game.tick(&mut input, &ground);
            if game.fadeout_tick.is_some() {
                break;
            }
        }
        assert!(game.level1c_exit_reached, "Penguin defeat should trigger exit fade");
        for _ in 0..42 {
            game.tick(&mut input, &ground);
            if game.screen == AppScreen::EscapeCard {
                break;
            }
        }
        assert_eq!(game.screen, AppScreen::EscapeCard);
        let score_at_escape = game.score;
        assert!(score_at_escape >= 505);

        assert!(game.skip_escape_card());
        for _ in 0..41 {
            game.tick(&mut input, &ground);
        }
        assert!(game.escape_card_complete);
        let handoff = game.pending_stage_entry().expect("Level2A handoff latched");
        assert_eq!(handoff.from, StageId::Level1C);
        assert_eq!(handoff.to, StageId::Level2A);
        assert_eq!(handoff.progress.score, score_at_escape);
        assert_eq!(handoff.progress.lives, 4);
        // Level 2A stays latched until B20+ stage assets/masks are enabled
        assert_eq!(game.consume_pending_stage_entry(), None);
    }

    #[test]
    fn integrated_bank_chapter_renders_legacy_and_hq_across_1b_1c_and_escape_card() {
        let mut assets = Assets::load();
        let mut legacy = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut base = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq = RenderSurface::new(HQ_W, HQ_H);

        let mut game_1a = Game::new();
        game_1a.screen = AppScreen::Playing;
        game_1a.ticks = 10;
        render::render(&mut legacy, &game_1a, &assets);
        let pixels_1a = legacy.pixels.clone();

        // Stage 1B render check (Legacy + HQ)
        let mut game_1b = Game::new();
        game_1b.start_stage(StageId::Level1B);
        game_1b.screen = AppScreen::Playing;
        game_1b.ticks = 10;
        assets.load_stage(StageId::Level1B);
        render::render(&mut legacy, &game_1b, &assets);
        assert_ne!(legacy.pixels, pixels_1a, "Level 1B must differ visually from Level 1A");
        assert!(legacy.pixels.iter().any(|&p| (p & 0x00ff_ffff) != 0));
        render::render_high_quality(&mut base, &mut hq, &game_1b, &assets);
        assert!(hq.pixels.iter().any(|&p| (p & 0x00ff_ffff) != 0));

        // Stage 1C render check (Legacy + HQ)
        let mut game_1c = Game::new();
        game_1c.start_stage(StageId::Level1C);
        game_1c.screen = AppScreen::Playing;
        game_1c.ticks = 10;
        assets.load_stage(StageId::Level1C);
        render::render(&mut legacy, &game_1c, &assets);
        assert_ne!(legacy.pixels, pixels_1a);
        assert!(legacy.pixels.iter().any(|&p| (p & 0x00ff_ffff) != 0));
        render::render_high_quality(&mut base, &mut hq, &game_1c, &assets);
        assert!(hq.pixels.iter().any(|&p| (p & 0x00ff_ffff) != 0));

        // Escape card render check across frames 0 and 112 (HQ + Legacy)
        let mut game_esc = Game::new();
        game_esc.screen = AppScreen::EscapeCard;
        game_esc.escape_card_tick = 0;
        render::render(&mut legacy, &game_esc, &assets);
        let esc0 = legacy.pixels.clone();
        game_esc.escape_card_tick = 112;
        render::render(&mut legacy, &game_esc, &assets);
        assert_ne!(legacy.pixels, esc0, "Escape card must animate between frame 0 and 112");
        render::render_high_quality(&mut base, &mut hq, &game_esc, &assets);
        assert!(hq.pixels.iter().any(|&p| (p & 0x00ff_ffff) != 0));
    }
}

#[cfg(test)]
mod b21_labs2a_camera_ground_tests {
    use super::{AppScreen, Game, InputState, VK_LEFT, VK_RIGHT, VK_SPACE};
    use crate::collision::CollisionMask;
    use crate::stage::{labs2a, LabsCameraMode, StageId};

    #[test]
    fn labs2a_spawn_camera_logic2_tracks_shaft_descent_and_clamps_left_wall() {
        let ground = CollisionMask::for_stage(StageId::Level2A);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level2A);

        assert_eq!(game.labs_camera_mode(), LabsCameraMode::CameraLogic2);
        assert_eq!(game.cam_height(), 275.0);
        assert_eq!(game.player.dir, -1);
        assert_eq!(game.player.dy, 3.0);

        let desc = StageId::Level2A.descriptor();
        let gx0 = game.player.x + desc.level_origin_game_local.x;
        let gy0 = game.player.y + desc.level_origin_game_local.y;
        assert!((gx0 - 227.2).abs() < 0.01);
        assert!((gy0 - 115.4).abs() < 0.01);
        assert!((game.camera_y - (275.0 - gy0)).abs() < 0.01);

        // Glide straight down inside the entry elevator shaft while steering left against wall/camMin=200.
        let mut input = InputState::default();
        for _ in 0..800 {
            input.held[VK_SPACE] = true;
            input.held[VK_LEFT] = true;
            game.tick(&mut input, &ground);
            if !game.on_platform && game.player.on_ground && !game.player.jumping {
                break;
            }
        }
        assert!(game.player.on_ground && !game.player.jumping, "Batman should land cleanly at bottom of entry shaft");

        let landed_gx = game.player.x + desc.level_origin_game_local.x;
        let landed_gy = game.player.y + desc.level_origin_game_local.y;
        assert!(landed_gx >= 162.0 && landed_gx <= 240.0, "left shaft wall should hold Batman around 162..240, got {landed_gx}");
        assert!((landed_gy - (2648.2 - 75.0)).abs() <= 5.0, "expected landing atop 2648.2 shaft floor, got {landed_gy}");
        assert!((game.camera_y - (275.0 - landed_gy)).abs() < 0.01, "cameraLogic2 must follow vertical floor depth across 2500px shaft");

        // At landed_gx <= cam_min (200), further horizontal camera ease must freeze.
        let cam_before = game.camera_x;
        input.held[VK_SPACE] = false;
        for _ in 0..5 {
            input.held[VK_LEFT] = true;
            game.tick(&mut input, &ground);
        }
        assert_eq!(game.camera_x, cam_before, "camera_x must clamp when player_x <= camMin(200)");
    }

    #[test]
    fn labs2a_camera_triggers_switch_camera_logic2_and_3_and_respect_airborne_gate() {
        let ground = CollisionMask::for_stage(StageId::Level2A);
        let desc = StageId::Level2A.descriptor();
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level2A);

        // Place Batman grounded inside camera_01 (gx=400, gy=2573.2).
        game.player.x = 400.0 - desc.level_origin_game_local.x;
        game.player.y = 2573.2 - desc.level_origin_game_local.y;
        game.player.dy = 0.0;
        game.player.jumping = false;
        game.player.on_ground = true;

        let mut input = InputState::default();
        game.tick(&mut input, &ground);
        assert_eq!(game.labs_camera_mode(), LabsCameraMode::CameraLogic2);
        assert_eq!(game.cam_height(), 275.0);

        // Jump right across camera_02 boundary while truly airborne inside its Y box -> trigger must NOT fire while jumping!
        game.player.x = 620.0 - desc.level_origin_game_local.x;
        game.player.y = 2450.0 - desc.level_origin_game_local.y;
        game.player.dy = -10.0;
        game.player.jumping = true;
        game.player.on_ground = false;
        game.tick(&mut input, &ground);
        assert_eq!(game.labs_camera_mode(), LabsCameraMode::CameraLogic2, "camera_02 must ignore overlapping airborne player");

        // Land and walk right grounded inside camera_02 -> switches to cameraLogic3 with targetX=150 and converges camera_y+player_y -> 250.
        game.player.y = 2573.2 - desc.level_origin_game_local.y;
        game.player.dy = 0.0;
        game.player.jumping = false;
        game.player.on_ground = true;
        for _ in 0..10 {
            input.held[VK_RIGHT] = true;
            game.tick(&mut input, &ground);
        }
        assert_eq!(game.labs_camera_mode(), LabsCameraMode::CameraLogic3);
        let gy = game.player.y + desc.level_origin_game_local.y;
        assert!((game.camera_y + gy - 250.0).abs() < 0.25, "cameraLogic3 settled screen Y should converge to ~250, got {}", game.camera_y + gy);

        // Airborne inside cameraLogic3 eases camera_y by 1/4 only when |(camera_y + player_y) - 150| > 1.
        let cam_y_before = game.camera_y;
        game.player.y -= 120.0;
        game.player.jumping = true;
        input.held[VK_RIGHT] = false;
        game.tick(&mut input, &ground);
        assert_ne!(game.camera_y, cam_y_before);

        // Walking grounded into camera_04 (gx ~ 2650) switches back to cameraLogic2 (camHeight = 275).
        game.player.x = 2650.0 - desc.level_origin_game_local.x;
        game.player.y = 2573.2 - desc.level_origin_game_local.y;
        game.player.dy = 0.0;
        game.player.jumping = false;
        game.tick(&mut input, &ground);
        assert_eq!(game.labs_camera_mode(), LabsCameraMode::CameraLogic2);
        assert_eq!(game.cam_height(), 275.0);

        // Platform mode in cameraLogic2 centers on 300 + (player_x - currentPlatform_x).
        game.set_platform_state(true, 2600.0);
        game.camera_x = -2300.0;
        game.tick(&mut input, &ground);
        // With player_x=2650 and platform_x=2600, targetX=350; screen x = 350 -> delta=0.
        assert!((game.camera_x - (-2300.0)).abs() < 1.0);

        // Verify all 10 camera zones match alternating source assignment.
        let mut l2 = 0usize;
        let mut l3 = 0usize;
        for c in labs2a::CAMERAS {
            match c.camera_mode() {
                LabsCameraMode::CameraLogic2 => l2 += 1,
                LabsCameraMode::CameraLogic3 => l3 += 1,
            }
        }
        assert_eq!((l2, l3), (5, 5));
    }
}

#[cfg(test)]
mod b22_labs2a_door_tests {
    use super::{AppScreen, AudioEvent, Game, InputState};
    use crate::collision::CollisionMask;
    use crate::stage::{labs2a, StageId};

    #[test]
    fn labs2a_initializes_18_closed_doors_and_leaves_bank_stages_empty() {
        let mut game = Game::new();
        assert!(game.labs_doors.is_empty());

        game.start_stage(StageId::Level2A);
        assert_eq!(game.labs_doors.len(), 18);
        assert!(game.labs_doors.iter().all(|d| d.frame == 0 && !d.playing));

        game.start_stage(StageId::Level1A);
        assert!(game.labs_doors.is_empty());
    }

    #[test]
    fn labs2a_door_opens_emits_sound_holds_open_and_closes_on_exit() {
        let ground = CollisionMask::for_stage(StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(StageId::Level2A);
        game.screen = AppScreen::Playing;

        // Place Batman directly centered on doorway gx=614, gy=2573 where BOTH
        // door_01 (gx<=725.41) and door_02 (gx>=503.34) sensors overlap Batman simultaneously!
        game.player.x = 614.0 - game.level_origin_x();
        game.player.y = 2573.0 - game.level_origin_y();
        game.player.jumping = false;
        game.player.on_ground = true;

        let mut input = InputState::default();
        game.tick(&mut input, &ground);
        assert_eq!(game.labs_doors[0].frame, 1);
        assert_eq!(game.labs_doors[1].frame, 1);
        assert!(game.labs_doors[0].playing);
        let door_sounds = game
            .take_audio_events()
            .into_iter()
            .filter(|e| *e == AudioEvent::DoorOpen)
            .count();
        assert_eq!(door_sounds, 2, "paired doors 0 & 1 must both emit DoorOpen sound #1323 on frame 1");

        // Advance 8 ticks -> frame reaches 9 (open) and Stop() halts playback.
        for _ in 0..8 {
            game.tick(&mut input, &ground);
        }
        assert_eq!((game.labs_doors[0].frame, game.labs_doors[0].playing), (9, false));
        assert_eq!((game.labs_doors[1].frame, game.labs_doors[1].playing), (9, false));

        // While still overlapping the doorway sensor, doors must stay open at frame 9.
        for _ in 0..4 {
            game.tick(&mut input, &ground);
        }
        assert_eq!((game.labs_doors[0].frame, game.labs_doors[0].playing), (9, false));

        // Move Batman away outside all door sensors; doors advance 10..19 -> 0 (closed).
        game.player.x = 200.0 - game.level_origin_x();
        game.player.y = 2573.0 - game.level_origin_y();
        game.tick(&mut input, &ground);
        assert_eq!((game.labs_doors[0].frame, game.labs_doors[0].playing), (10, true));

        for _ in 0..10 {
            game.tick(&mut input, &ground);
        }
        assert_eq!((game.labs_doors[0].frame, game.labs_doors[0].playing), (0, false));
        assert_eq!((game.labs_doors[1].frame, game.labs_doors[1].playing), (0, false));
        assert_eq!(labs2a::DOOR_JAMBS.len(), 18);
    }
}

#[cfg(test)]
mod b23_labs2a_elevator_tests {
    use super::{AppScreen, Game, GLOBAL_GRAVITY, InputState, VK_SPACE};
    use crate::collision::CollisionMask;
    use crate::stage::{labs2a, StageId};

    #[test]
    fn labs2a_initializes_3_elevators_and_drives_downward_ride_to_limit() {
        let ground = CollisionMask::for_stage(StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(StageId::Level2A);
        game.screen = AppScreen::Playing;

        assert_eq!(game.labs_elevators.len(), 3);
        assert_eq!(labs2a::ELEVATORS[0].limit_y, 2685);
        assert!(game.labs_elevators.iter().all(|e| e.active && !e.moving && e.dy == 0.0));

        // Let Batman drop onto elevator_01_2685 (y=429.90, x=308.95) and verify lockstep ride.
        let mut input = InputState::default();
        let mut rode_down = false;
        for _ in 0..25 {
            game.tick(&mut input, &ground);
            if game.on_platform && !game.player.jumping && (game.labs_elevators[0].dy - 12.0).abs() < 0.01 {
                rode_down = true;
                assert_eq!(game.gravity, 0.0, "gravity must zero out while lockstep riding elevator");
                assert!((game.current_platform_x - 308.95).abs() < 0.01);
                break;
            }
        }
        assert!(rode_down, "Batman must board elevator_01_2685 at spawn shaft and descend at dy=12");

        // Ride until limit=2685 stops the elevator (`delete onEnterFrame` -> active=false).
        for _ in 0..220 {
            game.tick(&mut input, &ground);
            if !game.labs_elevators[0].active {
                break;
            }
        }
        let e0 = game.labs_elevators[0];
        assert!(!e0.active && !e0.moving && e0.dy == 0.0);
        assert!((e0.y - 2685.0).abs() < 0.01);
        assert!(!game.on_platform);
        assert_eq!(game.gravity, GLOBAL_GRAVITY);
    }

    #[test]
    fn labs2a_elevator_jump_off_and_upward_branch_match_avm1_formulas() {
        let ground = CollisionMask::for_stage(StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(StageId::Level2A);
        game.screen = AppScreen::Playing;

        // Place Batman directly atop elevator_02_3480 (x=2863.1, y=2706.25, limit=3480).
        let e1 = labs2a::ELEVATORS[1];
        game.player.x = e1.x - game.level_origin_x();
        game.player.y = (e1.y - 95.0) - game.level_origin_y();
        game.player.jumping = false;
        game.player.on_ground = true;

        let mut input = InputState::default();
        game.tick(&mut input, &ground);
        assert!(game.on_platform && (game.labs_elevators[1].dy - 12.0).abs() < 0.01);

        // Press Jump -> triggers `-36 + dy/3 == -32` launch velocity and restores globalGravity.
        input.pressed[VK_SPACE] = true;
        input.held[VK_SPACE] = true;
        game.tick(&mut input, &ground);
        assert!(!game.on_platform);
        assert_eq!(game.gravity, GLOBAL_GRAVITY);
        assert!((game.player.dy - (-36.0 + 12.0 / 3.0)).abs() < 0.01);

        // Verify upward branch (`y >= limit` -> player._y -= 32, dy = -12, jump-off = -40):
        // Simulate starting elevator_01 below limit=2685 in unobstructed air (Level 2A ground is
        // solid beneath shaft 1's y=2685 floor, whereas Level 2B has upward shafts in open air).
        let open_air = CollisionMask::level1b(); // max y = 527, so y~2700 is empty air
        let mut up_game = Game::new();
        up_game.start_stage(StageId::Level2A);
        up_game.screen = AppScreen::Playing;
        up_game.labs_elevators[0].y = 2750.0;
        up_game.player.x = 308.95 - up_game.level_origin_x();
        up_game.player.y = (2750.0 - 75.0) - up_game.level_origin_y();
        up_game.player.dy = 0.0;
        up_game.player.jumping = false;
        let mut neutral = InputState::default();
        up_game.tick(&mut neutral, &open_air);
        assert!(up_game.on_platform && (up_game.labs_elevators[0].dy - (-12.0)).abs() < 0.01);
        up_game.tick(&mut neutral, &open_air);
        let expected_gy = (up_game.labs_elevators[0].y + 12.0) - 75.0 - 40.0;
        assert!(((up_game.player.y + up_game.level_origin_y()) - expected_gy).abs() < 0.01);

        let mut jump = InputState::default();
        jump.pressed[VK_SPACE] = true;
        up_game.tick(&mut jump, &open_air);
        assert!(!up_game.on_platform);
        assert!((up_game.player.dy - (-36.0 + (-12.0) / 3.0)).abs() < 0.01);
    }
}
