pub mod projectiles {
    use horizon_core::geometry::Vec3;
    use crate::mobs::Who;
    #[derive(Clone, Debug, PartialEq)]
    pub enum ProjectileKind { Arrow { base_damage: f64, critical: bool, effect: Option<(&'static str, i32)> }, SmallFireball }
    #[derive(Clone, Debug)]
    pub struct Projectile { pub kind: ProjectileKind, pub position: Vec3, pub owner: Option<Who>, pub velocity: Vec3 }
    impl Projectile {
        pub fn new(kind: ProjectileKind, position: Vec3, owner: Option<Who>) -> Self { Self { kind, position, owner, velocity: Vec3::new(0.0, 0.0, 0.0) } }
        pub fn shoot(&mut self, _r: &mut impl horizon_core::random::RandomSource, dir: [f64; 3], speed: f32, _u: f32) {
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-9);
            self.velocity = Vec3::new(dir[0] / l * f64::from(speed), dir[1] / l * f64::from(speed), dir[2] / l * f64::from(speed));
        }
        pub fn fireball(kind: ProjectileKind, position: Vec3, owner: Option<Who>, dir: [f64; 3]) -> Self {
            let mut p = Self::new(kind, position, owner);
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-9);
            p.velocity = Vec3::new(dir[0] / l * 0.1, dir[1] / l * 0.1, dir[2] / l * 0.1);
            p
        }
    }
    pub fn own_random(m: &crate::mobs::Mob) -> horizon_core::random::LegacyRandom { horizon_core::random::LegacyRandom::new(m.id as i64) }
}
pub mod mobs {
    use horizon_core::geometry::Vec3;
    use std::sync::Arc;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Who { Player, Mob(u32) }
    #[derive(Clone, Copy, Debug)]
    pub struct TargetInfo { pub position: Vec3, pub eye_y: f64, pub alive: bool, pub is_player: bool, pub creative: bool }
    pub mod witch {
        use super::{Mob, Who};
        use super::goals::{Cx, Goal};
        pub type Fire = fn(&mut Mob, Who, f32, &mut Cx);
        /// RangedAttackGoal(speed, intervalMin, intervalMax, radius): fires once a check here.
        #[derive(Clone, Debug)]
        pub struct RangedAttack { pub speed: f64, pub min: i32, pub max: i32, pub radius: f32, pub fire: Fire }
        impl RangedAttack { pub fn new(speed: f64, min: i32, max: i32, radius: f32, fire: Fire) -> Self { Self { speed, min, max, radius, fire } } }
        impl Goal for RangedAttack {
            fn flags(&self) -> u8 { 3 }
            fn can_use(&mut self, m: &mut Mob, _: &mut Cx) -> bool { m.ai.target.is_some() }
            fn tick(&mut self, m: &mut Mob, cx: &mut Cx) { if let Some(w) = m.ai.target { (self.fire)(m, w, 1.0, cx) } }
            fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) }
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct Flags { pub burns_in_day: bool, pub undead: bool }
    #[derive(Clone, Debug)]
    pub struct MobType {
        pub id: String, pub width: f64, pub height: f64, pub eye_height: f64, pub category: String,
        pub sounds: [&'static str; 3], pub step_sound: Option<&'static str>, pub ambient_interval: i32, pub flags: Flags,
    }
    impl MobType {
        /// A content mod's living type over the zombie, as Horizon loads it.
        pub fn zombie(id: &str) -> Self {
            MobType { id: id.into(), width: 0.6, height: 1.95, eye_height: 1.74, category: "monster".into(),
                sounds: ["entity.zombie.ambient", "entity.zombie.hurt", "entity.zombie.death"],
                step_sound: Some("entity.zombie.step"), ambient_interval: 80, flags: Flags { burns_in_day: true, undead: true } }
        }
    }
    #[derive(Default, Debug)]
    pub struct Extra {
        pub variant: u8, pub pick_up_loot: bool, pub can_break_doors: bool, pub main_hand: Option<Arc<str>>,
        pub off_hand: Option<Arc<str>>, pub head: Option<Arc<str>>, pub chest: Option<Arc<str>>, pub legs: Option<Arc<str>>,
        pub feet: Option<Arc<str>>, pub using_item: i32, pub sitting: bool, pub age: i32,
    }
    impl Extra { pub fn set_main_hand(&mut self, item: Option<Arc<str>>) { self.main_hand = item; } }
    #[derive(Default, Debug)]
    pub struct Nav { pub can_open_doors: bool, pub moving_to: Option<[f64; 3]> }
    impl Nav { pub fn is_done(&self) -> bool { self.moving_to.is_none() } }
    #[derive(Default, Debug)]
    pub struct MoveControl { pub wanted: [f64; 3], pub speed_modifier: f64, pub has: bool }
    impl MoveControl { pub fn has_wanted(&self) -> bool { self.has } }
    #[derive(Default, Debug)]
    pub struct LookControl { pub at: Option<[f64; 3]> }
    impl LookControl { pub fn set_look_at(&mut self, at: [f64; 3], _y: f32, _x: f32) { self.at = Some(at); } }
    #[derive(Default)]
    pub struct Ai { pub goals: goals::GoalSelector, pub targets: goals::GoalSelector, pub nav: Nav, pub mov: MoveControl, pub look: LookControl, pub target: Option<Who> }
    pub struct Mob {
        pub id: u32, pub kind: Arc<MobType>, pub extra: Extra, pub ai: Ai, pub on_ground: bool, pub velocity: Vec3,
        pub position: Vec3, pub random: horizon_core::random::LegacyRandom, pub dead: bool,
        pub health: f32, pub hurt_time: i32, pub age: u32,
    }
    impl Mob {
        pub fn new(id: u32, kind: Arc<MobType>) -> Mob {
            Mob { id, kind, extra: Extra::default(), ai: Ai::default(), on_ground: true, velocity: Vec3::new(0.0, 0.0, 0.0),
                position: Vec3::new(0.5, 64.0, 0.5), random: horizon_core::random::LegacyRandom::new(id as i64), dead: false,
                health: 10.0, hurt_time: 0, age: 0 }
        }
        pub fn alive(&self) -> bool { !self.dead }
        pub fn is_baby(&self) -> bool { self.extra.age < 0 }
        pub fn center(&self) -> [f64; 3] { [self.position.x, self.position.y + 0.5, self.position.z] }
        pub fn nav_stop(&mut self) { self.ai.nav.moving_to = None; }
        pub fn look_at_speed(&mut self, at: [f64; 3], _y: f32, _x: f32) { self.ai.look.at = Some(at); }
        pub fn nav_move_to(&mut self, _w: &MobWorld, at: [f64; 3], _speed: f64) -> bool { self.ai.nav.moving_to = Some(at); true }
    }
    #[derive(Clone, Debug)]
    pub struct Seen { pub id: u32, pub alive: bool, pub variant: u8, pub position: Vec3 }
    pub struct PlayerView { pub position: Vec3, pub eye: [f64; 3] }
    pub struct MobWorld<'a> {
        pub others: &'a [Seen],
        pub player: Option<PlayerView>,
        pub blocks: &'a std::collections::HashMap<[i32; 3], &'static str>,
    }
    impl MobWorld<'_> {
        pub fn target_info(&self, who: Who) -> Option<TargetInfo> {
            match who {
                Who::Player => self.player.as_ref().map(|p| TargetInfo { position: p.position, eye_y: p.eye[1], alive: true, is_player: true, creative: false }),
                Who::Mob(id) => self.others.iter().find(|s| s.id == id).map(|s| TargetInfo { position: s.position, eye_y: s.position.y + 1.0, alive: s.alive, is_player: false, creative: false }),
            }
        }
        pub fn block_name(&self, x: i32, y: i32, z: i32) -> &str { self.blocks.get(&[x, y, z]).copied().unwrap_or("minecraft:air") }
    }
    #[derive(Debug)]
    pub enum MobEvent {
        SoundId(String, [f64; 3], f32, f32),
        AttackPlayer(f32, [f64; 3]),
        SetVelocity { target: Who, velocity: [f64; 3] },
        Projectile(crate::projectiles::Projectile),
        DropItem { mob: u32, item: Arc<str>, count: u32, at: [f64; 3] },
        SetBlockName { mob: Option<u32>, pos: [i32; 3], name: &'static str, effect: bool, flags: i32 },
    }
    pub mod finalize {
        pub fn set_baby(m: &mut super::Mob, baby: bool) { m.extra.age = if baby { -24000 } else { 0 }; }
    }
    pub mod goals {
        use super::{Mob, MobEvent, MobWorld, Seen};
        use std::sync::Arc;
        pub const MOVE: u8 = 1;
        pub const LOOK: u8 = 2;
        pub struct Cx<'a, 'w> { pub w: &'a MobWorld<'w>, pub lr: &'a mut horizon_core::random::LegacyRandom, pub events: &'a mut Vec<MobEvent> }
        pub trait Goal: std::fmt::Debug {
            fn flags(&self) -> u8;
            fn can_use(&mut self, m: &mut Mob, cx: &mut Cx) -> bool;
            fn can_continue(&mut self, m: &mut Mob, cx: &mut Cx) -> bool { self.can_use(m, cx) }
            fn start(&mut self, _m: &mut Mob, _cx: &mut Cx) {}
            fn stop(&mut self, _m: &mut Mob, _cx: &mut Cx) {}
            fn tick(&mut self, _m: &mut Mob, _cx: &mut Cx) {}
            fn every_tick(&self) -> bool { false }
            fn box_clone(&self) -> Box<dyn Goal>;
            fn name(&self) -> &'static str { std::any::type_name::<Self>().rsplit("::").next().unwrap_or("Goal") }
        }
        #[derive(Default)]
        pub struct GoalSelector { pub goals: Vec<(i32, Box<dyn Goal>)> }
        impl GoalSelector {
            pub fn add(&mut self, priority: i32, goal: impl Goal + 'static) { self.goals.push((priority, Box::new(goal))); }
            pub fn clear(&mut self) { self.goals.clear(); }
            pub fn names(&self) -> Vec<(i32, &'static str)> { self.goals.iter().map(|(p, g)| (*p, g.name())).collect() }
        }
        macro_rules! simple { ($($t:ident),*) => { $(
            #[derive(Clone, Debug, Default)] pub struct $t;
            impl Goal for $t { fn flags(&self) -> u8 { 0 } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } }
        )* } }
        simple!(Float, RandomLookAround);
        macro_rules! with_new { ($($t:ident ($($a:ident: $ty:ty),*)),*) => { $(
            #[derive(Clone, Debug)] pub struct $t { $(pub $a: $ty),* }
            impl $t { pub fn new($($a: $ty),*) -> Self { Self { $($a),* } } }
            impl Goal for $t { fn flags(&self) -> u8 { 0 } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } }
        )* } }
        with_new!(LookAtPlayer(range: f32), MeleeAttack(speed: f64, follow_unseen: bool), Tempt(speed: f64, items: Arc<[String]>, can_scare: bool));
        #[derive(Clone, Debug)]
        pub struct Stroll { pub speed: f64, pub interval: i32, pub water_avoiding: bool }
        impl Stroll {
            pub fn avoiding_water(speed: f64) -> Self { Stroll { speed, interval: 120, water_avoiding: true } }
            pub fn interval(mut self, i: i32) -> Self { self.interval = i; self }
        }
        impl Goal for Stroll { fn flags(&self) -> u8 { MOVE } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } }
        #[derive(Clone, Debug)]
        pub struct HurtByTarget { pub alert: bool, pub alert_if: Option<fn(&Seen) -> bool> }
        impl HurtByTarget {
            pub fn new() -> Self { HurtByTarget { alert: false, alert_if: None } }
            pub fn alert(mut self) -> Self { self.alert = true; self }
        }
        impl Goal for HurtByTarget { fn flags(&self) -> u8 { 8 } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } }
        #[derive(Clone, Debug)]
        pub struct NearestTarget { pub must_see: bool }
        impl NearestTarget { pub fn player(must_see: bool) -> Self { NearestTarget { must_see } } }
        impl Goal for NearestTarget { fn flags(&self) -> u8 { 8 } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } }
    }
    pub mod tame {
        use super::Mob;
        use super::goals::{Cx, Goal};
        #[derive(Clone, Debug)] struct SitWhenOrdered;
        #[derive(Clone, Debug)] #[allow(dead_code)] struct FollowOwner { speed: f64, start: f64, stop: f64 }
        impl FollowOwner { fn new(speed: f64, start: f64, stop: f64) -> Self { Self { speed, start, stop } } }
        #[derive(Clone, Debug)] #[allow(dead_code)] struct OwnerHurtBy { timestamp: i64 }
        #[derive(Clone, Debug)] #[allow(dead_code)] struct OwnerHurt { timestamp: i64 }
        macro_rules! g { ($($t:ident),*) => { $( impl Goal for $t { fn flags(&self) -> u8 { 0 } fn can_use(&mut self, _: &mut Mob, _: &mut Cx) -> bool { false } fn box_clone(&self) -> Box<dyn Goal> { Box::new(self.clone()) } } )* } }
        g!(SitWhenOrdered, FollowOwner, OwnerHurtBy, OwnerHurt);
        pub mod the_sift_tame { include!("../../../../mixins/the_sift_native/tame.rs"); }
    }
    pub mod ai {
        use super::{Mob, MobEvent, MobWorld};
        pub mod the_sift_native { include!("../../../../mixins/the_sift_native/ai.rs"); }
        include!("../../../../mixins/the_sift_native/h_goals.rs");
        include!("../../../../mixins/the_sift_native/h_pre_tick.rs");
        pub fn native_register_goals(mob: &mut Mob) { the_sift_native_goals(mob); }
        pub fn pre_tick(m: &mut Mob, w: &MobWorld, events: &mut Vec<MobEvent>) { the_sift_native_pre_tick(m, w, events); }
    }
    pub mod undead {
        use super::Mob;
        include!("../../../../mixins/the_sift_native/h_reinforce.rs");
        include!("../../../../mixins/the_sift_native/h_convert.rs");
        pub fn after_hurt(m: &mut Mob) -> bool { the_sift_native_reinforce(m).is_continue() }
        pub fn custom_server_ai_step(m: &mut Mob) -> bool { the_sift_native_convert(m).is_continue() }
    }
}
