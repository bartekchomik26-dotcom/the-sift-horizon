#!/usr/bin/env python3
"""Builds the data pack that puts The Sift in the End's place (datapack/).

The End's level stem becomes The Sift's: its noise generator, multi-noise biomes and dimension
type. The Sift's worldgen comes as it is, except what Horizon can't run yet: features of The
Sift's own Java types (ichor lakes, willow trees, monoliths...) are taken out of the biomes (a data
pack can't name them; the native mod places them at decoration), and so are spawns of its own mobs
and its own ambient particles.
Everything taken out is listed in datapack/REMOVED.md.
    python3 gen_datapack.py <unzipped jar> <out dir>
"""
import json, shutil, sys
from pathlib import Path

JAR, OUT = Path(sys.argv[1]), Path(sys.argv[2])
NS = 'the_sift'
if OUT.exists():
    shutil.rmtree(OUT)
OUT.mkdir(parents=True)
D = JAR / 'data'
W = D / NS / 'worldgen'

# structure_set: (spacing, separation) in chunks; the jar has 26/7, 30/9 and 48/24.
DENSER = {'abandoned_camp_coral': (14, 4), 'ichor_dome': (16, 5), 'sculkers_lair': (28, 10)}

def load(p):
    return json.loads(Path(p).read_text())

def ichor_as_water(v):
    # Horizon's fluids are vanilla's: The Sift's ichor generates as water, which the native mod
    # draws as ichor (its sprites, untinted) and gives ichor's regeneration and drag.
    if isinstance(v, dict):
        return {k: ('minecraft:water' if k in ('id', 'Name') and x == f'{NS}:ichor' else ichor_as_water(x))
                for k, x in v.items()}
    if isinstance(v, list):
        return [ichor_as_water(x) for x in v]
    return v

def write(rel, obj):
    obj = ichor_as_water(obj)
    p = OUT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj, indent=2) + '\n')

def custom_types(v, out):
    if isinstance(v, dict):
        t = v.get('type')
        if isinstance(t, str) and t.startswith(f'{NS}:'):
            out.add(t)
        for x in v.values():
            custom_types(x, out)
    elif isinstance(v, list):
        for x in v:
            custom_types(x, out)
    return out

# Configured features of The Sift's own types, directly or nested.
# 26.2 (The Sift 2.0) names the folders configured_feature/ and configured_carver/; 26.3 feature/ and carver/.
RENAME = {'configured_feature': 'feature', 'configured_carver': 'carver'}
FEAT = W / 'configured_feature' if (W / 'configured_feature').exists() else W / 'feature'
configured = {p.stem: load(p) for p in FEAT.glob('*.json')}
def configured_custom(name, seen=()):
    j = configured.get(name.split(':', 1)[1]) if name.startswith(f'{NS}:') else None
    if j is None:
        return set()
    found = custom_types(j, set())
    for ref in refs(j):
        if ref not in seen:
            found |= configured_custom(ref, seen + (ref,))
    return found
def refs(v):
    if isinstance(v, str):
        return [v] if v.startswith(f'{NS}:') and v.split(':', 1)[1] in configured else []
    if isinstance(v, dict):
        return [r for x in v.values() for r in refs(x)]
    if isinstance(v, list):
        return [r for x in v for r in refs(x)]
    return []

removed = {'features': {}, 'spawns': set(), 'particles': set()}
placed_custom = {}
for p in (W / 'placed_feature').glob('*.json'):
    j = load(p)
    f = j['feature']
    found = configured_custom(f) if isinstance(f, str) else custom_types(f, set())
    found |= custom_types(j.get('placement', []), set())
    if found:
        placed_custom[f'{NS}:{p.stem}'] = sorted(found)


# ---- worldgen, as it is, but for the biomes
for p in sorted(W.rglob('*.json')):
    rel = p.relative_to(W)
    if rel.parts[0] == 'biome':
        continue
    rel = Path(RENAME.get(rel.parts[0], rel.parts[0]), *rel.parts[1:])
    j = load(p)
    if rel.parts[0] == 'processor_list':
        # The Sift's own processors (the lair's sculk, the camps' coral vines) are Java: left out.
        procs = j.get('processors', [])
        kept = [x for x in procs if not str(x.get('processor_type', '')).startswith(f'{NS}:')]
        for x in procs:
            if x not in kept:
                removed.setdefault('processors', set()).add(x['processor_type'])
        j['processors'] = kept
    if rel.parts[0] == 'structure' and j.get('type') == f'{NS}:sculkers_lair':
        # SculkersLairStructure: its corridors and rooms from its own pools, deep underground
        # (as vanilla's jigsaw places them; its own piece rules are Java).
        j = {'type': 'minecraft:jigsaw', 'biomes': j['biomes'], 'step': j['step'],
             'terrain_adaptation': j.get('terrain_adaptation', 'encapsulate'),
             'spawn_overrides': j.get('spawn_overrides', {}),
             'start_pool': f'{NS}:sculkers_lair/start', 'size': 7,
             'start_height': {'type': 'minecraft:uniform', 'min_inclusive': {'absolute': -40},
                              'max_inclusive': {'absolute': 0}},
             'max_distance_from_center': 80, 'use_expansion_hack': False}
        removed.setdefault('structures', set()).add(f'{NS}:sculkers_lair (as a jigsaw of its own pools)')
    if rel.parts[0] == 'structure_set' and rel.stem in DENSER:
        # Found more often than in the jar (asked for: they were hard to find).
        j['placement']['spacing'], j['placement']['separation'] = DENSER[rel.stem]
    write(f'data/{NS}/worldgen/{rel}', j)

def biome_26_3(j):
    # 26.2's spawners/spawn_costs/creature_spawn_probability and bare ambient particles become 26.3's
    # environment attributes (as The Sift 1.1.1, built for 26.3, has them).
    attrs = j.setdefault('attributes', {})
    if 'spawners' in j:
        cats = {}
        for cat, lst in j.pop('spawners').items():
            cats[cat] = [{'type': s['type'], 'weight': s['weight'],
                          'count': s['minCount'] if s['minCount'] == s['maxCount'] else
                          {'type': 'minecraft:uniform', 'min_inclusive': s['minCount'], 'max_inclusive': s['maxCount']}}
                         for s in lst]
        attrs['minecraft:gameplay/natural_mob_spawns'] = {'spawn_costs': j.pop('spawn_costs', {}), 'spawns_by_category': cats}
    if 'creature_spawn_probability' in j:
        attrs['minecraft:gameplay/creature_world_gen_spawn_probability'] = j.pop('creature_spawn_probability')
    amb = attrs.get('minecraft:visual/ambient_particles')
    if isinstance(amb, list):
        attrs['minecraft:visual/ambient_particles'] = {'argument': amb, 'modifier': 'append'}
    return j

for p in sorted((W / 'biome').glob('*.json')):
    j = biome_26_3(load(p))
    feats = []
    for step in j.get('features', []):
        keep = []
        for f in step:
            if f in placed_custom:
                removed['features'][f] = placed_custom[f]
            else:
                keep.append(f)
        feats.append(keep)
    j['features'] = feats
    # Water here is ichor (the native mod draws it with ichor's sprites), which has no tint.
    j.setdefault('effects', {})['water_color'] = '#ffffff'
    attrs = j.get('attributes', {})
    amb = attrs.get('minecraft:visual/ambient_particles')
    if amb:
        args = amb.get('argument', [])
        kept = [a for a in args if not str(a.get('particle', {}).get('type', '')).startswith(f'{NS}:')]
        for a in args:
            if a not in kept:
                removed['particles'].add(a['particle']['type'])
        if kept:
            amb['argument'] = kept
        else:
            del attrs['minecraft:visual/ambient_particles']
    spawns = attrs.get('minecraft:gameplay/natural_mob_spawns', {})
    for cat, lst in spawns.get('spawns_by_category', {}).items():
        keep = [s for s in lst if not s['type'].startswith(f'{NS}:')]
        removed['spawns'] |= {s['type'] for s in lst if s not in keep}
        spawns['spawns_by_category'][cat] = keep
    for mob in [m for m in spawns.get('spawn_costs', {}) if m.startswith(f'{NS}:')]:
        del spawns['spawn_costs'][mob]
    write(f'data/{NS}/worldgen/biome/{p.name}', j)

# ---- the End's level stem is The Sift's
stem = load(D / NS / 'dimension' / 'the_sift.json')
stem['type'] = f'{NS}:the_sift'
write('data/minecraft/dimension/the_end.json', stem)
dt = load(D / NS / 'dimension_type' / 'the_sift.json')
write(f'data/{NS}/dimension_type/the_sift.json', dt)

# ---- structures, tags, clock, timeline
def copy(src, dst):
    for p in sorted(src.rglob('*')):
        if p.is_file():
            q = OUT / dst / p.relative_to(src)
            q.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, q)
copy(D / NS / 'structure', f'data/{NS}/structure')
for sub in ['tags/worldgen', 'tags/block', 'timeline', 'world_clock']:
    if (D / NS / sub).exists():
        copy(D / NS / sub, f'data/{NS}/{sub}')
# The Sift's portal frames in Ancient Cities (vanilla's city centre, changed by The Sift).
copy(D / 'minecraft' / 'structure', 'data/minecraft/structure')

# What the native mod places, in each biome's own order (step, then list): one line a placed feature
# of The Sift's own type, with its placement (count, rarity, in_square, heightmap, biome filter).
order = []
for p in sorted((W / 'biome').glob('*.json')):
    for step, lst in enumerate(load(p).get('features', [])):
        for i, f in enumerate(lst):
            if f in removed['features']:
                order.append((step, i, f, p.stem))
rows = {}
for step, i, f, b in order:
    r = rows.setdefault(f, {'step': step, 'i': i, 'biomes': []})
    r['step'], r['i'] = min((r['step'], r['i']), (step, i))
    r['biomes'].append(b)
lines_p = []
for f, r in sorted(rows.items(), key=lambda kv: (kv[1]['step'], kv[1]['i'], kv[0])):
    pj = load(W / 'placed_feature' / (f.split(':')[1] + '.json'))
    cf = configured.get(pj['feature'].split(':')[1]) if isinstance(pj['feature'], str) else pj['feature']
    count, rarity, square, height, bfilter = 1, 1, 0, 0, 0
    for m in pj.get('placement', []):
        t = m['type'].split(':')[1]
        if t == 'count': count = m['count'] if isinstance(m['count'], int) else m['count'].get('max_inclusive', 1)
        elif t == 'rarity_filter': rarity = m['chance']
        elif t == 'in_square': square = 1
        elif t == 'heightmap': height = 1
        elif t == 'biome': bfilter = 1
    lines_p.append(f"{f.split(':')[1]} {cf['type'].split(':')[1]} {r['step']} {count} {rarity} {square} {height} {bfilter} "
                   + ','.join(sorted(set(r['biomes']))))
NATIVE = Path(__file__).resolve().parents[1] / 'mixins' / 'the_sift_native' / 'data' / 'placed.txt'
NATIVE.write_text('\n'.join(lines_p) + '\n')

write('pack.mcmeta', {'pack': {
    'description': "Mielon's The Sift in the End's place (Horizon port)",
    'min_format': [121, 0], 'max_format': [121, 0]}})

lines = ['# Left out until the native mod provides them', '',
         'Placed features of The Sift\'s own feature types (taken out of every biome):', '']
types = {}
for f, ts in sorted(removed['features'].items()):
    lines.append(f'- `{f}`: {", ".join(ts)}')
    for t in ts:
        types.setdefault(t, []).append(f)
lines += ['', 'Mob spawns (the native mod `the_sift_native` spawns them itself, by these lists: Horizon\'s',
          'spawner would spawn a zombie-template type by a zombie\'s rules): '
          + ', '.join(f'`{s}`' for s in sorted(removed['spawns'])),
          '', 'Processors: ' + ', '.join(f'`{s}`' for s in sorted(removed.get('processors', ()))),
          '', 'Structures changed: ' + ', '.join(f'`{s}`' for s in sorted(removed.get('structures', ()))),
          '', 'Ambient particles: ' + ', '.join(f'`{s}`' for s in sorted(removed['particles'])),
          '', 'Ichor (`the_sift:ichor`) generates as water: Horizon\'s fluids are vanilla\'s. In The Sift the',
          'native mod draws that water as ichor (ichor_still/ichor_flow, untinted) and gives it',
          'ichor\'s effects (Regeneration I, 101 ticks; drag x0.92/0.94/0.92). The features are placed',
          'by the native mod (`the_sift_native`, at chunk decoration).']
(OUT / 'REMOVED.md').write_text('\n'.join(lines) + '\n')
print(f"{len(removed['features'])} placed features held back ({len(types)} feature types), "
      f"{len(removed['spawns'])} mobs, {len(removed['particles'])} particles")
