#!/usr/bin/env python3
from pathlib import Path
import argparse, json, struct
from swf_re import read_rect, tags

def collect_sprite_payloads(data,start,end,out):
    for code,pl,_,_ in tags(data,start,end):
        if code==39 and len(pl)>=4:
            sid=struct.unpack_from("<H",pl,0)[0]
            out[sid]=pl
            collect_sprite_payloads(pl,4,len(pl),out)

def sound_events(sprite_payload):
    frame=0; out=[]
    for code,pl,_,_ in tags(sprite_payload,4,len(sprite_payload)):
        if code==1:
            frame+=1
        elif code==15 and len(pl)>=2:
            out.append({"frame":frame,"sound_id":struct.unpack_from("<H",pl,0)[0]})
    return out

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("swf",type=Path)
    ap.add_argument("symbols_json",type=Path)
    ap.add_argument("player_manifest",type=Path)
    ap.add_argument("sound_manifest",type=Path)
    ap.add_argument("output",type=Path)
    ns=ap.parse_args()
    b=ns.swf.read_bytes()
    _,pos=read_rect(b,8); pos+=4
    sprites={}
    collect_sprite_payloads(b,pos,len(b),sprites)
    player=json.loads(ns.player_manifest.read_text())
    sounds=json.loads(ns.sound_manifest.read_text())
    byid={x["id"]:x.get("export_name",f"sound_{x['id']}") for x in sounds}
    result=[]
    for state in player["states"]:
        sid=state["child_symbol"]
        ev=sound_events(sprites[sid])
        result.append({"name":state["name"],"child_symbol":sid,"events":[{**x,"name":byid.get(x["sound_id"],f"sound_{x['sound_id']}")} for x in ev]})
    ns.output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result,indent=2))
if __name__=="__main__": main()
