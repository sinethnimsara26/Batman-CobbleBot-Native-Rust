use crate::collision::CollisionMask;

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
const CAM_HEIGHT:f32=150.0;
const CAM_JUMP:f32=1.25;
const CAM_MIN:f32=-1350.0;
const CAM_MAX:f32=7650.0;
const CAM_OFFSET:f32=0.0;

pub const PICKUPS:[(f32,f32);7]=[
    (-803.05,19.85),(873.3,-9.0),(1682.9,73.45),(2546.5,62.35),
    (3396.0,112.3),(4448.1,133.4),(5534.95,162.25)
];

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

// Original overlay (symbol 737) label starts, converted to zero-based frames.
const OVERLAY_BLANK:usize=0;
const OVERLAY_GO_RIGHT:usize=1;
const OVERLAY_JUMP:usize=20;
const OVERLAY_WRONGWAY:usize=30;
const OVERLAY_GLIDE_DOWN:usize=39;
const OVERLAY_WALK:usize=58;
const OVERLAY_TO_STREET:usize=140;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum AppScreen{Title,Instructions,Playing}

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
    pub screen:AppScreen,
    pub player:Player,
    pub camera_x:f32,pub camera_y:f32,
    pub background_x:f32,pub background_y:f32,
    gravity:f32,
    pub batarangs:i32,
    pub score:i32,
    pub player_attacking:bool,
    pub overlay_frame:usize,
    overlay_playing:bool,
    walk_checkpoint_started_tick:Option<u64>,
    pub level1a_exit_reached:bool,
    pub pickups:[bool;7],
    pub shots:Vec<Shot>,
    pub ticks:u64,
}

impl Game{
    pub fn new()->Self{
        Self{
            screen:AppScreen::Title,
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
            gravity:GLOBAL_GRAVITY,
            batarangs:0,score:0,player_attacking:false,
            overlay_frame:OVERLAY_BLANK,overlay_playing:false,walk_checkpoint_started_tick:None,level1a_exit_reached:false,
            pickups:[false;7],shots:Vec::new(),ticks:0,
        }
    }

    pub fn animation_frame(&self)->usize{
        self.player.state.display_tick(self.player.state_tick)
    }

    fn enter(&mut self,state:PlayerState){
        if self.player.state!=state{
            self.player.state=state;
            self.player.state_tick=0;
        }
    }

    pub fn tick(&mut self,input:&mut InputState,ground:&CollisionMask){
        match self.screen{
            AppScreen::Title=>{
                if input.pressed(VK_I){self.screen=AppScreen::Instructions}
                else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE){
                    self.screen=AppScreen::Playing;
                    // The original level begins with playerJumping=true, so
                    // cameraLogic snaps vertically on its first enterFrame.
                    self.update_camera();
                }
                input.clear_pressed();
                return
            },
            AppScreen::Instructions=>{
                if input.pressed(VK_ESCAPE){self.screen=AppScreen::Title}
                else if input.pressed(VK_ENTER)||input.pressed(VK_SPACE){
                    self.screen=AppScreen::Playing;
                    self.update_camera();
                }
                input.clear_pressed();
                return
            },
            AppScreen::Playing=>{}
        }

        self.ticks+=1;
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

        self.collect_pickups();
        self.update_shots();
        self.update_tutorial_checkpoints();
        self.update_camera();

        if self.player.state==state_at_tick_start{
            self.player.state_tick=self.player.state_tick.saturating_add(1);
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
        }else{
            self.player.y+=self.player.dy;
        }

        let grounded=ground.check_ground(
            self.player.x,&mut self.player.y,PLAYER_H,&mut self.player.dy,0
        );
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

        let game_x=self.player.x+LEVEL_X;
        let game_y=self.player.y+LEVEL_Y;
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

    fn update_tutorial_checkpoints(&mut self){
        let x=self.player.x+LEVEL_X;
        let y=self.player.y+LEVEL_Y;
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

        if CP_NEXT.hits_player(x,y){
            // The original sets gotoNext="level1b" and starts fadeout here.
            // Campaign expansion stays frozen, so expose the exact exit event
            // now and wire the original fade-out before Level 1B is enabled.
            self.level1a_exit_reached=true;
        }
    }

    fn collect_pickups(&mut self){
        let px=self.player.x+LEVEL_X;
        let py=self.player.y+LEVEL_Y;
        for(i,&(x,y))in PICKUPS.iter().enumerate(){
            if self.pickups[i]{continue;}
            // Native approximation of Flash movie-clip hitTest bounds. The
            // art itself is original; exact alpha hit testing can replace this
            // once the pickup registration box is extracted.
            if(px-x).abs()<45.0&&(py-y).abs()<55.0{
                self.pickups[i]=true;
                self.batarangs+=5;
                self.score+=5;
            }
        }
    }

    fn update_shots(&mut self){
        let px=self.player.x+LEVEL_X;
        for shot in &mut self.shots{shot.x+=shot.dir as f32*50.0;}
        self.shots.retain(|shot|(shot.x-px).abs()<500.0);
    }

    fn update_camera(&mut self){
        // Exact translation of the original cameraLogic() recovered from the
        // decompiled ActionScript, using game-local player coordinates.
        let bg_width=2472.0_f32; // temporary measured/baked width; wrap rule is exact.
        let bg_diff=self.background_x+self.camera_x;
        if bg_diff>0.0{
            self.background_x-=bg_width/2.0;
        }else if bg_diff< -(bg_width/2.0){
            self.background_x+=bg_width/2.0;
        }

        let player_x=self.player.x+LEVEL_X;
        let player_y=self.player.y+LEVEL_Y;
        let target_x=if self.player.dir>0{125.0}else{475.0};
        let player_position_x=player_x+self.camera_x;

        if player_x>CAM_MIN&&player_x<CAM_MAX{
            let delta=player_position_x-target_x;
            if delta.abs()>4.0{
                let ease=(delta/10.0).trunc();
                self.camera_x-=ease;
                self.background_x+=ease/2.0;
            }
        }

        if self.player.jumping{
            self.camera_y=(CAM_HEIGHT-player_y)/CAM_JUMP;
            self.background_y=(player_y/2.0-CAM_HEIGHT/1.5)/CAM_JUMP-CAM_OFFSET;
        }else if (self.camera_y+player_y-150.0).abs()>1.0{
            let target=(CAM_HEIGHT-player_y)/CAM_JUMP;
            self.camera_y+=((target-self.camera_y)/4.0).trunc();
            self.background_y=((player_y/2.0-CAM_HEIGHT/1.5)/CAM_JUMP).trunc()-CAM_OFFSET;
        }
    }
}
