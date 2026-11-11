#!/usr/bin/env python3
"""B09: source-backed FULL frame-0..88 original Bank Street goon source proof.

Original FWS SHA gated; compares extracted DefineSprite timelines to existing
source labels and ten GAME-local placements. Renders all 178 parent frames 1x
and source rerasterizes nine label-starts per family at 6x→HQ3x.
Exports child-placement ages/source lengths but does NOT execute AVM1 or ship
original animation packs to the Rust game. Original SWF is never uploaded.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
from pathlib import Path
import re

from PIL import Image, ImageDraw
from b03_bake_stage_asset_proofs import expect_original_source, validate_stage
from b05_bake_bank_street_1x import source_definition_tags
from bake_level_tiles import load_bitmaps
from bake_sprite_frames import display_list, load_shapes, render_symbol, symbol_bounds

LABELS=(("stand",0),("walk",9),("retreat",19),("attack",29),
        ("hurt",39),("fall",49),("die",59),("idle",69),("intro",79))
EXPECTED={
    "goon1":(880,1105,-1086.15,398.1),
    "goon2":(964,1107,-746.15,398.1),
    "goon4":(880,1109,-2074.85,398.1),
    "goon3":(964,1111,-1734.85,398.1),
    "goon6":(964,1113,-3466.85,398.1),
    "goon5":(964,1115,-3126.85,398.1),
    "goon7":(964,1117,-5364.85,398.1),
    "goon9":(880,1119,-5970.75,398.1),
    "goon8":(880,1121,-4976.85,398.1),
    "goon10":(964,1123,-6110.45,398.1),
}
SOURCE_SHA="3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
ACTOR_SYMBOLS=(880,964)

def sha(data:bytes)->str:
    return hashlib.sha256(data).hexdigest()

def check_source(stage,symbols,labels,swf):
    assert (stage["root_frame"],stage["game_symbol_id"],
            stage["level_symbol_id"],stage["ground"]["character_id"]) == (20,1075,783,782)
    origin=[p for p in stage["named_game_instances"] if p["name"]=="level"]
    assert len(origin)==1 and origin[0]["depth"]==1
    assert tuple(origin[0]["matrix"][k] for k in ("tx","ty"))==(1034.65,239.25)
    player=[p for p in stage["named_game_instances"] if p["name"]=="player"]
    assert len(player)==1 and player[0]["depth"]==1127 and player[0]["character_id"]==691
    # Compare the committed source index AGAINST the freshly parsed SWF.
    game_display=display_list(symbols[str(stage["game_symbol_id"])],0)
    goons=[p for p in stage["named_game_instances"] if re.fullmatch(r"goon(?:10|[1-9])",p["name"])]
    assert len(goons)==10 and {p["name"] for p in goons}==set(EXPECTED)
    assert not any(p["name"] in EXPECTED for p in stage["named_level_instances"])
    counts={880:0,964:0}
    for p in goons:
        cid,depth,x,y=EXPECTED[p["name"]]
        m=p["matrix"]
        assert p["character_id"]==cid and p["depth"]==depth and p["flags"]==38 and p["frame"]==0
        assert (m["sx"],m["sy"],m["r0"],m["r1"])==(1,1,0,0)
        assert abs(m["tx"]-x)<1e-7 and abs(m["ty"]-y)<1e-7
        live=game_display.get(depth)
        assert live is not None and live.get("name")==p["name"] and live.get("character_id")==cid
        for k in ("sx","sy","r0","r1","tx","ty"):
            assert abs(live["matrix"][k]-m[k])<1e-7,(p["name"],k)
        assert 1097<depth<1127
        counts[cid]+=1
    assert counts=={880:4,964:6}
    defs=source_definition_tags(swf,set(ACTOR_SYMBOLS))
    assert defs=={880:39,964:39},defs
    for sid,key in ((880,"goon_a"),(964,"goon_b")):
        actual=symbols[str(sid)]
        index=labels[key]
        assert actual["frames"]==89 and index["frames"]==89
        parsed=[(p["label"],p["frame"]) for p in sorted(actual["labels"],key=lambda x:x["frame"])]
        reference=[(p["label"],p["frame"]) for p in index["labels"]]
        assert parsed==reference==list(LABELS),(sid,parsed,reference)
        assert len(actual["placements"])>0,(sid,"missing original sprite children")
    return sorted(goons,key=lambda p:p["depth"]),counts,defs

def child_frame_snapshot(symbol,frame,symbols):
    current=display_list(symbol,frame)
    report=[]
    for depth,placement in sorted(current.items()):
        child=placement.get("character_id")
        if child is None: continue
        birth=placement.get("born",0)
        length=max(1,int(symbols.get(str(child),{}).get("frames",1)))
        report.append({"depth":depth,"child_symbol":child,"placed_on_parent_frame_zero_based":birth,
                       "elapsed_parent_frames":frame-birth,
                       "child_source_frame_count":length,
                       "child_frame_zero_based":max(0,frame-birth)%length})
    return report

def source_frame_series(sid,symbols,shapes,bitmaps,out):
    sym=symbols[str(sid)]
    raw=[symbol_bounds(sid,f,(1,0,0,1,0,0),symbols,shapes) for f in range(89)]
    good=[box for box in raw if all(math.isfinite(n) for n in box)]
    assert good,(sid,"no original drawable frames")
    left=math.floor(min(box[0] for box in good))-3
    top=math.floor(min(box[1] for box in good))-3
    right=math.ceil(max(box[2] for box in good))+3
    bottom=math.ceil(max(box[3] for box in good))+3
    w,h=right-left,bottom-top
    assert 5<w<=900 and 5<h<=700,(sid,w,h)
    sub=out/"goon_1x"/str(sid)
    sub.mkdir(parents=True,exist_ok=True)
    hq=out/"goon_hq3x_label_starts"/str(sid)
    hq.mkdir(parents=True,exist_ok=True)
    frames=[]
    hq_frames=[]
    seen=set()
    for frame in range(89):
        canvas=Image.new("RGBA",(w,h),(0,0,0,0))
        missing=set()
        unsupported=set()
        render_symbol(sid,frame,(1.,0.,0.,1.,-float(left),-float(top)),
                      canvas,symbols,shapes,bitmaps,unsupported,missing)
        assert not missing and not unsupported,(sid,frame,missing,unsupported)
        name=f"{frame:03}.png"
        png=sub/name
        canvas.save(png,optimize=True)
        alpha=canvas.getchannel("A")
        nonempty=alpha.getbbox() is not None
        if nonempty:seen.add(frame)
        label=next(label for label,start in reversed(LABELS) if frame>=start)
        active=child_frame_snapshot(sym,frame,symbols)
        frames.append({"frame_zero_based":frame,"source_state_label":label,
                       "source_children":active,"not_empty":nonempty,
                       "file":str(png.relative_to(out)).replace("\\","/"),
                       "png_sha256":sha(png.read_bytes()),"rgba_sha256":sha(canvas.tobytes())})
        if frame in dict((start,label) for label,start in LABELS):
            assert nonempty,("source label-start frame empty",sid,frame)
            # 2 logical px source overscan allows Lanczos without clipped edges.
            margin=2
            hi=Image.new("RGBA",((w+margin*2)*6,(h+margin*2)*6),(0,0,0,0))
            render_symbol(sid,frame,(6.,0.,0.,6.,
                          -float(left-margin)*6,-float(top-margin)*6),
                          hi,symbols,shapes,bitmaps,unsupported,missing)
            assert not missing and not unsupported,(sid,frame,"HQ",missing,unsupported)
            hi=hi.convert("RGBa").resize(((w+margin*2)*3,(h+margin*2)*3),
                  Image.Resampling.LANCZOS).convert("RGBA")
            px=margin*3
            hi=hi.crop((px,px,px+w*3,px+h*3))
            assert hi.size==(w*3,h*3) and hi.getchannel("A").getbbox()
            to=hq/name
            hi.save(to,optimize=True)
            hq_frames.append({"frame_zero_based":frame,"source_state_label":label,
                "file":str(to.relative_to(out)).replace("\\","/"),
                "png_sha256":sha(to.read_bytes()),"rgba_sha256":sha(hi.tobytes())})
    assert len(frames)==89 and len(hq_frames)==9 and len(seen)>=9
    # Preserve the ORIGINAL parent timeline's exact label intervals,
    # but do not infer source script loop counts or combat state durations.
    spans=[]
    for i,(label,start) in enumerate(LABELS):
        end=LABELS[i+1][1] if i+1<len(LABELS) else 89
        children=[{"frame_zero_based":f,"children":frames[f]["source_children"]}
                  for f in range(start,end)]
        spans.append({"name":label,"first_parent_frame_zero_based":start,
                      "end_parent_frame_exclusive":end,
                      "source_parent_frames":end-start,
                      "child_placement_timeline":children})
    print("B09_ORIGINAL_GOON_TIMELINE",sid,89,"frames",len(seen),"visible",
          "9 original states","true HQ label views",len(hq_frames),flush=True)
    return {"source_symbol_id":sid,"parent_frames":89,"labels":spans,
            "raw_source_bounds_union":[left,top,right,bottom],
            "uniform_source_1x_canvas":[w,h],
            "frame0_through_88_1x":frames,
            "source_reraster_6x_to_hq3x_label_views":hq_frames,
            "source_direct_action_blocks":sym.get("action_blocks",0),
            "source_sprite_placements_events":len(sym["placements"]),
            "source_sprite_remove_events":len(sym.get("removals",[])),
            "visible_source_parent_frame_count":len(seen)}

def contact_sheet(out,rendered):
    # One frame per ORIGINAL label, family side-by-side. Static source artwork.
    cols=len(LABELS)
    cellw=185;cellh=235
    sheet=Image.new("RGB",(cols*cellw,2*cellh+32),"#242c39")
    draw=ImageDraw.Draw(sheet)
    for row,sid in enumerate(ACTOR_SYMBOLS):
        bundle=rendered[str(sid)]
        for col,(label,frame) in enumerate(LABELS):
            with Image.open(out/"goon_1x"/str(sid)/f"{frame:03}.png") as source:
                thumb=source.convert("RGBA")
            thumb.thumbnail((cellw-10,cellh-38),Image.Resampling.LANCZOS)
            left=col*cellw+(cellw-thumb.width)//2
            top=row*cellh+26+(cellh-38-thumb.height)//2
            sheet.paste(thumb,(left,top),thumb)
            draw.text((col*cellw+6,row*cellh+5),f"SWF #{sid} {label} f{frame}",fill="white")
    draw.text((7,2*cellh+8),"B09 SOURCE SPRITE LIBRARY - NOT original live Flash gameplay",fill="#ffc96b")
    path=out/"B09_ORIGINAL_BANK_GOON_LABEL_SOURCE_CONTACT_SHEET.png"
    sheet.save(path,optimize=True)
    return {"file":path.name,"png_sha256":sha(path.read_bytes()),"size":list(sheet.size)}

def main():
    ap=argparse.ArgumentParser()
    for arg in ("swf","symbols","bitmaps","instances","spec","labels","out"):
        ap.add_argument("--"+arg,type=Path,required=True)
    args=ap.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    original=expect_original_source(args.swf)
    assert original==SOURCE_SHA
    symbols=json.loads(args.symbols.read_text(encoding="utf-8"))
    stage=json.loads(args.instances.read_text(encoding="utf-8"))["level1b"]
    spec=json.loads(args.spec.read_text(encoding="utf-8"))["level1b"]
    source_labels=json.loads(args.labels.read_text(encoding="utf-8"))
    assert validate_stage("level1b",spec,stage,symbols)==(1034.65,239.25)
    spawns,counts,defs=check_source(stage,symbols,source_labels,args.swf)
    unsupported=set()
    shapes=load_shapes(args.swf,unsupported)
    assert not unsupported,("Unsupported source fill kinds",unsupported)
    bitmaps=load_bitmaps(args.bitmaps)
    rendered={str(sid):source_frame_series(sid,symbols,shapes,bitmaps,args.out)
              for sid in ACTOR_SYMBOLS}
    sheet=contact_sheet(args.out,rendered)
    instances=[]
    for p in spawns:
        m=p["matrix"]
        instances.append({"name":p["name"],"source_symbol_id":p["character_id"],
                          "game_depth":p["depth"],"original_parent_frame_zero_based":0,
                          "game_matrix":{k:m[k] for k in ("sx","sy","r0","r1","tx","ty")},
                          "source_level_local":[m["tx"]-1034.65,m["ty"]-239.25]})
    report={"batch":"LEVEL2-B09","original_sha256":original,
            "source_root_frame_zero_based":20,"source_game_id":1075,
            "source_level_id":783,"source_ground_id":782,
            "source_player_id":691,"source_player_game_depth":1127,
            "source_symbol_tag_codes":{str(k):v for k,v in defs.items()},
            "original_goon_family_counts":{str(k):v for k,v in counts.items()},
            "ten_original_game_named_spawn_instances":instances,
            "families":rendered,"contact_sheet":sheet,
            "source_library_only":True,"never_embed_as_complete_level":True,
            "original_parent_frame_intervals_not_proof_of_live_avm1_playback":True,
            "all_full_178_1x_frames_and_18_true_hq_label_starts":True,
            "goon_ai_combat_sound_and_stage_activation_not_implemented":True}
    (args.out/"B09_ORIGINAL_BANK_GOON_SOURCE_REPORT.json").write_text(
        json.dumps(report,indent=2)+"\n",encoding="utf-8")
    assert len(instances)==10 and sum(len(v["frame0_through_88_1x"]) for v in rendered.values())==178
    assert sum(len(v["source_reraster_6x_to_hq3x_label_views"]) for v in rendered.values())==18
    print("B09_ORIGINAL_GOON_SOURCE_GATE_PASS ten source spawns; "
          "178 original 1x parent frames; 18 true HQ3x label views",flush=True)

if __name__=="__main__":
    main()
