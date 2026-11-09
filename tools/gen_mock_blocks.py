#!/usr/bin/env python3
"""Writes harness/crates/world-mock/tests/sift_blocks.rs: every block of the content mod with the
properties its blockstate file names (and the vanilla blocks The Sift's templates hold), so the
terrain tests resolve states against what the mod really has."""
import json, re
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
props = {}
for p in sorted((ROOT / 'content/assets/the_sift/blockstates').glob('*.json')):
    j = json.loads(p.read_text())
    keys = {}
    variants = list(j.get('variants', {}).keys())
    for part in j.get('multipart', []):
        w = part.get('when', {})
        for cond in (w.get('OR') or w.get('AND') or [w]):
            for k, v in cond.items():
                keys.setdefault(k, set()).update(str(v).split('|'))
    for v in variants:
        for kv in filter(None, v.split(',')):
            k, val = kv.split('=')
            keys.setdefault(k, set()).add(val)
    props[f'the_sift:{p.stem}'] = keys
# Declared properties (state_properties) the blockstates may not list.
for p in (ROOT / 'content/data/the_sift/horizon/block').glob('*.json'):
    d = json.loads(p.read_text())
    for k, v in d.get('state_properties', {}).items():
        vals = v.get('values') or (['false', 'true'] if v.get('type') == 'boolean' else [str(i) for i in range(v.get('min', 0), v.get('max', 15) + 1)])
        props.setdefault(f'the_sift:{p.stem}', {}).setdefault(k, set()).update(map(str, vals))
# Templates' states fill in what the files leave to the template block (axis, facing, waterlogged...).
for t in (ROOT / 'mixins/the_sift_native/data/templates').glob('*.txt'):
    for line in t.read_text().splitlines():
        if not line.startswith('p '):
            continue
        s = line[2:]
        name, _, rest = s.partition('[')
        d = props.setdefault(name, {})
        for kv in filter(None, rest.rstrip(']').split(',')):
            k, v = kv.split('=')
            d.setdefault(k, set()).add(v)
FIRST = {'axis': 'y', 'distance': '7', 'half': 'lower', 'facing': 'north', 'type': 'single', 'mode': 'horn'}
for d in props.values():
    d.pop('ichorlogged', None)
    for k, vs in d.items():
        if vs <= {'true', 'false'}:
            vs |= {'true', 'false'}
        if k == 'distance':
            vs |= {str(i) for i in range(1, 8)}
def order(vals, key=''):
    vals = sorted(vals)
    if FIRST.get(key) in vals:
        return [FIRST[key]] + [v for v in order(set(vals) - {FIRST[key]})]
    return sorted(vals, key=lambda v: (not v.lstrip('-').isdigit(), int(v) if v.lstrip('-').isdigit() else 0, v))
lines = ['// Written by tools/gen_mock_blocks.py: the mod\'s blocks and the templates\' vanilla ones.', 'pub const SIFT_BLOCKS: &[(&str, &[(&str, &[&str])])] = &[']
for name in sorted(props):
    ps = ', '.join(f'("{k}", &[{", ".join(chr(34) + v + chr(34) for v in order(vs, k))}])' for k, vs in sorted(props[name].items()))
    lines.append(f'    ("{name}", &[{ps}]),')
lines.append('];')
(ROOT / 'harness/crates/world-mock/tests/sift_blocks.rs').write_text('\n'.join(lines) + '\n')
print(len(props), 'blocks')
