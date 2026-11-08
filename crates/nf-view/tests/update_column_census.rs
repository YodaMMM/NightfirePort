//! control-funcs-update: the update-slot census.
//!
//! The game's class table `control_funcs` @0x26f890 (85 entries × 12B: update/collide/delete)
//! [M ACTION.ELF @0x26f890, dumped per entry — the same table the collide census reads live in
//! ee_start_05.bin @0x26f890, docs/port-map/core-loop.md §2 item 7]. Classes with a non-NULL
//! update slot, and the placement flags (low byte, the `parsemap_create_dynamic_objects`
//! 0x1d04f0 dispatch — the same selector the collide census pins) that create them:
//!
//! 0x02←0x0F Drone, 0x07←0x71 rotor, 0x1A←0xD8 RB, 0x1B←0xD9 SP, 0x1C←0xDB/0xE8..0xF4 Door,
//! 0x1D←0xDC + 0xE8..0xF4/0xEA..0xEF Trigger, 0x20/0x21←0xE1 Break/Destroy, 0x22←0xDE
//! Searchlight, 0x23←0xE0 Creature, 0x24←0xD4 Cloud, 0x25←0xE2 SS, 0x26←0xE3 Flicker,
//! 0x27←0xE4 Hurt, 0x28←0xDA Car, 0x2F←0xF0 Pickup, 0x30←0xF1 DroneSpawner, 0x31←0x2B LeafGen,
//! 0x32←0xF2 Emitter, 0x33←0x2C Lightning, 0x34←0x21 Ripples, 0x36←0x27 GT, 0x37←0x28 Sensor,
//! 0x38←0x30 Monitor, 0x39←0x29 Switch, 0x3A←0x2E Lock, 0x3B←0x2F Copter, 0x3C←0x31 FuseBox,
//! 0x3F←0xF3 EnvTree, 0x40←0xF9 SoundTrigger, 0x43←0x33 Hint, 0x44←0xFB MusicTrigger,
//! 0x45←0xFC Corona, 0x47←0x34 GunImp, 0x48←0xFD CamSubject, 0x49←0x35 PCQWorm, 0x4A←0x36 Sub,
//! 0x4B←0xFE Mine, 0x4C←0xD5 OneSided, 0x4D←0x37 MiniSub, 0x4E←0xD2 Shooter, 0x4F←0x39
//! SpaceMissile, 0x50←0x38 SpaceLaser, 0x52←0x3B DynamicObject, 0x53←0x41 InverterTrigger,
//! 0x54←0x4A Apocalypse. Runtime-only (never placed): 0x03/0x12 Player, 0x05 Bullet, 0x0F
//! Explode, 0x11 MP-bot Drone, 0x35 MP_ObjectUpdate (MP only), plus the effect classes the
//! creators spawn (0x06 Bubble, 0x0C Particles, 0x0D casing, 0x0E gas/smoke, 0x10 Debris,
//! 0x15/0x1F script anims, 0x19 Sunreflection, 0x2A Occlude, 0x2E Leaf, 0x41 Swoosh, 0x46
//! BodyGlow). Classes whose update slot is NULL can never be update gaps (0x2B-flag CreepWall
//! → class 0x2B, 0x2C-flag Wire → 0x2C, 0x2D-flag Ladder → 0x2D, 0x3A-flag AIVolume → 0x3E,
//! 0xFA ThirdIcon → 0x42 collide-only, 0x3A-flag Grapple → 0x51, ThirdCam → 0x00, RainBox/
//! Corona helpers → 0x01).
//!
//! This census counts every level's placements per class (`Level::parse` over
//! `nf_view::dev::LEVELS`, loads only — no driving) so the update-gap list works from real
//! counts. Run with NF_ISO set.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev;

/// The 0x1d04f0 dispatch: placement flags (low byte) → the class byte the arm's creator stores
/// (every creator's decompile; `none` = no object). Identical to the collide census's table
/// (`crates/nf-view/tests/collide_column_census.rs`, branch s/control-funcs-collide).
const FLAGS_TO_CLASS: &[(u8, Option<u8>)] = &[
    (0x0F, Some(0x02)),
    (0x20, Some(0x20)),
    (0x21, Some(0x34)),
    (0x22, Some(0x2D)),
    (0x24, None),
    (0x25, None),
    (0x26, Some(0x35)),
    (0x27, Some(0x36)),
    (0x28, Some(0x37)),
    (0x29, Some(0x39)),
    (0x2A, None),
    (0x2B, Some(0x31)),
    (0x2C, Some(0x33)),
    (0x2D, None),
    (0x2E, Some(0x3A)),
    (0x2F, Some(0x3B)),
    (0x30, Some(0x38)),
    (0x31, Some(0x3C)),
    (0x32, Some(0x01)),
    (0x33, Some(0x43)),
    (0x34, Some(0x47)),
    (0x35, Some(0x49)),
    (0x36, Some(0x4A)),
    (0x37, Some(0x4D)),
    (0x38, Some(0x50)),
    (0x39, Some(0x4F)),
    (0x3A, Some(0x51)),
    (0x3B, Some(0x52)),
    (0x3E, Some(0x2C)),
    (0x3F, Some(0x2B)),
    (0x41, Some(0x53)),
    (0x4A, Some(0x54)),
    (0x4E, None),
    (0x71, Some(0x07)),
    (0xD2, Some(0x4E)),
    (0xD4, Some(0x24)),
    (0xD5, Some(0x4C)),
    (0xD8, Some(0x1A)),
    (0xD9, Some(0x1B)),
    (0xDA, Some(0x28)),
    (0xDB, Some(0x1C)),
    (0xDC, Some(0x1D)),
    (0xDE, Some(0x22)),
    (0xDF, None),
    (0xE0, Some(0x23)),
    (0xE1, Some(0x21)),
    (0xE2, Some(0x25)),
    (0xE3, Some(0x26)),
    (0xE4, Some(0x27)),
    (0xE5, None),
    (0xE6, None),
    (0xE7, Some(0x00)),
    (0xE8, Some(0x1D)),
    (0xE9, None),
    (0xEA, Some(0x1D)),
    (0xEB, Some(0x1D)),
    (0xEC, Some(0x1D)),
    (0xED, Some(0x1D)),
    (0xEE, Some(0x1D)),
    (0xEF, Some(0x1D)),
    (0xF0, Some(0x2F)),
    (0xF1, Some(0x30)),
    (0xF2, Some(0x32)),
    (0xF3, Some(0x3F)),
    (0xF4, Some(0x1D)),
    (0xF5, Some(0x3E)),
    (0xF8, Some(0x3E)),
    (0xF9, Some(0x40)),
    (0xFA, Some(0x42)),
    (0xFB, Some(0x44)),
    (0xFC, Some(0x01)),
    (0xFD, Some(0x48)),
    (0xFE, Some(0x4B)),
];

/// Flags whose jump-table entry is the shared default arm 0x1d0f98 (no creation) [M jump table
/// @0x304B20, entry (flags−0x0F)·4 == 0x1d0f98; verified for every value below]. 0xFF is outside
/// the 0x0F..0xFE table range [M sltiu gate 0x1d0580-0x1d059c].
const DEFAULT_ARM_FLAGS: &[u8] = &[0x17, 0x19, 0x23, 0x47, 0xDD, 0xF6, 0xF7, 0xFF];

/// Classes whose update slot in `control_funcs` is NULL [M ACTION.ELF @0x26f890 per-entry dump].
const UPDATE_NULL: &[u8] = &[
    0x00, 0x01, 0x04, 0x08, 0x09, 0x0a, 0x0b, 0x13, 0x14, 0x16, 0x17, 0x18, 0x1e, 0x29, 0x2b,
    0x2c, 0x2d, 0x3d, 0x3e, 0x42, 0x51,
];

fn class_of(flags: u32) -> Option<u8> {
    let lo = (flags & 0xff) as u8;
    FLAGS_TO_CLASS.iter().find(|(f, _)| *f == lo).and_then(|(_, c)| *c)
}

#[test]
fn update_slot_classes_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = path.to_str().unwrap();
    let iso = IsoImage::open(path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
    };

    // Per-class totals over all 34 archives, and per-level rows for every class that has both a
    // non-NULL update slot and at least one placement.
    let mut totals: [u32; 0x55] = [0; 0x55];
    let mut per_level: Vec<(u32, Vec<(u8, u32)>)> = Vec::new();
    let mut unknown_flags: Vec<u32> = Vec::new();

    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { panic!("no archive {name} on the disc") };
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        let mut level_counts: [u32; 0x55] = [0; 0x55];
        for map in &level.maps {
            for p in &map.placements {
                let lo = p.flags & 0xff;
                if DEFAULT_ARM_FLAGS.contains(&((lo) as u8)) { continue; }
                let Some(cls) = class_of(p.flags) else {
                    if lo != 0 && !unknown_flags.contains(&lo) {
                        unknown_flags.push(lo);
                    }
                    continue;
                };
                totals[cls as usize] += 1;
                level_counts[cls as usize] += 1;
            }
        }
        let row: Vec<(u8, u32)> = (0u8..0x55)
            .filter(|c| !UPDATE_NULL.contains(c) && level_counts[*c as usize] > 0)
            .map(|c| (c, level_counts[c as usize]))
            .collect();
        per_level.push((id, row));
    }

    eprintln!("== update-slot census (placements per level, update classes only) ==");
    for (id, row) in &per_level {
        if row.is_empty() { continue; }
        let cells: Vec<String> = row.iter().map(|(c, n)| format!("{c:#04x}×{n}")).collect();
        eprintln!("{id:08x}: {}", cells.join(" "));
    }
    eprintln!("== totals over {} levels ==", dev::LEVELS.len());
    for c in 0u8..0x55 {
        if UPDATE_NULL.contains(&c) { continue; }
        eprintln!("class {c:#04x}: {} placements", totals[c as usize]);
    }
    eprintln!("flags bytes outside the dispatch table: {unknown_flags:?}");

    // The census's closures: classes the letter's rule closes with the count itself — no shipped
    // archive ever places them, so no level can be missing their tick. Runtime-only classes
    // (created by the game's own systems, never by a placement) and the MP-only registration
    // class alike: zero on every level [D this census, 34 archives].
    const NEVER_PLACED: &[u8] = &[
        0x03, 0x05, 0x06, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x15, 0x19, 0x1f, 0x23,
        0x2a, 0x2e, 0x41, 0x45, 0x46, 0x4a, 0x4d,
    ];
    for c in NEVER_PLACED {
        assert_eq!(totals[*c as usize], 0, "class {c:#04x} placed somewhere — the census table or this closure is wrong");
    }
    // 0x35 (MP_ObjectUpdate, flag 0x26 = MP_RegisterMPObject) IS placed — 286 rows, but only on
    // the MP archives (0x07000023..29 + 0x4b); MP content is out of the port's scope
    // (docs/systems/mp-never-mentioned.md), so the count scopes the class instead of closing it.
    for &id in dev::LEVELS {
        // The MP archives carry the 0x35 rows [D this census: 07000023..29 + 0700004b]; skip
        // them (MP out of the port's scope) and driving (0x46/0x48/0x49), assert 0 elsewhere.
        if (0x0700_0023..=0x0700_0029).contains(&id) || id == 0x0700_004b
            || id == 0x0700_0046 || id == 0x0700_0048 || id == 0x0700_0049 { continue; }
        let name = nf_view::level_archive_name(id);
        let data = read(&name).unwrap_or_else(|| panic!("no archive {name}"));
        let level = Level::parse(&data).unwrap();
        let mp35: u32 = level.maps.iter()
            .flat_map(|m| &m.placements)
            .filter(|p| class_of(p.flags) == Some(0x35))
            .count() as u32;
        assert_eq!(mp35, 0, "class 0x35 placement on {id:08x} — outside the MP archives the census claimed");
    }
    // The anchors: the classes the walk's port ticks are really reachable (a zero here would
    // mean the port ticks an empty class while the real content moved).
    for c in [0x02u8, 0x07, 0x1a, 0x1b, 0x1c, 0x1d, 0x20, 0x21, 0x22, 0x2f, 0x30, 0x3c, 0x40, 0x43] {
        assert!(totals[c as usize] > 0, "class {c:#04x} closed at zero — the dispatch table is wrong");
    }
    // Sub (0x4A) and MiniSub (0x4D) close like the collide census's closures [D collide census].
    assert_eq!(totals[0x4a as usize], 0, "Sub placements exist — Sub_Update is not closable");
    assert_eq!(totals[0x4d as usize], 0, "MiniSub placements exist — MiniSub_Update is not closable");
    eprintln!("== census assertions passed ==");
}
