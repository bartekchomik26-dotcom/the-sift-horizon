#!/usr/bin/env python3
"""Reads The Sift's structure templates (NBT) into the text the native mod builds in
(mixins/the_sift_native/data/templates/<name>.txt): `size x y z`, one `p <state>` per palette
entry, then `x y z <palette index>` per block (structure voids left out).
    python3 tools/gen_templates.py <unzipped jar> <out dir>
"""
import gzip, re, struct, sys
from pathlib import Path

JAR, OUT = Path(sys.argv[1]), Path(sys.argv[2])
S = JAR / 'data' / 'the_sift' / 'structure'
WANT = re.compile(r'^(abandoned_portal_[a-z_]+_\d+|main_portal(_[a-z_]+)?|overgrown_willow_\d+|ash_willow_\d+|soulshale_shelter_\d+)$')
OLD = {'abandoned_portal_overgrown', 'abandoned_portal_wastes'}

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

def load(p):
    b = gzip.decompress(p.read_bytes())
    n = struct.unpack_from('>H', b, 1)[0]
    return rd(b, 3 + n, 10)[0]

OUT.mkdir(parents=True, exist_ok=True)
names = []
for p in sorted(S.glob('*.nbt')):
    if not WANT.match(p.stem) or re.sub(r'_\d+$', '', p.stem) in OLD:
        continue
    t = load(p)
    pal = t['palette'] if 'palette' in t else t['palettes'][0]
    states = []
    for e in pal:
        # The Sift's mixin gives vanilla blocks an `ichorlogged` property; Horizon's have none (ichor
        # is water there).
        props = {k: v for k, v in e.get('Properties', {}).items() if k != 'ichorlogged'}
        states.append(e['Name'] + ('[' + ','.join(f'{k}={v}' for k, v in sorted(props.items())) + ']' if props else ''))
    lines = ['size ' + ' '.join(map(str, t['size']))] + ['p ' + s for s in states]
    for blk in sorted(t['blocks'], key=lambda b: (b['pos'][1], b['pos'][2], b['pos'][0])):
        if states[blk['state']] == 'minecraft:structure_void':
            continue
        lines.append(' '.join(map(str, blk['pos'])) + f" {blk['state']}")
    (OUT / f'{p.stem}.txt').write_text('\n'.join(lines) + '\n')
    names.append(p.stem)
print(len(names), 'templates')

# The decoration hook builds them in (with the placed-feature table), one include each.
HOOK = Path(__file__).resolve().parents[1] / 'mixins' / 'the_sift_native' / 'h_decorate.rs'
inc = lambda f: f'include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../native-data/the_sift_native/{f}"))'
rows = [f'        ("@placed", {inc("placed.txt")}),'] + [f'        ("{n}", {inc("templates/" + n + ".txt")}),' for n in names]
HOOK.write_text('''// A chunk is decorated: The Sift's own features first (its templates and placed-feature table built in).
fn the_sift_native_decorate(data: &Arc<GeneratorData>, seed: i64, region: &mut [ProtoChunk; 9]) {
    static TEMPLATES: &[(&str, &str)] = &[
''' + '\n'.join(rows) + '''
    ];
    the_sift_terrain::decorate(data, seed, region, TEMPLATES);
}
''')
