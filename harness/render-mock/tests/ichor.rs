//! ichor.rs as it ships: in The Sift a section is meshed with ichor's sprites for water's.
use horizon_client_render::datapack::the_sift_end as pack;
use horizon_client_render::mesher::{MeshTables, the_sift_ichor as ichor};

#[test]
fn water_is_meshed_as_ichor_only_in_the_sift() {
    let t = MeshTables::new(64);
    // Sprites not loaded (no content mod): water stays water.
    pack::set_active(true);
    assert!(ichor::tables(&t).is_none());
    ichor::set_layers(10, 11);
    let copy = ichor::tables(&t).expect("in The Sift, ichor's tables");
    assert_eq!(copy.water(), (10, 11, 3, 4, 5), "water's sprites swapped, lava's and the overlay kept");
    assert_eq!(copy.visibility.0, 7);
    // The copy is kept: the same one for the same tables, and never copied again.
    let again = ichor::tables(&t).unwrap();
    assert!(std::ptr::eq(copy, again));
    // Asked with the ichor tables themselves (a re-entry): left as they are.
    assert!(ichor::tables(copy).is_none());
    // Other tables (a resource reload): their own copy.
    let u = MeshTables::new(65);
    assert!(!std::ptr::eq(ichor::tables(&u).unwrap(), copy));
    // Out of The Sift: water.
    pack::set_active(false);
    assert!(ichor::tables(&t).is_none());
    ichors_still_texture_spans_sixteen_blocks_as_the_sift_draws_it();
}

// Run from the test above: the sprite layers are one static for the whole binary.
fn ichors_still_texture_spans_sixteen_blocks_as_the_sift_draws_it() {
    use horizon_client_render::mesher::TerrainVertex;
    ichor::set_layers(10, 11);
    // A still top face at section-local block (3, 5, 14): its corners, as the fluid mesher writes them.
    let corners = [([3.0, 5.9, 14.0], [0.0, 0.0]), ([3.0, 5.9, 15.0], [0.0, 1.0]), ([4.0, 5.9, 15.0], [1.0, 1.0]), ([4.0, 5.9, 14.0], [1.0, 0.0])];
    let mut v: Vec<TerrainVertex> = corners.iter().map(|(p, uv)| TerrainVertex::new(*p, *uv, 10, 0, 0)).collect();
    // A flowing face and a water face elsewhere are left alone.
    v.push(TerrainVertex::new([3.0, 5.9, 14.0], [0.25, 0.75], 11, 0, 0));
    v.push(TerrainVertex::new([3.0, 5.9, 14.0], [1.0, 1.0], 1, 0, 0));
    ichor::spread_uvs(&mut v);
    let uv: Vec<[f32; 2]> = v.iter().map(TerrainVertex::uv).collect();
    let near = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4;
    // IchorQuadUv: block 3 of 16 along x, 14 along z; one sixteenth of the texture each way.
    assert!(near(uv[0], [3.0 / 16.0, 14.0 / 16.0]), "{:?}", uv[0]);
    assert!(near(uv[2], [4.0 / 16.0, 15.0 / 16.0]), "{:?}", uv[2]);
    assert!(near(uv[4], [0.25, 0.75]) && near(uv[5], [1.0, 1.0]));
    // The block beside it carries on where this one ends: no seam, no repeat.
    let mut next = vec![TerrainVertex::new([4.0, 5.9, 14.0], [0.0, 0.0], 10, 0, 0)];
    ichor::spread_uvs(&mut next);
    assert!(near(next[0].uv(), uv[3]));
    // The last block of a section reaches the texture's edge.
    let mut last = vec![TerrainVertex::new([16.0, 5.9, 16.0], [1.0, 1.0], 10, 0, 0)];
    ichor::spread_uvs(&mut last);
    assert!(near(last[0].uv(), [1.0, 1.0]));
}
