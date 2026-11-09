#!/usr/bin/env python3
"""A picture of the mock Sift the harness decorates (crates/world-mock, test export_preview):
every visible block as its own model, isometric, with The Sift's textures; water drawn as ichor,
as the native mod draws it there.

    SIFT_OUT=/tmp cargo test --release -p horizon-world export_preview -- --ignored
    python3 tools/render_terrain.py <unzipped jar> /tmp/sift_terrain <out.png> [tile px] [crop x z size]
"""
import json, os, sys
import numpy as np
from PIL import Image

JAR, BASE, OUT = sys.argv[1], sys.argv[2], sys.argv[3]
TILE = int(sys.argv[4]) if len(sys.argv) > 4 else 12
CROP = [int(v) for v in sys.argv[5:8]] if len(sys.argv) > 7 else None
sys.argv = [sys.argv[0], JAR, "/tmp/_unused"]
sys.path.insert(0, os.path.dirname(__file__))
import render_assets as ra  # noqa: E402

meta = json.load(open(BASE + ".json"))
SX, SY, SZ = meta["size"]
names = meta["names"]
grid = np.frombuffer(open(BASE + ".bin", "rb").read(), np.uint16).reshape(SY, SZ, SX)
if CROP:
    cx, cz, cs = CROP
    grid = grid[:, cz:cz + cs, cx:cx + cs]
    SZ = SX = cs

def solid_color(rgb, a=1.0):
    t = np.zeros((16, 16, 4), np.float32)
    t[..., :3] = np.array(rgb) / 255
    # A little texture so flat colours read as blocks.
    rnd = np.random.default_rng(sum(rgb)).random((16, 16)) * 0.12 - 0.06
    t[..., :3] = np.clip(t[..., :3] + rnd[..., None], 0, 1)
    t[..., 3] = a
    return t

# Vanilla blocks (their textures aren't in the mod): drawn in their colours.
VANILLA = {
    "minecraft:bedrock": (85, 85, 85), "minecraft:lava": (230, 110, 20), "minecraft:sculk": (13, 30, 36),
    "minecraft:sculk_catalyst": (40, 60, 66), "minecraft:sculk_sensor": (20, 80, 100),
    "minecraft:sculk_shrieker": (200, 200, 180), "minecraft:chest": (160, 110, 50), "minecraft:stone": (125, 125, 125),
}
VANILLA_PLANT = {"minecraft:torchflower": (240, 150, 40), "minecraft:pitcher_plant": (60, 150, 160),
                 "minecraft:sniffer_egg": (170, 70, 60), "minecraft:sculk_vein": (20, 70, 80),
                 "minecraft:wildflowers": (240, 220, 90)}

def cube(tex_top, tex_side=None, h=16):
    tex_side = tex_side if tex_side is not None else tex_top
    q = []
    a, b = (0, 0, 0), (16, h, 16)
    for face in ("up", "north", "south", "west", "east"):
        t = tex_top if face == "up" else tex_side
        pts = np.array(ra.FACE[face](a, b), np.float64) - 8
        th, tw = t.shape[:2]
        q.append((pts, [(0, 0), (tw, 0), (tw, th), (0, th)], t))
    return q

def cross(tex):
    q = []
    th, tw = tex.shape[:2]
    for ang in (45, -45):
        R = ra.rot("y", ang)
        pts = np.array([(-8, 8, 0), (8, 8, 0), (8, -8, 0), (-8, -8, 0)], np.float64) @ R.T
        q.append((pts, [(0, 0), (tw, 0), (tw, th), (0, th)], tex))
    return q

def state_quads(state):
    name, _, props = state.partition("[")
    props = dict(kv.split("=") for kv in props.rstrip("]").split(",") if "=" in kv)
    if name in VANILLA:
        return cube(solid_color(VANILLA[name]))
    if name in VANILLA_PLANT:
        return cross(solid_color(VANILLA_PLANT[name]))
    if name == "minecraft:water":
        t = ra.texture("the_sift:block/ichor_still")
        return [(np.array(ra.FACE["up"]((0, 0, 0), (16, 14, 16)), np.float64) - 8,
                 [(0, 0), (t.shape[1], 0), (t.shape[1], t.shape[0]), (0, t.shape[0])], t)]
    ns, path = name.split(":")
    bs = os.path.join(ra.A, "blockstates", path + ".json")
    if not os.path.exists(bs):
        return cube(solid_color((255, 0, 255)))
    j = json.load(open(bs))
    if "variants" in j:
        v = None
        for key, var in j["variants"].items():
            want = dict(kv.split("=") for kv in key.split(",") if "=" in kv)
            if all(props.get(k, v_) == v_ for k, v_ in want.items()):
                v = var
                break
        v = v or next(iter(j["variants"].values()))
        if isinstance(v, list):
            v = v[0]
        quads, _ = ra.model_quads(v["model"])
        R = ra.rot("y", -v.get("y", 0)) @ ra.rot("x", -v.get("x", 0))
        return [(p @ R.T, uv, t) for p, uv, t in quads]
    # Multipart (vines): the hanging sheet as a cross of its texture.
    tex = None
    for part in j.get("multipart", []):
        m = part["apply"]
        m = m[0] if isinstance(m, list) else m
        _, tref, _ = ra.resolve_model(m["model"])
        for k in ("texture", "vine", "side", "all", "particle"):
            t = tref("#" + k)
            if t:
                tex = ra.texture(t)
                break
        if tex is not None:
            break
    return cross(tex if tex is not None else solid_color((60, 160, 120)))

VIEW = ra.GUI_VIEW
def sprite(state):
    quads = state_quads(state)
    if not quads:
        return None
    # The block's 16-unit cube fits the tile (same framing for every block).
    span = 16 * 1.75
    size = TILE * 2
    img = ra.draw_quads(quads, VIEW, size, fit=(span, np.zeros(3)), ss=2)
    return img

print("sprites...", flush=True)
sprites = {}
for i, n in enumerate(names):
    if n in ("minecraft:air", "minecraft:cave_air", "minecraft:void_air"):
        sprites[i] = None
        continue
    try:
        sprites[i] = sprite(n)
    except Exception as e:  # a model this renderer can't read
        print("  sprite", n, e)
        sprites[i] = ra.draw_quads(cube(solid_color((255, 0, 255))), VIEW, TILE * 2, fit=(28, np.zeros(3)))

air = np.array([n in ("minecraft:air", "minecraft:cave_air", "minecraft:void_air") for n in names])
full = np.array([not air[i] and not any(k in n for k in ("water", "lava", "vines", "foliage", "plant", "chard", "stalks",
    "fronds", "lotus", "whisperbloom", "sunburst", "sprouts", "torchflower", "pitcher", "egg", "vein", "snow", "roots"))
    for i, n in enumerate(names)])
opaque = full[grid]
# A block shows if a neighbour isn't opaque (and it isn't air).
exposed = ~air[grid]
nb = np.zeros_like(exposed)
pad = np.pad(opaque, 1, constant_values=False)
for dy, dz, dx in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)):
    nb |= ~pad[1 + dy:1 + dy + SY, 1 + dz:1 + dz + SZ, 1 + dx:1 + dx + SX]
exposed &= nb
# Cut the block off a little under the lowest ground (the cave layers and the floor stay hidden).
top = SY - 1 - np.argmax((~air[grid])[::-1], axis=0)
cut = int(np.percentile(top, 1)) - 6
exposed[:cut] = False
# The cut's own faces: the solid blocks along it show as the section.
exposed[cut] |= ~air[grid[cut]]
ys, zs, xs = np.nonzero(exposed)
print(len(ys), "visible blocks", flush=True)

# Screen positions: the view of each block's centre (16 units a block).
P = np.stack([xs - SX / 2, ys - 128, zs - SZ / 2], 1).astype(np.float64) * 16 @ VIEW.T
scale = TILE * 2 / (16 * 1.75)
sx, sy, depth = P[:, 0] * scale, -P[:, 1] * scale, P[:, 2]
W = int(sx.max() - sx.min()) + TILE * 4
H = int(sy.max() - sy.min()) + TILE * 4
ox, oy = -sx.min() + TILE * 2, -sy.min() + TILE * 2
order = np.argsort(depth)
img = Image.new("RGBA", (W, H), (22, 18, 36, 255))
for k in order:
    s = sprites[grid[ys[k], zs[k], xs[k]]]
    if s is None:
        continue
    img.alpha_composite(s, (int(sx[k] + ox - TILE), int(sy[k] + oy - TILE)))
bbox = img.getbbox()
img.save(OUT)
print("wrote", OUT, img.size)
