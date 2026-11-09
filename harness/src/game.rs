use horizon_client::Result;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dim { Overworld, Nether, End }
impl Dim {
    fn id(self) -> &'static str {
        match self { Self::Overworld => "minecraft:overworld", Self::Nether => "minecraft:the_nether", Self::End => "minecraft:the_end" }
    }
}
/// horizon-world's level dimension (the vanilla block library's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelDimension { Overworld, Nether, End }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockStateId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct BlockState { pub name: String, pub properties: BTreeMap<String, String> }

/// The block table: every state the tests use.
pub struct Registry { pub states: Vec<BlockState> }
impl Registry {
    pub fn default_state(&self, name: &str) -> Option<BlockStateId> {
        self.states.iter().position(|s| s.name == name).map(|i| BlockStateId(i as u32))
    }
    pub fn state(&self, id: BlockStateId) -> Option<&BlockState> { self.states.get(id.0 as usize) }
    pub fn resolve(&self, d: &BlockState) -> std::io::Result<BlockStateId> {
        self.states.iter().position(|s| s == d).map(|i| BlockStateId(i as u32))
            .ok_or_else(|| std::io::Error::other("no such state"))
    }
}
pub fn registry() -> &'static Registry {
    let st = |n: &str, p: &[(&str, &str)]| BlockState {
        name: n.into(), properties: p.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() };
    Box::leak(Box::new(Registry { states: vec![
        st("minecraft:air", &[]),
        st("the_sift:siftslate", &[]),
        st("the_sift:reinforced_siftslate", &[]),
        st("the_sift:sift_portal", &[("axis", "x")]),
        st("the_sift:sift_portal", &[("axis", "z")]),
        st("the_sift:healthy_sculk", &[]),
        st("minecraft:reinforced_deepslate", &[]),
        st("the_sift:sonorous_deepslate", &[("mode", "horn")]),
        st("the_sift:sonorous_deepslate", &[("mode", "note")]),
        st("minecraft:note_block", &[("note", "0")]),
        st("minecraft:glow_lichen", &[]),
        st("minecraft:deepslate_tiles", &[]),
        st("the_sift:overgrown_willow_sapling", &[]),
        st("the_sift:ash_willow_sapling", &[]),
        st("the_sift:overgrown_willow_log", &[("axis", "y")]),
        st("the_sift:ash_willow_log", &[("axis", "y")]),
        st("the_sift:overgrown_willow_foliage", &[]),
        st("the_sift:ash_willow_foliage", &[]),
        st("the_sift:overgrown_willow_vines", &[]),
        st("the_sift:ash_willow_vines", &[]),
        st("the_sift:blubby_berries_bush", &[("grown", "false"), ("half", "lower")]),
        st("minecraft:cauldron", &[]),
        st("the_sift:sculkflower_crop", &[("age", "0")]),
        st("the_sift:sculkflower_crop", &[("age", "1")]),
        st("the_sift:sculkflower", &[]),
        st("the_sift:royal_sculk", &[]),
        st("the_sift:soulshale", &[]),
        st("the_sift:siftslate_growth", &[("snowy", "false")]),
        st("the_sift:siftslate_growth", &[("snowy", "true")]),
        st("the_sift:soulshale_growth", &[("snowy", "false")]),
        st("the_sift:ichor_snow", &[]),
        st("minecraft:water", &[]),
        st("minecraft:farmland", &[("moisture", "7")]),
        st("the_sift:ichor_cauldron", &[("level", "1")]),
        st("the_sift:ichor_cauldron", &[("level", "2")]),
        st("the_sift:ichor_cauldron", &[("level", "3")]),
        st("the_sift:blubby_berries_bush", &[("grown", "true"), ("half", "lower")]),
    ].into_iter().chain((1..25).map(|n| st("minecraft:note_block", &[("note", &n.to_string())]))).collect() }))
}

pub struct Assets { pub registry: &'static Registry }
pub struct Pack { pub assets: Assets }
pub struct Facts { pub names: Vec<String> }
pub struct FactRow { pub name: Arc<str>, pub can_occlude: bool, pub full_collision: bool }
impl Facts {
    pub fn is_air(&self, s: BlockStateId) -> bool { self.names[s.0 as usize] == "minecraft:air" }
    pub fn get(&self, s: BlockStateId) -> FactRow {
        let name = self.names[s.0 as usize].as_str();
        let full = matches!(name, "the_sift:siftslate" | "the_sift:soulshale" | "minecraft:deepslate_tiles" | "the_sift:reinforced_siftslate");
        FactRow { name: name.into(), can_occlude: full, full_collision: full }
    }
}
pub struct Res { pub facts: Facts }
impl Res { pub fn facts(&self) -> &Facts { &self.facts } }
pub struct World { pub blocks: HashMap<[i32; 3], BlockStateId>, pub day_time: i64, pub game_time: i64, pub chunks: Loaded }
/// The loaded chunks: those within 16 chunks of the origin here (`unloaded` takes some away).
#[derive(Default)]
pub struct Loaded { pub unloaded: std::collections::HashSet<(i32, i32)> }
impl Loaded {
    pub fn contains_key(&self, c: &(i32, i32)) -> bool { c.0.abs() <= 16 && c.1.abs() <= 16 && !self.unloaded.contains(c) }
}
impl World {
    pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId { *self.blocks.get(&[x, y, z]).unwrap_or(&BlockStateId(0)) }
    pub fn top_y(&self) -> i32 { 256 }
    pub fn bottom_y(&self) -> i32 { 0 }
}
pub struct Biomes(pub HashSet<String>);
impl Biomes { pub fn contains(&self, n: &str) -> bool { self.0.contains(n) } }

/// The player's motion, as far as ichor reaches it.
pub struct PlayerFluids { pub was_touching_water: bool }
pub struct PlayerMotion {
    pub velocity: horizon_core::geometry::Vec3, pub fluids: PlayerFluids, pub position: horizon_core::geometry::Vec3,
    pub jumping: bool, pub flying: bool, pub fall_flying: bool, pub ground: GroundState,
}
#[derive(Default, Clone, Copy, Debug)]
pub struct GroundState { pub on_ground: bool }
/// An item stack, as far as the armour slots go.
#[derive(Clone, Debug)]
pub struct ItemStack { pub item: std::sync::Arc<str>, pub count: u32 }
impl ItemStack { pub fn new(item: &str, count: u32) -> Self { Self { item: item.into(), count } } }

/// The sound engine: what was played.
#[derive(Default)]
pub struct Audio { pub played: Vec<(String, Option<[f64; 3]>, f32, f32)> }
impl Audio {
    pub fn play(&mut self, event: &str, at: Option<[f64; 3]>, volume: f32, pitch: f32) { self.played.push((event.to_owned(), at, volume, pitch)); }
}
/// The particle engine: what was added.
pub struct Particles { pub random: horizon_core::random::LegacyRandom, pub added: Vec<(horizon_client::particles::ParticleOptions, [f64; 3], [f64; 3])> }
impl Particles {
    pub fn level_random(&mut self) -> &mut horizon_core::random::LegacyRandom { &mut self.random }
    pub fn add(&mut self, o: &horizon_client::particles::ParticleOptions, p: [f64; 3], v: [f64; 3]) { self.added.push((o.clone(), p, v)); }
}
#[derive(Default, Debug, Clone)]
pub struct Extra { pub using_item: i32 }
/// A mob, as far as the singer reaches it.
#[derive(Default, Debug, Clone)]
pub struct Mob {
    pub id: u32, pub kind: String, pub position: [f64; 3], pub removed: bool, pub persistent: bool, pub no_ai: bool,
    pub yaw: f32, pub previous_yaw: f32, pub body_yaw: f32, pub previous_body_yaw: f32, pub head_yaw: f32, pub previous_head_yaw: f32,
    pub extra: Extra,
}
pub struct Level { pub seed: i64 }
pub mod creatures {
    #[derive(Debug, Clone)]
    pub(crate) struct NativeSound { pub dimension: &'static str, pub sound: String, pub at: [f64; 3], pub source: i32, pub volume: f32, pub pitch: f32 }
    #[derive(Default)]
    pub struct Creatures { pub(crate) lan_sounds: Vec<NativeSound> }
}
pub mod lan_particles {
    pub(crate) struct QueuedParticle {
        pub dimension: &'static str, pub options: horizon_client::particles::ParticleOptions, pub state: Option<super::BlockStateId>,
        pub at: [f64; 3], pub spread: [f64; 3], pub speed: [f64; 3], pub count: i32, pub randomization: i32,
    }
}
pub mod native_mobs {
    #[derive(Default, Clone, Debug)]
    pub(crate) struct NativeSpawn {
        pub(crate) name: Option<String>, pub(crate) health: Option<f32>, pub(crate) no_ai: bool, pub(crate) invisible: bool,
        pub(crate) silent: bool, pub(crate) invulnerable: bool, pub(crate) no_gravity: bool,
    }
}
pub mod native_world_draw {
    /// The world canvas: its pose is a stack of translations here.
    #[derive(Default)]
    pub struct NativeWorldDraw { pub partial: f32, pub camera: [f64; 3], pub stack: Vec<[f64; 3]>, pub at: [f64; 3] }
    impl NativeWorldDraw {
        pub fn push(&mut self) { self.stack.push(self.at); }
        pub fn pop(&mut self) { self.at = self.stack.pop().unwrap_or([0.0; 3]); }
        pub fn translate_world(&mut self, p: [f64; 3]) -> bool { self.at = [p[0] - self.camera[0], p[1] - self.camera[1], p[2] - self.camera[2]]; p.iter().all(|v| v.is_finite()) }
    }
}
/// A world quad drawn: its sprite, corners (world), uv and colour.
pub type Quad = (String, [[f64; 3]; 4], [f32; 4], [u8; 4]);
pub struct PotionState { pub effects: horizon_client::effects::PlayerEffects }

pub struct Game<'a> {
    pub remote: Option<()>,
    pub player: Option<PlayerMotion>,
    pub potions: PotionState,
    pub sections_changed: u32,
    pub dim: Dim,
    pub title: Option<()>,
    pub win: Option<()>,
    pub boss: bool,
    pub mock_biome: String,
    pub world_biomes: Option<Arc<Biomes>>,
    pub pack: Pack,
    pub res: Res,
    pub world: World,
    pub rain: f32,
    pub portals_told: Vec<(String, [i32; 3])>,
    pub audio: Audio,
    /// Armor slots: head, chest, legs, feet.
    pub armor: [Option<ItemStack>; 4],
    pub creatures: creatures::Creatures,
    /// Whether this is the world's server (single player's runs one; the window is its client).
    pub authoritative: bool,
    pub particles: Particles,
    pub mobs: Vec<Mob>,
    pub level: Level,
    pub level_events: Vec<(i32, [i32; 3], i32)>,
    pub quads: std::cell::RefCell<Vec<Quad>>,
    pub fills: std::cell::RefCell<Vec<([f32; 4], u32)>>,
    pub data_dir: std::path::PathBuf,
    /// Block.popResource's drops.
    pub popped: Vec<([i32; 3], ItemStack)>,
    /// The held stack, whether in creative, and what was given besides.
    pub hand: Option<ItemStack>, pub creative: bool, pub given: Vec<ItemStack>,
    pub _p: std::marker::PhantomData<&'a ()>,
}
impl Game<'_> {
    pub fn new(sift: bool) -> Self {
        let registry = registry();
        let names = registry.states.iter().map(|s| s.name.clone()).collect();
        Game {
            remote: None,
            player: Some(PlayerMotion { velocity: horizon_core::geometry::Vec3::new(0.2, -0.1, 0.2), fluids: PlayerFluids { was_touching_water: false }, position: horizon_core::geometry::Vec3::new(0.5, 64.0, 0.5),
                jumping: false, flying: false, fall_flying: false, ground: GroundState { on_ground: true } }),
            armor: [None, None, None, None],
            potions: PotionState { effects: Default::default() },
            sections_changed: 0,
            dim: Dim::End,
            title: None,
            win: None,
            boss: false,
            mock_biome: "the_sift:sift_wastes".into(),
            world_biomes: Some(Arc::new(Biomes(if sift { ["the_sift:sift_wastes".to_owned()].into() } else { HashSet::new() }))),
            pack: Pack { assets: Assets { registry } },
            res: Res { facts: Facts { names } },
            world: World { blocks: HashMap::new(), day_time: 6000, game_time: 0, chunks: Loaded::default() },
            rain: 0.0,
            portals_told: Vec::new(),
            audio: Audio::default(),
            creatures: Default::default(),
            authoritative: true,
            particles: Particles { random: horizon_core::random::LegacyRandom::new(7), added: Vec::new() },
            mobs: Vec::new(),
            level: Level { seed: 42 },
            level_events: Vec::new(),
            quads: Default::default(),
            fills: Default::default(),
            popped: Vec::new(), hand: None, creative: false, given: Vec::new(),
            data_dir: std::env::temp_dir().join(format!("the-sift-harness-{}-{:?}", std::process::id(), std::thread::current().id())),
            _p: Default::default(),
        }
    }
    fn held(&self) -> Option<&ItemStack> { self.hand.as_ref() }
    fn held_mut(&mut self) -> &mut Option<ItemStack> { &mut self.hand }
    /// ItemUtils.createFilledResult: the held stack gives one for `filled` (creative keeps it).
    fn filled_result(&mut self, filled: ItemStack) {
        if self.creative { self.given.push(filled); return; }
        match self.hand.as_mut() {
            Some(h) if h.count > 1 => { h.count -= 1; self.given.push(filled); }
            _ => self.hand = Some(filled),
        }
    }
    fn pop_resource(&mut self, pos: [i32; 3], stack: ItemStack) { self.popped.push((pos, stack)); }
    fn level_set(&mut self, pos: [i32; 3], state: BlockStateId) -> Result<()> {
        self.world.blocks.insert(pos, state);
        Ok(())
    }
    fn block_name(&self, p: [i32; 3]) -> Arc<str> { self.res.facts().get(self.world.block(p[0], p[1], p[2])).name }
    fn native_portal_create(&mut self, dimension: &str, pos: [i32; 3]) { self.portals_told.push((dimension.into(), pos)); }
    /// Every section meshed again (LevelRenderer.allChanged).
    fn all_sections_changed(&mut self) { self.sections_changed += 1; }
    /// Each player of the world in turn (one here).
    fn for_each_player(&mut self, mut f: impl FnMut(&mut Self)) { if self.player.is_some() { f(self); } }
    fn weather_levels(&self, _partial: f32) -> (f32, f32) { (self.rain, 0.0) }
    fn native_authoritative(&self) -> bool { self.authoritative }
    fn lan_send_particles(&mut self, q: lan_particles::QueuedParticle) {
        let _ = (q.dimension, q.state, q.speed, q.count, q.randomization);
        self.particles.added.push((q.options, q.at, q.spread));
    }
    fn level_event_at(&mut self, id: i32, pos: [i32; 3], data: i32) { self.level_events.push((id, pos, data)); }
    fn particle_sprite(&self, state: BlockStateId, _pos: [i32; 3]) -> Option<(String, u32)> { Some((format!("block/{}", state.0), 0xFFFFFF)) }
    fn native_spawn_mob(&mut self, dimension: &str, kind: &str, pos: [f64; 3], options: &native_mobs::NativeSpawn) -> Option<u32> {
        if dimension != self.dim.id() { return None; }
        let id = 1000 + self.mobs.len() as u32;
        self.mobs.push(Mob { id, kind: kind.to_owned(), position: pos, no_ai: options.no_ai, ..Default::default() });
        Some(id)
    }
    fn native_remove_mob(&mut self, dimension: &str, id: u32) -> bool {
        if dimension != self.dim.id() { return false; }
        let Some(m) = self.mobs.iter_mut().find(|m| m.id == id && !m.removed) else { return false };
        m.removed = true;
        true
    }
    fn native_world_quad(&self, draw: &mut native_world_draw::NativeWorldDraw, sprite: &str, corners: [[f32; 3]; 4], uv: [f32; 4], color: [u8; 4]) -> bool {
        let c = corners.map(|p| [draw.at[0] + f64::from(p[0]), draw.at[1] + f64::from(p[1]), draw.at[2] + f64::from(p[2])]);
        self.quads.borrow_mut().push((sprite.to_owned(), c, uv, color));
        true
    }
    fn native_fill(&self, _batch: &mut horizon_client::hud::HudBatch, rect: [f32; 4], argb: u32) { self.fills.borrow_mut().push((rect, argb)); }
    fn native_data_dir(&self, mod_id: &str) -> std::path::PathBuf { self.data_dir.join(mod_id) }
    fn acting_player_uuid(&mut self) -> [i32; 4] { [1, 2, 3, 4] }
    pub(crate) fn boss_music(&self) -> bool { self.boss }
    pub(crate) fn eye_position(&self) -> Option<[f64; 3]> { self.player.as_ref().map(|p| [p.position.x, p.position.y + 1.62, p.position.z]) }
    pub(crate) fn biome_at(&self, _p: [f64; 3]) -> &str { &self.mock_biome }
    pub fn name_at(&self, p: [i32; 3]) -> String { self.block_name(p).to_string() }
    pub fn state_at(&self, p: [i32; 3]) -> BlockState { self.pack.assets.registry.state(self.world.block(p[0], p[1], p[2])).unwrap().clone() }
}
pub(crate) mod the_sift {
    include!("../../mixins/the_sift_native/glue.rs");
}
pub(crate) mod the_sift_portal {
    include!("../../mixins/the_sift_native/portal.rs");
}
#[cfg(test)]
mod tests;
pub(crate) mod the_sift_items {
    include!("../../mixins/the_sift_native/items.rs");
}
#[cfg(test)]
mod portal_tests;
#[cfg(test)]
mod items_tests;
