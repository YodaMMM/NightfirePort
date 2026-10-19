//! guard-first-shot-timeline (playtest 1016): "guards still shoot way too quickly" and "literally
//! cant even stun the starting guard without him shooting me instantly before he can even turn".
//!
//! The starting guard of 07000001 (drone 0, placement #158) plays start script 0x06000169 through
//! PlayScript (3) — a 181-frame scripted walk down the drive [M NDrone2_DSTATE_PlayScript 0x15ba68
//! case 1; L tests/traces/01-doors.trace lines 0-150: the guard is in state 3 for the whole walk
//! while the player walks up to 5.4 u in his view cone]. The game's sight-interrupt of a running
//! script, `DroneVision_EnemyLookForOpponent` 0x177ed0, opens with a gate the port was missing:
//! `if (current AI state == 3 && alertness >= 1.0) return 0` [M 0x177ed0:12-14] — a fully-alert
//! scripted guard NEVER sight-interrupts (the guard's behaviour prop 0x20 = 3 → alertness 1.0 [D
//! placement #158 words 18-20 through behaviour_util_get 0x1c7a98; the alertness table {0,0,0.66,1.0}
//! [D ee_start_07000001.bin @ 0x2f5cc8]). The port interrupted at the sight ack (+17 ticks after
//! control) and his first bullet landed at +20 — the playtest's instant opening fire.
//!
//! The script's own end directs AimStandFire: DSTATE_PlayScript case 5 for 0x6000168/0x6000169 →
//! `CallAnim(0, 10, 0, 0x68, 0, 0)` [M 0x15ba68 case 5] — ported. From there the fire chain is the
//! game's: the AimStandFire enter fires with fresh LOS (`+0x274 < 0x10 → DroneWeap_Fire(1)`, [M
//! 0x166328 case 1]), DoFiring's sight gate (`+0x274 < 0x10 || prop 0x17`, [M 0x17a080:24-25]),
//! Ready2Fire's aim-class 2 gate with the dist < 2 always-ready arm ([M 0x178c48 SP arm
//! `+0x1a0 < 2.0 → return 1`]) and the burst schedule ([M 0x179d40]).
//!
//! Each test drives the real path only: `nf_view::load` + `Sim::step_bond` inputs (the intro runs
//! out, the stunner equips through the real switch, Bond walks with stick inputs). Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Ticks the intro out with real idle inputs; returns the tick control returned.
fn run_intro(scene: &mut nf_view::scene::Scene) -> u32 {
    for t in 0..4000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { return t; }
    }
    panic!("the 07000001 intro never returned control");
}

#[test]
fn a_starting_guard_finishes_his_start_script_before_any_shot() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let free = run_intro(&mut scene);
    let gi = 0usize; // drone 0 = the first guard (placement #158, script 0x06000169)

    // Bond stands where the intro left him (the real post-intro spot, 5.4 u in the guard's cone).
    let mut left_script_at = None;
    let mut first_shot = None;
    let mut los_while_script = false;
    let mut ack_while_script = false;
    for t in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        if dr.state == nf_game::drone::state::PLAY_SCRIPT {
            los_while_script |= dr.los;
            ack_while_script |= dr.seen;
        } else if left_script_at.is_none() {
            left_script_at = Some(t);
        }
        if !dr.shots.is_empty() { first_shot = Some(t); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[gi];
    eprintln!("[01] free t={free} left script at {left_script_at:?} first shot {first_shot:?} (script ticks {}) state {:#x}",
        left_script_at.unwrap_or(600), dr.state);
    // The game's chain: the guard holds PlayScript for the whole scripted walk — the game's own
    // recording shows state 3 from the trace's line 0 to ~150 with the player walking up in his
    // cone [L tests/traces/01-doors.trace lines 0-150]. The sight ack lands DURING the script
    // (Bond is visible; the 10-tick reaction [M DroneFunc_ReactionTime 0x14ac60 default arm] runs)
    // but the script is not interrupted [M 0x177ed0:12-14]. The port used to leave the script at
    // the ack (+17) and fired at +20 — the playtest's instant opening fire.
    assert!(los_while_script, "the guard never had Bond in sight during his script — the \
        pre-condition died (the vision pass should see a 5 u Bond in the cone)");
    assert!(ack_while_script, "the sight ack never landed during the script — the 10-tick \
        reaction [M 0x14ac60] should complete while the script holds");
    let left = left_script_at.expect("the guard never left PlayScript in 600 ticks");
    assert!(left >= 150, "the guard left his start script after only {left} ticks of control — \
        the game's scripted walk holds PlayScript for the whole 181-frame script [L 01-doors.trace \
        lines 0-150] because EnemyLookForOpponent returns 0 for a state-3 drone at alertness >= 1.0 \
        [M 0x177ed0:12-14]");
    if let Some(shot) = first_shot {
        assert!(shot >= left, "the guard's first bullet (tick {shot}) preceded his script end \
            (tick {left}) — the game's first shot rides the script's own end into AimStandFire \
            [M 0x15ba68 case 5] and its enter fire [M 0x166328 case 1]");
    }
}

/// The playtest's exact play: Bond walks BEHIND the starting guard while the guard is still
/// scripting, touches him with the stunner and holds. The guard must not fire while his script
/// still runs (Bond behind him gives no cone sight, and the full-alert script gate [M 0x177ed0:12-14]
/// holds the interrupt); the user's whole stun window IS that script walk.
#[test]
fn a_starting_guard_walked_behind_never_fires_during_his_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let free = run_intro(&mut scene);
    let gi = 0usize;
    // The stunner equips through the real switch input.
    let draw = Input { switch_to: Some(0x4a), ..Default::default() };
    let mut ok = false;
    for _ in 0..90 {
        scene.sim.as_mut().unwrap().step_bond(draw, false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == 0x4a { ok = true; break; }
    }
    assert!(ok, "the Stunner (0x4a) did not equip through the real switch input");
    // Walk to the guard's rear (1.6 u behind his facing) with real stick inputs while he scripts.
    let mut touched_at = None;
    for t in 0..1200u32 {
        let (turn, fwd) = {
            let sim = scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            let rear = [dr.pos[0] - 1.6 * dr.yaw.sin(), dr.pos[1], dr.pos[2] - 1.6 * dr.yaw.cos()];
            let bp = sim.bond.pos;
            let dx = rear[0] - bp[0];
            let dz = rear[2] - bp[2];
            let err = walker_wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            (err.clamp(-0.08, 0.08), if dx * dx + dz * dz > 0.25 { 0.55 } else { 0.0 })
        };
        scene.sim.as_mut().unwrap().step_bond(Input { forward: fwd, turn, ..Default::default() }, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        let d = ((dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2)).sqrt();
        if d < 1.7 { touched_at = Some(t); break; }
        assert!(dr.shots.is_empty(), "the guard fired at tick-walk {t} while Bond approached from \
            behind and his script still ran (state {:#x}) — no cone sight, no fire", dr.state);
    }
    let touched = touched_at.expect("Bond never reached the guard's rear through the real walk");
    eprintln!("[01] free t={free} touched the guard's rear at walk t={touched}");
    // Hold the tase. While the guard is still scripting he must not fire (the gate); once the
    // script ends the game's own AimStandFire enter may fire with fresh LOS [M 0x166328] — that
    // chain is the game's and is not asserted here.
    let mut scripted_with_contact = false;
    for t in 0..300u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), true, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        if dr.state == nf_game::drone::state::PLAY_SCRIPT { scripted_with_contact = true; }
        assert!(dr.shots.is_empty() || dr.state != nf_game::drone::state::PLAY_SCRIPT,
            "the guard fired at tase tick {t} while STILL in PlayScript (state {:#x}) — the \
             full-alert script gate [M 0x177ed0:12-14] and the cone [M 0x175af0] both hold fire",
            dr.state);
        if dr.state == nf_game::drone::state::TASER || dr.health <= 0.0 {
            eprintln!("[01] the stun landed at tase tick {t} (state {:#x} hp {:.0}), scripted-with-contact {scripted_with_contact}", dr.state, dr.health);
            return;
        }
    }
    assert!(scripted_with_contact, "the guard's script had already ended before the tase \
        connected — the walk window collapsed (the test's premise died; the script is 181 frames \
        [D nf-extract script-info])");
}

fn walker_wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}
