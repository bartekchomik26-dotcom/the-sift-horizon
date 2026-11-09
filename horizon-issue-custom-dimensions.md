**Title:** Content mods and data packs: custom dimensions and new biomes (`data/<ns>/dimension/`, `dimension_type/`, `worldgen/biome/`)

### What I'm trying to do

Port [Mielon's The Sift](https://www.curseforge.com/minecraft/mc-mods/mielons-the-sift) (MIT, Fabric, 26.3) to Horizon.
The mod is built around a dimension of its own. It ships it as data:

- `data/the_sift/dimension/the_sift.json`: a `minecraft:noise` generator with a `multi_noise` biome source.
- `data/the_sift/dimension_type/the_sift.json`.
- 10 new biomes: `the_sift:sift_wastes`, `overgrown_forest`, `ichor_snowy_peaks` and others.
- Its own noise settings, 15 noises, 21 density functions, 3 carvers, 46 configured and 59 placed features, and
  structures.

Its blocks, items, recipes, loot and tags already fit a 0.2 content mod. Its mobs fit a native mod with
`native_mob_submit`. The dimension is what's missing, and most of the mod lives there.

### What Horizon does today (f319701)

- `horizon_world::level::Dimension` is a closed enum, `Overworld | Nether | End`. About 70 matches across 21 files,
  plus about 90 `"minecraft:the_nether"`/`"minecraft:the_end"` string literals, assume those three.
- `datapack::dimension_stems` reads `data/<ns>/dimension/*.json`, but only `save::world_gen_unstable` uses it, for
  the experimental-settings warning. No level is created for a pack's or a mod's dimension.
- `datapack::worldgen_overrides` leaves a pack's new biome out: "the game has no colours, sounds or id for it".
- `docs/modpacks.md` lists "Dimensions: dimension types, levels, chunk generators" for 0.5. In the meantime there is
  no hook a native mod could use to add one (the mixin injection kinds can't add a variant to `Dimension`).

### What would make it possible

1. **Data-driven levels.** Create a server level for every `dimension/` stem in the enabled data packs and content
   mods, as vanilla's `WorldDimensions` does:
   - Save it under `dimensions/<ns>/<path>/`, which `save::dimension_dir` already lays out.
   - Park it and swap it in like the three built-in levels.
   - Send it to clients by its registry id.
   - Make `Dimension` an id-backed type, or add a `Custom(id)` variant, with the three built-in levels as constants.
2. **Dimension types.** Honour `data/<ns>/dimension_type/*.json`: height and `min_y`, `has_skylight`,
   `has_ceiling`, `ambient_light`, `effects`/sky, `fixed_time`, `bed_works`, `respawn_anchor_works`, `piglin_safe`,
   `natural`, `coordinate_scale` and the infiniburn tag.
3. **New biomes.** Give them registry ids, network ids and their `effects` (sky, fog, water, grass and foliage
   colours, particles, ambient sounds, music) from the biome JSON, instead of leaving them out.
4. **Travel.** A server helper to send an entity or player to another dimension by id with a position (vanilla's
   `Entity.teleport(TeleportTransition)`), so a mod's own portal block can use it, and a hook when a player changes
   dimension.
5. **Mod worldgen features.** A hook to register feature types in code (vanilla's `Registries.FEATURE`): a placed
   feature's `type: "mymod:ichor_lake"` calls the mod's `place(context)` with the level, origin, random and config
   JSON. Most dimension mods (The Sift, Twilight Forest, Aether, Undergarden) place their own features this way.

Even 1 to 3 alone would let a dimension's terrain, biomes and vanilla features generate from data. A mod could then
handle 4 and 5 through native hooks.

### Why it matters

Dimension mods are among the most played Fabric and NeoForge mods: The Sift, Twilight Forest, the Aether,
Undergarden, Ad Astra, Deeper and Darker and others. Right now none of them can be ported, even when their licence
allows it, because their main content has nowhere to go.

### Environment

Horizon Nightly 2026-11-05 (f319701), single player.
