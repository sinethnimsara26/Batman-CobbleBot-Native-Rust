#!/usr/bin/env python3
"""Inventory bitmap-backed Flash vector fills and their matrices.

This closes the link between decoded DefineBits images and the vector shapes
that use them, which is needed later when baking original level artwork.
"""
from __future__ import annotations
from pathlib import Path
import argparse, json, struct
from collections import Counter
from swf_re import Bits, read_rect, read_matrix, tags

SHAPE_TAGS={2,22,32}

def parse_fill_style(b,p,alpha):
    typ=b[p];p+=1; rec={'type':typ}
    if typ==0x00:
        p+=4 if alpha else 3
    elif typ in (0x10,0x12,0x13):
        m,p=read_matrix(b,p); rec['matrix']=m; n=b[p]&15;p+=1; p+=n*(1+(4 if alpha else 3))
    elif typ in (0x40,0x41,0x42,0x43):
        bid=struct.unpack_from('<H',b,p)[0];p+=2; m,p=read_matrix(b,p); rec.update(bitmap_id=bid,matrix=m)
    else: raise ValueError(f'fill 0x{typ:02x}')
    return rec,p

def parse_fill_array(b,p,alpha):
    n=b[p];p+=1
    if n==255:n=struct.unpack_from('<H',b,p)[0];p+=2
    a=[]
    for _ in range(n): r,p=parse_fill_style(b,p,alpha);a.append(r)
    return a,p

def skip_lines(b,p,alpha):
    n=b[p];p+=1
    if n==255:n=struct.unpack_from('<H',b,p)[0];p+=2
    p+=n*(2+(4 if alpha else 3));return p

def all_fills(payload,tag):
    _,p=struct.unpack_from('<H',payload,0)[0],2; bounds,p=read_rect(payload,p); alpha=tag==32
    fills,p=parse_fill_array(payload,p,alpha);p=skip_lines(payload,p,alpha)
    br=Bits(payload,p); nf=br.u(4); nl=br.u(4)
    while True:
        typ=br.u(1)
        if typ:
            straight=br.u(1);n=br.u(4)+2
            if straight:
                gen=br.u(1)
                if gen: br.s(n);br.s(n)
                elif br.u(1): br.s(n)
                else: br.s(n)
            else:
                br.s(n);br.s(n);br.s(n);br.s(n)
        else:
            ns=br.u(1); sl=br.u(1); sf1=br.u(1); sf0=br.u(1); mv=br.u(1)
            if not(ns or sl or sf1 or sf0 or mv): break
            if mv:
                n=br.u(5);br.s(n);br.s(n)
            if sf0: br.u(nf)
            if sf1: br.u(nf)
            if sl: br.u(nl)
            if ns:
                br.align();p=br.pos; more,p=parse_fill_array(payload,p,alpha);fills.extend(more);p=skip_lines(payload,p,alpha);br.bit=p*8;nf=br.u(4);nl=br.u(4)
    return bounds,fills

def main():
    ap=argparse.ArgumentParser();ap.add_argument('swf',type=Path);ap.add_argument('out',type=Path);ns=ap.parse_args()
    b=ns.swf.read_bytes();_,p=read_rect(b,8);p+=4; result=[];counts=Counter(); errors=[]
    for code,pl,toff,poff in tags(b,p,len(b)):
        if code not in SHAPE_TAGS: continue
        cid=struct.unpack_from('<H',pl,0)[0]
        try:
            bounds,fills=all_fills(pl,code)
            for f in fills: counts[f['type']]+=1
            bitmap=[f for f in fills if 'bitmap_id' in f]
            if bitmap: result.append({'shape_id':cid,'tag':code,'bounds_twips':bounds,'bitmap_fills':bitmap})
        except Exception as e: errors.append({'shape_id':cid,'tag':code,'error':repr(e)})
    obj={'shape_count_with_bitmap_fills':len(result),'fill_type_counts':{hex(k):v for k,v in sorted(counts.items())},'shapes':result,'errors':errors}
    ns.out.parent.mkdir(parents=True,exist_ok=True);ns.out.write_text(json.dumps(obj,indent=2)+"\n");print('bitmap-backed shapes',len(result),'errors',len(errors))
if __name__=='__main__':main()
