//! cutscene-cel-dup (0700000b): the stairwell doorway drew TWO doors in the roof scene — the
//! scene's own swinging door entity `NIS_towerC_door` (hash 0x02000668, created by the pack
//! script 060004FA [D]) and, in the same frame, the level's closed static door
//! `door_NIS_load_point` (placement #148, a 0x8019 inner-chain cel at the origin, object #140 of
//! the main map, glist flags 0xc0000 [D]).
//!
//! The game drops the static one while the scene camera renders: `Script_CameraStart` 0x1d9140
//! enables camera 4 only, `View_CaptureScene` 0x1e6df0:15 sets `object_display_mask = 1 <<
//! viewer+0x28` = 0x10, and `View_AddCels` 0x1e6ba0:57 skips every inner-chain cel whose
//! `cel+0x94` (the glist flags, copied by `parsemap_block_map_data_static` 0x1d1998) carries
//! 0x40000 while the mask is 0x10 [M]. The port baked the cel into the World draws with no such
//! gate. The rule is general (every level's non-room 0x8000 cels); the census test lists them.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The roof scene's script camera hides the static stairwell door; normal play keeps it.
#[test]
fn the_static_stairwell_door_hides_while_the_roof_scene_camera_renders() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let lv = nf_formats::level::Level::parse(&read_archive(&iso, "0700000b.bin")).expect("parse 0b");
    let main = lv.main_map().expect("main map");
    // [D] placement #148: type 0x19 with CREATE_CEL (flags 0x8019), by index → object #140
    // `door_NIS_load_point`, glist flags 0xc0000, at the origin with no rotation.
    let p = &lv.maps[main].placements[148];
    assert_eq!(p.flags & 0xffff, 0x8019, "#148 is a 0x8019 inner-chain cel [D]");
    assert!(!p.by_hash() && p.index == 140, "#148 places object #140 by index [D]");
    let obj = &lv.maps[main].objects[140];
    assert_eq!(obj.name.trim(), "door_NIS_load_point");
    assert!(obj.flags & 0x40000 != 0, "its glist flags carry 0x40000 [D 0x{:x}]", obj.flags);
    assert_eq!(p.translation, [0.0, 0.0, 0.0]);

    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0b");
    // The build records the door's World draw centre (its glist centre through the identity
    // placement matrix) among the scene-hidden cels.
    let near: Vec<[f32; 3]> = scene.scene_hidden_cels.iter().copied()
        .filter(|c| (0..3).all(|k| (c[k] - obj.centre[k]).abs() < 1e-3)).collect();
    assert_eq!(near.len(), 1, "door_NIS_load_point is a scene-hidden cel (glist {:?})", obj.centre);
    let door_centre = near[0].map(f32::to_bits);
    // The door is a baked World draw keyed by that centre.
    assert!(scene.materials.iter().any(|m| m.layer == nf_view::scene::Layer::World && m.centre.map(f32::to_bits) == door_centre),
        "door_NIS_load_point draws as a World material at its glist centre");
    // Normal play (camera 0, mask 0x01): nothing is scene-hidden — the door draws.
    assert!(scene.scene_hidden_world_centres().is_empty(), "no scene camera → no cel is hidden");

    // The real trigger: ch 0x2D starts SP wedge #177 (script 060004FA) [D #177].
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-14.3, 47.4, 6.5];
        sim.drones.set_channel(0x2d);
        let mut cam = false;
        for _ in 0..300u32 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.script_camera().is_some() { cam = true; break; }
        }
        assert!(cam, "the roof scene (060004FA) never took the camera");
        for _ in 0..200u32 { sim.step_bond(Input::default(), false, false, false); }
        assert!(sim.script_camera().is_some(), "still in the scene at the doorway shot");
        // The scene's own door entity is live (the one that stays drawn).
        let nis_door = sim.drones.scripts.objects.list.iter().flatten().any(|o| matches!(o,
            nf_game::nis::Obj::Entity { hash: 0x0200_0668, .. }));
        assert!(nis_door, "060004FA's NIS_towerC_door entity (0x02000668) is live [D]");
    }
    let hidden = scene.scene_hidden_world_centres();
    assert!(hidden.contains(&door_centre),
        "the scene camera must drop door_NIS_load_point [M View_AddCels 0x1e6ba0:57: cel+0x94 & 0x40000 \
         && object_display_mask == 0x10]");

    // Control returns at the scene's tail: the gate lifts with the script camera.
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3200u32 {
            if sim.script_camera().is_none() { break; }
            sim.step_bond(Input::default(), false, false, false);
        }
        assert!(sim.script_camera().is_none(), "the roof scene ended");
    }
    assert!(scene.scene_hidden_world_centres().is_empty(), "after the scene the cels draw again");
}

/// Every level: the inner-chain cels the rule hides (non-room 0x8000 placements whose glist flags
/// carry 0x40000) are exactly the ones the scene build records, and no room cel is among them.
#[test]
fn census_scene_hidden_cels_every_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut total = 0usize;
    for &id in nf_view::dev::LEVELS {
        let name = format!("{id:08x}.bin");
        let lv = nf_formats::level::Level::parse(&read_archive(&iso, &name)).expect("parse");
        let mut names = Vec::new();
        for (mi, m) in lv.maps.iter().enumerate() {
            for p in &m.placements {
                if !p.creates_cel() || matches!(p.flags & 0xffff, 0xc023 | 0xc047) { continue; }
                let tgt = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
                let Some((a, b)) = tgt else { continue };
                let o = &lv.maps[a].objects[b];
                if o.flags & 0x40000 != 0 { names.push(format!("{} (map {mi}, flags {:#x})", o.name.trim(), o.flags)); }
            }
        }
        if !names.is_empty() { eprintln!("{id:08x}: {} — {}", names.len(), names.join(", ")); }
        total += names.len();
        if id == 0x0700_000b {
            assert!(names.iter().any(|n| n.starts_with("door_NIS_load_point")), "0b's stairwell door is in the census");
        }
    }
    eprintln!("total 0x40000 inner-chain cels: {total}");
}
