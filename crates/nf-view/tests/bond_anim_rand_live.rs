//! bond-anim-rand: Bond's anim set draws from the shared random stream, as the game's does.
//!
//! Every draw here is a `Rand_Random` 0x1e36f0 on the one SEED_X/SEED_Y pair whose value nothing
//! keeps [M]:
//! - `AnimSetUpdate` 0x1c0938:213-221 draws once per call that finds `GlbAnimList != 1` scripts on
//!   the set's tag (`AnimListBuild(obj, 0x200, set+0x20)` 0x1c0fc0 counts every tagged script,
//!   fading or stopped). Standing, the set holds the idle alone (no draw); walking, the walk and
//!   its synced run (a draw every tick); after a change into or out of idle the faded scripts stay
//!   counted until `AnimFrameResolve` 0x1c35f8:131-141/:440-470 deletes them at the end of the
//!   8-tick fade, and the change's own `AnimObjectUpdate` 0x1c0938:202 takes one tick of it before
//!   the count — 7 draws after a stop.
//! - `AnimSetInit` 0x1c6a48:48 draws once per new set: `Player_ChangeSubState` 0x190630 into
//!   0 / 4 / 8 / 9 (the crouch and stand toggles included), the transition hold's end
//!   (0x192310:34-44, 0x1924c0:45-55) and `Player_SetWeaponAnim` 0x1a9338:28 (view state 3).
//! - `Player_Update` 0x193600:169 / :354-357: `BLData+0x93c` counts down on every live tick and
//!   refills from the stream when it falls below 1.
//!
//! The counts are taken around `Sim::advance_bond_anim` alone (the drones' own draws stay out).

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// One anim update with Bond's forward speed `fwd` (`BLData+0x18`); the draws it made.
fn tick(sim: &mut Sim, fwd: f32) -> u64 {
    sim.bond.local = [0.0, fwd];
    let before = sim.drones.rng.draws;
    sim.advance_bond_anim();
    sim.drones.rng.draws - before
}

fn fresh(sim: &mut Sim) {
    sim.bond_anim = nf_game::mp::body_anim::MpBodyAnim::new();
    sim.bond.anim_set_inits = 0;
    sim.bond.anim_set_reset = false;
    sim.bond.crouched = false;
    sim.bond.substate = nf_game::player::SS_WALK;
    sim.gun.weapon_anim_sets = 0;
    sim.disabled = false;
    // Far from the BL+0x93c refill so only the set's draws show.
    sim.bond.rand_timer = 30000;
}

#[test]
fn bond_anim_draws_per_tick_stand_walk_stop() {
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().expect("sim");
    fresh(sim);
    // Standing: the first update appends the idle (one tagged script), then nothing changes.
    let stand: Vec<u64> = (0..20).map(|_| tick(sim, 0.0)).collect();
    assert_eq!(stand, vec![0; 20], "standing makes no draw");
    // Walking: the change appends the walk + run over the fading idle (3 scripts), then 2 — a
    // draw on every walking tick.
    let walk: Vec<u64> = (0..30).map(|_| tick(sim, 0.05)).collect();
    assert_eq!(walk, vec![1; 30], "one draw per walking tick");
    // Stopping: the walk pair fades over the new idle — 7 ticks still count 3 scripts.
    let stop: Vec<u64> = (0..20).map(|_| tick(sim, 0.0)).collect();
    let mut want = vec![1u64; 7];
    want.extend(vec![0; 13]);
    assert_eq!(stop, want, "the stop's fade ticks draw, then nothing");
    // A quick start-stop inside the 4-tick debounce: the walk holds for the debounce.
    let flip: Vec<u64> = [0.05, 0.0, 0.0, 0.0, 0.0, 0.0].iter().map(|&f| tick(sim, f)).collect();
    assert_eq!(&flip[..4], &[1, 1, 1, 1], "the walk set stays through the debounce");
}

#[test]
fn bond_anim_draws_on_stance_weapon_and_timer() {
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load");
    let sim = scene.sim.as_mut().expect("sim");
    fresh(sim);
    for _ in 0..20 { tick(sim, 0.0); }
    // The crouch toggle (`Player_HandleJump` → `ChangeSubState(4)`, the Bond side's count): one
    // set init that tick; the set update ran on the old idle first (no draw).
    sim.bond.crouched = true;
    sim.bond.anim_set_inits += 1;
    assert_eq!(tick(sim, 0.0), 1, "the stance change's set init");
    assert!(sim.bond_anim.hold > 0, "the transition hold runs");
    // The hold: no set update, no draw, until it ends — then one set init (and the fresh set's
    // idle, one script, no draw).
    let mut hold = Vec::new();
    while sim.bond_anim.hold > 0 && hold.len() < 80 { hold.push(tick(sim, 0.0)); }
    assert!(!hold.is_empty() && hold.len() < 61, "the hold ends within FRAME_RATE ticks ({})", hold.len());
    assert_eq!(hold.iter().sum::<u64>(), 1, "one draw over the hold: its end's set init ({hold:?})");
    assert_eq!(*hold.last().unwrap(), 1, "on the end tick");
    assert_eq!((0..10).map(|_| tick(sim, 0.0)).sum::<u64>(), 0, "crouched and still: nothing");
    // A weapon set (view state 3 written last tick): one set init after the update.
    sim.gun.weapon_anim_sets += 1;
    assert_eq!(tick(sim, 0.0), 1, "Player_SetWeaponAnim's set init");
    assert_eq!((0..10).map(|_| tick(sim, 0.0)).sum::<u64>(), 0, "the rebuilt set's idle alone");
    // BLData+0x93c: 3 → 2 → 1 → 0, the refill draw on the third tick, 255..1278 after.
    sim.bond.rand_timer = 3;
    let t: Vec<u64> = (0..4).map(|_| tick(sim, 0.0)).collect();
    assert_eq!(t, vec![0, 0, 1, 0], "the timer's refill");
    assert!((254..=1278).contains(&sim.bond.rand_timer), "refilled to (r & 0x3ff) + 0xff, one tick spent: {}", sim.bond.rand_timer);
    // Off (`BLData+0x94a != 1`): nothing runs.
    sim.disabled = true;
    sim.bond.rand_timer = 1;
    assert_eq!((0..5).map(|_| tick(sim, 0.05)).sum::<u64>(), 0, "a disabled player update draws nothing");
}
