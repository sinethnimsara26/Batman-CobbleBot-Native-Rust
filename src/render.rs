use crate::assets::{Assets,Image,LevelTileSet};
use crate::game::{AppScreen,Game,LEVEL_X,LEVEL_Y,LOGICAL_H,LOGICAL_W};
use crate::surface::RenderSurface;

pub const HQ_SCALE:usize=3;
pub const HQ_W:usize=LOGICAL_W*HQ_SCALE;
pub const HQ_H:usize=LOGICAL_H*HQ_SCALE;

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
                render_legacy(base,game,assets);
                upscale_opaque_base(base,target);
            }else{
                // Round 8 preserves the original depth split while removing
                // the last small moving objects from the blurry 1x world:
                // scenery at 1x -> promote -> true 3x pickups/Batarangs ->
                // true 3x Batman -> true 3x foreground UI/overlays.
                base.fill(0xff000000);
                // Round 10 keeps only the cheap depth-1 symbol 14 in
                // the promoted 1x base. Symbols 146 and 147 interleave in the
                // original Flash display list, so 144-147 stay together in
                // one true-3x tile layer; that layer is copied 1:1 before
                // moving objects and Batman.
                render_backdrop_legacy(base,game,assets);
                draw_tiles(base,game,&assets.level1a_tiles_hq_base);
                upscale_opaque_base(base,target);
                draw_tiles_hq(target,game,&assets.level1a_detail_tiles_hq);
                draw_world_objects_hq(target,game,assets);
                draw_batman_hq(target,game,assets);
                draw_foreground_hq(target,game,assets);
            }
        }
    }
}

pub fn upscale_opaque_base(src:&RenderSurface,dst:&mut RenderSurface){
    assert_eq!(dst.w,src.w*HQ_SCALE);
    assert_eq!(dst.h,src.h*HQ_SCALE);
    assert_eq!(HQ_SCALE,3,"Round 3 bilinear kernel is specialized for 3x");

    // Pixel-center bilinear resampling. For an exact 3x scale, destination
    // pixel (3*x+1,3*y+1) lands exactly on source pixel (x,y), which gives us
    // a strong no-drift CI invariant while smoothing the two pixels between
    // neighboring logical samples.
    //
    // Precompute the horizontal lookup/weights once. The original proof
    // kernel recomputed div_euclid/clamping for every one of the 2.16M output
    // pixels; this keeps those divisions out of the hot inner loop while
    // preserving the exact same integer-weighted result.
    let mut xmap=Vec::with_capacity(dst.w);
    for dx in 0..dst.w{
        let xn=dx as isize-1;
        let xq=xn.div_euclid(3);
        let xr=xn.rem_euclid(3) as u32;
        let x0=xq.clamp(0,src.w as isize-1) as usize;
        let x1=(xq+1).clamp(0,src.w as isize-1) as usize;
        xmap.push((x0,x1,3-xr,xr));
    }

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

        for (dx,&(x0,x1,wx0,wx1)) in xmap.iter().enumerate(){
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

fn render_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    fb.fill(0xff000000);
    match game.screen{
        AppScreen::Title=>{blit(fb,&assets.title_screen,0,0,600,400,false);return},
        AppScreen::Instructions=>{blit(fb,&assets.instructions_screen,0,0,600,400,false);return},
        AppScreen::Playing=>{}
    }
    render_world_legacy(fb,game,assets);
    draw_batman_legacy(fb,game,assets);
    draw_foreground_legacy(fb,game,assets);
}

fn render_world_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_world_base_legacy(fb,game,assets);
    draw_world_objects_legacy(fb,game,assets);
}

fn render_world_base_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    render_backdrop_legacy(fb,game,assets);
    draw_tiles(fb,game,&assets.level1a_tiles);
}

fn render_backdrop_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
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
    blit(fb,&assets.city_background,bg_x,bg_y,bg_w,bg_h,false);
}

fn draw_world_objects_legacy(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    if !assets.pickup_frames.is_empty() {
        let pf=&assets.pickup_frames[(game.ticks as usize)%assets.pickup_frames.len()];
        for (i,&(wx,wy)) in crate::game::PICKUPS.iter().enumerate() {
            if game.pickups[i] { continue; }
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
    let scale=HQ_SCALE as f32;
    debug_assert!((assets.pickup_frames_hq.logical_pixel_scale-scale).abs()<0.001);
    debug_assert!((assets.batarang_frames_hq.logical_pixel_scale-scale).abs()<0.001);

    if !assets.pickup_frames_hq.is_empty() {
        let pf=&assets.pickup_frames_hq[(game.ticks as usize)%assets.pickup_frames_hq.len()];
        for (i,&(wx,wy)) in crate::game::PICKUPS.iter().enumerate() {
            if game.pickups[i] { continue; }
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
    let sx=(game.player.x+LEVEL_X+game.camera_x).round() as i32;
    let sy=(game.player.y+LEVEL_Y+game.camera_y).round() as i32;
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
    let sx=(game.player.x+LEVEL_X+game.camera_x).round() as i32*HQ_SCALE as i32;
    let sy=(game.player.y+LEVEL_Y+game.camera_y).round() as i32*HQ_SCALE as i32;
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

    draw_root_timeline_lazy_legacy(fb,&assets.level_title_frames,game.ticks,318.7,76.75);
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

    draw_root_timeline_hq_lazy(fb,&assets.level_title_frames_hq,game.ticks,318.7,76.75);
    draw_root_timeline_hq_lazy(fb,&assets.fadein_frames_hq,game.ticks,301.0,205.0);
    if let Some(t)=game.fadeout_tick{
        draw_root_timeline_hq_lazy(fb,&assets.fadeout_frames_hq,t as u64,301.0,205.0);
    }
}

fn draw_hud_hq(fb:&mut RenderSurface,game:&Game,assets:&Assets){
    const ROOT_X:f32=159.0;
    const ROOT_Y:f32=-11.0;
    const LEGACY_ANCHOR_X:f32=0.48106384;
    const LEGACY_ANCHOR_Y:f32=1.1601379;

    debug_assert!((assets.hud_shell_hq.logical_pixel_scale-HQ_SCALE as f32).abs()<0.001);
    debug_assert_eq!(assets.hud_shell_hq.len(),1);
    let shell=&assets.hud_shell_hq[0];
    let scale=assets.hud_shell_hq.logical_pixel_scale;
    let shell_x=(ROOT_X*scale-shell.anchor_x).round() as i32;
    let shell_y=(ROOT_Y*scale-shell.anchor_y).round() as i32;

    // Round 6 HUD assets already live at final presentation resolution.
    // Copy them 1:1; never enlarge the 1x HUD or digit atlases at runtime.
    blit(
        fb,&shell.image,
        shell_x,shell_y,
        shell.image.w as i32,shell.image.h as i32,false
    );

    // Preserve the exact legacy logical centers, then convert only at the
    // presentation boundary. This makes field registration independently
    // testable and keeps Game/HUD state in the original 600x400 coordinate system.
    let legacy_x=(ROOT_X-LEGACY_ANCHOR_X).round() as i32;
    let legacy_y=(ROOT_Y-LEGACY_ANCHOR_Y).round() as i32;
    let s=HQ_SCALE as i32;
    draw_number_centered_hq_1to1(
        fb,&assets.hud_digits_small_hq,
        (legacy_x+53)*s,(legacy_y+68)*s,game.batarangs.max(0)
    );
    draw_number_centered_hq_1to1(
        fb,&assets.hud_digits_large_hq,
        (legacy_x+228)*s,(legacy_y+47)*s,game.score.max(0)
    );
    draw_number_centered_hq_1to1(
        fb,&assets.hud_digits_small_hq,
        (legacy_x+155)*s,(legacy_y+69)*s,game.lives.max(0)
    );
}

fn draw_number_centered_hq_1to1(fb:&mut RenderSurface,atlas:&Image,cx:i32,cy:i32,value:i32){
    if atlas.w<10||atlas.h==0{return;}
    let cell_w=atlas.w/10;
    if cell_w==0{return;}
    let text=value.to_string();
    let total=(cell_w*text.len()) as i32;
    let mut x=cx-total/2;
    let y=cy-atlas.h as i32/2;
    for byte in text.bytes(){
        if !(b'0'..=b'9').contains(&byte){continue;}
        let digit=(byte-b'0') as usize;
        blit_region_1to1(
            fb,atlas,digit*cell_w,0,cell_w,atlas.h,x,y
        );
        x+=cell_w as i32;
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
    let text=value.to_string();
    let total=(cell_w*text.len()) as i32;
    let mut x=cx-total/2;
    let y=cy-atlas.h as i32/2;
    for byte in text.bytes(){
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

fn draw_root_timeline_lazy_legacy(
    fb:&mut RenderSurface,
    frames:&crate::assets::LazySpriteSet,
    tick:u64,x:f32,y:f32
){
    if frames.is_empty(){return;}
    debug_assert!((frames.logical_pixel_scale-1.0).abs()<0.001);
    let i=(tick as usize).min(frames.len()-1);
    let f=frames.frame(i);
    blit(
        fb,&f.image,
        (x-f.anchor_x).round() as i32,
        (y-f.anchor_y).round() as i32,
        f.image.w as i32,f.image.h as i32,false
    );
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
                blit(fb,&t.image,px,py,t.image.w as i32,t.image.h as i32,false);
            }
            x+=600;
        }
        y+=400;
    }
}
fn blend(dst:&mut u32,src:u32){let a=(src>>24)&255;if a==0{return}let sr=(src>>16)&255;let sg=(src>>8)&255;let sb=src&255;if a==255{*dst=(sr<<16)|(sg<<8)|sb;return}let inv=255-a;let dr=(*dst>>16)&255;let dg=(*dst>>8)&255;let db=*dst&255;*dst=(((sr*a+dr*inv+127)/255)<<16)|(((sg*a+dg*inv+127)/255)<<8)|((sb*a+db*inv+127)/255);}
fn blit(dst:&mut RenderSurface,src:&Image,x:i32,y:i32,rw:i32,rh:i32,flip:bool){if rw<=0||rh<=0{return}for oy in 0..rh{let dy=y+oy;if dy<0||dy>=dst.h as i32{continue}let sy=(oy as usize*src.h/rh as usize).min(src.h-1);for ox in 0..rw{let dx=x+ox;if dx<0||dx>=dst.w as i32{continue}let raw=(ox as usize*src.w/rw as usize).min(src.w-1);let sx=if flip{src.w-1-raw}else{raw};let sp=src.pixels[sy*src.w+sx];blend(&mut dst.pixels[dy as usize*dst.w+dx as usize],sp);}}}