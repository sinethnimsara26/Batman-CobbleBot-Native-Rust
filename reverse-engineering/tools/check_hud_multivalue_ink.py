#!/usr/bin/env python3
"""CI-only HQ HUD numeric ink-mask test. Never imported by native game."""
import json
from pathlib import Path
from PIL import Image, ImageChops

S = 3
CANVAS = (600 * S, 115 * S)
INK_THRESHOLD = 32
OPAQUE_SHELL_THRESHOLD = 208
CASES = (
    ("b20_l10_s1234", 20, 10, 1234),
    ("b99_l12_s123456", 99, 12, 123456),
    ("b20_l99_s2147483647", 20, 99, 2147483647),
)

def binary(im, threshold):
    return im.point(lambda a: 255 if a >= threshold else 0)

def intersection_count(a, b):
    return ImageChops.multiply(a, b).histogram()[255]

def positioned_mask(alpha, x, y):
    output = Image.new("L", CANVAS, 0)
    output.paste(alpha, (x, y))
    return binary(output, INK_THRESHOLD)

def glyph_masks(atlas, value, anchor_x, center_y, budget, preferred, centered):
    cell_w = atlas.width // 10
    digits = str(max(0, value))
    requested = cell_w if preferred <= 0 else min(preferred, cell_w)
    room = max(budget, cell_w)
    if len(digits) > 1 and cell_w + (len(digits) - 1) * requested > room:
        step = max(1, min(requested, (room - cell_w) // (len(digits) - 1)))
    else:
        step = requested
    total = cell_w + step * (len(digits) - 1)
    x = anchor_x - total // 2 if centered else anchor_x
    y = center_y - atlas.height // 2
    masks = []
    for i, digit in enumerate(digits):
        idx = int(digit)
        alpha = atlas.crop((idx * cell_w, 0, (idx + 1) * cell_w, atlas.height)).getchannel("A")
        masks.append(positioned_mask(alpha, x + i * step, y))
    return masks, {"text": digits, "step": step, "cell_width": cell_w, "total_span": total}

def run():
    layout = json.loads(Path("reverse-engineering/manifests/hud-hq-layout.json").read_text())
    meta = json.loads(Path("assets/hud_hq.json").read_text())
    small = Image.open("assets/hud_digits_small_hq.png").convert("RGBA")
    large = Image.open("assets/hud_digits_large_hq.png").convert("RGBA")
    shell = Image.open("assets/hud_base_hq.png").convert("RGBA")
    anchor = meta["anchor_in_bitmap"]
    shell_alpha = Image.new("L", CANVAS, 0)
    shell_alpha.paste(shell.getchannel("A"),
                      (round(159 * S - anchor[0]), round(-11 * S - anchor[1])))
    opaque_shell = binary(shell_alpha, OPAQUE_SHELL_THRESHOLD)
    output = {
        "method": "per-pixel alpha from Windows-generated 3x digit atlases",
        "ink_alpha_threshold": INK_THRESHOLD,
        "shell_opaque_threshold": OPAQUE_SHELL_THRESHOLD,
        "shell_mask_scope": "opaque icon/static-label ink; not the translucent panel",
        "canvas": list(CANVAS), "cases": [], "overlaps": [],
    }
    for name, b_count, l_count, score in CASES:
        b, l, sc = (layout["counters"][n] for n in ("batarangs", "lives", "score"))
        bm, bd = glyph_masks(small, b_count, b["center"][0]*S, b["center"][1]*S,
                             b["max_width"]*S, b["digit_advance"]*S, True)
        lm, ld = glyph_masks(small, l_count, l["left"]*S, l["center_y"]*S,
                             l["max_width"]*S, l["digit_advance"]*S, False)
        score_atlas = small if (large.width//10)*len(str(score)) > sc["max_width"]*S else large
        sm, sd = glyph_masks(score_atlas, score, sc["center"][0]*S, sc["center"][1]*S,
                             sc["max_width"]*S, 0, True)
        groups = {"batarangs":bm, "lives":lm, "score":sm}
        case = {"name":name,"values":[b_count,l_count,score],
                "score_atlas":"small" if score_atlas is small else "large",
                "geometry":{"batarangs":bd,"lives":ld,"score":sd},
                "within_counter":{},"counter_vs_opaque_shell":{},"cross_counter":{}}
        composites = {}
        for field, digits in groups.items():
            combined = Image.new("L", CANVAS, 0)
            collision = 0
            for glyph in digits:
                collision += intersection_count(combined, glyph)
                combined = ImageChops.lighter(combined, glyph)
            composites[field] = combined
            case["within_counter"][field] = collision
            case["counter_vs_opaque_shell"][field] = intersection_count(combined, opaque_shell)
        fields = list(composites)
        for i, field in enumerate(fields):
            for other in fields[i+1:]:
                case["cross_counter"][field+"_vs_"+other] = intersection_count(composites[field], composites[other])
        output["cases"].append(case)
        for category in ("within_counter", "counter_vs_opaque_shell", "cross_counter"):
            for field, pixels in case[category].items():
                if pixels:
                    output["overlaps"].append({"case":name,"category":category,"field":field,"pixels":pixels})
    path = Path(".generated/windows-gui-pilot/hud-multivalue-ink-report.json")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(output,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"checked_cases":len(CASES),"overlaps":output["overlaps"]},indent=2))
    if output["overlaps"]:
        raise AssertionError("HUD glyph/static ink collision detected; see "+str(path))

if __name__ == "__main__":
    run()
