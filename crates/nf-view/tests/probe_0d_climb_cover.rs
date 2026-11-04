//! probe (assertion-free, `--ignored`): the 0d climb-leg cover question. The real-damage walk
//! dies on the ladder/wire (tick 2988, 10 hits for 100: #31 perch, #28/#29 west roof, #33 dock,
//! #10 shipping ground, #16 a miss). For the climb tactic this maps, per candidate shooter:
//! (A) the census (pos, weapon, health, yaw, sight fields, state); (B) the climb/wire exposure —
//! teleport Bond to each climb point, step, print every shooter's dist/visibility()/los; (C) the
//! reverse shooting lines — raw rays from strip standpoints to each shooter's torso, and whether
//! the shooter's idle cone covers the standpoint (visibility() with Bond there); (D) Bond's own
//! weapon/ammo. Run: NF_ISO=… cargo test -p nf-view --test probe_0d_climb_cover -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn idx_of(w: &Walker, pred: impl Fn(&nf_game::drone::Drone) -> bool) -> Option<usize> {
    w.scene.sim.as_ref().unwrap().drones.list.iter().position(pred)
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_climb_shooters_and_lines() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);

    // Part A: the census — every drone near the yard/dock/roof, enabled or not (the west roof
    // trio + #10 fire later in the walk: WHEN do they enable?).
    eprintln!("== A. drone census (all, x 140..230, z -300..-215) ==");
    let shooters: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut v = Vec::new();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.corpse || d.health <= 0.0 { continue; }
            if !(140.0..=230.0).contains(&d.pos[0]) || !(-300.0..=-215.0).contains(&d.pos[2]) { continue; }
            eprintln!("[census] drone #{i} skin {:#x} weapon {:#x} pos ({:.2},{:.2},{:.2}) health {:.1} enabled {} present {} yaw {:.2} half_fov {:.3} sight_range {:.0} state {:#x} seen {} los {} team {}",
                d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2], d.health, d.enabled, d.present, d.yaw, d.half_fov, d.sight_range, d.state, d.seen, d.los, d.team);
            v.push(i);
        }
        v
    };

    // Part D: Bond's own loadout — every owned weapon + its clip/spare, and the ammo pools.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[bond] weapon {:#x} clip {} spare {} health {:.1} armour {:.1}",
            sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health, sim.drones.health.armour);
        let inv = &sim.drones.inventory;
        for (w, own) in inv.owned.iter().enumerate() {
            if *own && w > 0 {
                let clip = inv.clip.get(w).copied().unwrap_or(0);
                let _ = clip;
                eprintln!("[bond] owned {:#x} clip_slot_rounds {}", w, inv.clip.get(w).copied().unwrap_or(0));
            }
        }
        eprintln!("[bond] carried ammo: {:?} clips: {:?}", inv.carried, inv.clip.iter().enumerate().filter(|(w, c)| **c > 0 && *w < 0x80).map(|(w, c)| (w, c)).take(12).collect::<Vec<_>>());
    }

    // Part B: the climb exposure — teleport Bond to each climb/wire point, step, print.
    // The climb points (from walkreal_0700000d_live): the ladder base is approached along the
    // strip; the ladder #435 centre (212.22, 8.14, -234.51) spans y 0..16.35; the top grabs the
    // wire #271 (centre (205.61, 17.04, -248.52)); the ride lands on the deck (199.25, 14.16, -260.16).
    let climb_points: [(&str, [f32; 3]); 9] = [
        ("strip mid", [196.0, 0.5, -228.5]),
        ("strip east", [204.0, 0.5, -229.0]),
        ("ladder base", [213.2, 1.0, -234.8]),
        ("ladder low", [212.6, 4.5, -234.6]),
        ("ladder mid", [212.5, 8.5, -234.6]),
        ("ladder high", [212.5, 13.0, -234.6]),
        ("ladder top", [212.5, 16.3, -234.6]),
        ("wire early", [209.5, 17.3, -241.0]),
        ("wire mid", [205.0, 17.2, -250.0]),
    ];
    for (label, p) in climb_points {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = p;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..6 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let mut line = format!("[climb] {label} ({:.1},{:.1},{:.1}):", p[0], p[1], p[2]);
        for &i in &shooters {
            let d = &sim.drones.list[i];
            let dist = ((d.pos[0] - p[0]).powi(2) + (d.pos[1] - p[1]).powi(2) + (d.pos[2] - p[2]).powi(2)).sqrt();
            line.push_str(&format!("  #{} d {:.0} vis {:.2} los {} seen {}", i, dist, d.visibility(), d.los, d.seen));
        }
        eprintln!("{line}");
    }

    // Part C: the reverse lines — from each strip standpoint (Bond's eye ~ floor + 1.4) to each
    // shooter's torso (root + 0.3, the walker's aim point), the raw ray; plus the shooter's idle
    // visibility() with Bond standing there (does the cone cover the standpoint?).
    eprintln!("== C. strip standpoints → shooter torso rays ==");
    let stands: [(&str, [f32; 3]); 6] = [
        ("stand west", [180.0, 0.5, -228.0]),
        ("stand mid1", [188.0, 0.5, -228.0]),
        ("stand mid2", [196.0, 0.5, -228.5]),
        ("stand mid3", [204.0, 0.5, -229.0]),
        ("stand east", [211.0, 0.5, -231.0]),
        ("ladder base", [213.2, 1.0, -234.8]),
    ];
    for (slabel, sp) in stands {
        let eye = [sp[0], sp[1] + 1.4, sp[2]];
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = sp;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..6 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let mut line = format!("[lines] {slabel} eye ({:.1},{:.1},{:.1}):", eye[0], eye[1], eye[2]);
        for &i in &shooters {
            let d = &sim.drones.list[i];
            let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
            let dist = ((torso[0] - eye[0]).powi(2) + (torso[1] - eye[1]).powi(2) + (torso[2] - eye[2]).powi(2)).sqrt();
            let full = sim.world.segment_hit(eye, torso);
            let stat = sim.world.segment_hit_static(eye, torso);
            let fp = full.map(|(f, p)| format!("{f:.2}@y{:.1}", p[1])).unwrap_or("CLEAR".into());
            let sp2 = stat.map(|(f, _)| format!("{f:.2}")).unwrap_or_else(|| "clear".to_string());
            line.push_str(&format!("  #{} d {:.0} ray {fp}/{sp2} vis {:.2}", i, dist, d.visibility()));
        }
        eprintln!("{line}");
    }

    // Part E: the enablement timeline — teleport-walk Bond down the strip the way the walk's
    // legs do (the trigger volumes test position each tick) and print, every 60 ticks, which
    // shooters are enabled. When does the west roof wake?
    eprintln!("== E. enablement along the strip walk ==");
    {
        let legs: [[f32; 3]; 5] = [
            [181.0, 7.5, -227.5], [188.0, 0.5, -228.0], [196.0, 0.5, -228.5], [204.0, 0.5, -229.0], [212.0, 0.5, -230.0],
        ];
        for (li, leg) in legs.iter().enumerate() {
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = *leg;
                sim.bond.vel = [0.0; 3];
            }
            for k in 0..120 {
                w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
                if k % 60 == 0 {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let mut line = format!("[en] leg {li} tick +{k} bond ({:.1},{:.1},{:.1}):", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
                    for &i in &shooters {
                        let d = &sim.drones.list[i];
                        line.push_str(&format!("  #{} en {} st {:#x} yaw {:.1} seen {}", i, d.enabled, d.state, d.yaw, d.seen));
                    }
                    eprintln!("{line}");
                }
            }
        }
    }
}
