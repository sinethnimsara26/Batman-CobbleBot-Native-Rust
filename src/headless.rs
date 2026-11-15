use crate::assets::Assets;
use crate::collision::CollisionMask;
use crate::game::{AppScreen,AudioEvent,Game,InputState,LEVEL_X,LEVEL_Y,LOGICAL_H,LOGICAL_W,PICKUPS,VK_A,VK_D,VK_ENTER,VK_RIGHT,VK_S,VK_SPACE};
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
    object_game.shots.push(crate::game::Shot::new(
        PICKUPS[0].0+160.0,
        PICKUPS[0].1,
        1,
    ));
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

fn measure_hq_p95_ms(base:&mut RenderSurface,hq:&mut RenderSurface,game:&Game,assets:&Assets)->f64{
    render::render_high_quality(base,hq,game,assets);
    let mut times=Vec::with_capacity(20);
    for _ in 0..20 {
        let start=Instant::now();
        render::render_high_quality(base,hq,game,assets);
        times.push(start.elapsed().as_secs_f64()*1000.0);
    }
    times.sort_by(|a,b|a.partial_cmp(b).unwrap());
    times[((times.len()-1) as f64*0.95).round() as usize]
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
    let p95_1b=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

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

    let p95_1c=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

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

pub fn qa_labs2a_route(out_dir:&str){
    use crate::stage::{labs2a,StageId};
    let out=Path::new(out_dir);
    fs::create_dir_all(out).expect("create B26 Labs2A QA dir");
    let mut assets=Assets::load();
    let mut ground=CollisionMask::for_stage(StageId::Level1C);
    let mut game=Game::new();
    let mut input=InputState::default();

    // Enter Level2A via live EscapeCard -> consume_pending_stage_entry handoff
    game.screen=AppScreen::EscapeCard;
    game.escape_card_tick=224;
    game.score=650;
    game.lives=4;
    game.batarangs=15;
    assert!(game.skip_escape_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let next_2a=game.consume_pending_stage_entry().expect("Level2A live handoff");
    assert_eq!(next_2a,StageId::Level2A);
    assert_eq!(game.active_stage(),StageId::Level2A);
    ground=CollisionMask::for_stage(next_2a);
    assets.load_stage(next_2a);

    // 1. Level 2A spawn + LEVEL 2 banner
    for _ in 0..25 { game.tick(&mut input,&ground); }
    snapshot(false,out,"labs2a_spawn.png",&mut game,&assets);
    snapshot(true,out,"labs2a_spawn_hq.png",&mut game,&assets);
    let (dark_2a_spawn,fp_2a_spawn)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_2a_spawn<0.95,"Level 2A spawn frame unexpectedly blank");

    // 2. Finish LEVEL 2 banner, trigger checkpoint_2a_01 (findLab) & pickups/combat
    for _ in 0..80 { game.tick(&mut input,&ground); }
    let d2a=StageId::Level2A.descriptor();
    let cp01=labs2a::CHECKPOINTS[0];
    let (cx01,cy01,_,_)=cp01.game_bounds();
    game.player.x=(cx01+30.0)-d2a.level_origin_game_local.x;
    game.player.y=(cy01+30.0)-d2a.level_origin_game_local.y;
    game.tick(&mut input,&ground);
    assert_eq!(game.overlay_frame,66);
    snapshot(true,out,"labs2a_findlab_hq.png",&mut game,&assets);

    // Move near health01 + nearby batarang/halo/goon and let eased camera settle on Batman
    let h=labs2a::HEALTH_PICKUPS[0];
    game.player.x=(h.x-60.0)-d2a.level_origin_game_local.x;
    game.player.y=(h.y-20.0)-d2a.level_origin_game_local.y;
    for _ in 0..65 { game.tick(&mut input,&ground); }
    snapshot(false,out,"labs2a_pickups_combat.png",&mut game,&assets);
    snapshot(true,out,"labs2a_pickups_combat_hq.png",&mut game,&assets);
    let (dark_2a_mid,fp_2a_mid)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_2a_mid,fp_2a_spawn,"Labs2A camera pan must change rendered pixels");
    assert!(dark_2a_mid<0.85,"Labs2A mid-level frame too dark ({dark_2a_mid:.4})");

    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);
    let p95_2a=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

    // 3. Trigger checkpoint_2a_tv -> 41-frame fadeout -> TvCard
    let tv_cp=labs2a::CHECKPOINTS.iter().find(|c|c.name=="checkpoint_2a_tv").expect("checkpoint_2a_tv");
    let (tx,ty,_,_)=tv_cp.game_bounds();
    game.player.x=(tx+20.0)-d2a.level_origin_game_local.x;
    game.player.y=(ty+20.0)-d2a.level_origin_game_local.y;
    for _ in 0..42 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::TvCard { break; }
    }
    assert_eq!(game.screen,AppScreen::TvCard);
    snapshot(false,out,"tv_card_000.png",&mut game,&assets);
    let (_,fp_tv0)=intro_frame_metrics(false,&mut game,&assets);
    for _ in 0..86 { game.tick(&mut input,&ground); }
    snapshot(true,out,"tv_card_086_hq.png",&mut game,&assets);
    for _ in 0..24 { game.tick(&mut input,&ground); }
    snapshot(false,out,"tv_card_110.png",&mut game,&assets);
    let (_,fp_tv110)=intro_frame_metrics(false,&mut game,&assets);
    assert_ne!(fp_tv0,fp_tv110,"TvCard must animate");
    assert!(game.skip_tv_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    assert!(game.tv_card_complete);
    let pending=game.pending_stage_entry().expect("Level2B handoff latched");
    assert_eq!(pending.from,StageId::Level2A);
    assert_eq!(pending.to,StageId::Level2B);
    assert_eq!(pending.outgoing_root_sound_id,130);
    assert_eq!(game.consume_pending_stage_entry(),Some(StageId::Level2B));
    assert_eq!(game.active_stage(),StageId::Level2B);
    assert_eq!(game.pickups.len(),10);

    let report=format!(
        "{{\n  \"milestone\": \"B26\",\n  \"route\": [\"EscapeCard\", \"Level2A\", \"TvCard\", \"Pending(Level2B)\"],\n  \"final_score\": {},\n  \"final_lives\": {},\n  \"final_batarangs\": {},\n  \"fingerprints\": {{\n    \"labs2a_spawn_hq\": \"0x{fp_2a_spawn:016x}\",\n    \"labs2a_mid_hq\": \"0x{fp_2a_mid:016x}\",\n    \"tv_card_000\": \"0x{fp_tv0:016x}\",\n    \"tv_card_110\": \"0x{fp_tv110:016x}\"\n  }},\n  \"dark_fraction\": {{\n    \"labs2a_spawn_hq\": {dark_2a_spawn:.4},\n    \"labs2a_mid_hq\": {dark_2a_mid:.4}\n  }},\n  \"hq_frame_p95_ms\": {{\n    \"level2a\": {p95_2a:.4}\n  }},\n  \"keyframes\": [\n    \"labs2a_spawn.png\", \"labs2a_spawn_hq.png\", \"labs2a_findlab_hq.png\",\n    \"labs2a_pickups_combat.png\", \"labs2a_pickups_combat_hq.png\",\n    \"tv_card_000.png\", \"tv_card_086_hq.png\", \"tv_card_110.png\"\n  ]\n}}\n",
        pending.progress.score,pending.progress.lives,pending.progress.batarangs
    );
    fs::write(out.join("labs2a_route_qa_report.json"),report).expect("write B26 QA report");
}

pub fn qa_level2_route(out_dir:&str){
    use crate::stage::{labs2a,labs2b,StageId};
    let out=Path::new(out_dir);
    fs::create_dir_all(out).expect("create B32 Level2 QA dir");
    let mut assets=Assets::load();
    let mut ground=CollisionMask::for_stage(StageId::Level1C);
    let mut game=Game::new();
    let mut input=InputState::default();

    // 1. Live EscapeCard -> StageId::Level2A handoff
    game.screen=AppScreen::EscapeCard;
    game.escape_card_tick=224;
    game.score=650;
    game.lives=4;
    game.batarangs=15;
    assert!(game.skip_escape_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let next_2a=game.consume_pending_stage_entry().expect("Level2A live handoff");
    assert_eq!(next_2a,StageId::Level2A);
    assert_eq!(game.active_stage(),StageId::Level2A);
    let audio_2a=game.take_audio_events();
    assert!(audio_2a.contains(&AudioEvent::StageTrack(StageId::Level2A)));
    assert!(audio_2a.contains(&AudioEvent::MusicStart));
    ground=CollisionMask::for_stage(next_2a);
    assets.load_stage(next_2a);

    // 2. Level 2A: spawn + banner -> checkpoint_2a_01 -> health01 + item_br01 -> door + elevator + goon kill
    for _ in 0..25 { game.tick(&mut input,&ground); }
    snapshot(false,out,"l2_2a_spawn.png",&mut game,&assets);
    snapshot(true,out,"l2_2a_spawn_hq.png",&mut game,&assets);
    let (dark_2a_spawn,fp_2a_spawn)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_2a_spawn<0.95,"Level 2A spawn frame unexpectedly blank");

    for _ in 0..80 { game.tick(&mut input,&ground); }
    let d2a=StageId::Level2A.descriptor();
    let (cx01,cy01,_,_)=labs2a::CHECKPOINTS[0].game_bounds();
    game.player.x=(cx01+30.0)-d2a.level_origin_game_local.x;
    game.player.y=(cy01+30.0)-d2a.level_origin_game_local.y;
    game.tick(&mut input,&ground);
    assert_eq!(game.overlay_frame,66,"checkpoint_2a_01 must trigger findLab");

    // Collect health01 & item_br01
    let h=labs2a::HEALTH_PICKUPS[0];
    game.player.x=h.x-d2a.level_origin_game_local.x;
    game.player.y=h.y-d2a.level_origin_game_local.y;
    game.player.dy=0.0;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert!(game.labs_health_collected);
    assert!(game.take_audio_events().contains(&AudioEvent::Item));

    let p2a=labs2a::BATARANG_PICKUPS[0];
    game.player.x=p2a.x-d2a.level_origin_game_local.x;
    game.player.y=p2a.y-d2a.level_origin_game_local.y;
    game.tick(&mut input,&ground);
    assert!(game.pickups[0]);
    assert_eq!(game.batarangs,20);

    // Trigger #1325 door opening + down elevator movement
    let door2a=labs2a::DOORS[0];
    game.player.x=door2a.x-d2a.level_origin_game_local.x;
    game.player.y=door2a.y-d2a.level_origin_game_local.y;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert!(game.labs_doors[0].playing || game.labs_doors[0].frame>0);
    assert!(game.take_audio_events().contains(&AudioEvent::DoorOpen));

    // Engage & defeat goon2 with real batarang shots (`VK_A`, 5 dmg x 14 = 70 HP) while eagle1 remains inert (`y < 0`)
    assert_eq!(game.labs_eagles.len(),1);
    let eagle_y0=game.labs_eagles[0].y;
    let g0=labs2a::GOONS[0];
    let score_before_kill=game.score;
    game.player.x=(g0.x-75.0)-d2a.level_origin_game_local.x;
    game.player.y=(g0.y-75.0)-d2a.level_origin_game_local.y;
    game.player.dir=1;
    game.player.dx=0.0;
    game.player.dy=0.0;
    game.player.jumping=false;
    game.player.on_ground=true;
    for step in 0..280 {
        let (gx, gh) = {
            let g = &game.bank_goons.as_ref().unwrap().goons[0];
            (g.x, g.health)
        };
        if gh > 0 {
            game.player.x=(gx-90.0)-d2a.level_origin_game_local.x;
            game.player.y=(g0.y-75.0)-d2a.level_origin_game_local.y;
            game.player.dir=1;
            game.player.dy=0.0;
            if step%9==0 && game.batarangs>0 {
                input.set(VK_A,true);
            }
        }
        game.tick(&mut input,&ground);
        input.set(VK_A,false);
        if game.bank_goons.as_ref().map(|s|s.e_kills).unwrap_or(0)>=1 { break; }
    }
    let kills_2a=game.bank_goons.as_ref().map(|s|s.e_kills).unwrap_or(0);
    assert!(kills_2a>=1,"must defeat at least 1 Level 2A goon");
    assert!(game.score>score_before_kill);
    assert_eq!(game.labs_eagles[0].y,eagle_y0,"eagle1 must remain inert in Level 2A");

    // Verify real melee attack (`VK_S`) also lands on goon1 in Level 2A
    let (g1x, g1y, g1_hp_before) = {
        let g1 = &mut game.bank_goons.as_mut().unwrap().goons[1];
        if g1.health <= 0 || !g1.active {
            g1.health = 70;
            g1.active = true;
        }
        g1.state = crate::bank_goons::GoonState::Stand;
        g1.invincible = false;
        (g1.x, g1.y, g1.health)
    };
    game.player.state=crate::game::PlayerState::Stand;
    game.player.x=(g1x-50.0)-d2a.level_origin_game_local.x;
    game.player.y=(g1y-75.0)-d2a.level_origin_game_local.y;
    game.player.dir=1;
    game.player.dx=0.0;
    game.player.dy=0.0;
    game.player.jumping=false;
    game.player.on_ground=true;
    input.set(VK_S,true);
    game.tick(&mut input,&ground);
    input.set(VK_S,false);
    let g1_hp_after=game.bank_goons.as_ref().unwrap().goons[1].health;
    assert!(g1_hp_after<g1_hp_before,"standing punch must hit Level 2A goon ({g1_hp_after} vs {g1_hp_before})");

    // Board elevator_02_3480 (next unvisited down elevator after elevator_01_2685 descends from spawn)
    let elev2a=labs2a::ELEVATORS[1];
    let e2a_y0=game.labs_elevators[1].y;
    let (ex0,_,ex1,_)=elev2a.hitbox_bounds_at(e2a_y0);
    game.player.x=((ex0+ex1)*0.5)-d2a.level_origin_game_local.x;
    game.player.y=(e2a_y0-95.0)-d2a.level_origin_game_local.y;
    game.player.dy=0.0;
    game.tick(&mut input,&ground);
    assert!(game.labs_elevators[1].moving,"Level 2A elevator must engage");
    for _ in 0..12 { game.tick(&mut input,&ground); }
    assert!(game.labs_elevators[1].y>e2a_y0,"Level 2A DOWN elevator must descend");
    snapshot(false,out,"l2_2a_gameplay.png",&mut game,&assets);
    snapshot(true,out,"l2_2a_gameplay_hq.png",&mut game,&assets);
    let (dark_2a_mid,fp_2a_mid)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_2a_mid,fp_2a_spawn);
    assert!(dark_2a_mid<0.90);

    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);
    let p95_2a=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

    // 3. checkpoint_2a_tv -> TvCard -> consume_pending_stage_entry -> Level2B
    let tv_cp=labs2a::CHECKPOINTS.iter().find(|c|c.name=="checkpoint_2a_tv").expect("checkpoint_2a_tv");
    let (tx,ty,_,_)=tv_cp.game_bounds();
    game.player.x=(tx+20.0)-d2a.level_origin_game_local.x;
    game.player.y=(ty+20.0)-d2a.level_origin_game_local.y;
    for _ in 0..42 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::TvCard { break; }
    }
    assert_eq!(game.screen,AppScreen::TvCard);
    snapshot(false,out,"l2_tv_000.png",&mut game,&assets);
    let (_,fp_tv0)=intro_frame_metrics(false,&mut game,&assets);
    for _ in 0..110 { game.tick(&mut input,&ground); }
    snapshot(true,out,"l2_tv_110_hq.png",&mut game,&assets);
    let (_,fp_tv110)=intro_frame_metrics(false,&mut game,&assets);
    assert_ne!(fp_tv0,fp_tv110);

    let score_before_2b=game.score;
    let batarangs_before_2b=game.batarangs;
    assert!(game.skip_tv_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    assert!(game.tv_card_complete);
    let next_2b=game.consume_pending_stage_entry().expect("Level2B live handoff");
    assert_eq!(next_2b,StageId::Level2B);
    assert_eq!(game.active_stage(),StageId::Level2B);
    assert_eq!(game.pickups.len(),10);
    assert_eq!(game.score,score_before_2b);
    assert_eq!(game.batarangs,batarangs_before_2b);
    let audio_2b=game.take_audio_events();
    assert!(audio_2b.contains(&AudioEvent::StageTrack(StageId::Level2B)));
    assert!(audio_2b.contains(&AudioEvent::MusicStart));
    ground=CollisionMask::for_stage(next_2b);
    assets.load_stage(next_2b);

    // 4. Level 2B: fadein clear -> clock + halo -> checkpoints 2b_01..03 -> door + UP elevator + pickup + lava1
    for _ in 0..35 { game.tick(&mut input,&ground); }
    snapshot(false,out,"l2_2b_spawn.png",&mut game,&assets);
    snapshot(true,out,"l2_2b_spawn_hq.png",&mut game,&assets);
    let (dark_2b_spawn,fp_2b_spawn)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_2b_spawn<0.95,"Level 2B spawn frame unexpectedly blank");

    let d2b=StageId::Level2B.descriptor();
    for (name,expected_overlay) in [("checkpoint_2b_01",84_usize),("checkpoint_2b_02",93_usize),("checkpoint_2b_03",39_usize)] {
        let cp=labs2b::CHECKPOINTS.iter().find(|c|c.name==name).expect(name);
        let (x,y,_,_)=cp.game_bounds();
        game.player.x=(x+20.0)-d2b.level_origin_game_local.x;
        game.player.y=(y+20.0)-d2b.level_origin_game_local.y;
        game.tick(&mut input,&ground);
        assert_eq!(game.overlay_frame,expected_overlay);
    }

    let door2b=labs2b::DOORS[0];
    game.player.x=door2b.x-d2b.level_origin_game_local.x;
    game.player.y=door2b.y-d2b.level_origin_game_local.y;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert!(game.labs_doors[0].playing || game.labs_doors[0].frame>0);
    assert!(game.take_audio_events().contains(&AudioEvent::DoorOpen));

    let p2b=labs2b::BATARANG_PICKUPS[0];
    game.player.x=p2b.x-d2b.level_origin_game_local.x;
    game.player.y=p2b.y-d2b.level_origin_game_local.y;
    game.player.dy=0.0;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert!(game.pickups[0]);
    assert!(game.take_audio_events().contains(&AudioEvent::Item));

    let lava=labs2b::LAVAS[0];
    let (lx0,ly0,lx1,ly1)=lava.game_bounds();
    game.player.x=((lx0+lx1)*0.5)-d2b.level_origin_game_local.x;
    game.player.y=((ly0+ly1)*0.5)-30.0-d2b.level_origin_game_local.y;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert_eq!(game.health,90,"lava1 must deal 10 damage");
    assert!(game.take_audio_events().contains(&AudioEvent::Punch2));
    assert!(game.clock_secs<80,"Level 2B countdown clock must advance");

    let elev2b=labs2b::ELEVATORS[0];
    let e2b_y0=game.labs_elevators[0].y;
    let (bx0,by0,bx1,_)=elev2b.hitbox_bounds_at(e2b_y0);
    game.player.x=((bx0+bx1)*0.5)-d2b.level_origin_game_local.x;
    game.player.y=(by0-15.0)-d2b.level_origin_game_local.y;
    game.player.dy=0.0;
    game.tick(&mut input,&ground);
    assert!(game.labs_elevators[0].moving,"Level 2B elevator must engage");
    for _ in 0..4 { game.tick(&mut input,&ground); }
    assert!(game.labs_elevators[0].y<e2b_y0,"Level 2B UP elevator must ascend");

    snapshot(false,out,"l2_2b_gameplay.png",&mut game,&assets);
    snapshot(true,out,"l2_2b_gameplay_hq.png",&mut game,&assets);
    let (dark_2b_mid,fp_2b_mid)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_2b_mid,fp_2b_spawn);
    assert!(dark_2b_mid<0.95);
    let p95_2b=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

    // 5. checkpoint_2b_end -> halo fire plume -> KabukiCard -> completion
    input.set(VK_SPACE,true);
    game.tick(&mut input,&ground);
    input.set(VK_SPACE,false);
    let cp_end=labs2b::CHECKPOINTS.iter().find(|c|c.name=="checkpoint_2b_end").expect("checkpoint_2b_end");
    let (xe,ye,_,_)=cp_end.game_bounds();
    game.player.x=(xe+20.0)-d2b.level_origin_game_local.x;
    game.player.y=(ye+20.0)-d2b.level_origin_game_local.y;
    game.player.dx=0.0;
    game.player.dy=0.0;
    let _=game.take_audio_events();
    game.tick(&mut input,&ground);
    assert!(game.level2b_exit_reached);
    assert!(game.halo_fire);
    assert_eq!(game.cam_height(),d2b.camera.height+15.0);
    for _ in 0..4 { game.tick(&mut input,&ground); }
    assert!(game.halo_frame>=24);
    snapshot(true,out,"l2_2b_fire_hq.png",&mut game,&assets);
    let mut exit_audio=Vec::new();
    for _ in 0..40 {
        game.tick(&mut input,&ground);
        exit_audio.extend(game.take_audio_events());
        if game.screen==AppScreen::KabukiCard { break; }
    }
    assert_eq!(game.screen,AppScreen::KabukiCard);
    assert!(exit_audio.contains(&AudioEvent::MusicStop));
    assert!(exit_audio.contains(&AudioEvent::IntroStart));

    snapshot(false,out,"l2_kabuki_000.png",&mut game,&assets);
    let (_,fp_kb0)=intro_frame_metrics(false,&mut game,&assets);
    for _ in 0..135 { game.tick(&mut input,&ground); }
    snapshot(true,out,"l2_kabuki_135_hq.png",&mut game,&assets);
    let (_,fp_kb135)=intro_frame_metrics(false,&mut game,&assets);
    assert_ne!(fp_kb0,fp_kb135,"KabukiCard must animate");

    assert!(game.skip_kabuki_card());
    let mut kb_audio=Vec::new();
    for _ in 0..41 {
        game.tick(&mut input,&ground);
        kb_audio.extend(game.take_audio_events());
    }
    assert!(game.kabuki_card_complete);
    assert!(kb_audio.contains(&AudioEvent::IntroStop));
    assert_eq!(game.pending_stage_entry(),None);

    let report=format!(
        "{{\n  \"milestone\": \"B32\",\n  \"route\": [\"EscapeCard\", \"Level2A\", \"TvCard\", \"Level2B\", \"KabukiCard\", \"Complete\"],\n  \"final_score\": {},\n  \"final_lives\": {},\n  \"final_batarangs\": {},\n  \"final_health_after_lava\": {},\n  \"clock_remaining\": \"{}:{:02}\",\n  \"level2a_goon_kills\": {},\n  \"fingerprints\": {{\n    \"l2_2a_spawn_hq\": \"0x{fp_2a_spawn:016x}\",\n    \"l2_2a_gameplay_hq\": \"0x{fp_2a_mid:016x}\",\n    \"l2_tv_000\": \"0x{fp_tv0:016x}\",\n    \"l2_tv_110\": \"0x{fp_tv110:016x}\",\n    \"l2_2b_spawn_hq\": \"0x{fp_2b_spawn:016x}\",\n    \"l2_2b_gameplay_hq\": \"0x{fp_2b_mid:016x}\",\n    \"l2_kabuki_000\": \"0x{fp_kb0:016x}\",\n    \"l2_kabuki_135\": \"0x{fp_kb135:016x}\"\n  }},\n  \"dark_fraction\": {{\n    \"l2_2a_spawn_hq\": {dark_2a_spawn:.4},\n    \"l2_2a_gameplay_hq\": {dark_2a_mid:.4},\n    \"l2_2b_spawn_hq\": {dark_2b_spawn:.4},\n    \"l2_2b_gameplay_hq\": {dark_2b_mid:.4}\n  }},\n  \"hq_frame_p95_ms\": {{\n    \"level2a\": {p95_2a:.4},\n    \"level2b\": {p95_2b:.4}\n  }},\n  \"keyframes\": [\n    \"l2_2a_spawn.png\", \"l2_2a_spawn_hq.png\", \"l2_2a_gameplay.png\", \"l2_2a_gameplay_hq.png\",\n    \"l2_tv_000.png\", \"l2_tv_110_hq.png\",\n    \"l2_2b_spawn.png\", \"l2_2b_spawn_hq.png\", \"l2_2b_gameplay.png\", \"l2_2b_gameplay_hq.png\", \"l2_2b_fire_hq.png\",\n    \"l2_kabuki_000.png\", \"l2_kabuki_135_hq.png\"\n  ]\n}}\n",
        game.score,game.lives,game.batarangs,game.health,game.clock_secs,game.clock_millisecs,kills_2a
    );
    fs::write(out.join("level2_route_qa_report.json"),report).expect("write B32 Level2 QA report");
}

pub fn qa_full_campaign(out_dir:&str){
    use crate::stage::{labs2a,labs2b,StageId};
    let out=Path::new(out_dir);
    fs::create_dir_all(out).expect("create full campaign QA dir");
    let mut assets=Assets::load();
    let mut ground=CollisionMask::for_stage(StageId::Level1A);
    let mut game=Game::new();
    let mut input=InputState::default();
    let mut base=RenderSurface::new(LOGICAL_W,LOGICAL_H);
    let mut hq=RenderSurface::new(render::HQ_W,render::HQ_H);

    // 1. Title -> Intro -> Level 1A (collect pickup + clear 1A -> Level 1B)
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert_eq!(game.screen,AppScreen::Intro);
    assert!(game.skip_intro());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    for _ in 0..105 { game.tick(&mut input,&ground); }
    assert_eq!(game.screen,AppScreen::Playing);
    assert_eq!(game.active_stage(),StageId::Level1A);
    let (dark_1a,fp_1a)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_1a<0.95,"Level 1A spawn frame unexpectedly blank");

    game.player.x=PICKUPS[0].0-LEVEL_X;
    game.player.y=PICKUPS[0].1-LEVEL_Y;
    game.tick(&mut input,&ground);
    assert!(game.batarangs>0,"Level 1A pickup should grant batarangs");
    game.level1a_exit_reached=true;
    game.fadeout_tick=Some(0);
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let s1b=game.consume_pending_stage_entry().expect("Level1B handoff");
    assert_eq!(s1b,StageId::Level1B);
    ground=CollisionMask::for_stage(s1b);
    assets.load_stage(s1b);
    for _ in 0..45 { game.tick(&mut input,&ground); }

    // 2. Level 1B: verify standing Batarang hits BankBiker + melee hits goon, then exit -> Level 1C
    let d1b=StageId::Level1B.descriptor();
    {
        let biker=game.bank_biker.as_mut().expect("BankBiker spawned in Level 1B");
        biker.active=true;
        biker.x=-1200.0;
        biker.y=375.85;
        biker.dir=1.0;
    }
    game.player.state=crate::game::PlayerState::Stand;
    game.player_invincible_ticks=0;
    game.player.x=-1140.0-d1b.level_origin_game_local.x;
    game.player.y=323.25-d1b.level_origin_game_local.y;
    game.player.dir=-1;
    game.player.jumping=false;
    game.player.dx=0.0;
    game.player.dy=0.0;
    let score_before_biker=game.score;
    input.set(VK_A,true);
    game.tick(&mut input,&ground);
    input.set(VK_A,false);
    for _ in 0..6 { game.tick(&mut input,&ground); }
    let biker_defeated=!game.bank_biker.as_ref().unwrap().alive;
    let biker_score_gain=game.score-score_before_biker;
    assert!(biker_defeated && biker_score_gain>=100,"Standing Batarang must defeat BankBiker on 1B street floor");
    let (dark_1b,fp_1b)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_1b,fp_1a);
    assert!(dark_1b<0.95);
    snapshot(true,out,"campaign_1b_hq.png",&mut game,&assets);

    game.level1b_exit_reached=true;
    game.fadeout_tick=Some(0);
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let s1c=game.consume_pending_stage_entry().expect("Level1C handoff");
    assert_eq!(s1c,StageId::Level1C);
    ground=CollisionMask::for_stage(s1c);
    assets.load_stage(s1c);
    for _ in 0..45 { game.tick(&mut input,&ground); }

    // 3. Level 1C: standing Batarang + melee defeat of Penguin -> EscapeCard
    let d1c=StageId::Level1C.descriptor();
    let bx={
        let boss=game.penguin_boss.as_mut().expect("Penguin boss spawned in Level 1C");
        boss.health=5;
        boss.state=crate::penguin::PenguinState::Stand;
        boss.x
    };
    game.player.state=crate::game::PlayerState::Stand;
    game.player.x=(bx-150.0)-d1c.level_origin_game_local.x;
    game.player.y=288.25-d1c.level_origin_game_local.y;
    game.player.dir=1;
    game.player.jumping=false;
    game.player.on_ground=true;
    game.player.dy=0.0;
    input.set(VK_A,true);
    game.tick(&mut input,&ground);
    input.set(VK_A,false);
    let (dark_1c,fp_1c)=intro_frame_metrics(true,&mut game,&assets);
    assert_ne!(fp_1c,fp_1b);
    assert!(dark_1c<0.95);
    snapshot(true,out,"campaign_1c_hq.png",&mut game,&assets);
    for _ in 0..8 {
        game.tick(&mut input,&ground);
        if game.fadeout_tick.is_some() { break; }
    }
    if !game.level1c_exit_reached {
        let (bx_now,by_now)={
            let boss=game.penguin_boss.as_mut().expect("Penguin boss");
            boss.state=crate::penguin::PenguinState::Stand;
            boss.health=1;
            (boss.x,boss.y)
        };
        game.player.x=(bx_now-20.0)-d1c.level_origin_game_local.x;
        game.player.y=(by_now-28.0)-d1c.level_origin_game_local.y;
        game.player.state=crate::game::PlayerState::Stand;
        game.player.dir=1;
        input.set(VK_S,true);
        game.tick(&mut input,&ground);
        input.set(VK_S,false);
        for _ in 0..45 {
            game.tick(&mut input,&ground);
            if game.level1c_exit_reached { break; }
        }
    }
    assert!(game.level1c_exit_reached,"Defeating Penguin must trigger 1C exit");
    for _ in 0..45 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::EscapeCard { break; }
    }
    assert_eq!(game.screen,AppScreen::EscapeCard);

    // 4. EscapeCard -> Level 2A (elev1 descent + cp01 + door01 + goon kill + cp_2a_tv -> TvCard)
    assert!(game.skip_escape_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let s2a=game.consume_pending_stage_entry().expect("Level2A handoff");
    assert_eq!(s2a,StageId::Level2A);
    ground=CollisionMask::for_stage(s2a);
    assets.load_stage(s2a);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    let (dark_2a,fp_2a)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_2a<0.95);

    let d2a=StageId::Level2A.descriptor();
    let cp01=labs2a::CHECKPOINTS[0];
    let (cx01,cy01,_,_)=cp01.game_bounds();
    game.player.x=(cx01+25.0)-d2a.level_origin_game_local.x;
    game.player.y=(cy01+25.0)-d2a.level_origin_game_local.y;
    game.tick(&mut input,&ground);
    let g2a=game.bank_goons.as_ref().unwrap().goons[0].clone();
    game.player.x=(g2a.x-42.0)-d2a.level_origin_game_local.x;
    game.player.y=(g2a.y-10.0)-d2a.level_origin_game_local.y;
    game.player.dir=1;
    input.set(VK_S,true);
    game.tick(&mut input,&ground);
    input.set(VK_S,false);
    for _ in 0..10 { game.tick(&mut input,&ground); }
    snapshot(true,out,"campaign_2a_hq.png",&mut game,&assets);

    let cp_tv=labs2a::CHECKPOINTS.iter().find(|c|c.name=="checkpoint_2a_tv").expect("checkpoint_2a_tv");
    let (tx,ty,_,_)=cp_tv.game_bounds();
    game.player.x=(tx+20.0)-d2a.level_origin_game_local.x;
    game.player.y=(ty+20.0)-d2a.level_origin_game_local.y;
    for _ in 0..45 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::TvCard { break; }
    }
    assert_eq!(game.screen,AppScreen::TvCard);

    // 5. TvCard -> Level 2B (rising elevator_03 lockstep 45 ticks + lava + cp_2b_end -> KabukiCard)
    assert!(game.skip_tv_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    let s2b=game.consume_pending_stage_entry().expect("Level2B handoff");
    assert_eq!(s2b,StageId::Level2B);
    ground=CollisionMask::for_stage(s2b);
    assets.load_stage(s2b);
    let d2b=StageId::Level2B.descriptor();

    let elev3=labs2b::ELEVATORS[2];
    let e3_y0=game.labs_elevators[2].y;
    let (bx0,by0,bx1,_)=elev3.hitbox_bounds_at(e3_y0);
    game.player.x=((bx0+bx1)*0.5)-d2b.level_origin_game_local.x;
    game.player.y=(by0-15.0)-d2b.level_origin_game_local.y;
    game.player.dy=0.0;
    game.tick(&mut input,&ground);
    assert_eq!(game.active_platform_idx(),Some(2));
    for _ in 0..45 {
        game.tick(&mut input,&ground);
        assert_eq!(game.active_platform_idx(),Some(2),"Level 2B rising elevator_03 dropped Batman mid-ride");
    }
    let e3_ascended=e3_y0-game.labs_elevators[2].y;
    assert!(e3_ascended>=500.0);
    input.set(VK_SPACE,true);
    game.tick(&mut input,&ground);
    input.set(VK_SPACE,false);
    assert_eq!(game.active_platform_idx(),None);

    let (dark_2b,fp_2b)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_2b<0.95);
    snapshot(true,out,"campaign_2b_hq.png",&mut game,&assets);
    let p95_2b=measure_hq_p95_ms(&mut base,&mut hq,&game,&assets);

    let cp_end=labs2b::CHECKPOINTS.iter().find(|c|c.name=="checkpoint_2b_end").expect("checkpoint_2b_end");
    let (ex,ey,_,_)=cp_end.game_bounds();
    game.player.x=(ex+20.0)-d2b.level_origin_game_local.x;
    game.player.y=(ey+20.0)-d2b.level_origin_game_local.y;
    for _ in 0..45 {
        game.tick(&mut input,&ground);
        if game.screen==AppScreen::KabukiCard { break; }
    }
    assert_eq!(game.screen,AppScreen::KabukiCard);

    // 6. KabukiCard -> VK_ENTER back to fresh Title -> Replay Run 2 (Level 1A)
    assert!(game.skip_kabuki_card());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    assert!(game.kabuki_card_complete);
    let final_score=game.score;
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert_eq!(game.screen,AppScreen::Title);
    assert_eq!(game.active_stage(),StageId::Level1A);
    assert_eq!((game.score,game.lives,game.health,game.batarangs),(0,5,100,0));

    ground=CollisionMask::for_stage(StageId::Level1A);
    assets.load_stage(StageId::Level1A);
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert_eq!(game.screen,AppScreen::Intro);
    assert!(game.skip_intro());
    for _ in 0..41 { game.tick(&mut input,&ground); }
    assert_eq!(game.screen,AppScreen::Playing);
    assert_eq!(game.active_stage(),StageId::Level1A);
    assert!(!game.level1a_exit_reached && !game.level2a_exit_reached && !game.level2b_exit_reached);
    for _ in 0..55 { game.tick(&mut input,&ground); }
    let (dark_replay_1a,_)=intro_frame_metrics(true,&mut game,&assets);
    assert!(dark_replay_1a<0.95);
    snapshot(true,out,"campaign_run2_1a_hq.png",&mut game,&assets);

    let report=format!(
        "{{\n  \"route\": [\"Title\", \"Intro\", \"Level1A\", \"Level1B\", \"Level1C\", \"EscapeCard\", \"Level2A\", \"TvCard\", \"Level2B\", \"KabukiCard\", \"Title\", \"Run2:Level1A\"],\n  \"campaign_score_before_reset\": {final_score},\n  \"level1b_biker_defeated\": {biker_defeated},\n  \"level1b_biker_score_gain\": {biker_score_gain},\n  \"level2b_elev3_ascended_px\": {e3_ascended:.2},\n  \"run2_reset_verified\": true,\n  \"hq_p95_level2b_ms\": {p95_2b:.4},\n  \"fingerprints\": {{\n    \"level1a_hq\": \"0x{fp_1a:016x}\",\n    \"level1b_hq\": \"0x{fp_1b:016x}\",\n    \"level1c_hq\": \"0x{fp_1c:016x}\",\n    \"level2a_hq\": \"0x{fp_2a:016x}\",\n    \"level2b_hq\": \"0x{fp_2b:016x}\"\n  }}\n}}\n"
    );
    fs::write(out.join("full_campaign_qa_report.json"),report).expect("write full campaign QA report");
}
