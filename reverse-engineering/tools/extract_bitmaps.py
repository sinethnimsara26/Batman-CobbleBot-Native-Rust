#!/usr/bin/env python3
from pathlib import Path
import struct,zlib,io,json
from PIL import Image
from swf_re import read_rect,tags

def strip_jpeg_junk(x:bytes)->bytes:
    # Flash JPEG data sometimes starts with EOI/SOI boundary junk (FF D9 FF D8).
    if x.startswith(b'\xff\xd9\xff\xd8'): x=x[4:]
    return x

def merge_jpeg_tables(tables:bytes,jpeg:bytes)->bytes:
    tables=strip_jpeg_junk(tables); jpeg=strip_jpeg_junk(jpeg)
    # Make one valid JPEG: SOI + table body + image body, avoiding duplicate SOI/EOI.
    if tables.startswith(b'\xff\xd8'): tables=tables[2:]
    if tables.endswith(b'\xff\xd9'): tables=tables[:-2]
    if jpeg.startswith(b'\xff\xd8'): jpeg=jpeg[2:]
    return b'\xff\xd8'+tables+jpeg

def save_png(im:Image.Image,path:Path):
    path.parent.mkdir(parents=True,exist_ok=True); im.save(path,'PNG',optimize=True)

def decode_lossless(payload:bytes,has_alpha:bool):
    cid=struct.unpack_from('<H',payload,0)[0]; fmt=payload[2]; w,h=struct.unpack_from('<HH',payload,3); p=7
    ctable=None
    if fmt==3:
        n=payload[p]+1;p+=1
        raw=zlib.decompress(payload[p:]); ent=4 if has_alpha else 3
        pal=[]
        q=0
        for _ in range(n):
            if has_alpha:
                r,g,b,a=raw[q:q+4];q+=4
            else:
                r,g,b=raw[q:q+3];q+=3;a=255
            pal.append((r,g,b,a))
        stride=(w+3)&~3; idx=raw[q:q+stride*h]
        out=Image.new('RGBA',(w,h)); px=out.load()
        for y in range(h):
            for x in range(w): px[x,y]=pal[idx[y*stride+x]]
        return cid,out
    raw=zlib.decompress(payload[p:])
    out=Image.new('RGBA',(w,h)); px=out.load(); q=0
    if fmt==5: # 32-bit ARGB / XRGB
        for y in range(h):
            for x in range(w):
                a,r,g,b=raw[q:q+4];q+=4
                if not has_alpha:a=255
                px[x,y]=(r,g,b,a)
    elif fmt==4: # 15-bit RGB, rows padded to 32-bit
        stride=((w*2+3)//4)*4
        for y in range(h):
            row=y*stride
            for x in range(w):
                v=struct.unpack_from('<H',raw,row+x*2)[0]
                r=((v>>10)&31)*255//31; g=((v>>5)&31)*255//31; b=(v&31)*255//31
                px[x,y]=(r,g,b,255)
    else: raise ValueError(f'unsupported lossless format {fmt}')
    return cid,out

def main(src,outdir):
    b=Path(src).read_bytes(); out=Path(outdir);out.mkdir(parents=True,exist_ok=True)
    _,pos=read_rect(b,8);pos+=4
    jpeg_tables=b''; manifest=[]
    for code,pl,toff,poff in tags(b,pos,len(b)):
        try:
            if code==8: jpeg_tables=pl
            elif code==6:
                cid=struct.unpack_from('<H',pl,0)[0]; data=merge_jpeg_tables(jpeg_tables,pl[2:]); im=Image.open(io.BytesIO(data)).convert('RGBA'); save_png(im,out/f'{cid:04d}_jpeg.png'); manifest.append({'id':cid,'tag':code,'w':im.width,'h':im.height,'file':f'{cid:04d}_jpeg.png'})
            elif code==21:
                cid=struct.unpack_from('<H',pl,0)[0]; data=strip_jpeg_junk(pl[2:]); im=Image.open(io.BytesIO(data)).convert('RGBA'); save_png(im,out/f'{cid:04d}_jpeg2.png'); manifest.append({'id':cid,'tag':code,'w':im.width,'h':im.height,'file':f'{cid:04d}_jpeg2.png'})
            elif code==35:
                cid=struct.unpack_from('<H',pl,0)[0]; off=struct.unpack_from('<I',pl,2)[0]; jpeg=strip_jpeg_junk(pl[6:6+off]); alpha=zlib.decompress(pl[6+off:]); im=Image.open(io.BytesIO(jpeg)).convert('RGBA');
                if len(alpha)<im.width*im.height: raise ValueError(f'alpha short {len(alpha)} < {im.width*im.height}')
                im.putalpha(Image.frombytes('L',im.size,alpha[:im.width*im.height])); save_png(im,out/f'{cid:04d}_jpeg3.png'); manifest.append({'id':cid,'tag':code,'w':im.width,'h':im.height,'file':f'{cid:04d}_jpeg3.png'})
            elif code in (20,36):
                cid,im=decode_lossless(pl,code==36); save_png(im,out/f'{cid:04d}_lossless{2 if code==36 else 1}.png'); manifest.append({'id':cid,'tag':code,'w':im.width,'h':im.height,'file':f'{cid:04d}_lossless{2 if code==36 else 1}.png'})
        except Exception as e:
            manifest.append({'id':struct.unpack_from('<H',pl,0)[0] if len(pl)>=2 else None,'tag':code,'error':repr(e)})
    (out/'manifest.json').write_text(json.dumps(manifest,indent=2))
    ok=[x for x in manifest if 'file'in x]; bad=[x for x in manifest if 'error'in x]
    print('decoded',len(ok),'errors',len(bad));
    if bad:
        for x in bad: print('ERR',x)

if __name__=='__main__':
 import sys; main(sys.argv[1],sys.argv[2])
