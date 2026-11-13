use crate::assets::{Assets,Image,LevelTileSet};
use crate::game::{AppScreen,Game,LOGICAL_H,LOGICAL_W};
use crate::surface::RenderSurface;

pub const HQ_SCALE:usize=3;
pub const HQ_W:usize=LOGICAL_W*HQ_SCALE;
pub const HQ_H:usize=LOGICAL_H*HQ_SCALE;

const fn clamp_isize(val: isize, min: isize, max: isize) -> isize {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

pub const fn compute_xmap_600_to_1800() -> [(usize, usize, u32, u32); HQ_W] {
    let mut map = [(0, 0, 0, 0); HQ_W];
    let mut dx = 0;
    while dx < HQ_W {
        let xn = dx as isize - 1;
        let xq = xn.div_euclid(3);
        let xr = xn.rem_euclid(3) as u32;
        let x0 = clamp_isize(xq, 0, LOGICAL_W as isize - 1) as usize;
        let x1 = clamp_isize(xq + 1, 0, LOGICAL_W as isize - 1) as usize;
        map[dx] = (x0, x1, 3 - xr, xr);
        dx += 1;
    }
    map
}

pub const XMAP_600_TO_1800: [(usize, usize, u32, u32); HQ_W] = compute_xmap_600_to_1800();

#[inline]
pub fn format_u32_digits(mut val: u32, buf: &mut [u8; 16]) -> &[u8] {
    if val == 0 {
        buf[0] = b'0';
        return &buf[..1];
    }
    let mut len = 0;
    let mut temp = val;
    while temp > 0 {
        len += 1;
        temp /= 10;
    }
    let mut pos = len;
    while val > 0 {
        pos -= 1;
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
    }
    &buf[..len]
}

#[inline]
pub fn digit_count(mut n: i32) -> usize {
    if n <= 0 {
        return 1;
    }
    let mut count = 0;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

// Generated from one offline HUD manifest, never parsed at game runtime.
mod hud_hq_layout { include!("../assets/hud_hq_layout.rs"); }

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum RenderMode{Legacy1x,HighQuality}

#[derive(Copy,Clone,Debug)]
pub struct RenderConfig{pub logical_w:usize,pub logical_h:usize,pub hq_scale:usize,pub mode:RenderMode}
impl RenderConfig{
    pub const fn legacy()->Self{Self{logical_w:LOGICAL_W,logical_h:LOGICAL_H,hq_scale:1,mode:RenderMode::Legacy1x}}
    pub const fn high_quality()->Self{Self{logical_w:LOGICAL_W,logical_h:LOGICAL_H,hq_scale:HQ_SCALE,mode:RenderMode::HighQuality}}
}


pub fn render(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_with_config(fb,None,game,assets,RenderConfig::legacy());
}

pub fn render_high_quality(base:&mut RenderSurface,hq:&mut RenderSurface,game:&Game,assets:&Assets){
    render_with_config(hq,Some(base),game,assets,RenderConfig::high_quality());
}

pub fn render_with_config(
    target:&mut RenderSurface,
    scratch:Option<&mut RenderSurface>,
    game:&Game,
    assets:&Assets,
    config:RenderConfig,
){
    assert_eq!(config.logical_w,LOGICAL_W);
    assert_eq!(config.logical_h,LOGICAL_H);
    match config.mode{
        RenderMode::Legacy1x=>{
            assert_eq!(config.hq_scale,1);
            assert_eq!(target.w,LOGICAL_W);
            assert_eq!(target.h,LOGICAL_H);
            render_legacy(target,game,assets);
        }
        RenderMode::HighQuality=>{
            assert_eq!(config.hq_scale,HQ_SCALE);
            assert_eq!(target.w,HQ_W);
            assert_eq!(target.h,HQ_H);
            let base=scratch.expect("HQ rendering requires a logical-resolution scratch surface");
            assert_eq!(base.w,LOGICAL_W);
            assert_eq!(base.h,LOGICAL_H);

            if game.screen!=AppScreen::Playing{
                // Title/instructions still use the exact Round-3 path.
                let active_fade=match game.screen{
                    AppScreen::Intro=>game.intro_fadeout_tick,
                    AppScreen::EscapeCard=>game.escape_card_fadeout_tick,
                    AppScreen::TvCard=>game.tv_card_fadeout_tick,
                    _=>None,
                };
                if let Some(t)=active_fade{
                    // Do not double-composite a low-res fade. Rust draws the
                    // existing original sprite34 asset at genuine 3x.
                    render_legacy_intro_overlay(base,game,assets,false);
                    upscale_opaque_base(base,target);
                    draw_root_timeline_hq_lazy(target,&assets.fadeout_frames_hq,t as u64,301.0,205.0);
                }else{
                    render_legacy(base,game,assets);
                    upscale_opaque_base(base,target);
                }
            }else{
                // Round 8 preserves the original depth split while removing
                // the last small moving objects from the blurry 1x world:
                // scenery at 1x -> promote -> true 3x pickups/Batarangs ->
                // true 3x Batman -> true 3x foreground UI/overlays.
                // Round 11 lifts the root sky/moon out of the blurry
                // whole-frame promotion while preserving their original depth:
                // true-3x sky -> true-3x moon -> legacy city/base scenery
                // alpha-promoted to 3x -> true-3x Round-10 scenery detail.
                //
                // The 600x400 scratch surface becomes a transparent midground
                // only in HQ play. Legacy rendering still uses the historical
                // opaque composition path unchanged.
                draw_root_backdrop_hq(target,assets);
                base.fill(0x00000000);
                if assets.stage.stage_id == crate::stage::StageId::Level1A {
                    draw_city_background_alpha(base,game,assets);
                    draw_tiles_alpha(base,game,&assets.level1a_tiles_hq_base);
                    upscale_alpha_layer(base,target);
                    draw_tiles_hq(target,game,&assets.level1a_detail_tiles_hq);
                } else {
                    draw_city_background_alpha(base,game,assets);
                    draw_tiles_alpha(base,game,&assets.stage.tiles);
                    upscale_alpha_layer(base,target);
                }
                draw_world_objects_hq(target,game,assets);
                draw_stage_enemies_hq(target,game,assets);
                draw_batman_hq(target,game,assets);
                draw_world_foreground_hq(target,game,assets);
                draw_foreground_hq(target,game,assets);
            }
        }
    }
}

pub fn upscale_opaque_base(src:&RenderSurface,dst:&mut RenderSurface){
    assert_eq!(dst.w,src.w*HQ_SCALE);
    assert_eq!(dst.h,src.h*HQ_SCALE);
    assert_eq!(HQ_SCALE,3,"Round 3 bilinear kernel is specialized for 3x");
    assert_eq!(src.w,LOGICAL_W);
    assert_eq!(src.h,LOGICAL_H);
    assert_eq!(dst.w,HQ_W);
    assert_eq!(dst.h,HQ_H);

    // Pixel-center bilinear resampling. For an exact 3x scale, destination
    // pixel (3*x+1,3*y+1) lands exactly on source pixel (x,y), which gives us
    // a strong no-drift CI invariant while smoothing the two pixels between
    // neighboring logical samples.
    //
    // Uses compile-time precomputed XMAP_600_TO_1800 to eliminate per-frame
    // heap allocations and integer divisions from the inner loop.
    for dy in 0..dst.h{
        let yn=dy as isize-1;
        let yq=yn.div_euclid(3);
        let yr=yn.rem_euclid(3) as u32;
        let y0=yq.clamp(0,src.h as isize-1) as usize;
        let y1=(yq+1).clamp(0,src.h as isize-1) as usize;
        let wy0=3-yr;
        let wy1=yr;
        let row0=y0*src.w;
        let row1=y1*src.w;
        let dst_row=dy*dst.w;

        for (dx,&(x0,x1,wx0,wx1)) in XMAP_600_TO_1800.iter().enumerate(){
            let p00=src.pixels[row0+x0];
            let p10=src.pixels[row0+x1];
            let p01=src.pixels[row1+x0];
            let p11=src.pixels[row1+x1];

            let w00=wx0*wy0;
            let w10=wx1*wy0;
            let w01=wx0*wy1;
            let w11=wx1*wy1;

            let r=((((p00>>16)&255)*w00+((p10>>16)&255)*w10+((p01>>16)&255)*w01+((p11>>16)&255)*w11+4)/9)&255;
            let g=((((p00>>8)&255)*w00+((p10>>8)&255)*w10+((p01>>8)&255)*w01+((p11>>8)&255)*w11+4)/9)&255;
            let b=((p00&255)*w00+(p10&255)*w10+(p01&255)*w01+(p11&255)*w11+4)/9;

            dst.pixels[dst_row+dx]=(r<<16)|(g<<8)|b;
        }
    }
}

fn upscale_alpha_layer(src:&RenderSurface,dst:&mut RenderSurface){
    assert_eq!(dst.w,src.w*HQ_SCALE);
    assert_eq!(dst.h,src.h*HQ_SCALE);
    assert_eq!(HQ_SCALE,3,"Round 11 alpha promotion is specialized for 3x");
    assert_eq!(src.w,LOGICAL_W);
    assert_eq!(src.h,LOGICAL_H);
    assert_eq!(dst.w,HQ_W);
    assert_eq!(dst.h,HQ_H);

    for dy in 0..dst.h{
        let yn=dy as isize-1;
        let yq=yn.div_euclid(3);
        let yr=yn.rem_euclid(3) as u32;
        let y0=yq.clamp(0,src.h as isize-1) as usize;
        let y1=(yq+1).clamp(0,src.h as isize-1) as usize;
        let wy0=3-yr;
        let wy1=yr;
        let row0=y0*src.w;
        let row1=y1*src.w;
        let dst_row=dy*dst.w;

        for (dx,&(x0,x1,wx0,wx1)) in XMAP_600_TO_1800.iter().enumerate(){
            blend_bilinear_premul_3x(
                &mut dst.pixels[dst_row+dx],
                src.pixels[row0+x0],
                src.pixels[row0+x1],
                src.pixels[row1+x0],
                src.pixels[row1+x1],
                wx0*wy0,wx1*wy0,wx0*wy1,wx1*wy1
            );
        }
    }
}

fn render_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_legacy_intro_overlay(fb,game,assets,true);
}

fn render_story_timeline_legacy(
    fb:&mut RenderSurface,
    assets:&Assets,
    frames:&crate::assets::LazySpriteSet,
    tick:u16,
    fadeout_tick:Option<u16>,
    show_fade:bool,
){
    let frame=(tick as usize).min(frames.len()-1);
    let story=frames.frame(frame);
    blit(
        fb,&story.image,
        -story.anchor_x.round() as i32,-story.anchor_y.round() as i32,
        story.image.w as i32,story.image.h as i32,false
    );
    if show_fade {
        if let Some(t)=fadeout_tick{
            draw_root_timeline_lazy_legacy(fb,&assets.fadeout_frames,t as u64,301.0,205.0);
        }
    }
}

fn render_legacy_intro_overlay(fb:&mut RenderSurface,game:&Game,assets:&Assets,show_fade:bool){
    fb.fill(0xff000000);
    match game.screen{
        AppScreen::Title=>{blit(fb,&assets.title_screen,0,0,600,400,false);return},
        AppScreen::Instructions=>{blit(fb,&assets.instructions_screen,0,0,600,400,false);return},
        AppScreen::Intro=>{
            render_story_timeline_legacy(
                fb,assets,&assets.intro_frames,game.intro_tick,game.intro_fadeout_tick,show_fade,
            );
            return
        },
        AppScreen::EscapeCard=>{
            render_story_timeline_legacy(
                fb,assets,&assets.escape_card_frames,game.escape_card_tick,game.escape_card_fadeout_tick,show_fade,
            );
            return
        },
        AppScreen::TvCard=>{
            render_story_timeline_legacy(
                fb,assets,&assets.tv_card_frames,game.tv_card_tick,game.tv_card_fadeout_tick,show_fade,
            );
            return
        },
        AppScreen::Playing=>{}
    }
    render_world_legacy(fb,game,assets);
    draw_batman_legacy(fb,game,assets);
    draw_world_foreground_legacy(fb,game,assets);
    draw_foreground_legacy(fb,game,assets);
}

fn render_world_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_world_base_legacy(fb,game,assets);
    draw_world_objects_legacy(fb,game,assets);
    draw_stage_enemies_legacy(fb,game,assets);
}

fn render_world_base_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_backdrop_legacy(fb,game,assets);
    draw_tiles(fb,game,&assets.stage.tiles);
}

fn render_backdrop_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_root_backdrop_legacy(fb,assets);
    draw_city_background_legacy(fb,game,assets);
}

fn draw_root_backdrop_legacy(fb:&mut RenderSurface,assets:&Assets){
    if matches!(assets.stage.stage_id, crate::stage::StageId::Level2A | crate::stage::StageId::Level2B) {
        // Root frames 23/25 place symbol #14 at depth 1 (solid black lab backdrop) with no moon.
        fb.fill(0xff000000);
        return;
    }
    // Original root gameplay backdrop, frame 19:
    // depth 1 = symbol 131 at identity; depth 2 = symbol 134 at (300,130).
    blit(
        fb,&assets.root_sky,
        -1,-1,
        assets.root_sky.w as i32,assets.root_sky.h as i32,false
    );
    const MOON_SCALE:f32=0.76934814453125;
    const MOON_ANCHOR:f32=195.95;
    let moon_w=(assets.root_moon.w as f32*MOON_SCALE).round() as i32;
    let moon_h=(assets.root_moon.h as f32*MOON_SCALE).round() as i32;
    let moon_x=(300.0-MOON_ANCHOR*MOON_SCALE).round() as i32;
    let moon_y=(130.0-MOON_ANCHOR*MOON_SCALE).round() as i32;
    blit(fb,&assets.root_moon,moon_x,moon_y,moon_w,moon_h,false);
}

fn draw_root_backdrop_hq(fb:&mut RenderSurface,assets:&Assets){
    if matches!(assets.stage.stage_id, crate::stage::StageId::Level2A | crate::stage::StageId::Level2B) {
        // Root frames 23/25 place symbol #14 at depth 1 (solid black lab backdrop) with no moon.
        fb.fill(0xff000000);
        return;
    }
    // Symbol 131 is exactly one opaque RGB color across the visible stage:
    // (161,22,0). Its transparent pixels are only native-bounds bake padding.
    // Filling the presentation surface with that recovered source color is
    // pixel-faithful at every scale and avoids carrying/blitting an 8+ MiB
    // redundant 3x texture.
    fb.fill(0xffa11600);

    debug_assert_eq!(assets.root_moon_hq.len(),1);
    debug_assert!((assets.root_moon_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);

    let moon=&assets.root_moon_hq[0];
    let s=HQ_SCALE as f32;
    blit(
        fb,&moon.image,
        (300.0*s-moon.anchor_x).round() as i32,
        (130.0*s-moon.anchor_y).round() as i32,
        moon.image.w as i32,moon.image.h as i32,false
    );
}

fn city_geometry(game:&Game,assets:&Assets)->(i32,i32,i32,i32){
    const BG_SX:f32=1.029632568359375;
    const BG_SY:f32=1.029754638671875;
    const BG_ORIGIN_X:f32=-0.98048095703125;
    const BG_ORIGIN_Y:f32=66.1;
    const BG_TX:f32=0.0;
    const BG_TY:f32=-13.1;
    let bg_x=(game.camera_x+game.background_x+BG_TX+BG_ORIGIN_X*BG_SX).round() as i32;
    let bg_y=(game.camera_y+game.background_y+BG_TY+BG_ORIGIN_Y*BG_SY).round() as i32;
    let bg_w=(assets.city_background.w as f32*BG_SX).round() as i32;
    let bg_h=(assets.city_background.h as f32*BG_SY).round() as i32;
    (bg_x,bg_y,bg_w,bg_h)
}

fn draw_city_background_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    if matches!(assets.stage.stage_id, crate::stage::StageId::Level2A | crate::stage::StageId::Level2B) {
        return;
    }
    let(bg_x,bg_y,bg_w,bg_h)=city_geometry(game,assets);
    blit(fb,&assets.city_background,bg_x,bg_y,bg_w,bg_h,false);
}

fn draw_city_background_alpha(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    if matches!(assets.stage.stage_id, crate::stage::StageId::Level2A | crate::stage::StageId::Level2B) {
        return;
    }
    let(bg_x,bg_y,bg_w,bg_h)=city_geometry(game,assets);
    blit_alpha(fb,&assets.city_background,bg_x,bg_y,bg_w,bg_h,false);
}

fn draw_world_objects_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_labs_doors_scaled(fb,game,assets,1);
    draw_labs_elevators_scaled(fb,game,assets,1);
    let pickup_coords: &[(f32,f32)] = match game.active_stage() {
        crate::stage::StageId::Level1A => &crate::game::PICKUPS,
        crate::stage::StageId::Level1B => &crate::game::LEVEL1B_PICKUPS,
        crate::stage::StageId::Level1C => &crate::penguin::LEVEL1C_PICKUP_POINTS,
        _ => &[],
    };
    if !assets.pickup_frames.is_empty() {
        let pf=&assets.pickup_frames[(game.ticks as usize)%assets.pickup_frames.len()];
        for (i,&(wx,wy)) in pickup_coords.iter().enumerate() {
            if i < game.pickups.len() && game.pickups[i] { continue; }
            let px=(wx+game.camera_x-pf.anchor_x).round() as i32;
            let py=(wy+game.camera_y-pf.anchor_y).round() as i32;
            blit(fb,&pf.image,px,py,pf.image.w as i32,pf.image.h as i32,false);
        }
    }

    if !assets.batarang_frames.is_empty() {
        let bf=&assets.batarang_frames[(game.ticks as usize)%assets.batarang_frames.len()];
        for shot in &game.shots {
            let px=(shot.x+game.camera_x-bf.anchor_x).round() as i32;
            let py=(shot.y+game.camera_y-bf.anchor_y).round() as i32;
            blit(fb,&bf.image,px,py,bf.image.w as i32,bf.image.h as i32,shot.dir<0);
        }
    }
}

fn draw_world_objects_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_labs_doors_scaled(fb,game,assets,HQ_SCALE as i32);
    draw_labs_elevators_scaled(fb,game,assets,HQ_SCALE as i32);
    let scale=HQ_SCALE as f32;
    debug_assert!((assets.pickup_frames_hq.logical_pixel_scale-scale).abs()<0.001);
    debug_assert!((assets.batarang_frames_hq.logical_pixel_scale-scale).abs()<0.001);

    let pickup_coords: &[(f32,f32)] = match game.active_stage() {
        crate::stage::StageId::Level1A => &crate::game::PICKUPS,
        crate::stage::StageId::Level1B => &crate::game::LEVEL1B_PICKUPS,
        crate::stage::StageId::Level1C => &crate::penguin::LEVEL1C_PICKUP_POINTS,
        _ => &[],
    };

    if !assets.pickup_frames_hq.is_empty() {
        let pf=&assets.pickup_frames_hq[(game.ticks as usize)%assets.pickup_frames_hq.len()];
        for (i,&(wx,wy)) in pickup_coords.iter().enumerate() {
            if i < game.pickups.len() && game.pickups[i] { continue; }
            // Lock the stage origin to the exact same logical integer pixel
            // used by the legacy renderer, then convert that origin to HQ.
            // This avoids inventing sub-pixel world placement in Round 8.
            let logical_x=(wx+game.camera_x).round() as i32;
            let logical_y=(wy+game.camera_y).round() as i32;
            let px=logical_x*HQ_SCALE as i32-pf.anchor_x.round() as i32;
            let py=logical_y*HQ_SCALE as i32-pf.anchor_y.round() as i32;
            blit(fb,&pf.image,px,py,pf.image.w as i32,pf.image.h as i32,false);
        }
    }

    if !assets.batarang_frames_hq.is_empty() {
        let bf=&assets.batarang_frames_hq[(game.ticks as usize)%assets.batarang_frames_hq.len()];
        for shot in &game.shots {
            let logical_x=(shot.x+game.camera_x).round() as i32;
            let logical_y=(shot.y+game.camera_y).round() as i32;
            let px=logical_x*HQ_SCALE as i32-bf.anchor_x.round() as i32;
            let py=logical_y*HQ_SCALE as i32-bf.anchor_y.round() as i32;
            blit(fb,&bf.image,px,py,bf.image.w as i32,bf.image.h as i32,shot.dir<0);
        }
    }
}

fn draw_batman_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    let sx=(game.player.x+game.level_origin_x()+game.camera_x).round() as i32;
    let sy=(game.player.y+game.level_origin_y()+game.camera_y).round() as i32;
    let frame=&assets.batman_frames[game.animation_frame()];
    let flip=game.player.dir<0;
    let ax=if flip{frame.image.w as f32-frame.anchor_x}else{frame.anchor_x};
    blit(
        fb,&frame.image,
        sx-ax.round() as i32,sy-frame.anchor_y.round() as i32,
        frame.image.w as i32,frame.image.h as i32,flip
    );
}

fn draw_batman_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    debug_assert!((assets.batman_frames_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    let sx=(game.player.x+game.level_origin_x()+game.camera_x).round() as i32*HQ_SCALE as i32;
    let sy=(game.player.y+game.level_origin_y()+game.camera_y).round() as i32*HQ_SCALE as i32;
    let frame=&assets.batman_frames_hq[game.animation_frame()];
    let flip=game.player.dir<0;
    let ax=if flip{frame.image.w as f32-frame.anchor_x}else{frame.anchor_x};
    // True HQ frames are already at final presentation resolution: 1:1 copy,
    // never nearest-neighbor enlargement at runtime.
    blit(
        fb,&frame.image,
        sx-ax.round() as i32,sy-frame.anchor_y.round() as i32,
        frame.image.w as i32,frame.image.h as i32,flip
    );
}

#[inline]
fn draw_lazy_world_sprite(
    fb:&mut RenderSurface,
    frame:&crate::assets::LazySpriteFrame,
    world_x:f32,world_y:f32,
    camera_x:f32,camera_y:f32,
    flip:bool,
    pixel_scale:i32,
){
    let sx=(world_x+camera_x).round() as i32*pixel_scale;
    let sy=(world_y+camera_y).round() as i32*pixel_scale;
    let ax=if flip{frame.image.w as f32-frame.anchor_x}else{frame.anchor_x};
    let dw=frame.image.w as i32*pixel_scale;
    let dh=frame.image.h as i32*pixel_scale;
    let px=sx-(ax*pixel_scale as f32).round() as i32;
    let py=sy-(frame.anchor_y*pixel_scale as f32).round() as i32;
    if px+dw<=0 || px>=fb.w as i32 || py+dh<=0 || py>=fb.h as i32 {
        return;
    }
    blit(fb,&frame.image,px,py,dw,dh,flip);
}

fn draw_labs_doors_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    if game.active_stage()!=crate::stage::StageId::Level2A || assets.door_frames.len()<20 {
        return;
    }
    let placements=&crate::stage::labs2a::DOORS;
    for (i,&door) in placements.iter().enumerate(){
        let frame_idx=game.labs_doors.get(i).map_or(0,|s|(s.frame as usize).min(19));
        draw_lazy_world_sprite(
            fb,&assets.door_frames.frame(frame_idx),
            door.x,door.y,game.camera_x,game.camera_y,door.sx<0.0,pixel_scale,
        );
    }
}

fn draw_labs_elevators_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    if game.active_stage()!=crate::stage::StageId::Level2A || assets.elevator_frames.len()<2 {
        return;
    }
    let placements=&crate::stage::labs2a::ELEVATORS;
    for (i,&elev) in placements.iter().enumerate(){
        let y=game.labs_elevators.get(i).map_or(elev.y,|s|s.y);
        draw_lazy_world_sprite(
            fb,&assets.elevator_frames.frame(elev.pack_frame()),
            elev.x,y,game.camera_x,game.camera_y,elev.sx<0.0,pixel_scale,
        );
    }
}

fn draw_world_foreground_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    if game.active_stage()!=crate::stage::StageId::Level2A || assets.door_frames.len()<23 {
        return;
    }
    for &jamb in &crate::stage::labs2a::DOOR_JAMBS{
        if let Some(frame_idx)=jamb.jamb_pack_frame(){
            draw_lazy_world_sprite(
                fb,&assets.door_frames.frame(frame_idx),
                jamb.x,jamb.y,game.camera_x,game.camera_y,jamb.sx<0.0,pixel_scale,
            );
        }
    }
    // Flash depth 413 (`halo` #1399) follows Batman (`followPlayer`), sitting above front
    // door jambs (377..411) and underneath glinting `health01` (415) & `item_br*` (425..617).
    if assets.labs_pickups_frames.len() >= 2 {
        let player_gx = game.player.x + game.level_origin_x();
        let player_gy = game.player.y + game.level_origin_y();
        draw_lazy_world_sprite(
            fb,
            &assets.labs_pickups_frames.frame(1),
            player_gx,
            player_gy,
            game.camera_x,
            game.camera_y,
            false,
            pixel_scale,
        );
        if !game.labs_health_collected {
            let h = crate::stage::labs2a::HEALTH_PICKUPS[0];
            draw_lazy_world_sprite(
                fb,
                &assets.labs_pickups_frames.frame(0),
                h.x,
                h.y,
                game.camera_x,
                game.camera_y,
                false,
                pixel_scale,
            );
        }
    }
    if pixel_scale == HQ_SCALE as i32 && !assets.pickup_frames_hq.is_empty() {
        let pf = &assets.pickup_frames_hq[(game.ticks as usize) % assets.pickup_frames_hq.len()];
        for (i, &p) in crate::stage::labs2a::BATARANG_PICKUPS.iter().enumerate() {
            if i < game.pickups.len() && game.pickups[i] { continue; }
            let logical_x = (p.x + game.camera_x).round() as i32;
            let logical_y = (p.y + game.camera_y).round() as i32;
            let px = logical_x * pixel_scale - pf.anchor_x.round() as i32;
            let py = logical_y * pixel_scale - pf.anchor_y.round() as i32;
            blit(fb, &pf.image, px, py, pf.image.w as i32, pf.image.h as i32, false);
        }
    } else if !assets.pickup_frames.is_empty() {
        let pf = &assets.pickup_frames[(game.ticks as usize) % assets.pickup_frames.len()];
        let dw = pf.image.w as i32 * pixel_scale;
        let dh = pf.image.h as i32 * pixel_scale;
        for (i, &p) in crate::stage::labs2a::BATARANG_PICKUPS.iter().enumerate() {
            if i < game.pickups.len() && game.pickups[i] { continue; }
            let sx = (p.x + game.camera_x).round() as i32 * pixel_scale;
            let sy = (p.y + game.camera_y).round() as i32 * pixel_scale;
            let px = sx - (pf.anchor_x * pixel_scale as f32).round() as i32;
            let py = sy - (pf.anchor_y * pixel_scale as f32).round() as i32;
            blit(fb, &pf.image, px, py, dw, dh, false);
        }
    }
}

fn draw_world_foreground_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_world_foreground_scaled(fb,game,assets,1);
}

fn draw_world_foreground_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_world_foreground_scaled(fb,game,assets,HQ_SCALE as i32);
}

fn draw_stage_enemies_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    match game.active_stage() {
        crate::stage::StageId::Level1B | crate::stage::StageId::Level2A => {
            if let Some(squad) = &game.bank_goons {
                for goon in &squad.goons {
                    if !goon.active || !goon.visible { continue; }
                    let frames = match goon.family {
                        crate::bank_goons::GoonFamily::A => &assets.goon_a_frames,
                        crate::bank_goons::GoonFamily::B => &assets.goon_b_frames,
                        crate::bank_goons::GoonFamily::LabsC => &assets.labs_goon_frames,
                    };
                    if frames.is_empty() { continue; }
                    let clip = goon.family.child_clip(goon.state);
                    let rel = (goon.frame as usize).min(clip.frame_count().saturating_sub(1) as usize);
                    let idx = (clip.first_frame_inclusive as usize + rel).min(frames.len().saturating_sub(1));
                    draw_lazy_world_sprite(
                        fb,&frames.frame(idx),goon.x,goon.y,game.camera_x,game.camera_y,
                        goon.dir < 0,pixel_scale
                    );
                }
            }
            if let Some(biker) = &game.bank_biker {
                if biker.active && biker.visible && !assets.biker_frames.is_empty() {
                    let idx = match biker.state {
                        crate::bank_biker::BikerState::Drive => (biker.frame as usize) % 2,
                        crate::bank_biker::BikerState::Fall => (2 + biker.frame as usize).min(assets.biker_frames.len().saturating_sub(1)),
                    };
                    draw_lazy_world_sprite(
                        fb,&assets.biker_frames.frame(idx),biker.x,biker.y,game.camera_x,game.camera_y,
                        biker.dir < 0.0,pixel_scale
                    );
                }
            }
            if !assets.eagle_frames.is_empty() {
                for eagle in &game.labs_eagles {
                    let idx = eagle.pack_frame_index().min(assets.eagle_frames.len().saturating_sub(1));
                    draw_lazy_world_sprite(
                        fb,&assets.eagle_frames.frame(idx),eagle.x,eagle.y,game.camera_x,game.camera_y,
                        eagle.dir < 0,pixel_scale
                    );
                }
            }
        }
        crate::stage::StageId::Level1C => {
            if let Some(boss) = &game.penguin_boss {
                if boss.active && !assets.penguin_frames.is_empty() {
                    let clip = crate::penguin::penguin_clip_for_state(boss.state);
                    let rel = (boss.child_frame as usize).min(clip.child_frames.saturating_sub(1) as usize);
                    let idx = (clip.baked_start_frame as usize + rel).min(assets.penguin_frames.len().saturating_sub(1));
                    // Unflipped #1250 artwork faces left (-x), so flip horizontally when dir >= 0.
                    draw_lazy_world_sprite(
                        fb,&assets.penguin_frames.frame(idx),boss.x,boss.y,game.camera_x,game.camera_y,
                        boss.dir >= 0,pixel_scale
                    );
                }
            }
        }
        _ => {}
    }
}

fn draw_stage_enemies_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_stage_enemies_scaled(fb,game,assets,1);
}

fn draw_stage_enemies_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_stage_enemies_scaled(fb,game,assets,HQ_SCALE as i32);
}

fn draw_foreground_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_hud(fb,game,assets);

    if !assets.tutorial_overlay_frames.is_empty(){
        let i=game.overlay_frame.min(assets.tutorial_overlay_frames.len()-1);
        let f=assets.tutorial_overlay_frames.frame(i);
        blit(
            fb,&f.image,
            (311.4-f.anchor_x).round() as i32,
            (190.0-f.anchor_y).round() as i32,
            f.image.w as i32,f.image.h as i32,false
        );
    }

    match assets.stage.stage_id {
        crate::stage::StageId::Level1A => {
            draw_root_timeline_lazy_scaled(fb,&assets.level_title_frames,game.ticks,318.7,76.75,1);
        }
        crate::stage::StageId::Level2A => {
            draw_root_timeline_lazy_scaled(fb,&assets.level2a_title_frames,game.ticks,318.7,76.75,1);
        }
        _ => {}
    }
    draw_root_timeline_lazy_legacy(fb,&assets.fadein_frames,game.ticks,301.0,205.0);
    if let Some(t)=game.fadeout_tick{
        draw_root_timeline_lazy_legacy(fb,&assets.fadeout_frames,t as u64,301.0,205.0);
    }
}

fn draw_foreground_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_hud_hq(fb,game,assets);

    // Round 7 foreground timelines are already true 3x assets. Their SWF
    // display lists, embedded DefineText glyphs, masks and CXFORMWITHALPHA were
    // resolved at 6x before premultiplied-alpha Lanczos downsampling to 3x.
    if !assets.tutorial_overlay_frames_hq.is_empty(){
        debug_assert!((assets.tutorial_overlay_frames_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
        let i=game.overlay_frame.min(assets.tutorial_overlay_frames_hq.len()-1);
        let f=assets.tutorial_overlay_frames_hq.frame(i);
        let scale=assets.tutorial_overlay_frames_hq.logical_pixel_scale;
        blit(
            fb,&f.image,
            (311.4*scale-f.anchor_x).round() as i32,
            (190.0*scale-f.anchor_y).round() as i32,
            f.image.w as i32,f.image.h as i32,false
        );
    }

    match assets.stage.stage_id {
        crate::stage::StageId::Level1A => {
            draw_root_timeline_hq_lazy(fb,&assets.level_title_frames_hq,game.ticks,318.7,76.75);
        }
        crate::stage::StageId::Level2A => {
            draw_root_timeline_lazy_scaled(fb,&assets.level2a_title_frames,game.ticks,318.7,76.75,HQ_SCALE as i32);
        }
        _ => {}
    }
    draw_root_timeline_hq_lazy(fb,&assets.fadein_frames_hq,game.ticks,301.0,205.0);
    if let Some(t)=game.fadeout_tick{
        draw_root_timeline_hq_lazy(fb,&assets.fadeout_frames_hq,t as u64,301.0,205.0);
    }
}

fn draw_hud_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    const ROOT_X:f32=159.0;
    const ROOT_Y:f32=-11.0;
    debug_assert!((assets.hud_shell_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    debug_assert_eq!(assets.hud_shell_hq.len(),1);
    let shell=&assets.hud_shell_hq[0];
    let scale=assets.hud_shell_hq.logical_pixel_scale;
    let shell_x=(ROOT_X*scale-shell.anchor_x).round() as i32;
    let shell_y=(ROOT_Y*scale-shell.anchor_y).round() as i32;

    // Original sprite 716: icon art and rounded translucent backing remain.
    // The unwanted separate full-width counter band must never return.
    blit(fb,&shell.image,shell_x,shell_y,
         shell.image.w as i32,shell.image.h as i32,false);

    // Themed labels and dynamic counters share one stage-space layout; 
    // source-authentic 1x Flash HUD and game state remain unchanged.
    let s=HQ_SCALE as i32;
    let (bx,by)=hud_hq_layout::BATARANG_CENTER;
    draw_number_centered_hq_1to1(
        fb,&assets.hud_digits_batarang_hq,bx*s,by*s,game.batarangs.max(0),
        hud_hq_layout::BATARANG_MAX_WIDTH*s,
        hud_hq_layout::BATARANG_DIGIT_ADVANCE*s
    );

    let score=game.score.max(0);
    let (sx,sy)=hud_hq_layout::SCORE_CENTER;
    let score_budget=hud_hq_layout::SCORE_MAX_WIDTH*s;
    let large_cell=assets.hud_digits_large_hq.w/10;
    // Preserve true-3x pixels: use the already-baked small atlas when a
    // multi-digit score cannot fit the large glyphs into its column.
    let use_small_score=large_cell*digit_count(score)>score_budget as usize;
    let score_atlas=if use_small_score{
        &assets.hud_digits_small_hq
    }else{
        &assets.hud_digits_large_hq
    };
    // HUD-REAL-06C: compact transparent padding between the large SCORE
    // glyphs without rescaling the pixels or moving the source shell/labels.
    // Preserve the verified natural-step compact fallback for long scores.
    let score_advance=if use_small_score{
        0
    }else{
        hud_hq_layout::SCORE_LARGE_DIGIT_ADVANCE*s
    };
    draw_number_centered_hq_1to1(
        fb,score_atlas,sx*s,sy*s,score,score_budget,score_advance
    );

    // LIVES X is baked directly before this fixed value-left edge.
    // Widening numbers extend rightward, never back over the caption.
    draw_number_left_hq_1to1(
        fb,&assets.hud_digits_small_hq,
        hud_hq_layout::LIVES_VALUE_LEFT*s,
        hud_hq_layout::LIVES_CENTER_Y*s,
        game.lives.max(0),hud_hq_layout::LIVES_MAX_WIDTH*s,
        hud_hq_layout::LIVES_DIGIT_ADVANCE*s
    );
}

// Return full 1:1 glyph-cell span with an independently chosen digit pitch.
// Compact the transparent cell margins, never rescale runtime pixels or
// truncate numeric values. Long SCORE values retain their tested small-atlas fallback.
fn hud_digit_step_width(cell_w:i32,count:usize,max_width:i32,preferred:i32)->(i32,i32){
    if cell_w<=0 || count==0{return (0,0);}
    let n=count as i32;
    let room=max_width.max(cell_w);
    let requested=if preferred<=0{cell_w}else{preferred.min(cell_w)};
    let step=if n>1 && cell_w+(n-1)*requested>room{
        ((room-cell_w)/(n-1)).clamp(1,requested)
    }else{requested};
    (step,cell_w+step*(n-1))
}
fn draw_number_centered_hq_1to1(
    fb:&mut RenderSurface,atlas:&Image,cx:i32,cy:i32,value:i32,
    max_width:i32,preferred_advance:i32
){
    draw_number_hq_1to1(fb,atlas,cx,cy,value,max_width,preferred_advance,true);
}
fn draw_number_left_hq_1to1(
    fb:&mut RenderSurface,atlas:&Image,x:i32,cy:i32,value:i32,
    max_width:i32,preferred_advance:i32
){
    draw_number_hq_1to1(fb,atlas,x,cy,value,max_width,preferred_advance,false);
}
fn draw_number_hq_1to1(
    fb:&mut RenderSurface,atlas:&Image,ax:i32,cy:i32,value:i32,
    max_width:i32,preferred_advance:i32,centered:bool
){
    if atlas.w<10||atlas.h==0{return;}
    let cell_w=(atlas.w/10) as i32;
    if cell_w<=0{return;}
    let mut digit_buf=[0u8;16];
    let digits=format_u32_digits(value.max(0) as u32,&mut digit_buf);
    let (advance,total)=hud_digit_step_width(
        cell_w,digits.len(),max_width,preferred_advance
    );
    let x=if centered{ax-total/2}else{ax};
    let y=cy-atlas.h as i32/2;
    for (i,&byte) in digits.iter().enumerate(){
        let digit=(byte-b'0') as usize;
        blit_region_1to1(
            fb,atlas,digit*cell_w as usize,0,
            cell_w as usize,atlas.h,x+i as i32*advance,y
        );
    }
}

fn draw_root_timeline_hq_lazy(
    fb:&mut RenderSurface,
    frames:&crate::assets::LazySpriteSet,
    tick:u64,x:f32,y:f32
){
    if frames.is_empty(){return;}
    debug_assert!((frames.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    let i=(tick as usize).min(frames.len()-1);
    let f=frames.frame(i);
    let scale=frames.logical_pixel_scale;
    blit(
        fb,&f.image,
        (x*scale-f.anchor_x).round() as i32,
        (y*scale-f.anchor_y).round() as i32,
        f.image.w as i32,f.image.h as i32,false
    );
}

fn sample_region(src:&Image,sx0:usize,sy0:usize,sw:usize,sh:usize,x:i32,y:i32)->u32{
    if x<0||y<0||x>=sw as i32||y>=sh as i32{return 0;}
    src.pixels[(sy0+y as usize)*src.w+(sx0+x as usize)]
}

fn blend_bilinear_premul_3x(dst:&mut u32,p00:u32,p10:u32,p01:u32,p11:u32,w00:u32,w10:u32,w01:u32,w11:u32){
    let a00=(p00>>24)&255; let a10=(p10>>24)&255; let a01=(p01>>24)&255; let a11=(p11>>24)&255;

    // Round 11 hot path: city/scenery pixels are overwhelmingly either fully
    // transparent or fully opaque. Avoid the premultiply/unpremultiply work
    // for both cases; only antialiased edges need the general RGBA path.
    let a_or=a00|a10|a01|a11;
    if a_or==0{return;}
    let a_and=a00&a10&a01&a11;
    if a_and==255{
        let r=((((p00>>16)&255)*w00+((p10>>16)&255)*w10+((p01>>16)&255)*w01+((p11>>16)&255)*w11+4)/9)&255;
        let g=((((p00>>8)&255)*w00+((p10>>8)&255)*w10+((p01>>8)&255)*w01+((p11>>8)&255)*w11+4)/9)&255;
        let b=((p00&255)*w00+(p10&255)*w10+(p01&255)*w01+(p11&255)*w11+4)/9;
        *dst=(r<<16)|(g<<8)|b;
        return;
    }

    let asum=a00*w00+a10*w10+a01*w01+a11*w11;
    let a=(asum+4)/9;
    if a==0{return;}

    let rsum=((p00>>16)&255)*a00*w00+((p10>>16)&255)*a10*w10+((p01>>16)&255)*a01*w01+((p11>>16)&255)*a11*w11;
    let gsum=((p00>>8)&255)*a00*w00+((p10>>8)&255)*a10*w10+((p01>>8)&255)*a01*w01+((p11>>8)&255)*a11*w11;
    let bsum=(p00&255)*a00*w00+(p10&255)*a10*w10+(p01&255)*a01*w01+(p11&255)*a11*w11;
    let pr=(rsum+4)/9; let pg=(gsum+4)/9; let pb=(bsum+4)/9;

    let dr=(*dst>>16)&255; let dg=(*dst>>8)&255; let db=*dst&255;
    let inv=255-a;
    let r=(pr+dr*inv+127)/255;
    let g=(pg+dg*inv+127)/255;
    let b=(pb+db*inv+127)/255;
    *dst=(r<<16)|(g<<8)|b;
}

fn blit_region_hq3(
    dst:&mut RenderSurface,src:&Image,
    sx0:usize,sy0:usize,sw:usize,sh:usize,
    logical_x:i32,logical_y:i32
){
    if sw==0||sh==0{return;}
    debug_assert_eq!(HQ_SCALE,3);
    let scale=HQ_SCALE as i32;
    let out_w=sw as i32*scale;
    let out_h=sh as i32*scale;

    // Include the one-HQ-pixel filter fringe so transparent edges receive the
    // same pixel-center bilinear treatment as the Round-3 full-frame scaler.
    for oy in -1..=out_h{
        let dy=logical_y*scale+oy;
        if dy<0||dy>=dst.h as i32{continue;}
        let yn=oy-1;
        let yq=yn.div_euclid(scale);
        let yr=yn.rem_euclid(scale) as u32;
        let wy0=3-yr; let wy1=yr;
        for ox in -1..=out_w{
            let dx=logical_x*scale+ox;
            if dx<0||dx>=dst.w as i32{continue;}
            let xn=ox-1;
            let xq=xn.div_euclid(scale);
            let xr=xn.rem_euclid(scale) as u32;
            let wx0=3-xr; let wx1=xr;

            let p00=sample_region(src,sx0,sy0,sw,sh,xq,yq);
            let p10=sample_region(src,sx0,sy0,sw,sh,xq+1,yq);
            let p01=sample_region(src,sx0,sy0,sw,sh,xq,yq+1);
            let p11=sample_region(src,sx0,sy0,sw,sh,xq+1,yq+1);
            blend_bilinear_premul_3x(
                &mut dst.pixels[dy as usize*dst.w+dx as usize],
                p00,p10,p01,p11,wx0*wy0,wx1*wy0,wx0*wy1,wx1*wy1
            );
        }
    }
}

fn draw_hud(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    // Root frame 19 placement of symbol 716.
    const ROOT_X:f32=159.0;
    const ROOT_Y:f32=-11.0;
    // Deterministic native-bounds anchor produced from symbol 716.
    const ANCHOR_X:f32=0.48106384;
    const ANCHOR_Y:f32=1.1601379;
    let x=(ROOT_X-ANCHOR_X).round() as i32;
    let y=(ROOT_Y-ANCHOR_Y).round() as i32;
    blit(
        fb,&assets.hud_base,x,y,
        assets.hud_base.w as i32,assets.hud_base.h as i32,false
    );

    // Dynamic DefineEditText fields from the original HUD:
    // 702 = _root.batarangs, 713 = _root.score, 714 = _root.lives.
    draw_number_centered(fb,&assets.hud_digits_small,x+53,y+68,game.batarangs.max(0));
    draw_number_centered(fb,&assets.hud_digits_large,x+228,y+47,game.score.max(0));
    draw_number_centered(fb,&assets.hud_digits_small,x+155,y+69,game.lives.max(0));
}

fn draw_number_centered(fb:&mut RenderSurface,atlas:&Image,cx:i32,cy:i32,value:i32){
    if atlas.w<10 || atlas.h==0{return;}
    let cell_w=atlas.w/10;
    if cell_w==0{return;}
    let mut digit_buf=[0u8;16];
    let text=format_u32_digits(value.max(0) as u32,&mut digit_buf);
    let total=(cell_w*text.len()) as i32;
    let mut x=cx-total/2;
    let y=cy-atlas.h as i32/2;
    for &byte in text{
        if !(b'0'..=b'9').contains(&byte){continue;}
        let digit=(byte-b'0') as usize;
        blit_region(
            fb,atlas,
            digit*cell_w,0,cell_w,atlas.h,
            x,y,cell_w as i32,atlas.h as i32
        );
        x+=cell_w as i32;
    }
}

fn blit_region_1to1(
    dst:&mut RenderSurface,src:&Image,
    sx0:usize,sy0:usize,sw:usize,sh:usize,
    x:i32,y:i32
){
    if sw==0||sh==0{return;}
    for oy in 0..sh{
        let dy=y+oy as i32;
        if dy<0||dy>=dst.h as i32{continue;}
        let sy=sy0+oy;
        if sy>=src.h{continue;}
        for ox in 0..sw{
            let dx=x+ox as i32;
            if dx<0||dx>=dst.w as i32{continue;}
            let sx=sx0+ox;
            if sx>=src.w{continue;}
            let sp=src.pixels[sy*src.w+sx];
            blend(&mut dst.pixels[dy as usize*dst.w+dx as usize],sp);
        }
    }
}

fn blit_region(
    dst:&mut RenderSurface,src:&Image,
    sx0:usize,sy0:usize,sw:usize,sh:usize,
    x:i32,y:i32,rw:i32,rh:i32
){
    if sw==0||sh==0||rw<=0||rh<=0{return;}
    for oy in 0..rh{
        let dy=y+oy;
        if dy<0||dy>=dst.h as i32{continue;}
        let sy=sy0+(oy as usize*sh/rh as usize).min(sh-1);
        for ox in 0..rw{
            let dx=x+ox;
            if dx<0||dx>=dst.w as i32{continue;}
            let sx=sx0+(ox as usize*sw/rw as usize).min(sw-1);
            if sx>=src.w||sy>=src.h{continue;}
            let sp=src.pixels[sy*src.w+sx];
            blend(&mut dst.pixels[dy as usize*dst.w+dx as usize],sp);
        }
    }
}

fn draw_root_timeline_lazy_scaled(
    fb:&mut RenderSurface,
    frames:&crate::assets::LazySpriteSet,
    tick:u64,x:f32,y:f32,pixel_scale:i32
){
    if frames.is_empty(){return;}
    debug_assert!((frames.logical_pixel_scale-1.0).abs()<0.001);
    let i=(tick as usize).min(frames.len()-1);
    let f=frames.frame(i);
    let s=pixel_scale as f32;
    blit(
        fb,&f.image,
        ((x-f.anchor_x)*s).round() as i32,
        ((y-f.anchor_y)*s).round() as i32,
        f.image.w as i32*pixel_scale,f.image.h as i32*pixel_scale,false
    );
}
fn draw_root_timeline_lazy_legacy(
    fb:&mut RenderSurface,
    frames:&crate::assets::LazySpriteSet,
    tick:u64,x:f32,y:f32
){
    draw_root_timeline_lazy_scaled(fb,frames,tick,x,y,1);
}
fn draw_tiles(fb:&mut RenderSurface,game:&Game,tiles:&LevelTileSet){
    debug_assert!((tiles.logical_pixel_scale-1.0).abs()<0.001);
    let fw=((-game.camera_x/600.0).floor() as i32)*600;
    // Right/bottom edges are exclusive. Using ceil-1 preserves the old
    // result for every non-boundary camera position, but avoids decoding a
    // zero-coverage neighbor when the viewport ends exactly on a tile edge.
    let lw=((((600.0-game.camera_x)/600.0).ceil() as i32)-1)*600;
    let fh=((-game.camera_y/400.0).floor() as i32)*400;
    let lh=((((400.0-game.camera_y)/400.0).ceil() as i32)-1)*400;
    let mut y=fh;
    while y<=lh{
        let mut x=fw;
        while x<=lw{
            if let Some(t)=tiles.tile(x,y){
                let px=(x as f32+game.camera_x).round() as i32+t.crop_x;
                let py=(y as f32+game.camera_y).round() as i32+t.crop_y;
                blit(fb,&t.image,px,py,t.image.w as i32,t.image.h as i32,false);
            }
            x+=600;
        }
        y+=400;
    }
}

fn draw_tiles_alpha(fb:&mut RenderSurface,game:&Game,tiles:&LevelTileSet){
    debug_assert!((tiles.logical_pixel_scale-1.0).abs()<0.001);
    let fw=((-game.camera_x/600.0).floor() as i32)*600;
    let lw=((((600.0-game.camera_x)/600.0).ceil() as i32)-1)*600;
    let fh=((-game.camera_y/400.0).floor() as i32)*400;
    let lh=((((400.0-game.camera_y)/400.0).ceil() as i32)-1)*400;
    let mut y=fh;
    while y<=lh{
        let mut x=fw;
        while x<=lw{
            if let Some(t)=tiles.tile(x,y){
                let px=(x as f32+game.camera_x).round() as i32+t.crop_x;
                let py=(y as f32+game.camera_y).round() as i32+t.crop_y;
                blit_alpha(fb,&t.image,px,py,t.image.w as i32,t.image.h as i32,false);
            }
            x+=600;
        }
        y+=400;
    }
}

fn draw_tiles_hq(fb:&mut RenderSurface,game:&Game,tiles:&LevelTileSet){
    debug_assert!((tiles.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    let fw=((-game.camera_x/600.0).floor() as i32)*600;
    // Right/bottom edges are exclusive. Using ceil-1 preserves the old
    // result for every non-boundary camera position, but avoids decoding a
    // zero-coverage neighbor when the viewport ends exactly on a tile edge.
    let lw=((((600.0-game.camera_x)/600.0).ceil() as i32)-1)*600;
    let fh=((-game.camera_y/400.0).floor() as i32)*400;
    let lh=((((400.0-game.camera_y)/400.0).ceil() as i32)-1)*400;
    let s=HQ_SCALE as i32;
    let mut y=fh;
    while y<=lh{
        let mut x=fw;
        while x<=lw{
            if let Some(t)=tiles.tile(x,y){
                // Match the legacy renderer's integer logical tile origin,
                // then convert only that origin to presentation pixels.
                let px=(x as f32+game.camera_x).round() as i32*s+t.crop_x;
                let py=(y as f32+game.camera_y).round() as i32*s+t.crop_y;
                // Detail tiles are already baked at the exact 3x pixel
                // resolution. Avoid per-pixel resize math and out-of-bounds
                // checks of the general sprite blitter.
                blit_tile_hq_1to1(fb,&t.image,px,py);
            }
            x+=600;
        }
        y+=400;
    }
}

/// Clip a source-sized HQ detail tile once per row, then blend visible pixels.
/// Colors and alpha use the exact existing blend() routine: no approximation,
/// unsafe pointers, resampling changes or mutations to legacy sprite paths.
fn blit_tile_hq_1to1(dst:&mut RenderSurface,src:&Image,x:i32,y:i32){
    if src.w==0 || src.h==0 || dst.w==0 || dst.h==0 {return;}
    let x0=(x as i64).max(0).min(dst.w as i64) as usize;
    let y0=(y as i64).max(0).min(dst.h as i64) as usize;
    let x1=(x as i64+src.w as i64).max(0).min(dst.w as i64) as usize;
    let y1=(y as i64+src.h as i64).max(0).min(dst.h as i64) as usize;
    if x0>=x1 || y0>=y1 {return;}
    let sx=(x0 as i64-x as i64) as usize;
    let width=x1-x0;
    for dy in y0..y1{
        let sy=(dy as i64-y as i64) as usize;
        let src_start=sy*src.w+sx;
        let dst_start=dy*dst.w+x0;
        let src_row=&src.pixels[src_start..src_start+width];
        let dst_row=&mut dst.pixels[dst_start..dst_start+width];
        for (&pixel,target) in src_row.iter().zip(dst_row.iter_mut()){
            blend(target,pixel);
        }
    }
}
fn blend_rgba(dst:&mut u32,src:u32){
    let sa=(src>>24)&255;
    if sa==0{return;}
    if sa==255{*dst=src;return;}
    let da=(*dst>>24)&255;
    if da==0{*dst=src;return;}
    let inv=255-sa;
    let out_a=sa+(da*inv+127)/255;
    let sr=(src>>16)&255;let sg=(src>>8)&255;let sb=src&255;
    let dr=(*dst>>16)&255;let dg=(*dst>>8)&255;let db=*dst&255;
    let pr=sr*sa+(dr*da*inv+127)/255;
    let pg=sg*sa+(dg*da*inv+127)/255;
    let pb=sb*sa+(db*da*inv+127)/255;
    let r=(pr+out_a/2)/out_a;
    let g=(pg+out_a/2)/out_a;
    let b=(pb+out_a/2)/out_a;
    *dst=(out_a<<24)|(r<<16)|(g<<8)|b;
}
fn blit_alpha(dst:&mut RenderSurface,src:&Image,x:i32,y:i32,rw:i32,rh:i32,flip:bool){
    if rw<=0||rh<=0{return}
    for oy in 0..rh{
        let dy=y+oy;if dy<0||dy>=dst.h as i32{continue}
        let sy=(oy as usize*src.h/rh as usize).min(src.h-1);
        for ox in 0..rw{
            let dx=x+ox;if dx<0||dx>=dst.w as i32{continue}
            let raw=(ox as usize*src.w/rw as usize).min(src.w-1);
            let sx=if flip{src.w-1-raw}else{raw};
            let sp=src.pixels[sy*src.w+sx];
            blend_rgba(&mut dst.pixels[dy as usize*dst.w+dx as usize],sp);
        }
    }
}
fn blend(dst:&mut u32,src:u32){let a=(src>>24)&255;if a==0{return}let sr=(src>>16)&255;let sg=(src>>8)&255;let sb=src&255;if a==255{*dst=(sr<<16)|(sg<<8)|sb;return}let inv=255-a;let dr=(*dst>>16)&255;let dg=(*dst>>8)&255;let db=*dst&255;*dst=(((sr*a+dr*inv+127)/255)<<16)|(((sg*a+dg*inv+127)/255)<<8)|((sb*a+db*inv+127)/255);}
fn blit(dst:&mut RenderSurface,src:&Image,x:i32,y:i32,rw:i32,rh:i32,flip:bool){
    if rw<=0||rh<=0{return}
    let oy_start=(-y).max(0);
    let oy_end=(dst.h as i32-y).min(rh);
    let ox_start=(-x).max(0);
    let ox_end=(dst.w as i32-x).min(rw);
    if oy_start>=oy_end||ox_start>=ox_end{return}
    for oy in oy_start..oy_end{
        let dy=(y+oy) as usize;
        let sy=(oy as usize*src.h/rh as usize).min(src.h-1);
        let src_row=sy*src.w;
        let dst_row=dy*dst.w;
        for ox in ox_start..ox_end{
            let dx=(x+ox) as usize;
            let raw=(ox as usize*src.w/rw as usize).min(src.w-1);
            let sx=if flip{src.w-1-raw}else{raw};
            let sp=src.pixels[src_row+sx];
            blend(&mut dst.pixels[dst_row+dx],sp);
        }
    }
}

#[cfg(test)]
mod tests{
    use super::{
        blend,blend_rgba,blit,blit_tile_hq_1to1,digit_count,format_u32_digits,
        hud_digit_step_width,hud_hq_layout,HQ_SCALE,HQ_W,LOGICAL_W,XMAP_600_TO_1800,
    };
    use crate::assets::Image;
    use crate::surface::RenderSurface;

    #[test]
    fn hud_hq_layout_keeps_digits_in_readable_columns(){
        let s=HQ_SCALE as i32;
        assert_eq!(hud_hq_layout::BATARANG_CENTER,(211,58));
        assert_eq!(hud_hq_layout::BATARANG_DIGIT_ADVANCE,9);
        assert_eq!(hud_hq_layout::LIVES_DIGIT_ADVANCE,10);
        assert_eq!(hud_hq_layout::SCORE_CENTER,(387,35));
        assert_eq!(hud_hq_layout::SCORE_LARGE_DIGIT_ADVANCE,16);
        // The X remains baked at (306,63); move ONLY the live number closer.
        assert_eq!(hud_hq_layout::LIVES_VALUE_LEFT,309);
        assert_eq!(hud_hq_layout::LIVES_CENTER_Y,63);
        for &(len,budget,cell,preferred) in &[
            (1,hud_hq_layout::BATARANG_MAX_WIDTH*s,42,27),
            (3,hud_hq_layout::BATARANG_MAX_WIDTH*s,42,27),
            (2,hud_hq_layout::LIVES_MAX_WIDTH*s,42,30),
            (6,hud_hq_layout::SCORE_MAX_WIDTH*s,42,42),
            (10,hud_hq_layout::SCORE_MAX_WIDTH*s,42,42),
        ]{
            let (step,width)=hud_digit_step_width(cell,len,budget,preferred);
            assert!(step>0 && step<=cell && width<=budget,
                    "{len} digits exceed their HUD layout region");
        }
        assert!(66*6>hud_hq_layout::SCORE_MAX_WIDTH*s);
        assert!(42*6<=hud_hq_layout::SCORE_MAX_WIDTH*s);
    }

    #[test]
    fn hud_digit_layout_preserves_all_glyph_cells(){
        assert_eq!(hud_digit_step_width(42,1,180,27),(27,42));
        assert_eq!(hud_digit_step_width(42,3,180,27),(27,96));
        assert_eq!(hud_digit_step_width(42,2,180,27),(27,69));
        assert_eq!(hud_digit_step_width(42,2,180,30),(30,72));
        assert_eq!(hud_digit_step_width(42,3,180,42),(42,126));
        // 06C only: large 3x SCORE glyphs keep 66px cells but use 48px pitch.
        // Two digits occupy 114px, four occupy 210px; centered x unchanged.
        assert_eq!(hud_digit_step_width(66,2,276,16*3),(48,114));
        assert_eq!(hud_digit_step_width(66,4,276,16*3),(48,210));
        let (step,width)=hud_digit_step_width(42,10,270,42);
        assert!(step<42 && step>=1 && width<=270);
        // HUD-REAL-05C: the real 3x Impact small atlas has 42px cells;
        // at ten score digits, 26px pitch clears glyph ink while 25px
        // overlaps neighboring outlines. HQ-only 92 logical px fits 276px.
        assert_eq!(hud_hq_layout::SCORE_MAX_WIDTH,92);
        // HQ-only 06B: Batarang counter moves up 1 logical px while
        // retaining independent 9px digit pitch and original SCORE center.
        assert_eq!(hud_hq_layout::BATARANG_CENTER,(211,58));
        assert_eq!(hud_digit_step_width(42,10,92*HQ_SCALE as i32,0),(26,276));
    }

    #[test]
    fn hq_tile_clipped_blit_is_identical_to_general_blit(){
        // Retain all original alpha/blend semantics while testing clipping
        // on every side and partial visibility across the source rectangle.
        let palette=[
            0x00000000u32,0x00010203,0x01020408,0x208080ff,
            0x8040a0e0,0xfefdfcfb,0xffffffff,0xff123456,
        ];
        let image=Image{
            w:7,h:5,
            pixels:(0..35).map(|i|palette[(i*5+i/7)%palette.len()]).collect(),
        };
        for &x in &[-20,-7,-5,-1,0,2,8,12,13,17]{
            for &y in &[-20,-5,-3,-1,0,3,7,8,9,17]{
                let mut reference=RenderSurface::new(13,9);
                let mut optimized=RenderSurface::new(13,9);
                for (i,pixel) in reference.pixels.iter_mut().enumerate(){
                    *pixel=0x00102030u32.wrapping_add(i as u32*0x0003070b);
                }
                optimized.pixels.clone_from(&reference.pixels);
                blit(&mut reference,&image,x,y,7,5,false);
                blit_tile_hq_1to1(&mut optimized,&image,x,y);
                assert_eq!(optimized.pixels,reference.pixels,
                    "HQ 1:1 tile diverged from generic at x={x}, y={y}");
            }
        }
    }

    #[test]
    fn hq_tile_clipped_blit_keeps_zero_sized_and_offscreen_tiles_noop(){
        let image=Image{w:0,h:0,pixels:Vec::new()};
        let mut dst=RenderSurface::new(5,4);
        dst.fill(0xff336699);
        blit_tile_hq_1to1(&mut dst,&image,0,0);
        let other=Image{w:2,h:2,pixels:vec![0xffffffff;4]};
        blit_tile_hq_1to1(&mut dst,&other,10,-10);
        assert!(dst.pixels.iter().all(|&px|px==0xff336699));
    }


    #[test]
    fn rgba_scratch_blend_matches_legacy_over_opaque_destination(){
        let cases=[
            (0xff102030u32,0x8040a0e0u32),
            (0xffabcdefu32,0x20ff0000u32),
            (0xff000000u32,0xffffffffu32),
            (0xffffffffu32,0x00010203u32),
        ];
        for &(dst0,src) in &cases{
            let mut legacy=dst0;
            let mut rgba=dst0;
            blend(&mut legacy,src);
            blend_rgba(&mut rgba,src);
            assert_eq!(rgba>>24,0xff);
            assert_eq!(rgba&0x00ff_ffff,legacy&0x00ff_ffff);
        }
    }

    #[test]
    fn rgba_scratch_blend_preserves_source_over_transparent_destination(){
        for &src in &[0x8040a0e0u32,0xffffffffu32,0x01020304u32]{
            let mut dst=0u32;
            blend_rgba(&mut dst,src);
            assert_eq!(dst,src);
        }
    }

    #[test]
    fn xmap_600_to_1800_matches_runtime_formula(){
        assert_eq!(XMAP_600_TO_1800.len(), HQ_W);
        for dx in 0..HQ_W {
            let xn = dx as isize - 1;
            let xq = xn.div_euclid(3);
            let xr = xn.rem_euclid(3) as u32;
            let x0 = xq.clamp(0, LOGICAL_W as isize - 1) as usize;
            let x1 = (xq + 1).clamp(0, LOGICAL_W as isize - 1) as usize;
            let expected = (x0, x1, 3 - xr, xr);
            assert_eq!(XMAP_600_TO_1800[dx], expected, "Mismatch at dx = {dx}");
        }
    }

    #[test]
    fn format_u32_digits_and_digit_count_match_standard_formatting(){
        let test_values: [i32; 10] = [0, 1, 9, 10, 42, 99, 100, 1234, 999999, 2147483647];
        let mut buf = [0u8; 16];
        for &val in &test_values {
            let expected_str = val.to_string();
            assert_eq!(digit_count(val), expected_str.len(), "digit_count mismatch for {val}");
            let formatted = format_u32_digits(val as u32, &mut buf);
            assert_eq!(formatted, expected_str.as_bytes(), "format_u32_digits mismatch for {val}");
        }
        assert_eq!(digit_count(-1), 1);
        assert_eq!(digit_count(-999), 1);
    }

    #[test]
    fn labs2a_renders_multifloor_tiles_over_black_backdrop_legacy_and_hq(){
        let mut assets=crate::assets::Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        let mut game=crate::game::Game::new();
        game.screen=crate::game::AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level2A);
        game.ticks=45;

        let mut fb1x=RenderSurface::new(LOGICAL_W,super::LOGICAL_H);
        let mut scratch=RenderSurface::new(LOGICAL_W,super::LOGICAL_H);
        let mut fbhq=RenderSurface::new(HQ_W,super::HQ_H);

        for (cam_x,cam_y) in [(0.0f32,0.0f32),(-1800.0,-2400.0),(-4800.0,-3600.0),(-4200.0,-6800.0)]{
            game.camera_x=cam_x;
            game.camera_y=cam_y;
            game.player.x = -cam_x + 300.0 - game.level_origin_x();
            game.player.y = -cam_y + 200.0 - game.level_origin_y();
            super::render(&mut fb1x,&game,&assets);
            super::render_high_quality(&mut scratch,&mut fbhq,&game,&assets);
            let non_black_1x=fb1x.pixels.iter().filter(|&&p|(p&0x00ff_ffff)!=0).count();
            let non_black_hq=fbhq.pixels.iter().filter(|&&p|(p&0x00ff_ffff)!=0).count();
            assert!(non_black_1x>5_000,"legacy 1x Labs 2A render at ({cam_x},{cam_y}) must show lab geometry");
            assert!(non_black_hq>45_000,"HQ Labs 2A render at ({cam_x},{cam_y}) must show lab geometry");
            // Root #14 solid black void outside the mid-lab room wing (no cityscape/red-sky bleed)
            if (cam_x,cam_y)==(-4800.0,-3600.0){
                assert_eq!(fb1x.pixels[300*LOGICAL_W]&0x00ff_ffff,0,"mid-lab void margin must use #14 black backdrop");
                assert_eq!(fbhq.pixels[900*HQ_W]&0x00ff_ffff,0,"HQ mid-lab void margin must use #14 black backdrop");
            }
        }
    }

    #[test]
    fn b22_labs2a_doors_animate_and_jambs_occlude_batman_in_1x_and_hq3x() {
        use super::{
            draw_batman_legacy, draw_world_foreground_legacy, render, render_high_quality,
            HQ_H, LOGICAL_H,
        };
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level2A);
        game.screen = AppScreen::Playing;
        game.ticks = 45;
        // Center view on doorway 1 (`door_01`/`door_02` around gx=614, gy=2544) with Batman at doorway edge
        // so both the door leaf opening (0 -> 9) and front jamb overlap Batman's horizontal span.
        game.camera_x = -314.0;
        game.camera_y = -2344.0;
        game.player.x = 696.0 - game.level_origin_x();
        game.player.y = 2573.0 - game.level_origin_y();
        game.labs_doors[0].frame = 0;
        game.labs_doors[1].frame = 0;

        let mut closed_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut open_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut open_hq = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut closed_1x, &game, &assets);
        game.labs_doors[0].frame = 9;
        game.labs_doors[1].frame = 9;
        render(&mut open_1x, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut open_hq, &game, &assets);

        assert!(closed_1x.pixels != open_1x.pixels, "opening door frame 0 -> 9 must visibly change doorway pixels");
        let hq_non_black = open_hq.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count();
        assert!(hq_non_black > 20_000, "HQ Labs 2A doorway scene must render cleanly");

        // Verify jambs (#1394/#1395/#1396) draw AFTER Batman (`depth 377 > player 374`), occluding Batman
        // inside the jamb overlap strip at `jamb[1]` (`gx=702.15, gy=2565.3`).
        let mut without_jamb = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut with_jamb = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        draw_batman_legacy(&mut without_jamb, &game, &assets);
        draw_batman_legacy(&mut with_jamb, &game, &assets);
        draw_world_foreground_legacy(&mut with_jamb, &game, &assets);
        assert!(without_jamb.pixels != with_jamb.pixels, "front door jamb must overdraw Batman in foreground pass");
    }

    #[test]
    fn b23_labs2a_elevators_render_and_move_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level2A);
        game.screen = AppScreen::Playing;
        game.ticks = 45;
        // Frame entry elevator_01_2685 (x=308.95, y=429.9) in viewport.
        game.camera_x = 0.0;
        game.camera_y = -200.0;

        let mut at_top = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut moved = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut at_top, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        game.labs_elevators[0].y = 510.0;
        render(&mut moved, &game, &assets);

        assert!(at_top.pixels != moved.pixels, "moving elevator state.y must shift rendered platform in viewport");
        let hq_non_black = hq_out.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count();
        assert!(hq_non_black > 30_000, "HQ Labs 2A elevator view must render cleanly");
    }

    #[test]
    fn b24_labs2a_enemies_render_and_flip_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level2A);
        game.screen = AppScreen::Playing;
        game.ticks = 45;
        // Frame goon2 at (1574.85, 2656.35) near viewport center (Flash camera_x = -world_x + screen_x)
        game.camera_x = 300.0 - 1574.85;
        game.camera_y = 260.0 - 2656.35;
        game.player.x = 1574.85 - game.level_origin_x();
        game.player.y = 2656.35 - game.level_origin_y();

        let mut right_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut left_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut right_1x, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        game.bank_goons.as_mut().unwrap().goons[0].dir = -1;
        render(&mut left_1x, &game, &assets);

        assert!(right_1x.pixels != left_1x.pixels, "flipping Labs goon facing dir must alter rendered silhouette");
        let hq_non_black = hq_out.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count();
        assert!(hq_non_black > 25_000, "HQ Labs 2A goon scene must render cleanly");
    }

    #[test]
    fn b25_labs2a_pickups_halo_and_level2_title_render_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level2A);
        game.screen = AppScreen::Playing;
        game.ticks = 15; // LEVEL 2 banner visible
        let h = crate::stage::labs2a::HEALTH_PICKUPS[0];
        game.camera_x = 300.0 - h.x;
        game.camera_y = 220.0 - h.y;
        game.player.x = h.x - game.level_origin_x();
        game.player.y = h.y - game.level_origin_y();

        let mut uncollected = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut collected = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut uncollected, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        game.labs_health_collected = true;
        render(&mut collected, &game, &assets);

        assert!(uncollected.pixels != collected.pixels, "collecting health01 must hide #1401 pickup");
        let hq_non_black = hq_out.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count();
        assert!(hq_non_black > 25_000, "HQ Labs 2A halo + pickups + LEVEL 2 scene must render cleanly");
    }
}
