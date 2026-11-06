//! probe (assertion-free, `--ignored`): which drone placements carry behaviour property 0x52 —
//! the property `DroneFunc_FirstAttack` 0x148010:95-98 reads on the first-attack tail
//! (`behaviour_util_getProperty(0x52, Drone+0x4d8)` → `DroneFunc_SetMissionFailReason(drone, 8)`
//! = `Mission_SetFailLabel(0x4000033 "Cover Lost")` + `switch_channels[99]` [M 0x13e560:19-31]).
//! The port's `first_attack` does not make that call yet (first-attack-fail-reason).
//! The words are the placements' own behaviour-property words (params 18/19/20 — the same [D]
//! source the load path's `behaviour_words` uses) read through the disc's ACTION.ELF tables —
//! the same law `difficulty_placements_live` uses for the captain census.
//! Run: NF_ISO=… cargo test -p nf-view --test probe_prop52_census -- --ignored --nocapture

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

const LEVELS: [&str; 22] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d", "07000011",
    "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b", "07000041",
    "0700004a",
];

fn read_level(iso: &str, archive: &str) -> Level {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    let bytes = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    Level::parse(&bytes).expect("parse")
}

fn tables(iso: &str) -> nf_formats::drone::PropertyTables {
    let img = IsoImage::open(iso).expect("open iso");
    let entry = img.find("/ACTION.ELF").expect("ACTION.ELF");
    let elf = img.read_file(entry).expect("read elf");
    nf_formats::drone::PropertyTables::from_action_elf(&elf).expect("tables")
}

fn param_of(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_prop52_census() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let t = tables(&iso);
    for name in LEVELS {
        let level = read_level(&iso, &format!("{name}.bin"));
        let mut hits = 0;
        for (mi, m) in level.maps.iter().enumerate() {
            for (i, p) in m.placements.iter().enumerate() {
                if p.flags & 0xff != 0x0f { continue; } // the drone placements [M 0x136be0]
                let words = nf_formats::drone::behaviour_words(&|k| param_of(p, k)).0;
                let v = t.get(0x52, &words);
                if v == 0 { continue; }
                hits += 1;
                let setup = nf_formats::drone::drone_setup(param_of(p, 0), param_of(p, 9), 0);
                let pr = |id: usize| t.get(id, &words) != 0;
                eprintln!("[0x52] {name} map{mi} placement#{i} skin-family {} kind {} pos ({:.2},{:.2},{:.2}) prop52 {v:#x} 1a/1b/18/19 {}/{}/{}/{} params18/19/20 {:#x}/{:#x}/{:#x}",
                    setup.kind, param_of(p, 0), p.translation[0], p.translation[1], p.translation[2],
                    pr(0x1a) as u8, pr(0x1b) as u8, pr(0x18) as u8, pr(0x19) as u8,
                    param_of(p, 18), param_of(p, 19), param_of(p, 20));
            }
        }
        eprintln!("[census] {name}: {hits} drone placement(s) with prop 0x52");
    }
    eprintln!("== done ==");
}
