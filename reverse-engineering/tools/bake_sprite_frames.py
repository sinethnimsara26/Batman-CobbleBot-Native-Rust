#!/usr/bin/env python3
"""Bake one SWF sprite frame into a PNG for native-runtime asset generation.

This is development-only tooling. It interprets the source display list and
vector/bitmap/gradient fills and embedded DefineText glyphs; the resulting PNG
is intended to be embedded by Rust, not loaded from Flash at runtime.
"""
from __future__ import annotations

from pathlib import Path
import argparse
from functools import lru_cache
import json
import math
import struct

from PIL import Image, ImageChops, ImageDraw

from swf_re import Bits, read_matrix, read_rect, tags
from render_collision_masks import affine, compose, point, rasterize_winding
from bake_static_text import parse_define_text, parse_font2

SHAPE_TAGS = {2, 22, 32, 83}

TEXT_DEFS = {}
FONT_DEFS = {}


def _read_fill_style(data: bytes, pos: int, alpha: bool, unsupported: set[str]):
    kind = data[pos]
    pos += 1
    if kind == 0:
        r, g, b = data[pos : pos + 3]
        pos += 3
        a = data[pos] if alpha else 255
        if alpha:
            pos += 1
        return (r, g, b, a), pos
    if kind in (0x10, 0x12, 0x13):
        matrix, pos = read_matrix(data, pos)
        if pos >= len(data):
            raise ValueError("truncated gradient fill")
        flags = data[pos]
        spread_mode = (flags >> 6) & 0x03
        interpolation_mode = (flags >> 4) & 0x03
        count = flags & 0x0F
        pos += 1
        focal_point = 0.0
        if kind == 0x13:
            if pos + 2 > len(data):
                raise ValueError("truncated focal gradient")
            focal_point = struct.unpack_from("<h", data, pos)[0] / 256.0
            pos += 2
        stops = []
        for _ in range(count):
            if pos >= len(data):
                raise ValueError("truncated gradient stop")
            ratio = data[pos]
            pos += 1  # ratio
            channels = 4 if alpha else 3
            if pos + channels > len(data):
                raise ValueError("truncated gradient color")
            color = tuple(data[pos : pos + channels])
            pos += channels
            if channels == 3:
                color += (255,)
            stops.append((ratio, color))
        if not stops:
            return None, pos
        if interpolation_mode in (2, 3):
            unsupported.add("reserved_gradient_interpolation")
        if kind == 0x13:
            unsupported.add("focal_gradient")
        return {
            "gradient": kind,
            "matrix": matrix,
            "stops": tuple(stops),
            "spread": spread_mode,
            "interpolation": interpolation_mode,
            "focal": focal_point,
        }, pos
    if kind in (0x40, 0x41, 0x42, 0x43):
        bitmap_id = struct.unpack_from("<H", data, pos)[0]
        pos += 2
        matrix, pos = read_matrix(data, pos)
        # Fill matrices are stored against SWF twip coordinates, while all
        # recursively composed placement matrices in this baker use pixels.
        for key in ("sx", "sy", "r0", "r1"):
            matrix[key] /= 20.0
        return {
            "bitmap_id": bitmap_id,
            "matrix": matrix,
            "repeat": kind in (0x40, 0x42),
            "smooth": kind in (0x42, 0x43),
        }, pos
    raise ValueError(f"unsupported fill style 0x{kind:02x}")


def _read_fill_array(data: bytes, pos: int, alpha: bool, unsupported: set[str]):
    count = data[pos]
    pos += 1
    if count == 0xFF:
        count = struct.unpack_from("<H", data, pos)[0]
        pos += 2
    fills = []
    for _ in range(count):
        color, pos = _read_fill_style(data, pos, alpha, unsupported)
        fills.append(color)
    return fills, pos


def _read_line_array(data: bytes, pos: int, alpha: bool, shape4: bool, unsupported: set[str]):
    count = data[pos]
    pos += 1
    if count == 0xFF:
        count = struct.unpack_from("<H", data, pos)[0]
        pos += 2
    lines = []
    for _ in range(count):
        width = struct.unpack_from("<H", data, pos)[0] / 20.0
        pos += 2
        if shape4:
            bits = Bits(data, pos)
            bits.u(2)  # start cap
            join = bits.u(2)
            has_fill = bits.u(1)
            bits.u(1); bits.u(1); bits.u(1); bits.u(5); bits.u(1); bits.u(2)
            bits.align()
            pos = bits.pos
            if join == 2:
                pos += 2  # miter limit factor
            if has_fill:
                fill, pos = _read_fill_style(data, pos, alpha, unsupported)
                if isinstance(fill, tuple):
                    color = fill
                else:
                    unsupported.add("shape4_filled_stroke")
                    color = (255, 255, 255, 255)
            else:
                channels = 4 if alpha else 3
                color = tuple(data[pos : pos + channels])
                pos += channels
                if channels == 3:
                    color += (255,)
        else:
            channels = 4 if alpha else 3
            color = tuple(data[pos : pos + channels])
            pos += channels
            if channels == 3:
                color += (255,)
        lines.append({"width": width, "color": color})
    return lines, pos


def _flush(active: dict[int, list[tuple[float, float]]], paths: dict[int, list[list[tuple[float, float]]]], style: int | None = None):
    keys = list(active) if style is None else [style]
    for key in keys:
        contour = active.pop(key, None)
        if contour and len(contour) >= 3:
            paths.setdefault(key, []).append(contour)


def _append_edge(active, paths, style_index, edge, reverse=False):
    if style_index == 0:
        return
    contour = list(reversed(edge)) if reverse else edge
    key = style_index - 1
    current = active.get(key)
    if current is None:
        active[key] = list(contour)
    elif reverse and contour[-1] == current[0]:
        # FillStyle0 owns the left side of an edge, so its directed edge is
        # reversed. Preserve that direction by prepending it to the current
        # contour; appending breaks multi-edge outlines and their winding.
        contour.extend(current[1:])
        active[key] = list(contour)
    elif current[-1] == contour[0]:
        current.extend(contour[1:])
    else:
        _flush(active, paths, key)
        active[key] = list(contour)


def _stitch_contours(contours):
    """Join directed SWF fill fragments without changing their winding."""
    remaining=[list(c) for c in contours if len(c)>=2]
    out=[]
    while remaining:
        chain=remaining.pop(0)
        changed=True
        while changed and remaining:
            changed=False
            for i,candidate in enumerate(remaining):
                if chain[-1] == candidate[0]:
                    chain.extend(candidate[1:])
                    remaining.pop(i); changed=True; break
                if candidate[-1] == chain[0]:
                    chain=candidate[:-1] + chain
                    remaining.pop(i); changed=True; break
        if len(chain)>=3:
            out.append(chain)
    return out


def _pending_add(pending, style_id, segment, flip=False):
    """Ruffle-style directed edge-soup linking for one SWF fill style."""
    if style_id <= 0 or len(segment) < 2:
        return
    index = style_id - 1
    if index >= len(pending):
        return
    new_segment = list(reversed(segment)) if flip else list(segment)
    segments = pending[index]
    start_open = True
    end_open = True
    i = 0
    while (start_open or end_open) and i < len(segments):
        other = segments[i]
        if start_open and other[-1] == new_segment[0]:
            new_segment = other + new_segment[1:]
            segments.pop(i)
            start_open = False
        elif end_open and new_segment[-1] == other[0]:
            new_segment = new_segment + other[1:]
            segments.pop(i)
            end_open = False
        else:
            i += 1
    segments.append(new_segment)


def parse_shape(payload: bytes, tag: int, unsupported: set[str]):
    """Convert Flash's directed edge soup into ordered drawing layers.

    This follows Ruffle's ShapeConverter model: FillStyle0 and FillStyle1 own
    independent active paths, FillStyle0 is flipped when flushed, arbitrary
    edge fragments are linked only by directed endpoints, and StateNewStyles
    terminates the current drawing layer instead of extending style arrays.
    """
    _, pos = struct.unpack_from("<H", payload, 0)[0], 2
    _, pos = read_rect(payload, pos)
    has_alpha = tag in (32, 83)

    # Match Flash/Ruffle: legacy DefineShape/2/3 use even-odd.
    # DefineShape4 can explicitly select non-zero winding.
    even_odd = True
    if tag == 83:
        _, pos = read_rect(payload, pos)
        uses_fill_winding = bool(payload[pos] & 0x04)
        even_odd = not uses_fill_winding
        pos += 1

    fills, pos = _read_fill_array(payload, pos, has_alpha, unsupported)
    lines, pos = _read_line_array(payload, pos, has_alpha, tag == 83, unsupported)
    bits = Bits(payload, pos)
    fill_bits = bits.u(4)
    line_bits = bits.u(4)

    x = y = 0
    fill0 = fill1 = lineidx = 0
    active0 = [(x, y)]
    active1 = [(x, y)]
    active_line = [(x, y)]
    pending_fills = [[] for _ in fills]
    pending_lines = [[] for _ in lines]
    layers = []

    def reset_active(start):
        nonlocal active0, active1, active_line
        active0 = [start]
        active1 = [start]
        active_line = [start]

    def flush_fill(which, flip, start):
        nonlocal active0, active1
        active = active0 if which == 0 else active1
        style = fill0 if which == 0 else fill1
        if style > 0 and len(active) >= 2:
            _pending_add(pending_fills, style, active, flip)
        if which == 0:
            active0 = [start]
        else:
            active1 = [start]

    def flush_stroke(start):
        nonlocal active_line
        if lineidx > 0 and len(active_line) >= 2 and lineidx - 1 < len(pending_lines):
            pending_lines[lineidx - 1].append(list(active_line))
        active_line = [start]

    def flush_paths(start):
        flush_fill(1, False, start)
        flush_fill(0, True, start)
        flush_stroke(start)

    def emit_layer(start):
        nonlocal pending_fills, pending_lines
        flush_paths(start)
        if any(pending_fills) or any(pending_lines):
            layers.append({
                "fills": fills,
                "paths": {i: segs for i, segs in enumerate(pending_fills) if segs},
                "lines": lines,
                "line_paths": {i: segs for i, segs in enumerate(pending_lines) if segs},
                "even_odd": even_odd,
            })
        pending_fills = [[] for _ in fills]
        pending_lines = [[] for _ in lines]

    while True:
        edge_record = bits.u(1)
        if edge_record:
            straight = bits.u(1)
            bit_count = bits.u(4) + 2
            start_point = (x, y)
            if straight:
                general = bits.u(1)
                if general:
                    dx, dy = bits.s(bit_count), bits.s(bit_count)
                elif bits.u(1):
                    dx, dy = 0, bits.s(bit_count)
                else:
                    dx, dy = bits.s(bit_count), 0
                x += dx
                y += dy
                edge_points = [start_point, (x, y)]
            else:
                cdx, cdy = bits.s(bit_count), bits.s(bit_count)
                adx, ady = bits.s(bit_count), bits.s(bit_count)
                control = (x + cdx, y + cdy)
                end_point = (control[0] + adx, control[1] + ady)
                length = (
                    math.hypot(control[0] - x, control[1] - y)
                    + math.hypot(end_point[0] - control[0], end_point[1] - control[1])
                )
                steps = max(2, min(40, int(length / 120) + 1))
                edge_points = []
                for i in range(steps + 1):
                    t = i / steps
                    u = 1.0 - t
                    edge_points.append((
                        u * u * x + 2.0 * u * t * control[0] + t * t * end_point[0],
                        u * u * y + 2.0 * u * t * control[1] + t * t * end_point[1],
                    ))
                x, y = end_point

            # Ruffle's visit_point adds the same directed edge to all currently
            # active sides; orientation is handled only when a fill path flushes.
            tail = edge_points[1:]
            if fill0 > 0:
                active0.extend(tail)
            if fill1 > 0:
                active1.extend(tail)
            if lineidx > 0:
                active_line.extend(tail)
            continue

        new_styles = bits.u(1)
        state_line = bits.u(1)
        state_fill1 = bits.u(1)
        state_fill0 = bits.u(1)
        move_to = bits.u(1)
        if not (new_styles or state_line or state_fill1 or state_fill0 or move_to):
            emit_layer((x, y))
            break

        new_x = x
        new_y = y
        if move_to:
            count = bits.u(5)
            new_x, new_y = bits.s(count), bits.s(count)
        next_fill0 = bits.u(fill_bits) if state_fill0 else None
        next_fill1 = bits.u(fill_bits) if state_fill1 else None
        next_line = bits.u(line_bits) if state_line else None

        replacement_fills = replacement_lines = None
        if new_styles:
            bits.align()
            pos = bits.pos
            replacement_fills, pos = _read_fill_array(payload, pos, has_alpha, unsupported)
            replacement_lines, pos = _read_line_array(payload, pos, has_alpha, tag == 83, unsupported)
            bits.bit = pos * 8
            next_fill_bits = bits.u(4)
            next_line_bits = bits.u(4)

        # Ruffle applies MoveTo first, then treats NewStyles as a layer break,
        # then changes active style IDs against the replacement style arrays.
        if move_to:
            x, y = new_x, new_y
            flush_paths((x, y))

        if new_styles:
            emit_layer((x, y))
            fills = replacement_fills
            lines = replacement_lines
            pending_fills = [[] for _ in fills]
            pending_lines = [[] for _ in lines]
            fill_bits = next_fill_bits
            line_bits = next_line_bits

        if state_fill1:
            flush_fill(1, False, (x, y))
            fill1 = next_fill1
        if state_fill0:
            flush_fill(0, True, (x, y))
            fill0 = next_fill0
        if state_line:
            flush_stroke((x, y))
            lineidx = next_line

    return layers



def load_shapes(swf_path: Path, unsupported: set[str]):
    data = swf_path.read_bytes()
    if data[:3] != b"FWS":
        raise ValueError("expected the canonical uncompressed FWS from extract_projector.py")
    _, pos = read_rect(data, 8)
    pos += 4
    shapes = {}
    for code, payload, _, _ in tags(data, pos, len(data)):
        if code in SHAPE_TAGS:
            shape_id = struct.unpack_from("<H", payload, 0)[0]
            shapes[shape_id] = parse_shape(payload, code, unsupported)
    return shapes



def load_text_defs(swf_path: Path):
    global TEXT_DEFS, FONT_DEFS
    data = swf_path.read_bytes()
    _, pos = read_rect(data, 8)
    pos += 4
    font_payloads = {}
    text_payloads = {}
    for code, payload, _, _ in tags(data, pos, len(data)):
        if code == 48:  # DefineFont2
            font_payloads[struct.unpack_from("<H", payload, 0)[0]] = payload
        elif code == 11:  # DefineText
            text_payloads[struct.unpack_from("<H", payload, 0)[0]] = payload
    FONT_DEFS = {}
    for font_id, payload in font_payloads.items():
        parsed_id, font = parse_font2(payload)
        FONT_DEFS[parsed_id] = font
    TEXT_DEFS = {}
    for char_id, payload in text_payloads.items():
        parsed_id, bounds, matrix, records = parse_define_text(payload)
        TEXT_DEFS[parsed_id] = {"bounds": bounds, "matrix": matrix, "records": records}


def render_text(character_id, transform, canvas, missing):
    item = TEXT_DEFS[character_id]
    text_transform = compose(transform, affine(item["matrix"]))
    for record in item["records"]:
        font = FONT_DEFS.get(record["font"])
        if font is None:
            missing.add(record["font"] or -1)
            continue
        cursor_x = record["x"] / 20.0
        baseline_y = record["y"] / 20.0
        glyph_scale = record["height"] / (1024.0 * 20.0)
        for glyph_index, advance in record["glyphs"]:
            contours = []
            if glyph_index >= len(font["glyphs"]):
                missing.add(character_id)
                continue
            for contour in font["glyphs"][glyph_index]:
                transformed = []
                for gx, gy in contour:
                    px = cursor_x + gx * glyph_scale
                    py = baseline_y + gy * glyph_scale
                    transformed.append(point(text_transform, (px, py)))
                if transformed:
                    contours.append(transformed)
            if contours:
                mask = Image.new("1", canvas.size, 0)
                rasterize_winding(mask, contours, 0.0, 0.0, False)
                ink = Image.new("RGBA", canvas.size, record["color"])
                clear = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
                canvas.alpha_composite(Image.composite(ink, clear, mask))
            cursor_x += advance / 20.0


def text_bounds(character_id, transform):
    bounds = TEXT_DEFS[character_id]["bounds"]
    xmin, xmax, ymin, ymax = [v / 20.0 for v in bounds]
    pts = [
        point(transform, (xmin, ymin)), point(transform, (xmax, ymin)),
        point(transform, (xmin, ymax)), point(transform, (xmax, ymax)),
    ]
    return [
        min(p[0] for p in pts), min(p[1] for p in pts),
        max(p[0] for p in pts), max(p[1] for p in pts),
    ]


def display_list(symbol: dict, frame: int):
    frame = frame % max(1, symbol.get("frames", 1))
    depths = {}
    events = [(item.get("frame", 0), item.get("tag_offset", 0), "place", item)
              for item in symbol.get("placements", [])]
    events.extend((item.get("frame", 0), item.get("tag_offset", 0), "remove", item)
                  for item in symbol.get("removals", []))
    for event_frame, _, kind, item in sorted(events, key=lambda event: (event[0], event[1])):
        if event_frame > frame:
            continue
        depth = item["depth"]
        if kind == "remove":
            depths.pop(depth, None)
            continue
        previous = depths.get(depth)
        current = previous.copy() if previous else {}
        if "character_id" in item:
            # PlaceObject2 Move+HasCharacter replaces the character while
            # inheriting unspecified properties (matrix/cxform/name/etc.) from
            # the object already at that depth. A non-Move placement starts
            # from defaults instead.
            if not (item.get("flags", 0) & 0x01) or previous is None:
                current = {}
            current["character_id"] = item["character_id"]
            current["born"] = event_frame
        elif previous is None:
            # A Move without an existing object is malformed/no-op.
            continue
        for field in ("matrix", "ratio", "name", "clip_depth", "cxform"):
            if field in item:
                current[field] = item[field]
        if current:
            depths[depth] = current
    for depth, placement in depths.items():
        placement["depth"] = depth
    return depths


def _apply_cxform(image, cxform):
    """Apply a Flash placement CXFORMWITHALPHA to an RGBA child layer."""
    if not cxform:
        return image
    mult = cxform.get("mult", [256, 256, 256, 256])
    add = cxform.get("add", [0, 0, 0, 0])
    if mult == [256, 256, 256, 256] and add == [0, 0, 0, 0]:
        return image
    channels = image.split()
    transformed = []
    for channel, multiplier, offset in zip(channels, mult, add):
        lut = [max(0, min(255, (value * multiplier + 128) // 256 + offset))
               for value in range(256)]
        transformed.append(channel.point(lut))
    return Image.merge("RGBA", transformed)


def _inverse(matrix):
    a, b, c, d, e, f = matrix
    determinant = a * d - b * c
    if abs(determinant) < 1e-12:
        return None
    return (
        d / determinant,
        -b / determinant,
        -c / determinant,
        a / determinant,
        (b * f - d * e) / determinant,
        (c * e - a * f) / determinant,
    )


GRADIENT_RADIUS = 16384.0  # The gradient matrix consumes raw gradient-square twips.


def _spread_parameter(value: float, spread: int) -> float:
    if spread == 1:  # reflect
        value %= 2.0
        return 2.0 - value if value > 1.0 else value
    if spread == 2:  # repeat
        return value % 1.0
    return min(1.0, max(0.0, value))  # pad; reserved values fall back to pad


def _srgb_to_linear(value: float) -> float:
    value /= 255.0
    return value / 12.92 if value <= 0.04045 else ((value + 0.055) / 1.055) ** 2.4


def _linear_to_srgb(value: float) -> int:
    value = 12.92 * value if value <= 0.0031308 else 1.055 * (value ** (1.0 / 2.4)) - 0.055
    return max(0, min(255, round(value * 255.0)))


@lru_cache(maxsize=24)
def _gradient_lut(stops: tuple, interpolation: int, spread: int, radial: bool) -> bytes:
    size = 4096
    points = [(ratio / 255.0, color) for ratio, color in stops]
    rows = []
    for index in range(size):
        position = index / (size - 1)
        right = next((i for i, (ratio, _) in enumerate(points) if ratio >= position), len(points) - 1)
        if right == 0:
            color = points[0][1]
        else:
            left_ratio, left_color = points[right - 1]
            right_ratio, right_color = points[right]
            amount = 1.0 if right_ratio == left_ratio else (position - left_ratio) / (right_ratio - left_ratio)
            channels = []
            for channel, (a, b) in enumerate(zip(left_color, right_color)):
                if channel == 3:
                    value = a + (b - a) * amount
                elif interpolation == 1:
                    value = _linear_to_srgb(
                        _srgb_to_linear(a) + (_srgb_to_linear(b) - _srgb_to_linear(a)) * amount
                    )
                else:
                    value = round(a + (b - a) * amount)
                channels.append(max(0, min(255, round(value))))
            color = tuple(channels)
        rows.append(bytes(color))

    if not radial:
        # The source strip spans gradient coordinates [-3R, 3R], which maps
        # to normalized linear positions [-1, 2].
        linear = bytearray()
        for index in range(size):
            t = index / (size - 1) * 3.0 - 1.0
            mapped = _spread_parameter(t, spread)
            lut_index = min(size - 1, max(0, round(mapped * (size - 1))))
            linear.extend(rows[lut_index])
        return bytes(linear)

    radial_size = 512
    radial = bytearray(radial_size * radial_size * 4)
    coordinates = [((index + 0.5) / radial_size * 2.0 - 1.0) ** 2 for index in range(radial_size)]
    for y, y_squared in enumerate(coordinates):
        row = bytearray()
        for x_squared in coordinates:
            distance = math.sqrt(x_squared + y_squared)
            mapped = _spread_parameter(distance, spread)
            row.extend(rows[min(size - 1, round(mapped * (size - 1)))])
        start = y * radial_size * 4
        radial[start : start + radial_size * 4] = row
    return bytes(radial)


def _shape_mask(contours_px, left, top, right, bottom, even_odd):
    mask = Image.new("1", (right - left + 1, bottom - top + 1), 0)
    rasterize_winding(mask, contours_px, left, top, even_odd)
    return mask.convert("L")


def _render_gradient(fill, transform, contours_px, canvas, left, top, right, bottom, even_odd=False):
    kind = fill["gradient"]
    gradient_to_canvas = compose(transform, affine(fill["matrix"]))
    region_transform = compose((1.0, 0.0, 0.0, 1.0, -left, -top), gradient_to_canvas)
    inverse = _inverse(region_transform)
    if inverse is None:
        return

    region_size = (right - left + 1, bottom - top + 1)
    if kind == 0x10:
        linear_bytes = _gradient_lut(fill["stops"], fill["interpolation"], fill["spread"], False)
        strip = Image.frombytes("RGBA", (4096, 1), linear_bytes)
        source_start = -3.0 * GRADIENT_RADIUS
        source_span = 6.0 * GRADIENT_RADIUS
        scale = 4095.0 / source_span
        pillow_inverse = (
            inverse[0] * scale,
            inverse[1] * scale,
            (inverse[4] - source_start) * scale,
            0.0,
            0.0,
            0.0,
        )
        sampled = strip.transform(
            region_size,
            Image.Transform.AFFINE,
            pillow_inverse,
            resample=Image.Resampling.BICUBIC,
            fillcolor=tuple(fill["stops"][-1][1]),
        )
    else:
        radial_bytes = _gradient_lut(fill["stops"], fill["interpolation"], fill["spread"], True)
        side = 512
        texture = Image.frombytes("RGBA", (side, side), radial_bytes)
        source_scale = (side - 1) / (2.0 * GRADIENT_RADIUS)
        pillow_inverse = (
            inverse[0] * source_scale,
            inverse[1] * source_scale,
            (inverse[4] + GRADIENT_RADIUS) * source_scale,
            inverse[2] * source_scale,
            inverse[3] * source_scale,
            (inverse[5] + GRADIENT_RADIUS) * source_scale,
        )
        sampled = texture.transform(
            region_size,
            Image.Transform.AFFINE,
            pillow_inverse,
            resample=Image.Resampling.BICUBIC,
            fillcolor=tuple(fill["stops"][-1][1]),
        )

    mask = _shape_mask(contours_px, left, top, right, bottom, even_odd)
    sampled.putalpha(Image.composite(sampled.getchannel("A"), Image.new("L", region_size, 0), mask))
    canvas.alpha_composite(sampled, (left, top))


def render_symbol(symbol_id, frame, transform, canvas, symbols, shapes, bitmaps, unsupported, missing, depth=0, excluded_symbols=None):
    if depth > 32:
        raise RuntimeError(f"symbol recursion exceeded at character {symbol_id}")
    if symbol_id in shapes:
        draw = ImageDraw.Draw(canvas)
        for layer in shapes[symbol_id]:
            fills = layer["fills"]
            paths = layer["paths"]
            lines = layer["lines"]
            line_paths = layer["line_paths"]
            even_odd = layer["even_odd"]

            for fill_index, contours in paths.items():
                if fill_index >= len(fills):
                    continue
                fill = fills[fill_index]
                if fill is None:
                    continue
                contours_px = [
                    [
                        (round(px), round(py))
                        for px, py in (
                            point(transform, (x / 20.0, y / 20.0))
                            for x, y in contour
                        )
                    ]
                    for contour in contours
                ]
                flat_points = [p for contour in contours_px for p in contour]
                if not flat_points:
                    continue
                left = max(0, min(p[0] for p in flat_points))
                top = max(0, min(p[1] for p in flat_points))
                right = min(canvas.width - 1, max(p[0] for p in flat_points))
                bottom = min(canvas.height - 1, max(p[1] for p in flat_points))
                if left > right or top > bottom:
                    continue
                region_size = (right - left + 1, bottom - top + 1)
                mask = _shape_mask(contours_px, left, top, right, bottom, even_odd)
                if isinstance(fill, dict) and "gradient" in fill:
                    _render_gradient(
                        fill, transform, contours_px, canvas,
                        left, top, right, bottom, even_odd
                    )
                elif isinstance(fill, dict):
                    bitmap = bitmaps.get(fill["bitmap_id"])
                    if bitmap is None:
                        missing.add(fill["bitmap_id"])
                        continue
                    bitmap_to_canvas = compose(transform, affine(fill["matrix"]))
                    region_transform = compose(
                        (1.0, 0.0, 0.0, 1.0, -left, -top),
                        bitmap_to_canvas,
                    )
                    inverse = _inverse(region_transform)
                    if inverse is None:
                        continue
                    pillow_inverse = (
                        inverse[0], inverse[1], inverse[4],
                        inverse[2], inverse[3], inverse[5],
                    )
                    sampled = bitmap.transform(
                        region_size,
                        Image.Transform.AFFINE,
                        pillow_inverse,
                        resample=(
                            Image.Resampling.BILINEAR
                            if fill["smooth"] else Image.Resampling.NEAREST
                        ),
                        fillcolor=(0, 0, 0, 0) if not fill["repeat"] else None,
                    )
                    sampled.putalpha(ImageChops.multiply(sampled.getchannel("A"), mask))
                    canvas.alpha_composite(sampled, (left, top))
                else:
                    color_layer = Image.new("RGBA", region_size, fill)
                    color_layer.putalpha(
                        ImageChops.multiply(color_layer.getchannel("A"), mask)
                    )
                    canvas.alpha_composite(color_layer, (left, top))

            # Flash draws strokes after all fills in each drawing layer.
            matrix_scale = math.sqrt(
                abs(transform[0] * transform[3] - transform[1] * transform[2])
            )
            for line_index, segments in line_paths.items():
                if line_index >= len(lines):
                    continue
                style = lines[line_index]
                width = max(1, round(style["width"] * matrix_scale))
                for segment in segments:
                    points = [
                        point(transform, (x / 20.0, y / 20.0))
                        for x, y in segment
                    ]
                    if len(points) >= 2:
                        draw.line(
                            [(round(x), round(y)) for x, y in points],
                            fill=style["color"],
                            width=width,
                        )
        return

    if symbol_id in TEXT_DEFS:
        render_text(symbol_id, transform, canvas, missing)
        return

    symbol = symbols.get(str(symbol_id))
    if symbol is None:
        missing.add(symbol_id)
        return
    local_frame = frame % max(1, symbol.get("frames", 1))
    active_masks = []
    for placement in sorted(
        display_list(symbol, local_frame).values(),
        key=lambda item: item.get("depth", 0),
    ):
        child_id = placement.get("character_id")
        if child_id is None:
            continue
        if excluded_symbols and child_id in excluded_symbols:
            continue
        placement_depth = placement.get("depth", 0)
        active_masks = [
            (end_depth, mask)
            for end_depth, mask in active_masks
            if placement_depth <= end_depth
        ]
        child_transform = compose(transform, affine(placement.get("matrix", {})))
        child_frame = max(0, local_frame - placement.get("born", 0))

        def render_child(target):
            render_symbol(
                child_id,
                child_frame,
                child_transform,
                target,
                symbols,
                shapes,
                bitmaps,
                unsupported,
                missing,
                depth + 1,
                excluded_symbols,
            )

        clip_depth = placement.get("clip_depth")
        cxform = placement.get("cxform")
        if clip_depth is not None:
            mask_layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
            render_child(mask_layer)
            mask_layer = _apply_cxform(mask_layer, cxform)
            mask = mask_layer.getchannel("A")
            for _, parent_mask in active_masks:
                mask = ImageChops.multiply(mask, parent_mask)
            active_masks.append((clip_depth, mask))
        elif active_masks or cxform:
            layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
            render_child(layer)
            layer = _apply_cxform(layer, cxform)
            alpha = layer.getchannel("A")
            for _, mask in active_masks:
                alpha = ImageChops.multiply(alpha, mask)
            layer.putalpha(alpha)
            canvas.alpha_composite(layer)
        else:
            render_child(canvas)


def symbol_bounds(symbol_id, frame, transform, symbols, shapes, depth=0, excluded_symbols=None):
    if depth > 32:
        raise RuntimeError(f"symbol recursion exceeded at character {symbol_id}")
    bounds = [math.inf, math.inf, -math.inf, -math.inf]
    if symbol_id in shapes:
        for layer in shapes[symbol_id]:
            fills = layer["fills"]
            paths = layer["paths"]
            line_paths = layer["line_paths"]
            for fill_index, contours in paths.items():
                if fill_index >= len(fills) or fills[fill_index] is None:
                    continue
                for contour in contours:
                    for x, y in contour:
                        px, py = point(transform, (x / 20.0, y / 20.0))
                        bounds[0] = min(bounds[0], px)
                        bounds[1] = min(bounds[1], py)
                        bounds[2] = max(bounds[2], px)
                        bounds[3] = max(bounds[3], py)
            line_points = [
                point(transform, (x / 20.0, y / 20.0))
                for segments in line_paths.values()
                for segment in segments
                for x, y in segment
            ]
            if line_points:
                bounds[0] = min(bounds[0], min(p[0] for p in line_points))
                bounds[1] = min(bounds[1], min(p[1] for p in line_points))
                bounds[2] = max(bounds[2], max(p[0] for p in line_points))
                bounds[3] = max(bounds[3], max(p[1] for p in line_points))
        return bounds

    if symbol_id in TEXT_DEFS:
        return text_bounds(symbol_id, transform)

    symbol = symbols.get(str(symbol_id))
    if symbol is None:
        return bounds
    local_frame = frame % max(1, symbol.get("frames", 1))
    for placement in display_list(symbol, local_frame).values():
        child_id = placement.get("character_id")
        if child_id is None:
            continue
        if excluded_symbols and child_id in excluded_symbols:
            continue
        child_transform = compose(transform, affine(placement.get("matrix", {})))
        child_frame = max(0, local_frame - placement.get("born", 0))
        child_bounds = symbol_bounds(child_id, child_frame, child_transform, symbols, shapes, depth + 1, excluded_symbols)
        bounds[0] = min(bounds[0], child_bounds[0])
        bounds[1] = min(bounds[1], child_bounds[1])
        bounds[2] = max(bounds[2], child_bounds[2])
        bounds[3] = max(bounds[3], child_bounds[3])
    return bounds


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("swf", type=Path)
    parser.add_argument("symbols_json", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--symbol", type=int, default=691)
    parser.add_argument("--frame", type=int, default=0)
    parser.add_argument("--width", type=int, default=1200)
    parser.add_argument("--height", type=int, default=1200)
    parser.add_argument("--scale", type=float, default=0.163742, help="player placement scale recovered from level1a")
    parser.add_argument("--background", default="transparent", help="transparent or a hex RGB color such as #ffffff")
    parser.add_argument("--bitmaps", type=Path, help="directory produced by extract_bitmaps.py (defaults beside the SWF source directory)")
    parser.add_argument("--all-frames", action="store_true", help="bake the whole symbol timeline into a directory plus manifest")
    parser.add_argument("--native-bounds", action="store_true", help="size the single-frame PNG to the symbol's native transformed bounds")
    parser.add_argument("--clip-stage-width", type=float, help="for --all-frames, crop rendering to this logical stage width")
    parser.add_argument("--clip-stage-height", type=float, help="for --all-frames, crop rendering to this logical stage height")
    parser.add_argument("--place-x", type=float, default=0.0, help="logical stage X of the symbol origin when stage clipping")
    parser.add_argument("--place-y", type=float, default=0.0, help="logical stage Y of the symbol origin when stage clipping")
    args = parser.parse_args()

    symbols = json.loads(args.symbols_json.read_text(encoding="utf-8"))
    unsupported: set[str] = set()
    shapes = load_shapes(args.swf, unsupported)
    load_text_defs(args.swf)
    bitmap_dir = args.bitmaps or args.swf.resolve().parent.parent / "bitmaps"
    bitmap_manifest = json.loads((bitmap_dir / "manifest.json").read_text(encoding="utf-8"))
    bitmaps = {}
    for item in bitmap_manifest:
        if "file" not in item:
            continue
        with Image.open(bitmap_dir / item["file"]) as source:
            bitmaps[item["id"]] = source.convert("RGBA")
    if args.background == "transparent":
        background = (0, 0, 0, 0)
    else:
        color = args.background.removeprefix("#")
        if len(color) != 6:
            raise ValueError("--background must be 'transparent' or a six-digit RGB hex value")
        background = tuple(int(color[i : i + 2], 16) for i in (0, 2, 4)) + (255,)

    if args.all_frames:
        symbol = symbols.get(str(args.symbol))
        if symbol is None:
            raise ValueError(f"symbol {args.symbol} is not a movie clip in symbols.json")
        frame_count = symbol.get("frames", 0)
        frame_bounds = [
            symbol_bounds(args.symbol, frame, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes)
            for frame in range(frame_count)
        ]
        visible_bounds = [box for box in frame_bounds if all(math.isfinite(value) for value in box)]
        if not visible_bounds:
            raise ValueError(f"symbol {args.symbol} contains no supported solid vector fills")
        min_x = min(box[0] for box in visible_bounds)
        min_y = min(box[1] for box in visible_bounds)
        max_x = max(box[2] for box in visible_bounds)
        max_y = max(box[3] for box in visible_bounds)
        pad = 2
        clip_requested = args.clip_stage_width is not None or args.clip_stage_height is not None
        if clip_requested:
            if args.clip_stage_width is None or args.clip_stage_height is None:
                raise ValueError("--clip-stage-width and --clip-stage-height must be supplied together")
            if args.clip_stage_width <= 0 or args.clip_stage_height <= 0:
                raise ValueError("stage clip dimensions must be positive")

            # Work in integer logical-stage pixels so a supersampled temporary
            # bake (6x) can be downsampled exactly to the final 3x grid.
            # Only geometry that can actually intersect the visible stage is
            # rasterized; huge off-screen fade rectangles never allocate a
            # giant native-bounds canvas.
            placed_min_x = min_x + args.place_x
            placed_min_y = min_y + args.place_y
            placed_max_x = max_x + args.place_x
            placed_max_y = max_y + args.place_y
            crop_left = max(0.0, math.floor(placed_min_x) - 1.0)
            crop_top = max(0.0, math.floor(placed_min_y) - 1.0)
            crop_right = min(args.clip_stage_width, math.ceil(placed_max_x) + 1.0)
            crop_bottom = min(args.clip_stage_height, math.ceil(placed_max_y) + 1.0)
            if crop_right <= crop_left or crop_bottom <= crop_top:
                raise ValueError(
                    f"symbol {args.symbol} does not intersect the requested stage clip"
                )
            width = max(1, round((crop_right - crop_left) * args.scale))
            height = max(1, round((crop_bottom - crop_top) * args.scale))
            tx = (args.place_x - crop_left) * args.scale
            ty = (args.place_y - crop_top) * args.scale
            stage_clip = {
                "logical_rect": [crop_left, crop_top, crop_right, crop_bottom],
                "stage_size": [args.clip_stage_width, args.clip_stage_height],
                "placement": [args.place_x, args.place_y],
            }
        else:
            tx = pad - min_x * args.scale
            ty = pad - min_y * args.scale
            width = math.ceil((max_x - min_x) * args.scale) + pad * 2
            height = math.ceil((max_y - min_y) * args.scale) + pad * 2
            stage_clip = None
        transform = (args.scale, 0.0, 0.0, args.scale, tx, ty)
        outdir = args.output
        outdir.mkdir(parents=True, exist_ok=True)
        labels = sorted(symbol.get("labels", []), key=lambda item: item.get("frame", 0))
        manifest_frames = []
        all_missing: set[int] = set()
        for frame in range(frame_count):
            canvas = Image.new("RGBA", (width, height), background)
            missing: set[int] = set()
            render_symbol(args.symbol, frame, transform, canvas, symbols, shapes, bitmaps, unsupported, missing)
            all_missing.update(missing)
            filename = f"{frame:03}.png"
            canvas.save(outdir / filename)
            active_label = None
            for label in labels:
                if label.get("frame", 0) <= frame:
                    active_label = label.get("label")
                else:
                    break
            manifest_frames.append({"frame": frame, "animation": active_label, "file": filename})
        manifest = {
            "symbol": args.symbol,
            "frame_count": frame_count,
            "size": [width, height],
            "scale": [args.scale, args.scale],
            "source_bounds": [min_x, min_y, max_x, max_y],
            "anchor_in_bitmap": [tx, ty],
            "stage_clip": stage_clip,
            "unsupported_fills": sorted(unsupported),
            "missing_bitmap_ids": sorted(all_missing),
            "frames": manifest_frames,
        }
        (outdir / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        print(json.dumps({key: manifest[key] for key in ("symbol", "frame_count", "size", "scale", "anchor_in_bitmap", "unsupported_fills")}, indent=2))
        return

    missing: set[int] = set()
    source_bounds = symbol_bounds(args.symbol, args.frame, (1.0, 0.0, 0.0, 1.0, 0.0, 0.0), symbols, shapes)
    if not all(math.isfinite(value) for value in source_bounds):
        raise ValueError(f"symbol {args.symbol} frame {args.frame} has no supported solid vector fills")
    if args.native_bounds:
        pad = 1
        out_width = math.ceil((source_bounds[2] - source_bounds[0]) * args.scale) + 2 * pad
        out_height = math.ceil((source_bounds[3] - source_bounds[1]) * args.scale) + 2 * pad
        tx = pad - source_bounds[0] * args.scale
        ty = pad - source_bounds[1] * args.scale
    else:
        out_width, out_height = args.width, args.height
        tx = (out_width - (source_bounds[0] + source_bounds[2]) * args.scale) / 2.0
        ty = (out_height - (source_bounds[1] + source_bounds[3]) * args.scale) / 2.0
    canvas = Image.new("RGBA", (out_width, out_height), background)
    transform = (args.scale, 0.0, 0.0, args.scale, tx, ty)
    render_symbol(args.symbol, args.frame, transform, canvas, symbols, shapes, bitmaps, unsupported, missing)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(args.output)
    origin = [-tx / args.scale, -ty / args.scale]
    print(json.dumps({"symbol": args.symbol, "frame": args.frame, "size": list(canvas.size), "scale": args.scale, "source_bounds": source_bounds, "source_bounds_pixels": [value * args.scale for value in source_bounds], "translation": [tx, ty], "image_origin_in_symbol_space": origin, "shape_defs": len(shapes), "unsupported_fills": sorted(unsupported), "missing_bitmap_ids": sorted(missing), "missing_symbol_ids": sorted(missing)}, indent=2))


if __name__ == "__main__":
    main()
