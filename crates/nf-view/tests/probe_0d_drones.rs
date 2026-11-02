//! probe (assertion-free, `--ignored`): the 0700000d drone placements' firing-relevant stats [D].
//! For every kind-0x0f placement: index, skin (param 0), weapon param (9), sight class (param 14),
//! the `behaviour_util_get` stats-section decode (type, accuracy, aggression, health, reaction),
//! the derived DefaultInit sight/engage/combat numbers, and the spawner wiring (kind 0xf1).
//! Run: NF_ISO=… cargo test -p nf-view --test probe_0d_drones -- --ignored --nocapture

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::Placement;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn param(p: &Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0700000d_drone_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let level = std::env::var("NF_PROBE_LEVEL").unwrap_or_else(|_| "0700000d".into());
    let data = read_archive(&iso, &format!("{level}.bin"));
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let placements = &lv.maps[mm].placements;
    println!("== 0700000d: {} placements ==", placements.len());
    // Spawners first (kind 0xf1): group (param 1), max alive (2), start channel (4).
    for p in placements.iter().filter(|p| p.flags & 0xff == 0xf1) {
        if param(p, 0) > 1 {
            println!("spawner #{}: group {} max_alive {} start_ch {} at ({:.1},{:.1},{:.1})",
                p.index, param(p, 1), param(p, 2), param(p, 4),
                p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    for p in placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
        let skin = param(p, 0);
        let mut table = nf_formats::drone::DroneStatsTable::default();
        let ty = table.behaviour_get(&|k| param(p, k));
        let stats = table.stats(ty);
        let class = param(p, 14);
        // The spawn role: same rules as nf_formats::drone::spawn_roles (group param 4 vs the spawners).
        println!("drone grp{}: skin {:08x} wparam {} class {} at ({:.1},{:.1},{:.1})",
            param(p, 4), skin, param(p, 9), class, p.translation[0], p.translation[1], p.translation[2]);
        let pct = (100.0 - 5.0 * stats.accuracy as f32) * 0.6;
        println!("   stats: type {} acc {} aggr {} hp {} react {} recov {} b6 {} b7 {} ba {}  raw_hit% {pct:.1}",
            ty, stats.accuracy, stats.aggression, stats.health, stats.reaction, stats.recover, stats.b6, stats.b7, stats.ba);
        println!("   params: {:?}", p.params.iter().filter(|(k, _)| *k > 10 && *k < 30).collect::<Vec<_>>());
    }
}
