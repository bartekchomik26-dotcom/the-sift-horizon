// The ancient city's centre (h_city_template.rs): The Sift 2.0.1's three centres are read in
// place of vanilla's, by their path under the structure folder.
use std::io;
use std::path::Path;
include!("../../../../mixins/the_sift_native/h_city_template.rs");

#[test]
fn an_ancient_city_centre_reads_the_sifts() {
    for i in 1..=3 {
        let p = format!("/x/minecraft/structure/minecraft/ancient_city/city_center/city_center_{i}.nbt");
        let core::ops::ControlFlow::Break(Ok(bytes)) = the_sift_city_template(Path::new(&p)) else { panic!("{p}") };
        assert_eq!(&bytes[..2], &[0x1f, 0x8b], "gzip NBT");
    }
    assert!(matches!(the_sift_city_template(Path::new("/x/minecraft/ancient_city/city_center/city_center_4.nbt")), core::ops::ControlFlow::Continue(())));
    assert!(matches!(the_sift_city_template(Path::new("/x/minecraft/ancient_city/structures/barracks.nbt")), core::ops::ControlFlow::Continue(())));
    assert!(matches!(the_sift_city_template(Path::new("/x/minecraft/ancient_city/city_center/city_center_1.json")), core::ops::ControlFlow::Continue(())));
}

