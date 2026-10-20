//! weapon-fire-sound-ids (G-029 remainder): every player-holdable weapon's first-person fire
//! sound id, from the disc's own scripts through the game's op-1 remap.
//!
//! The path [M `AnimProcessScriptCmds` 0x1c1b38 case 1: cmd[1] = frame, cmd[2] = sfx id; the id is
//! replaced by the owner's weapon byte `*(char*)(*(int*)(obj+0xDC) + 0x62)` — six rules (1→2 on
//! 0x07/0x09, 0x403→2 on 0x03/0x05, 0xfa→0x1fb on 0x14, 0x106→0x284 on 0x1e..0x23, 0x1e1→0x602 on
//! 0x10, 4→0x604 on 0x16/0x17), everything else plays as-is — then `Sound_Play3D(id, pos, 100,
//! 100, 0, Rand(500)−250, 0, 0)` when the frame crosses cmd[1]]. The scripts are the disc's
//! kind-5 entries at the record's +0xB0 (fire) / +0xB4 (fire-last) hashcodes [D
//! docs/fields/weapon_data.md; the parse matches `AnimScriptInit` 0x1c1600's layout].
//!
//! The GUARDS' fire path is different and untouched [M `DroneWeap_FireWeapon` 0x178f68: it plays
//! `weapon_data[w] +0x60` (u16, `AnimDatumGetWeaponInfo` 0x1c7900's third out-param), SKIPPED when
//! 0x5ef, and never reads a script or the remap] — the sister test pins the remap is the identity
//! on every +0x60 value.
//!
//! Opt-in via `NF_ISO`; NF_SILENT keeps every probe off the speakers.

use nf_view::scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// One weapon's pinned fire sounds: the op-1 commands of the fire (+0xB0) and fire-last (+0xB4)
/// scripts in command order, ids AFTER the remap, as (frame, id). An empty vec = no op-1 in that
/// script (no first-person fire sound from it).
type Row = (u8, &'static [(u16, u32)], &'static [(u16, u32)]);

/// The table [D: the scripts parsed from the disc; R rows cite the remap rule M 0x1c1b38].
/// Every other id plays as the script wrote it.
#[rustfmt::skip]
const TABLE: &[Row] = &[
    (0x01, &[(7, 0x142)], &[(9, 0x142)]),                       // fists: the whoosh pair
    (0x02, &[(1, 0x403)], &[(1, 0x403)]),                       // P2K: no remap — 0x403 is real [M 0x1c1b38: 0x02 not in the table]
    (0x03, &[(1, 2)],     &[(1, 2)]),                           // silenced twin: 0x403 → 2 [M]
    (0x04, &[(1, 0x403)], &[(1, 0x403)]),
    (0x05, &[(1, 2)],     &[(1, 2)]),                           // silenced twin: 0x403 → 2 [M]
    (0x06, &[(1, 0x1)],   &[(1, 0x1)]),
    (0x07, &[(1, 2)],     &[(1, 2)]),                           // 1 → 2 [M]
    (0x08, &[(1, 0x1)],   &[(1, 0x1)]),
    (0x09, &[(1, 2)],     &[(1, 2)]),                           // 1 → 2 [M]
    (0x0a, &[(1, 0x1f7)], &[]),
    (0x0b, &[(1, 0x1f7)], &[]),
    (0x0c, &[(1, 0x1f7)], &[]),
    (0x0d, &[(1, 0x1f7)], &[(1, 0x1f7)]),
    (0x0e, &[(1, 0x1e1)], &[(1, 0x1e1)]),
    (0x0f, &[(1, 0x1e1)], &[(1, 0x1e1)]),
    (0x10, &[(1, 0x602)], &[(1, 0x602)]),                       // 0x1e1 → 0x602 [M]
    (0x11, &[(1, 0x449), (43, 0x44b)], &[(1, 0x449)]),
    (0x12, &[(1, 0x3)],   &[]),
    (0x13, &[(1, 0x3)],   &[]),
    (0x14, &[(1, 0x1fb)], &[]),                                 // 0xfa → 0x1fb [M]
    (0x15, &[(1, 0xfa)],  &[]),
    (0x16, &[(1, 0x604)], &[]),                                 // 4 → 0x604 [M]
    (0x17, &[(1, 0x604)], &[]),                                 // 4 → 0x604 [M]
    (0x18, &[(1, 0x4)],   &[]),                                 // outside 0x16/0x17 — no remap [M (w−0x16) < 2]
    (0x19, &[(1, 0x4)],   &[]),
    (0x1a, &[(1, 0x40d)], &[]),
    (0x1b, &[(1, 0x40c)], &[]),
    (0x1c, &[(1, 0x1de), (53, 0x1e0)], &[]),
    (0x1d, &[(1, 0x1de)], &[]),
    (0x1e, &[(1, 0x284), (54, 0x107)], &[]),  // 0x106 → 0x284 [M]; both ops live in the FIRE script — no fire-last script
    (0x1f, &[(1, 0x284), (54, 0x107)], &[]),
    (0x20, &[(1, 0x284), (54, 0x107)], &[]),
    (0x21, &[(1, 0x284), (54, 0x107)], &[]),
    (0x22, &[(1, 0x284), (54, 0x107)], &[]),
    (0x23, &[(1, 0x284), (54, 0x107)], &[]),
    (0x24, &[(1, 0x106), (54, 0x107)], &[]),  // outside 0x1e..0x23 — no remap [M]
    (0x25, &[(1, 0x106), (54, 0x107)], &[]),
    (0x26, &[(1, 0x106), (54, 0x107)], &[]),
    (0x27, &[(1, 0x106), (54, 0x107)], &[]),
    (0x28, &[(1, 0x106), (54, 0x107)], &[]),
    (0x29, &[(1, 0x106), (54, 0x107)], &[]),
    (0x2a, &[(1, 0x40b)], &[]),
    (0x2b, &[(1, 0x40b)], &[]),
    (0x2c, &[(1, 0x10c)], &[]),
    (0x2d, &[(1, 0x10c)], &[]),
    (0x2e, &[(1, 0x10c)], &[]),
    (0x2f, &[(1, 0x10c)], &[]),
    (0x30, &[(1, 0x10c)], &[]),
    (0x31, &[(1, 0x10c)], &[]),
    (0x32, &[(2, 0x421), (269, 0x423)], &[]),
    (0x33, &[(2, 0x420), (14, 0x422)],  &[]),
    (0x34, &[(16, 0x1e4)], &[(22, 0x41)]),
    (0x35, &[(16, 0x1e4)], &[(24, 0x41)]),
    (0x36, &[(16, 0x1e4)], &[(24, 0x41)]),
    (0x37, &[(18, 0x458)], &[]),
    (0x38, &[], &[]),                                           // no fire script at all
    (0x39, &[], &[]),
    (0x3a, &[(19, 0x445), (64, 0x446)], &[]),
    (0x3b, &[(22, 0x447), (84, 0x448)], &[]),
    (0x3c, &[(22, 0x447), (84, 0x448)], &[]),
    (0x3d, &[(22, 0x447), (84, 0x448)], &[]),
    (0x3e, &[(22, 0x447), (84, 0x448)], &[]),
    (0x3f, &[(22, 0x447), (84, 0x448)], &[]),
    (0x40, &[(22, 0x447), (84, 0x448)], &[]),
    (0x41, &[(2, 0x27f)], &[]),
    (0x42, &[(2, 0x41b)], &[]),
    (0x43, &[(2, 0x5ce)], &[]),
    (0x44, &[(2, 0x5ce)], &[]),
    (0x45, &[(15, 0x41)], &[]),                                 // the taser: +0x60 = 0x5ef — the guard path skips it [M 0x178f68]
    (0x46, &[(1, 0x1)],   &[(1, 0x1)]),
    (0x47, &[], &[]),
    (0x48, &[], &[]),
    (0x49, &[], &[]),
    (0x4a, &[(1, 0x3d)],  &[]),
    (0x4b, &[], &[]),
    (0x4c, &[(1, 0x3d)],  &[]),
    (0x4d, &[], &[]),
    (0x4e, &[(1, 0x12)],  &[]),
    (0x4f, &[(1, 0x12)],  &[]),
    (0x50, &[(1, 0x1f5)], &[]),
    (0x51, &[(1, 0x1f5)], &[]),
    (0x52, &[(29, 0x450), (31, 0x44e)], &[]),
    (0x53, &[], &[]),                                           // handles exist, no op-1 command
    (0x54, &[], &[]),
    (0x55, &[], &[]),
    (0x56, &[(11, 0x452), (18, 0x452), (26, 0x452), (33, 0x452), (39, 0x452), (48, 0x452), (55, 0x452), (62, 0x452)],
           &[(9, 0x452), (19, 0x452), (26, 0x452), (34, 0x452), (40, 0x452)]),
    (0x57, &[(11, 0x452), (18, 0x452), (26, 0x452), (33, 0x452), (39, 0x452), (48, 0x452), (55, 0x452), (62, 0x452)],
           &[(9, 0x452), (19, 0x452), (26, 0x452), (34, 0x452), (40, 0x452)]),
    (0x58, &[(16, 0x456), (72, 0x454)], &[]),
    (0x59, &[(24, 0x458), (65, 0x448)], &[]),
    (0x5a, &[], &[]),
    (0x5b, &[(24, 0x458), (65, 0x448)], &[]),
];

/// The pinned table against the disc, through the real load: every weapon 0x01..0x5b's fire and
/// fire-last scripts parse, resolve, and their op-1 ids after `remap_op1_sound` equal the table.
/// Sister: every non-zero fire/fire-last hashcode in the SP range resolves in the loaded scripts
/// (the merge must not leave a hole the game would have filled).
#[test]
fn every_holdable_weapons_fire_script_op1_matches_the_table() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load_dev(&iso, "07000001.bin", false).expect("load_dev 07000001");
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.weapons.records.len(), 115, "weapon_data is 115 records [M docs/formats/weapon-data.md]");
    assert!(!sim.scripts.is_empty(), "the weapon-asset merge must load the FP scripts");

    let mut unresolved: Vec<(u8, u32)> = Vec::new();
    for w in 0x01u8..=0x5b {
        let rec = &sim.weapons.records[w as usize];
        for h in [u32::from_le_bytes(rec[0xb0..0xb4].try_into().unwrap()),
                  u32::from_le_bytes(rec[0xb4..0xb8].try_into().unwrap())] {
            if h != 0 && !sim.scripts.contains_key(&h) { unresolved.push((w, h)); }
        }
    }
    assert!(unresolved.is_empty(), "unresolved fire-script hashes (the merge missed them): {unresolved:?}");

    for &(w, want_fire, want_last) in TABLE {
        let rec = &sim.weapons.records[w as usize];
        for (label, handle_off, want) in [
            ("fire", 0xb0usize, want_fire),
            ("fire-last", 0xb4, want_last),
        ] {
            let h = u32::from_le_bytes(rec[handle_off..handle_off + 4].try_into().unwrap());
            let got: Vec<(u16, u32)> = match sim.scripts.get(&h) {
                Some(sc) => sc.commands.iter()
                    .filter(|c| c.op == 1 && c.args.len() >= 2)
                    .map(|c| (c.args[0], scene::remap_op1_sound_for_test(c.args[1] as u32, w)))
                    .collect(),
                None => Vec::new(),
            };
            assert_eq!(got, want, "weapon {w:#04x} {label} script {h:#x}: op-1 (frame, post-remap id) mismatch");
        }
    }
    // Every row of the table was checked; the table itself covers all of 0x01..=0x5b.
    let covered: std::collections::HashSet<u8> = TABLE.iter().map(|&(w, _, _)| w).collect();
    let missing: Vec<u8> = (0x01u8..=0x5b).filter(|w| !covered.contains(w)).collect();
    assert!(missing.is_empty(), "the table must cover every SP-holdable record, missing {missing:?}");
}

/// SISTER — the guards' fire sounds are unchanged: `DroneWeap_FireWeapon` 0x178f68 plays
/// `weapon_data[w] +0x60` (u16, `AnimDatumGetWeaponInfo` 0x1c7900's third out-param), skipped when
/// 0x5ef, and never reads an animation script or the op-1 remap. The remap must therefore be the
/// IDENTITY on every record's +0x60 value (no guard sound collides with the op-1 table), and the
/// port's drone path reads the record directly. The sample rows pin the disc values.
#[test]
fn the_remap_never_touches_the_guard_fire_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load_dev(&iso, "07000001.bin", false).expect("load_dev 07000001");
    let sim = scene.sim.as_ref().unwrap();
    for (w, rec) in sim.weapons.records.iter().enumerate() {
        let drone_id = u16::from_le_bytes([rec[0x60], rec[0x61]]) as u32;
        let w = w as u8;
        assert_eq!(scene::remap_op1_sound_for_test(drone_id, w), drone_id,
            "weapon {w:#04x}: the guard fire sound {drone_id:#x} must pass the op-1 remap unchanged (DroneWeap_FireWeapon never remaps [M 0x178f68])");
    }
    // The disc's sample rows [D].
    let drone_id = |w: u8| u16::from_le_bytes([sim.weapons.records[w as usize][0x60], sim.weapons.records[w as usize][0x61]]);
    assert_eq!(drone_id(0x02), 0x42c, "the P2K's guard fire sound");
    assert_eq!(drone_id(0x03), 0x42b, "the silenced twin's own guard sound");
    assert_eq!(drone_id(0x16), 0x605, "the SG5's guard fire sound");
    assert_eq!(drone_id(0x14), 0x426);
    assert_eq!(drone_id(0x45), 0x5ef, "the taser's guard fire sound is 0x5ef — DroneWeap_FireWeapon SKIPS it [M 0x178f68]");
    // The op-1 remap's six rules stay exactly the decompile's (the fire-table's [M] spine):
    assert_eq!(scene::remap_op1_sound_for_test(1, 0x07), 2);
    assert_eq!(scene::remap_op1_sound_for_test(0x403, 0x05), 2);
    assert_eq!(scene::remap_op1_sound_for_test(0xfa, 0x14), 0x1fb);
    assert_eq!(scene::remap_op1_sound_for_test(0x106, 0x23), 0x284);
    assert_eq!(scene::remap_op1_sound_for_test(0x1e1, 0x10), 0x602);
    assert_eq!(scene::remap_op1_sound_for_test(4, 0x17), 0x604);
    // ...and the near-misses stay put (fail-able sisters of the range edges):
    assert_eq!(scene::remap_op1_sound_for_test(0x403, 0x02), 0x403, "the P2K is not in the table");
    assert_eq!(scene::remap_op1_sound_for_test(0x106, 0x24), 0x106, "0x24 is outside 0x1e..0x23");
    assert_eq!(scene::remap_op1_sound_for_test(4, 0x18), 4, "0x18 is outside 0x16/0x17");
    assert_eq!(scene::remap_op1_sound_for_test(1, 0x46), 1, "the Q-Worm's script id 1 is not remapped");
}
