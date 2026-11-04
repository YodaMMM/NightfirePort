//! audit-player-camera-modes: the death camera stands the eye back up out of a crouch — the dead
//! arm of `Player_PositionCamera` 0x18f5d8 zeroes `BL+0x90c` every dead tick, through the REAL
//! viewer path (`nf_view::load` + `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! Game evidence (ACTION.ELF):
//! - The dead arm [M 0x18f5d8:130-132]: `if (*(ushort *)(obj + 0xf4) - 2 < 2) BL+0x90c = 0` — the
//!   death state word (2 dying, 3 dead, [M `Player_CheckForDeath` 0x190810:64-97]) clears the
//!   crouch drop EVERY tick the case-0 camera runs.
//! - The camera head reads it [M `Player_GetHeadPos` 0x195aa0:39-51: eye.y = obj+0x34 +
//!   BL+0x908 − BL+0x90c] — a death out of the crouch rises the first-person eye 0.45 back to
//!   the standing head instead of freezing it at the crouched height. The port's dead branch
//!   runs no walk tick ([M `Player_Update` 0x193600:316-319 — only `AnimObjectUpdate` +
//!   `Player_HandleDeath`]), so `crouch_drop` used to freeze wherever the crouch ramp had it.
//! - The kill rides the real damage API the death suite uses (`pain_generic`, the
//!   `Player_HandlePain` port) and the death starts on the tick the health check sees 0
//!   [M 0x190810:34-97; death-animation-sp pins the whole sequence].

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Crouch Bond on the spot through the real input path until the crouch ramp fills
/// (+0.0225/tick toward 0.45 [M `Player_PositionCamera` 0x18f5d8:71-79 — the port keys it on the
/// `crouched` flag, `physics_tick`'s tail]).
fn crouch(sim: &mut nf_view::scene::Sim) {
    let input = Input { crouch: true, ..Default::default() };
    for _ in 0..120 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(input, false, false, false);
        if sim.bond.crouched && sim.bond.crouch_drop >= 0.44 { return; }
    }
    panic!("the crouch never filled (crouched {}, drop {})", sim.bond.crouched, sim.bond.crouch_drop);
}

fn kill(sim: &mut nf_view::scene::Sim) {
    let (h, r) = (&mut sim.drones.health, &mut sim.drones.rng);
    h.pain_generic(r, 500.0, 1, 0x0700_0005, 0);
}

/// Dying crouched stands the death camera eye back up: while crouched the head sits 0.45 low
/// (pos + 0.7 − 0.45), after the death tick `crouch_drop` is 0 and the head is the standing
/// pos + 0.7 — the game's dead arm zeroes BL+0x90c [M 0x18f5d8:130-132] where the pre-fix port
/// froze the crouched drop for the whole death sequence.
#[test]
fn dying_crouched_zeroes_the_crouch_drop_for_the_death_camera() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 05");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }

    crouch(sim);
    assert!(sim.bond.crouched && sim.bond.crouch_drop >= 0.44, "the crouch ramp did not fill");
    let pos = sim.bond.pos;
    let eye_low = pos[1] + sim.bond.eye - sim.bond.crouch_drop;
    assert!((eye_low - (pos[1] + 0.25)).abs() < 0.02, "the crouched eye is not the 0.45-dropped head");

    kill(sim);
    sim.step_bond(Default::default(), false, false, false);
    assert!(sim.drones.death.active, "the death sequence started [M 0x190810]");
    assert_eq!(sim.bond.cam_mode, 0, "the death camera is cam mode 0 [M 0x190810:64]");
    // The dead arm's zero [M 0x18f5d8:130-132] — the pre-fix port froze 0.45 here.
    assert_eq!(sim.bond.crouch_drop, 0.0, "the dead arm must zero BL+0x90c");
    let eye = sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop;
    assert!((eye - (sim.bond.pos[1] + 0.7)).abs() < 1e-4, "the death eye is the standing head again");

    // Sister: the zero holds for the whole sequence (the game re-runs it every dead tick).
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.death.active, "still dead");
    assert_eq!(sim.bond.crouch_drop, 0.0, "the zero holds every dead tick");
}

/// Dying STANDING keeps the standing eye: the zero is a no-op there ([S] regression guard for
/// the death camera the death-animation-sp suite pins).
#[test]
fn dying_standing_leaves_the_standing_eye_untouched() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 05");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.bond.crouch_drop.abs() < 1e-6, "standing Bond has no crouch drop");

    kill(sim);
    sim.step_bond(Default::default(), false, false, false);
    assert!(sim.drones.death.active, "the death sequence started [M 0x190810]");
    assert_eq!(sim.bond.crouch_drop, 0.0);
    let eye = sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop;
    assert!((eye - (sim.bond.pos[1] + 0.7)).abs() < 1e-4, "the standing death eye is unchanged");
}
