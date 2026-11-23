//! G-019: the `GfxList` sky slot table `View_AddSkyObj` 0x1e79a0 builds at load, per level.
//! Two creators fill it in placement order [M parsemap_block_map_data_dynamic 0x1d1b68]: type 0x2A
//! (`parsemap_create_dynamic_objects` 0x1d04f0 case 0x2a: slot = param 0, group = param 4) and the
//! clouds, type 0xD4 (`Cloud_Create` 0x18d620: slot = param 4, the cloud object in the +0x18 handle,
//! group 1 — the stack argument at 0x18d724-0x18d728 [M disassembly]). A normal call writes both
//! normal handles, so a later placement on the same slot replaces the earlier one [M 0x1e79a0:18-21].
//! Opt-in: set `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_game::sky::Sky;

fn level(name: &str) -> Option<Level> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Some(Level::parse(&data).unwrap())
}

/// (slot, drawn handle's object name, which handle, group) for every filled slot.
fn table(l: &Level) -> Vec<(usize, String, &'static str, u8)> {
    let m = l.main_map().unwrap();
    let ps = &l.maps[m].placements;
    let sky = Sky::from_placements(ps);
    let name = |h: u32| {
        let p = &ps[(h - 1) as usize];
        let (mm, o) = if p.by_hash() { l.by_hash[&p.hashcode] } else { (m, p.index as usize) };
        l.maps[mm].objects[o].name.trim().to_string()
    };
    let mut out = Vec::new();
    for (i, s) in sky.slots.iter().enumerate() {
        for (h, kind) in [(s.glist, "glist"), (s.obj, "obj"), (s.flash_glist, "flash_glist"), (s.flash_obj, "flash_obj")] {
            if h != 0 { out.push((i, name(h), kind, s.skyid)); }
        }
    }
    out
}

fn has(t: &[(usize, String, &str, u8)], slot: usize, name: &str, kind: &str, group: u8) -> bool {
    t.iter().any(|(s, n, k, g)| *s == slot && n == name && *k == kind && *g == group)
}

#[test]
fn clouds_fill_their_slot_with_the_object_handle_in_group_1() {
    let Some(l) = level("07000041.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let t = table(&l);
    // #62 CloudLayerXR9: params 0:20 1:111 2:2 3:100 4:3 [D] -> slot 3, object handle, group 1.
    assert!(has(&t, 3, "CloudLayerXR9", "obj", 1), "{t:?}");
    assert!(!t.iter().any(|(s, _, k, _)| *s == 3 && *k == "glist"), "a cloud slot has no glist: {t:?}");
    // Sister: the 0x2A slots keep their glist handles and groups (SkyBox slot 0 group 1, moon slot 1 group 1).
    assert!(has(&t, 0, "SkyBox", "glist", 1) && has(&t, 1, "moon", "glist", 1), "{t:?}");
}

#[test]
fn a_later_placement_on_a_shared_slot_replaces_the_earlier_one() {
    // 07000004: #63 mountains (slot 1, group 1) then #64 moon (slot 1, group 0) [D]: the slot ends
    // as the moon, in group 0 — the mountains never draw through the sky pass.
    let Some(l4) = level("07000004.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let t = table(&l4);
    assert!(has(&t, 1, "moon", "glist", 0), "{t:?}");
    assert!(!t.iter().any(|(_, n, _, _)| n == "mountains"), "{t:?}");
    // 07000006: #0 CloudLayerXR9 (slot 3) then #427 Sky_Tower12 (slot 3, group 1): the tower's normal
    // call zeroes the cloud's object handle; #786 moon then #787 skybox share slot 0 -> skybox.
    let l6 = level("07000006.bin").unwrap();
    let t = table(&l6);
    assert!(has(&t, 3, "Sky_Tower12", "glist", 1), "{t:?}");
    assert!(!t.iter().any(|(_, n, _, _)| n == "CloudLayerXR9"), "the 06 cloud is overwritten: {t:?}");
    assert!(has(&t, 0, "skybox", "glist", 0) && !t.iter().any(|(_, n, _, _)| n == "moon"), "{t:?}");
    // 07000003: #2 moon_old then #487 SkyBox1-terra on slot 0 -> SkyBox1-terra; moon on slot 1.
    let t = table(&level("07000003.bin").unwrap());
    assert!(has(&t, 0, "SkyBox1-terra", "glist", 0) && has(&t, 1, "moon", "glist", 0), "{t:?}");
    assert!(!t.iter().any(|(_, n, _, _)| n == "moon_old"), "{t:?}");
}

#[test]
fn slots_carry_the_placement_translation_and_rotation() {
    // View_AddSkyObj copies the placement translation to +0x20 and the rotation to +0x30 [M :41-60];
    // the 0x0700001B spin and the glist-path draw read them. 0700000b's Thunderhead [D].
    let Some(l) = level("0700000b.bin") else { eprintln!("skip: NF_ISO not set"); return };
    let m = l.main_map().unwrap();
    let p = l.maps[m].placements.iter().find(|p| p.flags & 0xff == 0x2a && p.params.iter().any(|&(i, v)| i == 1 && v == 1)).unwrap();
    let sky = Sky::from_placements(&l.maps[m].placements);
    assert_eq!(sky.slots[3].translation, p.translation);
    assert_eq!(sky.slots[3].rotation, p.rotation);
    assert!(p.rotation[0] != 0.0, "the 0b Thunderhead is rotated [D]");
}
