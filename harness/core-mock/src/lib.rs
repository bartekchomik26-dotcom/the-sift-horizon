pub mod geometry {
    #[derive(Clone, Copy, Debug, PartialEq, Default)]
    pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
    impl Vec3 { pub fn new(x: f64, y: f64, z: f64) -> Self { Self { x, y, z } } }
}
pub mod position {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
    impl BlockPos { pub fn new(x: i32, y: i32, z: i32) -> Self { Self { x, y, z } } }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct ChunkPos { pub x: i32, pub z: i32 }
    impl ChunkPos { pub fn new(x: i32, z: i32) -> Self { Self { x, z } } }
}
pub mod ticks {
    use crate::position::BlockPos;
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct TickTypeId(pub u32);
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub enum TickPriority { ExtremelyHigh = -3, VeryHigh = -2, High = -1, #[default] Normal = 0 }
    #[derive(Clone, Copy, Debug)]
    pub struct ScheduledTick { pub kind: TickTypeId, pub pos: BlockPos, pub trigger_tick: i64, pub priority: TickPriority, pub sub_tick_order: i64 }
    /// As Horizon's: one tick per (kind, position).
    #[derive(Default)]
    pub struct ProtoChunkTicks { pub ticks: Vec<ScheduledTick> }
    impl ProtoChunkTicks {
        pub fn schedule(&mut self, tick: ScheduledTick) -> bool {
            if self.ticks.iter().any(|t| t.kind == tick.kind && t.pos == tick.pos) { return false; }
            self.ticks.push(tick);
            true
        }
        pub fn count(&self) -> usize { self.ticks.len() }
    }
}
pub mod random {
    #[derive(Debug, Clone, Copy)]
    pub struct InvalidBound;
    /// As Horizon's RandomSource (the draws The Sift's code makes).
    pub trait RandomSource {
        fn next_float(&mut self) -> f32;
        fn next_int_bound(&mut self, n: i32) -> Result<i32, InvalidBound>;
        fn next_double(&mut self) -> f64 { f64::from(self.next_float()) }
        /// Box-Muller over next_double (as Random.nextGaussian, without the cached pair).
        fn next_gaussian(&mut self) -> f64 {
            loop {
                let (a, b) = (2.0 * self.next_double() - 1.0, 2.0 * self.next_double() - 1.0);
                let s = a * a + b * b;
                if s < 1.0 && s != 0.0 {
                    return a * (-2.0 * s.ln() / s).sqrt();
                }
            }
        }
    }
    /// java.util.Random's LCG.
    #[derive(Clone, Debug)]
    pub struct LegacyRandom(u64);
    impl LegacyRandom {
        pub fn new(seed: i64) -> Self { LegacyRandom((seed as u64 ^ 0x5DEECE66D) & ((1 << 48) - 1)) }
        fn next(&mut self, bits: u32) -> i32 {
            self.0 = (self.0.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & ((1 << 48) - 1);
            (self.0 >> (48 - bits)) as i32
        }
    }
    impl RandomSource for LegacyRandom {
        fn next_float(&mut self) -> f32 { self.next(24) as f32 / (1 << 24) as f32 }
        fn next_int_bound(&mut self, n: i32) -> Result<i32, InvalidBound> { if n > 0 { Ok((self.next(31) as i64 * n as i64 >> 31) as i32) } else { Err(InvalidBound) } }
    }
}
