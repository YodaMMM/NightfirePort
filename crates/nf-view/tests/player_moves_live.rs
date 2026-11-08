//! Bond's movement rules against the game's `Player_Update` 0x193600 / `Player_Move` 0x191dd0 /
//! `Player_CollisionHandler` 0x1949a8, through the real load path (`nf_view::load` +
//! `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! - The smoothers ease from the SCALED lanes: `Player_Update` copies the +0x10 move quad into
//!   +0x20 after the move [M 0x193600:296-299, disasm 0x193cd4/0x193cdc], and `Player_Move` stores
//!   the lanes multiplied by its argument [M 0x191dd0:166-170]. The crouch's 0.75 [M
//!   Player_SSCrouch 0x1924c0:58] therefore settles at 0.15/0.4 = 0.375 of the walk target.
//! - The 07000007 party: `Player_Move` multiplies its argument by 0.9 while channel 0x48 is off
//!   [M 0x191dd0, disasm 0x1921d4-0x19220c] — 0.18/0.28 of the walk target.
//! - The swim's forward lane reads back as the local quad's z — cos(view pitch)·lane [M
//!   0x191dd0:149-165].
//! - The fall timer BL+0x928 zeroes on every swim/climb/wire/... tick [M 0x1949a8:83-95, jump
//!   table @0x2fc050], and on any airborne tick whose y velocity is not negative [M :97-100].

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// `Player_Move`'s full-stick forward target: 0.1·1² + 0.005 [M 0x191dd0:109-111].
const T_FWD: f32 = 0.105;

/// The ease's fixed point with multiplier `k` below the ±0.1 cap [M 0x191dd0:126-131]:
/// v = k·(v + 0.2·(t − v)) → v = 0.2·k·t / (1 − 0.8·k). The plain walk (k = 1) sits on the cap.
fn settle(k: f32, t: f32) -> f32 { (0.2 * k * t / (1.0 - 0.8 * k)).min(0.1 * k) }

/// The full-stick walk's settled lane: the 0.105 target clamps at 0.1.
const CAP: f32 = 0.1;

/// The heading (of 8) the plain walk from `spot` covers the most ground on — an open lane for
/// the speed reads.
fn open_heading(sim: &mut Sim, spot: [f32; 3]) -> f32 {
    let mut best = (0.0f32, 0.0f32);
    for i in 0..8 {
        let yaw = i as f32 * std::f32::consts::FRAC_PI_4 - std::f32::consts::PI;
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, yaw);
        for _ in 0..90 { step(sim, Input::default()); }
        let p0 = sim.bond.pos;
        for _ in 0..80 { step(sim, Input { forward: 1.0, ..Default::default() }); }
        let d = hdist(sim.bond.pos, p0);
        if sim.bond.substate != nf_game::player::SS_WALK { continue; }
        if d > best.1 { best = (yaw, d); }
    }
    best.0
}

fn load(level: &str) -> Option<Sim> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, level, false).expect("load");
    let mut sim = scene.sim.take().expect("sim");
    // Release any intro cut-scene with the game's own skip action: the scene primes on the first
    // step, then the skip holds until it hands Bond back (fix_swim_live's load).
    let mut saw_scene = false;
    for n in 0..3000 {
        if sim.disabled { saw_scene = true; } else if saw_scene || n > 300 { break; }
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
    }
    for _ in 0..200 { step(&mut sim, Input::default()); }
    assert!(!sim.disabled, "Bond is free after the intro");
    Some(sim)
}

fn step(sim: &mut Sim, input: Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

fn hdist(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// The crouch walk on the 07000002 garden lawn (the pond bank, a flat open floor): full stick
/// forward crouched settles at 0.375·0.105 per tick — half what the port moved (0.75·0.1).
#[test]
fn the_crouch_walk_settles_at_the_scaled_lane_on_07000002() {
    let Some(mut sim) = load("07000002.bin") else { eprintln!("skip: NF_ISO not set"); return };
    // The lawn west of the pond.
    let spot = [-84.0, 5.3, 32.05];
    let yaw = open_heading(&mut sim, spot);
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, yaw);
    for _ in 0..90 { step(&mut sim, Input::default()); }
    assert!(sim.bond.grounded, "settled on the lawn: {:?}", sim.bond.pos);
    step(&mut sim, Input { crouch: true, ..Default::default() });
    for _ in 0..70 { step(&mut sim, Input::default()); }
    assert!(sim.bond.crouched, "the crouch press crouches");
    let fwd = Input { forward: 1.0, ..Default::default() };
    for _ in 0..60 { step(&mut sim, fwd); }
    let p0 = sim.bond.pos;
    for _ in 0..20 { step(&mut sim, fwd); }
    let speed = hdist(sim.bond.pos, p0) / 20.0;
    let want = settle(0.75, T_FWD);
    assert!(sim.bond.crouched, "still crouched");
    assert!((sim.bond.local[1] - want).abs() < 1e-4,
        "the crouch's forward smoother holds the 0.75-scaled lane: {} (want {want}; the unscaled ease reads the 0.1 cap and walks 0.075/tick)",
        sim.bond.local[1]);
    assert!((speed - want).abs() < 2e-3, "the crouch walk covers {speed}/tick (want {want})");
    // The stand-up returns the walk's own target.
    step(&mut sim, Input { crouch: true, ..Default::default() });
    for _ in 0..80 { step(&mut sim, fwd); }
    assert!(!sim.bond.crouched, "the second press stands");
    assert!((sim.bond.local[1] - CAP).abs() < 1e-4, "standing settles at the capped lane: {}", sim.bond.local[1]);
}

/// The 07000007 party: before the hall's channel 0x48 the walk runs on `Player_Move`'s 0.9
/// multiplier — it settles at 0.18/0.28·0.105 ≈ 0.0675 per tick; once the channel is on the walk
/// is the plain capped 0.1.
#[test]
fn the_party_walk_runs_at_the_0_9_multiplier_until_channel_0x48_on_07000007() {
    let Some(mut sim) = load("07000007.bin") else { eprintln!("skip: NF_ISO not set"); return };
    assert!(!sim.drones.channels.on(0x48), "the party starts with ch 0x48 off");
    let fwd = Input { forward: 1.0, ..Default::default() };
    for _ in 0..60 { step(&mut sim, fwd); }
    let want = settle(0.9, T_FWD);
    assert!((sim.bond.local[1] - want).abs() < 1e-4,
        "the party walk's forward lane settles at {want} [M 0x191dd0 ×0.9], got {}", sim.bond.local[1]);
    // Stop, raise the channel the hall walk-in writes [D #412], walk again: full speed.
    for _ in 0..60 { step(&mut sim, Input::default()); }
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x48, 1, frame);
    // The hall's release plays a scene: skip it and settle before the second walk.
    for _ in 0..5 { step(&mut sim, Input::default()); }
    for _ in 0..3000 { step(&mut sim, Input::default()); if !sim.disabled { break; } }
    for _ in 0..200 { step(&mut sim, Input::default()); }
    assert!(!sim.disabled);
    for _ in 0..60 { step(&mut sim, fwd); }
    assert!(sim.drones.channels.on(0x48));
    assert!((sim.bond.local[1] - CAP).abs() < 1e-4,
        "with ch 0x48 on the walk settles at the plain target, got {}", sim.bond.local[1]);
}

/// The 07000002 pond: a stroke pitched 45° down eases from cos(45°)·lane, so the forward lane
/// settles at 0.2·t / (1 − 0.8·cos) and the quad's z reads cos·that; the level stroke stays at t.
#[test]
fn the_pitched_swim_stroke_eases_from_the_local_z_lane_on_07000002() {
    let Some(mut sim) = load("07000002.bin") else { eprintln!("skip: NF_ISO not set"); return };
    // Into the pond from the bank (fix_swim_live's repro spot, yaw 1.57 walks +X into the water).
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [-82.3, 5.3, 32.05], 1.57);
    let fwd = Input { forward: 1.0, ..Default::default() };
    let mut t = 0;
    while sim.bond.substate != nf_game::player::SS_SWIM {
        step(&mut sim, fwd);
        t += 1;
        assert!(t < 600, "the walk never entered the pond: {:?}", sim.bond.pos);
    }
    // Level stroke: the lane settles at the walk target.
    for _ in 0..60 { step(&mut sim, fwd); }
    assert!((sim.bond.local[1] - CAP).abs() < 1e-3, "the level stroke settles at the cap: {}", sim.bond.local[1]);
    // Pitch 45° down (pitch is in π/2 units) and keep stroking.
    let dp = -0.5 - sim.bond.pitch;
    step(&mut sim, Input { forward: 1.0, look: dp, ..Default::default() });
    for _ in 0..60 {
        step(&mut sim, fwd);
        if sim.bond.substate != nf_game::player::SS_SWIM { break; }
    }
    assert_eq!(sim.bond.substate, nf_game::player::SS_SWIM, "still swimming");
    let c = std::f32::consts::FRAC_PI_4.cos();
    let lane = (0.2 * T_FWD / (1.0 - 0.8 * c)).min(CAP);
    assert!((sim.bond.local[1] - c * lane).abs() < 1e-3,
        "the pitched stroke's read-back lane is cos·lane = {} (the port eased the bare lane to the cap), got {}",
        c * lane, sim.bond.local[1]);
}

/// A long drop into the 07000002 pond: the swim substate zeroes the fall timer every tick, so
/// the count from the drop never reaches the next landing.
#[test]
fn a_drop_into_the_pond_clears_the_fall_timer_on_07000002() {
    let Some(mut sim) = load("07000002.bin") else { eprintln!("skip: NF_ISO not set"); return };
    // Over the pond's middle (the water cels sit at 3.765 [D probe]), 15 units above the surface.
    sim.bond = nf_game::player::Bond::new([-71.1, 18.5, 27.9], 0.4);
    let mut peak = 0;
    let mut t = 0;
    while sim.bond.substate != nf_game::player::SS_SWIM {
        step(&mut sim, Input::default());
        peak = peak.max(sim.bond.fall_timer);
        t += 1;
        assert!(t < 400, "the drop never reached the water: {:?}", sim.bond.pos);
    }
    assert!(peak > 70, "the drop counts past the 70-tick pain edge before the splash ({peak})");
    for _ in 0..5 { step(&mut sim, Input::default()); }
    assert_eq!(sim.bond.substate, nf_game::player::SS_SWIM);
    assert_eq!(sim.bond.fall_timer, 0, "the swim substate zeroes BL+0x928 [M 0x1949a8 switch]");
}
