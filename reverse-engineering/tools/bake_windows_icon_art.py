#!/usr/bin/env python3
"""Batch06: reproduce Batman icon ART from the native game's HQ HUD.

Uses original sprite #716 yellow Health icon components #705/#709.
Outputs PNGs only: no ICO/RC/EXE embedding until separately approved Batch07.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

SIZES = (16, 24, 32, 48, 64, 128, 256, 512)
HUD_CROP = (310, 80, 530, 196)  # Yellow bat ONLY, no labels or Lives X
PANEL_ALPHA = 82  # Existing sprite #716 translucent HUD backing

def extract_original_bat(hud: Image.Image) -> Image.Image:
    if hud.size != (837, 246):
        raise ValueError(f"Expected 3x source HUD shell 837x246, got {hud.size}")
    image = hud.convert("RGBA").crop(HUD_CROP)
    bg = (96.0, 115.0, 116.0)
    pixels = []
    data = image.get_flattened_data() if hasattr(image, "get_flattened_data") else image.getdata()
    for red, green, blue, alpha in data:
        # Reverse source alpha-over: panel alpha 82 under the icon.
        source_a = min(1.0, max(0.0, (alpha-PANEL_ALPHA)/(255-PANEL_ALPHA)))
        if source_a <= 0.01:
            pixels.append((0,0,0,0))
            continue
        color = []
        for out_color, bg_color in zip((red,green,blue),bg):
            c = ((out_color*alpha/255.0)
                 - bg_color*(PANEL_ALPHA/255.0)*(1-source_a))/source_a
            color.append(round(min(255,max(0,c))))
        pixels.append(tuple(color)+(round(source_a*255),))
    out = Image.new("RGBA", image.size)
    out.putdata(pixels)
    bbox = out.getchannel("A").point(lambda a: 255 if a>=32 else 0).getbbox()
    if bbox is None:
        raise ValueError("No original yellow bat ink extracted")
    out = out.crop((max(0,bbox[0]-2), max(0,bbox[1]-2),
                    min(out.width,bbox[2]+2), min(out.height,bbox[3]+2)))
    if not (190 <= out.width <= 230 and 95 <= out.height <= 119):
        raise ValueError(f"Unexpected original bat art geometry: {out.size}")
    return out

def draw_master(bat: Image.Image) -> Image.Image:
    size = 512
    out = Image.new("RGBA", (size,size), (0,0,0,0))
    draw = ImageDraw.Draw(out)
    draw.rounded_rectangle((4,4,508,508),radius=104,
                           fill=(15,19,29,255),outline=(215,153,47,255),width=12)
    draw.rounded_rectangle((17,17,495,495),radius=95,
                           outline=(91,33,26,255),width=9)
    # Decorative Gotham-red moon only; no regenerated game character art.
    moon = Image.new("RGBA", (512,512),(0,0,0,0))
    p = moon.load()
    for y in range(72,442):
        for x in range(72,442):
            dist = ((x-256)**2/183**2+(y-254)**2/183**2)**0.5
            if dist > 1:
                continue
            t = max(0,1-dist)
            p[x,y] = (int(114+105*t),int(20+38*t),int(20+19*t),
                      int(190*min(1,(1-dist)*22)))
    out = Image.alpha_composite(out,moon)
    target_w = 438
    w,h = bat.size
    scaled = bat.resize((target_w,round(h*target_w/w)),Image.Resampling.LANCZOS)
    out.alpha_composite(scaled,((size-scaled.width)//2,(size-scaled.height)//2+8))
    return out

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--hud",type=Path,required=True,
                    help="Original-source-derived assets/hud_base_hq.png")
    ap.add_argument("--out",type=Path,required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True,exist_ok=True)
    bat = extract_original_bat(Image.open(args.hud).convert("RGBA"))
    master = draw_master(bat)
    report = {"source":"sprite #716, Health yellow #705/#709, baked HQ HUD",
              "crop":list(HUD_CROP),"master_size":[512,512],
              "extracted_bat_size":list(bat.size),
              "windows_exe_resource_embedded":False,
              "input_hud_sha256":hashlib.sha256(args.hud.read_bytes()).hexdigest(),
              "files":{}}
    for size in SIZES:
        img = master if size==512 else master.resize((size,size),Image.Resampling.LANCZOS)
        # Lanczos can spread a 1-3 alpha fringe into corner pixels at
        # small sizes (notably 24px). Keep all four taskbar corners clear.
        for corner in ((0,0),(0,size-1),(size-1,0),(size-1,size-1)):
            img.putpixel(corner,(0,0,0,0))
        file = args.out/f"batman-cobblebot-{size}.png"
        img.save(file,optimize=True)
        bbox = img.getchannel("A").getbbox()
        if bbox is None or bbox[2]-bbox[0]<size*.70 or bbox[3]-bbox[1]<size*.70:
            raise ValueError(f"Icon disappeared at real {size}px")
        report["files"][str(size)]={"sha256":hashlib.sha256(file.read_bytes()).hexdigest(),
                                    "ink_bbox":list(bbox)}
    # Compare real downscaled resolutions, enlarged ONLY in the QA contact sheet.
    sheet = Image.new("RGB",(660,508),(228,231,236))
    draw = ImageDraw.Draw(sheet)
    for i,size in enumerate((16,24,32,48,64,128)):
        tile = Image.open(args.out/f"batman-cobblebot-{size}.png").convert("RGBA")
        nearest = tile.resize((192,192),Image.Resampling.NEAREST)
        bg = Image.new("RGBA",(192,192),(235,237,242,255))
        bg.alpha_composite(nearest)
        x,y = i%3*220+14,i//3*254+8
        sheet.paste(bg.convert("RGB"),(x,y))
        draw.text((x+15,y+204),f"Actual {size}x{size} pixels",fill=(20,24,34))
    sheet.save(args.out/"batman-cobblebot-small-size-review.png",optimize=True)
    (args.out/"artwork-report.json").write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(report,indent=2))

if __name__=="__main__":
    main()
