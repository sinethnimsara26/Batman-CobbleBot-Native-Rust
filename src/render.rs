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
                    AppScreen::KabukiCard=>game.kabuki_card_fadeout_tick,
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
        AppScreen::KabukiCard=>{
            render_story_timeline_legacy(
                fb,assets,&assets.kabuki_card_frames,game.kabuki_card_tick,game.kabuki_card_fadeout_tick,show_fade,
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
    draw_labs_lava_scaled(fb,game,assets,1);
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
    draw_labs_lava_scaled(fb,game,assets,HQ_SCALE as i32);
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

fn player_is_blink_hidden(game:&Game)->bool{
    game.player_invincible_ticks>0 && (game.player_invincible_ticks%4)>=2
}

fn draw_batman_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    if player_is_blink_hidden(game){return;}
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
    if player_is_blink_hidden(game){return;}
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
    let placements=game.active_stage().labs_doors();
    if placements.is_empty() || assets.door_frames.len()<20 {
        return;
    }
    for (i,&door) in placements.iter().enumerate(){
        let frame_idx=game.labs_doors.get(i).map_or(0,|s|(s.frame as usize).min(19));
        draw_lazy_world_sprite(
            fb,&assets.door_frames.frame(frame_idx),
            door.x,door.y,game.camera_x,game.camera_y,door.sx<0.0,pixel_scale,
        );
    }
}

fn draw_labs_elevators_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    let placements=game.active_stage().labs_elevators();
    if placements.is_empty() || assets.elevator_frames.len()<2 {
        return;
    }
    for (i,&elev) in placements.iter().enumerate(){
        let y=game.labs_elevators.get(i).map_or(elev.y,|s|s.y);
        draw_lazy_world_sprite(
            fb,&assets.elevator_frames.frame(elev.pack_frame()),
            elev.x,y,game.camera_x,game.camera_y,elev.sx<0.0,pixel_scale,
        );
    }
}

fn draw_labs_lava_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    let lavas=game.active_stage().labs_lavas();
    if lavas.is_empty() || assets.lava_frames.is_empty() {
        return;
    }
    let frame_idx=(game.ticks as usize)%assets.lava_frames.len();
    let frame=assets.lava_frames.frame(frame_idx);
    for &lava in lavas{
        draw_lazy_world_sprite(
            fb,&frame,lava.x,lava.y,game.camera_x,game.camera_y,false,pixel_scale,
        );
    }
}

/// Exact `#1488` red alarm `CXFORMWITHALPHA` (`mult[3]` out of `256`) across `#1495` (`halo`) frames `0..=29`.
const HALO_RED_ALPHA: [u32; 30] = [
    128, 114, 100, 85, 71, 57, 43, 28, 14, 0,
    0, 0, 0, 18, 37, 55, 73, 91, 110, 128,
    128, 128, 133, 138, 142, 147, 152, 157, 161, 166,
];

fn blend_full_viewport_red_tint(fb:&mut RenderSurface,alpha_256:u32){
    if alpha_256==0{return;}
    let inv=256-alpha_256;
    let tint_r=192*alpha_256;
    for px in &mut fb.pixels{
        let p=*px;
        let r=(((p>>16)&255)*inv+tint_r)>>8;
        let g=(((p>>8)&255)*inv)>>8;
        let b=((p&255)*inv)>>8;
        *px=0xff00_0000|(r<<16)|(g<<8)|b;
    }
}

fn draw_world_foreground_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    let stage=game.active_stage();
    if !matches!(stage,crate::stage::StageId::Level2A|crate::stage::StageId::Level2B) || assets.door_frames.len()<23 {
        return;
    }
    for &jamb in stage.labs_door_jambs(){
        if let Some(frame_idx)=jamb.jamb_pack_frame(){
            draw_lazy_world_sprite(
                fb,&assets.door_frames.frame(frame_idx),
                jamb.x,jamb.y,game.camera_x,game.camera_y,jamb.sx<0.0,pixel_scale,
            );
        }
    }
    // Flash depth 413 (`halo` #1399 on 2A) / depth 313 (`halo` #1495 on 2B) follows Batman.
    if stage == crate::stage::StageId::Level2B {
        let alpha=HALO_RED_ALPHA[(game.halo_frame as usize).min(HALO_RED_ALPHA.len()-1)];
        blend_full_viewport_red_tint(fb,alpha);
        if game.halo_fire && game.halo_frame>=21 && !assets.halo_fire_frames.is_empty(){
            let fire_idx=(game.halo_frame as usize - 21).min(assets.halo_fire_frames.len()-1);
            let player_gx=game.player.x+game.level_origin_x();
            let player_gy=game.player.y+game.level_origin_y();
            draw_lazy_world_sprite(
                fb,&assets.halo_fire_frames.frame(fire_idx),
                player_gx,player_gy,game.camera_x,game.camera_y,false,pixel_scale,
            );
        }
    }
    if stage == crate::stage::StageId::Level2A && assets.labs_pickups_frames.len() >= 2 {
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
            if let Some(&h) = stage.labs_health_pickups().first() {
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
    }
    let batarang_pickups = stage.labs_batarang_pickups();
    if batarang_pickups.is_empty() || (stage == crate::stage::StageId::Level2B && game.pickups.is_empty()) {
        return;
    }
    if pixel_scale == HQ_SCALE as i32 && !assets.pickup_frames_hq.is_empty() {
        let pf = &assets.pickup_frames_hq[(game.ticks as usize) % assets.pickup_frames_hq.len()];
        for (i, &p) in batarang_pickups.iter().enumerate() {
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
        for (i, &p) in batarang_pickups.iter().enumerate() {
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

const L2B_PANEL_Y:i32=4;
const L2B_PANEL_H:i32=82;
const L2B_PANEL_R:i32=7;
const L2B_CLOCK_W:i32=142;
const L2B_CLOCK_GAP:i32=8;
const L2B_HUD_X:i32=158;
const L2B_HUD_W:i32=280;
const L2B_HUD_DY:i32=8;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
struct TopBarLayout{
    hud_x:i32,
    hud_y:i32,
    clock_pos:Option<(i32,i32)>,
}

fn level2b_clock_visible(game:&Game,assets:&Assets,pixel_scale:i32)->bool{
    if game.active_stage()!=crate::stage::StageId::Level2B{
        return false;
    }
    let pack=if pixel_scale==HQ_SCALE as i32{&assets.clock_frames_hq}else{&assets.clock_frames};
    pack.len()>=12
}

fn compute_top_bar_layout(cw:i32,use_panel:bool,clock_visible:bool)->TopBarLayout{
    if !use_panel{
        return TopBarLayout{
            hud_x:L2B_HUD_X,
            hud_y:L2B_PANEL_Y,
            clock_pos:None,
        };
    }
    if clock_visible{
        let pair_w=L2B_CLOCK_W+L2B_CLOCK_GAP+L2B_HUD_W;
        if cw>=pair_w+16{
            let clock_x=(cw-pair_w)/2;
            let hud_x=clock_x+L2B_CLOCK_W+L2B_CLOCK_GAP;
            TopBarLayout{
                hud_x,
                hud_y:L2B_PANEL_Y,
                clock_pos:Some((clock_x,L2B_PANEL_Y)),
            }
        }else{
            let hud_x=(cw-L2B_HUD_W)/2;
            let clock_x=(cw-L2B_CLOCK_W)/2;
            let clock_y=L2B_PANEL_Y+L2B_PANEL_H+L2B_CLOCK_GAP;
            TopBarLayout{
                hud_x,
                hud_y:L2B_PANEL_Y,
                clock_pos:Some((clock_x,clock_y)),
            }
        }
    }else{
        TopBarLayout{
            hud_x:(cw-L2B_HUD_W)/2,
            hud_y:L2B_PANEL_Y,
            clock_pos:None,
        }
    }
}

fn draw_translucent_hud_panel(fb:&mut RenderSurface,lx:i32,ly:i32,lw:i32,lh:i32,radius:i32,scale:i32){
    let s=scale.max(1);
    let x0=(lx*s).max(0);
    let y0=(ly*s).max(0);
    let w=lw*s;
    let h=lh*s;
    if w<=0||h<=0{return;}
    let x1=(lx*s+w).min(fb.w as i32);
    let y1=(ly*s+h).min(fb.h as i32);
    if x0>=x1||y0>=y1{return;}

    let r=(radius as f32*s as f32).min((w.min(h) as f32)*0.5);
    let cx=(lx*s) as f32+(w as f32)*0.5;
    let cy=(ly*s) as f32+(h as f32)*0.5;
    let hx=(w as f32)*0.5-r;
    let hy=(h as f32)*0.5-r;
    let outer_b=if s>1{2.2_f32}else{1.0_f32};
    let inner_b=outer_b+if s>1{1.6_f32}else{1.0_f32};

    for py in y0..y1{
        let row=(py as usize)*fb.w;
        let dy=((py as f32+0.5)-cy).abs()-hy;
        for px in x0..x1{
            let dx=((px as f32+0.5)-cx).abs()-hx;
            let dist=dx.max(0.0).hypot(dy.max(0.0))+dx.max(dy).min(0.0)-r;
            if dist>=0.5{continue;}
            let src=if dist>=-outer_b{
                let cov=(0.5-dist).clamp(0.0,1.0);
                let a=(236.0*cov).round() as u32;
                (a<<24)|0x08_0b10
            }else if dist>=-inner_b{
                0x62_8296ac
            }else{
                0xb8_121820
            };
            blend(&mut fb.pixels[row+px as usize],src);
        }
    }
}

#[inline]
fn is_hud_shell_backdrop_px(src:u32,ox:i32,oy:i32,w:i32,scale:i32)->bool{
    let a=(src>>24)&255;
    if a==0{return true;}
    let s=scale.max(1);
    if ox<8*s||ox>=w-8*s||oy<12*s{
        return true;
    }
    let r=((src>>16)&255) as i32;
    let g=((src>>8)&255) as i32;
    let b=(src&255) as i32;
    let is_slate_fill=a<=190
        &&(50..=150).contains(&r)
        &&(70..=170).contains(&g)
        &&(75..=175).contains(&b)
        &&(g-b).abs()<=15
        &&(8..=40).contains(&(b-r));
    let is_slate_rim=(55..=130).contains(&r)
        &&(80..=145).contains(&g)
        &&(85..=150).contains(&b)
        &&(g-b).abs()<=10
        &&(15..=36).contains(&(g-r));
    let is_tiny_aa=a<=35&&r.max(g).max(b)<=140;
    is_slate_fill||is_slate_rim||is_tiny_aa
}

fn blit_hud_foreground_only(dst:&mut RenderSurface,src:&Image,x:i32,y:i32,scale:i32){
    let rw=src.w as i32;
    let rh=src.h as i32;
    if rw<=0||rh<=0{return;}
    let oy_start=(-y).max(0);
    let oy_end=(dst.h as i32-y).min(rh);
    let ox_start=(-x).max(0);
    let ox_end=(dst.w as i32-x).min(rw);
    if oy_start>=oy_end||ox_start>=ox_end{return;}
    for oy in oy_start..oy_end{
        let dy=(y+oy) as usize;
        let src_row=(oy as usize)*src.w;
        let dst_row=dy*dst.w;
        for ox in ox_start..ox_end{
            let sp=src.pixels[src_row+ox as usize];
            if is_hud_shell_backdrop_px(sp,ox,oy,rw,scale){continue;}
            blend(&mut dst.pixels[dst_row+(x+ox) as usize],sp);
        }
    }
}

fn draw_level2b_clock_scaled(fb:&mut RenderSurface,game:&Game,assets:&Assets,pixel_scale:i32){
    if game.active_stage()!=crate::stage::StageId::Level2B{return;}
    let pack=if pixel_scale==HQ_SCALE as i32{&assets.clock_frames_hq}else{&assets.clock_frames};
    if pack.len()<12{return;}
    let cw=(fb.w as i32)/pixel_scale.max(1);
    let layout=compute_top_bar_layout(cw,true,true);
    let Some((clock_x,clock_y))=layout.clock_pos else{return;};
    draw_translucent_hud_panel(fb,clock_x,clock_y,L2B_CLOCK_W,L2B_PANEL_H,L2B_PANEL_R,pixel_scale);
    let clock_origin_x:f32=(clock_x-1) as f32;
    let clock_origin_y:f32=(clock_y+4) as f32;
    const CLOCK_DIGITS_DX:i32=12;
    let scale=pixel_scale as f32;
    let shell=pack.frame(0);
    blit(
        fb,&shell.image,
        (clock_origin_x*scale-shell.anchor_x).round() as i32,
        (clock_origin_y*scale-shell.anchor_y).round() as i32,
        shell.image.w as i32,shell.image.h as i32,false,
    );
    // Centered 5-slot `MM:SS` countdown display (`00:00`..`99:59`).
    let total_secs=game.clock_secs.min(5999);
    let mins=total_secs/60;
    let rem=total_secs%60;
    let slots:[usize;5]=[
        (mins/10) as usize + 1,
        (mins%10) as usize + 1,
        11,
        (rem/10) as usize + 1,
        (rem%10) as usize + 1,
    ];
    let mut slot_x=CLOCK_DIGITS_DX*pixel_scale;
    for frame_idx in slots{
        let f=pack.frame(frame_idx);
        blit(
            fb,&f.image,
            (clock_origin_x*scale-f.anchor_x).round() as i32 + slot_x,
            (clock_origin_y*scale-f.anchor_y).round() as i32,
            f.image.w as i32,f.image.h as i32,false,
        );
        slot_x+=if frame_idx==11{5}else{17}*pixel_scale;
    }
}

fn draw_foreground_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    draw_hud(fb,game,assets);
    draw_level2b_clock_scaled(fb,game,assets,1);
    draw_penguin_boss_health_bar(fb,game,1);

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
    draw_level2b_clock_scaled(fb,game,assets,HQ_SCALE as i32);
    draw_penguin_boss_health_bar(fb,game,HQ_SCALE);

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
            draw_root_timeline_hq_lazy(fb,&assets.level2a_title_frames_hq,game.ticks,318.7,76.75);
        }
        _ => {}
    }
    draw_root_timeline_hq_lazy(fb,&assets.fadein_frames_hq,game.ticks,301.0,205.0);
    if let Some(t)=game.fadeout_tick{
        draw_root_timeline_hq_lazy(fb,&assets.fadeout_frames_hq,t as u64,301.0,205.0);
    }
}

fn draw_hud_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    let clock_visible=level2b_clock_visible(game,assets,HQ_SCALE as i32);
    draw_hud_hq_with_clock(fb,game,assets,clock_visible);
}

fn draw_hud_hq_with_clock(fb:&mut RenderSurface,game:&Game,assets:&Assets,clock_visible:bool){
    let use_panel=game.active_stage()!=crate::stage::StageId::Level1A;
    let s=HQ_SCALE as i32;
    let cw=(fb.w as i32)/s;
    let layout=compute_top_bar_layout(cw,use_panel,clock_visible);
    let dx_logical=layout.hud_x-L2B_HUD_X;
    let dy_logical=if use_panel{L2B_HUD_DY}else{0};
    let root_x:f32=(layout.hud_x+1) as f32;
    let root_y:f32=-11.0+dy_logical as f32;
    debug_assert!((assets.hud_shell_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    debug_assert_eq!(assets.hud_shell_hq.len(),1);
    let shell=&assets.hud_shell_hq[0];
    let scale=assets.hud_shell_hq.logical_pixel_scale;
    let shell_x=(root_x*scale-shell.anchor_x).round() as i32;
    let shell_y=(root_y*scale-shell.anchor_y).round() as i32;

    if use_panel{
        draw_translucent_hud_panel(fb,layout.hud_x,layout.hud_y,L2B_HUD_W,L2B_PANEL_H,L2B_PANEL_R,s);
        blit_hud_foreground_only(fb,&shell.image,shell_x,shell_y,s);
    }else{
        // Original sprite 716: icon art and rounded translucent backing remain.
        // The unwanted separate full-width counter band must never return.
        blit(fb,&shell.image,shell_x,shell_y,
             shell.image.w as i32,shell.image.h as i32,false);
    }

    // Themed labels and dynamic counters share one stage-space layout; 
    // source-authentic 1x Flash HUD and game state remain unchanged.
    let (bx,by)=hud_hq_layout::BATARANG_CENTER;
    draw_number_centered_hq_1to1(
        fb,&assets.hud_digits_batarang_hq,(bx+dx_logical)*s,(by+dy_logical)*s,game.batarangs.max(0),
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
        fb,score_atlas,(sx+dx_logical)*s,(sy+dy_logical)*s,score,score_budget,score_advance
    );

    // LIVES X is baked directly before this fixed value-left edge.
    // Widening numbers extend rightward, never back over the caption.
    draw_number_left_hq_1to1(
        fb,&assets.hud_digits_small_hq,
        (hud_hq_layout::LIVES_VALUE_LEFT+dx_logical)*s,
        (hud_hq_layout::LIVES_CENTER_Y+dy_logical)*s,
        game.lives.max(0),hud_hq_layout::LIVES_MAX_WIDTH*s,
        hud_hq_layout::LIVES_DIGIT_ADVANCE*s
    );
    apply_hud_health_bat_mask_offset(fb,game.health,HQ_SCALE,dx_logical,dy_logical);
}

fn blend_rect_argb(fb:&mut RenderSurface,x:i32,y:i32,w:i32,h:i32,argb:u32){
    if w<=0 || h<=0{return;}
    let x0=x.max(0) as usize;
    let y0=y.max(0) as usize;
    let x1=(x+w).min(fb.w as i32).max(0) as usize;
    let y1=(y+h).min(fb.h as i32).max(0) as usize;
    for py in y0..y1{
        let row=py*fb.w;
        for px in x0..x1{
            blend(&mut fb.pixels[row+px],argb);
        }
    }
}

/// Draws a thick, glowing, label-free Penguin boss health meter directly above his head during Level 1C.
fn draw_penguin_boss_health_bar(fb:&mut RenderSurface,game:&Game,scale:usize){
    if game.screen!=AppScreen::Playing || game.active_stage()!=crate::stage::StageId::Level1C{
        return;
    }
    let Some(boss)=&game.penguin_boss else{return;};
    // Source #1249 die clip has fully transparent local frames 38..39 (RE ledger).
    // Keep the empty meter during visible death animation, but not after Penguin vanishes.
    if !boss.active || (boss.state==crate::penguin::PenguinState::Die && boss.child_frame>=38){return;}
    let s=scale.max(1) as i32;
    let boss_sx=(boss.x+game.camera_x).round() as i32*s;
    let boss_sy=(boss.y+game.camera_y).round() as i32*s;

    let panel_w=116*s;
    let panel_h=20*s;
    if boss_sx+panel_w/2+5*s<0 || boss_sx-panel_w/2-5*s>fb.w as i32{
        return;
    }
    let panel_x=boss_sx-panel_w/2;
    let max_panel_y=(fb.h as i32-panel_h-4*s).max(4*s);
    let panel_y=(boss_sy-82*s).clamp(4*s,max_panel_y);

    let bar_x=panel_x+3*s;
    let bar_y=panel_y+3*s;
    let bar_w=panel_w-6*s;
    let bar_h=14*s;
    let inner_x=bar_x+s;
    let inner_y=bar_y+s;
    let inner_w=bar_w-2*s;
    let inner_h=12*s;

    let max_hp=boss.max_health.max(1);
    let hp=boss.health.clamp(0,max_hp);
    let fill_w=if hp>0{
        (((inner_w as f32)*(hp as f32)/(max_hp as f32)).round() as i32).clamp(s,inner_w)
    }else{
        0
    };

    // Multi-pass radiant outer halo around the active fill span before drawing the steel frame.
    if fill_w>0{
        let glow_w=((inner_x+fill_w+4*s)-panel_x).min(panel_w);
        blend_rect_argb(fb,panel_x-5*s,panel_y-5*s,glow_w+10*s,panel_h+10*s,0x1e_ff3314);
        blend_rect_argb(fb,panel_x-3*s,panel_y-3*s,glow_w+6*s,panel_h+6*s,0x32_ff481e);
        blend_rect_argb(fb,panel_x-2*s,panel_y-2*s,glow_w+4*s,panel_h+4*s,0x4a_ff6628);
        blend_rect_argb(fb,panel_x-s,panel_y-s,glow_w+2*s,panel_h+2*s,0x64_ff8838);
    }

    // Beveled dark-steel housing matching HUD casing.
    blend_rect_argb(fb,panel_x,panel_y,panel_w,panel_h,0xe8_10141c);
    blend_rect_argb(fb,panel_x,panel_y,panel_w,s,0xff_07090d);
    blend_rect_argb(fb,panel_x,panel_y+panel_h-s,panel_w,s,0xff_07090d);
    blend_rect_argb(fb,panel_x,panel_y,s,panel_h,0xff_07090d);
    blend_rect_argb(fb,panel_x+panel_w-s,panel_y,s,panel_h,0xff_07090d);
    blend_rect_argb(fb,panel_x+s,panel_y+s,panel_w-2*s,s,0xff_708299);
    blend_rect_argb(fb,panel_x+s,panel_y+s,s,panel_h-2*s,0xff_708299);
    blend_rect_argb(fb,panel_x+s,panel_y+panel_h-2*s,panel_w-2*s,s,0xff_28303d);
    blend_rect_argb(fb,panel_x+panel_w-2*s,panel_y+2*s,s,panel_h-4*s,0xff_28303d);

    // Inner ember halo around the active bar frame + recessed dark track.
    if fill_w>0{
        blend_rect_argb(fb,bar_x-s,bar_y-s,(fill_w+4*s).min(bar_w+2*s),bar_h+2*s,0x44_ff5a22);
    }
    blend_rect_argb(fb,bar_x,bar_y,bar_w,bar_h,0xff_0a0b0e);
    blend_rect_argb(fb,inner_x,inner_y,inner_w,inner_h,0xff_1c0e12);

    if fill_w>0{
        // Luminous 12-row vertical core gradient preserving fiery stops + hot specular top.
        const GRADIENT_STOPS:[u32;12]=[
            0xff_ffe699,
            0xff_ffb347,
            0xff_f06434,
            0xff_ff542b,
            0xff_ec3b24,
            0xff_d92622,
            0xff_d92622,
            0xff_c41f1d,
            0xff_b01918,
            0xff_9c1414,
            0xff_7e0e10,
            0xff_5c0a0d,
        ];
        for (i,&color) in GRADIENT_STOPS.iter().enumerate(){
            blend_rect_argb(fb,inner_x,inner_y+(i as i32)*s,fill_w,s,color);
        }
        // Subtle 25% segment dividers across the meter track.
        for q in 1..4{
            let qx=inner_x+(inner_w*q)/4;
            blend_rect_argb(fb,qx,inner_y,s,inner_h,0x44_080a0e);
        }
        // Bright active tip at the current HP edge when damaged.
        if fill_w<inner_w{
            if fill_w>=2*s{
                blend_rect_argb(fb,inner_x+fill_w-2*s,inner_y,s,inner_h,0xaa_fff2b2);
            }
            blend_rect_argb(fb,inner_x+fill_w-s,inner_y,s,inner_h,0xff_ffd24c);
        }
    }else{
        for q in 1..4{
            let qx=inner_x+(inner_w*q)/4;
            blend_rect_argb(fb,qx,inner_y,s,inner_h,0x44_080a0e);
        }
    }
}

#[allow(dead_code)]
fn apply_hud_health_bat_mask(fb:&mut RenderSurface,health:i32,scale:usize){
    apply_hud_health_bat_mask_offset(fb,health,scale,0,0);
}

fn apply_hud_health_bat_mask_offset(fb:&mut RenderSurface,health:i32,scale:usize,dx_logical:i32,dy_logical:i32){
    if health>=100 || fb.w==0 || fb.h==0{return;}
    let frac=(health.clamp(0,100) as f32)/100.0;
    let (base_cx,base_cy,max_r)=if scale==HQ_SCALE{
        (901.0_f32,104.5_f32,107.5_f32)
    }else{
        (301.0_f32,35.0_f32,36.0_f32)
    };
    let s=scale as i32;
    let cx=base_cx+(dx_logical*s) as f32;
    let cy=base_cy+(dy_logical*s) as f32;
    let cutoff=max_r*frac;
    let y_start=(15+dy_logical)*s;
    let y_end=(55+dy_logical)*s;
    let x_start=(265+dx_logical)*s;
    let x_end=(338+dx_logical)*s;
    if x_end<0 || y_end<0 || x_start>=fb.w as i32 || y_start>=fb.h as i32{return;}
    let y0=y_start.max(0) as usize;
    let y1=y_end.min(fb.h as i32-1) as usize;
    let x0=x_start.max(0) as usize;
    let x1=x_end.min(fb.w as i32-1) as usize;
    if x0>x1 || y0>y1{return;}
    for y in y0..=y1{
        let row=y*fb.w;
        let dy=(y as f32)-cy;
        for x in x0..=x1{
            let px=fb.pixels[row+x];
            let r=((px>>16)&0xff) as u8;
            let g=((px>>8)&0xff) as u8;
            let b=(px&0xff) as u8;
            // Peel concentric circular rings from the outside inward across both
            // bright yellow and orange divider rings, preserving straight bat frame.
            if r>120 && g>55 && b<95 && r>b.saturating_add(40){
                let dx=(x as f32)-cx;
                let dist=(dx*dx+dy*dy).sqrt();
                if health<=0 || dist>cutoff{
                    let nr=((r as u32*28)>>8) as u8;
                    let ng=((g as u32*26)>>8) as u8;
                    let nb=((b as u32*20)>>8) as u8;
                    fb.pixels[row+x]=(px&0xff00_0000)|((nr as u32)<<16)|((ng as u32)<<8)|(nb as u32);
                }
            }
        }
    }
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
    let clock_visible=level2b_clock_visible(game,assets,1);
    draw_hud_with_clock(fb,game,assets,clock_visible);
}

fn draw_hud_with_clock(fb:&mut RenderSurface,game:&Game,assets:&Assets,clock_visible:bool){
    let use_panel=game.active_stage()!=crate::stage::StageId::Level1A;
    let cw=fb.w as i32;
    let layout=compute_top_bar_layout(cw,use_panel,clock_visible);
    let dx_logical=layout.hud_x-L2B_HUD_X;
    let dy_logical=if use_panel{L2B_HUD_DY}else{0};
    // Root frame 19 placement of symbol 716 (shifted symmetrically inside aligned container post-1A).
    let root_x:f32=(layout.hud_x+1) as f32;
    let root_y:f32=-11.0+dy_logical as f32;
    // Deterministic native-bounds anchor produced from symbol 716.
    const ANCHOR_X:f32=0.48106384;
    const ANCHOR_Y:f32=1.1601379;
    let x=(root_x-ANCHOR_X).round() as i32;
    let y=(root_y-ANCHOR_Y).round() as i32;
    if use_panel{
        draw_translucent_hud_panel(fb,layout.hud_x,layout.hud_y,L2B_HUD_W,L2B_PANEL_H,L2B_PANEL_R,1);
        blit_hud_foreground_only(fb,&assets.hud_base,x,y,1);
    }else{
        blit(
            fb,&assets.hud_base,x,y,
            assets.hud_base.w as i32,assets.hud_base.h as i32,false
        );
    }

    // Dynamic DefineEditText fields from the original HUD:
    // 702 = _root.batarangs, 713 = _root.score, 714 = _root.lives.
    draw_number_centered(fb,&assets.hud_digits_small,x+53,y+68,game.batarangs.max(0));
    draw_number_centered(fb,&assets.hud_digits_large,x+228,y+47,game.score.max(0));
    draw_number_centered(fb,&assets.hud_digits_small,x+155,y+69,game.lives.max(0));
    apply_hud_health_bat_mask_offset(fb,game.health,1,dx_logical,dy_logical);
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
        hud_digit_step_width,hud_hq_layout,render,render_high_quality,HQ_H,HQ_SCALE,HQ_W,
        LOGICAL_H,LOGICAL_W,XMAP_600_TO_1800,
    };
    use crate::assets::{Assets, Image};
    use crate::game::{AppScreen, Game};
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
    fn labs2b_renders_multifloor_tiles_over_black_backdrop_legacy_and_hq(){
        let mut assets=crate::assets::Assets::load();
        assets.load_stage(crate::stage::StageId::Level2B);
        let mut game=crate::game::Game::new();
        game.screen=crate::game::AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level2B);
        game.ticks=45;
        game.halo_frame=9; // HALO_RED_ALPHA[9] == 0 clear phase to inspect #14 black void backdrop

        let mut fb1x=RenderSurface::new(LOGICAL_W,super::LOGICAL_H);
        let mut scratch=RenderSurface::new(LOGICAL_W,super::LOGICAL_H);
        let mut fbhq=RenderSurface::new(HQ_W,super::HQ_H);

        for (cam_x,cam_y) in [(-4200.0f32,-6800.0f32),(-7200.0,-6800.0),(-4800.0,-3200.0),(0.0,0.0)]{
            game.camera_x=cam_x;
            game.camera_y=cam_y;
            game.player.x = -cam_x + 300.0 - game.level_origin_x();
            game.player.y = -cam_y + 200.0 - game.level_origin_y();
            super::render(&mut fb1x,&game,&assets);
            super::render_high_quality(&mut scratch,&mut fbhq,&game,&assets);
            let non_black_1x=fb1x.pixels.iter().filter(|&&p|(p&0x00ff_ffff)!=0).count();
            let non_black_hq=fbhq.pixels.iter().filter(|&&p|(p&0x00ff_ffff)!=0).count();
            assert!(non_black_1x>5_000,"legacy 1x Labs 2B render at ({cam_x},{cam_y}) must show lab geometry");
            assert!(non_black_hq>45_000,"HQ Labs 2B render at ({cam_x},{cam_y}) must show lab geometry");
            if (cam_x,cam_y)==(-4800.0,-3200.0){
                assert_eq!(fb1x.pixels[390*LOGICAL_W]&0x00ff_ffff,0,"mid-corridor void margin must use #14 black backdrop");
                assert_eq!(fbhq.pixels[(390*3)*HQ_W]&0x00ff_ffff,0,"HQ mid-corridor void margin must use #14 black backdrop");
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

    #[test]
    fn b28_labs2b_doors_jambs_and_elevators_render_in_1x_and_hq3x() {
        use super::{
            draw_batman_legacy, draw_world_foreground_legacy, render, render_high_quality,
            HQ_H, LOGICAL_H,
        };
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2B);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level2B);
        game.screen = AppScreen::Playing;
        game.ticks = 45;

        // Frame doorway `door_16`/`door_17` (gx=4968, gy=7035.35) near spawn floor:
        // verify door animation updates rendered pixels and foreground jamb occludes Batman.
        game.camera_x = -4668.0;
        game.camera_y = -6835.0;
        game.player.x = 5006.0 - game.level_origin_x();
        game.player.y = 7060.0 - game.level_origin_y();

        let mut closed_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut opened_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut no_jambs = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut closed_1x, &game, &assets);
        game.labs_doors[2].frame = 9;
        game.labs_doors[3].frame = 9;
        render(&mut opened_1x, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        assert_ne!(closed_1x.pixels, opened_1x.pixels, "Level 2B door frame 0 -> 9 must change rendered pixels");

        draw_batman_legacy(&mut no_jambs, &game, &assets);
        let mut with_jambs = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        draw_batman_legacy(&mut with_jambs, &game, &assets);
        draw_world_foreground_legacy(&mut with_jambs, &game, &assets);
        assert_ne!(no_jambs.pixels, with_jambs.pixels, "Level 2B foreground door jambs must draw over Batman");

        // Also verify `#1330` middle UP elevator (`elevator_02_2675`, x=2863.1, y=3484.8 -> 3050.0) moves rendered sprite.
        let e1 = crate::stage::labs2b::ELEVATORS[1];
        game.camera_x = 300.0 - e1.x;
        game.camera_y = 220.0 - 3484.8;
        let mut elev_start = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut elev_up = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        render(&mut elev_start, &game, &assets);
        game.labs_elevators[1].y = 3050.0;
        render(&mut elev_up, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        assert_ne!(elev_start.pixels, elev_up.pixels, "Level 2B UP elevator displacement must move rendered #1330 platform");
        assert!(hq_out.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count() > 25_000);
    }

    #[test]
    fn b29_labs2b_clock_lava_and_halo_render_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2B);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level2B);
        game.ticks = 45;

        // Center camera on `lava1` (#1486) so animated lava + red #1488 halo + #1503 clock are all visible.
        let lava = crate::stage::labs2b::LAVAS[0];
        game.camera_x = 300.0 - (lava.x + 120.0);
        game.camera_y = 220.0 - (lava.y + 10.0);

        let mut base_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut halo_clear_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut clock_and_lava_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut halo_fire_1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_base = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_fire = RenderSurface::new(HQ_W, HQ_H);

        // 1) Halo pulse (`frame 0` alpha=128 vs `frame 9` alpha=0) changes viewport tint pixels.
        game.halo_frame = 0;
        render(&mut base_1x, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_base, &game, &assets);
        game.halo_frame = 9;
        render(&mut halo_clear_1x, &game, &assets);
        assert_ne!(base_1x.pixels, halo_clear_1x.pixels, "#1488 red halo alpha 128 -> 0 must alter viewport pixels");

        // 2) Clock countdown (`80 :99` -> `00 :00`) and lava (`ticks 45 -> 55`) change rendered pixels.
        game.clock_secs = 0;
        game.clock_millisecs = 0;
        game.ticks = 55;
        render(&mut clock_and_lava_1x, &game, &assets);
        assert_ne!(halo_clear_1x.pixels, clock_and_lava_1x.pixels, "Level 2B clock digits and lava1 frame must animate");

        // 3) Halo `"fire"` explosion plume (`halo_frame = 25`) renders in both 1x and HQ3x.
        game.trigger_halo_fire();
        game.halo_frame = 25;
        render(&mut halo_fire_1x, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_fire, &game, &assets);
        assert_ne!(clock_and_lava_1x.pixels, halo_fire_1x.pixels, "#1495 halo fire plume must render in 1x");
        assert_ne!(hq_base.pixels, hq_fire.pixels, "#1495 halo fire plume + 3x clock must render in HQ3x");
    }

    #[test]
    fn b30_labs2b_pickups_and_tutorial_overlays_render_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2B);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level2B);
        game.ticks = 45;
        game.halo_frame = 9;
        game.overlay_frame = 84; // escapeLab (#737)

        let p0 = crate::stage::labs2b::BATARANG_PICKUPS[0];
        game.camera_x = 300.0 - p0.x;
        game.camera_y = 220.0 - p0.y;
        game.player.x = p0.x - game.level_origin_x();
        game.player.y = p0.y - game.level_origin_y();

        let mut uncollected = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut collected = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        render(&mut uncollected, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);
        game.pickups[0] = true;
        game.overlay_frame = 93; // useShaft (#737)
        render(&mut collected, &game, &assets);

        assert_ne!(uncollected.pixels, collected.pixels, "collecting Level 2B item_br01 & advancing overlay must alter rendered output");
        assert!(hq_out.pixels.iter().filter(|&&p| (p & 0x00ff_ffff) != 0).count() > 25_000);
    }

    #[test]
    fn b31_kabuki_card_renders_timeline_and_fadeout_in_1x_and_hq3x() {
        use super::{render, render_high_quality, HQ_H, LOGICAL_H};
        use crate::assets::Assets;
        use crate::game::{AppScreen, Game};

        let assets = Assets::load();
        assert_eq!(assets.common.kabuki_card_frames.len(), 166);
        let mut game = Game::new();
        game.screen = AppScreen::KabukiCard;
        game.kabuki_card_tick = 110;

        let mut legacy_110 = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut legacy_135 = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut hq_out = RenderSurface::new(HQ_W, HQ_H);
        let mut hq_faded = RenderSurface::new(HQ_W, HQ_H);

        render(&mut legacy_110, &game, &assets);
        render_high_quality(&mut hq_scratch, &mut hq_out, &game, &assets);

        game.kabuki_card_tick = 135;
        render(&mut legacy_135, &game, &assets);
        assert_ne!(legacy_110.pixels, legacy_135.pixels);

        game.kabuki_card_fadeout_tick = Some(20);
        render_high_quality(&mut hq_scratch, &mut hq_faded, &game, &assets);
        assert_ne!(hq_out.pixels, hq_faded.pixels);
    }

    #[test]
    fn hud_yellow_health_bat_shrinks_as_health_declines_in_1x_and_hq3x() {
        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level1A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level1A);
        game.screen = AppScreen::Playing;
        game.ticks = 45;

        let count_yellow = |fb: &RenderSurface, scale: usize| -> usize {
            let mut n = 0;
            for y in (16 * scale)..=(55 * scale) {
                for x in (265 * scale)..=(338 * scale) {
                    let px = fb.pixels[y * fb.w + x];
                    let r = ((px >> 16) & 0xff) as u8;
                    let g = ((px >> 8) & 0xff) as u8;
                    let b = (px & 0xff) as u8;
                    if r > 150 && g > 110 && b < 95 {
                        n += 1;
                    }
                }
            }
            n
        };

        let mut fb1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut fbhq = RenderSurface::new(HQ_W, HQ_H);

        game.health = 100;
        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        let y1x_100 = count_yellow(&fb1x, 1);
        let yhq_100 = count_yellow(&fbhq, HQ_SCALE);

        game.health = 50;
        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        let y1x_50 = count_yellow(&fb1x, 1);
        let yhq_50 = count_yellow(&fbhq, HQ_SCALE);

        game.health = 0;
        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        let y1x_0 = count_yellow(&fb1x, 1);
        let yhq_0 = count_yellow(&fbhq, HQ_SCALE);

        assert!(y1x_100 > y1x_50 && y1x_50 > y1x_0 && y1x_0 == 0, "1x bat yellow count: 100={y1x_100}, 50={y1x_50}, 0={y1x_0}");
        assert!(yhq_100 > yhq_50 && yhq_50 > yhq_0 && yhq_0 == 0, "HQ bat yellow count: 100={yhq_100}, 50={yhq_50}, 0={yhq_0}");
    }

    #[test]
    fn hud_yellow_bat_drains_concentric_circular_rings_and_preserves_health_label() {
        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level1A);
        let mut game = Game::new();
        game.start_stage(crate::stage::StageId::Level1A);
        game.screen = AppScreen::Playing;
        game.ticks = 45;

        let mut fbhq_100 = RenderSurface::new(HQ_W, HQ_H);
        let mut fbhq_60 = RenderSurface::new(HQ_W, HQ_H);
        let mut scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        game.health = 100;
        render_high_quality(&mut scratch, &mut fbhq_100, &game, &assets);
        game.health = 60;
        render_high_quality(&mut scratch, &mut fbhq_60, &game, &assets);

        // "HEALTH" label sits at y < 45 (15 * HQ_SCALE) and must stay bit-identical when health drops.
        for y in 0..(15 * HQ_SCALE) {
            for x in (265 * HQ_SCALE)..=(338 * HQ_SCALE) {
                assert_eq!(fbhq_100.pixels[y * HQ_W + x], fbhq_60.pixels[y * HQ_W + x]);
            }
        }

        // At 60 HP (cutoff = 64.5 HQ px around bullseye (901, 104.5)), outer wing ring (dx=+84)
        // is darkened even near the center horizontal line, while inner concentric core (dx=+36) stays bright.
        let inner_px = fbhq_60.pixels[104 * HQ_W + 937];
        let outer_px = fbhq_60.pixels[104 * HQ_W + 985];
        let inner_r = (inner_px >> 16) & 0xff;
        let outer_r = (outer_px >> 16) & 0xff;
        assert!(inner_r > 200 && outer_r < 80, "expected bright inner ring ({inner_r}) and darkened outer ring ({outer_r})");
    }

    #[test]
    fn penguin_boss_health_bar_tracks_above_head_and_hq_level2_title_renders_cleanly() {
        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level1C);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level1C);
        game.ticks = 45;
        // Center camera near Penguin so his sprite and overhead health bar are onscreen.
        let boss_x = game.penguin_boss.as_ref().unwrap().x;
        let boss_y = game.penguin_boss.as_ref().unwrap().y;
        game.camera_x = 300.0 - boss_x;
        game.camera_y = 316.2 - boss_y;

        let mut fb1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut fbhq = RenderSurface::new(HQ_W, HQ_H);
        let mut scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        let count_bar_crimson_in = |fb: &RenderSurface, scale: usize, lx0: usize, lx1: usize, ly0: usize, ly1: usize| -> usize {
            let mut n = 0;
            for y in (ly0 * scale)..=(ly1 * scale).min(fb.h.saturating_sub(1)) {
                for x in (lx0 * scale)..=(lx1 * scale).min(fb.w.saturating_sub(1)) {
                    let rgb = fb.pixels[y * fb.w + x] & 0x00ff_ffff;
                    if rgb == 0x00d9_2622 || rgb == 0x00f0_6434 || rgb == 0x009c_1414 {
                        n += 1;
                    }
                }
            }
            n
        };

        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        // Overhead bar sits right above Penguin at screen x ~ 300, y ~ 234..254; old bottom 366..386 must be clear.
        let bar_1x_200 = count_bar_crimson_in(&fb1x, 1, 220, 380, 225, 265);
        let bar_hq_200 = count_bar_crimson_in(&fbhq, HQ_SCALE, 220, 380, 225, 265);
        assert_eq!(count_bar_crimson_in(&fbhq, HQ_SCALE, 160, 440, 366, 386), 0, "Old fixed bottom bar must be removed");

        game.penguin_boss.as_mut().unwrap().health = 80;
        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        let bar_1x_80 = count_bar_crimson_in(&fb1x, 1, 220, 380, 225, 265);
        let bar_hq_80 = count_bar_crimson_in(&fbhq, HQ_SCALE, 220, 380, 225, 265);

        assert!(bar_1x_200 > bar_1x_80 && bar_1x_80 > 0, "1x overhead bar: 200={bar_1x_200}, 80={bar_1x_80}");
        assert!(bar_hq_200 > bar_hq_80 && bar_hq_80 > 0, "HQ overhead bar: 200={bar_hq_200}, 80={bar_hq_80}");

        // Moving Penguin horizontally by -140 px moves the overhead health bar with him.
        game.penguin_boss.as_mut().unwrap().x -= 140.0;
        render(&mut fb1x, &game, &assets);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);
        assert!(count_bar_crimson_in(&fbhq, HQ_SCALE, 100, 230, 225, 265) > 0, "Bar should track Penguin leftward");
        assert_eq!(count_bar_crimson_in(&fbhq, HQ_SCALE, 270, 380, 225, 265), 0, "Old position should be empty after Penguin moves");
        assert!((assets.level2a_title_frames_hq.logical_pixel_scale - 3.0).abs() < 0.001);
        assert_eq!(assets.level2a_title_frames_hq.len(), 100);
    }

    #[test]
    fn level2b_clock_and_hud_panels_are_aligned_with_clean_top_corners_and_gap() {
        let mut assets = Assets::load();
        assets.load_stage(crate::stage::StageId::Level2B);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level2B);
        game.ticks = 45;
        game.halo_frame = 9; // Clear phase so top-bar background is pure black void.

        let mut fbhq = RenderSurface::new(HQ_W, HQ_H);
        let mut scratch = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        render_high_quality(&mut scratch, &mut fbhq, &game, &assets);

        // Centered 600px layout: clock [85..227], 8px gap [227..235], HUD [235..515], y=[4..86].
        // Top margin y=0..8 HQ (0..2.67 logical) must remain pure black across full width.
        for y in 0..8 {
            for x in 0..HQ_W {
                assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "Top margin clipped at ({x},{y})");
            }
        }
        // Symmetric 85px left and right margins outside the centered pair must stay black at top bar rows.
        for y in (4 * HQ_SCALE)..(86 * HQ_SCALE) {
            for x in 0..(84 * HQ_SCALE) {
                assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "Left margin contaminated at ({x},{y})");
            }
            for x in (516 * HQ_SCALE)..HQ_W {
                assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "Right margin contaminated at ({x},{y})");
            }
        }
        // Clean 8-logical-px gap between clock (85..227) and HUD (235..515) at x=228..=234 must stay pure black!
        for y in (4 * HQ_SCALE)..(86 * HQ_SCALE) {
            for x in (228 * HQ_SCALE)..=(234 * HQ_SCALE) {
                assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "Clock/HUD panel gap contaminated at ({x},{y})");
            }
        }
        // Both aligned panels must share exact matching interior fill inside logical y=4..86.
        let clock_panel_px = fbhq.pixels[(10 * HQ_SCALE) * HQ_W + (89 * HQ_SCALE)];
        let hud_panel_px = fbhq.pixels[(10 * HQ_SCALE) * HQ_W + (239 * HQ_SCALE)];
        assert_eq!(clock_panel_px, hud_panel_px, "Clock and HUD translucent panels should share identical fill");
        assert_ne!(clock_panel_px & 0x00ff_ffff, 0);
    }

    #[test]
    fn post_1a_hud_uses_balanced_panel_padding_and_unclipped_lives_x() {
        use super::{draw_hud, draw_hud_hq, L2B_HUD_DY};
        use crate::stage::StageId;

        for stage in [StageId::Level1B, StageId::Level1C, StageId::Level2A, StageId::Level2B] {
            let mut assets = Assets::load();
            assets.load_stage(stage);
            let mut game = Game::new();
            game.screen = AppScreen::Playing;
            game.start_stage(stage);
            game.ticks = 45;
            game.batarangs = 12;
            game.score = 1234;
            game.lives = 5;

            let expected_hud_x = if stage == StageId::Level2B { 235 } else { 160 };

            let mut fb1x = RenderSurface::new(LOGICAL_W, LOGICAL_H);
            let mut fbhq = RenderSurface::new(HQ_W, HQ_H);
            draw_hud(&mut fb1x, &game, &assets);
            draw_hud_hq(&mut fbhq, &game, &assets);

            // Outer top (y=0..2) and bottom (y=87..91) margins stay pure black in both 1x and HQ3x.
            for y in 0..3 {
                for x in (expected_hud_x..expected_hud_x + 280) {
                    assert_eq!(fb1x.pixels[y * LOGICAL_W + x] & 0x00ff_ffff, 0, "{stage:?} 1x top overflow at ({x},{y})");
                }
            }
            for y in 88..92 {
                for x in (expected_hud_x..expected_hud_x + 280) {
                    assert_eq!(fb1x.pixels[y * LOGICAL_W + x] & 0x00ff_ffff, 0, "{stage:?} 1x bottom overflow at ({x},{y})");
                }
            }
            for y in 0..8 {
                for x in (expected_hud_x * HQ_SCALE)..((expected_hud_x + 280) * HQ_SCALE) {
                    assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "{stage:?} HQ top overflow at ({x},{y})");
                }
            }
            for y in (88 * HQ_SCALE)..(92 * HQ_SCALE) {
                for x in (expected_hud_x * HQ_SCALE)..((expected_hud_x + 280) * HQ_SCALE) {
                    assert_eq!(fbhq.pixels[y * HQ_W + x] & 0x00ff_ffff, 0, "{stage:?} HQ bottom overflow at ({x},{y})");
                }
            }

            // Balanced ~6px top & bottom inner padding: top inner rows (y=18..28 HQ) and bottom inner rows (y=241..251 HQ)
            // remain uniform panel fill with zero label/digit ink crossing into the border.
            let panel_fill = fbhq.pixels[(10 * HQ_SCALE) * HQ_W + ((expected_hud_x + 4) * HQ_SCALE)];
            assert_ne!(panel_fill & 0x00ff_ffff, 0, "{stage:?} HQ panel fill must be non-black");
            for y in [20usize, 26, 243, 249] {
                for x in ((expected_hud_x + 15) * HQ_SCALE)..=((expected_hud_x + 265) * HQ_SCALE) {
                    assert_eq!(
                        fbhq.pixels[y * HQ_W + x], panel_fill,
                        "{stage:?} ink leaked into inner padding at ({x},{y})"
                    );
                }
            }

            // Bottom rows of "X" in "LIVES X" (shell oy=237..239) must NOT be clipped.
            let shell = &assets.hud_shell_hq[0];
            let shell_x = ((expected_hud_x + 1) as f32 * HQ_SCALE as f32 - shell.anchor_x).round() as usize;
            let shell_y = ((-11 + L2B_HUD_DY) as f32 * HQ_SCALE as f32 - shell.anchor_y).round() as isize;
            let mut x_bottom_ink = 0usize;
            for oy in 237isize..=239 {
                let py = (shell_y + oy) as usize;
                for ox in 427usize..=450 {
                    let px = shell_x + ox;
                    if fbhq.pixels[py * HQ_W + px] != panel_fill {
                        x_bottom_ink += 1;
                    }
                }
            }
            assert!(x_bottom_ink >= 20, "{stage:?} bottom of 'X' in 'LIVES X' was clipped (ink={x_bottom_ink})");
        }
    }

    #[test]
    fn top_bar_alignment_centers_hud_with_and_without_clock_across_canvases() {
        use super::{
            compute_top_bar_layout, draw_hud_hq_with_clock, draw_hud_with_clock,
            draw_level2b_clock_scaled,
        };
        use crate::stage::StageId;

        // Pure layout math across 600px, 960px, 320px and Level1A.
        let l1a = compute_top_bar_layout(960, false, false);
        assert_eq!((l1a.hud_x, l1a.hud_y, l1a.clock_pos), (158, 4, None));

        let l2b_600 = compute_top_bar_layout(600, true, true);
        assert_eq!((l2b_600.clock_pos, l2b_600.hud_x), (Some((85, 4)), 235));

        let l2b_960 = compute_top_bar_layout(960, true, true);
        assert_eq!((l2b_960.clock_pos, l2b_960.hud_x), (Some((265, 4)), 415));

        let l2b_320 = compute_top_bar_layout(320, true, true);
        assert_eq!((l2b_320.clock_pos, l2b_320.hud_x, l2b_320.hud_y), (Some((89, 94)), 20, 4));

        let hud_only_600 = compute_top_bar_layout(600, true, false);
        assert_eq!((hud_only_600.hud_x, hud_only_600.clock_pos), (160, None));

        let hud_only_960 = compute_top_bar_layout(960, true, false);
        assert_eq!((hud_only_960.hud_x, hud_only_960.clock_pos), (340, None));

        let mut assets = Assets::load();
        assets.load_stage(StageId::Level2B);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level2B);

        // 960px HQ with clock: [265..407] + 8px gap + [415..695] (symmetric 265px margins).
        let mut fb960_clk = RenderSurface::new(960 * HQ_SCALE, 400 * HQ_SCALE);
        draw_hud_hq_with_clock(&mut fb960_clk, &game, &assets, true);
        draw_level2b_clock_scaled(&mut fb960_clk, &game, &assets, HQ_SCALE as i32);
        for x in 0..(264 * HQ_SCALE) {
            assert_eq!(fb960_clk.pixels[(20 * HQ_SCALE) * fb960_clk.w + x] & 0x00ff_ffff, 0);
        }
        for x in (696 * HQ_SCALE)..fb960_clk.w {
            assert_eq!(fb960_clk.pixels[(20 * HQ_SCALE) * fb960_clk.w + x] & 0x00ff_ffff, 0);
        }
        assert_ne!(fb960_clk.pixels[(20 * HQ_SCALE) * fb960_clk.w + (270 * HQ_SCALE)] & 0x00ff_ffff, 0);
        assert_ne!(fb960_clk.pixels[(20 * HQ_SCALE) * fb960_clk.w + (690 * HQ_SCALE)] & 0x00ff_ffff, 0);
        assert_eq!(fb960_clk.pixels[(20 * HQ_SCALE) * fb960_clk.w + (411 * HQ_SCALE)] & 0x00ff_ffff, 0);

        // 960px HQ & 1x WITHOUT clock (e.g. Level2B clock hidden): HUD centered at [340..620] + health bat shifted +182.
        let count_yellow_shifted = |fb: &RenderSurface, scale: usize, dx: usize| -> usize {
            let mut n = 0;
            for y in (23 * scale)..=(63 * scale) {
                for x in ((265 + dx) * scale)..=((338 + dx) * scale) {
                    let px = fb.pixels[y * fb.w + x];
                    let r = ((px >> 16) & 0xff) as u8;
                    let g = ((px >> 8) & 0xff) as u8;
                    let b = (px & 0xff) as u8;
                    if r > 150 && g > 110 && b < 95 {
                        n += 1;
                    }
                }
            }
            n
        };

        let mut fb960_hq_100 = RenderSurface::new(960 * HQ_SCALE, 400 * HQ_SCALE);
        let mut fb960_hq_0 = RenderSurface::new(960 * HQ_SCALE, 400 * HQ_SCALE);
        let mut fb960_1x_100 = RenderSurface::new(960, 400);
        let mut fb960_1x_0 = RenderSurface::new(960, 400);
        game.health = 100;
        draw_hud_hq_with_clock(&mut fb960_hq_100, &game, &assets, false);
        draw_hud_with_clock(&mut fb960_1x_100, &game, &assets, false);
        game.health = 0;
        draw_hud_hq_with_clock(&mut fb960_hq_0, &game, &assets, false);
        draw_hud_with_clock(&mut fb960_1x_0, &game, &assets, false);

        for x in 0..(339 * HQ_SCALE) {
            assert_eq!(fb960_hq_100.pixels[(20 * HQ_SCALE) * fb960_hq_100.w + x] & 0x00ff_ffff, 0);
        }
        for x in (621 * HQ_SCALE)..fb960_hq_100.w {
            assert_eq!(fb960_hq_100.pixels[(20 * HQ_SCALE) * fb960_hq_100.w + x] & 0x00ff_ffff, 0);
        }
        assert_ne!(fb960_hq_100.pixels[(20 * HQ_SCALE) * fb960_hq_100.w + (345 * HQ_SCALE)] & 0x00ff_ffff, 0);
        assert_ne!(fb960_hq_100.pixels[(20 * HQ_SCALE) * fb960_hq_100.w + (615 * HQ_SCALE)] & 0x00ff_ffff, 0);
        // dx_logical = 340 - 158 = 182
        assert!(count_yellow_shifted(&fb960_hq_100, HQ_SCALE, 182) > 500);
        assert_eq!(count_yellow_shifted(&fb960_hq_0, HQ_SCALE, 182), 0);
        assert!(count_yellow_shifted(&fb960_1x_100, 1, 182) > 50);
        assert_eq!(count_yellow_shifted(&fb960_1x_0, 1, 182), 0);

        // 320px HQ stacked layout: HUD [20..300] at y=4..86, Clock [89..231] at y=94..176.
        let mut fb320 = RenderSurface::new(320 * HQ_SCALE, 400 * HQ_SCALE);
        game.health = 100;
        draw_hud_hq_with_clock(&mut fb320, &game, &assets, true);
        draw_level2b_clock_scaled(&mut fb320, &game, &assets, HQ_SCALE as i32);
        // Clean vertical gap at y=87..93
        for y in (87 * HQ_SCALE)..(93 * HQ_SCALE) {
            for x in 0..fb320.w {
                assert_eq!(fb320.pixels[y * fb320.w + x] & 0x00ff_ffff, 0, "Stacked gap contaminated at ({x},{y})");
            }
        }
        assert_ne!(fb320.pixels[(100 * HQ_SCALE) * fb320.w + (100 * HQ_SCALE)] & 0x00ff_ffff, 0);
        assert_eq!(fb320.pixels[(100 * HQ_SCALE) * fb320.w + (40 * HQ_SCALE)] & 0x00ff_ffff, 0);
    }

    #[test]
    fn level2b_clock_formats_mm_ss_centered() {
        use super::draw_level2b_clock_scaled;
        use crate::stage::StageId;

        let mut assets = Assets::load();
        assets.load_stage(StageId::Level2B);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level2B);

        let mut fb_240_a = RenderSurface::new(HQ_W, HQ_H);
        let mut fb_240_b = RenderSurface::new(HQ_W, HQ_H);
        let mut fb_185 = RenderSurface::new(HQ_W, HQ_H);
        let mut fb_0 = RenderSurface::new(HQ_W, HQ_H);
        let mut fb_1x_240 = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        let mut fb_1x_185 = RenderSurface::new(LOGICAL_W, LOGICAL_H);

        assert_eq!(game.clock_secs, 600);
        let mut fb_600 = RenderSurface::new(HQ_W, HQ_H);
        let mut fb_1x_600 = RenderSurface::new(LOGICAL_W, LOGICAL_H);
        draw_level2b_clock_scaled(&mut fb_600, &game, &assets, HQ_SCALE as i32);
        draw_level2b_clock_scaled(&mut fb_1x_600, &game, &assets, 1);
        game.clock_secs = 240;
        game.clock_millisecs = 0;
        draw_level2b_clock_scaled(&mut fb_240_a, &game, &assets, HQ_SCALE as i32);
        draw_level2b_clock_scaled(&mut fb_1x_240, &game, &assets, 1);
        assert_ne!(fb_600.pixels, fb_240_a.pixels, "10:00 vs 04:00 HQ3x");
        assert_ne!(fb_1x_600.pixels, fb_1x_240.pixels, "10:00 vs 04:00 1x");

        game.clock_millisecs = 99;
        draw_level2b_clock_scaled(&mut fb_240_b, &game, &assets, HQ_SCALE as i32);
        assert_eq!(fb_240_a.pixels, fb_240_b.pixels, "MM:SS display must ignore sub-second clock_millisecs");

        game.clock_secs = 185;
        draw_level2b_clock_scaled(&mut fb_185, &game, &assets, HQ_SCALE as i32);
        draw_level2b_clock_scaled(&mut fb_1x_185, &game, &assets, 1);
        assert_ne!(fb_240_a.pixels, fb_185.pixels, "04:00 vs 03:05 must differ in HQ3x");
        assert_ne!(fb_1x_240.pixels, fb_1x_185.pixels, "04:00 vs 03:05 must differ in 1x");

        game.clock_secs = 0;
        draw_level2b_clock_scaled(&mut fb_0, &game, &assets, HQ_SCALE as i32);
        assert_ne!(fb_185.pixels, fb_0.pixels, "03:05 vs 00:00 must differ in HQ3x");

        // Symmetric horizontal centering of "00:00" inside the 600px centered clock panel [85..227].
        let clock_left = 85 * HQ_SCALE;
        let clock_right = (85 + 142) * HQ_SCALE;
        let mut min_x = usize::MAX;
        let mut max_x = 0usize;
        for y in (36 * HQ_SCALE)..(74 * HQ_SCALE) {
            for x in clock_left..clock_right {
                let px = fb_0.pixels[y * fb_0.w + x];
                let r = ((px >> 16) & 0xff) as u8;
                let g = ((px >> 8) & 0xff) as u8;
                if r > 180 && g < 80 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
        }
        assert!(min_x < max_x, "MM:SS red LED digits must render");
        let left_margin = min_x - clock_left;
        let right_margin = clock_right - 1 - max_x;
        assert!((left_margin as isize - right_margin as isize).abs() <= 2, "MM:SS margins L={left_margin}, R={right_margin}");
    }

    #[test]
    fn penguin_bar_disappears_at_first_invisible_death_frame_in_both_render_modes() {
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(crate::stage::StageId::Level1C);
        let boss = game.penguin_boss.as_mut().unwrap();
        game.camera_x = 300.0 - boss.x;
        game.camera_y = 316.0 - boss.y;
        boss.health = 0;
        boss.state = crate::penguin::PenguinState::Die;
        for scale in [1usize, HQ_SCALE] {
            let mut visible = RenderSurface::new(LOGICAL_W * scale, LOGICAL_H * scale);
            let mut hidden = RenderSurface::new(LOGICAL_W * scale, LOGICAL_H * scale);
            let before = hidden.pixels.clone();
            game.penguin_boss.as_mut().unwrap().child_frame = 37;
            draw_penguin_boss_health_bar(&mut visible, &game, scale);
            assert_ne!(visible.pixels, before, "visible death frame must retain the meter");
            for frame in [38, 39] {
                game.penguin_boss.as_mut().unwrap().child_frame = frame;
                draw_penguin_boss_health_bar(&mut hidden, &game, scale);
                assert_eq!(hidden.pixels, before, "invisible death frame {frame} must not draw the meter");
            }
        }
    }

    #[test]
    fn penguin_boss_health_bar_is_label_free_thicker_and_glowing() {
        use super::draw_penguin_boss_health_bar;
        use crate::stage::StageId;

        let mut assets = Assets::load();
        assets.load_stage(StageId::Level1C);
        let mut game = Game::new();
        game.screen = AppScreen::Playing;
        game.start_stage(StageId::Level1C);

        let boss_x = game.penguin_boss.as_ref().unwrap().x;
        let boss_y = game.penguin_boss.as_ref().unwrap().y;
        game.camera_x = 300.0 - boss_x;
        game.camera_y = 316.0 - boss_y;

        for &scale in &[1usize, HQ_SCALE] {
            let mut fb_full = RenderSurface::new(LOGICAL_W * scale, LOGICAL_H * scale);
            let mut fb_half = RenderSurface::new(LOGICAL_W * scale, LOGICAL_H * scale);
            let mut fb_zero = RenderSurface::new(LOGICAL_W * scale, LOGICAL_H * scale);

            game.penguin_boss.as_mut().unwrap().health = 200;
            draw_penguin_boss_health_bar(&mut fb_full, &game, scale);

            game.penguin_boss.as_mut().unwrap().health = 100;
            draw_penguin_boss_health_bar(&mut fb_half, &game, scale);

            game.penguin_boss.as_mut().unwrap().health = 0;
            draw_penguin_boss_health_bar(&mut fb_zero, &game, scale);

            let panel_x = (300 - 58) * scale; // 116*s wide centered at 300*s
            let panel_y = (316 - 82) * scale; // 234*s..254*s
            let panel_w = 116 * scale;
            let panel_h = 20 * scale;
            let inner_x = panel_x + 4 * scale;
            let inner_y = panel_y + 4 * scale;
            let inner_w = 108 * scale;
            let inner_h = 12 * scale;

            // Label-free: row 2 of the fill (0xff_f06434) spans uniformly from the left edge without a "PENGUIN" label cutout.
            let sample_y = inner_y + 2 * scale;
            let left_px = fb_full.pixels[sample_y * fb_full.w + inner_x + scale] & 0x00ff_ffff;
            assert_eq!(left_px, 0x00f0_6434, "Left of bar must be filled meter, not PENGUIN label (scale={scale})");

            // Thicker 12*s inner gradient with hot specular top (0xff_ffe699) and fiery core stops.
            let spec_px = fb_full.pixels[inner_y * fb_full.w + inner_x + scale] & 0x00ff_ffff;
            let mid_px = fb_full.pixels[(inner_y + 5 * scale) * fb_full.w + inner_x + scale] & 0x00ff_ffff;
            let dark_px = fb_full.pixels[(inner_y + 9 * scale) * fb_full.w + inner_x + scale] & 0x00ff_ffff;
            assert_eq!(spec_px, 0x00ff_e699);
            assert_eq!(mid_px, 0x00d9_2622);
            assert_eq!(dark_px, 0x009c_1414);
            // Verify all 12*s rows in inner_h are bright active gradient pixels.
            let mut active_rows = 0usize;
            for y in inner_y..(inner_y + inner_h) {
                if (fb_full.pixels[y * fb_full.w + inner_x + scale] & 0x00ff_ffff) != 0x001c_0e12 {
                    active_rows += 1;
                }
            }
            assert_eq!(active_rows, inner_h);

            // 25% dividers appear at 1/4, 2/4, 3/4 of inner_w.
            for q in 1..4 {
                let qx = inner_x + (inner_w * q) / 4;
                let div_px = fb_full.pixels[sample_y * fb_full.w + qx] & 0x00ff_ffff;
                assert_ne!(div_px, 0x00f0_6434, "Missing 25% divider #{q} at scale={scale}");
            }

            // Bright active tip (0x00ff_d24c) appears at half health.
            let half_fill = (((inner_w as f32) * 0.5).round() as usize).clamp(scale, inner_w);
            let tip_px = fb_half.pixels[sample_y * fb_half.w + inner_x + half_fill - scale] & 0x00ff_ffff;
            assert_eq!(tip_px, 0x00ff_d24c, "Missing bright active tip at scale={scale}");

            // Multi-pass outer glow halo outside housing when hp > 0, absent when hp == 0.
            let halo_top = fb_full.pixels[(panel_y - 3 * scale) * fb_full.w + (300 * scale)];
            let halo_bot = fb_full.pixels[(panel_y + panel_h + 2 * scale) * fb_full.w + (300 * scale)];
            assert!(((halo_top >> 16) & 0xff) > 20 && ((halo_top >> 16) & 0xff) > (halo_top & 0xff));
            assert!(((halo_bot >> 16) & 0xff) > 20 && ((halo_bot >> 16) & 0xff) > (halo_bot & 0xff));
            assert_eq!(fb_zero.pixels[(panel_y - 3 * scale) * fb_zero.w + (300 * scale)] & 0x00ff_ffff, 0);
            assert_eq!(fb_zero.pixels[(panel_y + panel_h + 2 * scale) * fb_zero.w + (300 * scale)] & 0x00ff_ffff, 0);
            assert_eq!(panel_w, 116 * scale);
        }
    }
}
