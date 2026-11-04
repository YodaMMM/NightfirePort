//! Probe 7 (level-14-tripbomb-real-route-2): the thread around the 1.10 crate at ≈(-33, 29.4)
//! that blocks the lane's south mouth → north run. Floor grid + clearance rays over the
//! x -34.8..-33.0, z 28.6..31.5 band, then teleport-scout walks of the candidate threads
//! (scouting only — the live test walks everything for real).

#[path = "common/walker_b2.rs"]
mod walker;
use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_lane_south_thread() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_ref().unwrap();
        let xs = [-34.8f32, -34.6, -34.4, -34.2, -34.0, -33.8, -33.6, -33.4, -33.2, -33.0];
        for z in [28.6f32, 28.9, 29.2, 29.5, 29.8, 30.1, 30.4, 30.7, 31.0, 31.3] {
            let mut row = String::new();
            for x in xs {
                let cell = match sim.world.floor_below([x, 5.0, z], 10.0) {
                    Some(v) => format!("({x:.1},{v:.2})"),
                    None => format!("({x:.1},--)"),
                };
                row.push_str(&format!("{cell} "));
            }
            eprintln!("[probe7] z {z}: {row}");
        }
        // Head clearance up the two candidate threads: a floor + a 1.5-high segment northward
        // at each candidate x (the standing capsule needs ~1.5 clear).
        for x in [-34.4f32, -34.0, -33.6, -33.3] {
            for z0 in [28.8f32, 29.4, 30.0, 30.6] {
                let head = sim.world.segment([x, 1.55, z0], [x, 1.55, z0 + 1.2]);
                let floor = sim.world.floor_below([x, 5.0, z0 + 0.6], 10.0);
                eprintln!("[probe7] x {x} z {z0}..{:.1}: floor {floor:?} head-ray {head:?}", z0 + 1.2);
            }
        }
    }
    drop(scene);
    // The teleport-scout walks of the candidate threads (the probe pattern: scouting only).
    for (tag, legs) in [
        ("west-of-crate", vec![
            Wp::new(-33.08, 0.68, 28.60).r(0.5),
            Wp::new(-33.7, 0.7, 29.2).r(0.4),
            Wp::new(-33.6, 0.75, 30.9).r(0.5),
            Wp::new(-33.3, 0.8, 33.3).r(0.6),
            Wp::new(-33.2, 0.85, 35.7).r(0.4),
        ]),
        ("mid-x", vec![
            Wp::new(-33.08, 0.68, 28.60).r(0.5),
            Wp::new(-33.5, 0.7, 29.2).r(0.4),
            Wp::new(-33.4, 0.75, 30.9).r(0.5),
            Wp::new(-33.3, 0.8, 33.3).r(0.6),
            Wp::new(-33.2, 0.85, 35.7).r(0.4),
        ]),
    ] {
        let mut w = Walker::new(nf_view::load(&iso_path, "07000014.bin", false).unwrap());
        w.wait_enabled(3000, "the intro");
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [-33.08f32, 0.68, 28.60];
            sim.bond.vel = [0.0; 3];
            for _ in 0..15 { sim.step_bond(Default::default(), false, false, false); }
        }
        eprintln!("[probe7] {tag}: scouted from {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        for (k, wp) in legs.iter().enumerate() {
            let ok = w.try_walk_to(wp.clone(), &format!("{tag} leg {k}"));
            eprintln!("[probe7] {tag} leg {k} {:?}: ok {ok} bond {:?}",
                wp.at, w.scene.sim.as_ref().unwrap().bond.pos);
        }
    }
    // The WEST thread scout: the only body-clear crossing (z 33.0-33.25 [L probe6's ray grid])
    // is a steering problem, not geometry. Floors along z 33.05 first, then the fine-leg scout
    // from the catwalk head.
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_ref().unwrap();
        let mut row = String::new();
        for x in [-38.0f32, -37.6, -37.2, -36.8, -36.4, -36.0, -35.6, -35.2, -34.8, -34.4, -34.0, -33.6] {
            let cell = match sim.world.floor_below([x, 6.0, 33.05], 12.0) {
                Some(v) => format!("({x:.1},{v:.2})"),
                None => format!("({x:.1},--)"),
            };
            row.push_str(&format!("{cell} "));
        }
        eprintln!("[probe7] floor at z 33.05: {row}");
        let mut row2 = String::new();
        for x in [-38.0f32, -37.6, -37.2, -36.8, -36.4, -36.0, -35.6, -35.2, -34.8, -34.4, -34.0, -33.6] {
            let cell = match sim.world.floor_below([x, 6.0, 33.35], 12.0) {
                Some(v) => format!("({x:.1},{v:.2})"),
                None => format!("({x:.1},--)"),
            };
            row2.push_str(&format!("{cell} "));
        }
        eprintln!("[probe7] floor at z 33.35: {row2}");
    }
    drop(scene);
    let mut w = Walker::new(nf_view::load(&iso_path, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-38.27f32, 0.45, 33.21];
        sim.bond.vel = [0.0; 3];
        for _ in 0..15 { sim.step_bond(Default::default(), false, false, false); }
    }
    eprintln!("[probe7] west-thread: scouted from {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    for (k, wp) in [
        Wp::new(-37.9, 0.5, 33.15).r(0.25),
        Wp::new(-37.4, 0.55, 33.1).r(0.25),
        Wp::new(-36.9, 0.6, 33.05).r(0.25),
        Wp::new(-36.4, 0.65, 33.05).r(0.25),
        Wp::new(-35.9, 0.7, 33.05).r(0.25),
        Wp::new(-35.4, 0.75, 33.05).r(0.25),
        Wp::new(-34.9, 0.8, 33.05).r(0.25),
        Wp::new(-34.3, 0.8, 33.1).r(0.3),
        Wp::new(-33.6, 0.8, 33.15).r(0.3),
        Wp::new(-33.3, 0.8, 33.3).r(0.4),
        Wp::new(-33.2, 0.85, 34.4).r(0.4),
        Wp::new(-33.2, 0.85, 35.7).r(0.4),
    ].iter().enumerate() {
        let ok = w.try_walk_to(wp.clone(), &format!("west leg {k}"));
        eprintln!("[probe7] west leg {k} {:?}: ok {ok} bond {:?} grounded {}",
            wp.at, w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.grounded);
    }
}

#[test]
fn probe_lane_south_slalom() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_ref().unwrap();
        // Fine floor map: z 32.3..33.0 rows across the sill/crate/shelf band.
        for z in [32.3f32, 32.5, 32.7, 32.9, 33.0] {
            let mut row = String::new();
            for x in [-37.6f32, -37.4, -37.2, -37.0, -36.8, -36.6, -36.4, -36.2, -36.0, -35.8, -35.6, -35.4, -35.2, -35.0] {
                let cell = match sim.world.floor_below([x, 6.0, z], 12.0) {
                    Some(v) => format!("({x:.1},{v:.2})"),
                    None => format!("({x:.1},--)"),
                };
                row.push_str(&format!("{cell} "));
            }
            eprintln!("[probe8] z {z}: {row}");
        }
    }
    drop(scene);
    // The slalom scout: sill top → step down SOUTH of the 1.45 crate → east at floor 0.8 →
    // north-east under the shelf → the lane floor → the rack front.
    let mut w = Walker::new(nf_view::load(&iso_path, "07000014.bin", false).unwrap());
    w.wait_enabled(3000, "the intro");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-38.27f32, 0.45, 33.21];
        sim.bond.vel = [0.0; 3];
        for _ in 0..15 { sim.step_bond(Default::default(), false, false, false); }
    }
    for (k, wp) in [
        Wp::new(-37.9, 0.5, 33.15).r(0.25),
        Wp::new(-37.45, 0.6, 33.1).r(0.25),
        Wp::new(-37.0, 0.75, 32.7).r(0.3),
        Wp::new(-36.6, 0.8, 32.6).r(0.3),
        Wp::new(-36.1, 0.8, 32.8).r(0.3),
        Wp::new(-35.6, 0.8, 33.0).r(0.3),
        Wp::new(-35.0, 0.8, 33.15).r(0.3),
        Wp::new(-34.3, 0.8, 33.2).r(0.4),
        Wp::new(-33.6, 0.8, 33.2).r(0.4),
        Wp::new(-33.3, 0.8, 33.3).r(0.4),
        Wp::new(-33.2, 0.85, 34.4).r(0.4),
        Wp::new(-33.2, 0.85, 35.7).r(0.4),
    ].iter().enumerate() {
        let ok = w.try_walk_to(wp.clone(), &format!("slalom leg {k}"));
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe8] slalom leg {k} {:?}: ok {ok} bond {:?} grounded {}",
            wp.at, sim.bond.pos, sim.bond.grounded);
    }
}

#[test]
fn probe_lane_nav_route() {
    let iso_path = iso();
    let scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    // What does the game's own nav web say from the armoury branch to the rack front?
    for (tag, from, to) in [
        ("branch->rack", [-30.34f32, 0.6, 23.13], [-33.2f32, 0.85, 35.7]),
        ("branch->lane-mid", [-30.34f32, 0.6, 23.13], [-33.3f32, 0.8, 33.3]),
        ("catwalk-head->rack", [-38.27f32, 0.6, 33.21], [-33.2f32, 0.85, 35.7]),
    ] {
        match walker::nav_route(sim, from, to) {
            Some(pts) => {
                eprintln!("[probe9] {tag}: {} nodes", pts.len());
                for (k, p) in pts.iter().enumerate() {
                    eprintln!("[probe9]   {k}: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
                }
            }
            None => eprintln!("[probe9] {tag}: NO nav route"),
        }
    }
    // The lane's own nav nodes: scan the web for nodes inside the lane box (x -35.5..-32,
    // z 29..37.5, y < 2.5) — the game's own idea of how the lane is entered.
    let mut lane_nodes: Vec<[f32; 3]> = Vec::new();
    for path in &sim.nav.paths {
        for n in &path.nodes {
            let p = n.pos;
            if (-35.5..=-32.0).contains(&p[0]) && (29.0..=37.5).contains(&p[2]) && p[1] < 2.5 {
                lane_nodes.push(p);
            }
        }
    }
    lane_nodes.sort_by(|a, b| a[2].total_cmp(&b[2]));
    eprintln!("[probe9] {} lane-box nav nodes:", lane_nodes.len());
    for p in &lane_nodes { eprintln!("[probe9]   lane node ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
}
