use crate::assets::Assets;
use crate::collision::CollisionMask;
use crate::game::{AppScreen,AudioEvent,Game,InputState,LEVEL_X,LEVEL_Y,LOGICAL_H,LOGICAL_W,PICKUPS,VK_D,VK_ENTER,VK_RIGHT,VK_S,VK_SPACE};
use crate::render;
use png::{BitDepth,ColorType,Encoder};
use std::fs::{self,File};
use std::io::BufWriter;
use std::path::{Path,PathBuf};

pub fn render_smoke(out:&str){
    let out=PathBuf::from(out);
    fs::create_dir_all(&out).expect("create visual-smoke directory");

    let assets=Assets::load();
    let ground=CollisionMask::level1a();
    let mut game=Game::new();
    let mut input=InputState::default();

    // Enter Level 1A without requiring a mouse click.
    input.set(VK_ENTER,true);
    game.tick(&mut input,&ground);
    input.set(VK_ENTER,false);
    assert_eq!(game.screen,AppScreen::Playing);
    assert_eq!(game.take_audio_events(),vec![AudioEvent::MusicStart]);

    // Preserve the real root presentation instead of skipping straight into
    // gameplay: fade-in starts immediately, level-title continues longer.
    snapshot(&out,"00_fade_start.png",&mut game,&assets);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    snapshot(&out,"01_fade_mid.png",&mut game,&assets);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    snapshot(&out,"02_level_title.png",&mut game,&assets);

    // Let the 100-frame title timeline clear, then capture clean gameplay.
    while game.ticks < 105 { game.tick(&mut input,&ground); }
    for _ in 0..180 {
        game.tick(&mut input,&ground);
        if !game.player.jumping { break; }
    }
    snapshot(&out,"03_landed.png",&mut game,&assets);

    // Spawn lands inside checkpoint_1a_walk. Once levelTitle is stopped on
    // Flash frame 100, the original checkpoint repeatedly starts "goRight".
    assert!((1..=9).contains(&game.overlay_frame), "expected goRight tutorial, got {}", game.overlay_frame);
    snapshot(&out,"03_go_right_tutorial.png",&mut game,&assets);

    // Staying in that checkpoint for >3 seconds switches to the original
    // "walk" label exactly like getTimer() logic in the SWF.
    while game.ticks < 180 { game.tick(&mut input,&ground); }
    assert!((58..=65).contains(&game.overlay_frame), "expected walk tutorial, got {}", game.overlay_frame);
    snapshot(&out,"03_walk_tutorial.png",&mut game,&assets);

    // Original acceleration: walk frames should ramp into run rather than
    // teleporting straight to full speed.
    input.set(VK_RIGHT,true);
    for _ in 0..18 { game.tick(&mut input,&ground); }
    snapshot(&out,"04_running.png",&mut game,&assets);
    input.set(VK_RIGHT,false);
    game.tick(&mut input,&ground);

    // Jump and capture the upward pose.
    input.set(VK_SPACE,true);
    game.tick(&mut input,&ground);
    input.set(VK_SPACE,false);
    game.tick(&mut input,&ground);
    assert_eq!(game.take_audio_events(),vec![AudioEvent::Jump]);
    for _ in 0..2 { game.tick(&mut input,&ground); }
    snapshot(&out,"05_jump.png",&mut game,&assets);

    // Hold Space during descent until the original fall->glide rule enters.
    input.set(VK_SPACE,true);
    for _ in 0..80 {
        game.tick(&mut input,&ground);
        if format!("{:?}",game.player.state)=="Glide" { break; }
    }
    snapshot(&out,"06_glide.png",&mut game,&assets);
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
    snapshot(&out,"07_kick.png",&mut game,&assets);

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
    snapshot(&out,"08_held_punch.png",&mut game,&assets);
    input.set(VK_S,false);

    // Force only the recovered tutorial presentation state for a visual
    // checkpoint; gameplay checkpoint logic is separately exercised above.
    game.overlay_frame=140;
    snapshot(&out,"09_to_street.png",&mut game,&assets);

    // Hit the exact recovered Level 1A exit checkpoint and prove the original
    // root fadeout timeline runs to completion without entering Level 1B.
    game.player.x=7600.0-LEVEL_X;
    game.player.y=520.0-LEVEL_Y;
    game.tick(&mut input,&ground);
    assert!(game.level1a_exit_reached);
    assert_eq!(game.fadeout_tick,Some(0));
    snapshot(&out,"10_exit_fade_start.png",&mut game,&assets);
    for _ in 0..20 { game.tick(&mut input,&ground); }
    snapshot(&out,"11_exit_fade_mid.png",&mut game,&assets);
    for _ in 0..25 { game.tick(&mut input,&ground); }
    assert!(game.level1a_complete);
    assert_eq!(game.fadeout_tick,Some(40));
    snapshot(&out,"12_level1a_complete.png",&mut game,&assets);

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

fn snapshot(out:&Path,name:&str,game:&mut Game,assets:&Assets){
    let mut fb=vec![0u32;LOGICAL_W*LOGICAL_H];
    render::render(&mut fb,game,assets);
    write_png(&out.join(name),&fb);
}

fn write_png(path:&Path,fb:&[u32]){
    let file=File::create(path).expect("create PNG");
    let writer=BufWriter::new(file);
    let mut encoder=Encoder::new(writer,LOGICAL_W as u32,LOGICAL_H as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer=encoder.write_header().expect("PNG header");
    let mut bytes=Vec::with_capacity(LOGICAL_W*LOGICAL_H*4);
    for &px in fb {
        bytes.push(((px>>16)&255) as u8);
        bytes.push(((px>>8)&255) as u8);
        bytes.push((px&255) as u8);
        bytes.push(255);
    }
    writer.write_image_data(&bytes).expect("PNG pixels");
}
