#!/usr/bin/env python3
"""AUDIT-FIX-04: deterministic Pillow source-over strokes and legacy isolation."""
from PIL import Image, ImageDraw
from bake_sprite_frames import _draw_strokes_source_over, render_symbol


def check_pixel_math():
    for under in ((12, 40, 80, 255), (30, 50, 70, 96), (0, 0, 0, 0)):
        actual = Image.new("RGBA", (9, 9), under)
        paths = [[(1, 4), (7, 4)], [(4, 4), (4, 7)]]
        _draw_strokes_source_over(actual, paths, (255, 20, 0, 128), 1)
        ink = Image.new("RGBA", (9, 9), (0, 0, 0, 0))
        brush = ImageDraw.Draw(ink)
        for path in paths:
            brush.line(path, fill=(255, 20, 0, 128), width=1)
        expected = Image.alpha_composite(Image.new("RGBA", (9, 9), under), ink)
        assert actual.tobytes() == expected.tobytes(), "Porter-Duff stroke compositing changed"
        assert actual.getpixel((0, 0)) == under
        # Segment joint belongs to one line style and must not double its alpha.
        assert actual.getpixel((4, 4)) == expected.getpixel((4, 4))
        if under[3] == 255:
            assert actual.getpixel((4, 4))[3] == 255
    # Offscreen paths must clip, not allocate unbounded geometry or paint OOB.
    canvas = Image.new("RGBA", (3, 3), (0, 0, 0, 0))
    _draw_strokes_source_over(canvas, [[(-1000, 1), (1000, 1)]], (1, 2, 3, 128), 1)
    assert canvas.getpixel((1, 1)) == (1, 2, 3, 128)


def check_recursive_opt_in():
    shape = [{
        "fills": [], "paths": {}, "lines": [{"width": 1, "color": (255, 0, 0, 128)}],
        "line_paths": {0: [[(20, 60), (140, 60)]]}, "even_odd": False,
    }]
    # Render a nested source shape, using real display-list PlaceObject fields.
    symbols = {"42": {"frames": 1, "tags": []}}
    # Static shape itself exercises the identical renderer branch, while explicit
    # default-vs-opt-in contrast protects the old 1x hash behavior.
    legacy = Image.new("RGBA", (9, 9), (0, 0, 255, 255))
    corrected = legacy.copy()
    args = (7, 0, (1, 0, 0, 1, 0, 0))
    render_symbol(*args, legacy, symbols, {7: shape}, {}, set(), set())
    render_symbol(*args, corrected, symbols, {7: shape}, {}, set(), set(),
                  stroke_source_over=True)
    assert legacy.getpixel((4, 3)) == (255, 0, 0, 128)
    assert corrected.getpixel((4, 3))[3] == 255
    assert corrected.getpixel((4, 3)) != legacy.getpixel((4, 3))


if __name__ == "__main__":
    check_pixel_math()
    check_recursive_opt_in()
    print("AUDIT_FIX_04_STROKE_SOURCE_OVER_MATH_AND_LEGACY_PASS")
