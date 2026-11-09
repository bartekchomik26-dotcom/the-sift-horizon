// Mock of the horizon-client surface The Sift's game half touches (Nightly f319701).
extern crate self as horizon_client;
pub use horizon_client_render::{datapack, environment};
pub type Result<T> = horizon_client_render::Result<T>;
#[macro_export]
macro_rules! native_data {
    ($id:literal, $file:literal $(,)?) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../mixins/", $id, "/data/", $file))
    };
}
pub mod game;
pub mod music {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Music { pub sound: String, pub min_delay: i32, pub max_delay: i32, pub replace_current_music: bool }
    impl Music {
        pub fn new(sound: &str, min_delay: i32, max_delay: i32, replace_current_music: bool) -> Self {
            Self { sound: sound.to_owned(), min_delay, max_delay, replace_current_music }
        }
    }
}
/// The HUD's batch, as far as a full-screen picture reaches it.
pub mod hud {
    pub struct HudBatch { pub width: f32, pub height: f32 }
}
/// Particle options (horizon-client-render's particles.rs, the types The Sift's portal uses).
pub mod particles {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ParticleType { EndRod, Block, Note }
    #[derive(Clone, Debug, PartialEq)]
    pub struct ParticleOptions { pub ty: ParticleType, pub block: Option<(String, u32)>, pub mod_particle: Option<String> }
    impl ParticleOptions {
        pub fn simple(ty: ParticleType) -> Self { Self { ty, block: None, mod_particle: None } }
        pub fn named(name: impl Into<String>) -> Self { Self { ty: ParticleType::Note, block: None, mod_particle: Some(name.into()) } }
    }
}
/// Status effects (horizon-client-render's effects.rs, as far as ichor reaches it).
pub mod effects {
    #[derive(Clone, Debug, PartialEq)]
    pub struct EffectInstance { pub id: &'static str, pub duration: i32, pub amplifier: i32, pub ambient: bool, pub visible: bool }
    impl EffectInstance {
        pub fn new(id: &'static str, duration: i32, amplifier: i32) -> Self {
            Self { id, duration, amplifier: amplifier.clamp(0, 255), ambient: false, visible: true }
        }
    }
    #[derive(Default, Debug)]
    pub struct PlayerEffects { pub active: Vec<EffectInstance> }
    impl PlayerEffects {
        /// `LivingEntity.addEffect`: a longer one replaces a shorter one of the same kind.
        pub fn add(&mut self, e: EffectInstance) -> bool {
            match self.active.iter_mut().find(|a| a.id == e.id) {
                Some(a) if a.duration < e.duration || a.amplifier < e.amplifier => { *a = e; true }
                Some(_) => false,
                None => { self.active.push(e); true }
            }
        }
    }
}
