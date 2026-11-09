#!/usr/bin/env python3
"""A picture of what creatures.rs hands the game to draw (the quads the harness's export_drawn
test writes), rasterised with the mod's textures: the check that the Rust model reader draws
what GeckoLib draws.  python3 tools/render_drawn.py <unzipped jar> /tmp/sift_drawn.json <out.png>"""
import json, os, sys
import numpy as np
from PIL import Image, ImageDraw
JAR, DRAWN, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
sys.argv = [sys.argv[0], JAR, "/tmp/_unused"]
sys.path.insert(0, os.path.dirname(__file__))
import render_assets as ra  # noqa: E402

data = json.load(open(DRAWN))
VIEW = ra.rot("x", 18) @ ra.rot("y", -30)
tiles = []
for kind, quads in data.items():
    qs = []
    for sprite, corners, uv, light in quads:
        tex = ra.texture(sprite)
        th, tw = tex.shape[:2]
        u0, v0, u1, v1 = [x / 16 for x in uv]
        # The game's order (top left, bottom left, bottom right, top right) and its UVs.
        uvs = [(u0 * tw, v0 * th), (u0 * tw, v1 * th), (u1 * tw, v1 * th), (u1 * tw, v0 * th)]
        if "emissive" in sprite or "glowmask" in sprite:
            continue
        qs.append((np.array(corners) * 16, uvs, tex))
    tiles.append((kind, ra.draw_quads(qs, VIEW, 320)))
img = Image.new("RGBA", (330 * len(tiles), 360), ra.BG)
d = ImageDraw.Draw(img)
for i, (k, t) in enumerate(tiles):
    img.alpha_composite(t, (5 + i * 330, 5))
    d.text((5 + i * 330 + 10, 330), k + " (as the game draws it)", font=ra.font(14), fill=(220, 210, 240, 255))
img.save(OUT)
