# Notice

This is a Horizon port of **Mielon's The Sift** 2.0.1 (built for Minecraft 26.2, Fabric; ported to Horizon's 26.3), by
Mielon, released under the MIT licence (`LICENSE_the_sift` in the jar; <https://modrinth.com/mod/mielons-the-sift>,
<https://www.curseforge.com/minecraft/mc-mods/mielons-the-sift>, <https://x.com/MielonDev>). Earlier milestones ported
1.1.1, also MIT.

The Sift's own files are used as they are: its textures, models, blockstates, sounds, music, languages, recipes,
loot tables, tags, advancements and worldgen, its structure templates (read into text in
`mixins/the_sift_native/data/templates/`), its three ancient city centres with their sonorous consoles
(`mixins/the_sift_native/data/structure/`, as they are) and its mobs' GeckoLib geometry and animations
(`mixins/the_sift_native/data/geckolib/`). No GeckoLib code is used: the port reads those files itself. The Fabric mod credits these contributors, and so does this port:

- ImpSteve: the Dark Sniffer texture
- Fuzja Jądrowa: the "Rift" music disc
- StrawberrySushi_: the Korean translation
- Frumptle: the Russian translation
- Grelinho: the Brazilian Portuguese translation
- Echo_Dev: the "Wellspring" remix
- Lealeanz: the ambient music

The port's tinted note particles (`assets/the_sift/textures/particle/sift_note_1..8.png`) are The Sift's own note
texture in its eight tone colours, and `assets/the_sift/textures/block/sift_white.png` is a plain white sprite.

What's new in the port (block and item declarations, the End data pack's adjustments, the native Rust code, the
tools and tests) is MIT too, Copyright (c) 2026 bartekchomik26-dotcom. Mielon's Java code isn't included: the
port's behaviour is written fresh for Horizon, from what the mod's classes do (read with `javap`).

The test harness's `harness/render-mock/src/environment_real.rs` is a copy of Horizon's own
`crates/horizon-client-render/src/environment.rs` (MIT, Copyright (c) 2026 Ludovic Marie and the Horizon
contributors), so the tests read The Sift's sky with the game's real code.

`tools/hzupgrade/src/upgrade/` is a copy of Horizon's `crates/horizon-port-pack/src/upgrade/` (MIT, Copyright (c)
2026 Ludovic Marie and the Horizon contributors; only its zip reader is replaced by a folder reader). The build runs
it over the generated data pack so the pack is in 26.3's form, exactly as Horizon's own `upgrade-datapack` would.
