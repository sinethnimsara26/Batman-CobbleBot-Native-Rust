use crate::assets::Assets;
use crate::collision::CollisionMask;
use crate::game::{AppScreen,AudioEvent,Game,InputState,LEVEL_X,LEVEL_Y,LOGICAL_H,LOGICAL_W,PICKUPS,VK_D,VK_ENTER,VK_RIGHT,VK_S,VK_SPACE};
use crate::render;
use crate::surface::RenderSurface;
use png::{BitDepth,ColorType,Encoder};
use std::fs::{self,File};
use std::io::BufWriter;
use std::path::{Path,PathBuf};
use std::time::Instant;

pub fn render_smoke(out:&str){render_smoke_mode(out,false);}
pub fn render_smoke_hq(out:&str){render_smoke_mode(out,true);}

fn render_smoke_mode(out:&str,hq:bool){
    let out=PathBuf::from(out);
    fs::create_dir_all(&out).expect("create visual-smoke directory");

    let assets=Assets::load();
    let ground=CollisionMask::level1a();
    let mut game=Game::new();
    let mut input=InputState::default();

    // Start the recovered Flash intro with keyboard input, no mouse.
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert_eq!(game.screen,AppScreen::Intro);
    assert_eq!(game.intro_tick,0);
    assert_eq!(game.take_audio_events(),vec![AudioEvent::IntroStart]);
    assert_eq!(assets.intro_frames.len(),180);
    snapshot(hq,&out,"00_intro_start.png",&mut game,&assets);
    for _ in 0..80 {game.tick(&mut input,&ground);}
    assert_eq!(game.screen,AppScreen::Intro);
    snapshot(hq,&out,"00_intro_call.png",&mut game,&assets);
    // INTRO-QA-01: capture EVERY final intro sample 175..=179 (not just a
    // screenshot at 179), then the first playing frame. This deterministic
    // renderer evidence detects a stuck visual even if the game state changes.
    // A near-black source frame is NOT automatically a regression.
    for _ in 0..95 {game.tick(&mut input,&ground);}
    let mut transition_records=Vec::new();
    for source_frame in 175_u16..=179 {
        assert_eq!(game.screen,AppScreen::Intro);
        assert_eq!(game.intro_tick,source_frame);
        let (near_black,fingerprint)=intro_frame_metrics(hq,&mut game,&assets);
        transition_records.push(format!(
            "    {{\"phase\":\"intro\",\"frame\":{},\"near_black_fraction\":{:.6},\"pixel_fingerprint\":\"{:016x}\"}}",
            source_frame,near_black,fingerprint
        ));
        if source_frame<179 {game.tick(&mut input,&ground);}
    }
    assert_eq!(game.take_audio_events(),vec![AudioEvent::IntroStop]);
    snapshot(hq,&out,"00_intro_last.png",&mut game,&assets);
    let terminal_fingerprint=intro_frame_metrics(hq,&mut game,&assets).1;
    game.tick(&mut input,&ground);
    assert_eq!(game.screen,AppScreen::Playing);
    assert_eq!(game.ticks,0);
    assert_eq!(game.intro_fadeout_tick,None);
    assert_eq!(game.take_audio_events(),vec![AudioEvent::MusicStart]);
    let (playing_start_dark,playing_start_hash)=intro_frame_metrics(hq,&mut game,&assets);
    transition_records.push(format!(
        "    {{\"phase\":\"playing\",\"tick\":0,\"near_black_fraction\":{:.6},\"pixel_fingerprint\":\"{:016x}\"}}",
        playing_start_dark,playing_start_hash
    ));

    // Verify early keyboard skip remains source-correct: sprite34 has all 41
    // frames and StopSounds precedes gameplay entry/music.
    let mut skipped=Game::new();
    let mut skip_keys=InputState::default();
    skip_keys.set(VK_ENTER,true);
    skipped.tick(&mut skip_keys,&ground);
    skip_keys.set(VK_ENTER,false);
    skipped.tick(&mut skip_keys,&ground);
    skip_keys.set(VK_ENTER,true);
    skipped.tick(&mut skip_keys,&ground);
    skip_keys.set(VK_ENTER,false);
    assert_eq!(skipped.screen,AppScreen::Intro);
    assert_eq!(skipped.intro_fadeout_tick,Some(0));
    for _ in 0..38 { skipped.tick(&mut skip_keys,&ground); }
    assert_eq!(skipped.intro_fadeout_tick,Some(38));
    assert_eq!(skipped.take_audio_events(),vec![AudioEvent::IntroStart]);
    skipped.tick(&mut skip_keys,&ground); // fade #34 frame39 StopSounds
    assert_eq!(skipped.take_audio_events(),vec![AudioEvent::IntroStop]);
    skipped.tick(&mut skip_keys,&ground); // fade frame40 still displayed
    assert_eq!(skipped.screen,AppScreen::Intro);
    skipped.tick(&mut skip_keys,&ground); // original gotoNext=level1a
    assert_eq!(skipped.screen,AppScreen::Playing);
    assert_eq!(skipped.take_audio_events(),vec![AudioEvent::MusicStart]);

    // Win32 release-inside dispatches the same idempotent native Rust fade.
    let mut clicked=Game::new();
    let mut mouse_input=InputState::default();
    assert!(!clicked.skip_intro());
    clicked.screen=AppScreen::Intro;
    assert!(clicked.skip_intro());
    assert!(!clicked.skip_intro());
    assert_eq!(clicked.screen,AppScreen::Intro);
    for _ in 0..40 { clicked.tick(&mut mouse_input,&ground); }
    assert_eq!(clicked.intro_fadeout_tick,Some(40));
    clicked.tick(&mut mouse_input,&ground);
    assert_eq!(clicked.screen,AppScreen::Playing);
    assert_eq!(clicked.take_audio_events(),vec![AudioEvent::IntroStop,AudioEvent::MusicStart]);

    // Preserve the real root presentation instead of skipping straight into
    // gameplay: fade-in starts immediately, level-title continues longer.
    snapshot(hq,&out,"00_fade_start.png",&mut game,&assets);
    // Keep the historical fade screenshots at EXACT ticks 0, 25, 50.
    // Verify no duplicate intro/music events on the first gameplay tick
    // without shifting all subsequent hash-pinned images by one frame.
    game.tick(&mut input,&ground);
    assert!(game.take_audio_events().is_empty());
    for _ in 0..24 { game.tick(&mut input,&ground); }
    snapshot(hq,&out,"01_fade_mid.png",&mut game,&assets);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    snapshot(hq,&out,"02_level_title.png",&mut game,&assets);
    let (playing_50_dark,playing_50_hash)=intro_frame_metrics(hq,&mut game,&assets);
    transition_records.push(format!(
        "    {{\"phase\":\"playing\",\"tick\":50,\"near_black_fraction\":{:.6},\"pixel_fingerprint\":\"{:016x}\"}}",
        playing_50_dark,playing_50_hash
    ));
    // Save evidence BEFORE the assertions so a failing Windows CI uploads it.
    let report=format!(
        "{{\n  \"scenario\": \"natural-intro-to-playing\",\n  \"mode\": \"{}\",\n  \"near_black_rgb_threshold\": 20,\n  \"samples\": [\n{}\n  ]\n}}\n",
        if hq {"hq3x"} else {"legacy1x"},
        transition_records.join(",\n")
    );
    fs::write(out.join("intro-transition-qa.json"),report)
        .expect("write natural intro transition diagnostics");
    assert_ne!(terminal_fingerprint,playing_50_hash,
        "intro terminal image must not remain visually identical to gameplay tick 50");
    assert!(playing_50_dark<0.95,
        "gameplay tick 50 is unexpectedly near-black ({:.3}); inspect intro-transition-qa.json",
        playing_50_dark);

    // Let the 100-frame title timeline clear, then capture clean gameplay.
    while game.ticks < 105 { game.tick(&mut input,&ground); }
    for _ in 0..180 {
        game.tick(&mut input,&ground);
        if !game.player.jumping { break; }
    }
    snapshot(hq,&out,"03_landed.png",&mut game,&assets);

    // Spawn lands inside checkpoint_1a_walk. Once levelTitle is stopped on
    // Flash frame 100, the original checkpoint repeatedly starts "goRight".
    assert!((1..=9).contains(&game.overlay_frame), "expected goRight tutorial, got {}", game.overlay_frame);
    snapshot(hq,&out,"03_go_right_tutorial.png",&mut game,&assets);

    // Staying in that checkpoint for >3 seconds switches to the original
    // "walk" label exactly like getTimer() logic in the SWF.
    while game.ticks < 180 { game.tick(&mut input,&ground); }
    assert!((58..=65).contains(&game.overlay_frame), "expected walk tutorial, got {}", game.overlay_frame);
    snapshot(hq,&out,"03_walk_tutorial.png",&mut game,&assets);

    // Original acceleration: walk frames should ramp into run rather than
    // teleporting straight to full speed.
    input.set(VK_RIGHT,true);
    for _ in 0..4 { game.tick(&mut input,&ground); }
    assert_eq!(format!("{:?}",game.player.state),"Walk");
    snapshot(hq,&out,"04_walk.png",&mut game,&assets);
    for _ in 0..14 { game.tick(&mut input,&ground); }
    assert_eq!(format!("{:?}",game.player.state),"Run");
    snapshot(hq,&out,"04_running.png",&mut game,&assets);
    input.set(VK_RIGHT,false);
    game.tick(&mut input,&ground);

    // Jump and capture the upward pose.
    input.set(VK_SPACE,true);
    game.tick(&mut input,&ground);
    input.set(VK_SPACE,false);
    game.tick(&mut input,&ground);
    assert_eq!(game.take_audio_events(),vec![AudioEvent::Jump]);
    for _ in 0..2 { game.tick(&mut input,&ground); }
    snapshot(hq,&out,"05_jump.png",&mut game,&assets);

    // Hold Space during descent until the original fall->glide rule enters.
    input.set(VK_SPACE,true);
    for _ in 0..80 {
        game.tick(&mut input,&ground);
        if format!("{:?}",game.player.state)=="Glide" { break; }
    }
    snapshot(hq,&out,"06_glide.png",&mut game,&assets);
    input.set(VK_SPACE,false);

    // Settle again, then verify true child-timeline attack animation.
    for _ in 0..180 {
        game.tick(&mut input,&ground);
        if !game.player.jumping { break; }
    }
    input.set(VK_D,true);
    game.tick(&mut input,&ground);
    input.set(VK_D,false);
    for _ in 0..3 { game.tick(&mut input,&ground); }
    snapshot(hq,&out,"07_kick.png",&mut game,&assets);

    // Punch has a special original frame-4 hold gate. A quick tap must return
    // to stand early; a held S must still be punching after that checkpoint.
    for _ in 0..20 { game.tick(&mut input,&ground); }
    input.set(VK_S,true);
    game.tick(&mut input,&ground);
    input.set(VK_S,false);
    for _ in 0..6 { game.tick(&mut input,&ground); }
    assert_ne!(format!("{:?}",game.player.state),"Punch");

    input.set(VK_S,true);
    game.tick(&mut input,&ground);
    for _ in 0..6 { game.tick(&mut input,&ground); }
    assert_eq!(format!("{:?}",game.player.state),"Punch");
    snapshot(hq,&out,"08_held_punch.png",&mut game,&assets);
    input.set(VK_S,false);

    // Force only the recovered tutorial presentation state for a visual
    // checkpoint; gameplay checkpoint logic is separately exercised above.
    game.overlay_frame=140;
    snapshot(hq,&out,"09_to_street.png",&mut game,&assets);

    // Hit the exact recovered Level 1A exit checkpoint and prove the original
    // root fadeout timeline runs to completion without entering Level 1B.
    game.player.x=7600.0-LEVEL_X;
    game.player.y=520.0-LEVEL_Y;
    game.tick(&mut input,&ground);
    assert!(game.level1a_exit_reached);
    assert_eq!(game.fadeout_tick,Some(0));
    snapshot(hq,&out,"10_exit_fade_start.png",&mut game,&assets);
    for _ in 0..20 { game.tick(&mut input,&ground); }
    snapshot(hq,&out,"11_exit_fade_mid.png",&mut game,&assets);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    assert!(game.level1a_complete);
    assert_eq!(game.fadeout_tick,Some(40));
    snapshot(hq,&out,"12_level1a_complete.png",&mut game,&assets);

    // Round 8 visual checkpoint: isolate an uncollected pickup and a live
    // Batarang in the same viewport. Legacy mode proves the original 1x path;
    // HQ mode proves both objects are composited from their true-3x packs.
    let mut object_game=Game::new();
    object_game.screen=AppScreen::Playing;
    object_game.ticks=105;
    object_game.camera_x=-PICKUPS[0].0+180.0;
    object_game.camera_y=-PICKUPS[0].1+180.0;
    object_game.player.x=-LEVEL_X-object_game.camera_x-1000.0;
    object_game.player.y=-LEVEL_Y-object_game.camera_y-1000.0;
    object_game.shots.push(crate::game::Shot{
        x:PICKUPS[0].0+160.0,
        y:PICKUPS[0].1,
        dir:1,
    });
    snapshot(hq,&out,"13_world_objects.png",&mut object_game,&assets);

    // Audio/item regression gate from original itemLogic.
    let mut item_game=Game::new();
    item_game.screen=AppScreen::Playing;
    item_game.player.x=PICKUPS[0].0-LEVEL_X;
    item_game.player.y=PICKUPS[0].1-LEVEL_Y;
    item_game.tick(&mut input,&ground);
    assert_eq!(item_game.batarangs,5);
    assert_eq!(item_game.score,5);
    assert!(item_game.take_audio_events().contains(&AudioEvent::Item));

    // Level 1A pitLogic regression gate. Symbol 149 is invisible gameplay
    // geometry, so force Batman into its exact level-local hit region.
    let mut pit_game=Game::new();
    pit_game.screen=AppScreen::Playing;
    pit_game.player.x=0.0;
    pit_game.player.y=500.0;
    pit_game.tick(&mut input,&ground);
    assert_eq!(pit_game.lives,4);
    assert_eq!(pit_game.fadeout_tick,Some(0));
    for _ in 0..41 { pit_game.tick(&mut input,&ground); }
    assert_eq!(pit_game.screen,AppScreen::Playing);
    assert_eq!(pit_game.lives,4);
    assert_eq!(pit_game.fadeout_tick,None);
    assert_eq!(pit_game.ticks,0);
}

fn snapshot(hq:bool,out:&Path,name:&str,game:&mut Game,assets:&Assets){
    if hq{
        let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
        let mut fb=RenderSurface::new(render::HQ_W,render::HQ_H);
        render::render_high_quality(&mut base,&mut fb,game,assets);
        write_png(&out.join(name),&fb);
    }else{
        let mut fb=RenderSurface::new(LOGICAL_W,LOGICAL_H);
        render::render(&mut fb,game,assets);
        write_png(&out.join(name),&fb);
    }
}

// Returns a deterministic visual fingerprint and fraction of pixels whose
// RGB channels are ALL below 20. Platform screen capture is not involved:
// these are actual native Rust renderer pixels, not a Win32 window test.
fn intro_frame_metrics(hq:bool,game:&mut Game,assets:&Assets)->(f64,u64){
    let fb=if hq {
        let mut logical=RenderSurface::new(LOGICAL_W,LOGICAL_H);
        let mut output=RenderSurface::new(render::HQ_W,render::HQ_H);
        render::render_high_quality(&mut logical,&mut output,game,assets);
        output
    } else {
        let mut output=RenderSurface::new(LOGICAL_W,LOGICAL_H);
        render::render(&mut output,game,assets);
        output
    };
    let dark=fb.pixels.iter().filter(|&&px|{
        ((px>>16)&255)<20 && ((px>>8)&255)<20 && (px&255)<20
    }).count();
    let fingerprint=fb.pixels.iter().fold(0xcbf29ce484222325_u64,|hash,&px|{
        (hash ^ u64::from(px)).wrapping_mul(0x100000001b3)
    });
    (dark as f64/fb.pixels.len() as f64,fingerprint)
}

pub fn benchmark_render(iterations:usize,out:&str){
    assert!(iterations>0,"benchmark iterations must be positive");
    let assets=Assets::load();
    let ground=CollisionMask::level1a();
    let mut game=Game::new();
    let mut input=InputState::default();

    // Benchmark a real gameplay frame rather than the lightweight title page.
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    // Native completion after the final 180 story frames must be non-blocking
    // (the source's final sample is nearly black). SKIP still takes its own
    // source fade path; this benchmark measures natural Level1A entry.
    for _ in 0..179 { game.tick(&mut input,&ground); }
    assert_eq!(game.screen,AppScreen::Intro);
    assert_eq!(game.intro_tick,179);
    game.tick(&mut input,&ground);
    assert_eq!(game.screen,AppScreen::Playing);
    assert_eq!(game.intro_fadeout_tick,None);
    assert_eq!(game.ticks,0);
    while game.ticks<105 { game.tick(&mut input,&ground); }

    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);

    // Warm caches and branch predictors before collecting samples.
    for _ in 0..5{
        render::render_high_quality(&mut base,&mut hq,&game,&assets);
    }

    let mut samples=Vec::with_capacity(iterations);
    for _ in 0..iterations{
        let start=Instant::now();
        render::render_high_quality(&mut base,&mut hq,&game,&assets);
        samples.push(start.elapsed().as_secs_f64()*1000.0);
    }
    samples.sort_by(|a,b|a.partial_cmp(b).unwrap());
    let mean=samples.iter().sum::<f64>()/samples.len() as f64;
    let p50=samples[((samples.len()-1) as f64*0.50).round() as usize];
    let p95=samples[((samples.len()-1) as f64*0.95).round() as usize];
    let max=*samples.last().unwrap();
    let framebuffer_bytes=base.byte_len()+hq.byte_len();
    let report=format!(
        "{{\n  \"mode\": \"hq3x\",\n  \"iterations\": {},\n  \"logical_width\": {},\n  \"logical_height\": {},\n  \"hq_width\": {},\n  \"hq_height\": {},\n  \"mean_ms\": {:.4},\n  \"p50_ms\": {:.4},\n  \"p95_ms\": {:.4},\n  \"max_ms\": {:.4},\n  \"framebuffer_bytes\": {}\n}}\n",
        iterations,LOGICAL_W,LOGICAL_H,render::HQ_W,render::HQ_H,mean,p50,p95,max,framebuffer_bytes
    );
    fs::write(out,report).expect("write render benchmark report");
}

pub fn benchmark_scenery_scroll(out:&str){
    let assets=Assets::load();
    let mut game=Game::new();
    game.screen=AppScreen::Playing;
    game.ticks=105;
    game.camera_y=0.0;
    game.player.x=100000.0;
    game.player.y=100000.0;
    game.pickups.fill(true);
    game.shots.clear();

    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);

    let forward:Vec<i32>=(-1200..=10800).step_by(600).collect();
    let positions:Vec<i32>=forward.iter().copied()
        .chain(forward.iter().rev().skip(1).copied())
        .collect();
    let mut records=Vec::with_capacity(positions.len());
    let mut timings=Vec::with_capacity(positions.len());

    for world_left in positions{
        game.camera_x=-(world_left as f32);
        let start=Instant::now();
        render::render_high_quality(&mut base,&mut hq,&game,&assets);
        let ms=start.elapsed().as_secs_f64()*1000.0;
        records.push((world_left,ms));
        timings.push(ms);
    }

    let mut sorted=timings.clone();
    sorted.sort_by(|a,b|a.partial_cmp(b).unwrap());
    let mean=sorted.iter().sum::<f64>()/sorted.len() as f64;
    let p50=sorted[((sorted.len()-1) as f64*0.50).round() as usize];
    let p95=sorted[((sorted.len()-1) as f64*0.95).round() as usize];
    let max=*sorted.last().unwrap();

    let samples=records.iter().map(|(left,ms)|
        format!("    {{\"world_left\": {}, \"ms\": {:.4}}}",left,ms)
    ).collect::<Vec<_>>().join(",\n");
    let report=format!(
        "{{\n  \"mode\": \"hq3x-scenery-scroll\",\n  \"step_pixels\": 600,\n  \"samples_count\": {},\n  \"mean_ms\": {:.4},\n  \"p50_ms\": {:.4},\n  \"p95_ms\": {:.4},\n  \"max_ms\": {:.4},\n  \"samples\": [\n{}\n  ]\n}}\n",
        records.len(),mean,p50,p95,max,samples
    );
    fs::write(out,report).expect("write scenery scroll benchmark report");
}

fn write_png(path:&Path,fb:&RenderSurface){
    let file=File::create(path).expect("create PNG");
    let writer=BufWriter::new(file);
    let mut encoder=Encoder::new(writer,fb.w as u32,fb.h as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer=encoder.write_header().expect("PNG header");
    let mut bytes=Vec::with_capacity(fb.byte_len());
    for &px in &fb.pixels {
        bytes.push(((px>>16)&255) as u8);
        bytes.push(((px>>8)&255) as u8);
        bytes.push((px&255) as u8);
        bytes.push(255);
    }
    writer.write_image_data(&bytes).expect("PNG pixels");
}

pub fn qa_bank_route(out_dir:&str){
    let out=Path::new(out_dir);
    fs::create_dir_all(out).expect("create B19 bank QA dir");
    let mut assets=Assets::load();
    let mut ground=CollisionMask::for_stage(crate::stage::StageId::Level1A);
    let mut game=Game::new();
    let mut input=InputState::default();

    // Title -> Intro -> Level 1A
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert!(game.skip_intro());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    for _ in 0..105 { game.tick(&mut input,&ground); }
    assert_eq!(game.active_stage(),crate::stage::StageId::Level1A);
    snapshot(false,out,"bank_1a.png",&mut game,&assets);
    snapshot(true,out,"bank_1a_hq.png",&mut game,&assets);
    let (_,fp_1a)=intro_frame_metrics(true,&mut game,&assets);

    // Progress through Level 1A exit -> Level 1B
    game.player.x=PICKUPS[0].0-LEVEL_X;
    game.player.y=PICKUPS[0].1-LEVEL_Y;
    game.tick(&mut input,&ground);
    game.level1a_exit_reached=true;
    game.fadeout_tick=Some(0);
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let next_1b=game.consume_pending_stage_entry().expect("Level1B handoff");
    assert_eq!(next_1b,crate::stage::StageId::Level1B);
    ground=CollisionMask::for_stage(next_1b);
    assets.load_stage(next_1b);
    // Position Batman in combat range of the first Bank goon & biker for visual QA
    let d1b=crate::stage::StageId::Level1B.descriptor();
    game.player.x=-1180.0-d1b.level_origin_game_local.x;
    game.player.dir=1;
    for _ in 0..45 { game.tick(&mut input,&ground); }
    snapshot(false,out,"bank_1b.png",&mut game,&assets);
    snapshot(true,out,"bank_1b_hq.png",&mut game,&assets);
    let (dark_1b,fp_1b)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_1b,fp_1a,"Level 1B HQ frame must differ from Level 1A");
    assert!(dark_1b<0.95,"Level 1B frame unexpectedly blank");

    // Measure HQ frame time on Level 1B
    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);
    render::render_high_quality(&mut base,&mut hq,&game,&assets);
    let mut t1b=Vec::with_capacity(20);
    for _ in 0..20 {
        let start=Instant::now();
        render::render_high_quality(&mut base,&mut hq,&game,&assets);
        t1b.push(start.elapsed().as_secs_f64()*1000.0);
    }
    t1b.sort_by(|a,b|a.partial_cmp(b).unwrap());
    let p95_1b=t1b[((t1b.len()-1) as f64*0.95).round() as usize];

    // Clear Level 1B waves -> Level 1C
    if let Some(squad)=&mut game.bank_goons {
        squad.e_kills=10;
    }
    game.tick(&mut input,&ground);
    if let Some(squad)=&mut game.bank_goons {
        assert_eq!(squad.respawned,1);
        squad.e_kills=8;
    }
    game.tick(&mut input,&ground);
    assert!(game.level1b_exit_reached);
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let next_1c=game.consume_pending_stage_entry().expect("Level1C handoff");
    assert_eq!(next_1c,crate::stage::StageId::Level1C);
    ground=CollisionMask::for_stage(next_1c);
    assets.load_stage(next_1c);
    let d1c=crate::stage::StageId::Level1C.descriptor();
    if let Some(boss)=&game.penguin_boss {
        game.player.x=boss.x-180.0-d1c.level_origin_game_local.x;
        game.player.dir=1;
    }
    for _ in 0..45 { game.tick(&mut input,&ground); }
    snapshot(false,out,"bank_1c.png",&mut game,&assets);
    snapshot(true,out,"bank_1c_hq.png",&mut game,&assets);
    let (dark_1c,fp_1c)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_1c,fp_1a);
    assert_ne!(fp_1c,fp_1b);
    assert!(dark_1c<0.95,"Level 1C frame unexpectedly blank");

    let mut t1c=Vec::with_capacity(20);
    for _ in 0..20 {
        let start=Instant::now();
        render::render_high_quality(&mut base,&mut hq,&game,&assets);
        t1c.push(start.elapsed().as_secs_f64()*1000.0);
    }
    t1c.sort_by(|a,b|a.partial_cmp(b).unwrap());
    let p95_1c=t1c[((t1c.len()-1) as f64*0.95).round() as usize];

    // Defeat Penguin -> EscapeCard -> Level2A pending handoff
    game.player.jumping=false;
    game.player.on_ground=true;
    game.player.dy=0.0;
    let (bx,by)= {
        let boss=game.penguin_boss.as_mut().expect("1C boss");
        boss.health=1;
        boss.state=crate::penguin::PenguinState::Stand;
        (boss.x,boss.y)
    };
    let d1c=crate::stage::StageId::Level1C.descriptor();
    game.player.x=bx-d1c.level_origin_game_local.x-20.0;
    game.player.y=by-d1c.level_origin_game_local.y;
    game.player.dir=1;
    game.player.state=crate::game::PlayerState::Punch;
    game.player.state_tick=1;
    game.player_attacking=true;
    for _ in 0..45 {
        game.tick(&mut input,&ground);
        if game.fadeout_tick.is_some() { break; }
    }
    for _ in 0..42 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::EscapeCard { break; }
    }
    assert_eq!(game.screen,AppScreen::EscapeCard);
    snapshot(false,out,"escape_card_000.png",&mut game,&assets);
    let (_,fp_esc0)=intro_frame_metrics(false,&mut game,&assets);
    for _ in 0..112 { game.tick(&mut input,&ground); }
    snapshot(false,out,"escape_card_112.png",&mut game,&assets);
    snapshot(true,out,"escape_card_112_hq.png",&mut game,&assets);
    let (_,fp_esc112)=intro_frame_metrics(false,&mut game,&assets);
    assert_ne!(fp_esc0,fp_esc112,"EscapeCard must animate");
    assert!(game.skip_escape_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let pending=game.pending_stage_entry().expect("Level2A handoff latched");
    assert_eq!(pending.to,crate::stage::StageId::Level2A);

    let report=format!(
        "{{\n  \"milestone\": \"B19\",\n  \"route\": [\"Level1A\", \"Level1B\", \"Level1C\", \"EscapeCard\", \"Pending(Level2A)\"],\n  \"final_score\": {},\n  \"final_lives\": {},\n  \"final_batarangs\": {},\n  \"fingerprints\": {{\n    \"level1a_hq\": \"0x{fp_1a:016x}\",\n    \"level1b_hq\": \"0x{fp_1b:016x}\",\n    \"level1c_hq\": \"0x{fp_1c:016x}\",\n    \"escape_card_000\": \"0x{fp_esc0:016x}\",\n    \"escape_card_112\": \"0x{fp_esc112:016x}\"\n  }},\n  \"dark_fraction\": {{\n    \"level1b_hq\": {dark_1b:.4},\n    \"level1c_hq\": {dark_1c:.4}\n  }},\n  \"hq_frame_p95_ms\": {{\n    \"level1b\": {p95_1b:.4},\n    \"level1c\": {p95_1c:.4}\n  }},\n  \"keyframes\": [\n    \"bank_1a.png\", \"bank_1a_hq.png\", \"bank_1b.png\", \"bank_1b_hq.png\",\n    \"bank_1c.png\", \"bank_1c_hq.png\", \"escape_card_000.png\", \"escape_card_112.png\", \"escape_card_112_hq.png\"\n  ]\n}}\n",
        pending.progress.score,pending.progress.lives,pending.progress.batarangs
    );
    fs::write(out.join("bank_route_qa_report.json"),report).expect("write B19 QA report");
}
