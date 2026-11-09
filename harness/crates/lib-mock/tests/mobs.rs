//! ai.rs and tame.rs as they ship: The Sift's mobs in the mob code.
use horizon_client_lib::mobs::goals::Cx;
use horizon_client_lib::mobs::{Mob, MobEvent, MobType, MobWorld, Seen, ai, undead};
use horizon_core::geometry::Vec3;
use std::collections::HashMap;
use std::sync::Arc;

fn mob(id: u32, kind: &str) -> Mob {
    let mut m = Mob::new(id, Arc::new(MobType::zombie(kind)));
    ai::native_register_goals(&mut m);
    m
}

fn world<'a>(others: &'a [Seen], blocks: &'a HashMap<[i32; 3], &'static str>) -> MobWorld<'a> {
    MobWorld { others, player: Some(horizon_client_lib::mobs::PlayerView { position: Vec3::new(4.0, 64.0, 0.0), eye: [4.0, 65.6, 0.0] }), blocks }
}

#[test]
fn each_mob_has_its_own_type_and_goals() {
    let b = mob(1, "the_sift:blub");
    assert_eq!((b.kind.width, b.kind.height, b.kind.eye_height), (0.665, 0.875, 0.4375));
    assert_eq!((b.kind.category.as_str(), b.kind.flags.burns_in_day, b.kind.flags.undead), ("creature", false, false));
    assert_eq!(b.kind.sounds[0], "entity.axolotl.idle_air");
    assert_eq!(b.ai.goals.names().iter().map(|g| g.0).collect::<Vec<_>>(), [0, 1, 2, 3, 4, 7, 8, 9]);
    assert_eq!(b.ai.targets.goals.len(), 3);
    let s = mob(2, "the_sift:stalker");
    assert_eq!((s.kind.width, s.kind.height, s.kind.category.as_str()), (0.91, 2.375, "monster"));
    assert!(s.ai.goals.names().iter().any(|g| g.1 == "MeleeAttack") && s.ai.targets.goals.len() == 2);
    let e = mob(3, "the_sift:echo_golem");
    assert!(e.ai.goals.names().iter().any(|g| g.1 == "SoulQuest") && e.ai.targets.goals.is_empty());
    let g = mob(4, "the_sift:singer");
    assert_eq!((g.kind.height, g.kind.category.as_str()), (4.8, "misc"));
    assert!(g.ai.goals.names().iter().any(|g| g.1 == "Sing"));
    // One copy per type it's made from, shared by every mob of it.
    let zombie_blub = Arc::new(MobType::zombie("the_sift:blub"));
    let (mut b1, mut b2) = (Mob::new(7, zombie_blub.clone()), Mob::new(8, zombie_blub));
    ai::native_register_goals(&mut b1);
    ai::native_register_goals(&mut b2);
    assert!(Arc::ptr_eq(&b1.kind, &b2.kind) && b1.kind.width == 0.665);
    // A zombie is left a zombie.
    let z = mob(6, "minecraft:zombie");
    assert!(z.kind.flags.burns_in_day && z.ai.goals.goals.is_empty());
    // A zombie's reinforcements and drowning: not for The Sift's.
    let (mut b, mut z) = (b, z);
    assert!(!undead::after_hurt(&mut b) && !undead::custom_server_ai_step(&mut b));
    assert!(undead::after_hurt(&mut z) && undead::custom_server_ai_step(&mut z));
}

#[test]
fn a_zombies_spawn_gear_is_undone() {
    let mut s = mob(2, "the_sift:stalker");
    s.extra.age = -24000;
    s.extra.main_hand = Some("minecraft:iron_sword".into());
    s.extra.head = Some("minecraft:iron_helmet".into());
    s.extra.pick_up_loot = true;
    let blocks = HashMap::new();
    let mut ev = Vec::new();
    ai::pre_tick(&mut s, &world(&[], &blocks), &mut ev);
    assert!(!s.is_baby() && s.extra.main_hand.is_none() && s.extra.head.is_none() && !s.extra.pick_up_loot);
    assert_eq!(s.extra.variant, 0xB5);
}

#[test]
fn blubs_hop_as_they_go() {
    let mut b = mob(1, "the_sift:blub");
    let blocks = HashMap::new();
    let mut ev = Vec::new();
    b.ai.mov.has = true;
    b.ai.mov.speed_modifier = 1.0;
    b.ai.nav.moving_to = Some([5.0, 64.0, 0.0]);
    b.ai.mov.wanted = [5.0, 64.0, 0.0];
    ai::pre_tick(&mut b, &world(&[], &blocks), &mut ev);
    assert_eq!(b.velocity.y, 0.3);
    b.velocity.y = 0.0;
    b.ai.mov.wanted = [5.0, 65.0, 0.0];
    ai::pre_tick(&mut b, &world(&[], &blocks), &mut ev);
    assert_eq!(b.velocity.y, 0.5, "higher when its path climbs");
    // Sitting: still.
    b.velocity.y = 0.0;
    b.extra.sitting = true;
    ai::pre_tick(&mut b, &world(&[], &blocks), &mut ev);
    assert_eq!(b.velocity.y, 0.0);
}

#[test]
fn echo_golems_carry_souls_to_singers() {
    let mut g = mob(3, "the_sift:echo_golem");
    let mut blocks = HashMap::new();
    blocks.insert([6, 60, 0], "the_sift:soul_block");
    let singer = [Seen { id: 9, alive: true, variant: 0xB4, position: Vec3::new(-20.0, 64.0, 0.0) }];
    let mut lr = horizon_core::random::LegacyRandom::new(1);
    let mut events: Vec<MobEvent> = Vec::new();
    let mut quest = g.ai.goals.goals.iter().find(|(_, q)| q.name() == "SoulQuest").unwrap().1.box_clone();
    let w = world(&singer, &blocks);
    let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
    assert!(quest.can_use(&mut g, &mut cx), "a soul block in reach");
    quest.start(&mut g, &mut cx);
    quest.tick(&mut g, &mut cx);
    assert!(g.ai.nav.moving_to.is_some(), "it walks there");
    // At the block: it lifts it out.
    g.position = Vec3::new(6.5, 60.6, 0.5);
    quest.tick(&mut g, &mut cx);
    assert_eq!(g.extra.main_hand.as_deref(), Some("the_sift:soul_block"));
    assert!(cx.events.iter().any(|e| matches!(e, MobEvent::SetBlockName { pos: [6, 60, 0], name: "minecraft:air", .. })));
    // Carrying: off to the singer, and at the singer it hands the soul over for a reward.
    quest.stop(&mut g, &mut cx);
    let mut tries = 0;
    while !quest.can_use(&mut g, &mut cx) {
        tries += 1;
        assert!(tries < 200);
    }
    quest.start(&mut g, &mut cx);
    quest.tick(&mut g, &mut cx);
    assert_eq!(g.ai.nav.moving_to, Some([-20.0, 64.0, 0.0]));
    g.position = Vec3::new(-18.0, 64.0, 0.0);
    quest.tick(&mut g, &mut cx);
    assert!(g.extra.main_hand.is_none());
    assert!(cx.events.iter().any(|e| matches!(e, MobEvent::DropItem { mob: 9, .. })));
}

#[test]
fn every_barter_is_one_of_the_singers() {
    let mut r = horizon_core::random::LegacyRandom::new(3);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..2000 {
        let (item, count) = ai::the_sift_native::barter(&mut r);
        assert!(count >= 1 && count <= 24, "{item} {count}");
        seen.insert(item);
    }
    assert!(seen.len() >= 30, "all seventeen, the wool in its sixteen colours: {}", seen.len());
}

#[test]
fn the_nuzzle_sounds_its_own_not_a_sheep() {
    let mut n = mob(301, "the_sift:nuzzle");
    assert_eq!(n.kind.sounds, ["intentionally_empty"; 3], "no vanilla stand-in");
    assert_eq!(n.kind.step_sound, None);
    let (others, blocks) = (Vec::new(), HashMap::new());
    let mut events = Vec::new();
    n.hurt_time = 10;
    ai::pre_tick(&mut n, &world(&others, &blocks), &mut events);
    let ids: Vec<String> = events.iter().filter_map(|e| match e { MobEvent::SoundId(s, ..) => Some(s.clone()), _ => None }).collect();
    assert!(ids.iter().any(|s| s == "the_sift:entity.nuzzle.hurt"), "{ids:?}");
    events.clear();
    n.health = 0.0;
    ai::pre_tick(&mut n, &world(&others, &blocks), &mut events);
    ai::pre_tick(&mut n, &world(&others, &blocks), &mut events);
    let deaths = events.iter().filter(|e| matches!(e, MobEvent::SoundId(s, ..) if s == "the_sift:entity.nuzzle.death")).count();
    assert_eq!(deaths, 1);
}

#[test]
fn the_sift_2_mobs_fight_their_own_way() {
    use horizon_client_lib::mobs::Who;
    use horizon_client_lib::mobs::goals::Goal;
    let blocks = HashMap::new();
    // The stalker lunges at a player 5 blocks off (StalkerEntity.ClawAttackGoal), then rests.
    let s = mob(401, "the_sift:stalker");
    assert!(s.ai.goals.names().iter().any(|g| g.1 == "Lunge") && s.ai.goals.names().iter().any(|g| g.1 == "MeleeAttack"));
    let mut lunge = ai::the_sift_native::Lunge::stalker();
    let mut m = mob(402, "the_sift:stalker");
    m.ai.target = Some(Who::Player);
    m.position = Vec3::new(-1.0, 64.0, 0.0);
    let w = world(&[], &blocks);
    let mut events = Vec::new();
    let mut lr = horizon_core::random::LegacyRandom::new(1);
    let mut used = false;
    for _ in 0..200 {
        let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
        if lunge.can_use(&mut m, &mut cx) {
            lunge.start(&mut m, &mut cx);
            used = true;
            break;
        }
    }
    assert!(used, "it lunges");
    assert!(m.velocity.x > 1.0 && m.velocity.y > 0.3, "at the player (east): {:?}", m.velocity);
    assert!(events.iter().any(|e| matches!(e, MobEvent::SoundId(s, ..) if s == "the_sift:entity.stalker.dash")));
    // Too close or too far: no lunge.
    m.position = Vec3::new(3.0, 64.0, 0.0);
    let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
    let mut fresh = ai::the_sift_native::Lunge::stalker();
    assert!((0..200).all(|_| !fresh.can_use(&mut m, &mut cx)));

    // The scavenger keeps its distance and shoots (6 damage); the pollinator drools fire.
    let sc = mob(403, "the_sift:scavenger");
    assert!(sc.ai.goals.names().iter().any(|g| g.1 == "RangedAttack") && !sc.ai.goals.names().iter().any(|g| g.1 == "MeleeAttack"));
    let po = mob(404, "the_sift:pollinator");
    assert!(po.ai.goals.names().iter().any(|g| g.1 == "RangedAttack") && !po.ai.targets.goals.is_empty());
    for (id, kind, want) in [(405, "the_sift:scavenger", "Arrow"), (406, "the_sift:pollinator", "SmallFireball")] {
        let mut m = mob(id, kind);
        m.ai.target = Some(Who::Player);
        m.position = Vec3::new(-8.0, 64.0, 0.0);
        let mut events = Vec::new();
        let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
        let (_, g) = m.ai.goals.goals.iter_mut().find(|g| g.1.name() == "RangedAttack").map(|(p, g)| (*p, g.box_clone())).unwrap();
        let mut g = g;
        g.tick(&mut m, &mut cx);
        let shot = events.iter().find_map(|e| match e { MobEvent::Projectile(p) => Some(p.clone()), _ => None }).expect("a shot");
        assert!(format!("{:?}", shot.kind).starts_with(want), "{kind}: {:?}", shot.kind);
        assert!(shot.velocity.x > 0.0, "toward the player");
        if let horizon_client_lib::projectiles::ProjectileKind::Arrow { base_damage, .. } = shot.kind {
            assert_eq!((1.6 * base_damage).ceil(), 6.0, "ScavengerProjectileEntity's 6 damage");
        }
    }

    // The nester warns for 32 ticks, then rams: 5 damage, once.
    let mut ram = ai::the_sift_native::Ram::default();
    let mut n = mob(407, "the_sift:nester");
    n.ai.target = Some(Who::Player);
    n.position = Vec3::new(2.5, 64.0, 0.0);
    let mut events = Vec::new();
    let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
    assert!(ram.can_use(&mut n, &mut cx));
    ram.start(&mut n, &mut cx);
    for _ in 0..ai::the_sift_native::RAM_WARNING {
        ram.tick(&mut n, &mut cx);
    }
    assert!(!cx.events.iter().any(|e| matches!(e, MobEvent::AttackPlayer(..))), "no blow while it warns");
    for _ in 0..ai::the_sift_native::RAM_TICKS {
        ram.tick(&mut n, &mut cx);
    }
    let blows: Vec<f32> = cx.events.iter().filter_map(|e| match e { MobEvent::AttackPlayer(a, _) => Some(*a), _ => None }).collect();
    assert_eq!(blows, [5.0]);
    assert!(!ram.can_continue(&mut n, &mut cx), "8 ticks of ram and it's done");

    // The monarch: lunges and volleys of three.
    let mo = mob(408, "the_sift:monarch");
    let names: Vec<&str> = mo.ai.goals.names().iter().map(|g| g.1).collect();
    assert!(names.contains(&"Lunge") && names.contains(&"Volley") && names.contains(&"MeleeAttack"), "{names:?}");
    let mut v = ai::the_sift_native::Volley::default();
    let mut m = mob(409, "the_sift:monarch");
    m.ai.target = Some(Who::Player);
    m.position = Vec3::new(-12.0, 64.0, 0.0);
    let mut events = Vec::new();
    let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
    assert!(v.can_use(&mut m, &mut cx));
    v.start(&mut m, &mut cx);
    assert_eq!(cx.events.iter().filter(|e| matches!(e, MobEvent::Projectile(_))).count(), 3);
}

#[test]
fn the_dark_sniffer_bites_close_and_sniffs_a_far_player_in() {
    use horizon_client_lib::mobs::Who;
    use horizon_client_lib::mobs::goals::Goal;
    let d = mob(501, "the_sift:dark_sniffer");
    assert_eq!((d.kind.width, d.kind.height, d.kind.category.as_str()), (1.9, 1.75, "monster"));
    let names: Vec<&str> = d.ai.goals.names().iter().map(|g| g.1).collect();
    assert!(names.contains(&"Sniff") && names.contains(&"MeleeAttack"), "{names:?}");
    let blocks = HashMap::new();
    let w = world(&[], &blocks);
    let mut lr = horizon_core::random::LegacyRandom::new(1);
    let mut s = ai::the_sift_native::Sniff::default();
    let mut m = mob(502, "the_sift:dark_sniffer");
    m.ai.target = Some(Who::Player);
    // Within 5 blocks: no sniff (it bites instead).
    m.position = Vec3::new(1.0, 64.0, 0.0);
    let mut events = Vec::new();
    let mut cx = Cx { w: &w, lr: &mut lr, events: &mut events };
    assert!((0..20).all(|_| !s.can_use(&mut m, &mut cx)));
    // 8 blocks off for 8 ticks (4 checks): it sniffs, and after its 10-tick windup pulls the player in.
    m.position = Vec3::new(-4.0, 64.0, 0.0);
    let used = (0..4).filter(|_| s.can_use(&mut m, &mut cx)).count();
    assert_eq!(used, 1, "after 8 ticks out of reach");
    s.start(&mut m, &mut cx);
    for _ in 0..ai::the_sift_native::SNIFF_WINDUP {
        s.tick(&mut m, &mut cx);
    }
    let pull = cx.events.iter().find_map(|e| match e { MobEvent::SetVelocity { target: Who::Player, velocity } => Some(*velocity), _ => None }).expect("a pull");
    assert!(pull[0] < 0.0, "toward the sniffer (west): {pull:?}");
    assert!(cx.events.iter().any(|e| matches!(e, MobEvent::SoundId(s, ..) if s == "the_sift:entity.dark_sniffer.sniff")));
    s.stop(&mut m, &mut cx);
    assert!(!s.can_use(&mut m, &mut cx), "then it rests");
}
