//! floating-rifle-02: the baked identity-rotation 0x8019 statics don't draw.
//!
//! The user's F12 005-07000002 (frame 4206) and the driver's nf-view render
//! (`--bond -43.11,6.46,10.37,2.305,0 --hold-npcs --frame 300`) showed a small dark
//! rifle-shaped mesh with a red highlight over the wall right of the 07000002 bathhouse
//! doorway; PCSX2 at the same spot shows nothing [L 2026-10-20, fresh boot]. The object is
//! `bath-sandalorange` (map 51 object 144, kind 0x8019, rotation [0,0,0], glist centre
//! (−39.073, 5.396, 9.961) [D placements + glist bounds]) with its twin `bath-sandalblue`
//! (obj 143, (−39.098, 5.395, 11.266)) — a sandal pair the port drew a second time over the
//! wall. The game's rule [M]: `parseentity_fixup_entity` 0x1d00c0:152-166 (from the static
//! parse `parsemap_block_map_data_static` 0x1d1998:110) marks every 0x8019/0x8018 record whose
//! matrix rows 1 and 2 are the identity with flags bit 0x8000000 — the bake mark: the art
//! already lives in the room's cel geometry, the record is a marker and draws nothing.
//! Rotated 0x19 records (row 2 ≠ identity) stay live inner-node statics — 07000009's
//! `liftswitch_fake` dressing (yaw π) is visible in the game [D placements; L visible].
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test baked_statics_02_live -- --test-threads=1`

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The world centre of the named main-map object's first instance (the placements are main-map
/// here [D placements-all 07000002: all records are map 51]).
fn named_instance_centre(level: &Level, name: &str) -> Option<[f32; 3]> {
    let (instances, _) = level.instances(true);
    instances.iter().find_map(|inst| {
        let obj = &level.maps[inst.map].objects[inst.object];
        if obj.name.trim() != name { return None; }
        let c = obj.centre;
        let m = &inst.matrix;
        Some([m[0][0] * c[0] + m[1][0] * c[1] + m[2][0] * c[2] + m[3][0],
              m[0][1] * c[0] + m[1][1] * c[1] + m[2][1] * c[2] + m[3][1],
              m[0][2] * c[0] + m[1][2] * c[1] + m[2][2] * c[2] + m[3][2]])
    })
}

fn load_02_hold() -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.hold_npcs = true;
    sim.bond.pos = [-43.11, 6.46, 10.37];
    sim.bond.yaw = 2.305;
    sim.bond.pitch = 0.0;
    for _ in 0..=300u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
    }
    scene
}

/// The draw list maps 1:1 onto `scene.materials` (the gpu draw index IS the material index
/// [M gpu.rs 615-616]), so a missing material is a missing draw.
fn draws_near(scene: &nf_view::scene::Scene, p: [f32; 3], r: f32) -> Vec<usize> {
    scene.materials.iter().enumerate().filter_map(|(k, m)| {
        let d = ((m.centre[0] - p[0]).powi(2) + (m.centre[1] - p[1]).powi(2) + (m.centre[2] - p[2]).powi(2)).sqrt();
        (d <= r).then_some(k)
    }).collect()
}

#[test]
fn the_baked_sandals_do_not_draw_and_rotated_statics_survive() {
    let level_data = {
        let im = IsoImage::open(&iso()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&im).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).unwrap();
        Level::parse(&im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap()).unwrap()
    };
    // The two sandal records are kind 0x8019 with identity rotation — the game's bake mark.
    let (instances, _) = level_data.instances(true);
    for name in ["bath-sandalorange", "bath-sandalblue"] {
        let inst = instances.iter().find(|i| level_data.maps[i.map].objects[i.object].name.trim() == name)
            .unwrap_or_else(|| panic!("{name} placed [D placements]"));
        assert_eq!(inst.placement.flags & 0xffff, 0x8019, "{name} is kind 0x8019 [D]");
        assert!(inst.placement.rotation.iter().all(|v| v.abs() < 1e-6), "{name} rotation identity [D]");
    }
    // A rotated 0x19 (the positive control): the light fixture at its placement-102 spot keeps
    // its yaw 0.733 (the object name is reused by several placements — match the translation).
    let fixture = instances.iter().find(|i| {
        level_data.maps[i.map].objects[i.object].name.trim() == "northerngarden4_lightfixture"
            && (i.placement.translation[0] + 41.21).abs() < 0.05
            && (i.placement.translation[2] + 34.74).abs() < 0.05
    }).expect("the light fixture placed [D placements]");
    assert_eq!(fixture.placement.flags & 0xffff, 0x8019, "the fixture is kind 0x8019 [D]");
    assert!((fixture.placement.rotation[1] - 0.733).abs() < 0.01, "the fixture is rotated (yaw 0.733) [D]");

    let scene = load_02_hold();
    let orange = named_instance_centre(&level_data, "bath-sandalorange").expect("orange centre");
    let blue = named_instance_centre(&level_data, "bath-sandalblue").expect("blue centre");
    let fixture_at = {
        let c = level_data.maps[fixture.map].objects[fixture.object].centre;
        let m = &fixture.matrix;
        [m[0][0] * c[0] + m[1][0] * c[1] + m[2][0] * c[2] + m[3][0],
         m[0][1] * c[0] + m[1][1] * c[1] + m[2][1] * c[2] + m[3][1],
         m[0][2] * c[0] + m[1][2] * c[1] + m[2][2] * c[2] + m[3][2]]
    };

    // THE FIX: no draw within 0.5 of either baked sandal (pre-fix: 5 material batches on the
    // orange one — the probe's mat[1687..1691], 4× texture 142 + the strap's texture 12).
    let hit_orange = draws_near(&scene, orange, 0.5);
    let hit_blue = draws_near(&scene, blue, 0.5);
    assert!(hit_orange.is_empty(), "the baked orange sandal draws nothing (got material indices {hit_orange:?})");
    assert!(hit_blue.is_empty(), "the baked blue sandal draws nothing (got material indices {hit_blue:?})");

    // SISTER (can fail): the ROTATED 0x19 static still draws — the skip must not over-reach.
    let fixture_draws = draws_near(&scene, fixture_at, 1.0);
    assert!(!fixture_draws.is_empty(),
        "the rotated light fixture (0x8019, yaw 0.733) still draws at {fixture_at:?} [M 0x1d00c0 row-2 arm]");
    // SISTER (can fail): the room geometry itself still draws — the wall the artifact floated on.
    assert!(scene.materials.iter().any(|m| m.layer == nf_view::scene::Layer::World && m.cel.is_some()),
        "room-cel World draws survive [D]");
}

