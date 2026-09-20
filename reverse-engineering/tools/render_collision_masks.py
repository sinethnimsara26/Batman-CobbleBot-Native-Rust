#!/usr/bin/env python3
"""Rasterize the original SWF collision `ground` symbols to 1-bit PNG masks.

The game uses Flash shape hitTest() against game.level.ground. This tool parses
DefineShape/2/3 geometry directly and recursively applies the transforms inside
the ground movie clip plus the level's named `ground` placement. Curves are
sampled finely enough for the game's pixel-scale point probes.

Output masks are *level-space* pixels with origin recorded in manifest.json.
Requires Pillow only for PNG writing/filling; the parser itself is local.
"""
from __future__ import annotations
from pathlib import Path
import argparse, json, math, struct
from PIL import Image, ImageDraw
from swf_re import Bits, read_rect, read_matrix, tags

SHAPE_TAGS={2,22,32,83}

def u8(b,p): return b[p],p+1
def ui16(b,p): return struct.unpack_from('<H',b,p)[0],p+2

def skip_color(b,p,alpha): return p+(4 if alpha else 3)

def skip_gradient(b,p,alpha):
    n=b[p]&15; p+=1
    for _ in range(n): p+=1; p=skip_color(b,p,alpha)
    return p

def skip_fill_style(b,p,alpha):
    typ=b[p];p+=1
    if typ==0x00: return skip_color(b,p,alpha)
    if typ in (0x10,0x12,0x13):
        _,p=read_matrix(b,p); return skip_gradient(b,p,alpha)
    if typ in (0x40,0x41,0x42,0x43):
        p+=2; _,p=read_matrix(b,p); return p
    raise ValueError(f'unsupported fill style 0x{typ:02x}')

def skip_fill_array(b,p,alpha):
    n=b[p];p+=1
    if n==0xff: n=struct.unpack_from('<H',b,p)[0];p+=2
    for _ in range(n): p=skip_fill_style(b,p,alpha)
    return p

def skip_line_array(b,p,alpha,shape4=False):
    n=b[p];p+=1
    if n==0xff: n=struct.unpack_from('<H',b,p)[0];p+=2
    for _ in range(n):
        p+=2
        if shape4:
            bits=Bits(b,p); bits.u(2); join=bits.u(2); has_fill=bits.u(1)
            bits.u(1); bits.u(1); bits.u(1); bits.u(5); bits.u(1); bits.u(2); bits.align(); p=bits.pos
            if join==2: p+=2
            if has_fill: p=skip_fill_style(b,p,alpha)
            else: p=skip_color(b,p,alpha)
        else: p=skip_color(b,p,alpha)
    return p

def flush_paths(active,paths,style=None):
    keys=list(active) if style is None else [style]
    for key in keys:
        contour=active.pop(key,None)
        if contour and len(contour)>=3: paths.setdefault(key,[]).append(contour)

def append_fill_edge(active,paths,style,edge,reverse=False):
    if style==0: return
    contour=list(reversed(edge)) if reverse else edge
    key=style-1
    current=active.get(key)
    if current is None:
        active[key]=list(contour)
    elif reverse and contour[-1]==current[0]:
        # FillStyle0 edges bound the same filled region in the opposite
        # direction. Prepend each reversed edge to reconnect its contour;
        # reversing edges while appending them breaks every multi-edge path.
        contour.extend(current[1:])
        active[key]=list(contour)
    elif current[-1]==contour[0]:
        current.extend(contour[1:])
    else:
        flush_paths(active,paths,key)
        active[key]=list(contour)

def parse_shape_paths(payload:bytes,tag:int):
    # payload starts with ShapeId, RECT, ShapeWithStyle.
    _,p=ui16(payload,0); _,p=read_rect(payload,p); alpha=(tag in (32,83)); even_odd=True
    if tag==83:
        _,p=read_rect(payload,p)
        uses_fill_winding=bool(payload[p]&0x04)
        even_odd=not uses_fill_winding
        p+=1
    p=skip_fill_array(payload,p,alpha); p=skip_line_array(payload,p,alpha,tag==83)
    br=Bits(payload,p); nfill=br.u(4); nline=br.u(4)
    x=y=0; fill0=fill1=lineidx=0; paths={}; active={}
    while True:
        typ=br.u(1)
        if typ:
            straight=br.u(1); n=br.u(4)+2
            x0,y0=x,y
            if straight:
                general=br.u(1)
                if general: dx=br.s(n); dy=br.s(n)
                else:
                    vert=br.u(1)
                    if vert: dx=0;dy=br.s(n)
                    else: dx=br.s(n);dy=0
                x+=dx;y+=dy
                pts=[(x0,y0),(x,y)]
            else:
                cdx=br.s(n);cdy=br.s(n); adx=br.s(n);ady=br.s(n)
                cx=x+cdx;cy=y+cdy; ex=cx+adx;ey=cy+ady
                # Quadratic Bezier. Sampling density depends on approximate length.
                length=math.hypot(cx-x,cy-y)+math.hypot(ex-cx,ey-cy)
                steps=max(2,min(32,int(length/80)+1))
                pts=[]
                for i in range(steps+1):
                    t=i/steps; u=1-t
                    pts.append((u*u*x+2*u*t*cx+t*t*ex,u*u*y+2*u*t*cy+t*t*ey))
                x,y=ex,ey
            append_fill_edge(active,paths,fill1,pts)
            if fill0!=fill1: append_fill_edge(active,paths,fill0,pts,True)
        else:
            newstyles=br.u(1); state_line=br.u(1); state_fill1=br.u(1); state_fill0=br.u(1); move=br.u(1)
            if not (newstyles or state_line or state_fill1 or state_fill0 or move):
                flush_paths(active,paths); break
            if move:
                flush_paths(active,paths); n=br.u(5); x=br.s(n); y=br.s(n)
            if state_fill0:
                flush_paths(active,paths,fill0-1 if fill0 else -1)
                fill0=br.u(nfill)
            if state_fill1:
                flush_paths(active,paths,fill1-1 if fill1 else -1)
                fill1=br.u(nfill)
            if state_line:
                lineidx=br.u(nline); flush_paths(active,paths)
            if newstyles:
                br.align(); pp=br.pos; pp=skip_fill_array(payload,pp,alpha); pp=skip_line_array(payload,pp,alpha,tag==83); br.bit=pp*8; nfill=br.u(4); nline=br.u(4)
    return paths,even_odd

def rasterize_winding(mask, contours, origin_x, origin_y, even_odd=False):
    """Paint one SWF fill using its declared non-zero or even-odd rule."""
    width, height = mask.size
    edges=[]
    for contour in contours:
        if len(contour)<3: continue
        points=list(contour)
        for (x1,y1),(x2,y2) in zip(points,points[1:]+points[:1]):
            if y1!=y2: edges.append((x1,y1,x2,y2))
    if not edges: return
    min_y=max(0,math.ceil(min(min(y1,y2) for _,y1,_,y2 in edges)-origin_y-0.5))
    max_y=min(height,math.ceil(max(max(y1,y2) for _,y1,_,y2 in edges)-origin_y-0.5))
    draw=ImageDraw.Draw(mask)
    for py in range(min_y,max_y):
        scan_y=origin_y+py+0.5
        crossings=[]
        for x1,y1,x2,y2 in edges:
            if y1<=scan_y<y2:
                delta=-1
            elif y2<=scan_y<y1:
                delta=1
            else:
                continue
            x=x1+(scan_y-y1)*(x2-x1)/(y2-y1)-origin_x
            crossings.append((x,delta))
        crossings.sort(key=lambda item:item[0])
        winding=0
        start=None
        i=0
        while i<len(crossings):
            x=crossings[i][0]
            delta=0
            while i<len(crossings) and abs(crossings[i][0]-x)<1e-8:
                delta+=crossings[i][1]
                i+=1
            previous=winding
            winding=(winding+delta)%2 if even_odd else winding+delta
            if previous==0 and winding!=0:
                start=x
            elif previous!=0 and winding==0 and start is not None:
                left=max(0,math.ceil(start-0.5))
                right=min(width,math.ceil(x-0.5))
                if left<right: draw.line((left,py,right-1,py),fill=1)
                start=None
    # SWF point hit-tests can land exactly on an edge; include the rounded
    # contour boundary pixels as Pillow's former polygon fill did.
    for contour in contours:
        points=[(round(x-origin_x),round(y-origin_y)) for x,y in contour]
        if len(points)>=3: draw.line(points+[points[0]],fill=1,width=1)

def affine(m):
    return (m.get('sx',1.0),m.get('r1',0.0),m.get('r0',0.0),m.get('sy',1.0),m.get('tx',0.0),m.get('ty',0.0))
def compose(a,b):
    # a(b(point))
    A,B,C,D,E,F=a; a2,b2,c2,d2,e2,f2=b
    return (A*a2+B*c2,A*b2+B*d2,C*a2+D*c2,C*b2+D*d2,A*e2+B*f2+E,C*e2+D*f2+F)
def point(m,p):
    A,B,C,D,E,F=m; x,y=p; return (A*x+B*y+E,C*x+D*y+F)
I=(1.,0.,0.,1.,0.,0.)

def load_defs(swf):
    b=Path(swf).read_bytes(); _,p=read_rect(b,8);p+=4; shapes={}
    for code,pl,toff,poff in tags(b,p,len(b)):
        if code in SHAPE_TAGS:
            cid=struct.unpack_from('<H',pl,0)[0]; shapes[cid]=(code,pl)
    return shapes

def collect_paths(cid,m,symbols,shapes,depth=0):
    if depth>12: raise RuntimeError('sprite recursion too deep')
    if cid in shapes:
        code,pl=shapes[cid]; result=[]
        shape_paths,even_odd=parse_shape_paths(pl,code)
        for contours in shape_paths.values():
            transformed=[]
            for path in contours:
                transformed.append([point(m,(x/20.0,y/20.0)) for x,y in path])
            if transformed: result.append((transformed,even_odd))
        return result
    s=symbols.get(str(cid))
    if not s: return []
    out=[]
    # Ground clips in this title are single-frame; only frame-0 placements matter.
    for p in s.get('placements',[]):
        if p.get('frame',0)!=0 or 'character_id' not in p: continue
        child=compose(m,affine(p.get('matrix',{})))
        out.extend(collect_paths(p['character_id'],child,symbols,shapes,depth+1))
    return out

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('swf',type=Path); ap.add_argument('symbols_json',type=Path); ap.add_argument('levels_json',type=Path); ap.add_argument('outdir',type=Path); ap.add_argument('--level',action='append',help='only bake the named level; may be repeated'); ns=ap.parse_args()
    symbols=json.loads(ns.symbols_json.read_text()); levels=json.loads(ns.levels_json.read_text()); shapes=load_defs(ns.swf); ns.outdir.mkdir(parents=True,exist_ok=True); manifest=[]
    for name,lev in levels.items():
        if ns.level and name not in ns.level: continue
        g=lev.get('ground')
        if not g: manifest.append({'level':name,'error':'no named ground placement'}); continue
        paths=collect_paths(g['character_id'],affine(g.get('matrix',{})),symbols,shapes)
        pts=[p for group,_ in paths for contour in group for p in contour]
        if not pts: manifest.append({'level':name,'ground_character_id':g['character_id'],'error':'no geometry'}); continue
        minx=math.floor(min(x for x,y in pts))-2; miny=math.floor(min(y for x,y in pts))-2; maxx=math.ceil(max(x for x,y in pts))+2; maxy=math.ceil(max(y for x,y in pts))+2
        w=maxx-minx+1; h=maxy-miny+1
        # Safety: these are level-space collision masks, not full scene screenshots.
        if w*h>120_000_000: raise RuntimeError(f'{name} mask unexpectedly huge: {w}x{h}')
        im=Image.new('1',(w,h),0)
        for group,even_odd in paths:
            rasterize_winding(im,group,minx,miny,even_odd)
        fn=f'{name}_ground.png'; im.save(ns.outdir/fn,optimize=True)
        contour_count=sum(len(group) for group,_ in paths)
        manifest.append({'level':name,'file':fn,'ground_character_id':g['character_id'],'origin_level_px':[minx,miny],'size_px':[w,h],'fill_groups':len(paths),'contours':contour_count,'occupied_pixels':sum(im.histogram()[1:])})
        print(name, w,h,'fill_groups',len(paths),'contours',contour_count)
    (ns.outdir/'manifest.json').write_text(json.dumps(manifest,indent=2)+"\n")
if __name__=='__main__': main()