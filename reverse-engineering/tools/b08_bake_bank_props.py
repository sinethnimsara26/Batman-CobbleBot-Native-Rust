#!/usr/bin/env python3
"""B08 original Bank Street #1075 game-level prop placement/depth proof.

Original FWS + SWF-authored transforms only. Outputs partial frame-0 1x cutouts,
independently rerasterized true 3x source cutouts, and authored-depth source
composites over static #783 street. No game executable or complete stage pack.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess
import sys

from PIL import Image, ImageChops, ImageDraw
from b03_bake_stage_asset_proofs import expect_original_source, static_filter, validate_stage
from bake_sprite_frames import (
    display_list, load_shapes, load_text_defs, render_symbol, symbol_bounds,
)
from bake_level_tiles import load_bitmaps
from b05_bake_bank_street_1x import source_definition_tags

SYMBOLS = {"can_": (785,4), "trash_": (790,4), "mailbox_": (793,1)}
NAMES = {
    "can_03":(785,1077,-3391.85,379.55,1.0),
    "can_02":(785,1079,-1600.8,379.55,1.0),
    "can_01":(785,1081,-376.95,379.55,1.0),
    "can_04":(785,1083,-5231.3,379.55,1.0),
    "trash_01":(790,1085,243.95,252.3,1.1087646484375),
    "trash_02":(790,1088,-1184.6,252.3,1.1087646484375),
    "trash_03":(790,1091,-2726.1,252.3,1.1087646484375),
    "trash_04":(790,1094,-4677.0,252.3,1.1087646484375),
    "mailbox_01":(793,1097,-4155.8,246.45,1.1978607177734375),
}
SOURCE_STAGE=(20,1075,783,782)
SOURCE_LEVEL_ORIGIN=(1034.65,239.25)
SOURCE_PLAYER_DEPTH=1127

def sha(data:bytes)->str:
    return hashlib.sha256(data).hexdigest()

def file_sha(path:Path)->str:
    return sha(path.read_bytes())

def check_props(stage,symbols,shapes):
    assert (stage["root_frame"],stage["game_symbol_id"],stage["level_symbol_id"],
            stage["ground"]["character_id"]) == SOURCE_STAGE
    named=[p for p in stage["named_game_instances"] if p.get("name") in NAMES]
    assert len(named)==9 and {p["name"] for p in named}==set(NAMES)
    assert not any(p["name"] in NAMES for p in stage["named_level_instances"])
    game_display=display_list(symbols[str(stage["game_symbol_id"])],0)
    level_display=display_list(symbols[str(stage["level_symbol_id"])],0)
    assert any(p.get("character_id")==782 for p in level_display.values())
    assert not {p.get("character_id") for p in level_display.values()} & {785,790,793}
    level_placement=[p for p in stage["named_game_instances"] if p.get("name")=="level"]
    assert len(level_placement)==1 and level_placement[0]["depth"]==1
    player_placement=[p for p in stage["named_game_instances"] if p.get("name")=="player"]
    assert len(player_placement)==1 and player_placement[0]["depth"]==SOURCE_PLAYER_DEPTH
    assert player_placement[0]["character_id"]==691
    result=[]
    for p in sorted(named,key=lambda p:p["depth"]):
        name=p["name"]; cid,depth,x,y,s=NAMES[name]; m=p["matrix"]
        assert p["character_id"]==cid and p["depth"]==depth and p["frame"]==0,(name,p)
        assert p["flags"]==38 and m["r0"]==0 and m["r1"]==0,(name,p)
        assert all(abs(m[k]-v)<1e-7 for k,v in
                   (("sx",s),("sy",s),("tx",x),("ty",y))), (name,m)
        live=game_display.get(depth)
        assert live is not None,(name,depth)
        assert live.get("name")==name and live.get("character_id")==cid,(name,live)
        raw=live["matrix"]
        assert all(abs(raw[k]-m[k])<1e-7 for k in ("sx","sy","tx","ty"))
        assert 1<depth<SOURCE_PLAYER_DEPTH
        assert str(cid) in symbols or cid in shapes, ("no source definition",cid)
        result.append(p)
    assert sorted(p["depth"] for p in result)==sorted({v[1] for v in NAMES.values()})
    return result

def prop_transform(p,scale:float,left:float,top:float):
    m=p["matrix"]
    return (m["sx"]*scale,m["r1"]*scale,m["r0"]*scale,m["sy"]*scale,
            (m["tx"]-SOURCE_LEVEL_ORIGIN[0]-left)*scale,
            (m["ty"]-SOURCE_LEVEL_ORIGIN[1]-top)*scale)

def cutout(p,shapes,symbols,bitmaps,folder):
    cid=p["character_id"]
    m=p["matrix"]
    level_matrix=(m["sx"],m["r1"],m["r0"],m["sy"],
                  m["tx"]-SOURCE_LEVEL_ORIGIN[0],
                  m["ty"]-SOURCE_LEVEL_ORIGIN[1])
    bounds=symbol_bounds(cid,0,level_matrix,symbols,shapes)
    assert all(math.isfinite(v) for v in bounds),(p["name"],bounds)
    left=math.floor(bounds[0])-3
    top=math.floor(bounds[1])-3
    right=math.ceil(bounds[2])+3
    bottom=math.ceil(bounds[3])+3
    w,h=right-left,bottom-top
    assert 2<w<=1000 and 2<h<=1000,(p["name"],bounds,w,h)
    results={}
    for output_scale,supersample in ((1,1),(3,2)):
        scale=output_scale*supersample
        margin=2 if scale>1 else 0
        canvas=Image.new("RGBA",((w+margin*2)*scale,(h+margin*2)*scale),(0,0,0,0))
        unsupported=set()
        missing=set()
        render_symbol(cid,0,prop_transform(p,scale,left-margin,top-margin),
                      canvas,symbols,shapes,bitmaps,unsupported,missing)
        assert not unsupported and not missing,(p["name"],unsupported,missing)
        if supersample>1:
            canvas=canvas.convert("RGBa").resize(
                ((w+margin*2)*output_scale,(h+margin*2)*output_scale),
                Image.Resampling.LANCZOS).convert("RGBA")
            pad=margin*output_scale
            canvas=canvas.crop((pad,pad,pad+w*output_scale,pad+h*output_scale))
        assert canvas.size==(w*output_scale,h*output_scale)
        assert canvas.getchannel("A").getbbox(),("empty source prop",p["name"])
        out=folder/("cutouts_1x" if output_scale==1 else "cutouts_3x")
        out.mkdir(parents=True,exist_ok=True)
        file=p["name"]+".png"
        path=out/file
        canvas.save(path,optimize=True)
        results[str(output_scale)]={"file":str(path.relative_to(folder)).replace("\\","/"),
                                    "png_sha256":file_sha(path),
                                    "rgba_sha256":sha(canvas.tobytes()),
                                    "pixel_size":list(canvas.size)}
    return {"level_local_pixel_bounds":[left,top,right,bottom],
            "source_vector_bounds":list(bounds),"images":results}

def source_scenery(args,stage,origin,props):
    columns=sorted({math.floor(p["matrix"]["tx"]/600)*600 for p in props})
    assert len(columns)>=7,columns
    dest=args.out/"street_static_sample_only"
    excluded=sorted(set(static_filter(stage))|{758,771})
    command=[sys.executable,str(Path(__file__).with_name("bake_level_tiles.py")),
             str(args.swf),str(args.symbols),str(dest),
             "--bitmaps",str(args.bitmaps),"--symbol","783",
             "--place-x",str(origin[0]),"--place-y",str(origin[1]),
             "--include-source-static-text"]
    for cid in excluded: command+=["--exclude-symbol",str(cid)]
    for x in columns:command+=["--only-tile",str(x),"0"]
    subprocess.run(command,check=True)
    m=json.loads((dest/"manifest.json").read_text(encoding="utf-8"))
    assert m["symbol"]==783 and m["partial_proof_only"] and m["never_embed_as_complete_level"]
    assert m["missing_ids"]==[] and m["unsupported_fills"]==[]
    assert m["source_static_text_enabled"] and m["excluded_symbols"]==excluded
    assert len(m["tiles"])==len(columns),(columns,m["tiles"])
    forbidden=args.out/"DO_NOT_PACK_B08_PARTIAL_LEVEL.bin"
    check=subprocess.run([sys.executable,
         str(Path(__file__).with_name("pack_level_tiles.py")),str(dest),str(forbidden)],
         capture_output=True,text=True)
    assert check.returncode!=0 and not forbidden.exists()
    assert "partial B03 source proof tiles cannot be packed" in check.stderr,check.stderr
    return {p["world_x"]:p["file"] for p in m["tiles"]},columns

def composed_proofs(out,tiles,columns,props,records):
    base_dir=out/"street_static_sample_only"
    outdir=out/"source_depth_composites_1x"
    outdir.mkdir(parents=True,exist_ok=True)
    composites=[]
    for x in columns:
        with Image.open(base_dir/tiles[x]) as src:
            frame=src.convert("RGBA")
        initial=sha(frame.tobytes())
        visible=[]
        for p in props:
            imgmeta=records[p["name"]]
            imagepath=out/imgmeta["images"]["1"]["file"]
            with Image.open(imagepath) as src:
                layer=src.convert("RGBA")
            left,top,*_=imgmeta["level_local_pixel_bounds"]
            placed_x=round(left+SOURCE_LEVEL_ORIGIN[0]-x)
            placed_y=round(top+SOURCE_LEVEL_ORIGIN[1])
            cut=Image.new("RGBA",frame.size,(0,0,0,0))
            cut.paste(layer,(placed_x,placed_y))
            if cut.getchannel("A").getbbox() is not None:
                visible.append(p["name"])
                frame.alpha_composite(cut)
        assert visible,("source prop absent at sample",x)
        assert sha(frame.tobytes())!=initial,("prop depth composition empty",x)
        fn=f"bank_game_depth_x{x:+06d}_y0000.png"
        dest=outdir/fn
        frame.save(dest,optimize=True)
        composites.append({"world_origin":[x,0],"visible_source_props":visible,
                           "file":str(dest.relative_to(out)).replace("\\","/"),
                           "png_sha256":file_sha(dest),
                           "rgba_sha256":sha(frame.tobytes())})
    return composites

def make_sheet(out,composites):
    cols=3;rows=math.ceil(len(composites)/cols)
    sheet=Image.new("RGB",(cols*612,rows*432+30),"#202633")
    draw=ImageDraw.Draw(sheet)
    for i,v in enumerate(composites):
        tx=(i%cols)*612; ty=(i//cols)*432
        with Image.open(out/v["file"]) as src:
            bg=Image.new("RGBA",(600,400),"#303744")
            bg.alpha_composite(src.convert("RGBA"))
        sheet.paste(bg.convert("RGB"),(tx+6,ty+24))
        draw.text((tx+7,ty+4),f"SWF game frame0 x{v['world_origin'][0]}: "+",".join(v["visible_source_props"]),fill="white")
    draw.text((8,rows*432+8),"SOURCE COMPOSITES ONLY; NOT live Flash, NO 1B gameplay",fill="#ffcc80")
    dest=out/"B08_BANK_PROPS_SOURCE_DEPTH_CONTACT_SHEET.png"
    sheet.save(dest,optimize=True)
    return {"file":dest.name,"png_sha256":file_sha(dest),"size":list(sheet.size)}

def main():
    ap=argparse.ArgumentParser()
    for name in ("swf","symbols","bitmaps","instances","spec","out"):
        ap.add_argument("--"+name,type=Path,required=True)
    args=ap.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    original_sha=expect_original_source(args.swf)
    symbols=json.loads(args.symbols.read_text(encoding="utf-8"))
    instance_data=json.loads(args.instances.read_text(encoding="utf-8"))
    stage=instance_data["level1b"]
    spec=json.loads(args.spec.read_text(encoding="utf-8"))["level1b"]
    origin=validate_stage("level1b",spec,stage,symbols)
    assert tuple(origin)==SOURCE_LEVEL_ORIGIN
    unsupported=set()
    shapes=load_shapes(args.swf,unsupported)
    assert not unsupported,("unknown shape fills",unsupported)
    load_text_defs(args.swf)
    defs=source_definition_tags(args.swf,{785,790,793})
    assert set(defs)=={785,790,793} and all(defs[k]==39 for k in defs),defs
    props=check_props(stage,symbols,shapes)
    bitmaps=load_bitmaps(args.bitmaps)
    records={}
    for p in props:
        data=cutout(p,shapes,symbols,bitmaps,args.out)
        records[p["name"]]=data
        print("B08_SOURCE_PROP_FRAME0",p["name"],p["character_id"],
              p["depth"],data["images"]["1"]["rgba_sha256"],flush=True)
    tiles,columns=source_scenery(args,stage,origin,props)
    composites=composed_proofs(args.out,tiles,columns,props,records)
    sheet=make_sheet(args.out,composites)
    entries=[]
    for p in props:
        m=p["matrix"]
        entries.append({
            "name":p["name"],"symbol_id":p["character_id"],"game_depth":p["depth"],
            "source_parent_symbol":1075,"original_frame_zero_based":0,
            "source_game_matrix":{k:m[k] for k in ("sx","sy","r0","r1","tx","ty")},
            "source_level_local_origin":[m["tx"]-origin[0],m["ty"]-origin[1]],
            **records[p["name"]],
        })
    report={
        "batch":"LEVEL2-B08","source_fws_sha256":original_sha,
        "source_root_zero_based":20,"source_game_symbol":1075,
        "source_street_level_symbol":783,"source_ground_symbol":782,
        "source_level_origin_game_local":list(origin),"source_sprite_frame_zero_based":0,
        "source_definition_tags":{str(k):v for k,v in defs.items()},
        "authored_game_scene_depth":{"level":1,"props":[p["depth"] for p in props],
                                   "player":1127},
        "counts":{"can_785":4,"trash_790":4,"mailbox_793":1},
        "game_vs_level_display_ownership_verified":True,
        "ground_not_in_source_visible_prop_images":True,
        "source_collision_status":"visual objects are separate from level-local ground #782; AVM1 prop interactions UNVERIFIED",
        "source_named_prop_instances":entries,
        "source_frame0_composite_proofs":composites,
        "contact_sheet":sheet,
        "partial_proof_only":True,"never_embed_as_complete_level":True,
        "source_library_stills_not_live_flash_gameplay":True,
        "no_native_gameplay_activation":True,
        "source_hq_frame0_reraster_at_6x_then_premultiplied_lanczos_3x":True,
    }
    (args.out/"B08_ORIGINAL_BANK_PROPS_SOURCE_REPORT.json").write_text(
        json.dumps(report,indent=2)+"\n",encoding="utf-8")
    print("B08_ORIGINAL_BANK_PROPS_SOURCE_GATE_PASS",len(props),"props",
          len(composites),"source 1x authored-depth composites",flush=True)

if __name__=="__main__":
    main()
