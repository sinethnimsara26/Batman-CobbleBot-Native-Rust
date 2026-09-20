#!/usr/bin/env python3
"""Generate durable reverse-engineering manifests from swf_re.py output.

This is deliberately data-oriented: the Rust port should consume/re-implement
these recovered facts rather than depending on Flash at runtime.
"""
from __future__ import annotations
from pathlib import Path
import argparse, gzip, json, re

GAME_FRAMES={
  19:('level1a',696),20:('level1b',1075),21:('level1c_penguin',1251),
  23:('level2a',1402),25:('level2b',1496),27:('level3a',1779),
  28:('level3b_kabuki',1922),30:('level4a',2040),31:('level4b_cobblebot',2141),
}
CHARACTERS={
  'batman':691,'batman_highway':1862,'tutorial_overlay':737,
  'goon_a':880,'goon_b':964,'goon_c':1365,'fatgoon':1772,
  'penguin':1250,'bird_eagle':1393,'kabuki':1919,'maid':2037,'cobblebot':2135,
}

def push_values(a):
    return [v.get('resolved') if v.get('resolved') is not None else v.get('value') for v in a.get('values',[])]

def infer_functions(actions):
    funcs=[]
    for block in actions:
        if block['context']!='root' or block['frame']!=15: continue
        aa=block['actions']
        for i,a in enumerate(aa):
            if a.get('op') not in ('DefineFunction','DefineFunction2'): continue
            name=a.get('function') or ''
            if not name and i and aa[i-1].get('op')=='Push':
                vals=push_values(aa[i-1])
                if vals and isinstance(vals[-1],str): name=vals[-1]
            refs=[]; numbers=[]
            stack=list(a.get('body',[]))
            while stack:
                x=stack.pop()
                if x.get('op')=='Push':
                    for v in push_values(x):
                        if isinstance(v,str): refs.append(v)
                        elif isinstance(v,(int,float)) and not isinstance(v,bool): numbers.append(v)
                stack.extend(x.get('body',[]))
            funcs.append({
              'name':name,'action':a['op'],'code_size':a.get('code_size'),
              'body_offset':a.get('body_offset'),'params':a.get('params',[]),
              'registers':a.get('registers'),'flags':a.get('flags'),
              'referenced_strings':sorted(set(refs)),
              'numeric_literals':sorted(set(numbers), key=lambda x:(float(x),str(type(x))))
            })
    return funcs

def root_block(actions,frame):
    return [b for b in actions if b['context']=='root' and b['frame']==frame]

def candidate_pairs(blocks, names):
    out=[]
    for block in blocks:
      stack=list(block['actions'])
      while stack:
        a=stack.pop()
        if a.get('op')=='Push':
          vals=push_values(a)
          if len(vals)>=2 and isinstance(vals[-2],str) and vals[-2] in names and isinstance(vals[-1],(int,float,bool)):
            out.append({'offset':a['offset'],'property':vals[-2],'value':vals[-1]})
        stack.extend(a.get('body',[]))
    return out

def route_pushes(actions):
    routes=[]
    for block in actions:
      stack=list(block['actions'])
      while stack:
        a=stack.pop()
        if a.get('op')=='Push':
          vals=push_values(a)
          if any(v in ('gotoNext','gotoLast') for v in vals):
            routes.append({'context':block['context'],'frame':block['frame'],'tag_offset':block['tag_offset'],'action_offset':a['offset'],'values':vals})
        stack.extend(a.get('body',[]))
    return routes

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('reports',type=Path); ap.add_argument('outdir',type=Path); ap.add_argument('--bitmaps',type=Path); ap.add_argument('--sounds',type=Path); ns=ap.parse_args()
    ns.outdir.mkdir(parents=True,exist_ok=True)
    movie=json.loads((ns.reports/'movie.json').read_text()); symbols=json.loads((ns.reports/'symbols.json').read_text()); actions=json.loads((ns.reports/'actions.json').read_text()); roots=json.loads((ns.reports/'root_placements.json').read_text())

    funcs=infer_functions(actions)
    (ns.outdir/'function-index.json').write_text(json.dumps(funcs,indent=2)+"\n")

    anim={}
    for name,sid in CHARACTERS.items():
        s=symbols.get(str(sid),{})
        anim[name]={'symbol_id':sid,'frames':s.get('frames'),'labels':s.get('labels',[])}
    (ns.outdir/'animation-labels.json').write_text(json.dumps(anim,indent=2)+"\n")

    # Map root gameplay frame -> game sprite -> level sprite -> named level/game placements.
    level_instances={}
    for frame,(name,gid) in GAME_FRAMES.items():
        game=symbols[str(gid)]
        gp=game.get('placements',[])
        levelp=next((p for p in gp if p.get('name')=='level'),None)
        lid=levelp.get('character_id') if levelp else None
        lp=symbols.get(str(lid),{}).get('placements',[]) if lid else []
        level_instances[name]={
          'root_frame':frame,'game_symbol_id':gid,'level_symbol_id':lid,
          'player':next((p for p in gp if p.get('name')=='player'),None),
          'ground':next((p for p in lp if p.get('name')=='ground'),None),
          'named_game_instances':[p for p in gp if p.get('name')],
          'named_level_instances':[p for p in lp if p.get('name')],
        }
    (ns.outdir/'level-instances.json').write_text(json.dumps(level_instances,indent=2)+"\n")

    props={'gravity','globalGravity','camHeight','camJump','camMin','camMax','camZoom','camOffset','playerDx','playerDy','playerDir','playerSpeed','playerWalkSpeed','playerLives','playerHealth','playerStrength','playerHeight','playerWidth','health','strength','stength','speed','enemyHeight','range','xMax','xMin'}
    level_constants={}
    for frame,(name,gid) in GAME_FRAMES.items(): level_constants[name]={'root_frame':frame,'candidates':candidate_pairs(root_block(actions,frame),props)}
    (ns.outdir/'level-constants-raw.json').write_text(json.dumps(level_constants,indent=2)+"\n")

    routes=route_pushes(actions)
    (ns.outdir/'routing-evidence.json').write_text(json.dumps(routes,indent=2)+"\n")

    # Compact raw disassembly of the 45 global game functions: invaluable when
    # an implementation detail needs re-checking without reopening Flash.
    raw_funcs=[]
    for block in actions:
        if block['context']!='root' or block['frame']!=15: continue
        aa=block['actions']
        for i,a in enumerate(aa):
            if a.get('op') not in ('DefineFunction','DefineFunction2'): continue
            name=''
            if i and aa[i-1].get('op')=='Push':
                vals=push_values(aa[i-1]); name=vals[-1] if vals and isinstance(vals[-1],str) else ''
            raw_funcs.append({'name':name,**a})
    with gzip.open(ns.outdir/'core-functions.disasm.json.gz','wt',encoding='utf-8',compresslevel=9) as z: json.dump(raw_funcs,z,separators=(',',':'))

    selected=[]
    for frame in GAME_FRAMES:
        selected.extend(root_block(actions,frame))
    with gzip.open(ns.outdir/'level-init-actions.disasm.json.gz','wt',encoding='utf-8',compresslevel=9) as z: json.dump(selected,z,separators=(',',':'))

    if ns.bitmaps:
        bm=json.loads(ns.bitmaps.read_text()); (ns.outdir/'bitmap-manifest.json').write_text(json.dumps(bm,indent=2)+"\n")
    if ns.sounds:
        sm=json.loads(ns.sounds.read_text()); (ns.outdir/'sound-manifest.json').write_text(json.dumps(sm,indent=2)+"\n")

    # Root frame placements are small enough to preserve verbatim.
    (ns.outdir/'root-placements.json').write_text(json.dumps(roots,indent=2)+"\n")
    (ns.outdir/'movie.json').write_text(json.dumps(movie,indent=2)+"\n")
    print('generated manifests in',ns.outdir)
if __name__=='__main__': main()
