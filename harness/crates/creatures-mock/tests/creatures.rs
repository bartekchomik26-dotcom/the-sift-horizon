//! creatures.rs as it ships, on a mock game: The Sift's mobs drawn from their GeckoLib models.
use horizon_client::game::Game;
use horizon_client::game::native_block_entity_draw::BlockEntityDraw;
use horizon_client::game::native_mob_draw::NativeMobRender;
use horizon_client::game::the_sift_creatures as c;
use horizon_client::items::ItemStack;
use horizon_client::mobs::Mob;

macro_rules! data { ($f:literal) => { include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../mixins/the_sift_native/data/geckolib/", $f)) }; }
static FILES: &[(&str, &[u8], &[u8])] = &[
    ("blub", data!("blub.geo.json"), data!("blub.animation.json")),
    ("stalker", data!("stalker.geo.json"), data!("stalker.animation.json")),
    ("nuzzle", data!("nuzzle.geo.json"), data!("nuzzle.animation.json")),
    ("slabber", data!("slabber.geo.json"), data!("slabber.animation.json")),
    ("nester", data!("nester.geo.json"), data!("nester.animation.json")),
    ("pollinator", data!("pollinator.geo.json"), data!("pollinator.animation.json")),
    ("seedling", data!("seedling.geo.json"), data!("seedling.animation.json")),
    ("scavenger", data!("scavenger.geo.json"), data!("scavenger.animation.json")),
    ("monarch", data!("monarch.geo.json"), data!("monarch.animation.json")),
    ("dark_sniffer", data!("dark_sniffer.geo.json"), data!("dark_sniffer.animation.json")),
    ("echo_golem", data!("echo_golem.geo.json"), data!("echo_golem.animation.json")),
    ("singer", data!("singer.geo.json"), data!("singer.animation.json")),
];

fn render<'a>(id: u32, kind: &'a str) -> NativeMobRender<'a> {
    NativeMobRender { id, kind, partial: 0.0, age: 10.0, walk_speed: 0.0, hurt_time: 0, death_time: 0, on_ground: true }
}

fn drawn(g: &Game<'_>, r: &NativeMobRender<'_>) -> (BlockEntityDraw, bool) {
    let mut d = BlockEntityDraw { light: 0x50, quads: Vec::new() };
    let mut done = false;
    c::draw(g, c::kit(FILES), r, &mut d, &mut done);
    (d, done)
}

fn bounds(d: &BlockEntityDraw) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for (_, k, ..) in &d.quads {
        for p in k {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
    }
    (lo, hi)
}

#[test]
fn every_mob_is_drawn_its_size_facing_forward() {
    let mut g = Game::new();
    let all = [(0, "the_sift:blub", 0.665, 0.875), (1, "the_sift:stalker", 0.91, 2.375), (2, "the_sift:echo_golem", 1.15, 1.65),
        (3, "the_sift:singer", 0.96, 4.8), (4, "the_sift:nuzzle", 0.9, 1.25), (5, "the_sift:slabber", 1.5, 1.6875),
        (6, "the_sift:nester", 0.95, 2.25), (7, "the_sift:pollinator", 0.675, 1.4375), (8, "the_sift:seedling", 0.72, 0.76),
        (9, "the_sift:scavenger", 1.19, 1.275), (10, "the_sift:monarch", 1.45, 4.8125), (11, "the_sift:dark_sniffer", 1.9, 1.75)];
    for (i, k, ..) in all {
        g.mobs.push(Mob::new(i, k));
    }
    // Sizes: about their hitboxes (ModEntities' sizes; the singer's times its 1.2 scale).
    for (i, kind, w, h) in all {
        let (d, done) = drawn(&g, &render(i, kind));
        assert!(done, "{kind} drawn by the mod");
        let (lo, hi) = bounds(&d);
        eprintln!("{kind}: {} quads, x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}", d.quads.len(), lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]);
        assert!(lo[1] > -0.1 && lo[1] < 0.2, "{kind} stands on its feet");
        let (width, height) = (hi[0] - lo[0], hi[1] - lo[1]);
        assert!(width > w * 0.4 && width < w * 4.0, "{kind} width {width}");
        assert!(height > h * 0.4 && height < h * 1.8, "{kind} height {height}");
    }
    // The blub's face (its head's north side in the model, the cube's widest north face) is at its
    // front, +z: the model's north is turned to face the way the mob does.
    let (d, _) = drawn(&g, &render(0, "the_sift:blub"));
    let (lo, hi) = bounds(&d);
    let flat = |q: &&(String, [[f32; 3]; 4], [f32; 4], u32, u32)| q.1.iter().all(|p| (p[2] - q.1[0][2]).abs() < 1e-4);
    let front = d.quads.iter().filter(flat).map(|q| q.1[0][2]).fold(f32::MIN, f32::max);
    assert!(front > hi[2] - 0.15 && hi[2] > -lo[2] - 0.2, "the face looks the way the mob faces: {front} of {}", hi[2]);
    // Its texture: a tamed blub's own.
    g.mobs[0].extra.tame = true;
    let (d, _) = drawn(&g, &render(0, "the_sift:blub"));
    assert_eq!(d.quads[0].0, "the_sift:entity/blub/tamed_blub");
    // Not The Sift's: left to vanilla.
    let (d, done) = drawn(&g, &render(9, "minecraft:zombie"));
    assert!(!done && d.quads.is_empty());
}

#[test]
fn animations_move_the_models() {
    let mut g = Game::new();
    g.mobs.push(Mob::new(1, "the_sift:stalker"));
    g.mobs.push(Mob::new(2, "the_sift:echo_golem"));
    let still = drawn(&g, &render(1, "the_sift:stalker")).0;
    let mut run = render(1, "the_sift:stalker");
    run.walk_speed = 0.8;
    run.age = 27.0;
    let running = drawn(&g, &run).0;
    assert_eq!(still.quads.len(), running.quads.len());
    assert!(still.quads.iter().zip(&running.quads).any(|(a, b)| a.1 != b.1), "the stalker's legs move when it walks");
    // The echo golem's soul shows only while it carries one, glowing.
    let empty = drawn(&g, &render(2, "the_sift:echo_golem")).0;
    g.mobs[1].extra.main_hand = Some("the_sift:soul_block".into());
    let carrying = drawn(&g, &render(2, "the_sift:echo_golem")).0;
    assert!(carrying.quads.len() > empty.quads.len(), "{} > {}", carrying.quads.len(), empty.quads.len());
    // (2.0 draws the golem without its 1.1 emissive layer.)
    assert!(carrying.quads.iter().filter(|q| q.0 == "the_sift:entity/echo_golem/echo_golem").all(|q| q.4 == 0x50), "the rest lit as the mob is");
    // Hurt: tinted red; dying: tipped over.
    let mut hurt = render(2, "the_sift:echo_golem");
    hurt.hurt_time = 5;
    assert!(drawn(&g, &hurt).0.quads.iter().all(|q| q.0.ends_with("emissive") || q.3 == 0xFF_9999));
    let mut dead = render(2, "the_sift:echo_golem");
    dead.death_time = 20;
    let (lo, hi) = bounds(&drawn(&g, &dead).0);
    let (alo, ahi) = bounds(&drawn(&g, &render(2, "the_sift:echo_golem")).0);
    assert!(hi[1] - lo[1] < (ahi[1] - alo[1]) * 0.85 && hi[0] - lo[0] > ahi[0] - alo[0], "lying down");
}

#[test]
fn blubs_are_tamed_with_soul_blocks_and_singers_barter_for_them() {
    let mut g = Game::new();
    g.mobs.push(Mob::new(5, "the_sift:blub"));
    g.mobs.push(Mob::new(6, "the_sift:singer"));
    let soul = || Some(ItemStack { item: "the_sift:soul_block".into(), count: 64 });
    g.held = soul();
    let mut tamed = false;
    for _ in 0..20 {
        let stack = g.held.clone();
        let mut claimed = false;
        c::interact(&mut g, 5, "the_sift:blub", stack.as_ref(), &mut claimed);
        if g.mobs[0].extra.tame {
            tamed = true;
            break;
        }
        assert!(claimed);
    }
    assert!(tamed, "a blub takes to a soul block sooner or later");
    assert_eq!(g.mobs[0].extra.owner, Some([1, 2, 3, 4]));
    assert!(g.mobs[0].extra.sitting && g.mobs[0].persistent);
    assert!(g.particles.iter().any(|p| p.1 == 7));
    assert!(g.held.as_ref().unwrap().count < 64, "the soul blocks were used");
    // Its owner, empty-handed, tells it to stand.
    g.held = None;
    let mut claimed = false;
    c::interact(&mut g, 5, "the_sift:blub", None, &mut claimed);
    assert!(claimed && !g.mobs[0].extra.sitting);
    // A singer: a soul for a reward.
    g.held = soul();
    let stack = g.held.clone();
    let mut claimed = false;
    c::interact(&mut g, 6, "the_sift:singer", stack.as_ref(), &mut claimed);
    assert!(claimed);
    assert!(g.events.iter().any(|e| matches!(e, horizon_client::mobs::MobEvent::DropItem { .. })));
    assert_eq!(g.held.as_ref().unwrap().count, 63);
    // Creative: nothing used up.
    g.creative = true;
    let stack = g.held.clone();
    c::interact(&mut g, 6, "the_sift:singer", stack.as_ref(), &mut claimed);
    assert_eq!(g.held.as_ref().unwrap().count, 63);
    // Another mod's mob: not claimed.
    let mut claimed = false;
    c::interact(&mut g, 6, "minecraft:cow", stack.as_ref(), &mut claimed);
    assert!(!claimed);
}

#[test]
fn creatures_spawn_about_the_player_in_the_sift() {
    let mut g = Game::new();
    for t in 0..4000 {
        g.world.game_time = t;
        c::spawn_tick(&mut g);
    }
    let count = |k: &str| g.mobs.iter().filter(|m| &*m.kind.id == k).count();
    eprintln!("allays {} stalkers {} scavengers {} slabbers {} nuzzles {} blubs {} golems {} singers {}", count("minecraft:allay"),
        count("the_sift:stalker"), count("the_sift:scavenger"), count("the_sift:slabber"), count("the_sift:nuzzle"),
        count("the_sift:blub"), count("the_sift:echo_golem"), count("the_sift:singer"));
    assert!((2..=16 + 3).contains(&count("minecraft:allay")), "allays in the forest, up to its cap");
    // The overgrown forest's lists: stalkers and scavengers, slabbers and nuzzles, a few blubs.
    for k in ["the_sift:stalker", "the_sift:scavenger", "the_sift:slabber", "the_sift:nuzzle"] {
        assert!((1..=8 + 3).contains(&count(k)), "{k}");
    }
    assert!(count("the_sift:blub") <= 8 + 3, "blubs");
    assert!(count("the_sift:echo_golem") <= 2 && count("the_sift:singer") <= 1);
    assert!(g.spawned.iter().all(|(k, p)| p[1] == if k == "minecraft:allay" { 66.0 } else { 64.0 }), "on the ground (allays above)");
    // Elsewhere: none.
    let mut o = Game::new();
    o.in_sift = false;
    for t in 0..400 {
        o.world.game_time = t;
        c::spawn_tick(&mut o);
    }
    assert!(o.mobs.is_empty());
}

/// The drawn quads as JSON, for tools/render_mobs_check.py (a picture of what the game draws).
#[test]
#[ignore]
fn export_drawn() {
    let mut g = Game::new();
    let kinds = ["the_sift:blub", "the_sift:stalker", "the_sift:echo_golem", "the_sift:singer", "the_sift:nuzzle",
        "the_sift:slabber", "the_sift:nester", "the_sift:pollinator", "the_sift:seedling", "the_sift:scavenger", "the_sift:monarch", "the_sift:dark_sniffer"];
    for (i, k) in kinds.iter().enumerate() {
        g.mobs.push(Mob::new(i as u32, k));
    }
    g.mobs[2].extra.main_hand = Some("the_sift:soul_block".into());
    let mut out = String::from("{");
    for (i, k) in kinds.iter().enumerate() {
        let (d, _) = drawn(&g, &render(i as u32, k));
        let quads: Vec<String> = d.quads.iter().map(|(s, c, uv, _, l)| format!("[{s:?},{c:?},{uv:?},{l}]")).collect();
        out += &format!("{}{k:?}:[{}]", if i > 0 { "," } else { "" }, quads.join(","));
    }
    out += "}";
    std::fs::write("/tmp/sift_drawn.json", out).unwrap();
}

#[test]
#[ignore]
fn bench_draw() {
    let mut g = Game::new();
    let kinds = ["the_sift:blub", "the_sift:stalker", "the_sift:echo_golem", "the_sift:singer", "the_sift:nuzzle",
        "the_sift:slabber", "the_sift:nester", "the_sift:pollinator", "the_sift:seedling", "the_sift:scavenger", "the_sift:monarch", "the_sift:dark_sniffer"];
    for (i, k) in kinds.iter().enumerate() {
        g.mobs.push(Mob::new(i as u32, k));
    }
    let _ = drawn(&g, &render(0, kinds[0]));
    for (i, k) in kinds.iter().enumerate() {
        let mut r = render(i as u32, k);
        r.walk_speed = 0.5;
        let t = std::time::Instant::now();
        let n = 2000;
        for f in 0..n {
            r.age = f as f32;
            std::hint::black_box(drawn(&g, &r));
        }
        eprintln!("{k}: {:.1} µs a frame", t.elapsed().as_secs_f64() * 1e6 / n as f64);
    }
}
