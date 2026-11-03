//! probe (assertion-free, `--ignored`): the PLATFORM KILL stand map for the 0d dock patroller
//! #33. The report lever: from the warehouse platform's south edge the shot at #33 (census
//! (191.26, 13.66, −232.22), walking east ~0.003 u/tick) is nearly flat, so the dock's south
//! face cannot eat the ray. First probe stand (179, 11.2, −226.5) walked straight OFF the edge
//! (the walker arrived at y 1.10 — the edge is north of z −225.3). This probe maps the real
//! edge: walk Bond south along the platform in small steps (real movement, no teleport) and
//! print y + the flat rays to #33/#31/#16 where the floor still holds. Also reads #33's x over
//! time (its west window). Run: NF_ISO=… cargo test -p nf-view --test probe_0d_platform -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_platform_edge_and_rays() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let drone_idx = |w: &Walker, x: f32, z: f32, tol: f32| -> Option<usize> {
        w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
            d.enabled && d.present && !d.corpse && d.health > 0.0
                && (d.pos[0] - x).abs() < tol && (d.pos[2] - z).abs() < tol
        })
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.corpse || d.health <= 0.0 { continue; }
            if d.pos[0] > 150.0 && d.pos[0] < 230.0 && d.pos[2] < -215.0 && d.pos[2] > -260.0 {
                eprintln!("[census] #{i} skin {:#x} pos ({:.2},{:.2},{:.2}) health {:.1} enabled {} yaw {:.2} half_fov {:.3} sight_range {:.0} state {:#x}",
                    d.skin, d.pos[0], d.pos[1], d.pos[2], d.health, d.enabled, d.yaw, d.half_fov, d.sight_range, d.state);
            }
        }
        eprintln!("[bond] start pos {:?} weapon {:#x} clip {} spare {}", sim.bond.pos, sim.gun.weapon, sim.gun.clip, sim.gun.spare);
    }

    // The edge map: real steps south along x 178, printing y every 10 ticks.
    eprintln!("== A. the platform south edge map (real steps, x ~178) ==");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let desired = (200.0f32 - sim.bond.pos[0]).atan2(-232.0 - sim.bond.pos[2]);
        for _ in 0..30 {
            let sim = w.scene.sim.as_mut().unwrap();
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
        }
    }
    let mut fell_at = None;
    for k in 0..400 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 0.5, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if p[1] < 9.0 && fell_at.is_none() {
            fell_at = Some((k, p));
            eprintln!("[edge] y dropped below 9 at step {k} pos ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
            break;
        }
        if k % 10 == 0 {
            eprintln!("[edge] step {k} pos ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        }
    }

    // The rays from the last standing platform spot (the eye at the reported floor + 1.4) to the
    // dock shooters' torsos, every x-z stand candidate along the platform's south side.
    eprintln!("== B. the platform stand rays ==");
    let stands: [(&str, [f32; 3]); 12] = [
        ("start (179.4,-205)", [179.4, 12.19, -205.0]),
        ("armour arrive (180.5,-207.9)", [180.46, 12.19, -207.93]),
        ("kill stand (182,-207)", [182.0, 12.2, -207.0]),
        ("edge arrive (178.4,-207.4)", [178.44, 11.45, -207.4]),
        ("north (177.9,-206.5)", [177.9, 11.2, -206.5]),
        ("east (181,-207)", [181.0, 12.19, -207.0]),
        ("east2 (183,-208)", [183.0, 12.19, -208.0]),
        ("p1 (178,-210)", [178.0, 11.2, -210.0]),
        ("p2 (178,-215)", [178.0, 11.2, -215.0]),
        ("p4 (178,-222)", [178.0, 11.2, -222.0]),
        ("p6 (181,-222)", [181.0, 11.2, -222.0]),
        ("p7 (184,-222)", [184.0, 11.2, -222.0]),
    ];
    for (label, sp) in stands {
        // teleport = probe-only; step to settle.
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = sp;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..6 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if p[1] < 9.0 { eprintln!("[rays] {label}: FELL to y {:.2} — not platform", p[1]); continue; }
        let eye = [p[0], p[1] + 1.4, p[2]];
        let mut line = format!("[rays] {label} floor y {:.2}:", p[1]);
        for (x, z, tol, name) in [(194.0f32, -232.2f32, 6.5f32, "#33"), (161.1, -235.2, 2.5, "#31"), (185.0, -230.0, 6.0, "#16?")] {
            let Some(di) = drone_idx(&w, x, z, tol) else { line.push_str(&format!("  {name} none")); continue; };
            let d = &sim.drones.list[di];
            let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
            let dist = ((torso[0] - eye[0]).powi(2) + (torso[1] - eye[1]).powi(2) + (torso[2] - eye[2]).powi(2)).sqrt();
            let full = sim.world.segment_hit(eye, torso);
            let fp = full.map(|(f, pt)| format!("{:.2}@y{:.1}", f, pt[1])).unwrap_or("CLEAR".into());
            line.push_str(&format!("  {name}({:.1},{:.1},{:.1}) d {:.1} ray {fp} vis {:.2}", d.pos[0], d.pos[1], d.pos[2], dist, d.visibility()));
        }
        eprintln!("{line}");
    }
    let _ = (Act::None, Wp::at(0.0, 0.0, 0.0, Act::None));
}
