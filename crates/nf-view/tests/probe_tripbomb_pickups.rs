//! Probe (level-14-tripbomb-real-route): where the 07000014 PICKUP_Tripbomb placements resolve.
//! Walks EVERY map's 0xF0 placements (the audit's #1341/#1342 are at -32.19,1.41,31.12/35.86
//! [D docs/levels/07000014-audit.md]; map-local vs world is what this pins), prints each map's
//! 0xF0 entries and the sim's own pickup list after `nf_view::load`.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_tripbomb_pickup_placements() {
    let iso_path = iso();
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    eprintln!("[probe] maps {} main_map {:?}", level.maps.len(), level.main_map());
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            if p.flags & 0xff == 0xf0 {
                let id = p.params.iter().find(|(i, _)| *i == 1).map(|(_, v)| *v).unwrap_or(0);
                let ty = p.params.iter().find(|(i, _)| *i == 0).map(|(_, v)| *v).unwrap_or(0);
                if id == 0x3a || ty == 0 {
                    eprintln!("[probe] map {mi} placement {pi}: t=({:.2},{:.2},{:.2}) params {:?} index {}",
                        p.translation[0], p.translation[1], p.translation[2], p.params, p.index);
                }
            }
        }
    }
    // The sim's own resolution.
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[probe] bond spawn {:?}", sim.bond.pos);
    for (k, p) in sim.drones.pickups.iter().enumerate() {
        eprintln!("[probe] sim pickup {k}: kind {} weapon {:#x} rounds {} state {} at ({:.3},{:.3},{:.3})",
            p.kind, p.weapon, p.rounds, p.state, p.m[3][0], p.m[3][1], p.m[3][2]);
    }
}
