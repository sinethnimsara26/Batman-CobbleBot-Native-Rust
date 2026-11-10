#!/usr/bin/env python3
"""HUD-REAL-06B: shrink SWF-originated HUD icon vectors, HQ bake ONLY.

Adjust four proven Flash display-list placements in a throwaway symbols
metadata copy. Original symbols.json and the 1x source oracle are untouched.
Nested blue Batarang #693 must NOT move sibling text fields #700/#702.
Yellow Health is the composite of root placements #698/#705/#709; scale
their individual original vectors together around one source-space center.
"""
import argparse
import json
from pathlib import Path

def active_first_frame(symbol):
    return [p for p in symbol.get("placements",[]) if p.get("frame",0)==0]

def scale_matrix(placement, factor, anchor):
    m=placement.get("matrix")
    if m is None or any(k not in m for k in ("sx","sy","tx","ty")):
        raise ValueError("Missing original SWF placement matrix")
    original={k:float(m.get(k,0)) for k in ("sx","sy","r0","r1","tx","ty")}
    if abs(original["sx"])<0.01 or abs(original["sy"])<0.01:
        raise ValueError("Cannot scale degenerate HUD icon")
    for k in ("sx","sy","r0","r1"):
        if k in m: m[k]=original[k]*factor
    m["tx"]=anchor[0]+factor*(original["tx"]-anchor[0])
    m["ty"]=anchor[1]+factor*(original["ty"]-anchor[1])
    return {"child":placement["character_id"],"before":original,"after":m.copy()}

def exact_target(symbol,child_id):
    found=[p for p in active_first_frame(symbol) if p.get("character_id")==child_id]
    if len(found)!=1:
        raise ValueError(f"Expected exactly one original SWF child {child_id}: {len(found)}")
    return found[0]

def transform_symbols(source,layout):
    # Preserve original layers and change nothing outside four selected nodes.
    symbols=json.loads(json.dumps(source))
    root=symbols.get("716")
    blue=symbols.get("703")
    if root is None or blue is None:
        raise ValueError("Missing original SWF 716/703 sprite definitions")
    b=layout["hq_icon_resize"]
    factor=b["scale_percent"]/100
    if not 0.92<=factor<=0.98:
        raise ValueError("HQ-only icon scale must be 92..98%")
    if set(b["blue"])!={ "parent","child","center"} or b["blue"]["parent"]!=703 or b["blue"]["child"]!=693:
        raise ValueError("Blue icon source ID changed")
    if b["yellow"]["parent"]!=716 or b["yellow"]["children"]!=[698,705,709]:
        raise ValueError("Yellow icon composite source IDs changed")
    blue_changed=scale_matrix(exact_target(blue,693),factor,b["blue"]["center"])
    yellow_changed=[
        scale_matrix(exact_target(root,cid),factor,b["yellow"]["center"])
        for cid in b["yellow"]["children"]
    ]
    return symbols,{"SWF_sprite":716,"style":"HQ only","scale_percent":b["scale_percent"],
                    "blue":blue_changed,"yellow":yellow_changed,
                    "preserved":[697,700,702,706,707,708,710,711,713,714,715]}

def main():
    p=argparse.ArgumentParser()
    p.add_argument("symbols_json",type=Path)
    p.add_argument("layout_json",type=Path)
    p.add_argument("output_json",type=Path)
    p.add_argument("report_json",type=Path)
    args=p.parse_args()
    source=json.loads(args.symbols_json.read_text())
    layout=json.loads(args.layout_json.read_text())
    modified, report=transform_symbols(source,layout)
    args.output_json.parent.mkdir(parents=True,exist_ok=True)
    args.report_json.parent.mkdir(parents=True,exist_ok=True)
    args.output_json.write_text(json.dumps(modified,separators=(",",":")))
    args.report_json.write_text(json.dumps(report,indent=2)+"\n")
    print("HUD-REAL-06B SWF vector placements:",json.dumps(report))
if __name__=="__main__":
    main()
