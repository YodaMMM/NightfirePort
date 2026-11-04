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
    // Re-edge the aim button first: a button held across the level intro consumed the aim-on
    // edge (prev_aiming stuck true, the bit never sets — measured), so release ~4 ticks and
    // hold until the gated bit actually sets (the raise completes ~20 ticks).
    for _ in 0..4 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    if aiming {
        for _ in 0..60 {
            let sim = w.scene.sim.as_mut().unwrap();
            if sim.aiming { break; }
            sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
        eprintln!("[{label}] aim bit set: {}", w.scene.sim.as_ref().unwrap().aiming);
    }
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
    // Calibration: settle bond.pitch via the look input, WAIT out the fire script (the sniper's
    // +0x8000 refire gate [M 0x1a5460:631-636] blocks any shot while the 130-tick script plays —
    // the old 30-tick gap never fired), then tap and read the bullet's flight pitch against
    // bond.pitch. player-upward-shot (0c) measured the settled aimed round landing exactly where
    // the view aims at pitch 0.195; the 0d probe measured slope 0.024 vs want 0.0402 at pitch
    // 0.0256 — this loop fits that curve.
    {
        for want in [0.0256f32, 0.05, 0.10, 0.195, -0.05] {
            // Drain the previous shot's fire script with the button RELEASED (the raise queue
            // refuses during state 9 [M 0x1a24c8:66-99], and the edge needs a true up->down
            // transition), then press and hold until the gated bit sets.
            for _ in 0..220 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
            let mut bit = false;
            for _ in 0..60 {
                let sim = w.scene.sim.as_mut().unwrap();
                if sim.aiming { bit = true; break; }
                sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            eprintln!("[cal-edge] bit {bit}");
            // settle bond.pitch to `want` (normalized) exactly, aim held
            for _ in 0..300 {
                let sim = w.scene.sim.as_mut().unwrap();
                let perr = want - sim.bond.pitch;
                if perr.abs() < 0.0004 { break; }
                sim.step_bond(nf_game::player::Input { look: perr.clamp(-0.02, 0.02), aiming: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            // hold the aim, no trigger: the fire script drains (130 ticks [D +0x40 = 120; the
            // 0x8000 gate M 0x1a5460:631-636]) and the aim-raise replays settle
            for _ in 0..170 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            let sim = w.scene.sim.as_ref().unwrap();
            let held_pitch = sim.bond.pitch;
            let look_y = sim.bond.look()[1];
            let (aiming, cross, zoom) = (sim.aiming, sim.cross, sim.zoom);
            eprintln!("[gate] aiming {aiming} view_aim_anim {:?} raising {} view_script {:#x} view_time {:.1} free_aim {} auto_aim {} disabled {} prev_aiming {}",
                sim.view_aim_anim, sim.view_aim_raising, sim.view_script, sim.view_time, sim.free_aim, sim.auto_aim, sim.disabled, sim.prev_aiming);
            let clip_empty = sim.gun.clip == 0;
            let clip0 = sim.gun.clip;
            let sim = w.scene.sim.as_mut().unwrap();
            sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, !clip_empty, false, false);
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(b) = sim.drones.player_bullets.iter().find(|b| !b.dead) {
                let pitch_rad = b.dir[1].asin();
                eprintln!("[cal] want {want:.4} held {held_pitch:.4} (rad {:.4}) look_y {look_y:.4} aiming {aiming} cross {:?} zoom {zoom} clip {clip0}->{} -> bullet dir1 {:.4} pitch_rad {:.4} pitch_norm {:.4}  bias_rad {:.4}",
                    held_pitch * std::f32::consts::FRAC_PI_2, cross, sim.gun.clip, b.dir[1], pitch_rad, pitch_rad / std::f32::consts::FRAC_PI_2,
                    held_pitch * std::f32::consts::FRAC_PI_2 - pitch_rad);
            } else {
                eprintln!("[cal] want {want:.4} held {held_pitch:.4} -> NO bullet (clip {} aiming {aiming} view_script {:#x} view_time {:.1})",
                    sim.gun.clip, sim.view_script, sim.view_time);
            }
            // drain the shot's fire script before the next iteration
            for _ in 0..200 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
        }
    }
}

/// The crossfire inputs probe (assertion-free): the tuning firing constants, the wire
/// shooters' skill bytes and the difficulty, and Bond's move quad at the wire grab — the
/// level-0d-real-health-crossfire accuracy audit.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_crossfire_inputs() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = &sim.tuning;
        eprintln!("[tuning] TooClose_Distance {} TooClose_Accuracy {} TargetMoving {} FirstMoved {} FirstStopped {} FirstStopped_TTH {} Acc_Easy {} Acc_Normal {} Acc_Hard {}",
            t.DroneFiring_TooClose_Distance, t.DroneFiring_TooClose_Accuracy, t.DroneFiring_TargetMoving_Accuracy,
            t.DroneFiring_TargetFirstMoved_Accuracy, t.DroneFiring_TargetFirstStopped_Accuracy, t.DroneFiring_TargetFirstStopped_TimeToHit,
            t.DroneFiring_Accuracy_Easy, t.DroneFiring_Accuracy_Normal, t.DroneFiring_Accuracy_Hard);
        eprintln!("[diff] difficulty {} level {:#x}", sim.drones.difficulty, sim.level);
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.enabled && d.present && d.health > 0.0 && d.weapon == 0x1e {
                eprintln!("[shooter] #{i} skin {:#x} acc {} pos {:?}", d.skin, d.accuracy, d.pos);
            }
        }
    }
    // Walk-in to the grab (the walkreal shape): face the wire, run, jump 60..63, read the quad
    // at the grab and on the ride.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [212.0, 1.5, -234.8];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..30 {
        let sim = w.scene.sim.as_mut().unwrap();
        let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        sim.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 0.3, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    for k in 0..80u32 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { jump: (60..=63).contains(&k), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if sim.bond.substate == nf_game::player::SS_WIRE {
            eprintln!("[grab] tick {} quad {:?} substate {}", k, sim.bond.move_local, sim.bond.substate);
            break;
        }
    }
    for k in 0..12 {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[ride t{k}] quad {:?} substate {}", sim.bond.move_local, sim.bond.substate);
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { strafe: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
}

/// The crossfire ray scan (assertion-free): the reverse rays from the wire-ride positions and
/// the strip cover stands to the wire shooters (#17/#32 ground, #10/#18 south roof, #28/#29
/// west roof) — the same `line_of_sight(head, target, LOS_GUARD_VISION)` the drone vision
/// gates on [M NDrone2 sight path]. level-0d-real-health-crossfire.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_crossfire_rays() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let shooters: [(usize, f32, f32, f32); 6] = [
        (17, 158.2, -225.5, 1.0),
        (32, 162.0, -216.5, 1.0),
        (10, 209.8, -288.5, 12.1),
        (18, 212.0, -288.8, 12.1),
        (28, 151.5, -241.2, 22.7),
        (29, 151.4, -255.8, 22.7),
    ];
    // The wire-ride positions (the wire A->B span (205.61, 17.04, -248.52) r 14.02) + the deck
    // arrival + the strip cover stands (the y-0 yard).
    let stands: [([f32; 3], &str); 8] = [
        ([212.0, 17.4, -234.6], "the ladder top"),
        ([208.0, 17.0, -240.0], "the wire 1/4"),
        ([203.0, 17.0, -247.0], "the wire mid"),
        ([199.5, 16.5, -256.0], "the wire 3/4"),
        ([199.3, 14.2, -260.2], "the deck arrival"),
        ([196.0, 1.0, -228.5], "the strip stand 2"),
        ([204.0, 1.0, -229.0], "the strip stand 3"),
        ([212.0, 1.0, -230.0], "the strip stand 4"),
    ];
    // Wake the crossfire the way the game does: stand at the platform-drop landing first (the
    // drop-point trigger wakes the roof/south within a tick [probe_0d_climb_cover part E]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [181.0, 1.1, -227.5];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..30 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); w.total_ticks += 1; }
    for (at, label) in stands {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = at;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..3 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); w.total_ticks += 1; }
        let sim = w.scene.sim.as_ref().unwrap();
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        for &(i, sx, sz, sy) in &shooters {
            let state = sim.drones.list.get(i).map(|d| (d.enabled, d.present, d.corpse, d.health > 0.0));
            // The GEOMETRY answer first: the fixed census position is the ray endpoint either way.
            let head = [sx, sy + 1.4, sz];
            let clear = sim.world.line_of_sight(eye, head, nf_game::world::LOS_GUARD_VISION);
            let d3 = ((sx - eye[0]).powi(2) + (sz - eye[2]).powi(2) + (sy + 1.4 - eye[1]).powi(2)).sqrt();
            eprintln!("[rays] {label} -> #{i} d {:.1} clear {} state {:?}", d3, clear, state);
        }
    }
}
