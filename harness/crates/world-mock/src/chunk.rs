use crate::block_state_properties::BlockProperties;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub name: String,
    pub properties: BlockProperties,
}
