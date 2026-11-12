#!/usr/bin/env python3
"""AF08: canonical original Bank Street prop timeline and AVM1 callback gate.
Offline source evidence only; no Flash runtime or active Level1B.
"""
from __future__ import annotations
import argparse,copy,hashlib,json
from pathlib import Path

CANON="3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed"
FAMILIES={"can":(785,"bgTouch",741856,741895,4),
          "trash":(790,"bgHit",741933,741972,4),
          "mailbox":(793,"bgHit",742010,742049,1)}
FUNCTIONS={"bgTouch":(147901,85,["hitTest","hit","play","onEnterFrame"]),
           "bgHit":(148009,125,["hitTest","hit","play","onEnterFrame","playerAttacking","metal1"])}
NAMES={"can_03":(785,1077),"can_02":(785,1079),"can_01":(785,1081),
       "can_04":(785,1083),"trash_01":(790,1085),"trash_02":(790,1088),
       "trash_03":(790,1091),"trash_04":(790,1094),"mailbox_01":(793,1097)}
def load(p):return json.loads(p.read_text(encoding="utf-8"))
def push(a):return [v.get("resolved") if v.get("resolved") is not None else v.get("value") for v in a.get("values",[])]
def flatten(ops):
    for op in ops:
        yield op
        if op.get("body"):yield from flatten(op["body"])
def callback_functions(actions):
    found={}
    for block in actions:
        if (block["context"],block["frame"])!=("root",15):continue
        for i,op in enumerate(block["actions"]):
            if op["op"] not in ("DefineFunction","DefineFunction2"):continue
            name=op.get("function")
            if not name and i and block["actions"][i-1]["op"]=="Push":
                v=push(block["actions"][i-1]);name=v[-1] if v else None
            if name in FUNCTIONS:
                assert name not in found,("duplicate source function",name)
                found[name]=op
    assert set(found)==set(FUNCTIONS),("missing source functions",found.keys())
    out={}
    for name,(offset,size,required) in FUNCTIONS.items():
        op=found[name]
        assert (op["op"],op["body_offset"],op["code_size"])==("DefineFunction2",offset,size),(name,op)
        body=list(flatten(op.get("body",[])))
        values=[v for a in body if a["op"]=="Push" for v in push(a)]
        assert set(required)<=set(values),(name,"required bytecode operands missing",values)
        out[name]={"body_absolute_fws_offset":offset,"bytecode_size":size,
                   "required_operands":required,
                   "original_source_operations":[
                       {"offset":a.get("offset"),"op":a["op"],
                        **({"push":push(a)} if a["op"]=="Push" else {}),
                        **({"branch":a["branch"]} if "branch" in a else {})}
                       for a in body]}
    return out
def dispatch(actions):
    roots=[b for b in actions if (b["context"],b["frame"])==("root",20)]
    assert len(roots)==2 and {b["tag_offset"] for b in roots}=={740341,742716}
    ops=next(b["actions"] for b in roots if b["tag_offset"]==740341)
    found={}
    for prefix,(sid,fn,name_off,cb_off,count) in FAMILIES.items():
        matches=[i for i,a in enumerate(ops) if a.get("offset")==name_off and a["op"]=="Push" and push(a)==[prefix]]
        assert len(matches)==1,(prefix,"prefix match missing")
        i=matches[0]
        assert [a["op"] for a in ops[i+1:i+4]]==["Equals2","Not","If"],prefix
        assert ops[i-1]["op"]=="CallMethod",prefix
        cb=[i for i,a in enumerate(ops) if a.get("offset")==cb_off]
        assert len(cb)==1,(prefix,"callback not found")
        j=cb[0]
        assert ops[j]["op"]=="Push" and push(ops[j])==[fn],prefix
        assert [a["op"] for a in ops[j-1:j+3]]==["GetVariable","Push","GetMember","SetMember"],prefix
        assert ops[j-2]["op"]=="Push" and push(ops[j-2])==["onEnterFrame","_global"],prefix
        found[prefix]={"symbol":sid,"instances":count,"source_on_enter_frame":fn,
                       "source_prefix_fws_offset":name_off,"handler_fws_offset":cb_off}
    return found
def sprite_graph(symbols,actions,roots):
    byctx={}
    for block in actions:byctx.setdefault(block["context"],[]).append(block)
    graph={}
    def visit(cid,parents=()):
        if str(cid) not in symbols:return
        assert cid not in parents,("recursive sprite display definitions",cid)
        if cid in graph:return
        s=symbols[str(cid)];frame_count=s["frames"]
        assert isinstance(frame_count,int) and frame_count>0,(cid,frame_count)
        placed=s.get("placements",[]);removed=s.get("removals",[])
        scripts=byctx.get("sprite:"+str(cid),[])
        assert len(scripts)==s.get("action_blocks"),(cid,"source sprite action count drift")
        assert all(0<=x["frame"]<frame_count for x in placed+removed+scripts),cid
        kids=sorted({p["character_id"] for p in placed if p.get("character_id") is not None})
        graph[cid]={"declared_frames":frame_count,"frames_zero_based":[0,frame_count-1],
                    "labels":s.get("labels",[]),"child_ids":kids,
                    "placement_count":len(placed),"removal_count":len(removed),
                    "DoAction_count":len(scripts),
                    "DoAction_frames_zero_based":[b["frame"] for b in scripts],
                    "DoAction_tag_offsets":[b["tag_offset"] for b in scripts],
                    "direct_DoAction_strings":sorted({str(v) for b in scripts for v in b.get("strings",[])})}
        for kid in kids:visit(kid,parents+(cid,))
    for cid in roots:visit(cid)
    assert all(cid in graph for cid in roots)
    return graph
def main():
    ap=argparse.ArgumentParser()
    for n in ("swf","symbols","actions","instances","out"):ap.add_argument("--"+n,type=Path,required=True)
    args=ap.parse_args()
    data=args.swf.read_bytes();sha=hashlib.sha256(data).hexdigest()
    assert data[:3]==b"FWS" and len(data)==2819100 and sha==CANON,("noncanonical FWS",sha)
    symbols=load(args.symbols);actions=load(args.actions);inst=load(args.instances)["level1b"]
    assert (inst["root_frame"],inst["game_symbol_id"],inst["level_symbol_id"],inst["ground"]["character_id"])==(20,1075,783,782)
    named={p["name"]:p for p in inst["named_game_instances"] if p.get("name") in NAMES}
    assert set(named)==set(NAMES) and not {p.get("name") for p in inst["named_level_instances"]}&set(NAMES)
    for name,(sid,d) in NAMES.items():assert (named[name]["character_id"],named[name]["depth"],named[name]["frame"])==(sid,d,0)
    bindings=dispatch(actions);functions=callback_functions(actions)
    graph=sprite_graph(symbols,actions,[785,790,793])
    broken=copy.deepcopy(actions)
    for b in broken:
        if b["context"]=="root" and b["frame"]==20 and b["tag_offset"]==740341:
            for op in b["actions"]:
                if op.get("offset")==741895:
                    op["values"][0]["resolved"]="bgHit"
    try:dispatch(broken)
    except AssertionError:negative="PASS_MALFORMED_CAN_CALLBACK_REJECTED"
    else:raise AssertionError("mutation guard failed")
    report={"batch":"AF08","source_fws_sha256":sha,"root_frame_zero_based":20,
            "classification":"ORIGINAL_STATIC_AVM1_AND_SPRITE_SOURCE_PROOF_NOT_FLASH_PLAYBACK",
            "bank_game_symbol":1075,"level_symbol":783,"ground_symbol":782,
            "nine_original_prop_instances":[{"name":n,"symbol_id":sid,"game_depth":d} for n,(sid,d) in NAMES.items()],
            "family_handlers":bindings,"global_callbacks":functions,
            "reachable_source_sprite_definitions":{str(k):v for k,v in sorted(graph.items())},
            "negative_test":negative,"playable_level1b":False,
            "warning":"Original function bytecode observed; hitTest live results, sprite advance and actor collisions remain unverified."}
    args.out.mkdir(parents=True,exist_ok=True)
    (args.out/"AF08_ORIGINAL_BANK_PROP_AVM1_AND_TIMELINES.json").write_text(json.dumps(report,indent=2)+"\n")
    print("AF08_ORIGINAL_SOURCE_PASS",[(k,v["source_on_enter_frame"]) for k,v in bindings.items()])
    print("AF08_SOURCE_PROP_TIMELINES",[(k,graph[k]["declared_frames"],graph[k]["DoAction_count"]) for k in (785,790,793)])
    print("AF08_NEGATIVE_MUTATED_CALLBACK_REJECTED")
if __name__=="__main__":main()
