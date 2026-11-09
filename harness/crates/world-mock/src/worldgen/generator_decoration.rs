//! Where Horizon's `Decoration::decorate` lives: the shipped modules and helper, as the mixin
//! planner places them (modules beside it, the helper as a function of this file).
use super::generator::{GeneratorData, ProtoChunk};
use std::sync::Arc;

#[path = "../../../../../mixins/the_sift_native/features.rs"]
pub mod the_sift_features;
#[path = "../../../../../mixins/the_sift_native/terrain.rs"]
pub mod the_sift_terrain;

include!("../../../../../mixins/the_sift_native/h_decorate.rs");

/// What the HEAD hook does at the start of `Decoration::decorate`.
pub fn decorate_head(data: &Arc<GeneratorData>, seed: i64, region: &mut [ProtoChunk; 9]) {
    the_sift_native_decorate(data, seed, region);
}
