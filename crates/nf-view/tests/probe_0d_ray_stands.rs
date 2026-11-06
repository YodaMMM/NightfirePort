//! probe (assertion-free, `--ignored`): the level-0d-real-health-crossfire-3 stand scan. The -2
//! audit measured the two remaining drains (#10's 46 at the ladder top, #29's 54 on the ride) and
//! the slab-edge law (the eye→chest ray to the west-roof guard dies at 0.972-0.974 from the
//! ladder top). The driver's lever: a stand whose static eye→chest ray IS clear, used before Bond
//! goes up. This probe teleports Bond to every candidate stand the route can reach BEFORE the
//! ladder (the warehouse floor, the dock, the strip, the trench south) plus the ladder top/wire,
//! and prints the same eye→chest ray through `segment_hit`/`segment_hit_static`/
//! `segment_bullet_scoped` to EVERY live guard in the region — the matrix the route redesign
//! reads. Also dumps the level's own pickups [D placements] (the driver's pickups lever).
//! Run: NF_ISO=… cargo test -p nf-view --test probe_0d_ray_stands -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_ray_stands() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);

    // Part A: the pickup placements the level itself offers [D] — kind/weapon/pos.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== A. pickups (the level's own offers) ==");
        for (i, p) in sim.drones.pickups.iter().enumerate() {
            let pos = [p.m[3][0], p.m[3][1], p.m[3][2]];
            if !(100.0..=240.0).contains(&pos[0]) { continue; }
            eprintln!("[pickup] #{i} kind {} weapon {:#x} rounds {} pos ({:.2},{:.2},{:.2}) state {}",
                p.kind, p.weapon, p.rounds, pos[0], pos[1], pos[2], p.state);
        }
    }

    // Part B: the shooter census (the same region filter the climb probe used).
    let guards: Vec<(usize, [f32; 3], u32, u32)> = {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== B. drone census (x 140..230, z -300..-215) ==");
        let mut v = Vec::new();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.corpse || d.health <= 0.0 { continue; }
            if !(140.0..=230.0).contains(&d.pos[0]) || !(-300.0..=-215.0).contains(&d.pos[2]) { continue; }
            eprintln!("[census] drone #{i} skin {:#x} weapon {:#x} pos ({:.2},{:.2},{:.2}) enabled {} present {} state {:#x} sight {:.0}",
                d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.state, d.sight_range);
            v.push((i, d.pos, d.weapon, d.skin));
        }
        v
    };

    // Part C: the stands × guards ray matrix. Every stand is a teleport + settle (probes are
    // measurement tools; the walk itself stays teleport-free). The stands cover the EARLY route
    // (before the ladder): the warehouse floor, the dock, the drop point, the blind strip, the
    // trench south, the ladder base — plus the ladder top and the wire mid (the -2 measured
    // stands, for reference rows).
    let stands: [(&str, [f32; 3]); 22] = [
        ("armour arrive", [180.46, 12.19, -207.93]),
        ("warehouse a", [178.0, 12.2, -212.0]),
        ("warehouse b", [184.0, 12.2, -215.0]),
        ("dock edge", [178.0, 12.2, -206.5]),
        ("dock east", [194.0, 13.5, -232.0]),
        ("drop point", [181.0, 7.5, -227.5]),
        ("strip w1", [188.0, 0.5, -228.0]),
        ("strip w2", [196.0, 0.5, -228.5]),
        ("strip e3", [204.0, 0.5, -229.0]),
        ("strip e4", [212.0, 0.5, -230.0]),
        ("crane leg", [215.0, 0.8, -233.2]),
        ("ladder base", [213.2, 1.5, -234.8]),
        ("trench a", [210.0, 0.5, -240.0]),
        ("trench b", [210.0, 0.5, -250.0]),
        ("trench c", [208.0, 0.5, -258.0]),
        ("trench d", [206.0, 0.5, -263.0]),
        ("trench e", [202.0, 0.5, -248.0]),
        ("trench f", [214.0, 0.5, -245.0]),
        ("ladder top", [212.5, 17.4, -234.5]),
        ("wire mid", [205.0, 17.2, -248.5]),
        ("strip s", [206.0, 0.5, -236.0]),
        ("strip s2", [200.0, 0.5, -240.0]),
    ];
    for (label, sp) in stands {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = sp;
            sim.bond.vel = [0.0; 3];
            sim.bond.substate = 0; // walk state
        }
        for _ in 0..8 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        eprintln!("== stand {label} settled ({:.2},{:.2},{:.2}) eye {:.2}", bp[0], bp[1], bp[2], eye[1]);
        for (gi, gp, gweapon, gskin) in &guards {
            let chest = [gp[0], gp[1] + 0.3, gp[2]];
            let dist = ((eye[0] - chest[0]).powi(2) + (eye[2] - chest[2]).powi(2)).sqrt();
            let full = sim.world.segment_hit(eye, chest);
            let stat = sim.world.segment_hit_static(eye, chest);
            let bullet = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, chest);
            let fmt = |r: Option<(f32, [f32; 3])>, scoped: bool| match r {
                Some((f, p)) => if scoped { format!("{f:.3}") } else { format!("{f:.3}@y{:.1}", p[1]) },
                None => "CLEAR".into(),
            };
            eprintln!("[ray] {label} -> g{gi} skin {gskin:#x} wpn {gweapon:#x} d {dist:.1} full {} static {} bullet {}",
                fmt(full, false), fmt(stat, false), bullet.map(|f| format!("{f:.3}")).unwrap_or_else(|| "CLEAR".into()));
        }
    }
    eprintln!("== done ==");
}
