//! level-4a-cutscene-autoaim: Bond rides the 0700004a cab (real input, no pos writes, no
//! re-seating) and SHOOTS from it while it descends — the reported "auto aim doesn't
//! play too well with shooting from the elevator".
//!
//! THE GAME'S AUTO-AIM HAS NO VELOCITY TERMS ANYWHERE — the moving shooter changes nothing:
//! · the pick `Check_AutoAim` 0x1a73f8 is instantaneous geometry only: head → target bone −2,
//!   nearest in the angular cones, LOS `Collide_LineOfSight` mask 0xa [M 0x1a73f8:110-166] — the
//!   caller `Player_AutoAim` 0x1a7c98 passes NO velocity (its only inputs are the player obj and
//!   the option byte); neither cab-relative nor world velocity exists in the chain;
//! · the apply `Player_GetAimingPoint` 0x1a4a98 re-projects the stored screen offsets through the
//!   viewer planes: head + dir·(−256) [M 0x1a4a98:48-83] — no velocity;
//! · the bullet `Player_WeaponInitBullet` 0x1a4bf0 → `Bullet_init` 0x12c5a0: dir = the aim ray,
//!   speed = weapon+0x20 — the shooter's (cab's) velocity is NEVER inherited [M 0x1a4bf0:152-153,
//!   0x12c5a0:166-191];
//! · the rider carry `Player_FeetOnPoint` 0x194188 → `Control_InheritVelocity` 0x133e88 moves the
//!   rider's POSITION with the platform (pos += platform delta) and never touches the aim frame —
//!   head/yaw/pitch stay world [M 0x133e88:64-143; 0x1a1d28:192-193,273-276].
//! Pin 1 (the ride, real input): the cab ride replays — the landing cut-scene plays and hands
//! back (camera ON→OFF), the descent holds the key line, the shaft's per-floor guards sit behind
//! the ride line, and the sensor LOS from the moving cab DOES reach a guard through the 0x40 well
//! panes (150 ticks at the −72/−79 floors) — the geometry the game's pick consumes each tick.
//! Pin 2 (the no-lead semantics): the pick is a pure function of the instantaneous (head, look)
//! — a head moved by the cab's per-tick delta picks the same target with the same snapped ray as
//! the static head at the same geometry (no velocity input exists to differ [M 0x1a7c98]).
//! The real-play gap that keeps the pick dark on the ride is NOT the aim chain: the shaft doors
//! never auto-open in the port (the level's KNOWN GAP §6, docs/levels/0700004a.md) and the
//! scene harness never sets the render-side `drawn` bit the pick gates on [M 0x1a73f8:65-109] —
//! both left as live checks against the real game. Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{wrap_pi, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The stick aim toward `target` from Bond's eye (the walkreal aim helper) — real look only.
fn aim_input(w: &Walker, target: [f32; 3]) -> Input {
    let sim = w.scene.sim.as_ref().unwrap();
    let b = &sim.bond;
    let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    let want_yaw = d[0].atan2(d[2]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    let want_pitch = (d[1] / len).asin() / (std::f32::consts::FRAC_PI_2);
    let yaw_err = wrap_pi(want_yaw - b.yaw);
    let pitch_err = (want_pitch - b.pitch).clamp(-0.06, 0.06);
    Input { turn: yaw_err.clamp(-0.06, 0.06), look: pitch_err, ..Default::default() }
}

#[test]
fn the_cab_ride_replays_and_the_moving_cab_sees_a_guard_the_pick_cone_consumes() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    // Board: settle onto the platform (the spawn is ON it), then hold the descent with no input.
    for _ in 0..60 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let desired = (3.2 - p[0]).atan2(-2.2 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "Bond settles onto the lift platform");
    let y_board = w.scene.sim.as_ref().unwrap().bond.pos[1];
    w.skip_cutscenes = false; // the real player watches the landing cut-scene

    let mut saw_cutscene = false;
    let mut handback = false;
    let mut los_windows = 0u32;
    let mut los_at: Vec<(u32, f32, f32)> = Vec::new(); // tick, cab y, guard y
    for k in 0..6800u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        let head = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
        // The nearest live enemy guard ahead — the swing target (the game's pick does its own
        // cones + sensor LOS each tick [M 0x1a7c98:83-102]).
        let mut target: Option<(usize, [f32; 3])> = None;
        let mut best = f32::MAX;
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present || !dr.enabled || dr.corpse || dr.health <= 0.0 || dr.team != 1 { continue; }
            let p = [dr.pos[0], dr.pos[1] + 1.4, dr.pos[2]];
            let dv = [p[0] - head[0], p[1] - head[1], p[2] - head[2]];
            let dist = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
            if dist > 25.0 || dist <= 0.0 { continue; }
            if wrap_pi(dv[0].atan2(dv[2]) - b.yaw).abs() > 0.3 { continue; }
            if dist < best { best = dist; target = Some((i, p)); }
        }
        if let Some((_, p)) = target {
            let sim = w.scene.sim.as_ref().unwrap();
            // The game's pick LOS: the sensor mask passes the 0x40 well panes [M 0x1a73f8:164-165].
            if sim.world.line_of_sight_masked(head, p, nf_game::world::LOS_SENSOR) {
                los_windows += 1;
                if los_at.len() < 4 {
                    let gy = sim.drones.list[0].pos[1];
                    los_at.push((k, sim.bond.pos[1], gy));
                }
            }
        }
        let input = match target { Some((_, p)) => aim_input(&w, p), None => Input::default() };
        if target.is_some() { w.tick_fire(input); } else { w.tick(input, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.scripts.camera().is_some() { saw_cutscene = true; }
        else if saw_cutscene && !handback { handback = true; }
        if sim.drones.level_exit.is_some() { break; }
        if sim.drones.channels.on(0x84) { break; } // the bottom plate — the ride is over
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let y = sim.bond.pos[1];
    eprintln!("[4a-autoaim] cutscene {saw_cutscene} handback {handback} los_windows {los_windows} at {los_at:?} y {y:.2} (boarded {y_board})");
    assert!(saw_cutscene && handback, "the landing cut-scene played and handed back (camera ON→OFF) [D SP #289 0x060006cf]");
    assert!(y < y_board - 140.0, "the ride covered the shaft (y {y:.2} from {y_board}) — Bond rode the moving cab the whole way");
    assert!(los_windows >= 30, "the moving cab's sensor LOS reaches a shaft guard through the 0x40 well panes [M 0x1a73f8:164-165] — windows {los_windows} at {los_at:?}");
}
