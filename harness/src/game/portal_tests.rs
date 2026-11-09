// The portal opening (portal.rs) on a mock ancient city centre: the frame and the eight
// consoles where The Sift 2.0.1's city_center_1 has them (frame x=13, y 17..24, z 10..31;
// consoles x=8, y=10, z 13..27 step 2), the city's floor at y=9 under the consoles.
use super::the_sift_portal as portal;
use super::{BlockStateId, Dim, Game, LevelDimension};
use std::sync::Mutex;

/// The portal's statics are shared: one test at a time.
static SERIAL: Mutex<()> = Mutex::new(());

const CONSOLES: [[i32; 3]; 8] = [[8, 10, 13], [8, 10, 15], [8, 10, 17], [8, 10, 19], [8, 10, 21], [8, 10, 23], [8, 10, 25], [8, 10, 27]];

fn id(game: &Game<'_>, name: &str, props: &[(&str, &str)]) -> BlockStateId {
    let r = game.pack.assets.registry;
    let mut s = r.state(r.default_state(name).unwrap()).unwrap().clone();
    for (k, v) in props {
        s.properties.insert((*k).into(), (*v).into());
    }
    r.resolve(&s).unwrap()
}
fn set(game: &mut Game<'_>, p: [i32; 3], name: &str, props: &[(&str, &str)]) {
    let s = id(game, name, props);
    game.world.blocks.insert(p, s);
}

/// A city centre: its frame, its consoles (as horns) with note blocks on them, its floor.
fn city(notes: bool) -> Game<'static> {
    let mut g = Game::new(true);
    g.dim = Dim::Overworld;
    for y in 17..=24 {
        for z in 10..=31 {
            if y == 17 || y == 24 || z == 10 || z == 31 {
                set(&mut g, [13, y, z], "minecraft:reinforced_deepslate", &[]);
            }
        }
    }
    // Glow lichen in the frame grows over, as the city's does.
    set(&mut g, [13, 18, 12], "minecraft:glow_lichen", &[]);
    for x in 0..18 {
        for z in 0..41 {
            set(&mut g, [x, 9, z], "minecraft:deepslate_tiles", &[]);
        }
    }
    for c in CONSOLES {
        set(&mut g, c, "the_sift:sonorous_deepslate", &[("mode", "horn")]);
        if notes {
            set(&mut g, [c[0], c[1] + 1, c[2]], "minecraft:note_block", &[]);
        }
    }
    g.player.as_mut().unwrap().position = horizon_core::geometry::Vec3::new(10.5, 10.0, 20.5);
    g
}
fn wake(g: &mut Game<'_>) {
    for c in CONSOLES {
        set(g, c, "the_sift:sonorous_deepslate", &[("mode", "note")]);
    }
}
fn interior() -> Vec<[i32; 3]> {
    let mut v = Vec::new();
    for y in 18..=23 {
        for z in 11..=30 {
            v.push([13, y, z]);
        }
    }
    v
}
fn reset() {
    *portal::STATE.lock().unwrap() = None;
    let _ = portal::take_heard();
}
/// A player strikes the note block on console `i` tuned to tone `tone` (1..8).
fn strike(g: &Game<'_>, i: usize, tone: u8) {
    let c = CONSOLES[i];
    let below = g.state_at(c);
    let mut cancel = false;
    portal::note_play(&below.name, below.properties.get("mode").map(String::as_str), LevelDimension::Overworld, true, [c[0], c[1] + 1, c[2]], i32::from(tone) - 1, &mut cancel);
    assert!(cancel, "a console's note block sounds The Sift's tone, not its instrument");
}
fn ticks(g: &mut Game<'_>, n: u32) {
    for _ in 0..n {
        portal::tick(g);
        g.world.game_time += 1;
    }
}
/// Sounds the server sent its players, and those played in the window.
fn played(g: &Game<'_>, name: &str) -> usize {
    g.creatures.lan_sounds.iter().filter(|p| p.sound == name).count() + g.audio.played.iter().filter(|p| p.0 == name).count()
}

#[test]
fn a_horn_glitches_and_keeps_its_note() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(true);
    let (mut new, mut cancel) = (5, false);
    portal::note_change("the_sift:sonorous_deepslate", Some("horn"), LevelDimension::Overworld, [8, 11, 13], 4, &mut new, &mut cancel);
    assert!(cancel && new == 5);
    strike(&g, 0, 1);
    ticks(&mut g, 1);
    assert_eq!(played(&g, "the_sift:sonorous_autoplay_glitch"), 2);
    assert_eq!(played(&g, "the_sift:block.sonorous_deepslate.sound_1"), 0);
    // A note block on anything else is left to vanilla.
    let mut cancel = false;
    portal::note_play("minecraft:stone", None, LevelDimension::Overworld, true, [0, 1, 0], 3, &mut cancel);
    assert!(!cancel);
}

#[test]
fn a_waking_console_is_tuned_through_eight_tones() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    for (old, want) in [(0, 1), (6, 7), (7, 0), (15, 0), (24, 1)] {
        let (mut new, mut cancel) = (old + 1, false);
        portal::note_change("the_sift:sonorous_deepslate", Some("note"), LevelDimension::Overworld, [8, 11, 13], old, &mut new, &mut cancel);
        assert_eq!((new, cancel), (want, false), "from {old}");
    }
    // The tuned note sounds as a preview: not played to the console.
    let mut cancel = false;
    portal::note_play("the_sift:sonorous_deepslate", Some("note"), LevelDimension::Overworld, true, [8, 11, 13], 1, &mut cancel);
    let heard = portal::take_heard();
    assert!(matches!(heard.last(), Some(portal::Heard::Note { struck: false, index: 2, .. })), "{heard:?}");
}

#[test]
fn the_sequence_opens_the_frame_from_its_edges_and_backwards_closes_it() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(true);
    wake(&mut g);
    for (i, tone) in portal::TARGET.iter().enumerate() {
        strike(&g, i, *tone);
        ticks(&mut g, 4);
    }
    assert_eq!(played(&g, "the_sift:block.sonorous_deepslate.sound_7"), 1);
    // Burst after 10 ticks, playback 20 later, a tone every 15: beams rise as it plays.
    ticks(&mut g, 30 + 15 * 4);
    let beams = g.quads.borrow().len();
    let mut draw = super::native_world_draw::NativeWorldDraw { partial: 0.5, ..Default::default() };
    portal::draw_world(&g, &mut draw);
    let beam_quads: Vec<_> = g.quads.borrow()[beams..].iter().filter(|q| q.0 == "the_sift:block/sonorous_deepslate_beam").cloned().collect();
    assert!(!beam_quads.is_empty(), "the consoles played so far have beams");
    for q in &beam_quads {
        assert!(q.2.iter().all(|v| (0.0..=16.0).contains(v)), "a beam's uv stays in its sprite: {:?}", q.2);
        assert!(q.1.iter().all(|p| p[1] >= 11.0 - 1e-6), "a beam rises from the console's top");
    }
    ticks(&mut g, 15 * 4 + 5);
    assert_eq!(played(&g, "the_sift:the_sift_portal_open"), 1, "the frame opens with its sound");
    ticks(&mut g, 400);
    let portal_z = id(&g, "the_sift:sift_portal", &[("axis", "z")]);
    for p in interior() {
        assert_eq!(g.world.block(p[0], p[1], p[2]), portal_z, "{p:?} holds the portal, spanning z");
    }
    assert_eq!(g.name_at([13, 17, 20]), "minecraft:reinforced_deepslate", "the frame stays");
    assert!(g.particles.added.iter().any(|(o, _, _)| o.ty == horizon_client::particles::ParticleType::EndRod));
    // Its beams retract 60 ticks after, over 40.
    ticks(&mut g, 120);
    let before = g.quads.borrow().len();
    portal::draw_world(&g, &mut draw);
    assert!(g.quads.borrow()[before..].iter().all(|q| q.0 != "the_sift:block/sonorous_deepslate_beam"));

    // Backwards it closes again.
    for (i, tone) in portal::CLOSING.iter().enumerate() {
        strike(&g, i, *tone);
        ticks(&mut g, 2);
    }
    ticks(&mut g, 30 + 15 * 8 + 5 + 400);
    assert_eq!(played(&g, "the_sift:the_sift_portal_close"), 1);
    assert!(interior().iter().all(|p| g.name_at(*p) == "minecraft:air"));
}

#[test]
fn a_wrong_sequence_or_a_broken_console_opens_nothing() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(true);
    wake(&mut g);
    for (i, tone) in [1, 3, 7, 6, 5, 2, 8, 4].iter().enumerate() {
        strike(&g, i, *tone);
    }
    ticks(&mut g, 400);
    assert!(interior().iter().all(|p| g.name_at(*p) != "the_sift:sift_portal"));
    // Right, but a note block taken away while it plays back: the run fails with the glitch.
    for (i, tone) in portal::TARGET.iter().enumerate() {
        strike(&g, i, *tone);
    }
    ticks(&mut g, 30 + 15 * 3);
    g.world.blocks.remove(&[8, 11, 25]);
    ticks(&mut g, 400);
    assert!(interior().iter().all(|p| g.name_at(*p) != "the_sift:sift_portal"));
    assert!(played(&g, "the_sift:sonorous_autoplay_glitch") >= 1);
}

#[test]
fn a_goat_horn_by_the_frame_brings_the_singer_who_wakes_the_consoles() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(false);
    portal::horn_blown([10.5, 10.0, 20.5]);
    ticks(&mut g, 59);
    assert!(g.mobs.is_empty(), "it comes 60 ticks after the horn");
    ticks(&mut g, 2);
    let singer = g.mobs.first().cloned().expect("a singer");
    assert_eq!(singer.kind, "the_sift:singer");
    assert!(singer.no_ai);
    assert!((singer.position[1] - 18.0).abs() < 1e-9 && (singer.position[0] - 13.5).abs() < 1e-9, "on the frame's floor: {:?}", singer.position);
    assert!((singer.position[2] - 21.0).abs() < 1e-9, "in its middle: {:?}", singer.position);
    assert_eq!(singer.yaw, 90.0, "facing the consoles (west)");
    assert!(singer.extra.using_item < 0, "rising");
    // A second horn while it is there brings no other.
    portal::horn_blown([10.5, 10.0, 20.5]);
    ticks(&mut g, 148);
    assert!(g.mobs[0].extra.using_item > 0, "singing");
    // The window, whose copy of the mob has no using_item, reads the song from the glue.
    let song = super::the_sift::singer_song([13.5, 18.0, 21.0]).expect("the window hears the song");
    assert!(song > 0, "singing in the window too: {song}");
    assert_eq!(played(&g, "the_sift:entity.singer.sing"), 1);
    ticks(&mut g, 161);
    for c in CONSOLES {
        assert_eq!(g.state_at(c).properties.get("mode").map(String::as_str), Some("note"), "{c:?} woke");
    }
    assert_eq!(g.level_events.iter().filter(|e| e.0 == 2001).count(), 8);
    assert!(g.mobs[0].extra.using_item <= -10_000, "sinking");
    ticks(&mut g, 101);
    assert!(g.mobs[0].removed, "and gone");
    assert_eq!(g.mobs.len(), 1);
    assert!(g.particles.added.iter().any(|(o, _, _)| o.mod_particle.as_deref() == Some("the_sift:singer_sound_wave")));
}

#[test]
fn a_frame_scan_needs_a_closed_frame() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(false);
    let block = |p: [i32; 3]| super::Game::block_name(&g, p);
    let f = portal::scan(&block, [13, 17, 20], false).expect("the city's frame");
    assert!(f.axis_z);
    assert_eq!(f.interior.len(), 120);
    assert_eq!(f.bottom_center(), [13.5, 18.0, 21.0]);
    let order = portal::edges_first(&f);
    let c = f.center();
    let d = |p: &[i32; 3]| (f64::from(p[2]) + 0.5 - c[2]).powi(2) + (f64::from(p[1]) + 0.5 - c[1]).powi(2);
    assert!(d(&order[0]) >= d(order.last().unwrap()));
    // A gap in the frame: no portal.
    g.world.blocks.remove(&[13, 24, 20]);
    let block = |p: [i32; 3]| super::Game::block_name(&g, p);
    assert!(portal::scan(&block, [13, 17, 20], false).is_none());
}

#[test]
fn the_pixel_surface_covers_each_block_and_breathes() {
    // Every leaf of the k-d tree: together they cover the block once.
    let mut squares = Vec::new();
    for x in -5..9 {
        for y in -3..7 {
            squares.clear();
            portal::surface(x, y, true, 12.3, 8, &mut squares);
            assert_eq!(squares[0], ([0.0, 0.0, 1.0, 1.0], 0.505, portal::BASE));
            let area: f32 = squares.iter().filter(|s| s.1 == 0.56).map(|s| (s.0[2] - s.0[0]) * (s.0[3] - s.0[1])).sum();
            let faded = squares.iter().filter(|s| s.1 == 0.56).count();
            assert!(area <= 1.0 + 1e-4, "{x},{y}: {area}");
            assert!(faded > 0);
            for s in &squares {
                assert!(s.0[0] >= 0.0 && s.0[1] >= 0.0 && s.0[2] <= 1.0 && s.0[3] <= 1.0);
            }
        }
    }
    let mut a = Vec::new();
    let mut b = Vec::new();
    portal::surface(3, 4, true, 1.0, 8, &mut a);
    portal::surface(3, 4, true, 4.0, 8, &mut b);
    assert_eq!(a.iter().map(|s| s.0).collect::<Vec<_>>(), b.iter().map(|s| s.0).collect::<Vec<_>>(), "the same pixels");
    assert_ne!(a.iter().map(|s| s.2).collect::<Vec<_>>(), b.iter().map(|s| s.2).collect::<Vec<_>>(), "their alpha breathes");
}

#[test]
fn the_screen_on_the_way_is_covered() {
    let mut fills = Vec::new();
    portal::draw_background(427.0, 240.0, 5.0, 1.0, &mut |r, c| fills.push((r, c)));
    assert_eq!(fills[0], ([0.0, 0.0, 427.0, 240.0], 0xFF26_C0C5));
    assert_eq!(*fills.last().unwrap(), ([0.0, 0.0, 427.0, 240.0], 0x6500_0000));
    assert!(fills.len() > 100);
    assert!(fills.iter().all(|(r, _)| r[0] >= 0.0 && r[1] >= 0.0 && r[0] + r[2] <= 427.0 + 1e-3 && r[1] + r[3] <= 240.0 + 1e-3));
}

#[test]
fn a_portal_in_view_is_drawn_both_ways_with_its_rim() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(false);
    set(&mut g, [13, 18, 11], "the_sift:sift_portal", &[("axis", "z")]);
    set(&mut g, [13, 18, 12], "the_sift:sift_portal", &[("axis", "z")]);
    let s = id(&g, "the_sift:sift_portal", &[("axis", "z")]);
    portal::seen_portal(&g, [13, 18, 11], s);
    portal::seen_portal(&g, [13, 18, 11], s);
    let mut draw = super::native_world_draw::NativeWorldDraw::default();
    portal::draw_world(&g, &mut draw);
    let quads = g.quads.borrow();
    assert!(quads.len() >= 10 && quads.len() % 2 == 0, "{}", quads.len());
    assert!(quads.iter().all(|q| q.0 == "the_sift:block/sift_white"));
    // In the block's x plane (axis z): every corner at x 13.375..13.625.
    assert!(quads.iter().all(|q| q.1.iter().all(|p| (13.37..=13.63).contains(&p[0]))));
    // Seen twice, drawn once; seen again next frame, drawn again.
    let rims = quads.iter().filter(|q| q.3 == [242, 255, 255, 255]).count();
    assert_eq!(rims, 12, "three sides end (south has the next portal block): a line on each face, each seen both ways");
}

#[test]
fn the_intro_plays_on_a_first_entry_only() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut g = Game::new(true);
    let _ = std::fs::remove_dir_all(&g.data_dir);
    let first = super::the_sift_portal::first_entry(&mut g);
    let again = super::the_sift_portal::first_entry(&mut g);
    assert!(first && !again);
    let _ = std::fs::remove_dir_all(&g.data_dir);
}

#[test]
fn bench_surface_and_screen() {
    let mut squares = Vec::new();
    let n = 20_000;
    let t = std::time::Instant::now();
    for i in 0..n {
        squares.clear();
        portal::surface(i % 20, i / 20 % 6, i % 2 == 0, 3.5, 8, &mut squares);
    }
    let per_block = t.elapsed().as_secs_f64() / f64::from(n) * 1e6;
    let t = std::time::Instant::now();
    let mut count = 0;
    for _ in 0..50 {
        portal::draw_background(640.0, 360.0, 3.5, 1.0, &mut |_, _| count += 1);
    }
    let per_screen = t.elapsed().as_secs_f64() / 50.0 * 1e3;
    println!("BENCH portal surface {per_block:.2} us/block face, screen {per_screen:.3} ms/frame ({} fills)", count / 50);
    assert!(per_block < 50.0 && per_screen < 5.0);
}

/// The previews' data: a 20 by 6 portal's surface at twelve moments, and the screen on the way
/// at twelve more (previews/portal_*.png, .gif, by tools/render_portal.py).
#[test]
#[ignore]
fn export_portal_preview() {
    let mut frames = Vec::new();
    for f in 0..12 {
        let time = 40.0 + f64::from(f) * 0.5;
        let mut blocks = Vec::new();
        for y in 0..6 {
            for x in 0..20 {
                let mut squares = Vec::new();
                portal::surface(x, y, true, time, 8, &mut squares);
                blocks.push(serde_json::json!({ "x": x, "y": y, "squares": squares.iter().map(|s| (s.0, s.1, s.2)).collect::<Vec<_>>() }));
            }
        }
        let mut fills = Vec::new();
        portal::draw_background(640.0, 360.0, time, 1.0, &mut |r, c| fills.push((r, c)));
        frames.push(serde_json::json!({ "time": time, "blocks": blocks, "screen": fills }));
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/portal_data.json");
    std::fs::write(out, serde_json::to_string(&frames).unwrap()).unwrap();
}

#[test]
fn a_client_plays_no_part_of_the_opening() {
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(true);
    g.authoritative = false;
    wake(&mut g);
    for (i, tone) in portal::TARGET.iter().enumerate() {
        strike(&g, i, *tone);
    }
    portal::horn_blown([10.5, 10.0, 20.5]);
    ticks(&mut g, 400);
    assert!(g.mobs.is_empty() && interior().iter().all(|p| g.name_at(*p) != "the_sift:sift_portal"));
    // What it heard waits for the server.
    assert_eq!(portal::take_heard().len(), 8);
}

#[test]
fn a_tuned_note_block_keeps_its_new_note() {
    // The game keeps a tuned note only if the note's sound plays; a console's tone replaces it,
    // so the server's tick sets the note itself.
    let _s = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut g = city(false);
    wake(&mut g);
    let c = CONSOLES[0];
    let note = [c[0], c[1] + 1, c[2]];
    set(&mut g, note, "minecraft:note_block", &[("note", "0")]);
    for want in ["1", "2", "3"] {
        let old: i32 = g.state_at(note).properties["note"].parse().unwrap();
        let (mut new, mut cancel) = (old + 1, false);
        portal::note_change("the_sift:sonorous_deepslate", Some("note"), LevelDimension::Overworld, note, old, &mut new, &mut cancel);
        let mut play_cancel = false;
        portal::note_play("the_sift:sonorous_deepslate", Some("note"), LevelDimension::Overworld, true, note, new, &mut play_cancel);
        ticks(&mut g, 1);
        assert_eq!(g.state_at(note).properties["note"], want);
    }
}
