//! probe (assertion-free, `--ignored`): the platform-kill bullet trace. The walk's aimed,
//! dead-centre 0x1e rounds at #33/#31 from the warehouse main floor (probe-verified CLEAR rays,
//! d 27-33) land NOTHING while the previous branch's hip one-tap at d 21 landed twice. This
//! probe fires ONE shot at a time and traces every player bullet: spawn pos/dir, per-tick
//! flight, and the outcome (drone hit / world hit / expiry). Teleports are probe-only.
//! Run: NF_ISO=… cargo test -p nf-view --test probe_0d_bullet -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn fire_one(w: &mut Walker, at: [f32; 3], aiming: bool, label: &str) {
    // Aim: turn Bond to the target over up to 400 ticks (aimed pose settles), then one trigger
    // tap, then trace every player bullet until all are dead (max 200 ticks).
    for _ in 0..400 {
        let sim = w.scene.sim.as_mut().unwrap();
        let bp = sim.bond.pos;
        let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dx = at[0] - bp[0];
        let dz = at[2] - bp[2];
        let dy = at[1] - eye;
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        if err.abs() < 0.0005 && perr.abs() < 0.0005 { break; }
        let input = nf_game::player::Input {
            turn: err.clamp(-0.02, 0.02),
            look: perr.clamp(-0.02, 0.02),
            aiming,
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    let before = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
    let hp_before: Vec<(usize, f32)> = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.enabled && d.present && !d.corpse && d.health > 0.0)
        .map(|(i, d)| (i, d.health)).collect();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let bp = sim.bond.pos;
        let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dy = at[1] - eye;
        let want_pitch = (dy / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let input = nf_game::player::Input { aiming, ..Default::default() };
        let trigger = perr.abs() < 0.002;
        sim.step_bond(input, trigger, false, false);
        eprintln!("[{label}] trigger tap {} (perr {:.4}) clip {} -> ?", trigger, perr, sim.gun.clip);
    }
    w.total_ticks += 1;
    for k in 0..200 {
        let sim = w.scene.sim.as_ref().unwrap();
        if k == 0 {
            let spawned = sim.drones.player_bullets.len();
            eprintln!("[{label}] bullets after tap: {spawned} (before {before}) impacts {}", sim.drones.impacts.len());
        }
        for (i, b) in sim.drones.player_bullets.iter().enumerate() {
            eprintln!("[{label}] t+{k} bullet{i} pos ({:.2},{:.2},{:.2}) dir ({:.3},{:.3},{:.3}) spd {:.1} trav {:.1} dead {}",
                b.pos[0], b.pos[1], b.pos[2], b.dir[0], b.dir[1], b.dir[2], b.speed, b.travelled, b.dead);
        }
        if !sim.drones.impacts.is_empty() {
            for ev in &sim.drones.impacts {
                eprintln!("[{label}] t+{k} impact {ev:?}");
            }
        }
        let hurt: Vec<(usize, f32)> = sim.drones.list.iter().enumerate()
            .filter(|(i, d)| d.enabled && d.present && !d.corpse && d.health > 0.0)
            .filter(|(i, _)| hp_before.iter().any(|(j, _)| j == i))
            .map(|(i, d)| (i, d.health)).collect();
        for (i, h) in &hurt {
            let h0 = hp_before.iter().find(|(j, _)| j == i).unwrap().1;
            if *h < h0 { eprintln!("[{label}] t+{k} DRONE #{i} HEALTH {:.1} -> {:.1}  *** HIT ***", h0, h); }
        }
        w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        w.total_ticks += 1;
        if k > 6 && w.scene.sim.as_ref().unwrap().drones.player_bullets.iter().all(|b| b.dead) { break; }
    }
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_bullet_trace() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let find = |w: &Walker, x: f32, z: f32, tol: f32| -> Option<usize> {
        w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
            d.enabled && d.present && !d.corpse && d.health > 0.0
                && (d.pos[0] - x).abs() < tol && (d.pos[2] - z).abs() < tol
        })
    };
    let sim = w.scene.sim.as_ref().unwrap();
    let d33 = find(&w, 194.0, -232.2, 6.5).expect("#33");
    let d31 = find(&w, 161.1, -235.2, 2.5).expect("#31");
    let (p33, p31) = (sim.drones.list[d33].pos, sim.drones.list[d31].pos);
    eprintln!("[probe] #33 at {:?}  #31 at {:?}", p33, p31);
    eprintln!("[probe] bond {:?} eye {} weapon {:#x} clip {}", sim.bond.pos, sim.bond.eye, sim.gun.weapon, sim.gun.clip);

    // Stand on the warehouse main floor near the armour arrive (probe-verified CLEAR rays).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [180.46, 12.19, -207.93];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..10 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }

    // Shot 1: AIMED at #33's torso.
    fire_one(&mut w, [p33[0], p33[1] + 0.3, p33[2]], true, "aimed #33");
    // Calibration: fire at three KNOWN settled pitches and read the bullet's actual flight pitch —
    // the camera-vs-bond pitch mapping (the aimed round flew 0.016 rad flat at pitch ~0.026).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let fwd = 30.0f32;
        let yaw = sim.bond.yaw;
        for want in [0.10f32, 0.0f32, -0.10f32] {
            // settle bond.pitch to `want` (normalized) exactly
            for _ in 0..200 {
                let sim = w.scene.sim.as_mut().unwrap();
                let perr = want - sim.bond.pitch;
                if perr.abs() < 0.0004 { break; }
                sim.step_bond(nf_game::player::Input { look: perr.clamp(-0.02, 0.02), aiming: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            let sim = w.scene.sim.as_mut().unwrap();
            let held_pitch = sim.bond.pitch;
            let clip_empty = sim.gun.clip == 0;
            sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, !clip_empty, clip_empty, false);
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(b) = sim.drones.player_bullets.iter().find(|b| !b.dead) {
                let pitch_rad = b.dir[1].asin();
                let yaw_b = b.dir[0].atan2(-b.dir[2]);
                eprintln!("[cal] bond.pitch {held_pitch:.4} -> bullet pitch (norm) {:.4} rad {:.4} yaw {:.4} (bond yaw {:.4})",
                    pitch_rad / std::f32::consts::FRAC_PI_2, pitch_rad, yaw_b, yaw);
            } else {
                eprintln!("[cal] bond.pitch {held_pitch:.4} -> NO bullet (clip {})", sim.gun.clip);
            }
            for _ in 0..30 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); w.total_ticks += 1; }
            let _ = (fwd, eye);
        }
    }
}
