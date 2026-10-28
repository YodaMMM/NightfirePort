//! level-01-exit-after-the-wait probe: dump 07000001's mission-kind AI paths (kind & 8) — node
//! positions + flags — so the escort route's END (where the game's AllyLeadDone parks Mayhew,
//! [M NDrone2_DSTATE_AllyLeadDone 0x15f2e8: UPDATE returns 1, no movement]) can be compared with
//! the LoadPointActivator plate volume. Diagnostic only — run with --nocapture.

use nf_formats::iso9660::IsoImage;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_07000001_mission_paths() {
    let img = IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000001.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = nf_formats::level::Level::parse(&data).unwrap();
    let world = nf_game::world::CollisionWorld::build(&level, &level.instances(true).0);
    let nav = nf_game::nav::Network::build(level.maps[level.main_map().unwrap()].ai_network.as_ref().unwrap(), &world, 0x0700_0001);
    let mut any = false;
    for (pi, p) in nav.paths.iter().enumerate() {
        if p.kind & 8 == 0 { continue; }
        any = true;
        eprintln!("=== path {pi} kind {:#x}: {} nodes, {} links", p.kind, p.nodes.len(), p.links.len());
        for (ni, n) in p.nodes.iter().enumerate() {
            eprintln!("  node {ni:3} ({:8.3},{:7.3},{:8.3}) flags {:#x}", n.pos[0], n.pos[1], n.pos[2], n.flags);
        }
    }
    assert!(any, "07000001 carries no kind-8 mission path");
    // The plate for reference (the AND-touch out 0x10).
    // Path node 33 is the ally-wait node (the door hold); print its neighbours.
}
