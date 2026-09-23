#![allow(non_snake_case)]
use crate::assets::Assets;
use crate::audio::Audio;
use crate::collision::CollisionMask;
use crate::game::{Game,InputState,FIXED_STEP,LOGICAL_H,LOGICAL_W,VK_A,VK_D,VK_DOWN,VK_ENTER,VK_ESCAPE,VK_I,VK_LEFT,VK_RIGHT,VK_S,VK_SPACE,VK_UP};
use crate::render;
use crate::surface::RenderSurface;
use std::ffi::c_void;
use std::mem::{size_of,transmute,zeroed};
use std::ptr::{null,null_mut};
use std::time::Instant;

const WM_DESTROY:u32=0x0002; const WM_SETFOCUS:u32=0x0007; const WM_KILLFOCUS:u32=0x0008; const WM_PAINT:u32=0x000F; const WM_CLOSE:u32=0x0010; const WM_ERASEBKGND:u32=0x0014; const WM_ACTIVATE:u32=0x0006; const WM_LBUTTONDOWN:u32=0x0201; const WM_TIMER:u32=0x0113; const WM_DPICHANGED:u32=0x02E0;
const CS_HREDRAW:u32=2; const CS_VREDRAW:u32=1; const IDC_ARROW:*const u16=32512usize as *const u16; const WS_OVERLAPPEDWINDOW:u32=0x00CF0000; const WS_VISIBLE:u32=0x10000000; const CW_USEDEFAULT:i32=i32::MIN; const SW_SHOW:i32=5; const SRCCOPY:u32=0x00CC0020; const BLACKNESS:u32=0x00000042; const DIB_RGB_COLORS:u32=0; const BI_RGB:u32=0; const HALFTONE:i32=4; const WA_INACTIVE:usize=0; const SWP_NOZORDER:u32=0x0004; const SWP_NOACTIVATE:u32=0x0010; const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2:isize=-4;
type HWND=*mut c_void; type HINSTANCE=*mut c_void; type HICON=*mut c_void; type HCURSOR=*mut c_void; type HBRUSH=*mut c_void; type HDC=*mut c_void; type WPARAM=usize; type LPARAM=isize; type LRESULT=isize;
#[repr(C)]struct POINT{x:i32,y:i32} #[repr(C)]struct RECT{left:i32,top:i32,right:i32,bottom:i32}
#[repr(C)]struct MSG{hwnd:HWND,message:u32,wParam:WPARAM,lParam:LPARAM,time:u32,pt:POINT,lPrivate:u32}
#[repr(C)]struct PAINTSTRUCT{hdc:HDC,fErase:i32,rcPaint:RECT,fRestore:i32,fIncUpdate:i32,rgbReserved:[u8;32]}
#[repr(C)]struct WNDCLASSW{style:u32,lpfnWndProc:Option<unsafe extern "system" fn(HWND,u32,WPARAM,LPARAM)->LRESULT>,cbClsExtra:i32,cbWndExtra:i32,hInstance:HINSTANCE,hIcon:HICON,hCursor:HCURSOR,hbrBackground:HBRUSH,lpszMenuName:*const u16,lpszClassName:*const u16}
#[repr(C)]struct BITMAPINFOHEADER{biSize:u32,biWidth:i32,biHeight:i32,biPlanes:u16,biBitCount:u16,biCompression:u32,biSizeImage:u32,biXPelsPerMeter:i32,biYPelsPerMeter:i32,biClrUsed:u32,biClrImportant:u32}
#[repr(C)]struct RGBQUAD{rgbBlue:u8,rgbGreen:u8,rgbRed:u8,rgbReserved:u8} #[repr(C)]struct BITMAPINFO{bmiHeader:BITMAPINFOHEADER,bmiColors:[RGBQUAD;1]}
#[link(name="user32")]extern "system"{
 fn RegisterClassW(c:*const WNDCLASSW)->u16; fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,parent:HWND,menu:*mut c_void,inst:HINSTANCE,param:*mut c_void)->HWND; fn DefWindowProcW(hwnd:HWND,msg:u32,w:WPARAM,l:LPARAM)->LRESULT;
 fn ShowWindow(hwnd:HWND,cmd:i32)->i32; fn UpdateWindow(hwnd:HWND)->i32; fn GetMessageW(msg:*mut MSG,hwnd:HWND,min:u32,max:u32)->i32; fn TranslateMessage(msg:*const MSG)->i32; fn DispatchMessageW(msg:*const MSG)->LRESULT; fn PostQuitMessage(code:i32);
 fn BeginPaint(hwnd:HWND,ps:*mut PAINTSTRUCT)->HDC; fn EndPaint(hwnd:HWND,ps:*const PAINTSTRUCT)->i32; fn GetClientRect(hwnd:HWND,r:*mut RECT)->i32; fn InvalidateRect(hwnd:HWND,r:*const RECT,erase:i32)->i32; fn LoadCursorW(inst:HINSTANCE,name:*const u16)->HCURSOR;
 fn SetTimer(hwnd:HWND,id:usize,ms:u32,proc_:*const c_void)->usize; fn KillTimer(hwnd:HWND,id:usize)->i32; fn DestroyWindow(hwnd:HWND)->i32; fn SetProcessDPIAware()->i32; fn SetWindowPos(hwnd:HWND,after:HWND,x:i32,y:i32,cx:i32,cy:i32,flags:u32)->i32;
 fn SetForegroundWindow(hwnd:HWND)->i32; fn SetActiveWindow(hwnd:HWND)->HWND; fn SetFocus(hwnd:HWND)->HWND; fn GetForegroundWindow()->HWND; fn GetAsyncKeyState(vkey:i32)->i16;
}
#[link(name="gdi32")]extern "system"{fn StretchDIBits(hdc:HDC,x:i32,y:i32,dw:i32,dh:i32,sx:i32,sy:i32,sw:i32,sh:i32,bits:*const c_void,bmi:*const BITMAPINFO,usage:u32,rop:u32)->i32;fn PatBlt(hdc:HDC,x:i32,y:i32,w:i32,h:i32,rop:u32)->i32;fn SetStretchBltMode(hdc:HDC,mode:i32)->i32;fn SetBrushOrgEx(hdc:HDC,x:i32,y:i32,prev:*mut POINT)->i32;}
#[link(name="kernel32")]extern "system"{fn GetModuleHandleW(name:*const u16)->HINSTANCE;fn GetProcAddress(module:HINSTANCE,name:*const u8)->*mut c_void;}

const KEYS:[usize;10]=[VK_LEFT,VK_RIGHT,VK_UP,VK_DOWN,VK_SPACE,VK_ENTER,VK_ESCAPE,VK_I,VK_A,VK_D];
const EXTRA_KEYS:[usize;1]=[VK_S];

struct App{base_fb:RenderSurface,fb:RenderSurface,assets:Assets,audio:Audio,ground:CollisionMask,game:Game,input:InputState,last:Instant,acc:f32,active:bool}
impl App{
 fn new()->Self{Self{base_fb:RenderSurface::new(LOGICAL_W,LOGICAL_H),fb:RenderSurface::new(render::HQ_W,render::HQ_H),assets:Assets::load(),audio:Audio::new(),ground:CollisionMask::level1a(),game:Game::new(),input:InputState::default(),last:Instant::now(),acc:0.0,active:true}}
 unsafe fn poll_keyboard(&mut self,hwnd:HWND){
  // WM_ACTIVATE / WM_SETFOCUS are authoritative. GetForegroundWindow is
  // allowed to promote us to active, but a transient mismatch must not undo
  // a focus message and recreate the old "click around before keys work" bug.
  if GetForegroundWindow()==hwnd{self.active=true;}
  if !self.active{self.input.release_all();return;}
  for &k in KEYS.iter().chain(EXTRA_KEYS.iter()){
   self.input.set(k,(GetAsyncKeyState(k as i32) as u16&0x8000)!=0);
  }
 }
 fn timer(&mut self,hwnd:HWND)->bool{unsafe{self.poll_keyboard(hwnd);}let now=Instant::now();let dt=(now-self.last).as_secs_f32().min(0.2);self.last=now;if !self.active{return false;}self.acc+=dt;let mut stepped=false;while self.acc>=FIXED_STEP{self.game.tick(&mut self.input,&self.ground);for event in self.game.take_audio_events(){self.audio.handle(event);}self.acc-=FIXED_STEP;stepped=true;}stepped}
 fn render(&mut self){render::render_high_quality(&mut self.base_fb,&mut self.fb,&self.game,&self.assets);}
}
static mut APP:*mut App=null_mut();
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}

unsafe fn enable_best_dpi_awareness()->bool{
 // SetProcessDpiAwarenessContext was added after the legacy API. Resolve it
 // dynamically so older Windows versions can still start and use the fallback.
 let user32_name=wide("user32.dll");
 let user32=GetModuleHandleW(user32_name.as_ptr());
 if !user32.is_null(){
  let proc=GetProcAddress(user32,b"SetProcessDpiAwarenessContext\0".as_ptr());
  if !proc.is_null(){
   type SetProcessDpiAwarenessContextFn=unsafe extern "system" fn(*mut c_void)->i32;
   let set_context:SetProcessDpiAwarenessContextFn=transmute(proc);
   let context=DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 as *mut c_void;
   if set_context(context)!=0{return true;}
  }
 }
 SetProcessDPIAware()!=0
}

fn viewport_for_client(cw:i32,ch:i32)->(i32,i32,i32,i32){
 let cw=cw.max(1);let ch=ch.max(1);
 let(dw,dh)=if(cw as i64)*(LOGICAL_H as i64)<=(ch as i64)*(LOGICAL_W as i64){
  (cw,((cw as i64)*(LOGICAL_H as i64)/(LOGICAL_W as i64))as i32)
 }else{
  (((ch as i64)*(LOGICAL_W as i64)/(LOGICAL_H as i64))as i32,ch)
 };
 ((cw-dw)/2,(ch-dh)/2,dw,dh)
}
unsafe fn viewport(hwnd:HWND)->(i32,i32,i32,i32){let mut r:RECT=zeroed();GetClientRect(hwnd,&mut r);viewport_for_client(r.right-r.left,r.bottom-r.top)}
unsafe extern "system" fn wndproc(hwnd:HWND,msg:u32,w:WPARAM,l:LPARAM)->LRESULT{match msg{
 WM_ERASEBKGND=>1,
 WM_PAINT=>{let mut ps:PAINTSTRUCT=zeroed();let hdc=BeginPaint(hwnd,&mut ps);if !APP.is_null(){let app=&mut*APP;app.render();let mut r:RECT=zeroed();GetClientRect(hwnd,&mut r);let cw=(r.right-r.left).max(1);let ch=(r.bottom-r.top).max(1);let(vx,vy,vw,vh)=viewport(hwnd);if vy>0{PatBlt(hdc,0,0,cw,vy,BLACKNESS);PatBlt(hdc,0,vy+vh,cw,ch-vy-vh,BLACKNESS);}if vx>0{PatBlt(hdc,0,vy,vx,vh,BLACKNESS);PatBlt(hdc,vx+vw,vy,cw-vx-vw,vh,BLACKNESS);}let bmi=BITMAPINFO{bmiHeader:BITMAPINFOHEADER{biSize:size_of::<BITMAPINFOHEADER>()as u32,biWidth:app.fb.w as i32,biHeight:-(app.fb.h as i32),biPlanes:1,biBitCount:32,biCompression:BI_RGB,biSizeImage:app.fb.byte_len() as u32,biXPelsPerMeter:0,biYPelsPerMeter:0,biClrUsed:0,biClrImportant:0},bmiColors:[RGBQUAD{rgbBlue:0,rgbGreen:0,rgbRed:0,rgbReserved:0}]};/* Phase 1 HQ proof retained: high-quality final GDI presentation. Phase 2 makes source dimensions explicit without changing them yet. */SetStretchBltMode(hdc,HALFTONE);SetBrushOrgEx(hdc,0,0,null_mut());StretchDIBits(hdc,vx,vy,vw,vh,0,0,app.fb.w as i32,app.fb.h as i32,app.fb.pixels.as_ptr()as*const c_void,&bmi,DIB_RGB_COLORS,SRCCOPY);}EndPaint(hwnd,&ps);0},
 WM_TIMER=>{if !APP.is_null()&&(&mut*APP).timer(hwnd){InvalidateRect(hwnd,null(),0);}0},
 WM_SETFOCUS=>{if !APP.is_null(){(&mut*APP).active=true;}0},
 WM_KILLFOCUS=>{if !APP.is_null(){let app=&mut*APP;app.active=false;app.input.release_all();}0},
 WM_ACTIVATE=>{if !APP.is_null(){let active=(w&0xffff)!=WA_INACTIVE;let app=&mut*APP;app.active=active;if !active{app.input.release_all();}}0},
 WM_LBUTTONDOWN=>{SetActiveWindow(hwnd);SetFocus(hwnd);0},
 WM_DPICHANGED=>{if l!=0{let r=&*(l as *const RECT);let width=(r.right-r.left).max(1);let height=(r.bottom-r.top).max(1);SetWindowPos(hwnd,null_mut(),r.left,r.top,width,height,SWP_NOZORDER|SWP_NOACTIVATE);}InvalidateRect(hwnd,null(),0);0},
 WM_CLOSE=>{DestroyWindow(hwnd);0}, WM_DESTROY=>{KillTimer(hwnd,1);PostQuitMessage(0);0}, _=>DefWindowProcW(hwnd,msg,w,l)}}

pub fn run(){unsafe{let _dpi_aware=enable_best_dpi_awareness();APP=Box::into_raw(Box::new(App::new()));let inst=GetModuleHandleW(null());let cls=wide("BatmanCobbleBotLevel1AFidelity");let title=wide("The Batman - CobbleBot Caper - Level 1A Fidelity Slice");let wc=WNDCLASSW{style:CS_HREDRAW|CS_VREDRAW,lpfnWndProc:Some(wndproc),cbClsExtra:0,cbWndExtra:0,hInstance:inst,hIcon:null_mut(),hCursor:LoadCursorW(null_mut(),IDC_ARROW),hbrBackground:null_mut(),lpszMenuName:null(),lpszClassName:cls.as_ptr()};if RegisterClassW(&wc)==0{return;}let hwnd=CreateWindowExW(0,cls.as_ptr(),title.as_ptr(),WS_OVERLAPPEDWINDOW|WS_VISIBLE,CW_USEDEFAULT,CW_USEDEFAULT,930,660,null_mut(),null_mut(),inst,null_mut());if hwnd.is_null(){return;}ShowWindow(hwnd,SW_SHOW);SetActiveWindow(hwnd);SetForegroundWindow(hwnd);SetFocus(hwnd);UpdateWindow(hwnd);SetTimer(hwnd,1,8,null());let mut msg:MSG=zeroed();while GetMessageW(&mut msg,null_mut(),0,0)>0{TranslateMessage(&msg);DispatchMessageW(&msg);}let _=Box::from_raw(APP);APP=null_mut();}}

#[cfg(test)]
mod tests{
 use super::viewport_for_client;

 #[test]
 fn dpi_scaled_3_by_2_clients_fill_exactly(){
  for &(w,h) in &[(900,600),(1125,750),(1350,900)]{
   assert_eq!(viewport_for_client(w,h),(0,0,w,h));
  }
 }

 #[test]
 fn dpi_scaled_nonmatching_clients_keep_centered_3_by_2_viewport(){
  for &(w,h) in &[(930,660),(1163,825),(1395,990)]{
   let(x,y,vw,vh)=viewport_for_client(w,h);
   assert!(x>=0&&y>=0);
   assert_eq!(vw as i64*2,vh as i64*3);
   assert!((w-vw-2*x).abs()<=1);
   assert!((h-vh-2*y).abs()<=1);
  }
 }
}
