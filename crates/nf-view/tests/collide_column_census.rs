//! control-funcs-collide: the collide-slot census.
//!
//! The game's class table `control_funcs` @0x26f890 (85 entries × 12B: update/collide/delete)
//! has a non-NULL collide slot for classes 0x02 0x03 0x05 0x0f 0x12 0x1a 0x1c 0x21 0x28 0x42
//! 0x4a 0x4d [M ACTION.ELF @0x26f890, dumped per entry]. Placement-reachable among them (the
//! `parsemap_create_dynamic_objects` 0x1d04f0 dispatch, selector = placement flags & 0xff):
//! 0x02←0x0F (Drone_Create), 0x1A←0xD8 (RB_Create), 0x1C←0xDB (Door_Create), 0x21←0xE1
//! (Destroy_Create), 0x28←0xDA (Car_Create — gated on MPSettings, refused in SP [M 0x1305b0]),
//! 0x42←0xFA (ThirdIcon_Create), 0x4A←0x36 (Sub_Create), 0x4D←0x37 (MiniSub_Create).
//! Player 0x03/0x12, Bullet 0x05 and Explode 0x0F are runtime-created, never placed.
//!
//! This census counts every level's placements (`Level::parse` over `nf_view::dev::LEVELS`,
//! loads only — no driving) and asserts the two stub collide slots (Sub_Collide 0x1ad868,
//! MiniSub_Collide 0x182128 — both 8-byte `jr ra; nop` [M]) never get an object on any shipped
//! level, which closes them with the count as proof. Run with NF_ISO set.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev;

/// The 0x1d04f0 dispatch: placement flags (low byte) → the class byte the arm's creator stores
/// (every creator's decompile; `none` = no object). Selected arms only — the full table is in
/// docs/reports/control-funcs-collide.md.
const FLAGS_TO_CLASS: &[(u8, Option<u8>)] = &[
    (0x0F, Some(0x02)), // Drone_Create 0x136be0 → NDrone2_CreateObj 0x14d7c8 (class @0x14d7c8:10,14)
    (0x20, Some(0x20)), // Break_Create 0x12c230 (kind==2 only pre-creates psi data)
    (0x21, Some(0x34)), // Ripples_Create 0x1acc90
    (0x22, Some(0x2D)), // Ladder_Create 0x1966c0
    (0x24, None),       // Player_AddNewStartPos (SP start positions)
    (0x25, None),       // MP_RegisterSpawnPoint (MP only)
    (0x26, Some(0x35)), // MP_RegisterMPObject → MP_CreateObject (MP only; kinds 3,5-8 register nothing)
    (0x27, Some(0x36)), // GT_Create 0x17ffa0
    (0x28, Some(0x37)), // Sensor_Create 0x1af010 (extra class-0x01 helpers on two params)
    (0x29, Some(0x39)), // Switch_Create 0x1adcc0
    (0x2A, None),       // View_AddSkyObj (GfxList slot, no object)
    (0x2B, Some(0x31)), // LeafGen_Create 0x1acbd0
    (0x2C, Some(0x33)), // Lightning_Create 0x18cde8 (8×0x01 helpers on a repeat channel)
    (0x2D, None),       // Player_AddNewStartPos
    (0x2E, Some(0x3A)), // Lock_Create 0x1ae1f8
    (0x2F, Some(0x3B)), // Copter_Create 0x17d340
    (0x30, Some(0x38)), // Monitor_Create 0x1b08b0
    (0x31, Some(0x3C)), // FuseBox_Create 0x1b0448
    (0x32, Some(0x01)), // RainBox_Create 0x18d840
    (0x33, Some(0x43)), // Hint_Create 0x1b3248
    (0x34, Some(0x47)), // GunImp_Create 0x131f20
    (0x35, Some(0x49)), // PCQWorm_Create 0x1b07a8
    (0x36, Some(0x4A)), // Sub_Create 0x1ad6c0
    (0x37, Some(0x4D)), // MiniSub_Create 0x182130
    (0x38, Some(0x50)), // Create_SpaceLaser 0x1aa8b8
    (0x39, Some(0x4F)), // Create_SpaceMissile 0x1a9ef0
    (0x3A, Some(0x51)), // Grapple_Create 0x198298
    (0x3B, Some(0x52)), // DynamicObject_Create 0x18dbf8
    (0x3E, Some(0x2C)), // Wire_Create 0x1981d8
    (0x3F, Some(0x2B)), // CreepWall_Create 0x196fc8
    (0x41, Some(0x53)), // InverterTrigger_Create 0x1b3040
    (0x4A, Some(0x54)), // Apocalypse_Create 0x18dcc0
    (0x4E, None),       // Light_Create (light node, no obj_tag)
    (0x71, Some(0x07)), // rotor_init 0x18f4b8
    (0xD2, Some(0x4E)), // Shooter_Create 0x17d030
    (0xD4, Some(0x24)), // Cloud_Create 0x18d620
    (0xD5, Some(0x4C)), // OneSided_Create 0x1e7ce8
    (0xD8, Some(0x1A)), // RB_Create 0x1d6bf0
    (0xD9, Some(0x1B)), // SP_CreateScriptPlayer 0x1db9a8
    (0xDA, Some(0x28)), // Car_Create 0x1305b0 (MPSettings-gated, refused in SP)
    (0xDB, Some(0x1C)), // Door_Create 0x136100
    (0xDC, Some(0x1D)), // Trigger_Create 0x1b1720
    (0xDE, Some(0x22)), // Searchlight_Create 0x1cb4f8 (the one arm with no param_7 gate)
    (0xDF, None),       // Env_Create (GlobalEnv, no object)
    (0xE0, Some(0x23)), // Creature_Create 0x134ab8
    (0xE1, Some(0x21)), // Destroy_Create 0x12c318
    (0xE2, Some(0x25)), // SS_Create 0x1a9c58
    (0xE3, Some(0x26)), // Flicker_Create 0x1acb00
    (0xE4, Some(0x27)), // Hurt_Create 0x17e870
    (0xE5, None),       // Drone_CoverCornerNode (AI node array)
    (0xE6, None),       // Drone_CoverLowNode
    (0xE7, Some(0x00)), // ThirdCam_Create 0x1bc510 (never stores a class byte → 0x00)
    (0xE8, Some(0x1D)), // Trigger_LoadLevelCreate → Trigger_Create
    (0xE9, None),       // Drone_AIPoint
    (0xEA, Some(0x1D)), // Trigger_TouchOnce
    (0xEB, Some(0x1D)), // Trigger_Touch
    (0xEC, Some(0x1D)), // Trigger_MultiplexIn
    (0xED, Some(0x1D)), // Trigger_MultiplexSIn
    (0xEE, Some(0x1D)), // Trigger_MultiplexOut
    (0xEF, Some(0x1D)), // Trigger_MultiplexOrIn
    (0xF0, Some(0x2F)), // Pickup_Create (kinds 7-11 detour, same class)
    (0xF1, Some(0x30)), // DroneSpawner_Create 0x1378d0 (needs param0>1 + drone system on)
    (0xF2, Some(0x32)), // Emitter_CreatePlist → Emitter_Create
    (0xF3, Some(0x3F)), // Env_Tree_Create 0x17c200
    (0xF4, Some(0x1D)), // Trigger_MoviePlayer
    (0xF5, Some(0x3E)), // Drone_AIVolume_Create (shared arm with 0xF8)
    (0xF8, Some(0x3E)),
    (0xF9, Some(0x40)), // SoundTrigger_Create 0x1b2628
    (0xFA, Some(0x42)), // ThirdIcon_Create 0x197070
    (0xFB, Some(0x44)), // MusicTrigger_Create 0x1b30e0
    (0xFC, Some(0x01)), // Corona_Create 0x18da00
    (0xFD, Some(0x48)), // CamSubject_Create 0x1bc5d8
    (0xFE, Some(0x4B)), // Mine_Create 0x17ce28
];

/// Classes with a non-NULL collide slot [M control_funcs @0x26f890, per-entry dump].
const COLLIDE_CLASSES: &[u8] = &[0x02, 0x03, 0x05, 0x0F, 0x12, 0x1A, 0x1C, 0x21, 0x28, 0x42, 0x4A, 0x4D];

/// Flags whose jump-table entry is the shared default arm 0x1d0f98 (no creation) [M jump table
/// @0x304B20, entry (flags−0x0F)·4 == 0x1d0f98; verified for every value below]. 0xFF is outside
/// the 0x0F..0xFE table range [M sltiu gate 0x1d0580-0x1d059c].
const DEFAULT_ARM_FLAGS: &[u8] = &[0x17, 0x19, 0x23, 0x47, 0xDD, 0xF6, 0xF7, 0xFF];

fn class_of(flags: u32) -> Option<u8> {
    let lo = (flags & 0xff) as u8;
    FLAGS_TO_CLASS.iter().find(|(f, _)| *f == lo).and_then(|(_, c)| *c)
}

#[test]
fn collide_slot_classes_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let path = path.to_str().unwrap();
    let iso = IsoImage::open(path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        Some(iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap())
    };

    // Per-class totals, and per-level counts for the collide-relevant placed classes.
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
        let row: Vec<(u8, u32)> = COLLIDE_CLASSES.iter().map(|&c| (c, level_counts[c as usize])).filter(|(_, n)| *n > 0).collect();
        per_level.push((id, row));
    }

    eprintln!("== collide-slot census (placements per level, collide classes only) ==");
    for (id, row) in &per_level {
        if row.is_empty() { continue; }
        let cells: Vec<String> = row.iter().map(|(c, n)| format!("{c:#04x}×{n}")).collect();
        eprintln!("{id:08x}: {}", cells.join(" "));
    }
    eprintln!("== totals over {} levels ==", dev::LEVELS.len());
    for c in COLLIDE_CLASSES {
        eprintln!("class {c:#04x}: {} placements", totals[*c as usize]);
    }
    // The unmodelled-but-placed classes for context.
    for c in [0x20u8, 0x2F, 0x37u8, 0x39, 0x1D, 0x1B, 0x23, 0x25, 0x4B] {
        eprintln!("(context) class {c:#04x}: {} placements", totals[c as usize]);
    }
    eprintln!("flags bytes outside the dispatch table: {unknown_flags:?}");

    // Sub_Collide 0x1ad868 and MiniSub_Collide 0x182128 are 8-byte no-ops [M]; close them with
    // the count: no shipped level ever places a class-0x4A or class-0x4D object.
    assert_eq!(totals[0x4A as usize], 0, "Sub (0x36) placements exist — Sub_Collide is not closable");
    assert_eq!(totals[0x4D as usize], 0, "MiniSub (0x37) placements exist — MiniSub_Collide is not closable");
    // Car (0xDA → class 0x28, collide 0x131838): the only non-origin car placements are on
    // non-campaign archives (0x07000023..29/0x46/0x4b — none has a campaign loading image:
    // ResetMap_LevelCode2Img 0x1d5258 groups 0x21-0x29+0x4b+0x4c with the MP image and has no
    // case for 0x46 [M 0x1d5258 cases]); the campaign archives' 3 rows (07000002 ×2, 07000025 ×1)
    // all sit at the exact origin with no params — dummy rows [D census dump]. MP/driving content
    // is out of the port's scope (docs/systems/mp-never-mentioned.md); the contact path stays a
    // hand-off stub.
    for (id, row) in &per_level {
        let cars = row.iter().find(|(c, _)| *c == 0x28).map(|(_, n)| *n).unwrap_or(0);
        if cars > 0 && *id <= 0x0700_001b {
            eprintln!("campaign {id:08x}: {cars} car rows — must all be origin dummies");
        }
    }
    for &id in dev::LEVELS {
        let name = nf_view::level_archive_name(id);
        let Some(data) = read(&name) else { continue };
        let level = Level::parse(&data).unwrap();
        let campaign = id <= 0x0700_001b;
        for map in &level.maps {
            for p in &map.placements {
                if p.flags & 0xff != 0xda { continue; }
                let at_origin = p.translation == [0.0, 0.0, 0.0];
                if campaign {
                    assert!(at_origin, "campaign {id:08x}: functional car placement at {:?} — car contact path owed", p.translation);
                }
            }
        }
    }
    // The player, bullet and explode collide slots are runtime classes, never placed:
    for c in [0x03u8, 0x05, 0x0F, 0x12] {
        assert_eq!(totals[c as usize], 0, "class {c:#04x} placed — census table is wrong");
    }
    // The placed collide classes must be non-zero somewhere on the disc (the contact paths the
    // port audits are reachable): drone, RB, door, destroy.
    assert!(totals[0x02 as usize] > 0, "no drone placements on any level");
    assert!(totals[0x1A as usize] > 0, "no RB placements on any level");
    assert!(totals[0x1C as usize] > 0, "no door placements on any level");
    assert!(totals[0x21 as usize] > 0, "no destroy placements on any level");
    assert!(totals[0x42 as usize] > 0, "no third-icon placements on any level");
    eprintln!("== census assertions passed ==");
}
