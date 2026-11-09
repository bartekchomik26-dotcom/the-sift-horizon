// items.rs on the mock game: willow saplings, charoite at the enchanting table, the monarch elytra.
use super::the_sift_items as items;
use super::{Dim, Game, ItemStack};

fn templates() {
    let mut t = items::TEMPLATES.lock().unwrap();
    if t.is_empty() {
        for kind in ["overgrown_willow", "ash_willow"] {
            for i in 1..=7 {
                let path = format!("{}/../mixins/the_sift_native/data/templates/{kind}_{i}.txt", env!("CARGO_MANIFEST_DIR"));
                let text: &'static str = Box::leak(std::fs::read_to_string(path).unwrap().into_boxed_str());
                let name: &'static str = Box::leak(format!("{kind}_{i}").into_boxed_str());
                t.push((name, text));
            }
        }
    }
}

fn plant(g: &mut Game<'_>, name: &str) {
    let id = g.pack.assets.registry.default_state(name).unwrap();
    g.world.blocks.insert([0, 64, 0], id);
}

#[test]
fn a_willow_sapling_grows_one_of_its_willows() {
    templates();
    for (sapling, ash, log) in [("the_sift:overgrown_willow_sapling", false, "the_sift:overgrown_willow_log"), ("the_sift:ash_willow_sapling", true, "the_sift:ash_willow_log")] {
        for pick in 0..7 {
            let mut g = Game::new(true);
            g.dim = Dim::End;
            plant(&mut g, sapling);
            assert!(items::grow(&mut g, [0, 64, 0], ash, pick), "{sapling} grows its willow {pick}");
            assert_eq!(g.name_at([0, 64, 0]), log, "the trunk where the sapling stood");
            let leaves = g.world.blocks.keys().filter(|p| g.name_at(**p).ends_with("_foliage")).count();
            assert!(leaves > 10, "{sapling} {pick}: {leaves} leaves");
        }
    }
    // Random ticks: one in seven grows it, and its own tick (an oak's) never runs.
    let mut handled = false;
    items::random_tick("minecraft:the_end", "the_sift:ash_willow_sapling", [0, 64, 0], 3, &mut handled);
    assert!(handled);
    let mut handled = false;
    items::random_tick("minecraft:the_end", "minecraft:oak_sapling", [0, 64, 0], 0, &mut handled);
    assert!(!handled, "vanilla's saplings are left alone");
    // Grown on the server's tick.
    let mut g = Game::new(true);
    g.dim = Dim::End;
    plant(&mut g, "the_sift:overgrown_willow_sapling");
    items::bonemeal("minecraft:the_end", "the_sift:overgrown_willow_sapling", [0, 64, 0]);
    items::tick(&mut g);
    assert_eq!(g.name_at([0, 64, 0]), "the_sift:overgrown_willow_log");
}

#[test]
fn charoite_lifts_the_tables_offers_to_fifty() {
    let mut costs = [8, 19, 30];
    items::table_fuel([1, 2, 3], Some("minecraft:lapis_lazuli"));
    items::enchant_costs([1, 2, 3], &mut costs);
    assert_eq!(costs, [8, 19, 30], "lapis: vanilla's");
    items::table_fuel([1, 2, 3], Some("the_sift:charoite"));
    items::enchant_costs([1, 2, 3], &mut costs);
    assert_eq!(costs, [28, 39, 50]);
    let mut none = [0, 0, 0];
    items::enchant_costs([1, 2, 3], &mut none);
    assert_eq!(none, [0, 0, 0], "no item, no offers");
}

#[test]
fn the_monarch_elytra_lifts_while_jump_is_held() {
    let mut g = Game::new(true);
    g.remote = Some(());
    g.authoritative = false;
    g.armor[1] = Some(ItemStack { item: "the_sift:monarch_elytra".into(), count: 1 });
    {
        let p = g.player.as_mut().unwrap();
        p.ground.on_ground = false;
        p.fall_flying = true;
    }
    // MonarchFlight: it starts only from a glide already under way.
    items::tick(&mut g);
    g.player.as_mut().unwrap().jumping = true;
    let start = g.player.as_ref().unwrap().position.y;
    let mut rose = 0;
    for _ in 0..80 {
        items::tick(&mut g);
        let p = g.player.as_mut().unwrap();
        if p.velocity.y > 0.1 {
            rose += 1;
        }
        p.position.y += p.velocity.y;
    }
    let p = g.player.as_ref().unwrap();
    assert!(rose > 10, "it lifts");
    assert!((p.position.y - start - 18.0).abs() < 1.0, "up to 18 blocks: {}", p.position.y - start);
    assert!(p.fall_flying, "then glides");
    assert!(items::lift(18.0) <= 0.65 && items::lift(0.1) >= 0.13);
    // Without it, nothing.
    let mut h = Game::new(true);
    h.remote = Some(());
    h.authoritative = false;
    h.player.as_mut().unwrap().jumping = true;
    h.player.as_mut().unwrap().ground.on_ground = false;
    items::tick(&mut h);
    assert!(h.player.as_ref().unwrap().velocity.y < 0.0);
}

#[test]
fn a_blubby_berry_bush_grows_and_is_picked_for_its_own_berries() {
    let mut g = Game::new(true);
    let pos = [3, 64, 3];
    let r = g.pack.assets.registry;
    let bare = r.default_state(items::BERRY_BUSH).unwrap();
    g.world.blocks.insert(pos, bare);
    let grown = |g: &Game<'_>| g.pack.assets.registry.state(g.world.block(3, 64, 3)).unwrap().properties["grown"].clone();
    // Bare: nothing to pick, the click goes on as the template's.
    let mut canceled = false;
    items::right_click(&mut g, pos, &mut canceled);
    assert!(!canceled && g.popped.is_empty());
    // Bone meal while bare (always a success), never once grown.
    assert_eq!(items::berry_bonemeal_valid(items::BERRY_BUSH, Some("false")), Some(true));
    assert_eq!(items::berry_bonemeal_valid(items::BERRY_BUSH, Some("true")), Some(false));
    assert_eq!(items::berry_bonemeal_valid("minecraft:sweet_berry_bush", Some("false")), None);
    // One random tick in ten grows it; the others don't, and vanilla's own is held back.
    let mut handled = false;
    items::random_tick("minecraft:the_end", items::BERRY_BUSH, pos, 3, &mut handled);
    assert!(handled);
    items::tick(&mut g);
    assert_eq!(grown(&g), "false");
    items::random_tick("minecraft:the_end", items::BERRY_BUSH, pos, 20, &mut handled);
    items::tick(&mut g);
    assert_eq!(grown(&g), "true");
    // Picked: its own berries, 1 to 4, and bare again.
    items::right_click(&mut g, pos, &mut canceled);
    assert!(canceled);
    assert_eq!(g.popped.len(), 1);
    let (at, stack) = &g.popped[0];
    assert_eq!((*at, &*stack.item), (pos, items::BLUBBY_BERRIES));
    assert!((1..=4).contains(&stack.count));
    assert_eq!(grown(&g), "false");
    assert!(g.creatures.lan_sounds.iter().any(|s| s.sound == "minecraft:block.sweet_berry_bush.pick_berries" && s.source == 4));
    // BlubbyBerriesBushBlock.harvestCount's odds: 10, 6, 3 and 1 in 20.
    let counts: Vec<u32> = (0..20).map(items::harvest_count).collect();
    for (n, want) in [(1, 10), (2, 6), (3, 3), (4, 1)] {
        assert_eq!(counts.iter().filter(|c| **c == n).count(), want);
    }
}

#[test]
fn ichor_fills_and_empties_the_cauldron_as_the_jar_has_it() {
    use items::{ICHOR_BOTTLE, ICHOR_BUCKET, ICHOR_CAULDRON};
    // The rules, case by case (IchorCauldronInteractions, IchorBottleItem.fillFromCauldron).
    let rule = |b, l, h| items::cauldron_use(b, l, h).map(|r| (r.0, r.1));
    assert_eq!(rule("minecraft:cauldron", 0, ICHOR_BUCKET), Some((Some((ICHOR_CAULDRON, 3)), Some("minecraft:bucket"))));
    assert_eq!(rule("minecraft:cauldron", 0, ICHOR_BOTTLE), Some((Some((ICHOR_CAULDRON, 1)), Some("minecraft:glass_bottle"))));
    assert_eq!(rule(ICHOR_CAULDRON, 2, ICHOR_BOTTLE), Some((Some((ICHOR_CAULDRON, 3)), Some("minecraft:glass_bottle"))));
    assert_eq!(rule(ICHOR_CAULDRON, 3, ICHOR_BOTTLE), Some((None, None)), "full: the bottle keeps its ichor");
    assert_eq!(rule(ICHOR_CAULDRON, 3, "minecraft:bucket"), Some((Some(("minecraft:cauldron", 0)), Some(ICHOR_BUCKET))));
    assert_eq!(rule(ICHOR_CAULDRON, 2, "minecraft:bucket"), Some((None, None)), "no water bucket out of ichor");
    assert_eq!(rule(ICHOR_CAULDRON, 1, "minecraft:glass_bottle"), Some((Some(("minecraft:cauldron", 0)), Some(ICHOR_BOTTLE))));
    assert_eq!(rule(ICHOR_CAULDRON, 3, "minecraft:glass_bottle"), Some((Some((ICHOR_CAULDRON, 2)), Some(ICHOR_BOTTLE))));
    assert_eq!(rule(ICHOR_CAULDRON, 3, "minecraft:leather_chestplate"), Some((None, None)), "ichor doesn't wash dye off");
    assert_eq!(rule(ICHOR_CAULDRON, 3, "minecraft:water_bucket"), None, "vanilla's own fill stays");
    assert_eq!(rule("minecraft:cauldron", 0, "minecraft:water_bucket"), None);
    assert_eq!(rule("minecraft:water_cauldron", 3, "minecraft:glass_bottle"), None);

    // In the world: a bucket of ichor into an empty cauldron, then a bottle out of it.
    let mut g = Game::new(true);
    let pos = [5, 64, 5];
    let r = g.pack.assets.registry;
    g.world.blocks.insert(pos, r.default_state("minecraft:cauldron").unwrap());
    g.hand = Some(ItemStack::new(ICHOR_BUCKET, 1));
    let mut canceled = false;
    items::right_click(&mut g, pos, &mut canceled);
    assert!(canceled);
    let s = g.pack.assets.registry.state(g.world.block(5, 64, 5)).unwrap().clone();
    assert_eq!((s.name.as_str(), s.properties["level"].as_str()), (ICHOR_CAULDRON, "3"));
    assert_eq!(g.hand.as_ref().map(|h| &*h.item), Some("minecraft:bucket"));
    assert!(g.creatures.lan_sounds.iter().any(|s| s.sound == "the_sift:item.ichor_bucket.empty"));
    g.hand = Some(ItemStack::new("minecraft:glass_bottle", 2));
    let mut canceled = false;
    items::right_click(&mut g, pos, &mut canceled);
    let s = g.pack.assets.registry.state(g.world.block(5, 64, 5)).unwrap().clone();
    assert_eq!(s.properties["level"], "2");
    assert_eq!(g.hand.as_ref().map(|h| h.count), Some(1));
    assert_eq!(g.given.first().map(|h| &*h.item), Some(ICHOR_BOTTLE));
}

#[test]
fn sculkflower_seeds_plant_the_sifts_crop_which_becomes_a_sculkflower() {
    use items::{SCULKFLOWER, SCULKFLOWER_CROP, SCULKFLOWER_SEEDS};
    let mut g = Game::new(true);
    let r = g.pack.assets.registry;
    let soil = [7, 63, 7];
    let crop = [7, 64, 7];
    g.world.blocks.insert(soil, r.default_state("the_sift:royal_sculk").unwrap());
    g.hand = Some(ItemStack::new(SCULKFLOWER_SEEDS, 3));
    // Not on a side, and not on stone: nothing planted, and no beetroots either.
    let mut claimed = false;
    items::use_item_on(&mut g, SCULKFLOWER_SEEDS, soil, [1, 0, 0], &mut claimed);
    assert!(claimed && g.world.blocks.get(&[8, 63, 7]).is_none());
    // On top of royal sculk: The Sift's crop, one seed used.
    items::use_item_on(&mut g, SCULKFLOWER_SEEDS, soil, [0, 1, 0], &mut claimed);
    let name = |g: &Game<'_>| g.pack.assets.registry.state(g.world.block(7, 64, 7)).unwrap().clone();
    assert_eq!(name(&g).name, SCULKFLOWER_CROP);
    assert_eq!(g.hand.as_ref().unwrap().count, 2);
    assert!(items::crop_soil("the_sift:healthy_sculk") && items::crop_soil("minecraft:farmland") && !items::crop_soil("minecraft:dirt"));
    // CropBlock's odds: 1 in 26 on sculk, 1 in 7 on moist farmland.
    assert_eq!((0..2600).filter(|r| items::crop_grows("the_sift:royal_sculk", 0, *r)).count(), 100);
    assert_eq!((0..700).filter(|r| items::crop_grows("minecraft:farmland", 7, *r)).count(), 100);
    // A random tick that misses, then one that grows it.
    let mut handled = false;
    items::random_tick("minecraft:the_end", SCULKFLOWER_CROP, crop, 1, &mut handled);
    assert!(handled);
    items::tick(&mut g);
    assert_eq!(name(&g).properties["age"], "0");
    items::random_tick("minecraft:the_end", SCULKFLOWER_CROP, crop, 26, &mut handled);
    items::tick(&mut g);
    assert_eq!(name(&g).properties["age"], "1");
    // Bone meal: one stage, and the last is the sculkflower.
    items::crop_bonemeal("minecraft:the_end", crop);
    items::tick(&mut g);
    assert_eq!(name(&g).name, SCULKFLOWER);
}

#[test]
fn siftslate_and_soulshale_grow_over_with_bone_meal_and_die_back_when_covered() {
    use items::{SIFTSLATE_GROWTH, SOULSHALE_GROWTH};
    assert_eq!(items::spreads_to("the_sift:siftslate"), Some(SIFTSLATE_GROWTH));
    assert_eq!(items::spreads_to("the_sift:soulshale"), Some(SOULSHALE_GROWTH));
    assert_eq!(items::spreads_to("minecraft:stone"), None);
    assert!(items::smothers("minecraft:water", false, false) && items::smothers("x:y", true, false));
    assert!(items::smothers("the_sift:siftslate", false, true));
    assert!(!items::smothers("the_sift:ichor_snow", false, true) && !items::smothers("minecraft:air", false, false));
    let mut g = Game::new(true);
    let r = g.pack.assets.registry;
    let p = [9, 64, 9];
    let name = |g: &Game<'_>| g.pack.assets.registry.state(g.world.block(9, 64, 9)).unwrap().clone();
    // Bone meal: the stone becomes its growth.
    g.world.blocks.insert(p, r.default_state("the_sift:soulshale").unwrap());
    items::spread("minecraft:the_end", p, SOULSHALE_GROWTH);
    items::tick(&mut g);
    assert_eq!(name(&g).name, SOULSHALE_GROWTH);
    // Ichor snow on it: snowy (its random tick holds vanilla's back).
    g.world.blocks.insert(p, r.default_state(SIFTSLATE_GROWTH).unwrap());
    g.world.blocks.insert([9, 65, 9], r.default_state("the_sift:ichor_snow").unwrap());
    let mut handled = false;
    items::random_tick("minecraft:the_end", SIFTSLATE_GROWTH, p, 0, &mut handled);
    assert!(handled);
    items::tick(&mut g);
    assert_eq!(name(&g).properties["snowy"], "true");
    // Covered by a full block: back to siftslate.
    g.world.blocks.insert([9, 65, 9], r.default_state("the_sift:siftslate").unwrap());
    items::random_tick("minecraft:the_end", SIFTSLATE_GROWTH, p, 0, &mut handled);
    items::tick(&mut g);
    assert_eq!(name(&g).name, "the_sift:siftslate");
}

#[test]
fn the_sift_plays_its_biome_music_and_keeps_its_beds() {
    use crate::game::the_sift as g;
    let mut game = Game::new(true);
    assert_eq!(g::in_sift(&game), true, "the mock world is The Sift");
    game.mock_biome = "the_sift:coral_meadow".into();
    let (m, v) = g::sift_music(&game).unwrap();
    let m = m.unwrap();
    assert_eq!((m.sound.as_str(), m.min_delay, m.max_delay, m.replace_current_music, v),
        ("the_sift:music.biome.coral", 12000, 24000, true, 1.0));
    game.mock_biome = "the_sift:overgrown_jungle".into();
    assert_eq!(g::sift_music(&game).unwrap().0.unwrap().sound, "the_sift:music.biome.overgrown");
    assert!(g::sift_beds_safe(&game), "beds don't blow up in The Sift");
    game.boss = true;
    assert!(g::sift_music(&game).is_none(), "a boss's music wins");
    game.boss = false;
    game.dim = Dim::Overworld;
    assert!(g::sift_music(&game).is_none() && !g::sift_beds_safe(&game), "elsewhere, vanilla's");
}
