// Mock of the horizon-client-render surface The Sift's native half touches (Nightly f319701).
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub mod clouds { pub const CLOUD_HEIGHT: f64 = 192.33; }
#[path = "environment_real.rs"]
pub mod environment;
pub mod zip {
    use std::path::{Path, PathBuf};
    /// A zip on disk (the mock checks the local file header only).
    pub struct Zip(pub PathBuf);
    impl Zip {
        pub fn open(path: &Path) -> std::io::Result<Self> {
            let bytes = std::fs::read(path)?;
            if bytes.starts_with(b"PK\x03\x04") { Ok(Self(path.to_owned())) }
            else { Err(std::io::Error::other("not a zip")) }
        }
    }
}
pub mod datapack {
    use crate::zip::Zip;
    use std::path::{Path, PathBuf};
    pub struct DataPack { pub id: String, source: Source }
    enum Source { Dir(PathBuf), Zip(Box<Zip>) }
    impl DataPack {
        pub fn dir(id: &str, p: &Path) -> Self { Self { id: id.into(), source: Source::Dir(p.into()) } }
        pub fn zip_path(&self) -> Option<&Path> { match &self.source { Source::Zip(z) => Some(&z.0), Source::Dir(_) => None } }
    }
    pub mod the_sift_end { include!("../../../mixins/the_sift_native/end_pack.rs"); }
}
pub mod gfx {
    pub mod native_render {
        pub mod the_sift_sky { include!("sky_settings.rs"); }
    }
}
/// The mesher's tables, as Horizon's MeshTables holds them (f319701), and ichor.rs beside them.
pub mod mesher {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Info { pub solid: bool }
    #[derive(Clone, Debug, Default)]
    pub struct FaceVisibility(pub u32);
    pub struct MeshTables {
        pub(crate) info: Vec<Info>,
        pub(crate) solid: Vec<bool>,
        pub(crate) full_faces: Vec<u8>,
        pub visibility: FaceVisibility,
        pub(crate) water_still: u16,
        pub(crate) water_flow: u16,
        pub(crate) water_overlay: u16,
        pub(crate) lava_still: u16,
        pub(crate) lava_flow: u16,
        pub(crate) animated_layers: Vec<bool>,
    }
    impl MeshTables {
        pub fn new(n: usize) -> Self {
            MeshTables { info: vec![Info::default(); n], solid: vec![false; n], full_faces: vec![0; n],
                visibility: FaceVisibility(7), water_still: 1, water_flow: 2, water_overlay: 3, lava_still: 4,
                lava_flow: 5, animated_layers: vec![true; 12] }
        }
        pub fn water(&self) -> (u16, u16, u16, u16, u16) {
            (self.water_still, self.water_flow, self.water_overlay, self.lava_still, self.lava_flow)
        }
    }
    /// TerrainVertex as Horizon's mesher has it: section-relative position, UV in steps of 1/32768.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TerrainVertex { pub pos: [f32; 3], pub uv: [u16; 2], pub layer: u16, pub light: u16, pub color: u32 }
    pub const UV_STEPS: f32 = 32768.0;
    impl TerrainVertex {
        pub fn new(pos: [f32; 3], uv: [f32; 2], layer: u16, color: u32, light: u16) -> Self {
            Self { pos, uv: uv.map(|v| (v * UV_STEPS).round() as u16), layer, light, color }
        }
        pub fn uv(&self) -> [f32; 2] { self.uv.map(|u| f32::from(u) / UV_STEPS) }
    }
    pub mod the_sift_ichor { include!("../../../mixins/the_sift_native/ichor.rs"); }
}
