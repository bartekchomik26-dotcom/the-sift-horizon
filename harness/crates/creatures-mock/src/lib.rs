extern crate self as horizon_client;


pub mod items {
    #[derive(Clone, Debug)]
    pub struct ItemStack { pub item: std::sync::Arc<str>, pub count: u32 }
}

pub mod mobs {
    use horizon_core::geometry::Vec3;
    use std::sync::Arc;
    pub struct MobType { pub id: String }
    #[derive(Default)]
    pub struct Extra { pub sitting: bool, pub tame: bool, pub owner: Option<[i32; 4]>, pub main_hand: Option<Arc<str>>, pub using_item: i32 }
    #[derive(Default)]
    pub struct Fluids { pub in_water: bool }
    #[derive(Default)]
    pub struct Ai { pub swinging: bool, pub swing_time: i32, pub swing_duration: i32, pub target: Option<u32>, pub jumping: bool }
    pub struct Mob {
        pub id: u32, pub kind: Arc<MobType>, pub extra: Extra, pub fluids: Fluids, pub velocity: Vec3, pub ai: Ai,
        pub random: horizon_core::random::LegacyRandom, pub persistent: bool, pub position: Vec3, pub dead: bool,
    }
    impl Mob {
        pub fn new(id: u32, kind: &str) -> Mob {
            Mob { id, kind: Arc::new(MobType { id: kind.into() }), extra: Extra::default(), fluids: Fluids::default(),
                velocity: Vec3::new(0.0, 0.0, 0.0), ai: Ai::default(), random: horizon_core::random::LegacyRandom::new(id as i64),
                persistent: false, position: Vec3::new(0.5, 64.0, 0.5), dead: false }
        }
        pub fn alive(&self) -> bool { !self.dead }
        pub fn center(&self) -> [f64; 3] { [self.position.x, self.position.y + 0.5, self.position.z] }
        pub fn nav_stop(&mut self) {}
    }
    #[derive(Debug)]
    pub enum MobEvent {
        SoundId(String, [f64; 3], f32, f32),
        DropItem { mob: u32, item: Arc<str>, count: u32, at: [f64; 3] },
    }
    pub mod ai {
        pub mod the_sift_native {
            pub fn barter(r: &mut impl horizon_core::random::RandomSource) -> (&'static str, u32) {
                let _ = r.next_int_bound(17);
                ("minecraft:echo_shard", 2)
            }
        }
    }
}

pub mod game {
    use crate::items::ItemStack;
    use crate::mobs::{Mob, MobEvent};
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    pub mod native_block_entity_draw {
        pub struct BlockEntityDraw { pub light: u32, pub quads: Vec<(String, [[f32; 3]; 4], [f32; 4], u32, u32)> }
    }
    pub mod native_mob_draw {
        #[derive(Clone, Debug)]
        pub struct NativeMobRender<'a> {
            pub id: u32, pub kind: &'a str, pub partial: f32, pub age: f32, pub walk_speed: f32,
            pub hurt_time: i32, pub death_time: i32, pub on_ground: bool,
        }
    }
    pub mod native_mobs {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub(crate) struct NativeSpawn { pub(crate) no_ai: bool }
    }
    pub mod the_sift {
        pub(crate) fn in_sift(g: &super::Game<'_>) -> bool { g.in_sift }
        pub(crate) fn singer_song(_at: [f64; 3]) -> Option<i32> { None }
    }
    pub struct Facts;
    impl Facts { pub fn is_air(&self, s: u32) -> bool { s == 0 } }
    pub struct Res(Facts);
    impl Res { pub fn facts(&self) -> &Facts { &self.0 } }
    pub struct World {
        pub random: horizon_core::random::LegacyRandom, pub game_time: i64, pub modified: HashSet<(i32, i32)>,
        pub chunks: HashMap<(i32, i32), ()>, pub ground: i32,
    }
    impl World {
        pub fn top_y(&self) -> i32 { 256 }
        pub fn bottom_y(&self) -> i32 { 0 }
        pub fn block(&self, _x: i32, y: i32, _z: i32) -> u32 { (y < self.ground) as u32 }
    }
    pub struct Player { pub position: horizon_core::geometry::Vec3 }
    pub struct Game<'a> {
        pub mobs: Vec<Mob>, pub remote: Option<()>, pub creative: bool, pub world: World, pub res: Res,
        pub player: Option<Player>, pub in_sift: bool, pub held: Option<ItemStack>, pub events: Vec<MobEvent>,
        pub particles: Vec<(u32, i8)>, pub spawned: Vec<(String, [f64; 3])>, pub _p: std::marker::PhantomData<&'a ()>,
    }
    impl Game<'_> {
        pub fn new() -> Self {
            Game { mobs: Vec::new(), remote: None, creative: false,
                world: World { random: horizon_core::random::LegacyRandom::new(7), game_time: 0, modified: HashSet::new(),
                    chunks: (-8..8).flat_map(|x| (-8..8).map(move |z| ((x, z), ()))).collect(), ground: 64 },
                res: Res(Facts), player: Some(Player { position: horizon_core::geometry::Vec3::new(0.5, 64.0, 0.5) }),
                in_sift: true, held: None, events: Vec::new(), particles: Vec::new(), spawned: Vec::new(), _p: Default::default() }
        }
        /// ServerLevel.players() in turn (here the one player, in place).
        pub(super) fn for_each_player(&mut self, mut f: impl FnMut(&mut Self)) {
            if self.player.is_some() {
                f(self);
            }
        }
        pub(super) fn native_draw_quad(&self, draw: &mut native_block_entity_draw::BlockEntityDraw, sprite: &str,
            corners: [[f32; 3]; 4], uv: [f32; 4], tint: u32) -> bool {
            draw.quads.push((sprite.into(), corners, uv, tint, draw.light));
            true
        }
        pub(super) fn acting_player_uuid(&mut self) -> [i32; 4] { [1, 2, 3, 4] }
        pub(super) fn mob_event_particles(&mut self, id: u32, event: i8) { self.particles.push((id, event)); }
        fn held_slot(&self) -> usize { 0 }
        pub(super) fn stack_in_mut(&mut self, _slot: usize) -> &mut Option<ItemStack> { &mut self.held }
        pub(super) fn handle_mob_events(&mut self, events: Vec<MobEvent>) -> Result<(), ()> { self.events.extend(events); Ok(()) }
        pub(super) fn biome_name_at(&self, _p: horizon_core::geometry::Vec3) -> Option<String> { Some("the_sift:overgrown_forest".into()) }
        pub(super) fn native_spawn_mob(&mut self, _dim: &str, kind: &str, pos: [f64; 3], _o: &native_mobs::NativeSpawn) -> Option<u32> {
            self.spawned.push((kind.into(), pos));
            let id = 100 + self.mobs.len() as u32;
            let mut m = Mob::new(id, kind);
            m.position = horizon_core::geometry::Vec3::new(pos[0], pos[1], pos[2]);
            self.mobs.push(m);
            Some(id)
        }
        pub(super) fn block_name(&self, p: [i32; 3]) -> Arc<str> {
            if p[1] < self.world.ground { "the_sift:siftslate_growth".into() } else { "minecraft:air".into() }
        }
    }
    pub mod the_sift_creatures { include!("../../../../mixins/the_sift_native/creatures.rs"); }
}
