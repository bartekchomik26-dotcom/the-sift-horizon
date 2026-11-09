use super::the_sift as sift;
use super::*;
use horizon_client_render::datapack::{DataPack, the_sift_end as pack};
use horizon_client_render::gfx::native_render::the_sift_sky as sky;
use horizon_core::geometry::Vec3;
use std::sync::Mutex;

static SERIAL: Mutex<()> = Mutex::new(());
fn lock() -> std::sync::MutexGuard<'static, ()> { SERIAL.lock().unwrap_or_else(|e| e.into_inner()) }

static TYPE: &[u8] = crate::native_data!("the_sift_native", "sky/dimension_type.json");
static TIMELINE: &[u8] = crate::native_data!("the_sift_native", "sky/timeline.json");
static PACK: &[u8] = crate::native_data!("the_sift_native", "the_sift_end.zip");
static MAIN_PORTAL: &[u8] = crate::native_data!("the_sift_native", "templates/main_portal.txt");

/// A flat Sift: siftslate up to y 63.
fn flat(g: &mut Game<'_>, top: i32) {
    let stone = g.pack.assets.registry.default_state("the_sift:siftslate").unwrap();
    for x in 90..105 { for z in -6..7 { for y in 0..=top { g.world.blocks.insert([x, y, z], stone); } } }
}

#[test]
fn the_end_pack_joins_every_worlds_packs_once() {
    let _l = lock();
    let world = vec![DataPack::dir("file/zz", std::path::Path::new("/tmp")), DataPack::dir("file/aa", std::path::Path::new("/tmp"))];
    let packs = pack::with_pack(world, PACK);
    let ids: Vec<&str> = packs.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ids, ["file/aa", "file/the_sift_end", "file/zz"], "sorted, as discover sorts");
    let path = packs[1].zip_path().unwrap().to_owned();
    assert_eq!(std::fs::read(&path).unwrap(), PACK, "the zip written is the pack");
    // Asked again (every world, every reload): the same file, not a second copy.
    let again = pack::with_pack(Vec::new(), PACK);
    assert_eq!(again.len(), 1);
    assert_eq!(again[0].zip_path().unwrap(), path);
    // A world that has its own pack of that id keeps it.
    let own = pack::with_pack(vec![DataPack::dir(pack::PACK_ID, std::path::Path::new("/x"))], PACK);
    assert_eq!(own.len(), 1);
    assert!(own[0].zip_path().is_none());
}

#[test]
fn the_pack_holds_the_end_as_the_sift() {
    // The zip's own directory: the End's level stem, The Sift's type, its worldgen.
    let text = String::from_utf8_lossy(PACK);
    for name in ["pack.mcmeta", "data/minecraft/dimension/the_end.json", "data/the_sift/dimension_type/the_sift.json",
                 "data/the_sift/worldgen/noise_settings/the_sift.json", "data/the_sift/worldgen/biome/sift_wastes.json",
                 "data/minecraft/structure/ancient_city/city_center/city_center_1.nbt"] {
        assert!(text.contains(name), "{name}");
    }
}

#[test]
fn the_sifts_sky_follows_its_three_tides() {
    let env = pack::environment(TYPE, TIMELINE).expect("The Sift's sky reads with Horizon's environment.rs");
    let rgb = |hex: u32| [((hex >> 16) & 255) as f32 / 255.0, ((hex >> 8) & 255) as f32 / 255.0, (hex & 255) as f32 / 255.0];
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02);
    // sift_tides: the thrive tide (teal), the flow tide (pink), the endure tide (blue), 8400 ticks each.
    let thrive = env.sample(4000, None, None);
    let flow = env.sample(12_000, None, None);
    let endure = env.sample(21_000, None, None);
    assert!(near(thrive.sky_color, rgb(0x60d5c8)), "teal in the thrive tide: {:?}", thrive.sky_color);
    assert!(near(flow.sky_color, rgb(0xf7cfe0)), "pink in the flow tide: {:?}", flow.sky_color);
    assert!(near(endure.sky_color, rgb(0x176db5)), "blue in the endure tide: {:?}", endure.sky_color);
    // Darker in the endure tide: sky light 1, then 0.6.
    let d = horizon_client::environment::sky_darken(env, 12_000, 0.0, 0.0);
    let n = horizon_client::environment::sky_darken(env, 21_000, 0.0, 0.0);
    assert!(d < n, "{d} {n}");
}

#[test]
fn travellers_arrive_on_the_surface_beside_a_portal_home() {
    let _l = lock();
    let mut g = Game::new(true);
    flat(&mut g, 63);
    let (at, yaw) = sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), "").unwrap().unwrap();
    assert_eq!((at, yaw), (Vec3::new(100.5, 64.0, 0.5), Some(90.0)), "on top of the ground, facing west");
    // The frame: x 96, z -1..2, y 63..68; the portal inside, along z.
    for dz in -1..=2 {
        for y in 63..=68 {
            let edge = y == 63 || y == 68 || dz == -1 || dz == 2;
            let s = g.state_at([96, y, dz]);
            if edge {
                assert_eq!(s.name, "the_sift:reinforced_siftslate", "{dz} {y}");
            } else {
                assert_eq!((s.name.as_str(), s.properties["axis"].as_str()), ("the_sift:sift_portal", "z"), "{dz} {y}");
            }
        }
    }
    assert_eq!(g.portals_told, [("minecraft:the_end".to_owned(), [100, 63, 0])]);
    // Arriving again: the same portal, not another.
    let before = g.world.blocks.clone();
    sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), "").unwrap();
    assert_eq!(g.world.blocks, before);
}

#[test]
fn the_portal_home_never_floats() {
    let _l = lock();
    let mut g = Game::new(true);
    flat(&mut g, 63);
    // A drop west of the arrival: the frame's column is open down to y 40.
    for z in -6..7 { for y in 41..=63 { g.world.blocks.remove(&[96, y, z]); } }
    sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), "").unwrap();
    for dz in -1..=2 {
        for y in 41..63 {
            assert_eq!(g.name_at([96, y, dz]), "the_sift:siftslate", "filled under the frame at {dz} {y}");
        }
    }
}

#[test]
fn a_sift_portal_is_an_end_portal() {
    let _l = lock();
    let mut g = Game::new(true);
    flat(&mut g, 63);
    sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), "").unwrap();
    // A player walking into the portal (0.6 wide, 1.8 tall), and one beside it.
    assert!(sift::inside_sift_portal(&g, Vec3::new(96.5, 64.0, 0.5), 0.6, 1.8));
    assert!(!sift::inside_sift_portal(&g, Vec3::new(98.5, 64.0, 0.5), 0.6, 1.8));
    assert!(!sift::inside_sift_portal(&g, Vec3::new(96.5, 64.0, -2.5), 0.6, 1.8));
}

#[test]
fn only_a_sift_worlds_end_is_the_sift() {
    let _l = lock();
    let mut g = Game::new(true);
    assert!(sift::in_sift(&g));
    sift::refresh(&g);
    assert!(pack::active());
    g.dim = Dim::Overworld;
    assert!(!sift::in_sift(&g) && sift::sift_world(&g));
    sift::refresh(&g);
    assert!(!pack::active(), "the sky flag is for being in The Sift");
    let plain = Game::new(false);
    assert!(!sift::in_sift(&plain));
    let none = Game { world_biomes: None, ..Game::new(true) };
    assert!(!sift::in_sift(&none));
}

#[test]
fn the_sky_pass_gets_the_sifts_sky() {
    let _l = lock();
    let mut g = Game::new(true);
    sift::push_sky(&g, TYPE, TIMELINE);
    let (on, s) = sky::get();
    assert!(on);
    assert_eq!(s.sun_angle, 45.0);
    assert!(s.day > 0.999);
    g.world.day_time = 18_000;
    g.rain = 0.5;
    sift::push_sky(&g, TYPE, TIMELINE);
    let (_, s) = sky::get();
    assert_eq!((s.day, s.rain), (0.0, 0.5));
    g.dim = Dim::Nether;
    sift::push_sky(&g, TYPE, TIMELINE);
    assert!(!sky::get().0, "off out of The Sift");
    let mut depth = false;
    sky::needs(&mut depth);
    assert!(!depth);
}

#[test]
fn the_bodies_and_the_uniform() {
    let near = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5);
    assert!(near(sky::body_direction(0.0), [0.0, 1.0, 0.0]), "angle 0: overhead");
    let h = std::f32::consts::FRAC_1_SQRT_2;
    assert!(near(sky::body_direction(45.0), [-h, h, 0.0]), "45: halfway down to the west");
    assert!(near(sky::body_direction(225.0), [h, -h, 0.0]), "225: below the horizon");
    let s = sky::Sky { sky: [0.1, 0.2, 0.3], fog: [0.4, 0.5, 0.6], sun_angle: 45.0, moon_angle: 225.0, day: 1.0, rain: 0.25 };
    let mut m = [0f32; 16];
    m[5] = 2.0;
    let b = sky::uniform_bytes(m, &s, [1920, 1080], [960, 540]);
    let f = |i: usize| f32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
    assert_eq!(f(5), 2.0);
    assert_eq!((f(16), f(17), f(18), f(19)), (0.1, 0.2, 0.3, 1.0));
    assert_eq!((f(20), f(21), f(22)), (0.4, 0.5, 0.6));
    assert_eq!(f(27), 1.0);
    assert_eq!(f(31), 0.25);
    assert_eq!((f(32), f(33), f(34), f(35)), (1920.0, 1080.0, 960.0, 540.0));
}

#[test]
fn ichor_heals_and_holds_back_who_is_in_it() {
    let _l = lock();
    let mut g = Game::new(true);
    pack::set_active(false);
    // Entering The Sift: water turns to ichor, so every section is meshed again, once.
    sift::refresh_mut(&mut g);
    assert!(pack::active());
    assert_eq!(g.sections_changed, 1);
    sift::refresh_mut(&mut g);
    assert_eq!(g.sections_changed, 1, "not again while it stays The Sift");
    // Out of the water: nothing.
    sift::ichor(&mut g);
    assert!(g.potions.effects.active.is_empty());
    // In it: Regeneration I, 101 ticks, ambient, no particles; pulled back.
    g.player.as_mut().unwrap().fluids.was_touching_water = true;
    sift::ichor(&mut g);
    let e = &g.potions.effects.active[0];
    assert_eq!((e.id, e.duration, e.amplifier, e.ambient, e.visible), ("regeneration", 101, 0, true, false));
    let v = g.player.as_ref().unwrap().velocity;
    assert!((v.x - 0.2 * 0.92).abs() < 1e-9 && (v.y + 0.1 * 0.94).abs() < 1e-9 && (v.z - 0.2 * 0.92).abs() < 1e-9);
    // A server's world (remote) or another dimension: water is water.
    let mut o = Game::new(false);
    o.player.as_mut().unwrap().fluids.was_touching_water = true;
    sift::ichor(&mut o);
    assert!(o.potions.effects.active.is_empty());
    sift::refresh_mut(&mut o);
    assert!(!pack::active());
    pack::set_active(false);
}

#[test]
fn the_portal_home_is_the_sifts_main_portal() {
    let _l = lock();
    let mut g = Game::new(true);
    flat(&mut g, 63);
    let template = std::str::from_utf8(MAIN_PORTAL).unwrap();
    let (at, _) = sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), template).unwrap().unwrap();
    assert_eq!(at, Vec3::new(100.5, 64.0, 0.5));
    // Its portal: 20 wide (z -10..9) and 6 high (y 64..69) at x 96, on the z axis, framed.
    for z in -10..=9 {
        for y in 64..=69 {
            let s = g.state_at([96, y, z]);
            assert_eq!((s.name.as_str(), s.properties["axis"].as_str()), ("the_sift:sift_portal", "z"), "{z} {y}");
        }
    }
    let frame = g.world.blocks.keys().filter(|p| g.name_at(**p) == "the_sift:reinforced_siftslate").count();
    assert_eq!(frame, 55, "its reinforced siftslate frame");
    // The mound of The Sift's own blocks about it.
    let mound = g.world.blocks.keys().filter(|p| p[0] >= 93 && p[0] <= 101 && p[1] > 63).count();
    assert!(mound > 500, "{mound}");
    // A traveller walking west goes home through it.
    assert!(sift::inside_sift_portal(&g, Vec3::new(96.5, 64.0, 5.5), 0.6, 1.8));
    // Once.
    let before = g.world.blocks.clone();
    sift::arrival(&mut g, Vec3::new(100.5, 49.0, 0.5), template).unwrap();
    assert_eq!(g.world.blocks, before);
}

#[test]
fn arriving_waits_for_the_ground_to_load() {
    let mut g = Game::new(true);
    g.world.chunks.unloaded.insert((1, 0));
    let r = crate::game::the_sift::arrival(&mut g, horizon_core::geometry::Vec3::new(8.5, 49.0, 0.5), "");
    assert!(matches!(r, Ok(None)), "not yet: a chunk of the home is still loading");
}
