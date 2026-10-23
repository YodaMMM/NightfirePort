//! level-1b-drake-fight probe: the built sim's AI volumes (the game's kind-3 combat box is
//! Drake's strafe-candidate pool [M DroneMove_AstronautCombat 0x154618:94]) and whether the
//! fight's key spots sit inside: Drake's placement (−7.32,−1.42,0.06), the island top, the
//! mid-pit, the deck's SW corner. Assertion-free.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0700001b_built_volumes() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    println!("=== built AI volumes: {} ===", sim.volumes.len());
    for (i, v) in sim.volumes.iter().enumerate() {
        println!("vol[{i}] placement {} kind {} centre ({:.2},{:.2},{:.2}) half ({:.2},{:.2},{:.2})",
            v.placement, v.kind, v.m[3][0], v.m[3][1], v.m[3][2], v.half[0], v.half[1], v.half[2]);
        for (name, p) in [
            ("drake placement", [-7.32f32, -1.42, 0.06]),
            ("mid-pit", [5.0, -1.6, 0.0]),
            ("deck SW corner", [12.0, -1.6, -6.5]),
            ("island top", [2.0, -0.5, 0.0]),
            ("arena", [-7.0, -1.6, 0.1]),
        ] {
            println!("   {name} {p:?}: {}", if v.contains(p) { "INSIDE" } else { "outside" });
        }
    }
}
