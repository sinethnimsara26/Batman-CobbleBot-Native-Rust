use crate::assets::{Assets,Image,LevelTileSet};
use crate::game::{AppScreen,Game,LEVEL_X,LEVEL_Y,LOGICAL_H,LOGICAL_W};

pub fn render(fb:&mut[u32],game:&Game,assets:&Assets){
    fb.fill(0xff000000);
    match game.screen{
        AppScreen::Title=>{blit(fb,&assets.title_screen,0,0,600,400,false);return},
        AppScreen::Instructions=>{blit(fb,&assets.instructions_screen,0,0,600,400,false);return},
        AppScreen::Playing=>{}
    }
    // Original game-sprite placement for background symbol 142:
    // sx=1.029632568359375, sy=1.029754638671875, tx=0, ty=-13.1.
    // The baked PNG is native-bounds cropped; its top-left represents
    // symbol-space (-0.980480957, 66.1), so transform that crop origin too.
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
    draw_tiles(fb,game,&assets.level1a_tiles);

    // Original collectible Batarangs (symbol 695) live in game-space, not
    // inside the level sprite, so render them independently like the SWF does.
    if !assets.pickup_frames.is_empty() {
        let pf=&assets.pickup_frames[(game.ticks as usize)%assets.pickup_frames.len()];
        for (i,&(wx,wy)) in crate::game::PICKUPS.iter().enumerate() {
            if game.pickups[i] { continue; }
            let px=(wx+game.camera_x-pf.anchor_x).round() as i32;
            let py=(wy+game.camera_y-pf.anchor_y).round() as i32;
            blit(fb,&pf.image,px,py,pf.image.w as i32,pf.image.h as i32,false);
        }
    }

    // Thrown Batarang projectile (symbol 694).
    if !assets.batarang_frames.is_empty() {
        let bf=&assets.batarang_frames[(game.ticks as usize)%assets.batarang_frames.len()];
        for shot in &game.shots {
            let px=(shot.x+game.camera_x-bf.anchor_x).round() as i32;
            let py=(shot.y+game.camera_y-bf.anchor_y).round() as i32;
            blit(fb,&bf.image,px,py,bf.image.w as i32,bf.image.h as i32,shot.dir<0);
        }
    }

    let sx=(game.player.x+LEVEL_X+game.camera_x).round() as i32;
    let sy=(game.player.y+LEVEL_Y+game.camera_y).round() as i32;
    let frame=&assets.batman_frames[game.animation_frame()]; let flip=game.player.dir<0;
    let ax=if flip{frame.image.w as f32-frame.anchor_x}else{frame.anchor_x};
    blit(fb,&frame.image,sx-ax.round() as i32,sy-frame.anchor_y.round() as i32,frame.image.w as i32,frame.image.h as i32,flip);

    // Root depth 1443: original HUD symbol 716.
    draw_hud(fb,game,assets);

    // Root depth 1490: original tutorial overlay symbol 737.
    // Its frame is controlled by the recovered checkpoint scripts.
    if !assets.tutorial_overlay_frames.is_empty(){
        let i=game.overlay_frame.min(assets.tutorial_overlay_frames.len()-1);
        let f=&assets.tutorial_overlay_frames[i];
        blit(
            fb,&f.image,
            (311.4-f.anchor_x).round() as i32,
            (190.0-f.anchor_y).round() as i32,
            f.image.w as i32,f.image.h as i32,false
        );
    }

    // Root frame 19 places the original level-title clip at (318.7,76.75)
    // and the original fade-in clip above it at (301,205). Both child
    // timelines play at the movie's 25 Hz and Stop() on their final frame.
    draw_root_timeline(fb,&assets.level_title_frames,game.ticks,318.7,76.75);
    draw_root_timeline(fb,&assets.fadein_frames,game.ticks,301.0,205.0);
    if let Some(t)=game.fadeout_tick{
        draw_root_timeline(fb,&assets.fadeout_frames,t as u64,301.0,205.0);
    }
}
fn draw_hud(fb:&mut[u32],game:&Game,assets:&Assets){
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

fn draw_number_centered(fb:&mut[u32],atlas:&Image,cx:i32,cy:i32,value:i32){
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

fn blit_region(
    dst:&mut[u32],src:&Image,
    sx0:usize,sy0:usize,sw:usize,sh:usize,
    x:i32,y:i32,rw:i32,rh:i32
){
    if sw==0||sh==0||rw<=0||rh<=0{return;}
    for oy in 0..rh{
        let dy=y+oy;
        if dy<0||dy>=LOGICAL_H as i32{continue;}
        let sy=sy0+(oy as usize*sh/rh as usize).min(sh-1);
        for ox in 0..rw{
            let dx=x+ox;
            if dx<0||dx>=LOGICAL_W as i32{continue;}
            let sx=sx0+(ox as usize*sw/rw as usize).min(sw-1);
            if sx>=src.w||sy>=src.h{continue;}
            let sp=src.pixels[sy*src.w+sx];
            blend(&mut dst[dy as usize*LOGICAL_W+dx as usize],sp);
        }
    }
}

fn draw_root_timeline(fb:&mut[u32],frames:&[crate::assets::SpriteFrame],tick:u64,x:f32,y:f32){
    if frames.is_empty(){return;}
    let i=(tick as usize).min(frames.len()-1);
    let f=&frames[i];
    blit(
        fb,&f.image,
        (x-f.anchor_x).round() as i32,
        (y-f.anchor_y).round() as i32,
        f.image.w as i32,f.image.h as i32,false
    );
}
fn draw_tiles(fb:&mut[u32],game:&Game,tiles:&LevelTileSet){
    let fw=(( -game.camera_x/600.0).floor() as i32)*600; let lw=(((600.0-game.camera_x)/600.0).floor() as i32)*600;
    let fh=(( -game.camera_y/400.0).floor() as i32)*400; let lh=(((400.0-game.camera_y)/400.0).floor() as i32)*400;
    let mut y=fh;while y<=lh{let mut x=fw;while x<=lw{if let Some(t)=tiles.tile(x,y){blit(fb,&t,(x as f32+game.camera_x).round() as i32,(y as f32+game.camera_y).round() as i32,600,400,false)}x+=600;}y+=400;}
}
fn blend(dst:&mut u32,src:u32){let a=(src>>24)&255;if a==0{return}let sr=(src>>16)&255;let sg=(src>>8)&255;let sb=src&255;if a==255{*dst=(sr<<16)|(sg<<8)|sb;return}let inv=255-a;let dr=(*dst>>16)&255;let dg=(*dst>>8)&255;let db=*dst&255;*dst=(((sr*a+dr*inv+127)/255)<<16)|(((sg*a+dg*inv+127)/255)<<8)|((sb*a+db*inv+127)/255);}
fn blit(dst:&mut[u32],src:&Image,x:i32,y:i32,rw:i32,rh:i32,flip:bool){if rw<=0||rh<=0{return}for oy in 0..rh{let dy=y+oy;if dy<0||dy>=LOGICAL_H as i32{continue}let sy=(oy as usize*src.h/rh as usize).min(src.h-1);for ox in 0..rw{let dx=x+ox;if dx<0||dx>=LOGICAL_W as i32{continue}let raw=(ox as usize*src.w/rw as usize).min(src.w-1);let sx=if flip{src.w-1-raw}else{raw};let sp=src.pixels[sy*src.w+sx];blend(&mut dst[dy as usize*LOGICAL_W+dx as usize],sp);}}}