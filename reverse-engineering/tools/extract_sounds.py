#!/usr/bin/env python3
"""Extract DefineSound assets and a sound manifest from an uncompressed SWF.

MP3 DefineSound records contain a signed 16-bit SeekSamples value followed by
MP3 frames; those frames are written as playable .mp3 files. SWF ADPCM records
are preserved losslessly as .swf-adpcm because transcoding is not required for
reverse engineering and would add a decoder dependency.
"""
from __future__ import annotations
from pathlib import Path
import argparse, json, struct
from swf_re import read_rect, tags, cstr

RATE_HZ={0:5512,1:11025,2:22050,3:44100}
FORMAT_NAME={0:'uncompressed-native-endian',1:'swf-adpcm',2:'mp3',3:'uncompressed-le',4:'nellymoser-16k',5:'nellymoser-8k',6:'nellymoser',11:'speex'}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('swf',type=Path); ap.add_argument('outdir',type=Path); ns=ap.parse_args()
    b=ns.swf.read_bytes()
    if b[:3]!=b'FWS': raise SystemExit('expected uncompressed FWS')
    _,pos=read_rect(b,8); pos+=4
    ns.outdir.mkdir(parents=True,exist_ok=True)
    exports={}; sounds=[]
    all_tags=list(tags(b,pos,len(b)))
    # First pass: collect export names so extracted filenames are stable.
    for code,pl,toff,poff in all_tags:
        if code==56:
            q=0; n=struct.unpack_from('<H',pl,q)[0]; q+=2
            for _ in range(n):
                cid=struct.unpack_from('<H',pl,q)[0]; q+=2; name,q=cstr(pl,q); exports[cid]=name
    # Second pass: extract DefineSound records.
    for code,pl,toff,poff in all_tags:
        if code!=14 or len(pl)<7: continue
        sid=struct.unpack_from('<H',pl,0)[0]; flags=pl[2]; fmt=flags>>4; rate=(flags>>2)&3; bits16=bool((flags>>1)&1); stereo=bool(flags&1); samples=struct.unpack_from('<I',pl,3)[0]; data=pl[7:]
        name=exports.get(sid,f'sound_{sid:04d}')
        rec={'id':sid,'export_name':name,'format':fmt,'format_name':FORMAT_NAME.get(fmt,f'format-{fmt}'),'rate_hz':RATE_HZ[rate],'bits_per_sample':16 if bits16 else 8,'channels':2 if stereo else 1,'sample_count':samples,'tag_offset':toff,'data_bytes':len(data)}
        safe=''.join(ch if ch.isalnum() or ch in ('-','_') else '_' for ch in name)
        if fmt==2:
            if len(data)<2: raise ValueError(f'MP3 sound {sid} missing SeekSamples')
            seek=struct.unpack_from('<h',data,0)[0]; mp3=data[2:]; fn=f'{sid:04d}_{safe}.mp3'; (ns.outdir/fn).write_bytes(mp3); rec.update(file=fn,seek_samples=seek,extracted_bytes=len(mp3))
        elif fmt==1:
            fn=f'{sid:04d}_{safe}.swf-adpcm'; (ns.outdir/fn).write_bytes(data); rec.update(file=fn,extracted_bytes=len(data),note='raw SWF ADPCM bitstream; decode during asset-build phase if desired')
        else:
            fn=f'{sid:04d}_{safe}.soundbin'; (ns.outdir/fn).write_bytes(data); rec.update(file=fn,extracted_bytes=len(data))
        sounds.append(rec)
    (ns.outdir/'manifest.json').write_text(json.dumps(sounds,indent=2)+"\n")
    byfmt={}
    for x in sounds: byfmt[x['format_name']]=byfmt.get(x['format_name'],0)+1
    print(json.dumps({'sounds':len(sounds),'formats':byfmt},indent=2))
if __name__=='__main__': main()
