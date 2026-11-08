//! placement-kind-census: every case of `parsemap_create_dynamic_objects` 0x1d04f0, counted per
//! level on the real archives, each with the port's create arm — or closed as zero.
//!
//! The game's create pass ([M] `parsemap_block_map_data_dynamic` 0x1d1b68:88-96, reached from
//! `ResetMap_Load` 0x1d6360:146 with create=1) walks ONE map block per archive: the LAST block-0x1a
//! parsed, which is the type-1 main map's (the last map-type entry of all 35 archives [D bin-list]).
//! Per placement it requires:
//! - the flags word's spawn gate `(flags & 0xa000) == 0` [M 0x1d1b68:88],
//! - the object resolved: by-hash placements through the archive-wide hash table, by-index ones
//!   through the main map's object list with `index < MapEntityCount` [M 0x1d1b68:76-86],
//! - the kind word in the dispatch range: the jump-table bounds check is
//!   `(kind - 0xf) < 0xf0` on the FULL flags word ([M disasm 0x1d04f4 addiu v1,a2,-0xf;
//!   0x1d0518 sltiu v0,v1,0xf0] — a flags word >= 0x100 falls through), then the table at
//!   0x304b20 dispatches.
//!
//! Kinds the port has no create arm for (and that occur on an on-foot level) are test failures:
//! the arm table below is the census' own classification, so a new kind showing up or an arm
//! disappearing turns this red.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_view::dev::LEVELS;

/// The game's kind set: every case label of the 0x1d04f0 switch ([M] the decompile
/// work/decomp/ACTION.ELF/001d04f0_*.c, 0xf..0xfe; 0xf5 and 0xf8 share the AIVolume arm).
const GAME_KINDS: &[u32] = &[
    0x0f, 0x20, 0x21, 0x22, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e,
    0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3e, 0x3f,
    0x41, 0x4a, 0x4e, 0x71, 0xd2, 0xd4, 0xd5, 0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xde, 0xdf, 0xe0,
    0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
];

/// The port's create arm per kind (census 2026-11-08): the module that consumes the placements.
/// A kind that occurs on some level MUST be in this table — the test enforces it.
const PORT_ARMS: &[(u32, &str)] = &[
    (0x0f, "nf-game drone.rs level_drones (Drone_Create 0x18a838)"),
    (0x20, "nf-game breakable.rs Breaks::create (Break_Create 0x12c230)"),
    (0x21, "nf-game world_fx.rs RipplesFx (Ripples_Create 0x1acc90)"),
    (0x22, "nf-game ladder.rs Ladders::create (Ladder_Create 0x1ad3a0)"),
    (0x24, "nf-game player.rs starts_from_placements kind 1 (Player_AddNewStartPos 0x195c00)"),
    (0x25, "nf-game mp/mod.rs spawn points (MP_RegisterSpawnPoint 0x184f20)"),
    (0x26, "nf-game mp/mod.rs MP objects (MP_RegisterMPObject 0x185990)"),
    (0x27, "nf-game drone.rs GT walk + gt.rs (GT_Create 0x17ffa0)"),
    (0x28, "nf-game sensor.rs Sensors::create (Sensor_Create 0x1af010)"),
    (0x29, "nf-game useobj.rs switches (Switch_Create 0x1adcc0)"),
    (0x2a, "nf-game sky.rs Sky::from_placements (View_AddSkyObj 0x1e79a0)"),
    (0x2b, "nf-game world_fx.rs LeafGenFx (LeafGen_Create 0x1acbd0)"),
    (0x2c, "nf-game world_fx.rs LightningStrike (Lightning_Create 0x18cde8)"),
    (0x2d, "nf-game player.rs starts_from_placements kind 0 (Player_AddNewStartPos 0x195c00)"),
    (0x2e, "nf-game lock.rs Lock::create (Lock_Create 0x1ae1f8)"),
    (0x2f, "nf-game copter.rs CopterPath/Copter (Copter_Create 0x17d340)"),
    (0x30, "nf-game monitor.rs Monitors::create (Monitor_Create 0x1b08b0)"),
    (0x31, "nf-game fusebox.rs FuseBox::create (FuseBox_Create 0x1b0448)"),
    (0x32, "nf-game drone.rs env walk rain boxes (RainBox_Create 0x17b5d8)"),
    (0x33, "nf-game player.rs hints (Hint_Create 0x1b3248)"),
    (0x34, "nf-game gunimp.rs (GunImp_Create 0x131f20)"),
    (0x35, "nf-game useobj.rs PC/Q-Worm (PCQWorm_Create 0x1aeb58)"),
    (0x36, "nf-game none — closed: zero placements on every level [D census]"),
    (0x37, "nf-game none — closed: zero placements on every level [D census]"),
    (0x38, "nf-game space_laser.rs (Create_SpaceLaser 0x1aa8b8)"),
    (0x39, "nf-game space_missile.rs (Create_SpaceMissile 0x1a9ef0)"),
    (0x3a, "nf-game gadget.rs GrapplePoint (Grapple_Create 0x198298)"),
    (0x3b, "nf-game worldobj.rs WorldObjects (DynamicObject_Create 0x18dbf8)"),
    (0x3e, "nf-game wire.rs (Wire_Create 0x1986c0)"),
    (0x3f, "nf-game creepwall.rs (CreepWall_Create 0x1b39c8)"),
    (0x41, "nf-game worldobj.rs InverterTrigger (InverterTrigger_Create 0x1b3040)"),
    (0x4a, "nf-game world_fx.rs Apocalypse (Apocalypse_Create 0x1b1e98)"),
    (0x4e, "nf-game light.rs level_lights 0x4e walk (Light_Create 0x1cc0e0)"),
    (0x71, "nf-game worldobj.rs Rotor (rotor_init 0x18f4b8)"),
    (0xd2, "nf-game worldobj.rs Shooter (Shooter_Create 0x17d030)"),
    (0xd4, "nf-game world_fx.rs CloudFx (Cloud_Create 0x18d620)"),
    (0xd5, "nf-view scene.rs draw-side culling (OneSided_Create 0x1e7ce8; Update is backface cull)"),
    (0xd8, "nf-game rigid_body.rs Rbodies::create (RB_Create 0x1d6bf0)"),
    (0xd9, "nf-game nis.rs Scripts::new (SP_CreateScriptPlayer 0x1db9a8)"),
    (0xda, "nf-game mp car wiring — see the census report (Car_Create 0x199558)"),
    (0xdb, "nf-game door.rs Doors::create (Door_Create 0x1baa28)"),
    (0xdc, "nf-game triggers.rs build (Trigger_Create 0x1b2c68)"),
    (0xde, "nf-game searchlight.rs Searchlights::create (Searchlight_Create 0x1cb4f8)"),
    (0xdf, "nf-game drone.rs env walk (Env_Create 0x17b968)"),
    (0xe0, "nf-game none — closed: zero placements on every level [D census]"),
    (0xe1, "nf-game props.rs Props::create (Destroy_Create 0x12c318)"),
    (0xe2, "nf-game ss.rs SsList::create (SS_Create 0x1a9c58)"),
    (0xe3, "nf-game world_fx.rs FlickerFx (Flicker_Create 0x1acb00)"),
    (0xe4, "nf-game hurt.rs Hurts::create (Hurt_Create 0x17e870)"),
    (0xe5, "nf-game cover.rs (Drone_CoverCornerNode 0x136ff0)"),
    (0xe6, "nf-game cover.rs (Drone_CoverLowNode 0x137340)"),
    (0xe7, "nf-game worldobj.rs ThirdCam (ThirdCam_Create 0x1bb160)"),
    (0xe8, "nf-game triggers.rs kind::LOAD_LEVEL (Trigger_LoadLevelCreate 0x1b2950)"),
    (0xe9, "nf-game aipoint.rs (Drone_AIPoint 0x137550)"),
    (0xea, "nf-game triggers.rs kind::TouchOnce (Trigger_TouchOnce 0x1b2e08)"),
    (0xeb, "nf-game triggers.rs kind::Touch (Trigger_Touch 0x1b2d38)"),
    (0xec, "nf-game triggers.rs kind::MultiplexIn (Trigger_MultiplexIn 0x1b2f28)"),
    (0xed, "nf-game triggers.rs kind::MultiplexSIn (Trigger_MultiplexSIn 0x1b2f88)"),
    (0xee, "nf-game triggers.rs kind::MultiplexOut (Trigger_MultiplexOut 0x1b2fc8)"),
    (0xef, "nf-game triggers.rs kind::OR (Trigger_MultiplexOrIn 0x1b3020)"),
    (0xf0, "nf-view scene.rs placed_pickup_from_placement + sim build (Pickup_Create 0x18dda0)"),
    (0xf1, "nf-game spawners.rs walk in drone.rs attach (DroneSpawner_Create 0x137b28)"),
    (0xf2, "nf-game world_fx.rs EmitterFx (Emitter_CreatePlist 0x17abc8)"),
    (0xf3, "nf-game world_fx.rs TreeFx (Env_Tree_Create 0x17c200)"),
    (0xf4, "nf-game triggers.rs kind::MOVIE (Trigger_MoviePlayer 0x1b2a08)"),
    (0xf5, "nf-game aivolume.rs (Drone_AIVolume_Create 0x149c60)"),
    (0xf8, "nf-game aivolume.rs (Drone_AIVolume_Create 0x149c60)"),
    (0xf9, "nf-game sound.rs SoundTriggers::create (SoundTrigger_Create 0x1b2628)"),
    (0xfa, "nf-game wire.rs ThirdIcon (ThirdIcon_Create 0x197070)"),
    (0xfb, "nf-game worldobj.rs MusicTrigger (MusicTrigger_Create 0x1b30e0)"),
    (0xfc, "nf-game corona.rs Coronas::create (Corona_Create 0x18da00)"),
    (0xfd, "nf-game props.rs Props::create (CamSubject_Create 0x1bc5d8)"),
    (0xfe, "nf-game worldobj.rs Mine::create (Mine_Create 0x17ce28)"),
];

/// The driving/MP mission archives (DRIVING.ELF's levels; the port's MP matches run on-foot on
/// them, docs/systems/mp-menus.md "Select Map"). Counted, but the report separates them.
const DRIVING_MP: &[u32] = &[
    0x0700_0023, 0x0700_0024, 0x0700_0025, 0x0700_0026, 0x0700_0027, 0x0700_0028, 0x0700_0029,
];

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(img: &IsoImage, archive: &str) -> Vec<u8> {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
fn placement_kind_census() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set; skipping"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    // counts[kind] = (resolved, unresolved, total) summed over levels; per-level rows printed.
    let mut counts: std::collections::BTreeMap<u32, (usize, usize)> = GAME_KINDS.iter().map(|&k| (k, (0, 0))).collect();
    let mut rows: Vec<String> = Vec::new();
    let mut off_range_kinds: Vec<(u32, u32)> = Vec::new(); // (level, flags) with flags >= 0x100 that pass the gate
    for &level in LEVELS {
        let name = format!("{level:08x}.bin");
        let data = read_archive(&img, &name);
        let lv = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e}"));
        let Some(mm) = lv.main_map() else { rows.push(format!("{name}: NO MAIN MAP")); continue };
        let mut row = std::collections::BTreeMap::new();
        for p in &lv.maps[mm].placements {
            if p.flags & 0xa000 != 0 { continue; } // the caller's spawn gate [M 0x1d1b68:88]
            if p.flags >= 0x100 { off_range_kinds.push((level, p.flags)); continue; }
            let kind = p.flags & 0xff;
            if !GAME_KINDS.contains(&kind) { continue; } // kinds the switch falls through on
            // The glist must resolve [M 0x1d1b68:76-86].
            let resolved = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < lv.maps[mm].objects.len() };
            let e = counts.get_mut(&kind).unwrap();
            if resolved { e.0 += 1; } else { e.1 += 1; }
            *row.entry(kind).or_insert(0usize) += 1;
        }
        let cells: Vec<String> = row.iter().map(|(k, n)| format!("{k:#04x}:{n}")).collect();
        let tag = if DRIVING_MP.contains(&level) { " [driving/MP]" } else { "" };
        rows.push(format!("{name}{tag}: {}", cells.join(" ")));
    }
    for r in &rows { println!("{r}"); }
    println!();
    println!("== per-kind totals (resolved/unresolved) ==");
    let mut total_all = 0usize;
    for (&k, &(r, u)) in &counts {
        total_all += r + u;
        let arm = PORT_ARMS.iter().find(|(kk, _)| *kk == k).map(|(_, a)| *a).unwrap_or("NO PORT ARM");
        println!("{k:#04x}: {r}/{u}  {arm}");
    }
    println!("total counted placements: {total_all}");
    if !off_range_kinds.is_empty() {
        println!("flags >= 0x100 that pass the spawn gate (dispatch misses): {off_range_kinds:?}");
    }
    // Every kind that occurs must have a port arm entry that is not a closed zero.
    for (&k, &(r, _)) in &counts {
        if r == 0 { continue; }
        let arm = PORT_ARMS.iter().find(|(kk, _)| *kk == k).map(|(_, a)| *a).unwrap_or("NO PORT ARM");
        assert!(!arm.starts_with("nf-game none"), "kind {k:#04x} occurs ({r}) but the census table closes it as zero");
        assert!(arm != "NO PORT ARM", "kind {k:#04x} occurs ({r}) and has no port create arm — port it");
    }
    // The closed zero kinds: the never-called proofs [D census] (drone.rs "Never-called proofs":
    // Sub 0x36, MiniSub 0x37, Creature 0xe0 have no row in the disc placement census).
    for &k in &[0x36u32, 0x37, 0xe0] {
        let (r, u) = counts[&k];
        assert_eq!(r + u, 0, "kind {k:#04x} was closed as zero; a placement showed up");
    }
}
