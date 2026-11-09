#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FluidKind { Empty, FlowingWater, Water, FlowingLava, Lava }
