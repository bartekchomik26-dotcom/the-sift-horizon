# The Sift for Horizon

A port of **[Mielon's The Sift](https://modrinth.com/mod/mielons-the-sift) 2.0.1** to
[Horizon](https://github.com/horizon-rust), the Rust Minecraft client (26.3).

Horizon can't add new dimensions yet, so **The Sift takes the End's place**: go through an End portal,
or open the ancient city's Sift portal, and you arrive in The Sift.

> Unofficial port. All of The Sift's art, sounds, music and world data are Mielon's (MIT). The Rust code
> and tools here are new; none of Mielon's Java code is included.

## Download

Get both files from [Releases](../../releases):

| File | What it is |
|---|---|
| `the_sift-2.0.1.hmod` | Content mod: 121 blocks, 50 items, 12 entity types, recipes, loot, textures, sounds, music, languages |
| `the_sift_native-2.0.1.hnmod` | Native mod: The Sift in the End's place, terrain, ichor, sky, the portal opening, blocks and items with their own behaviour, mobs |

**Requires Horizon Nightly 2026-11-08 (`fdc87ae`).** The native mod is built for that revision exactly.

## Installing

1. Remove any older `the_sift_world` / `the_sift_mobs` `.hnmod` files.
2. Put both files in the launcher's mods and let it build the game.
3. Use a **new world**, or one whose End you haven't visited (End chunks already generated stay vanilla).
4. Horizon may show its "experimental settings" notice; that's the data pack that turns the End into The Sift.
   To keep a world's End vanilla, turn off the `file/the_sift_end` data pack for that world.

## Features

**The world**
- All eleven 2.0.1 biomes, the three cave biomes, and The Sift's own terrain features rewritten in Rust:
  willows from the mod's templates, monoliths, ichor lakes and springs, soul canyons, sniffer caves,
  abandoned portals with loot, sculk regions, ichor snow.
- Structures: coral camps, ichor domes and the sculkers' lair. They spawn about twice as often as in the
  jar (they were hard to find). Find one with `/locate structure the_sift:ichor_dome`.
- **Ichor** is The Sift's water: drawn with ichor's own textures spread over 16 blocks, gives
  Regeneration I and slows you. The ichor bucket places it.
- The Sift's sky (pink days, deep blue nights, its fog and weather), with or without a shader pack.
- **Each biome plays its own music** from the mod instead of the End's.
- **Beds work** in The Sift: no explosion, you can sleep at night and set your spawn.
- No dragon fight, exit portal or gateways.

**The portal opening** (as in the jar)
1. Find an ancient city. Its centre is The Sift's, with eight sonorous deepslate consoles.
2. Blow a goat horn near the frame. A singer rises, sings, and wakes the consoles.
3. Put note blocks on them. Right click tunes through The Sift's 8 tones, left click plays.
4. Strike 0 → 2 → 6 → 5 → 4 → 1 → 3 → 7: beams rise and the frame fills with a Sift portal.
   Strike it backwards to close it.
5. The first time you enter, The Sift's intro plays.

**Blocks and items**
- Every block and item of 2.0.1 drops correctly (not in Creative), including with Silk Touch.
- Willow saplings grow The Sift's seven willows; willow vines; blubby berry bushes; sculkflower crops;
  the ichor cauldron; growth spreading on siftslate and soulshale.
- Charoite in an enchanting table lifts offers by 20 levels (top one to 50).
- **Monarch elytra**, as in the jar: while gliding, press jump to rise up to 18 blocks while you hold it,
  then it eases off and you keep gliding; 10-second cooldown.

**Mobs**
- Blub (tameable with soul blocks), Echo Golem, Singer (barters for soul blocks), Stalker, Scavenger,
  Monarch, Nester, Seedling, Nuzzle, Slabber, Pollinator and the Dark Sniffer, with their models,
  animations, own sounds, attacks and biome spawning.

## Known gaps

Things Horizon can't do yet, or that aren't done:
- A real separate dimension (The Sift replaces the End). Proposal: `horizon-issue-custom-dimensions.md`.
- Ichor is water underneath (Horizon has only water and lava).
- Custom projectiles: the scavenger shoots arrows, the pollinator and monarch use small fireballs.
- The monarch's full boss fight (echoes, summons, boss bar).
- The sculkers' lair's sculk processor.
- Echo Golem missions and the singer's soul event beyond bartering.
- Multiplayer is untested; LAN guests don't hear the portal tones.
- Flowing ichor's texture spread; the Dark Sniffer's sniff animation on the client.
- Spawn eggs don't work on Peaceful.

**Tested in game:** the build compiles and runs in Horizon fdc87ae and the ichor dome generates.
The latest changes (elytra, beds, music, denser structures, the merged native mod) pass the mock tests
but haven't been tried in game yet. Bug reports welcome in [Issues](../../issues).

## Building from source

Needs Python 3, Rust, an unzipped The Sift 2.0.1 jar, and Horizon's source at `fdc87ae` with
[horizon-mixins](https://github.com/horizon-rust) for planning.

```sh
python3 tools/build.py path/to/unzipped-the-sift-2.0.1   # -> target/*.hmod, target/*.hnmod
cd harness && cargo test --release --workspace            # the mock tests
```

```text
content/                 the content mod (generated by tools/gen_content.py)
datapack/                the End data pack (tools/gen_datapack.py)
mixins/the_sift_native/  the native bundle (Rust): world, portal, items, ichor, sky, mobs
harness/                 mocks of the Horizon code the bundle touches, and the tests
tools/                   build, generators, preview renderers
previews/                pictures of blocks, mobs, terrain and the portal
```

## Credits

- **Mielon**, author of The Sift ([CurseForge](https://www.curseforge.com/minecraft/mc-mods/mielons-the-sift),
  [Modrinth](https://modrinth.com/mod/mielons-the-sift), [X](https://x.com/MielonDev)).
- From the original mod's credits: ImpSteve (Dark Sniffer texture), Fuzja Jądrowa ("Rift" disc),
  StrawberrySushi_ (Korean translation), Echo_Dev ("Wellspring" remix).
- The test harness includes Horizon's `environment.rs` (MIT, Ludovic Marie and the Horizon contributors).
- Horizon port by **bartekchomik26-dotcom**.

## License

MIT, see [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md). Both The Sift and this port are MIT.
