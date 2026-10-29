//! walkreal probe 3 for 0700000c: the last unknowns — the ring→band connection strip (x 36..56,
//! z −202..−212, under the 15.7 roof), the ladder-top surroundings, the ring west/south-west
//! sides, the skylight photograph segment (can the walkway shoot the display?), and the
//! display→warehouse floor descent. Diagnostic only.

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_0700000c_last() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("--- ring-to-band strip (x 34..56 step 2, z -202..-213 step 1, y 13->6) ---");
    for z in (-213..=-202).rev() {
        let z = z as f32;
        let mut row = String::new();
        for x in (34..=56).step_by(2) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 13.0, z], [x, 6.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 13.0 - 7.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:      {}", (34..=56).step_by(2).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- ladder top surroundings (x 6..15 step 1, z -183..-194 step 1, y 13->9) ---");
    for z in (-194..=-183).rev() {
        let z = z as f32;
        let mut row = String::new();
        for x in 6..=15 {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 13.0, z], [x, 9.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 13.0 - 4.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:      {}", (6..=15).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- ring west/south (x 0..14 step 2, z -200..-224 step 2, y 13->8) ---");
    for z in (-224..=-200).rev().step_by(2) {
        let z = z as f32;
        let mut row = String::new();
        for x in (0..=14).step_by(2) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 13.0, z], [x, 8.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 13.0 - 5.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:      {}", (0..=14).step_by(2).map(|x| format!("{x:>4}")).collect::<String>());

    eprintln!("--- display -> warehouse descent (x 8..56 step 2, z -110..-160 step 2, y 8->-1) ---");
    for z in (-160..=-110).rev().step_by(2) {
        let z = z as f32;
        let mut row = String::new();
        for x in (8..=56).step_by(2) {
            let x = x as f32;
            let hit = sim.world.segment_hit([x, 8.0, z], [x, -1.0, z]);
            match hit {
                Some((t, _)) => row.push_str(&format!("{:5.1}", 8.0 - 9.0 * t)),
                None => row.push_str("    ."),
            }
        }
        eprintln!("z={z:6.1}: {row}");
    }
    eprintln!("    x:      {}", (8..=56).step_by(2).map(|x| format!("{x:>4}")).collect::<String>());

    // The walkway photograph: can a camera shot from the walkway north end reach subject #414?
    // Segment from the eye at the walkway north end to the jetpack centre.
    eprintln!("--- photograph segment checks (world.segment eye->jetpack, hit means blocked) ---");
    let eye_spots: &[(&str, [f32; 3])] = &[
        ("walkway north end root 15.04", [5.79, 16.1, -111.1]),
        ("walkway north end eye ~16.9", [5.79, 16.9, -111.1]),
        ("walkway at z -113", [5.79, 16.9, -113.0]),
        ("walkway at z -116", [5.79, 16.9, -116.0]),
        ("display ground (the old stand-in)", [5.4, 8.4, -113.0]),
    ];
    for (name, eye) in eye_spots.iter() {
        for (tname, tgt) in [("subject414 centre", [5.49, 7.59, -110.37]), ("subject454 centre", [5.31, 8.77, -109.14])] {
            let seg = sim.world.segment(*eye, tgt);
            eprintln!("  {name} -> {tname}: segment={seg:?}");
        }
    }

    eprintln!("--- start walkway walk north: head bumps / wall scan along x 5.8, z -148..-110 (segment at root+1, eye) ---");
    for z in (-110..=-149).rev().step_by(2) {
        let z = z as f32;
        let floor = sim.world.segment_hit([5.79, 16.5, z], [5.79, 10.5, z]).map(|(t, _)| 16.5 - 6.0 * t);
        let head = sim.world.segment([5.79, 15.3, z], [5.79, 17.5, z]);
        eprintln!("  z={z:6.1}: floor={floor:?} headroom_seg={head:?}");
    }

    eprintln!("--- loadout weapons ---");
    for (i, owned) in sim.drones.inventory.owned.iter().enumerate() {
        if *owned { eprintln!("  owned weapon {:#x}", i); }
    }
}
