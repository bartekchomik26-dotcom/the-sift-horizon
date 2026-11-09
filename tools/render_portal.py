#!/usr/bin/env python3
"""Draws the portal opening's previews from what the shipped code computes (the harness's
export_portal_preview writes harness/target/portal_data.json: portal.rs's pixel surface for a
20 by 6 portal and the screen on the way, at twelve moments) and from The Sift's own city centre:
  previews/portal_surface.png   the portal in its frame (the frame a plain tile standing in for
                                vanilla's reinforced deepslate)
  previews/portal_surface.gif   its surface breathing
  previews/portal_screen.gif    the screen on the way through
  previews/city_consoles.png    city_center_1 from above: the frame, the eight consoles, the tones
    python3 tools/render_portal.py <unzipped jar>
"""
import gzip, json, struct, sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
JAR = Path(sys.argv[1])
OUT = ROOT / 'previews'
data = json.loads((ROOT / 'harness' / 'target' / 'portal_data.json').read_text())
TEX = JAR / 'assets' / 'the_sift' / 'textures' / 'block'

def argb(c):
    return ((c >> 16) & 255, (c >> 8) & 255, c & 255, (c >> 24) & 255)

S = 32  # pixels a block
def surface_frame(frame, border=True):
    w, h = 20, 6
    pad = 1 if border else 0
    img = Image.new('RGBA', ((w + 2 * pad) * S, (h + 2 * pad) * S), (14, 16, 22, 255))
    if border:
        side = Image.open(TEX / 'reinforced_siftslate_side.png').convert('RGBA').resize((S, S), Image.NEAREST)
        for x in range(w + 2):
            for y in range(h + 2):
                if x in (0, w + 1) or y in (0, h + 1):
                    img.alpha_composite(side, (x * S, y * S))
    layer = Image.new('RGBA', img.size, (0, 0, 0, 0))
    for b in frame['blocks']:
        bx, by = b['x'] + pad, h - 1 - b['y'] + pad
        for (x0, y0, x1, y1), z, color in sorted(b['squares'], key=lambda s: s[1]):
            sq = Image.new('RGBA', img.size, (0, 0, 0, 0))
            d = ImageDraw.Draw(sq)
            px0, px1 = bx * S + round(x0 * S), bx * S + round(x1 * S)
            py0, py1 = by * S + round((1 - y1) * S), by * S + round((1 - y0) * S)
            d.rectangle([px0, py0, px1 - 1, py1 - 1], fill=argb(color))
            layer.alpha_composite(sq)
    # The rim where the portal meets the frame (portal.rs: RIM, 1/8 of a block).
    d = ImageDraw.Draw(layer)
    r = S // 8
    x0, y0, x1, y1 = pad * S, pad * S, (pad + w) * S, (pad + h) * S
    for rect in ([x0, y0, x0 + r, y1], [x1 - r, y0, x1, y1], [x0, y0, x1, y0 + r], [x0, y1 - r, x1, y1]):
        d.rectangle([rect[0], rect[1], rect[2] - 1, rect[3] - 1], fill=(242, 255, 255, 255))
    img.alpha_composite(layer)
    return img

def frame_tile():
    # Reinforced deepslate, as a plain dark tile with its bevel (vanilla's texture isn't here).
    t = Image.new('RGBA', (S, S), (58, 62, 64, 255))
    d = ImageDraw.Draw(t)
    d.rectangle([0, 0, S - 1, S - 1], outline=(92, 98, 98, 255), width=2)
    d.rectangle([5, 5, S - 6, S - 6], outline=(40, 44, 46, 255), width=2)
    d.rectangle([11, 11, S - 12, S - 12], fill=(104, 116, 112, 255))
    return t

def fast_surface(frame):
    # The same, drawn square by square straight onto one layer (fast, for the animation).
    w, h = 20, 6
    img = Image.new('RGBA', ((w + 2) * S, (h + 2) * S), (14, 16, 22, 255))
    side = frame_tile()
    for x in range(w + 2):
        for y in range(h + 2):
            if x in (0, w + 1) or y in (0, h + 1):
                img.alpha_composite(side, (x * S, y * S))
    for b in frame['blocks']:
        bx, by = b['x'] + 1, h - 1 - b['y'] + 1
        for (x0, y0, x1, y1), z, color in sorted(b['squares'], key=lambda s: s[1]):
            px0, px1 = bx * S + round(x0 * S), bx * S + round(x1 * S)
            py0, py1 = by * S + round((1 - y1) * S), by * S + round((1 - y0) * S)
            if px1 > px0 and py1 > py0:
                sq = Image.new('RGBA', (px1 - px0, py1 - py0), argb(color))
                img.alpha_composite(sq, (px0, py0))
    d = ImageDraw.Draw(img)
    r = S // 8
    x0, y0, x1, y1 = S, S, (1 + w) * S, (1 + h) * S
    for rect in ([x0, y0, x0 + r, y1], [x1 - r, y0, x1, y1], [x0, y0, x1, y0 + r], [x0, y1 - r, x1, y1]):
        d.rectangle([rect[0], rect[1], rect[2] - 1, rect[3] - 1], fill=(242, 255, 255, 255))
    return img

frames = [fast_surface(f) for f in data]
frames[0].convert('RGB').save(OUT / 'portal_surface.png')
frames[0].save(OUT / 'portal_surface.gif', save_all=True, append_images=frames[1:], duration=120, loop=0,
               disposal=2)

screens = []
for f in data:
    img = Image.new('RGBA', (640, 360), (0, 0, 0, 255))
    for (x, y, w, h), c in f['screen']:
        x0, y0, x1, y1 = round(x), round(y), round(x + w), round(y + h)
        if x1 > x0 and y1 > y0:
            img.alpha_composite(Image.new('RGBA', (x1 - x0, y1 - y0), argb(c)), (x0, y0))
    screens.append(img.convert('RGB'))
screens[0].save(OUT / 'portal_screen.gif', save_all=True, append_images=screens[1:], duration=120, loop=0)

# ---- city_center_1 from above: the highest block of each column, the frame and consoles marked.
def rd(b, i, t):
    if t == 1: return struct.unpack_from('>b', b, i)[0], i + 1
    if t == 2: return struct.unpack_from('>h', b, i)[0], i + 2
    if t == 3: return struct.unpack_from('>i', b, i)[0], i + 4
    if t == 4: return struct.unpack_from('>q', b, i)[0], i + 8
    if t == 5: return struct.unpack_from('>f', b, i)[0], i + 4
    if t == 6: return struct.unpack_from('>d', b, i)[0], i + 8
    if t == 7:
        n = struct.unpack_from('>i', b, i)[0]; i += 4; return b[i:i + n], i + n
    if t == 8:
        n = struct.unpack_from('>H', b, i)[0]; i += 2; return b[i:i + n].decode('utf8', 'replace'), i + n
    if t == 9:
        et = b[i]; n = struct.unpack_from('>i', b, i + 1)[0]; i += 5; out = []
        for _ in range(n):
            v, i = rd(b, i, et); out.append(v)
        return out, i
    if t == 10:
        d = {}
        while True:
            et = b[i]; i += 1
            if et == 0: return d, i
            n = struct.unpack_from('>H', b, i)[0]; i += 2; k = b[i:i + n].decode(); i += n
            d[k], i = rd(b, i, et)
    if t == 11:
        n = struct.unpack_from('>i', b, i)[0]; i += 4; return list(struct.unpack_from('>%di' % n, b, i)), i + 4 * n
    if t == 12:
        n = struct.unpack_from('>i', b, i)[0]; i += 4; return list(struct.unpack_from('>%dq' % n, b, i)), i + 8 * n

raw = gzip.decompress((JAR / 'data/minecraft/structure/ancient_city/city_center/city_center_1.nbt').read_bytes())
n = struct.unpack_from('>H', raw, 1)[0]
t = rd(raw, 3 + n, 10)[0]
pal = t['palette']
size = t['size']
top = {}
for blk in t['blocks']:
    x, y, z = blk['pos']
    name = pal[blk['state']]['Name']
    if name in ('minecraft:air', 'minecraft:structure_void', 'minecraft:jigsaw'):
        continue
    if y <= 24 and (y > top.get((x, z), (-1, ''))[0]):
        top[(x, z)] = (y, name)
COL = {'minecraft:reinforced_deepslate': (52, 64, 60), 'the_sift:sonorous_deepslate': (240, 240, 240)}
def colour(name, y):
    if name in COL: return COL[name]
    k = 70 + y * 5
    if 'sculk' in name: return (12, 40 + y * 2, 50 + y * 2)
    if 'lantern' in name or 'candle' in name: return (230, 190, 90)
    if 'wool' in name: return (110, 120, 140)
    return (k, k, k + 6)
P = 22
img = Image.new('RGB', (size[2] * P + 380, size[0] * P + 40), (18, 20, 26))
d = ImageDraw.Draw(img)
for (x, z), (y, name) in top.items():
    d.rectangle([z * P + 20, x * P + 20, z * P + 20 + P - 2, x * P + 20 + P - 2], fill=colour(name, y))
TONES = [0xB02E26, 0x3C44AA, 0xF9801D, 0x8932B8, 0x169C9C, 0x80C71F, 0xFED83D, 0xF38BAA]
consoles = sorted(blk['pos'] for blk in t['blocks'] if pal[blk['state']]['Name'] == 'the_sift:sonorous_deepslate')
try:
    font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 13)
    small = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf', 12)
except OSError:
    font = small = ImageFont.load_default()
for (x, y, z) in consoles:
    d.rectangle([z * P + 20, x * P + 20, z * P + 20 + P - 2, x * P + 20 + P - 2], fill=(236, 240, 244), outline=(140, 255, 246), width=2)
for z in range(10, 32):
    d.rectangle([z * P + 20, 13 * P + 20, z * P + 20 + P - 2, 13 * P + 20 + P - 2], outline=(140, 255, 246), width=2)
lx = size[2] * P + 36
lines = [('city_center_1 (The Sift 2.0.1), from above', font),
         ('outlined: the reinforced deepslate frame', small),
         ('(x=13, z 10..31, y 17..24)', small),
         ('white: the eight sonorous consoles (y=10)', small),
         ('', small), ('Blow a goat horn by the frame:', font),
         ('a singer rises in it, sings, and', small), ('wakes each console (horn -> note).', small),
         ('', small), ('Put note blocks on them, tune by', font), ('right click, strike in this order:', font)]
y = 24
for text, f in lines:
    d.text((lx, y), text, fill=(230, 236, 240), font=f); y += 18
for i, tone in enumerate([1, 3, 7, 6, 5, 2, 4, 8]):
    c = TONES[tone - 1]
    d.rectangle([lx + i * 28, y + 4, lx + i * 28 + 22, y + 26], fill=((c >> 16) & 255, (c >> 8) & 255, c & 255))
    d.text((lx + i * 28 + 7, y + 7), str(tone), fill=(0, 0, 0), font=font)
y += 36
for text in ['tone n: the note block\'s note mod 8, +1', '(any console may play any tone)', '',
             'The frame fills with a Sift portal', 'from its edges in. Backwards',
             '(8 4 2 5 6 7 3 1) closes it.']:
    d.text((lx, y), text, fill=(230, 236, 240), font=small); y += 18
img.save(OUT / 'city_consoles.png')
print('portal previews written')
