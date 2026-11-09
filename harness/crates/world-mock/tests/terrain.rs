//! The Sift's terrain features on a mock Sift: a noise terrain with The Sift's biomes, decorated
//! chunk by chunk through the shipped hook (h_decorate.rs → terrain.rs → features.rs), as Horizon
//! decorates (each chunk with its 3x3 neighbourhood, in an order of its own).
use horizon_core::position::ChunkPos;
use horizon_world::registry::{BlockRegistry, BlockStateId};
use horizon_world::worldgen::biome::BiomeId;
use horizon_world::worldgen::generator::{GeneratorData, ProtoChunk};
use horizon_world::worldgen::generator_decoration::{decorate_head, the_sift_features as f};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, OnceLock};

const B: &[&str] = &["false", "true"];
include!("sift_blocks.rs");

fn registry() -> &'static BlockRegistry {
    static R: OnceLock<BlockRegistry> = OnceLock::new();
    R.get_or_init(|| {
        let levels: Vec<&str> = (0..16).map(|i| &*Box::leak(i.to_string().into_boxed_str())).collect();
        let dist: Vec<&str> = ["7", "1", "2", "3", "4", "5", "6"].to_vec();
        let vine = || vec![("bottom", B.to_vec()), ("east", B.to_vec()), ("east_bottom", B.to_vec()),
            ("north", B.to_vec()), ("north_bottom", B.to_vec()), ("south", B.to_vec()),
            ("south_bottom", B.to_vec()), ("up", B.to_vec()), ("west", B.to_vec()), ("west_bottom", B.to_vec())];
        let plain = |n: &'static str| (n, vec![]);
        let wl = || ("waterlogged", B.to_vec());
        let mut all: Vec<(&str, Vec<(&str, Vec<&str>)>)> = SIFT_BLOCKS
            .iter()
            .map(|(n, ps)| (*n, ps.iter().map(|(k, v)| (*k, v.to_vec())).collect()))
            .collect();
        all.extend([
            plain("minecraft:air"), plain("minecraft:cave_air"), plain("minecraft:void_air"),
            ("minecraft:water", vec![("level", levels.clone())]), ("minecraft:lava", vec![("level", levels.clone())]),
            plain("minecraft:bedrock"), plain("minecraft:stone"),
            ("minecraft:chest", vec![("facing", vec!["north", "east", "south", "west"]), ("type", vec!["single", "left", "right"]), wl()]),
            plain("minecraft:sculk"),
            ("minecraft:sculk_vein", vec![("down", B.to_vec()), ("east", B.to_vec()), ("north", B.to_vec()), ("south", B.to_vec()), ("up", B.to_vec()), wl(), ("west", B.to_vec())]),
            ("minecraft:sculk_sensor", vec![("power", levels.clone()), ("sculk_sensor_phase", vec!["inactive", "active", "cooldown"]), wl()]),
            ("minecraft:sculk_shrieker", vec![("can_summon", B.to_vec()), ("shrieking", B.to_vec()), wl()]),
            ("minecraft:sculk_catalyst", vec![("bloom", B.to_vec())]),
            plain("minecraft:torchflower"),
            ("minecraft:pitcher_plant", vec![("half", vec!["lower", "upper"])]),
            ("minecraft:sniffer_egg", vec![("hatch", vec!["0", "1", "2"])]),
            ("minecraft:wildflowers", vec![("facing", vec!["north", "east", "south", "west"]), ("flower_amount", vec!["1", "2", "3", "4"])]),
            ("minecraft:barrel", vec![("facing", vec!["north", "east", "south", "west", "up", "down"]), ("open", B.to_vec())]),
            ("minecraft:white_candle", vec![("candles", vec!["1", "2", "3", "4"]), ("lit", B.to_vec()), wl()]),
            ("minecraft:black_candle", vec![("candles", vec!["1", "2", "3", "4"]), ("lit", B.to_vec()), wl()]),
            ("minecraft:white_wall_banner", vec![("facing", vec!["north", "east", "south", "west"])]),
            ("minecraft:smoker", vec![("facing", vec!["north", "east", "south", "west"]), ("lit", B.to_vec())]),
            ("minecraft:furnace", vec![("facing", vec!["north", "east", "south", "west"]), ("lit", B.to_vec())]),
            ("minecraft:blast_furnace", vec![("facing", vec!["north", "east", "south", "west"]), ("lit", B.to_vec())]),
            plain("minecraft:smithing_table"), plain("minecraft:crafting_table"),
            ("minecraft:skeleton_skull", vec![("powered", B.to_vec()), ("rotation", levels.clone())]),
            ("minecraft:bone_block", vec![("axis", vec!["y", "x", "z"])]),
        ]);
        BlockRegistry::new(&all)
    })
}

const BIOMES: &[&str] = &[
    "the_sift:sift_wastes", "the_sift:overgrown_clearing", "the_sift:overgrown_forest", "the_sift:coral_forest",
    "the_sift:coral_meadow", "the_sift:corrupted_sift", "the_sift:ichor_snowy_peaks", "the_sift:organic_desert",
    "the_sift:overgrown_caves", "the_sift:siftslate_caves", "the_sift:soulshale_caves", "minecraft:plains",
];

fn id(name: &str) -> BlockStateId {
    registry().default_state(name).unwrap()
}

/// A Sift: rolling noise hills (peaks above 120), the biomes in bands, a cave layer.
struct Gen {
    data: Arc<GeneratorData>,
    seed: u64,
}
impl Gen {
    fn height(&self, x: i32, z: i32) -> i32 {
        let n = |s: f32| f::noise2(self.seed, x as f32 / s, z as f32 / s);
        (70.0 + 52.0 * n(90.0).powf(1.6) + 12.0 * n(23.0)) as i32
    }
    fn biome(&self, x: i32, z: i32, h: i32) -> &'static str {
        let b = f::noise2(self.seed ^ 77, x as f32 / 70.0, z as f32 / 70.0);
        if h > 122 {
            return "the_sift:ichor_snowy_peaks";
        }
        match (b * 8.0) as i32 {
            0 => "the_sift:sift_wastes",
            1 => "the_sift:overgrown_clearing",
            2 | 3 => "the_sift:overgrown_forest",
            4 => "the_sift:coral_forest",
            5 => "the_sift:coral_meadow",
            6 => "the_sift:organic_desert",
            _ => "the_sift:corrupted_sift",
        }
    }
    fn chunk(&self, cx: i32, cz: i32, plains: bool) -> ProtoChunk {
        ProtoChunk::new(self.data.clone(), ChunkPos::new(cx, cz), 0, 256, |x, z, col| {
            let h = self.height(x, z);
            let biome = if plains { "minecraft:plains" } else { self.biome(x, z, h) };
            let wastes = matches!(biome, "the_sift:sift_wastes" | "the_sift:organic_desert" | "the_sift:corrupted_sift");
            for y in 0..h {
                col[y as usize] = if y == 0 {
                    id("minecraft:bedrock")
                } else if y == h - 1 && !plains {
                    if wastes { id("the_sift:dry_healthy_sculk") } else { id("the_sift:siftslate_growth") }
                } else if plains {
                    id("minecraft:stone")
                } else {
                    id("the_sift:siftslate")
                };
            }
            // A cave layer, y 28 to 36.
            if f::noise2(self.seed ^ 5, x as f32 / 14.0, z as f32 / 14.0) > 0.62 {
                for y in 28..36 {
                    col[y as usize] = id("minecraft:air");
                }
            }
            if f::noise2(self.seed ^ 9, x as f32 / 9.0, z as f32 / 9.0) > 0.7 {
                for y in 3..9 {
                    col[y as usize] = id("minecraft:air");
                }
            }
            self.data.biome_id(biome).unwrap_or(BiomeId(0))
        })
    }
}

/// `n` x `n` chunks, every one but the rim decorated (in a scattered order).
fn world(seed: i64, n: i32, plains: bool) -> (HashMap<(i32, i32), ProtoChunk>, std::time::Duration, usize) {
    let data = Arc::new(GeneratorData::new(registry(), BIOMES));
    let g = Gen { data, seed: seed as u64 };
    let mut chunks: HashMap<(i32, i32), ProtoChunk> = HashMap::new();
    for cx in 0..n {
        for cz in 0..n {
            chunks.insert((cx, cz), g.chunk(cx, cz, plains));
        }
    }
    let mut order: Vec<(i32, i32)> = (1..n - 1).flat_map(|x| (1..n - 1).map(move |z| (x, z))).collect();
    order.sort_by_key(|&(x, z)| (x * 7919 + z * 104729) % 97);
    let mut spent = std::time::Duration::ZERO;
    for &(cx, cz) in &order {
        let mut region: [ProtoChunk; 9] = std::array::from_fn(|i| {
            chunks.remove(&(cx + i as i32 % 3 - 1, cz + i as i32 / 3 - 1)).unwrap()
        });
        let t = std::time::Instant::now();
        decorate_head(&g.data, seed, &mut region);
        spent += t.elapsed();
        region[4].set_decorated();
        for (i, c) in region.into_iter().enumerate() {
            chunks.insert((cx + i as i32 % 3 - 1, cz + i as i32 / 3 - 1), c);
        }
    }
    (chunks, spent, order.len())
}

fn census(chunks: &HashMap<(i32, i32), ProtoChunk>) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for c in chunks.values() {
        let p = c.position();
        for y in 0..256 {
            for z in 0..16 {
                for x in 0..16 {
                    *out.entry(c.block_name(p.x * 16 + x, y, p.z * 16 + z).to_owned()).or_default() += 1;
                }
            }
        }
    }
    out
}

#[test]
fn every_feature_state_resolves() {
    // The features' palette, and every template state turned every way, are real states.
    let templates = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../../native-data/the_sift_native/templates")).unwrap();
    let texts: Vec<(String, String)> = templates
        .map(|e| e.unwrap().path())
        .map(|p| (p.file_stem().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()))
        .collect();
    let refs: Vec<(&str, &str)> = texts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let mut sift = f::Sift::new(&refs);
    assert_eq!(sift.templates.len(), 148);
    let mut p = f::Palette::new();
    sift.bake(&mut p);
    let turned = sift.turned("overgrown_willow_1", f::Turn { quarter: 3, mirror: true }).unwrap();
    assert_eq!(turned.len(), sift.templates["overgrown_willow_1"].blocks.len());
    let mut bad = Vec::new();
    for s in &p.names {
        if s == "?" || s.starts_with("minecraft:jigsaw") {
            continue;
        }
        let (name, props) = f::parse_state(s);
        let mut properties = horizon_world::block_state_properties::BlockProperties::new();
        for (k, v) in props {
            properties.insert(k.to_owned(), v.to_owned());
        }
        if registry().resolve(&horizon_world::chunk::BlockState { name: name.to_owned(), properties }).is_err() {
            bad.push(s.clone());
        }
    }
    assert!(bad.is_empty(), "unresolved: {bad:?}");
}

#[test]
fn templates_turn_and_vines_keep_their_bottoms() {
    let t = f::Turn { quarter: 1, mirror: false };
    assert_eq!(t.pos([1, 0, 0]), [0, 0, 1]);
    assert_eq!(t.state("the_sift:overgrown_willow_log[axis=x]"), "the_sift:overgrown_willow_log[axis=z]");
    assert_eq!(t.state("minecraft:chest[facing=north,type=single,waterlogged=false]"),
        "minecraft:chest[facing=east,type=single,waterlogged=false]");
    let v = f::vine_state(&t.state("the_sift:overgrown_willow_vines[bottom=true,east=false,north=true,south=false,up=false,west=false]"));
    assert!(v.contains("east=true") && v.contains("east_bottom=true") && v.contains("north_bottom=false"), "{v}");
    let m = f::Turn { quarter: 0, mirror: true };
    assert_eq!(m.state("minecraft:chest[facing=east]"), "minecraft:chest[facing=west]");
    // Four quarter turns are none.
    for q in 0..4u8 {
        let p = [3, 5, -2];
        let once = f::Turn { quarter: q, mirror: false }.pos(p);
        let back = f::Turn { quarter: (4 - q) % 4, mirror: false }.pos(once);
        assert_eq!(back, p);
    }
}

#[test]
fn the_sift_is_decorated_with_its_features() {
    let (chunks, spent, n) = world(20_26, 14, false);
    let c = census(&chunks);
    let get = |k: &str| c.get(k).copied().unwrap_or(0);
    let has = |prefix: &str| c.iter().filter(|(k, _)| k.starts_with(prefix)).map(|(_, v)| v).sum::<usize>();
    eprintln!("{n} chunks decorated in {spent:?} ({:?} a chunk)", spent / n as u32);
    for (k, v) in &c {
        eprintln!("  {k}: {v}");
    }
    assert!(has("the_sift:overgrown_willow_log") > 200, "willows");
    assert!(has("the_sift:overgrown_willow_foliage") > 500, "foliage");
    assert!(has("the_sift:overgrown_willow_vines") > 50, "vines");
    assert!(get("minecraft:water") > 200, "ichor lakes");
    assert!(get("the_sift:dry_healthy_sculk") > 0 && get("minecraft:sculk") > 50, "sculk");
    assert!(get("the_sift:whisperbloom") + get("the_sift:sunburst_plant") > 5, "flowers");
    assert!(get("the_sift:overgrown_chard") + get("the_sift:overgrown_stalks") + get("the_sift:overgrown_fronds") > 20, "plants");
    assert!(get("minecraft:lava") > 0, "lava floor");
    assert!(get("the_sift:ichor_snow") > 0, "ichor snow");
}

#[test]
fn decoration_is_deterministic_and_order_free() {
    let (a, _, _) = world(7, 9, false);
    let (b, _, _) = world(7, 9, false);
    assert_eq!(census(&a), census(&b));
    for (k, ca) in &a {
        let cb = &b[k];
        for y in (0..256).step_by(3) {
            for z in 0..16 {
                for x in 0..16 {
                    let p = ca.position();
                    assert_eq!(ca.block(p.x * 16 + x, y, p.z * 16 + z), cb.block(p.x * 16 + x, y, p.z * 16 + z));
                }
            }
        }
    }
    let (c, _, _) = world(8, 9, false);
    assert_ne!(census(&a), census(&c), "another seed, another Sift");
}

#[test]
fn other_worlds_are_left_alone() {
    let (chunks, _, _) = world(3, 6, true);
    assert!(chunks.values().all(|c| c.terrain.writes == 0 && c.block_entities.is_empty()));
}

#[test]
fn decorated_chunks_are_skipped() {
    let data = Arc::new(GeneratorData::new(registry(), BIOMES));
    let g = Gen { data: data.clone(), seed: 11 };
    let mut region: [ProtoChunk; 9] = std::array::from_fn(|i| g.chunk(i as i32 % 3 - 1, i as i32 / 3 - 1, false));
    region[4].set_decorated();
    decorate_head(&data, 11, &mut region);
    assert!(region.iter().all(|c| c.terrain.writes == 0));
    // A region that isn't a 3x3 neighbourhood is left alone too.
    let mut odd: [ProtoChunk; 9] = std::array::from_fn(|i| g.chunk(i as i32, 0, false));
    decorate_head(&data, 11, &mut odd);
    assert!(odd.iter().all(|c| c.terrain.writes == 0));
}

#[test]
fn springs_flow_and_portal_chests_hold_loot() {
    let (chunks, _, _) = world(20_26, 14, false);
    let ticks: usize = chunks.values().map(|c| c.terrain.fluid_ticks().count()).sum();
    assert!(ticks > 0, "springs schedule their fluid ticks");
    for c in chunks.values() {
        for (p, nbt) in &c.block_entities {
            assert_eq!((p[0] >> 4, p[2] >> 4), (c.position().x, c.position().z), "a block entity in its own chunk");
            assert!(c.block_name(p[0], p[1], p[2]).starts_with("minecraft:chest"));
            assert!(nbt.contains_key(&"LootTable".into()));
        }
    }
}

/// The decorated world as a voxel file for tools/render_terrain.py (and the bench's timing).
#[test]
#[ignore]
fn export_preview() {
    let n = 18;
    let seed: i64 = std::env::var("SIFT_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(8);
    let (chunks, spent, count) = world(seed, n, false);
    eprintln!("decorated {count} chunks in {spent:?}: {:?} a chunk", spent / count as u32);
    let mut names: Vec<String> = Vec::new();
    let mut index: HashMap<String, u16> = HashMap::new();
    let (lo, hi) = (16, (n - 1) * 16);
    let size = hi - lo;
    let mut out = Vec::new();
    for y in 0..256 {
        for z in lo..hi {
            for x in lo..hi {
                let c = &chunks[&(x >> 4, z >> 4)];
                let full = registry().state(c.block(x, y, z)).unwrap();
                let mut s = full.name.clone();
                if let Some(v) = full.properties.get("axis") { s += &format!("[axis={v}]"); }
                if let Some(v) = full.properties.get("half") { s += &format!("[half={v}]"); }
                let k = *index.entry(s.clone()).or_insert_with(|| { names.push(s); (names.len() - 1) as u16 });
                out.extend_from_slice(&k.to_le_bytes());
            }
        }
    }
    let dir = std::env::var("SIFT_OUT").unwrap_or("/tmp".into());
    std::fs::write(format!("{dir}/sift_terrain.bin"), &out).unwrap();
    std::fs::write(format!("{dir}/sift_terrain.json"), format!("{{\"size\": [{size}, 256, {size}], \"names\": {:?}}}", names)).unwrap();
}

#[test]
fn landmarks_are_placed_whole() {
    // A wide Sift: its sniffer caves (with their nests) and abandoned portals (with loot).
    let (chunks, _, _) = world(1, 34, false);
    let c = census(&chunks);
    eprintln!("eggs {:?} reinforced {:?} chests {:?} soul {:?}", c.get("minecraft:sniffer_egg"),
        c.get("the_sift:reinforced_siftslate"), c.get("minecraft:chest"), c.get("the_sift:soul_block"));
    assert!(c.get("minecraft:sniffer_egg").copied().unwrap_or(0) > 0, "sniffer nests");
    assert!(c.get("minecraft:chest").copied().unwrap_or(0) > 0, "abandoned portals");
    assert!(c.get("the_sift:soul_block").copied().unwrap_or(0) > 0, "soul canyons");
}
