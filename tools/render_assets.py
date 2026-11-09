#!/usr/bin/env python3
"""Pictures of The Sift's content, from the mod's own assets: every item as the inventory shows
it (block models in 3D), the spawn eggs, and the mobs from their GeckoLib models and textures.

    python3 tools/render_assets.py <unzipped jar> <out dir>

A small software rasteriser (numpy): textured quads, a depth buffer, cutout alpha and
Minecraft's face shading. Block models resolve their parents (the vanilla parents The Sift uses
are written out below, as vanilla defines them)."""
import json, math, os, sys
import numpy as np
from PIL import Image, ImageDraw, ImageFont

JAR, OUT = sys.argv[1], sys.argv[2]
A = os.path.join(JAR, "assets", "the_sift")
os.makedirs(OUT, exist_ok=True)

# ---------------------------------------------------------------- textures

_tex = {}
def texture(ref):
    """`the_sift:block/x` → RGBA float array (first frame of an animated strip)."""
    if ref in _tex:
        return _tex[ref]
    ns, path = ref.split(":") if ":" in ref else ("minecraft", ref)
    f = os.path.join(JAR, "assets", ns, "textures", path + ".png")
    if not os.path.exists(f):
        img = np.zeros((16, 16, 4), np.float32)
        img[..., 0] = img[..., 2] = 1; img[..., 3] = 1
        img[::2, ::2, :3] = 0; img[1::2, 1::2, :3] = 0
    else:
        im = Image.open(f).convert("RGBA")
        w, h = im.size
        if h > w and h % w == 0:
            im = im.crop((0, 0, w, w))
        img = np.asarray(im, np.float32) / 255
    _tex[ref] = img
    return img

# ---------------------------------------------------------------- the rasteriser

class Canvas:
    def __init__(s, size, ss=3):
        s.n = size * ss
        s.ss = ss
        s.rgb = np.zeros((s.n, s.n, 3), np.float32)
        s.a = np.zeros((s.n, s.n), np.float32)
        s.z = np.full((s.n, s.n), -1e9, np.float32)

    def quad(s, pts, uvs, tex, shade, cutoff=0.1):
        """pts: 4 screen points (x, y in 0..1, z towards the viewer); uvs: texture pixels."""
        for tri in ((0, 1, 2), (0, 2, 3)):
            s._tri([pts[i] for i in tri], [uvs[i] for i in tri], tex, shade, cutoff)

    def _tri(s, p, uv, tex, shade, cutoff):
        P = np.array(p, np.float64) * [s.n, s.n, 1]
        x0, y0 = np.floor(P[:, :2].min(0)).astype(int)
        x1, y1 = np.ceil(P[:, :2].max(0)).astype(int)
        x0, y0 = max(x0, 0), max(y0, 0)
        x1, y1 = min(x1, s.n - 1), min(y1, s.n - 1)
        if x1 < x0 or y1 < y0:
            return
        (ax, ay, az), (bx, by, bz), (cx, cy, cz) = P
        d = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy)
        if abs(d) < 1e-9:
            return
        ys, xs = np.mgrid[y0:y1 + 1, x0:x1 + 1] + 0.5
        w0 = ((by - cy) * (xs - cx) + (cx - bx) * (ys - cy)) / d
        w1 = ((cy - ay) * (xs - cx) + (ax - cx) * (ys - cy)) / d
        w2 = 1 - w0 - w1
        inside = (w0 >= -1e-6) & (w1 >= -1e-6) & (w2 >= -1e-6)
        if not inside.any():
            return
        z = w0 * az + w1 * bz + w2 * cz
        U = w0 * uv[0][0] + w1 * uv[1][0] + w2 * uv[2][0]
        V = w0 * uv[0][1] + w1 * uv[1][1] + w2 * uv[2][1]
        th, tw = tex.shape[:2]
        ti = np.clip(np.floor(U).astype(int), 0, tw - 1)
        tj = np.clip(np.floor(V).astype(int), 0, th - 1)
        c = tex[tj, ti]
        zb = s.z[y0:y1 + 1, x0:x1 + 1]
        m = inside & (c[..., 3] > cutoff) & (z > zb)
        if not m.any():
            return
        rgb = s.rgb[y0:y1 + 1, x0:x1 + 1]
        al = s.a[y0:y1 + 1, x0:x1 + 1]
        alpha = c[..., 3]
        col = c[..., :3] * shade
        solid = m & (alpha >= 0.99)
        glass = m & (alpha < 0.99)
        rgb[solid] = col[solid]
        al[solid] = 1
        zb[solid] = z[solid]
        # Translucent texels blend over what's there (drawn after the solid ones).
        rgb[glass] = col[glass] * alpha[glass, None] + rgb[glass] * (1 - alpha[glass, None])
        al[glass] = alpha[glass] + al[glass] * (1 - alpha[glass])

    def image(s, size):
        img = np.dstack([s.rgb, s.a])
        im = Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8), "RGBA")
        return im.resize((size, size), Image.LANCZOS)

def rot(axis, deg):
    t = math.radians(deg)
    c, s_ = math.cos(t), math.sin(t)
    if axis == "x":
        return np.array([[1, 0, 0], [0, c, -s_], [0, s_, c]])
    if axis == "y":
        return np.array([[c, 0, s_], [0, 1, 0], [-s_, 0, c]])
    return np.array([[c, -s_, 0], [s_, c, 0], [0, 0, 1]])

# Minecraft's face shading.
def shade_for(n):
    n = n / (np.linalg.norm(n) + 1e-9)
    return float(abs(n[0]) * 0.6 + abs(n[2]) * 0.8 + (1.0 if n[1] > 0 else 0.5) * abs(n[1]))

def draw_quads(quads, view, size, fit=None, ss=3):
    """quads: (world points 4x3, uvs, texture). view: 3x3 world→camera (x right, y up, z out)."""
    cam = []
    for pts, uvs, tex in quads:
        P = np.array(pts, np.float64)
        n = np.cross(P[1] - P[0], P[2] - P[0])
        C = P @ view.T
        cam.append((C, uvs, tex, shade_for(n)))
    allp = np.concatenate([c[0] for c in cam]) if cam else np.zeros((1, 3))
    lo, hi = allp.min(0), allp.max(0)
    if fit is None:
        span = max(hi[0] - lo[0], hi[1] - lo[1]) * 1.08
        mid = (lo + hi) / 2
    else:
        span, mid = fit
    cv = Canvas(size, ss)
    # Opaque first, translucent after (far to near).
    order = sorted(cam, key=lambda c: (c[2][..., 3].min() < 0.99, c[0][:, 2].mean()))
    for C, uvs, tex, sh in order:
        pts = [((p[0] - mid[0]) / span + 0.5, 0.5 - (p[1] - mid[1]) / span, p[2]) for p in C]
        cv.quad(pts, uvs, tex, sh)
    return cv.image(size)

# ---------------------------------------------------------------- block models

VANILLA = {
    "block/block": {},
    "block/cube": {"elements": [{"from": [0, 0, 0], "to": [16, 16, 16], "faces": {
        "down": {"texture": "#down"}, "up": {"texture": "#up"}, "north": {"texture": "#north"},
        "south": {"texture": "#south"}, "west": {"texture": "#west"}, "east": {"texture": "#east"}}}]},
    "block/cube_all": {"parent": "block/cube", "textures": {"particle": "#all", "down": "#all", "up": "#all",
        "north": "#all", "south": "#all", "west": "#all", "east": "#all"}},
    "block/leaves": {"parent": "block/cube_all"},
    "block/cube_column": {"parent": "block/cube", "textures": {"down": "#end", "up": "#end", "north": "#side",
        "south": "#side", "west": "#side", "east": "#side"}},
    "block/cube_column_horizontal": {"parent": "block/cube_column"},
    "block/cube_bottom_top": {"parent": "block/cube", "textures": {"down": "#bottom", "up": "#top",
        "north": "#side", "south": "#side", "west": "#side", "east": "#side"}},
    "block/cross": {"elements": [
        {"from": [0.8, 0, 8], "to": [15.2, 16, 8], "rotation": {"origin": [8, 8, 8], "axis": "y", "angle": 45, "rescale": True},
         "faces": {"north": {"uv": [0, 0, 16, 16], "texture": "#cross"}, "south": {"uv": [0, 0, 16, 16], "texture": "#cross"}}},
        {"from": [8, 0, 0.8], "to": [8, 16, 15.2], "rotation": {"origin": [8, 8, 8], "axis": "y", "angle": 45, "rescale": True},
         "faces": {"west": {"uv": [0, 0, 16, 16], "texture": "#cross"}, "east": {"uv": [0, 0, 16, 16], "texture": "#cross"}}}]},
    "block/slab": {"elements": [{"from": [0, 0, 0], "to": [16, 8, 16], "faces": {
        "down": {"texture": "#bottom"}, "up": {"texture": "#top"}, "north": {"texture": "#side"},
        "south": {"texture": "#side"}, "west": {"texture": "#side"}, "east": {"texture": "#side"}}}]},
    "block/stairs": {"elements": [
        {"from": [0, 0, 0], "to": [16, 8, 16], "faces": {"down": {"texture": "#bottom"}, "up": {"texture": "#top"},
            "north": {"texture": "#side"}, "south": {"texture": "#side"}, "west": {"texture": "#side"}, "east": {"texture": "#side"}}},
        {"from": [8, 8, 0], "to": [16, 16, 16], "faces": {"up": {"texture": "#top"}, "north": {"texture": "#side"},
            "south": {"texture": "#side"}, "west": {"texture": "#side"}, "east": {"texture": "#side"}}}]},
    "block/wall_inventory": {"elements": [
        {"from": [4, 0, 4], "to": [12, 16, 12], "faces": {d: {"texture": "#wall"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [5, 0, 0], "to": [11, 13, 16], "faces": {d: {"texture": "#wall"} for d in ("down", "up", "north", "south", "west", "east")}}]},
    "block/button_inventory": {"elements": [{"from": [5, 6, 6], "to": [11, 10, 10],
        "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}}]},
    "block/pressure_plate_up": {"elements": [{"from": [1, 0, 1], "to": [15, 1, 15],
        "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}}]},
    "block/fence_inventory": {"elements": [
        {"from": [6, 0, 0], "to": [10, 16, 4], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [6, 0, 12], "to": [10, 16, 16], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [7, 13, -2], "to": [9, 15, 18], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [7, 5, -2], "to": [9, 7, 18], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}}]},
    "block/template_fence_gate": {"elements": [
        {"from": [0, 5, 7], "to": [2, 16, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [14, 5, 7], "to": [16, 16, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [6, 6, 7], "to": [10, 15, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [2, 6, 7], "to": [6, 9, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [2, 12, 7], "to": [6, 15, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [10, 6, 7], "to": [14, 9, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}},
        {"from": [10, 12, 7], "to": [14, 15, 9], "faces": {d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}}]},
    "block/template_trapdoor_bottom": {"elements": [{"from": [0, 0, 0], "to": [16, 3, 16], "faces": {
        "down": {"uv": [0, 0, 16, 16], "texture": "#texture"}, "up": {"uv": [0, 0, 16, 16], "texture": "#texture"},
        "north": {"uv": [0, 16, 16, 13], "texture": "#texture"}, "south": {"uv": [0, 16, 16, 13], "texture": "#texture"},
        "west": {"uv": [0, 16, 16, 13], "texture": "#texture"}, "east": {"uv": [0, 16, 16, 13], "texture": "#texture"}}}]},
    "block/snow_height2": {"elements": [{"from": [0, 0, 0], "to": [16, 2, 16], "faces": {
        d: {"texture": "#texture"} for d in ("down", "up", "north", "south", "west", "east")}}]},
}

def load_model(ref):
    ns, path = ref.split(":") if ":" in ref else ("minecraft", ref)
    if ns == "minecraft":
        return VANILLA.get(path)
    f = os.path.join(JAR, "assets", ns, "models", path + ".json")
    return json.load(open(f)) if os.path.exists(f) else None

def resolve_model(ref):
    """Elements and textures with parents merged (child wins)."""
    textures, elements, chain = {}, None, []
    m = load_model(ref)
    while m is not None:
        chain.append(m)
        p = m.get("parent")
        m = load_model(p) if p else None
    for m in reversed(chain):
        textures.update(m.get("textures", {}))
        if "elements" in m:
            elements = m["elements"]
    def tref(t, depth=0):
        while depth < 10:
            if isinstance(t, dict):
                t = t.get("sprite")
            if not (t and t.startswith("#")):
                break
            t = textures.get(t[1:])
            depth += 1
        return t
    flat = [c.get("parent", "") for c in chain]
    return elements or [], tref, flat

FACE = {  # corner order: top-left, top-right, bottom-right, bottom-left as the texture sees it
    "north": lambda a, b: [(b[0], b[1], a[2]), (a[0], b[1], a[2]), (a[0], a[1], a[2]), (b[0], a[1], a[2])],
    "south": lambda a, b: [(a[0], b[1], b[2]), (b[0], b[1], b[2]), (b[0], a[1], b[2]), (a[0], a[1], b[2])],
    "west": lambda a, b: [(a[0], b[1], a[2]), (a[0], b[1], b[2]), (a[0], a[1], b[2]), (a[0], a[1], a[2])],
    "east": lambda a, b: [(b[0], b[1], b[2]), (b[0], b[1], a[2]), (b[0], a[1], a[2]), (b[0], a[1], b[2])],
    "up": lambda a, b: [(a[0], b[1], a[2]), (b[0], b[1], a[2]), (b[0], b[1], b[2]), (a[0], b[1], b[2])],
    "down": lambda a, b: [(a[0], a[1], b[2]), (b[0], a[1], b[2]), (b[0], a[1], a[2]), (a[0], a[1], a[2])],
}
def default_uv(face, a, b):
    if face in ("up", "down"):
        return [a[0], a[2], b[0], b[2]] if face == "up" else [a[0], 16 - b[2], b[0], 16 - a[2]]
    if face in ("north", "south"):
        return [16 - b[0], 16 - b[1], 16 - a[0], 16 - a[1]] if face == "north" else [a[0], 16 - b[1], b[0], 16 - a[1]]
    return [a[2], 16 - b[1], b[2], 16 - a[1]] if face == "west" else [16 - b[2], 16 - b[1], 16 - a[2], 16 - a[1]]

def model_quads(ref):
    elements, tref, chain = resolve_model(ref)
    quads = []
    for e in elements:
        a, b = e["from"], e["to"]
        r = e.get("rotation")
        for face, f in e.get("faces", {}).items():
            t = tref(f.get("texture"))
            if not t:
                continue
            tex = texture(t)
            th, tw = tex.shape[:2]
            uv = f.get("uv") or default_uv(face, a, b)
            u0, v0, u1, v1 = [x / 16 for x in uv]
            corners = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
            k = (f.get("rotation", 0) // 90) % 4
            corners = corners[k:] + corners[:k]
            uvs = [(u * tw, v * th) for u, v in corners]
            pts = np.array(FACE[face](a, b), np.float64)
            if r:
                o = np.array(r["origin"], np.float64)
                if "axis" in r:
                    R = rot(r["axis"], r["angle"])
                else:  # 26.x: Euler angles, X·Y·Z
                    R = rot("x", r.get("x", 0)) @ rot("y", r.get("y", 0)) @ rot("z", r.get("z", 0))
                pts = (pts - o) @ R.T
                if r.get("rescale") and "axis" in r:
                    sc = 1 / math.cos(math.radians(abs(r["angle"])))
                    m = np.ones(3); m["xyz".index(r["axis"])] = 1
                    for i in range(3):
                        if i != "xyz".index(r["axis"]):
                            m[i] = sc
                    pts = pts * m
                pts = pts + o
            quads.append((pts - 8, uvs, tex))
    return quads, chain

# The inventory's view of a block: 30° down, 225° around (vanilla's block display).
GUI_VIEW = rot("x", 30) @ rot("y", 225)
GUI_VIEW = np.diag([1, 1, 1]) @ GUI_VIEW

def block_icon(ref, size=128):
    quads, chain = model_quads(ref)
    if not quads:
        return None
    # Vanilla's gui display scales a full block to 0.625 of the slot; fit a 16-unit cube.
    span = 16 * 1.62
    return draw_quads(quads, GUI_VIEW, size, fit=(span, np.zeros(3)))

def flat_icon(ref, size=128):
    elements, tref, chain = resolve_model(ref)
    layers = []
    m = load_model(ref)
    tex = {}
    while m:
        for k, v in m.get("textures", {}).items():
            tex.setdefault(k, v)
        m = load_model(m["parent"]) if m.get("parent") else None
    for i in range(5):
        t = tex.get(f"layer{i}")
        if t:
            layers.append(t)
    if not layers and tex:
        layers = [next(iter(tex.values()))]
    out = Image.new("RGBA", (16, 16))
    for t in layers:
        if isinstance(t, dict):
            t = t.get("sprite")
        arr = (texture(t) * 255).astype(np.uint8)
        out.alpha_composite(Image.fromarray(arr, "RGBA").resize((16, 16), Image.NEAREST))
    return out.resize((size, size), Image.NEAREST)

def item_model_ref(item):
    j = json.load(open(os.path.join(A, "items", item + ".json")))["model"]
    while j.get("type") == "minecraft:select":
        j = j.get("fallback") or j["cases"][0]["model"]
    if j.get("type") == "minecraft:condition":
        j = j["on_false"]
    return j.get("model")

def item_icon(item, size=128):
    ref = item_model_ref(item)
    if not ref:
        return None
    elements, tref, chain = resolve_model(ref)
    if any(p.startswith("minecraft:item/") or p.startswith("item/") for p in chain) or not elements:
        return flat_icon(ref, size)
    return block_icon(ref, size)

# ---------------------------------------------------------------- lang

LANG = json.load(open(os.path.join(A, "lang", "en_us.json")))
def title(kind, name):
    if f"{kind}.the_sift.{name}.desc" in LANG:
        return LANG[f"{kind}.the_sift.{name}.desc"]
    return LANG.get(f"{kind}.the_sift.{name}") or LANG.get(f"block.the_sift.{name}") or name.replace("_", " ").title()

def font(px):
    for f in ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"):
        if os.path.exists(f):
            return ImageFont.truetype(f, px)
    return ImageFont.load_default()

BG = (24, 20, 40, 255)
SLOT = (52, 44, 78, 255)

def sheet(entries, cols, cell, path, heading):
    rows = (len(entries) + cols - 1) // cols
    pad, label = 14, 34
    W = cols * (cell + pad) + pad
    H = 70 + rows * (cell + label + pad) + pad
    img = Image.new("RGBA", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((pad, 18), heading, font=font(30), fill=(235, 225, 255, 255))
    f = font(13)
    for i, (name, icon) in enumerate(entries):
        x = pad + (i % cols) * (cell + pad)
        y = 70 + (i // cols) * (cell + label + pad)
        d.rounded_rectangle((x, y, x + cell, y + cell), 10, fill=SLOT)
        if icon is not None:
            ic = icon.resize((cell - 16, cell - 16), Image.LANCZOS)
            img.alpha_composite(ic, (x + 8, y + 8))
        words, line, lines = name.split(), "", []
        for w_ in words:
            t = (line + " " + w_).strip()
            if d.textlength(t, font=f) > cell and line:
                lines.append(line); line = w_
            else:
                line = t
        lines.append(line)
        for j, ln in enumerate(lines[:2]):
            d.text((x + cell / 2 - d.textlength(ln, font=f) / 2, y + cell + 3 + j * 15), ln, font=f, fill=(210, 200, 235, 255))
    img.save(path)
    return img

# ---------------------------------------------------------------- GeckoLib models

def geo_quads(name, tex_ref, pose=None):
    """The model's cubes as quads in Java model space (GeckoLib's reading of Bedrock geometry:
    x mirrored, bone and cube rotations (-x, -y, z), Z·Y·X order)."""
    g = json.load(open(os.path.join(A, "geckolib", "models", "entity", name + ".geo.json")))["minecraft:geometry"][0]
    tw, th = g["description"]["texture_width"], g["description"]["texture_height"]
    tex = texture(tex_ref)
    sx, sy = tex.shape[1] / tw, tex.shape[0] / th
    bones = {b["name"]: b for b in g["bones"]}
    pose = pose or {}

    def R(r):
        rx, ry, rz = r
        return rot("z", rz) @ rot("y", -ry) @ rot("x", -rx)

    def mirror(p):
        return np.array([-p[0], p[1], p[2]], np.float64)

    def bone_matrix(bname):
        """4x4 bone → model."""
        b = bones[bname]
        M = np.eye(4)
        piv = mirror(b.get("pivot", [0, 0, 0]))
        r = np.array(b.get("rotation", [0, 0, 0]), np.float64)
        pr, pt = pose.get(bname, ((0, 0, 0), (0, 0, 0)))
        r = r + np.array(pr)
        T1, T2, Rm = np.eye(4), np.eye(4), np.eye(4)
        T1[:3, 3] = piv + np.array([-pt[0], pt[1], pt[2]])
        T2[:3, 3] = -piv
        Rm[:3, :3] = R(r)
        M = T1 @ Rm @ T2
        if "parent" in b:
            M = bone_matrix(b["parent"]) @ M
        return M

    quads = []
    for bname, b in bones.items():
        BM = bone_matrix(bname)
        for c in b.get("cubes", []):
            o, sz = np.array(c["origin"], np.float64), np.array(c["size"], np.float64)
            inf = c.get("inflate", 0)
            lo = np.array([-(o[0] + sz[0]), o[1], o[2]]) - inf
            hi = lo + sz + 2 * inf
            CM = np.eye(4)
            if "rotation" in c:
                piv = mirror(c.get("pivot", [0, 0, 0]))
                T1, T2, Rm = np.eye(4), np.eye(4), np.eye(4)
                T1[:3, 3] = piv; T2[:3, 3] = -piv; Rm[:3, :3] = R(c["rotation"])
                CM = T1 @ Rm @ T2
            M = BM @ CM
            w, h, dd = sz
            uv = c.get("uv", [0, 0])
            mir = c.get("mirror", False)
            if isinstance(uv, dict):
                faces = {}
                for fname, fu in uv.items():
                    u, v = fu["uv"]; us, vs = fu.get("uv_size", [0, 0])
                    faces[fname] = (u, v, u + us, v + vs)
            else:
                u, v = uv
                # Bedrock box UV (as Java's ModelPart.Cube).
                faces = {
                    "up": (u + dd + w, v + dd, u + dd, v),
                    "down": (u + dd + w, v, u + dd + 2 * w, v + dd),
                    "east": (u, v + dd, u + dd, v + dd + h),
                    "north": (u + dd, v + dd, u + dd + w, v + dd + h),
                    "west": (u + dd + w, v + dd, u + 2 * dd + w, v + dd + h),
                    "south": (u + 2 * dd + w, v + dd, u + 2 * dd + 2 * w, v + dd + h),
                }
                if mir:
                    faces["east"], faces["west"] = faces["west"], faces["east"]
            # After the x mirror, Bedrock's east (+x) is Java's west side (-x) and vice versa.
            geom = {
                "north": FACE["north"], "south": FACE["south"],
                "east": FACE["west"], "west": FACE["east"],
                "up": FACE["up"], "down": FACE["down"],
            }
            for fname, (u0, v0, u1, v1) in faces.items():
                if fname not in geom:
                    continue
                if mir and not isinstance(uv, dict):
                    u0, u1 = u1, u0
                pts = np.array(geom[fname](lo, hi), np.float64)
                if fname == "up" and not isinstance(uv, dict):
                    pts = pts[[3, 2, 1, 0]]
                P = np.c_[pts, np.ones(4)] @ M.T
                uvs = [(u0 * sx, v0 * sy), (u1 * sx, v0 * sy), (u1 * sx, v1 * sy), (u0 * sx, v1 * sy)]
                quads.append((P[:, :3], uvs, tex))
    return quads

def anim_pose(name, anim, t):
    """Bone (rotation, position) at time t from a GeckoLib animation (linear keyframes)."""
    f = os.path.join(A, "geckolib", "animations", "entity", name + ".animation.json")
    a = json.load(open(f))["animations"].get(anim)
    if not a:
        return {}
    length = a.get("animation_length", 1) or 1
    t = t % length
    def sample(ch):
        if isinstance(ch, (int, float)):
            return [ch] * 3
        if isinstance(ch, list):
            return [x if isinstance(x, (int, float)) else 0 for x in ch]
        if isinstance(ch, dict):
            if "vector" in ch:
                return sample(ch["vector"])
            keys = sorted((float(k), sample(v.get("post", v.get("vector", v)) if isinstance(v, dict) else v)) for k, v in ch.items())
            if t <= keys[0][0]:
                return keys[0][1]
            for (t0, v0), (t1, v1) in zip(keys, keys[1:]):
                if t0 <= t <= t1:
                    k = (t - t0) / (t1 - t0 or 1)
                    return [a_ + (b_ - a_) * k for a_, b_ in zip(v0, v1)]
            return keys[-1][1]
        return [0, 0, 0]
    pose = {}
    for bone, ch in a.get("bones", {}).items():
        r = sample(ch.get("rotation", [0, 0, 0]))
        p = sample(ch.get("position", [0, 0, 0]))
        pose[bone] = (r, p)
    return pose

def animations(name):
    f = os.path.join(A, "geckolib", "animations", "entity", name + ".animation.json")
    return list(json.load(open(f))["animations"]) if os.path.exists(f) else []

MOB_VIEW = rot("x", 18) @ rot("y", 150)

def mob_picture(name, tex_ref, size=512, pose=None, glow=None, view=MOB_VIEW, fit=None):
    quads = geo_quads(name, tex_ref, pose)
    if glow:
        # The emissive layer drawn as a second skin, a hair outside (GeckoLib's AutoGlowing layer).
        quads += [(p, uv, texture(glow)) for p, uv, _ in geo_quads(name, tex_ref, pose)]
    return draw_quads(quads, view, size, fit=fit)

if __name__ == "__main__":
    items = sorted(f[:-5] for f in os.listdir(os.path.join(A, "items")))
    icons = {}
    for it in items:
        try:
            icons[it] = item_icon(it)
        except Exception as e:
            print("item", it, e)
            icons[it] = None
    for it, ic in icons.items():
        if ic is not None:
            os.makedirs(os.path.join(OUT, "icons"), exist_ok=True)
            ic.save(os.path.join(OUT, "icons", it + ".png"))
    blocks = [i for i in items if os.path.exists(os.path.join(A, "blockstates", i + ".json"))]
    eggs = [i for i in items if i.endswith("_spawn_egg")]
    rest = [i for i in items if i not in blocks and i not in eggs]
    sheet([(title("block", b), icons[b]) for b in blocks], 8, 128, os.path.join(OUT, "blocks.png"), f"The Sift: blocks ({len(blocks)})")
    sheet([(title("item", b), icons[b]) for b in rest], 8, 128, os.path.join(OUT, "items.png"), f"The Sift: items ({len(rest)})")
    sheet([(title("item", b), icons[b]) for b in eggs], 5, 128, os.path.join(OUT, "spawn_eggs.png"), "The Sift: spawn eggs")
    print(len(blocks), "blocks", len(rest), "items", len(eggs), "eggs")

    # The mobs: front and back, and a frame of their own animations; a GIF of each animation.
    MOBS = [
        ("blub", "Blub", "the_sift:entity/blub", ["hopping", "flapping", "sitting"]),
        ("sifter", "Sifter", "the_sift:entity/sifter", ["running", "attack"]),
        ("echo_golem", "Echo Golem", "the_sift:entity/echo_golem", ["walk", "carry_idle", "bow"]),
        ("singer", "Singer", "the_sift:entity/singer", ["appear", "sing", "disappear", "walk", "receive_soul"]),
        ("rift", "Rift", "the_sift:entity/rift/sift", ["appear"]),
    ]
    MOBS = [m for m in MOBS if os.path.exists(os.path.join(A, "geckolib", "models", "entity", m[0] + ".geo.json"))]
    cell = 300
    W = 4 * cell + 5 * 14
    H = 70 + len(MOBS) * (cell + 40) + 14
    img = Image.new("RGBA", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((14, 18), "The Sift: mobs (GeckoLib models, posed from their own animations)", font=font(28), fill=(235, 225, 255, 255))
    f = font(15)
    back = rot("x", 18) @ rot("y", -30)
    side = rot("x", 8) @ rot("y", 90)
    os.makedirs(os.path.join(OUT, "mobs"), exist_ok=True)
    for row, (name, label, tex, anims) in enumerate(MOBS):
        y = 70 + row * (cell + 40)
        shots = [("front", mob_picture(name, tex, cell)), ("back", mob_picture(name, tex, cell, view=back))]
        for a in anims[:2]:
            ln = json.load(open(os.path.join(A, "geckolib", "animations", "entity", name + ".animation.json")))["animations"][a].get("animation_length", 1) or 1
            shots.append((a, mob_picture(name, tex, cell, pose=anim_pose(name, a, ln * 0.35))))
        for i, (cap, im) in enumerate(shots[:4]):
            x = 14 + i * (cell + 14)
            d.rounded_rectangle((x, y, x + cell, y + cell), 12, fill=SLOT)
            img.alpha_composite(im, (x, y))
            t = f"{label}: {cap}"
            d.text((x + cell / 2 - d.textlength(t, font=f) / 2, y + cell + 8), t, font=f, fill=(210, 200, 235, 255))
        mob_picture(name, tex, 512).save(os.path.join(OUT, "mobs", name + ".png"))
        # A GIF per animation, framed on the whole animation so the model doesn't jump about.
        for a in anims:
            ln = json.load(open(os.path.join(A, "geckolib", "animations", "entity", name + ".animation.json")))["animations"][a].get("animation_length", 1) or 1
            n = max(8, min(40, int(ln * 20)))
            poses = [anim_pose(name, a, ln * k / n) for k in range(n)]
            allq = [q for p_ in poses for q in geo_quads(name, tex, p_)]
            C = np.concatenate([np.array(q[0]) @ MOB_VIEW.T for q in allq])
            lo, hi = C.min(0), C.max(0)
            fit = (max(hi[0] - lo[0], hi[1] - lo[1]) * 1.08, (lo + hi) / 2)
            frames = []
            for p_ in poses:
                fr = Image.new("RGBA", (256, 256), BG)
                fr.alpha_composite(mob_picture(name, tex, 256, pose=p_, fit=fit))
                frames.append(fr.convert("P", palette=Image.ADAPTIVE))
            frames[0].save(os.path.join(OUT, "mobs", f"{name}_{a}.gif"), save_all=True, append_images=frames[1:], duration=50, loop=0)
    img.save(os.path.join(OUT, "mobs.png"))
