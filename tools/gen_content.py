#!/usr/bin/env python3
"""Builds The Sift's Horizon content mod (content/) from the Fabric jar (MIT, Mielon).

Blocks and items become declarations over vanilla templates (data/the_sift/horizon/), the
templates being the blocks The Sift's own code copies (BlockBehaviour.Properties.ofFullCopy).
Assets, recipes, loot, tags and advancements are the jar's own, as they are (26.3 formats).
The mobs are living types over the zombie, brought to life by the native mod (the_sift_native);
what a content mod can't hold (the Ichor fluid, particles, the worldgen) the native mods and the
End data pack carry.
    python3 gen_content.py <unzipped jar> <out dir>
"""
import json, shutil, sys
from pathlib import Path

JAR, OUT = Path(sys.argv[1]), Path(sys.argv[2])
NS = 'the_sift'
if OUT.exists():
    shutil.rmtree(OUT)
OUT.mkdir(parents=True)

def write(rel, obj):
    p = OUT / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(obj, indent=2) + '\n')

# ------------------------------------------------------------------ blocks
# id: (template, extra declaration keys). The template is what ModBlocks copies.
B = {}
def block(name, template, **kw):
    B[name] = dict(template=f'minecraft:{template}', **kw)

for n, t in [
    ('siftslate', 'deepslate'), ('cobbled_siftslate', 'cobbled_deepslate'),
    ('cobbled_siftslate_stairs', 'cobbled_deepslate_stairs'), ('cobbled_siftslate_slab', 'cobbled_deepslate_slab'),
    ('cobbled_siftslate_wall', 'cobbled_deepslate_wall'), ('siftslate_bricks', 'deepslate_bricks'),
    ('siftslate_brick_stairs', 'deepslate_brick_stairs'), ('siftslate_brick_slab', 'deepslate_brick_slab'),
    ('siftslate_brick_wall', 'deepslate_brick_wall'), ('chiseled_siftslate', 'chiseled_deepslate'),
    ('ichor_glass', 'glass'), ('ichor_glass_pane', 'glass_pane'), ('ichor_snow_block', 'snow_block'),
    ('ichor_snow', 'snow'), ('sculkflower', 'cactus_flower'), ('healthy_sculk', 'sculk'),
    ('dry_healthy_sculk', 'sculk'),
    ('overgrown_willow_planks', 'oak_planks'), ('overgrown_willow_log', 'oak_log'),
    ('overgrown_willow_wood', 'oak_wood'), ('stripped_overgrown_willow_log', 'stripped_oak_log'),
    ('stripped_overgrown_willow_wood', 'stripped_oak_wood'), ('overgrown_willow_stairs', 'oak_stairs'),
    ('overgrown_willow_slab', 'oak_slab'), ('overgrown_willow_fence', 'oak_fence'),
    ('overgrown_willow_fence_gate', 'oak_fence_gate'), ('overgrown_willow_door', 'oak_door'),
    ('overgrown_willow_trapdoor', 'oak_trapdoor'), ('overgrown_willow_pressure_plate', 'oak_pressure_plate'),
    ('overgrown_willow_button', 'oak_button'), ('overgrown_willow_shelf', 'oak_shelf'),
    ('siftslate_coal_ore', 'deepslate_coal_ore'), ('siftslate_diamond_ore', 'deepslate_diamond_ore'),
    ('siftslate_emerald_ore', 'deepslate_emerald_ore'), ('siftslate_charoite_ore', 'deepslate_lapis_ore'),
    ('siftslate_siftite_ore', 'deepslate_diamond_ore'), ('reinforced_siftslate', 'reinforced_deepslate'),
]:
    block(n, t)
block('sift_rubble', 'gravel', properties={'sounds': 'minecraft:suspicious_gravel'})
# SiftslateGrowthBlock: stone with deepslate's sounds; `snowy` is its own state.
block('siftslate_growth', 'stone', properties={'sounds': 'minecraft:deepslate', 'random_ticks': True},
      state_properties={'snowy': {'type': 'boolean'}})
block('dry_healthy_sculk_growth', 'sculk', properties={'random_ticks': True})
# Sculkflower crop: beetroots' template; the world mod plants, grows and ripens it into a sculkflower.
block('sculkflower_crop', 'beetroots', item=False)
block('ichor_cauldron', 'water_cauldron', properties={'light': 10}, item=False)
block('soul_block', 'soul_soil', properties={'hardness': 0.35, 'explosion_resistance': 0.35, 'light': 14})
# SiftPlantBlock: a no-collision plant that is replaced when built over.
block('overgrown_chard', 'short_grass', properties={'sounds': 'minecraft:beetroots'})
for n in ['overgrown_stalks', 'overgrown_fronds', 'overgrown_lotus', 'sunburst_plant', 'whisperbloom',
          'siftslate_stalks', 'healthy_sculk_sprouts', 'dry_healthy_sculk_sprouts']:
    block(n, 'nether_sprouts')
block('siftslate_hanging_roots', 'hanging_roots', properties={'sounds': 'minecraft:nether_sprouts'})
block('overgrown_hanging_roots', 'hanging_roots', properties={'sounds': 'minecraft:nether_sprouts'})
block('overgrown_willow_foliage', 'oak_leaves', properties={'hardness': 0.7, 'sounds': 'minecraft:nether_wart'})
# Grows a Sift willow, not an oak: no growth until the native mod grows its own tree.
block('overgrown_willow_sapling', 'oak_sapling', properties={'random_ticks': False})
VINE_SIDES = {f'{s}_bottom': {'type': 'boolean'} for s in ['north', 'east', 'south', 'west']}
VINE_SIDES['bottom'] = {'type': 'boolean'}
block('overgrown_willow_vines', 'vine', state_properties=VINE_SIDES)
block('overgrown_willow_vines_plant', 'vine', state_properties=VINE_SIDES, item=False)
block('overgrown_willow_sign', 'oak_sign', wood_type='the_sift:overgrown_willow',
      wall_block='the_sift:overgrown_willow_wall_sign')
block('overgrown_willow_wall_sign', 'oak_wall_sign', wood_type='the_sift:overgrown_willow', item=False)
block('overgrown_willow_hanging_sign', 'oak_hanging_sign', wood_type='the_sift:overgrown_willow',
      wall_block='the_sift:overgrown_willow_wall_hanging_sign')
block('overgrown_willow_wall_hanging_sign', 'oak_wall_hanging_sign', wood_type='the_sift:overgrown_willow',
      item=False)
# The Sift's resonant blocks: reinforced deepslate with its horn/note mode.
# SonorousDeepslateBlock's default is the horn: an ancient city's consoles sleep until a singer wakes them.
block('sonorous_deepslate', 'reinforced_deepslate',
      state_properties={'mode': {'type': 'enum', 'values': ['note', 'horn']}}, default_state={'mode': 'horn'})
# The portal: an End portal (The Sift stands in the End's place), standing up like a Nether portal.
block('sift_portal', 'end_portal', properties={'light': 12, 'render_shape': 'MODEL', 'occludes': False},
      state_properties={'axis': {'type': 'enum', 'values': ['x', 'z']}},
      shapes={'property': 'axis', 'values': {'x': [[0, 0, 0.375, 1, 1, 0.625]], 'z': [[0.375, 0, 0, 0.625, 1, 1]]}},
      collision_shapes={'default': []}, item=False)

# ---- The Sift 2.0's blocks (ModBlocks: what each copies).
for n, t in [
    ('soulshale', 'cobbled_deepslate'), ('soulshale_stairs', 'cobbled_deepslate_stairs'),
    ('soulshale_slab', 'cobbled_deepslate_slab'), ('soulshale_wall', 'cobbled_deepslate_wall'),
    ('polished_soulshale', 'polished_deepslate'), ('polished_soulshale_stairs', 'polished_deepslate_stairs'),
    ('polished_soulshale_slab', 'polished_deepslate_slab'), ('polished_soulshale_wall', 'polished_deepslate_wall'),
    ('soulshale_bricks', 'deepslate_bricks'), ('soulshale_brick_stairs', 'deepslate_brick_stairs'),
    ('soulshale_brick_slab', 'deepslate_brick_slab'), ('soulshale_brick_wall', 'deepslate_brick_wall'),
    ('chiseled_soulshale', 'chiseled_deepslate'), ('soulshale_totem', 'chiseled_deepslate'),
    ('soulshale_coal_ore', 'deepslate_coal_ore'), ('soulshale_diamond_ore', 'deepslate_diamond_ore'),
    ('soulshale_emerald_ore', 'deepslate_emerald_ore'), ('soulshale_charoite_ore', 'deepslate_lapis_ore'),
    ('soulshale_siftite_ore', 'deepslate_diamond_ore'), ('reinforced_soulshale', 'reinforced_deepslate'),
    ('charoite_block', 'lapis_block'), ('siftite_block', 'netherite_block'),
    ('coral_soil', 'dirt'), ('coral_growth', 'moss_block'), ('bright_coral_growth', 'moss_block'),
    ('healthy_sculk_cartilage', 'sculk'), ('healthy_sculk_pores', 'sculk'),
    ('ash_willow_planks', 'oak_planks'), ('ash_willow_log', 'oak_log'), ('ash_willow_wood', 'oak_wood'),
    ('stripped_ash_willow_log', 'stripped_oak_log'), ('stripped_ash_willow_wood', 'stripped_oak_wood'),
    ('ash_willow_stairs', 'oak_stairs'), ('ash_willow_slab', 'oak_slab'), ('ash_willow_fence', 'oak_fence'),
    ('ash_willow_fence_gate', 'oak_fence_gate'), ('ash_willow_door', 'oak_door'),
    ('ash_willow_trapdoor', 'oak_trapdoor'), ('ash_willow_pressure_plate', 'oak_pressure_plate'),
    ('ash_willow_button', 'oak_button'), ('ash_willow_shelf', 'oak_shelf'),
]:
    block(n, t)
# SoulshaleGrowthBlock: soulshale that spreads, snowy like the siftslate growth.
block('soulshale_growth', 'cobbled_deepslate', properties={'random_ticks': True},
      state_properties={'snowy': {'type': 'boolean'}})
# RoyalSculkBlock: sculk that wakes (`active`) by its lair.
block('royal_sculk', 'sculk', state_properties={'active': {'type': 'boolean'}})
for n in ['coral_bush', 'bright_coral_bush', 'grim_bush', 'blue_saucer_bloom', 'green_saucer_bloom',
          'healthy_sculk_cartilage_sprout']:
    block(n, 'nether_sprouts')
# BlubbyBerriesBushBlock: a two-high berry bush (sweet berries' behaviour, its own halves and `grown`).
# Its template is the pitcher plant: a two-high plant with nothing of the sweet berry bush's (no
# scratching, no berries of vanilla's); the world mod picks it, grows its berries and takes bone meal.
block('blubby_berries_bush', 'pitcher_plant', item=False, properties={'random_ticks': True},
      state_properties={'grown': {'type': 'boolean'}})
block('ash_willow_foliage', 'oak_leaves', properties={'hardness': 0.7, 'sounds': 'minecraft:nether_wart'})
block('ash_willow_sapling', 'oak_sapling', properties={'random_ticks': False})
block('ash_willow_vines', 'vine', state_properties=VINE_SIDES)
block('ash_willow_vines_plant', 'vine', state_properties=VINE_SIDES, item=False)
block('ash_willow_sign', 'oak_sign', wood_type='the_sift:ash_willow', wall_block='the_sift:ash_willow_wall_sign')
block('ash_willow_wall_sign', 'oak_wall_sign', wood_type='the_sift:ash_willow', item=False)
block('ash_willow_hanging_sign', 'oak_hanging_sign', wood_type='the_sift:ash_willow',
      wall_block='the_sift:ash_willow_wall_hanging_sign')
block('ash_willow_wall_hanging_sign', 'oak_wall_hanging_sign', wood_type='the_sift:ash_willow', item=False)

# Only what this jar has (1.1.1's lotus is gone from 2.0).
JAR_BLOCKS = {p.stem for p in (JAR / 'assets' / NS / 'blockstates').glob('*.json')}
B = {n: d for n, d in B.items() if n in JAR_BLOCKS}
for name, decl in B.items():
    d = dict(decl)
    write(f'data/{NS}/horizon/block/{name}.json', d)

# ------------------------------------------------------------------ items
I = {}
# The mobs the native mod brings to life (those with their own GeckoLib model in this jar).
GECKO = JAR / 'assets' / NS / 'geckolib' / 'models' / 'entity'
# (each also an entity type below)
MOBS = [m for m in ['blub', 'sifter', 'echo_golem', 'singer', 'stalker', 'nuzzle', 'slabber', 'nester', 'pollinator',
        'seedling', 'scavenger', 'monarch'] if (GECKO / f'{m}.geo.json').exists()]  # 2.0 has no sifter
# The dark sniffer is vanilla's sniffer retextured (DarkSnifferRenderer): the native mod draws it from vanilla's
# sniffer geometry (data/geckolib/dark_sniffer.geo.json) with The Sift's texture.
if (JAR / 'assets' / NS / 'textures' / 'entity' / 'dark_sniffer' / 'dark_sniffer.png').exists():
    MOBS.append('dark_sniffer')
def item(name, template, **kw):
    I[name] = dict(template=f'minecraft:{template}', **kw)

for n, t in [('ichor_shard', 'amethyst_shard'), ('charoite', 'lapis_lazuli'), ('siftite_nugget', 'iron_nugget'),
             ('siftite_ingot', 'netherite_ingot'), ('raw_sifter_meat', 'porkchop'),
             ('cooked_sifter_meat', 'cooked_porkchop'),
             ('siftite_upgrade_smithing_template', 'netherite_upgrade_smithing_template')]:
    item(n, t)
item('ichor_snowball', 'snowball')
# The Ichor bucket: a water bucket (water in The Sift is drawn and acts as ichor).
item('ichor_bucket', 'water_bucket')
item('ichor_bottle', 'honey_bottle')
item('sift_rift', 'echo_shard', max_stack_size=1, rarity='epic')
item('sculkflower_seeds', 'beetroot_seeds')
for disc, song in [('music_disc_rift', 'rift'), ('music_disc_wellspring', 'wellspring')]:
    item(disc, 'music_disc_13', rarity='rare', components={'minecraft:jukebox_playable': f'{NS}:{song}'})
item('overgrown_willow_boat', 'oak_boat')
item('overgrown_willow_chest_boat', 'oak_chest_boat')
# The Sift 2.0's items.
item('ash_willow_boat', 'oak_boat')
item('ash_willow_chest_boat', 'oak_chest_boat')
# Blubby berries: food (3, saturation 0.6). A content item can't place a block of its own yet, so the
# berries are eaten, not planted (the bush grows where The Sift puts it).
item('blubby_berries', 'apple', components={'minecraft:food': {'nutrition': 3, 'saturation': 3.6}})
for n, t in [('raw_slabber_meat', 'beef'), ('cooked_slabber_meat', 'cooked_beef'), ('raw_nuzzle_meat', 'mutton'),
             ('cooked_nuzzle_meat', 'cooked_mutton'), ('sifter_hide', 'leather')]:
    item(n, t)
item('pollinator_drool', 'honey_bottle', max_stack_size=16)
item('monarch_membrane', 'phantom_membrane', rarity='rare')
item('monarch_upgrade_smithing_template', 'netherite_upgrade_smithing_template', rarity='uncommon')
# MonarchElytraItem: an elytra (432 durability, repaired with phantom membrane, no damage when hurt).
item('monarch_elytra', 'elytra', rarity='epic', components={
    'minecraft:max_damage': 432,
    'minecraft:repairable': {'items': 'minecraft:phantom_membrane'},
    'minecraft:equippable': {'slot': 'chest', 'asset_id': f'{NS}:monarch_elytra',
                             'equip_sound': 'minecraft:item.armor.equip_elytra', 'damage_on_hurt': False}})
# SifterCapItem: a leather helmet in The Sift's own look.
item('sifter_cap', 'leather_helmet', components={
    'minecraft:equippable': {'slot': 'head', 'asset_id': f'{NS}:sifter_cap',
                             'equip_sound': 'minecraft:item.armor.equip_leather'}})

# Siftite tools: netherite's tier plus 470 durability, 1.2x speed and +1 damage (ModItems).
REPAIR = {'items': f'#{NS}:siftite_repair_materials'}
DUR, SPEED, BONUS = 2031 + 470, 9.0 * 1.2, 4.0 + 1.0
def tool(name, kind, base_damage, attack_speed, mineable):
    comps = {
        'minecraft:max_damage': DUR,
        'minecraft:repairable': REPAIR,
        'minecraft:attribute_modifiers': [
            {'type': 'minecraft:attack_damage', 'id': 'minecraft:base_attack_damage',
             'amount': base_damage + BONUS, 'operation': 'add_value', 'slot': 'mainhand'},
            {'type': 'minecraft:attack_speed', 'id': 'minecraft:base_attack_speed',
             'amount': attack_speed, 'operation': 'add_value', 'slot': 'mainhand'}],
    }
    if mineable:
        comps['minecraft:tool'] = {'rules': [
            {'blocks': '#minecraft:incorrect_for_netherite_tool', 'correct_for_drops': False},
            {'blocks': f'#minecraft:mineable/{mineable}', 'correct_for_drops': True, 'speed': SPEED}],
            'damage_per_block': 1}
    item(name, f'netherite_{kind}', components=comps)
tool('siftite_sword', 'sword', 3.0, -2.4, None)
tool('siftite_shovel', 'shovel', 1.5, -3.0, 'shovel')
tool('siftite_pickaxe', 'pickaxe', 1.0, -2.8, 'pickaxe')
tool('siftite_axe', 'axe', 5.0, -3.0, 'axe')
tool('siftite_hoe', 'hoe', -BONUS, 0.0, 'hoe')  # hoes: damage 0 + bonus offset (vanilla: -bonus)
item('siftite_spear', 'netherite_spear', components={'minecraft:max_damage': DUR, 'minecraft:repairable': REPAIR})

# Siftite armour: netherite's durability + 4, defence 3/6/8/3, toughness 3, knockback 0.15.
ARMOR_DUR = 37 + 4
for piece, slot, mult, defence in [('helmet', 'head', 11, 3), ('chestplate', 'chest', 16, 8),
                                   ('leggings', 'legs', 15, 6), ('boots', 'feet', 13, 3)]:
    gid = f'minecraft:armor.{piece}'
    item(f'siftite_{piece}', f'netherite_{piece}', components={
        'minecraft:max_damage': ARMOR_DUR * mult,
        'minecraft:repairable': REPAIR,
        'minecraft:equippable': {'slot': slot, 'asset_id': f'{NS}:siftite',
                                 'equip_sound': f'{NS}:item.siftite_armor.equip'},
        'minecraft:attribute_modifiers': [
            {'type': 'minecraft:armor', 'id': gid, 'amount': defence, 'operation': 'add_value', 'slot': slot},
            {'type': 'minecraft:armor_toughness', 'id': gid, 'amount': 3.0, 'operation': 'add_value', 'slot': slot},
            {'type': 'minecraft:knockback_resistance', 'id': gid, 'amount': 0.15, 'operation': 'add_value',
             'slot': slot}],
    })

# Spawn eggs of the mobs the native mod brings to life (the dark sniffer's waits for its model).
for mob in MOBS:
    item(f'{mob}_spawn_egg', 'zombie_spawn_egg')

JAR_ITEMS = {p.stem for p in (JAR / 'assets' / NS / 'items').glob('*.json')}
I = {n: d for n, d in I.items() if n in JAR_ITEMS}
for name, decl in I.items():
    write(f'data/{NS}/horizon/item/{name}.json', decl)

# ------------------------------------------------------------------ entity types
# The Sift's mobs as living types over the zombie (the one template with a mob's goals): their
# createAttributes; the native mod (the_sift_native) gives them their size, sounds, goals and models.
ENTITY = {
    'blub': {'minecraft:max_health': 20.0, 'minecraft:movement_speed': 0.3, 'minecraft:attack_damage': 3.0,
             'minecraft:follow_range': 24.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'sifter': {'minecraft:max_health': 40.0, 'minecraft:movement_speed': 0.31, 'minecraft:attack_damage': 10.0,
               'minecraft:follow_range': 28.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'echo_golem': {'minecraft:max_health': 34.0, 'minecraft:movement_speed': 0.23, 'minecraft:follow_range': 128.0,
                   'minecraft:step_height': 1.5, 'minecraft:knockback_resistance': 0.25, 'minecraft:armor': 0.0},
    'singer': {'minecraft:max_health': 20.0, 'minecraft:movement_speed': 0.27, 'minecraft:knockback_resistance': 0.0,
               'minecraft:armor': 0.0},
    # The Sift 2.0's mobs.
    'stalker': {'minecraft:max_health': 40.0, 'minecraft:movement_speed': 0.3, 'minecraft:attack_damage': 7.0,
                'minecraft:follow_range': 32.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 2.0},
    'scavenger': {'minecraft:max_health': 24.0, 'minecraft:movement_speed': 0.28, 'minecraft:attack_damage': 4.0,
                  'minecraft:follow_range': 24.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'monarch': {'minecraft:max_health': 300.0, 'minecraft:movement_speed': 0.3, 'minecraft:attack_damage': 12.0,
                'minecraft:follow_range': 48.0, 'minecraft:step_height': 1.5, 'minecraft:knockback_resistance': 0.8,
                'minecraft:armor': 6.0},
    'nester': {'minecraft:max_health': 26.0, 'minecraft:movement_speed': 0.25, 'minecraft:attack_damage': 5.0,
               'minecraft:follow_range': 16.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'seedling': {'minecraft:max_health': 10.0, 'minecraft:movement_speed': 0.32, 'minecraft:attack_damage': 2.0,
                 'minecraft:follow_range': 16.0, 'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'nuzzle': {'minecraft:max_health': 16.0, 'minecraft:movement_speed': 0.23, 'minecraft:follow_range': 16.0,
               'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    'slabber': {'minecraft:max_health': 22.0, 'minecraft:movement_speed': 0.2, 'minecraft:follow_range': 16.0,
                'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
    # DarkSnifferEntity.createAttributes.
    'dark_sniffer': {'minecraft:max_health': 40.0, 'minecraft:movement_speed': 0.1, 'minecraft:attack_damage': 7.0,
                     'minecraft:follow_range': 32.0, 'minecraft:step_height': 1.25, 'minecraft:attack_knockback': 0.75,
                     'minecraft:armor': 0.0},
    'pollinator': {'minecraft:max_health': 14.0, 'minecraft:movement_speed': 0.25, 'minecraft:follow_range': 16.0,
                   'minecraft:step_height': 1.0, 'minecraft:armor': 0.0},
}
ENTITY = {m: a for m, a in ENTITY.items() if m in MOBS}
for mob, attrs in ENTITY.items():
    for a in attrs:
        attrs[a] = float(attrs[a])
    attrs['minecraft:spawn_reinforcements'] = 0.0
    write(f'data/{NS}/horizon/entity_type/{mob}.json', {'template': 'minecraft:zombie', 'attributes': attrs})

# ------------------------------------------------------------------ files as they are
def copytree(src, dst, skip=lambda p: False):
    for p in sorted(src.rglob('*')):
        if p.is_file() and not skip(p.relative_to(src)):
            q = dst / p.relative_to(src)
            q.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(p, q)

A = JAR / 'assets' / NS
# Geckolib models and Fabric core shaders aren't a content mod's (the native mods read the models).
copytree(A, OUT / 'assets' / NS, skip=lambda r: r.parts[0] in ('geckolib', 'shaders'))

# The particles the portal opening draws, as Horizon's mod particles (a sprite set each, with
# `horizon:behaviour`): The Sift's note in each tone's colour (SonorousColors.BASE_RGB tints the
# grey note the mod draws, NOTE's friction and lifetime), and the singer's sound wave.
from PIL import Image
COLORS = [0xB02E26, 0x3C44AA, 0xF9801D, 0x8932B8, 0x169C9C, 0x80C71F, 0xFED83D, 0xF38BAA]
note = Image.open(A / 'textures' / 'particle' / 'sift_note.png').convert('RGBA')
for i, c in enumerate(COLORS, 1):
    rgb = ((c >> 16) & 255, (c >> 8) & 255, c & 255)
    px = [(r * rgb[0] // 255, g * rgb[1] // 255, b * rgb[2] // 255, a) for r, g, b, a in note.convert("RGBA").get_flattened_data()]
    img = Image.new('RGBA', note.size)
    img.putdata(px)
    q = OUT / 'assets' / NS / 'textures' / 'particle' / f'sift_note_{i}.png'
    img.save(q, optimize=False)
    write(f'assets/{NS}/particles/sift_note_{i}.json', {'textures': [f'{NS}:sift_note_{i}'],
          'horizon:behaviour': {'gravity': 0, 'friction': 0.66, 'lifetime': 6, 'size': [0.2, 0.2]}})
write(f'assets/{NS}/particles/singer_sound_wave.json', {'textures': [f'{NS}:sound_wave'],
      'horizon:behaviour': {'gravity': 0, 'friction': 0.6, 'lifetime': 10, 'size': [0.35, 0.35],
                            'alpha': [1, 0]}})
# A white sprite in the block atlas: the portal's pixel surface is drawn with it, tinted.
white = Image.new('RGBA', (16, 16), (255, 255, 255, 255))
white.save(OUT / 'assets' / NS / 'textures' / 'block' / 'sift_white.png', optimize=False)
D = JAR / 'data' / NS
keep = ('recipe', 'loot_table', 'advancement', 'tags', 'jukebox_song', 'damage_type', 'enchantment', 'painting_variant')
copytree(D, OUT / 'data' / NS, skip=lambda r: r.parts[0] not in keep or r.parts[:2] == ('tags', 'fluid'))
copytree(JAR / 'data' / 'minecraft' / 'tags', OUT / 'data' / 'minecraft' / 'tags')

# ModLootFunctions.EXPLORATION_MAP: The Sift's map to one of its structures is vanilla's exploration map to
# the structure's map tag (#the_sift:on_<structure>_maps), as the function finds it.
def maps(o):
    if isinstance(o, dict):
        if o.get('function') == 'the_sift:exploration_map':
            dest = o.get('destination', '')
            return {'function': 'minecraft:exploration_map', 'destination': f'the_sift:on_{dest}_maps',
                    'decoration': 'minecraft:target_x', 'zoom': 2, 'search_radius': 50, 'skip_existing_chunks': True}
        return {k: maps(v) for k, v in o.items()}
    if isinstance(o, list):
        return [maps(v) for v in o]
    return o
for p in (OUT / 'data' / NS / 'loot_table').rglob('*.json'):
    t = p.read_text()
    if 'the_sift:exploration_map' in t:
        p.write_text(json.dumps(maps(json.loads(t)), indent=2) + '\n')

# The Sift stands in the End's place: its arrival advancements trigger on entering the End.
for p in (OUT / 'data' / NS / 'advancement').rglob('*.json'):
    t = p.read_text()
    if '"to": "the_sift:the_sift"' in t:
        p.write_text(t.replace('"to": "the_sift:the_sift"', '"to": "minecraft:the_end"'))

# Blockstates: the crop's 4th age (beetroots') shows its last stage.
bs = OUT / 'assets' / NS / 'blockstates' / 'sculkflower_crop.json'
j = json.loads(bs.read_text())
j['variants']['age=3'] = j['variants']['age=2']
bs.write_text(json.dumps(j, indent=2) + '\n')

# ------------------------------------------------------------------ creative tab, manifest
lang = json.loads((A / 'lang' / 'en_us.json').read_text())
order = []
for k in lang:
    parts = k.split('.')
    if len(parts) == 3 and parts[0] == 'item' and parts[1] == NS and parts[2] not in order:
        order.append(parts[2])
declared = {n for n, d in B.items() if d.get('item', True)} | set(I)
tab = [f'{NS}:{n}' for n in order if n in declared] + \
      sorted(f'{NS}:{n}' for n in declared if n not in order)
write(f'data/{NS}/horizon/creative_tab/the_sift.json',
      {'title': 'itemGroup.the_sift', 'icon': f'{NS}:siftslate_growth', 'items': tab})

VERSION = json.loads((JAR / 'fabric.mod.json').read_text())['version']
(OUT / 'horizon-mod.toml').write_text(f'''id = "the_sift"
name = "Mielon's The Sift"
version = "{VERSION}"
description = "The Sift's blocks, items, recipes and sounds (Horizon port of Mielon's The Sift, MIT)."
authors = ["Mielon", "Horizon port: bartekchomik26-dotcom"]
api = "0.2"
side = "both"
capabilities = ["registries"]
''')
print(f'{len(B)} blocks, {len(I)} items, {len(tab)} in the tab')
