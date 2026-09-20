#!/usr/bin/env python3
"""Render a SWF DefineText character with its embedded DefineFont2 glyphs.

Development-only fidelity tooling for static text that the main sprite baker
cannot yet rasterize. Glyph outlines come from the SWF's embedded font; no host
font substitution is used.
"""
from __future__ import annotations

from pathlib import Path
import argparse, json, math, struct
from PIL import Image

from swf_re import Bits, read_matrix, read_rect, tags
from render_collision_masks import point, rasterize_winding


def glyph_contours(data: bytes, start: int, end: int):
    br = Bits(data, start)
    fill_bits = br.u(4)
    line_bits = br.u(4)
    x = y = 0
    contours = []
    current = []

    def flush():
        nonlocal current
        if len(current) >= 3:
            contours.append(current)
        current = []

    while br.pos < end:
        edge = br.u(1)
        if edge:
            straight = br.u(1)
            n = br.u(4) + 2
            x0, y0 = x, y
            if straight:
                general = br.u(1)
                if general:
                    dx, dy = br.s(n), br.s(n)
                elif br.u(1):
                    dx, dy = 0, br.s(n)
                else:
                    dx, dy = br.s(n), 0
                x += dx
                y += dy
                pts = [(x0, y0), (x, y)]
            else:
                cdx, cdy = br.s(n), br.s(n)
                adx, ady = br.s(n), br.s(n)
                cx, cy = x + cdx, y + cdy
                ex, ey = cx + adx, cy + ady
                length = math.hypot(cx - x, cy - y) + math.hypot(ex - cx, ey - cy)
                steps = max(2, min(40, int(length / 40) + 1))
                pts = []
                for i in range(steps + 1):
                    t = i / steps
                    u = 1 - t
                    pts.append((
                        u*u*x + 2*u*t*cx + t*t*ex,
                        u*u*y + 2*u*t*cy + t*t*ey,
                    ))
                x, y = ex, ey
            if not current:
                current = list(pts)
            elif current[-1] == pts[0]:
                current.extend(pts[1:])
            else:
                flush()
                current = list(pts)
            continue

        new_styles = br.u(1)
        state_line = br.u(1)
        state_fill1 = br.u(1)
        state_fill0 = br.u(1)
        move = br.u(1)
        if not (new_styles or state_line or state_fill1 or state_fill0 or move):
            flush()
            break
        if move:
            flush()
            n = br.u(5)
            x, y = br.s(n), br.s(n)
        if state_fill0:
            br.u(fill_bits)
        if state_fill1:
            br.u(fill_bits)
        if state_line:
            br.u(line_bits)
        if new_styles:
            raise ValueError("DefineFont2 glyph unexpectedly contains new styles")
    return contours


def parse_font2(payload: bytes):
    font_id = struct.unpack_from("<H", payload, 0)[0]
    p = 2
    flags = payload[p]; p += 1
    p += 1  # LanguageCode
    name_len = payload[p]; p += 1
    name = payload[p:p+name_len].decode("latin1", "replace").rstrip("\0")
    p += name_len
    count = struct.unpack_from("<H", payload, p)[0]; p += 2
    wide_offsets = bool(flags & 0x08)
    wide_codes = bool(flags & 0x04)
    fmt, size = ("<I", 4) if wide_offsets else ("<H", 2)
    table_start = p
    offsets = [struct.unpack_from(fmt, payload, p + i*size)[0] for i in range(count)]
    p += count * size
    code_offset = struct.unpack_from(fmt, payload, p)[0]
    code_pos = table_start + code_offset
    codes = [
        struct.unpack_from("<H", payload, code_pos + i*2)[0]
        if wide_codes else payload[code_pos + i]
        for i in range(count)
    ]
    glyphs = []
    for i in range(count):
        start = table_start + offsets[i]
        end = table_start + (offsets[i+1] if i + 1 < count else code_offset)
        glyphs.append(glyph_contours(payload, start, end))
    return font_id, {"name": name, "codes": codes, "glyphs": glyphs}


def parse_define_text(payload: bytes):
    character_id = struct.unpack_from("<H", payload, 0)[0]
    p = 2
    bounds, p = read_rect(payload, p)
    matrix, p = read_matrix(payload, p)
    glyph_bits = payload[p]
    advance_bits = payload[p+1]
    p += 2

    records = []
    font = None
    color = (255, 255, 255, 255)
    xoff = yoff = 0
    height = 1024
    while True:
        flags = payload[p]; p += 1
        if flags == 0:
            break
        has_font = bool(flags & 0x08)
        has_color = bool(flags & 0x04)
        has_y = bool(flags & 0x02)
        has_x = bool(flags & 0x01)
        if has_font:
            font = struct.unpack_from("<H", payload, p)[0]; p += 2
        if has_color:
            color = tuple(payload[p:p+3]) + (255,); p += 3
        if has_x:
            xoff = struct.unpack_from("<h", payload, p)[0]; p += 2
        if has_y:
            yoff = struct.unpack_from("<h", payload, p)[0]; p += 2
        if has_font:
            height = struct.unpack_from("<H", payload, p)[0]; p += 2
        count = payload[p]; p += 1
        br = Bits(payload, p)
        glyphs = [(br.u(glyph_bits), br.s(advance_bits)) for _ in range(count)]
        br.align(); p = br.pos
        records.append({
            "font": font, "color": color, "x": xoff, "y": yoff,
            "height": height, "glyphs": glyphs,
        })
    return character_id, bounds, matrix, records


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("swf", type=Path)
    ap.add_argument("output", type=Path)
    ap.add_argument("--character", type=int, required=True)
    ns = ap.parse_args()

    data = ns.swf.read_bytes()
    _, pos = read_rect(data, 8)
    pos += 4
    font_payloads = {}
    text_payload = None
    for code, payload, _, _ in tags(data, pos, len(data)):
        if code == 48:
            font_payloads[struct.unpack_from("<H", payload, 0)[0]] = payload
        elif code == 11 and struct.unpack_from("<H", payload, 0)[0] == ns.character:
            text_payload = payload
    if text_payload is None:
        raise SystemExit(f"DefineText character {ns.character} not found")

    character_id, bounds, matrix, records = parse_define_text(text_payload)
    required_fonts = {r["font"] for r in records}
    fonts = {}
    for font_id in required_fonts:
        payload = font_payloads.get(font_id)
        if payload is None:
            raise SystemExit(f"DefineFont2 {font_id} not found")
        parsed_id, font = parse_font2(payload)
        fonts[parsed_id] = font

    xmin, xmax, ymin, ymax = [v / 20.0 for v in bounds]
    pad = 2
    origin_x, origin_y = xmin - pad, ymin - pad
    width = math.ceil(xmax - xmin) + pad * 2
    height = math.ceil(ymax - ymin) + pad * 2
    canvas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    strings = []

    text_matrix = (
        matrix["sx"], matrix["r1"], matrix["r0"], matrix["sy"],
        matrix["tx"], matrix["ty"],
    )
    for record in records:
        font = fonts[record["font"]]
        strings.append("".join(chr(font["codes"][i]) for i, _ in record["glyphs"]))
        cursor_x = record["x"] / 20.0
        baseline_y = record["y"] / 20.0
        glyph_scale = record["height"] / (1024.0 * 20.0)
        for glyph_index, advance in record["glyphs"]:
            contours = []
            for contour in font["glyphs"][glyph_index]:
                transformed = []
                for gx, gy in contour:
                    px = cursor_x + gx * glyph_scale
                    py = baseline_y + gy * glyph_scale
                    transformed.append(point(text_matrix, (px, py)))
                if transformed:
                    contours.append(transformed)
            if contours:
                mask = Image.new("1", (width, height), 0)
                rasterize_winding(mask, contours, origin_x, origin_y, False)
                ink = Image.new("RGBA", (width, height), record["color"])
                clear = Image.new("RGBA", (width, height), (0, 0, 0, 0))
                canvas.alpha_composite(Image.composite(ink, clear, mask))
            cursor_x += advance / 20.0

    ns.output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(ns.output, optimize=True)
    meta = {
        "character_id": character_id,
        "size": [width, height],
        "origin_in_symbol_px": [origin_x, origin_y],
        "bounds_px": [xmin, ymin, xmax, ymax],
        "fonts": [{"id": k, "name": v["name"]} for k, v in sorted(fonts.items())],
        "text_records": strings,
    }
    ns.output.with_suffix(".json").write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(meta, indent=2))


if __name__ == "__main__":
    main()