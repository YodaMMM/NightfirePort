//! cameras-turrets-census: the port creates every camera/turret-family object the on-foot SP
//! levels carry, through the real placement creators.
//!
//! The [D] counts below are read straight from this ISO's level bins (placement kind =
//! `flags & 0xff`, the creators' shared spawn gate `flags & 0xa000 != 0` skipped, main map
//! only — the same map `parsemap_create_dynamic_objects` 0x1d04f0 walks):
//!
//! | bin | GT 0x27 | sensor 0x28 | monitor 0x30 | gunimp 0x34 | shooter 0xd2 |
//! |---|---|---|---|---|---|
//! | 07000001 Double Cross 1 | – | 1 | 1 | – | 1 |
//! | 07000002 Double Cross 2 | – | 2 | 1 | – | – |
//! | 0700000a Night Shift 2 | 6 | 2 | 1 | – | – |
//! | 0700000b Night Shift 3 | 5 | 2 | – | – | – |
//! | 0700000d Chain Reaction 2 | – | – | – | 1 | – |
//! | 07000012 Phoenix Fire 2 | 3 | 5 | 1 | – | – |
//! | 07000014 Countdown 1 | – | 14 | 12 | – | – |
//! | 07000015 Countdown 2 | – | – | – | – | 1 |
//! | 07000041 (Deep Descent part) | – | – | – | – | 1 |
//! | 0700004a Phoenix Fire 3 (elevator) | – | – | – | – | 2 |
//! | 0700001b Equinox | – | – | – | – | – |
//!
//! Every camera-family creator: `parsemap_create_dynamic_objects` 0x1d04f0 case 0x27
//! `GT_Create` 0x17ffa0 (class 0x36), case 0x28 `Sensor_Create` 0x1af010 (class 0x37),
//! case 0x30 `Monitor_Create` 0x1b08b0 (class 0x38), case 0x34 `GunImp_Create` 0x131f20
//! (class 0x47), case 0xd2 `Shooter_Create` 0x17d030 (class 0x4e) [M all].
//! The searchlight (case 0xde) is its own system (crates/nf-game/src/searchlight.rs), not a
//! camera. The per-level details live in docs/systems/cameras-turrets.md.
//!
//! Opt-in: `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::{disc, levelbin::LevelBin};

/// The gated main-map placement count for one kind.
fn count_kind(level: &Level, kind: u32) -> usize {
    let m = level.main_map().expect("main map");
    level.maps[m].placements.iter().filter(|p| p.flags & 0xff == kind && p.flags & 0xa000 == 0).count()
}

fn load(iso: &IsoImage<std::fs::File>, bin: &str) -> (Level, LevelBin, Vec<u8>) {
    let (entries, files) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(bin)).unwrap_or_else(|| panic!("{bin} not in FILES.BIN"));
    let data = iso.read_bytes(files.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    (Level::parse(&data).unwrap(), LevelBin::parse(&data).unwrap(), data)
}

/// The port's creators over one level: (sensors, monitors, gunimps, shooters, gt_from_placement).
fn port_counts(level: &Level) -> (usize, usize, usize, usize, usize) {
    let m = level.main_map().expect("main map");
    let paths = level.path_blocks();
    let sensors = nf_game::sensor::Sensors::create(&level.maps[m].placements, &paths).list.len();
    let monitors = nf_game::monitor::Monitors::create(level).list.len();
    let props = nf_game::props::Props::create(level);
    let world = nf_game::worldobj::WorldObjects::create(&level.maps[m].placements, &paths, |_| (0, [0.0; 3]));
    let gts = level.maps[m].placements.iter()
        .filter(|p| p.flags & 0xff == 0x27 && p.flags & 0xa000 == 0)
        .filter(|p| nf_game::gt::from_placement(0, &p.params).is_some()).count();
    (sensors, monitors, props.gunimps.len(), world.shooters.len(), gts)
}

/// (bin, sensors, monitors, gunimps, shooters, gts) — the [D] census above.
const CENSUS: [(&str, usize, usize, usize, usize, usize); 11] = [
    ("07000001.bin", 1, 1, 0, 1, 0),
    ("07000002.bin", 2, 1, 0, 0, 0),
    ("0700000a.bin", 2, 1, 0, 0, 6),
    ("0700000b.bin", 2, 0, 0, 0, 5),
    ("0700000d.bin", 0, 0, 1, 0, 0),
    ("07000012.bin", 5, 1, 0, 0, 3),
    ("07000014.bin", 14, 12, 0, 0, 0),
    ("07000015.bin", 0, 0, 0, 1, 0),
    ("07000041.bin", 0, 0, 0, 1, 0),
    ("0700004a.bin", 0, 0, 0, 2, 0),
    ("0700001b.bin", 0, 0, 0, 0, 0),
];

#[test]
fn every_camera_turret_placement_reaches_its_port_creator() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap();
    for (bin, s, mo, gi, sh, gt) in CENSUS {
        let (level, _lb, _data) = load(&iso, bin);
        // The level carries what the census says [D].
        assert_eq!(count_kind(&level, 0x28), s, "{bin}: sensor placements");
        assert_eq!(count_kind(&level, 0x30), mo, "{bin}: monitor placements");
        assert_eq!(count_kind(&level, 0x34), gi, "{bin}: gunimp placements");
        assert_eq!(count_kind(&level, 0xd2), sh, "{bin}: shooter placements");
        assert_eq!(count_kind(&level, 0x27), gt, "{bin}: GT placements");
        // The port's creators produce exactly that [M 0x1af010/0x1b08b0/0x131f20/0x17d030/0x17ffa0].
        let (ps, pmo, pgi, psh, pgt) = port_counts(&level);
        assert_eq!(ps, s, "{bin}: Sensors::create");
        assert_eq!(pmo, mo, "{bin}: Monitors::create");
        assert_eq!(pgi, gi, "{bin}: Props::create gunimps");
        assert_eq!(psh, sh, "{bin}: WorldObjects::create shooters");
        assert_eq!(pgt, gt, "{bin}: gt::from_placement (the script-presence gate rides the full load, see gun_turrets_live)");
        let _ = _lb;
    }
}
