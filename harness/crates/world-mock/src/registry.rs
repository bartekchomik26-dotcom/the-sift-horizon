//! As Horizon's BlockRegistry: every state of every block, a default per block, and `resolve`
//! filling a descriptor's omitted properties from the default and refusing unknown ones.
use crate::chunk::BlockState;
use std::collections::HashMap;
use std::io;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockStateId(pub u32);
pub struct BlockRegistry {
    states: Vec<BlockState>,
    ids: HashMap<BlockState, u32>,
    defaults: HashMap<String, BlockStateId>,
}
impl BlockRegistry {
    /// Blocks with their properties' values (the first value of each is the default).
    pub fn new(blocks: &[(&str, Vec<(&str, Vec<&str>)>)]) -> Self {
        let mut r = BlockRegistry { states: Vec::new(), ids: HashMap::new(), defaults: HashMap::new() };
        for (name, props) in blocks {
            let mut combos: Vec<Vec<(&str, &str)>> = vec![Vec::new()];
            for (k, vs) in props {
                combos = combos.into_iter().flat_map(|c| vs.iter().map(move |v| { let mut c = c.clone(); c.push((k, v)); c })).collect();
            }
            for (i, c) in combos.into_iter().enumerate() {
                let mut properties = crate::block_state_properties::BlockProperties::new();
                for (k, v) in c { properties.insert(k.to_owned(), v.to_owned()); }
                let s = BlockState { name: (*name).to_owned(), properties };
                let id = r.states.len() as u32;
                if i == 0 { r.defaults.insert((*name).to_owned(), BlockStateId(id)); }
                r.ids.insert(s.clone(), id);
                r.states.push(s);
            }
        }
        r
    }
    pub fn len(&self) -> usize { self.states.len() }
    pub fn state(&self, id: BlockStateId) -> Option<&BlockState> { self.states.get(id.0 as usize) }
    pub fn default_state(&self, name: &str) -> Option<BlockStateId> { self.defaults.get(name).copied() }
    pub fn resolve(&self, descriptor: &BlockState) -> io::Result<BlockStateId> {
        let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_owned());
        let default = self.default_state(&descriptor.name).ok_or_else(|| bad("unknown block name"))?;
        let mut state = self.states[default.0 as usize].clone();
        for (k, v) in &descriptor.properties {
            if !state.properties.contains_key(k) { return Err(bad("unknown block property")); }
            state.properties.insert(k.clone(), v.clone());
        }
        self.ids.get(&state).map(|&i| BlockStateId(i)).ok_or_else(|| bad("invalid block property value"))
    }
}
