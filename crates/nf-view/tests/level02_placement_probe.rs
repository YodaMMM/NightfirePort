//! real-health-00agent-02-5 probe: the 07000002 [D] placement facts the geisha-1 stand-off
//! economy needs — the drone roster (skin, held weapon, enable/second channel, captor link,
//! spawn spot) and the placed pickups (kind, weapon, rounds, spot) as the level loads. Prints
//! only; the walkreal_07000002_live tests stay the pass/fail side.

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000002_drones_and_pickups() {
    let Some(iso) = std::env::var_os("NF_ISO").map(|s| s.into_string().unwrap()) else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("== drones ({} total) ==", sim.drones.list.len());
    for (i, d) in sim.drones.list.iter().enumerate() {
        eprintln!(
            "drone {i}: skin {:#x} weapon {:#x} team {} health {:.1} enabled {} ch2 {:#x} captor {:?} pos ({:.2},{:.2},{:.2})",
            d.skin, d.weapon, d.team, d.health, d.enabled, d.second_channel, d.captor,
            d.pos[0], d.pos[1], d.pos[2]);
    }
    eprintln!("== pickups ({} total) ==", sim.drones.pickups.len());
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        eprintln!(
            "pickup {i}: kind {:#x} weapon {:#x} rounds {} hidden {} pos ({:.2},{:.2},{:.2})",
            p.kind, p.weapon, p.rounds, p.hidden, p.m[3][0], p.m[3][1], p.m[3][2]);
    }
}
