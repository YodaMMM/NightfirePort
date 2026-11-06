//! probe 0700000d's west-roof guard rays (level-0d-real-health-crossfire-5): which stand gives
//! the bullet a clear flight to the west-roof guards (#27/#28/#29) — their ASLEEP placements
//! (pre-wake, the platform-kill phase) and their SETTLED combat spots (post-wake, the ladder-top
//! phase), plus the driver's lever-1 stands (the route's own roof platform 19.65 and the
//! catwalk). Also a roof map of the guards' building (the slab-edge law) and the wake→reposition
//! timing. Teleport probe (assertion-free diagnostic; never runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const SETTLED: &[(&str, [f32; 3])] = &[
    ("#27 settled", [151.4, 22.7, -245.6]),
    ("#29 settled", [151.3, 22.7, -255.7]),
    ("#28 settled", [151.43, 22.7, -241.25]),
    ("g27 spawn", [151.06, 22.7, -255.81]),
    ("g28 spawn", [146.4, 22.7, -237.7]),
    ("g29 spawn", [151.05, 22.7, -250.82]),
];

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_west_roof_rays() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // 1. The load census: where do the west-roof guards sleep?
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== load census (drones 8..40) ===");
        for (i, d) in sim.drones.list.iter().enumerate().skip(8).take(32) {
            if !d.present { continue; }
            eprintln!("[census] g{i} skin {:#x} pos ({:.2},{:.2},{:.2}) enabled {} present {} corpse {} health {:.0} state {:#x}",
                d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.corpse, d.health, d.state);
        }
    }

    // 2. Rays from every candidate stand to every present guard's chest/head, pre-wake.
    let stands: &[(&str, [f32; 3])] = &[
        ("platform-kill stand", [180.46, 12.19, -207.93]),
        ("dock south edge", [178.0, 11.2, -206.5]),
        ("drop point", [181.0, 7.5, -227.5]),
        ("strip 1", [188.0, 0.5, -228.0]),
        ("strip 2", [196.0, 0.5, -228.5]),
        ("strip 4", [212.0, 0.5, -230.0]),
        ("ladder base", [213.2, 1.5, -234.8]),
        ("ladder top", [212.47, 17.37, -234.51]),
        ("roof platform w", [155.0, 19.65, -268.0]),
        ("roof platform nw", [158.0, 19.65, -268.7]),
        ("catwalk w", [160.0, 13.22, -265.7]),
        ("catwalk mid", [172.0, 13.22, -265.7]),
        ("catwalk e", [184.0, 13.22, -265.7]),
    ];
    {
        eprintln!("=== pre-wake rays: stand -> guard chest (y+0.3) / head (y+0.9) ===");
        let sim = w.scene.sim.as_ref().unwrap();
        let targets: Vec<(String, [f32; 3])> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && !d.corpse)
            .flat_map(|(i, d)| {
                let p = d.pos;
                vec![
                    (format!("g{i}chest"), [p[0], p[1] + 0.3, p[2]]),
                    (format!("g{i}head"), [p[0], p[1] + 0.9, p[2]]),
                ]
            })
            .chain(SETTLED.iter().flat_map(|(n, p)| vec![
                (format!("{n}chest"), [p[0], p[1] + 0.3, p[2]]),
                (format!("{n}head"), [p[0], p[1] + 0.9, p[2]]),
            ]))
            .collect();
        for (label, at) in stands {
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = *at;
                sim.bond.vel = [0.0; 3];
            }
            w.wait(6);
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            for (tn, tp) in &targets {
                let (chest, head) = (*tp, [tp[0], tp[1] + 0.6, tp[2]]);
                let c = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, chest);
                let h = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, head);
                let ch = c.map(|f| format!("{f:.3}")).unwrap_or_else(|| "CLEAR".into());
                let hh = h.map(|f| format!("{f:.3}")).unwrap_or_else(|| "CLEAR".into());
                if ch != "CLEAR" && hh != "CLEAR" && !tn.contains("g27") && !tn.contains("g28") && !tn.contains("g29") { continue; }
                eprintln!("[ray] {label} ({:.1},{:.2},{:.1}) eye_y {:.2} -> {tn}: chest {ch} head {hh}", eye[0], eye[1], eye[2], eye[1]);
            }
        }
    }

    // 3. The guards' building roof map: vertical columns over x 146..158, z -236..-258.
    {
        eprintln!("=== roof map (up-walk columns, surfaces y) ===");
        let sim = w.scene.sim.as_ref().unwrap();
        for x in [146.0f32, 148.0, 150.0, 152.0, 154.0, 156.0] {
            for z in [-236.0f32, -239.0, -242.0, -245.0, -248.0, -251.0, -254.0, -257.0] {
                let mut hits = Vec::new();
                let mut bot = 15.0f32;
                for _ in 0..8 {
                    match sim.world.segment_hit([x, bot, z], [x, 26.0, z]) {
                        Some((f, _)) => { let y = bot + (26.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                        None => break,
                    }
                }
                if !hits.is_empty() {
                    eprintln!("[roof] x {x} z {z}: {:?}", hits.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>());
                }
            }
        }
        // The obstruction the settled rays die on: a west-east scan at y 22.9 (chest height)
        // from the ladder top bearing, plus north-south scans at the settled x.
        for z in [-240.0f32, -243.0, -244.0, -245.0, -246.0, -250.0, -254.0] {
            let mut hits = Vec::new();
            let mut bot = 15.0f32;
            for _ in 0..8 {
                match sim.world.segment_hit([152.0, bot, z], [152.0, 26.0, z]) {
                    Some((f, _)) => { let y = bot + (26.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                    None => break,
                }
            }
            eprintln!("[roof] col x152 z {z}: {:?}", hits.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>());
        }
    }

    // 4. The wake→reposition timing: stand at the drop point (the wake), then watch the
    //    west-roof guards walk off their placements.
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [181.0, 7.5, -227.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(60);
        eprintln!("=== wake timing (Bond parked at the drop point) ===");
        for k in 0..48 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
            if k % 5 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                let line: Vec<String> = sim.drones.list.iter().enumerate()
                    .filter(|(i, d)| [27usize, 28, 29].contains(i) && d.present && !d.corpse)
                    .map(|(i, d)| format!("g{i} ({:.2},{:.2},{:.2}) en {} st {:#x}", d.pos[0], d.pos[1], d.pos[2], d.enabled, d.state))
                    .collect();
                eprintln!("[wake] +{}/50: {}", k, line.join("  |  "));
            }
        }
    }
}
