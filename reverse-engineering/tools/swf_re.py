#!/usr/bin/env python3
from pathlib import Path
import struct, json, zlib, io, hashlib, re
from collections import Counter, defaultdict

class Bits:
    def __init__(self,b,pos=0): self.b=b; self.bit=pos*8
    def u(self,n):
        v=0
        for _ in range(n):
            byte=self.b[self.bit>>3]; sh=7-(self.bit&7); self.bit+=1; v=(v<<1)|((byte>>sh)&1)
        return v
    def s(self,n):
        v=self.u(n)
        if n and (v>>(n-1))&1: v-=1<<n
        return v
    def align(self): self.bit=(self.bit+7)&~7
    @property
    def pos(self): return self.bit>>3

def read_rect(b,pos):
    br=Bits(b,pos); n=br.u(5); vals=[br.s(n) for _ in range(4)]; br.align(); return vals,br.pos

def read_matrix(b,pos):
    br=Bits(b,pos); sx=sy=1.0; r0=r1=0.0
    if br.u(1):
        n=br.u(5); sx=br.s(n)/65536; sy=br.s(n)/65536
    if br.u(1):
        n=br.u(5); r0=br.s(n)/65536; r1=br.s(n)/65536
    n=br.u(5); tx=br.s(n)/20; ty=br.s(n)/20; br.align()
    return {'sx':sx,'sy':sy,'r0':r0,'r1':r1,'tx':tx,'ty':ty},br.pos

def cstr(b,pos):
    e=b.find(b'\0',pos)
    if e<0: return b[pos:].decode('latin1','replace'),len(b)
    return b[pos:e].decode('latin1','replace'),e+1

def tags(data,start,end=None):
    pos=start; end=len(data) if end is None else end
    while pos+2<=end:
        tag_off=pos; h=struct.unpack_from('<H',data,pos)[0]; pos+=2; code=h>>6; ln=h&63
        if ln==63:
            if pos+4>end: break
            ln=struct.unpack_from('<I',data,pos)[0]; pos+=4
        payload_off=pos; payload=data[pos:pos+ln]; pos+=ln
        yield code,payload,tag_off,payload_off
        if code==0: break

# AVM1 action decoder
OP={
0x00:'End',0x04:'NextFrame',0x05:'PreviousFrame',0x06:'Play',0x07:'Stop',0x08:'ToggleQuality',0x09:'StopSounds',
0x0A:'Add',0x0B:'Subtract',0x0C:'Multiply',0x0D:'Divide',0x0E:'Equals',0x0F:'Less',0x10:'And',0x11:'Or',0x12:'Not',
0x13:'StringEquals',0x14:'StringLength',0x15:'StringExtract',0x17:'Pop',0x18:'ToInteger',0x1C:'GetVariable',0x1D:'SetVariable',
0x20:'SetTarget2',0x21:'StringAdd',0x22:'GetProperty',0x23:'SetProperty',0x24:'CloneSprite',0x25:'RemoveSprite',0x26:'Trace',0x27:'StartDrag',0x28:'EndDrag',0x29:'StringLess',
0x2A:'Throw',0x2B:'CastOp',0x2C:'ImplementsOp',0x30:'RandomNumber',0x31:'MBStringLength',0x32:'CharToAscii',0x33:'AsciiToChar',0x34:'GetTime',0x35:'MBStringExtract',0x36:'MBCharToAscii',0x37:'MBAsciiToChar',
0x3A:'Delete',0x3B:'Delete2',0x3C:'DefineLocal',0x3D:'CallFunction',0x3E:'Return',0x3F:'Modulo',0x40:'NewObject',0x41:'DefineLocal2',0x42:'InitArray',0x43:'InitObject',0x44:'TypeOf',0x45:'TargetPath',0x46:'Enumerate',0x47:'Add2',0x48:'Less2',0x49:'Equals2',0x4A:'ToNumber',0x4B:'ToString',0x4C:'PushDuplicate',0x4D:'StackSwap',0x4E:'GetMember',0x4F:'SetMember',0x50:'Increment',0x51:'Decrement',0x52:'CallMethod',0x53:'NewMethod',0x54:'InstanceOf',0x55:'Enumerate2',
0x60:'BitAnd',0x61:'BitOr',0x62:'BitXor',0x63:'BitLShift',0x64:'BitRShift',0x65:'BitURShift',0x66:'StrictEquals',0x67:'Greater',0x68:'StringGreater',0x69:'Extends',
0x81:'GotoFrame',0x83:'GetURL',0x87:'StoreRegister',0x88:'ConstantPool',0x8A:'WaitForFrame',0x8B:'SetTarget',0x8C:'GoToLabel',0x8D:'WaitForFrame2',0x8E:'DefineFunction2',0x8F:'Try',0x94:'With',0x96:'Push',0x99:'Jump',0x9A:'GetURL2',0x9B:'DefineFunction',0x9D:'If',0x9E:'Call',0x9F:'GotoFrame2'
}

def avm1_value(data,p):
    t=data[p];p+=1
    if t==0:
        s,p=cstr(data,p); return {'type':'string','value':s},p
    if t==1: return {'type':'float','value':struct.unpack_from('<f',data,p)[0]},p+4
    if t==2: return {'type':'null','value':None},p
    if t==3: return {'type':'undefined','value':None},p
    if t==4: return {'type':'register','value':data[p]},p+1
    if t==5: return {'type':'bool','value':bool(data[p])},p+1
    if t==6:
        # SWF double bytes are word-swapped
        raw=data[p:p+8]; raw=raw[4:8]+raw[0:4]
        return {'type':'double','value':struct.unpack('<d',raw)[0]},p+8
    if t==7: return {'type':'int','value':struct.unpack_from('<i',data,p)[0]},p+4
    if t==8: return {'type':'const8','value':data[p]},p+1
    if t==9: return {'type':'const16','value':struct.unpack_from('<H',data,p)[0]},p+2
    return {'type':f'unknown_{t}','value':None},len(data)

def disasm_actions(data,base=0,initial_pool=None):
    """Decode an AVM1 action stream, including nested function bodies.

    Important SWF quirk: for ActionDefineFunction / ActionDefineFunction2 the
    action record's declared Length covers only the function metadata. The
    CodeSize bytes that follow are *outside* that Length and must be consumed
    separately. Treating them as top-level actions loses function boundaries.
    """
    out=[]; p=0; pool=list(initial_pool or [])
    while p<len(data):
        off=p; op=data[p];p+=1; ln=0; payload=b''
        if op>=0x80:
            if p+2>len(data): break
            ln=struct.unpack_from('<H',data,p)[0];p+=2
            if p+ln>len(data):
                out.append({'offset':base+off,'opcode':op,'op':OP.get(op,f'Action_{op:02X}'),'decode_error':'truncated action payload'})
                break
            payload=data[p:p+ln];p+=ln
        name=OP.get(op,f'Action_{op:02X}'); rec={'offset':base+off,'opcode':op,'op':name}
        try:
            if op==0x96:
                vals=[]; q=0
                while q<len(payload):
                    v,q=avm1_value(payload,q)
                    if v['type'].startswith('const'):
                        idx=v['value']; v['resolved']=pool[idx] if idx<len(pool) else None
                    vals.append(v)
                rec['values']=vals
            elif op==0x88:
                q=0; n=struct.unpack_from('<H',payload,q)[0];q+=2; pool=[]
                for _ in range(n): s,q=cstr(payload,q); pool.append(s)
                rec['pool']=pool.copy()
            elif op in (0x99,0x9D): rec['branch']=struct.unpack_from('<h',payload,0)[0]
            elif op==0x81: rec['frame']=struct.unpack_from('<H',payload,0)[0]
            elif op in (0x8B,0x8C): rec['string']=payload.rstrip(b'\0').decode('latin1','replace')
            elif op==0x87: rec['register']=payload[0]
            elif op==0x8A: rec['frame']=struct.unpack_from('<H',payload,0)[0]; rec['skip']=payload[2]
            elif op==0x9B:
                q=0; fn,q=cstr(payload,q); n=struct.unpack_from('<H',payload,q)[0];q+=2; params=[]
                for _ in range(n): s,q=cstr(payload,q);params.append(s)
                size=struct.unpack_from('<H',payload,q)[0];q+=2
                # Function body follows the ActionRecord, outside its Length.
                code=data[p:p+size]; body_base=base+p; p+=size
                rec.update(function=fn,params=params,code_size=size,body_offset=body_base,body=disasm_actions(code,body_base,pool))
            elif op==0x8E:
                q=0; fn,q=cstr(payload,q); n=struct.unpack_from('<H',payload,q)[0];q+=2; regcnt=payload[q];q+=1; flags=struct.unpack_from('<H',payload,q)[0];q+=2; params=[]
                for _ in range(n): reg=payload[q];q+=1;s,q=cstr(payload,q);params.append({'reg':reg,'name':s})
                size=struct.unpack_from('<H',payload,q)[0];q+=2
                code=data[p:p+size]; body_base=base+p; p+=size
                rec.update(function=fn,params=params,registers=regcnt,flags=flags,code_size=size,body_offset=body_base,body=disasm_actions(code,body_base,pool))
            elif payload: rec['raw']=payload.hex()
        except Exception as e:
            rec['decode_error']=str(e); rec['raw']=payload.hex()
        out.append(rec)
        if op==0: break
    return out

def collect_strings_actions(actions):
    res=[]
    for a in actions:
        if a.get('op')=='ConstantPool': res.extend(a.get('pool',[]))
        if a.get('op')=='Push':
            for v in a.get('values',[]):
                if v.get('type')=='string': res.append(v['value'])
                if v.get('resolved') is not None: res.append(v['resolved'])
        if 'body' in a: res.extend(collect_strings_actions(a['body']))
    return res

def parse_place2(pl):
    if len(pl)<3:return None
    flags=pl[0]; depth=struct.unpack_from('<H',pl,1)[0];p=3; d={'depth':depth,'flags':flags}
    # HasClipActions 0x80 HasClipDepth 0x40 HasName 0x20 HasRatio 0x10 HasColorTransform 0x08 HasMatrix 0x04 HasCharacter 0x02 Move 0x01
    if flags&0x02:
        d['character_id']=struct.unpack_from('<H',pl,p)[0];p+=2
    if flags&0x04:
        d['matrix'],p=read_matrix(pl,p)
    # CXFORMWITHALPHA skip roughly with bit parser
    if flags&0x08:
        br=Bits(pl,p); has_add=br.u(1);has_mult=br.u(1);n=br.u(4)
        if has_mult:
            for _ in range(4): br.s(n)
        if has_add:
            for _ in range(4): br.s(n)
        br.align();p=br.pos
    if flags&0x10: d['ratio']=struct.unpack_from('<H',pl,p)[0];p+=2
    if flags&0x20:
        d['name'],p=cstr(pl,p)
    if flags&0x40: d['clip_depth']=struct.unpack_from('<H',pl,p)[0];p+=2
    return d

def main(src,outdir):
    b=Path(src).read_bytes(); out=Path(outdir); out.mkdir(parents=True,exist_ok=True)
    if b[:3]!=b'FWS': raise SystemExit('expected uncompressed FWS')
    rect,pos=read_rect(b,8); fps=struct.unpack_from('<H',b,pos)[0]/256;pos+=2; frame_count=struct.unpack_from('<H',b,pos)[0];pos+=2
    report={'sha256':hashlib.sha256(b).hexdigest(),'version':b[3],'declared_length':struct.unpack_from('<I',b,4)[0], 'stage_twips':rect,'stage_px':[rect[0]/20,rect[1]/20,rect[2]/20,rect[3]/20], 'fps':fps,'root_frames':frame_count,'root_tag_start':pos}
    counts=Counter(); exports={}; symbols={}; actions=[]; sounds=[]; images=[]; root_labels=[]; placements=[]

    def scan_tag_stream(data,start,end,context):
        frame=0; local_labels=[]; local_places=[]; local_removals=[]; local_actions=[]
        for code,pl,toff,poff in tags(data,start,end):
            counts[code]+=1
            if code==1: frame+=1
            elif code==43:
                lab,_=cstr(pl,0); local_labels.append({'frame':frame,'label':lab})
            elif code==26:
                x=parse_place2(pl)
                if x: x['frame']=frame; x['tag_offset']=toff; local_places.append(x)
            elif code==28 and len(pl)>=2:
                local_removals.append({
                    'depth':struct.unpack_from('<H',pl,0)[0],
                    'frame':frame,
                    'tag_offset':toff,
                })
            elif code==12:
                aa=disasm_actions(pl,poff); local_actions.append({'frame':frame,'tag_offset':toff,'actions':aa,'strings':collect_strings_actions(aa)})
                actions.append({'context':context,'frame':frame,'tag_offset':toff,'actions':aa,'strings':collect_strings_actions(aa)})
            elif code==39 and len(pl)>=4:
                sid=struct.unpack_from('<H',pl,0)[0]; fc=struct.unpack_from('<H',pl,2)[0]
                sub={'id':sid,'frames':fc}; symbols[sid]=sub
                labs,pls,rems,acts=scan_tag_stream(pl,4,len(pl),f'sprite:{sid}')
                sub['labels']=labs;sub['placements']=pls;sub['removals']=rems;sub['action_blocks']=len(acts)
            elif code==56:
                q=0;n=struct.unpack_from('<H',pl,q)[0];q+=2
                for _ in range(n):
                    cid=struct.unpack_from('<H',pl,q)[0];q+=2;name,q=cstr(pl,q);exports[name]=cid
            elif code==14 and len(pl)>=7:
                sid=struct.unpack_from('<H',pl,0)[0]; flags=pl[2]; fmt=flags>>4; rate=(flags>>2)&3; size=(flags>>1)&1; typ=flags&1; samples=struct.unpack_from('<I',pl,3)[0]
                sounds.append({'id':sid,'format':fmt,'rate_code':rate,'bits16':bool(size),'stereo':bool(typ),'samples':samples,'tag_offset':toff,'payload_offset':poff+7,'payload_len':len(pl)-7})
            elif code in (6,20,21,35,36):
                cid=struct.unpack_from('<H',pl,0)[0] if len(pl)>=2 else None; images.append({'id':cid,'tag':code,'tag_offset':toff,'payload_offset':poff,'payload_len':len(pl)})
        return local_labels,local_places,local_removals,local_actions

    rl,rp,rr,ra=scan_tag_stream(b,pos,len(b),'root'); root_labels.extend(rl); placements.extend(rp)
    report['tag_counts']={str(k):v for k,v in sorted(counts.items())};report['root_labels']=root_labels;report['exports']=exports
    (out/'movie.json').write_text(json.dumps(report,indent=2))
    (out/'symbols.json').write_text(json.dumps({str(k):v for k,v in sorted(symbols.items())},indent=2))
    (out/'actions.json').write_text(json.dumps(actions,indent=2))
    (out/'sounds.json').write_text(json.dumps(sounds,indent=2))
    (out/'images.json').write_text(json.dumps(images,indent=2))
    (out/'root_placements.json').write_text(json.dumps(placements,indent=2))
    (out/'root_removals.json').write_text(json.dumps(rr,indent=2))
    # index interesting symbols by labels/strings
    interesting=[]
    keys=re.compile(r'walk|run|jump|glide|punch|kick|batarang|grappl|penguin|kabuki|robo|goon|maid|biker|eagle|ground|level[1-4]|checkpoint|gameover|capespin|electro|hurt|die',re.I)
    for sid,s in symbols.items():
        labs=[x['label'] for x in s.get('labels',[])]
        ss=[]
        for a in actions:
            if a['context']==f'sprite:{sid}': ss+=a['strings']
        hits=sorted(set(x for x in labs+ss if keys.search(x)))
        if hits: interesting.append({'id':sid,'frames':s['frames'],'labels':labs,'hits':hits[:200]})
    (out/'interesting_symbols.json').write_text(json.dumps(interesting,indent=2))
    print(json.dumps(report,indent=2)[:4000])
    print('symbols',len(symbols),'action_blocks',len(actions),'sounds',len(sounds),'images',len(images),'exports',len(exports))

if __name__=='__main__':
    import sys
    main(sys.argv[1],sys.argv[2])
