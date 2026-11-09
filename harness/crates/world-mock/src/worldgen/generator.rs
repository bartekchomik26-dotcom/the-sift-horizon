//! ProtoChunk, its terrain and GeneratorData, as far as the features reach them.
use super::biome::BiomeId;
use crate::registry::{BlockRegistry, BlockStateId};
use horizon_core::position::ChunkPos;
use horizon_core::ticks::ProtoChunkTicks;
use std::sync::Arc;

pub struct GeneratorData {
    pub registry: &'static BlockRegistry,
    biome_names: Vec<Arc<str>>,
}
impl GeneratorData {
    pub fn new(registry: &'static BlockRegistry, biomes: &[&str]) -> Self {
        Self { registry, biome_names: biomes.iter().map(|b| Arc::from(*b)).collect() }
    }
    pub fn registry(&self) -> &'static BlockRegistry { self.registry }
    pub fn biome_names(&self) -> &[Arc<str>] { &self.biome_names }
    pub fn biome_name(&self, id: BiomeId) -> &str { &self.biome_names[id.0 as usize] }
    pub fn biome_id(&self, name: &str) -> Option<BiomeId> {
        self.biome_names.iter().position(|b| &**b == name).map(|i| BiomeId(i as u32))
    }
}

#[derive(Debug)]
pub struct TerrainError;

pub struct TerrainChunk {
    pos: ChunkPos,
    min_y: i32,
    height: i32,
    blocks: Vec<BlockStateId>,
    fluid_ticks: ProtoChunkTicks,
    /// Non-air blocks per section, as Horizon's TerrainBlocks keeps them.
    non_air: Vec<u32>,
    pub writes: usize,
}
impl TerrainChunk {
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let ly = y - self.min_y;
        (0..self.height).contains(&ly).then(|| ((ly * 16 + (z & 15)) * 16 + (x & 15)) as usize)
    }
    pub fn position(&self) -> ChunkPos { self.pos }
    pub fn min_y(&self) -> i32 { self.min_y }
    pub fn height(&self) -> i32 { self.height }
    /// Absolute coordinates; X/Z masked to the chunk, as Horizon's.
    pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        self.index(x, y, z).map_or(BlockStateId(0), |i| self.blocks[i])
    }
    pub fn set_block_worldgen(&mut self, x: i32, y: i32, z: i32, state: BlockStateId) -> Result<BlockStateId, TerrainError> {
        let Some(i) = self.index(x, y, z) else { return Ok(BlockStateId(0)) };
        self.writes += 1;
        let old = std::mem::replace(&mut self.blocks[i], state);
        let section = ((y - self.min_y) >> 4) as usize;
        if old.0 == 0 && state.0 != 0 { self.non_air[section] += 1; }
        if old.0 != 0 && state.0 == 0 { self.non_air[section] -= 1; }
        Ok(old)
    }
    pub fn highest_section_y(&self) -> i32 {
        self.non_air.iter().rposition(|&c| c != 0).map_or(self.min_y, |i| self.min_y + i as i32 * 16)
    }
    pub fn fluid_ticks_mut(&mut self) -> &mut ProtoChunkTicks { &mut self.fluid_ticks }
    pub fn fluid_ticks(&self) -> &ProtoChunkTicks { &self.fluid_ticks }
}

pub struct ProtoChunk {
    pub terrain: TerrainChunk,
    biomes: Vec<BiomeId>,
    data: Arc<GeneratorData>,
    decorated: bool,
    pub block_entities: Vec<([i32; 3], crate::nbt::Compound)>,
}
impl ProtoChunk {
    /// Filled by `column(x, z) -> (blocks bottom-up, biome)` for each of its columns.
    pub fn new(data: Arc<GeneratorData>, pos: ChunkPos, min_y: i32, height: i32, mut column: impl FnMut(i32, i32, &mut [BlockStateId]) -> BiomeId) -> Self {
        let mut blocks = vec![BlockStateId(0); (16 * 16 * height) as usize];
        let mut biomes = vec![BiomeId(0); 16 * 16];
        let mut col = vec![BlockStateId(0); height as usize];
        for lz in 0..16 {
            for lx in 0..16 {
                col.iter_mut().for_each(|b| *b = BlockStateId(0));
                let biome = column(pos.x * 16 + lx, pos.z * 16 + lz, &mut col);
                biomes[(lz * 16 + lx) as usize] = biome;
                for (ly, b) in col.iter().enumerate() {
                    blocks[((ly as i32 * 16 + lz) * 16 + lx) as usize] = *b;
                }
            }
        }
        // Stored per 4x4 quart column, as Horizon's (the biome of the quart's corner column).
        let mut quarts = vec![BiomeId(0); 16];
        for qz in 0..4 { for qx in 0..4 { quarts[qz * 4 + qx] = biomes[(qz * 4 * 16 + qx * 4) as usize]; } }
        let mut non_air = vec![0u32; (height / 16) as usize];
        for (i, b) in blocks.iter().enumerate() {
            if b.0 != 0 { non_air[i / 256 / 16] += 1; }
        }
        ProtoChunk {
            terrain: TerrainChunk { pos, min_y, height, blocks, fluid_ticks: ProtoChunkTicks::default(), non_air, writes: 0 },
            biomes: quarts, data, decorated: false, block_entities: Vec::new(),
        }
    }
    pub fn position(&self) -> ChunkPos { self.terrain.position() }
    pub fn block(&self, x: i32, y: i32, z: i32) -> BlockStateId { self.terrain.block(x, y, z) }
    pub fn block_name(&self, x: i32, y: i32, z: i32) -> &str {
        &self.data.registry.state(self.block(x, y, z)).unwrap().name
    }
    pub fn biome_id_at(&self, x: i32, _y: i32, z: i32) -> BiomeId {
        self.biomes[(((z & 15) >> 2) * 4 + ((x & 15) >> 2)) as usize]
    }
    pub fn is_decorated(&self) -> bool { self.decorated }
    pub fn set_decorated(&mut self) { self.decorated = true; }
    pub fn data(&self) -> &Arc<GeneratorData> { &self.data }
}
