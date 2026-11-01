//! lamp-wind: the game's hanging lamps never settle — the cel wind keeps them drifting.
//!
//! The chain, every step [M]: `Env_Create` 0x17b968 (placement 0xDF) writes a wind struct on
//! the cel `build_FindCel` 0x1c8cb0 picks (cel+0x2c; strength = target = param3·0.1, the dir
//! row of `RotMatrix` 0x1e2368 over the placement rotation, the falloff 1/(pCelList
//! record+0x1c radius), phase −1, the grow latch 1). `Env_Update` 0x17bc88 animates it once
//! per frame from the one `Rand_Rand(20000)` the frame drew (the phase sweeps −1..1, the
//! strength doubles up / decays between 0.25×target and target). `RB_Update` 0x1d6e38 calls
//! `RB_ApplyWind` 0x1d7c78, which lands `Env_GetWindVel` 0x17c048's cel wind AT A POINT for
//! type != 0 (obj+0xc4 + rb+0x9c − rb+0x13c) — torque, the only per-frame torque a freely
//! hanging Rot body gets; the up-righting spring (0x1d79f4, FRAME_RATE_MUL = 1 [L
//! ee_start_05.bin]) balances it. The port's lamps sat dead still because nothing fed the
//! wind in.
//!
//! The data [D the port's level parse]: 07000004's 0xDF #5 (rain, param3 = 6) lands in room
//! cel #1 — the same cel as lamps #39/#40 (type 1, mass 1). The live fresh-boot recording
//! [L, the task brief] shows the game's lamps 2-4° off rest and drifting, shot or not; this
//! file pins the drift (the port's gust cycle carries the tilt through the live band
//! repeatedly) and its zero-wind sister (the `Env_GetWindVel` target-0 gate [M 0x17c048]
//! leaves the lamps bit-identical). The AMPLITUDE question (the port's steady swing runs
//! wider than the live 2-4° band — see the report's live checks) is NOT pinned here: only
//! the proven chain is.
//!
//! Opt-in via `NF_ISO` (the slim ISO carries 07000004).

use nf_game::player::Input;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn load_04() -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004")
}

/// The tilt of the body's up row off vertical, in degrees.
fn tilt_deg(o: &nf_game::rigid_body::RbObj) -> f32 {
    let up = o.st.matrix[1];
    (up[0] * up[0] + up[2] * up[2]).sqrt().atan2(up[1]) * (180.0 / std::f32::consts::PI)
}

fn rotation_slots(o: &nf_game::rigid_body::RbObj) -> [u8; 76] {
    let mut b = [0u8; 76];
    let mut w = 0usize;
    for v in o.st.omega.iter().chain(o.st.tor_impulse.iter()) {
        b[w..w + 4].copy_from_slice(&v.to_le_bytes()); w += 4;
    }
    for v in &rb_quat(o) { b[w..w + 4].copy_from_slice(&v.to_le_bytes()); w += 4; }
    for row in &o.st.matrix { for v in row { b[w..w + 4].copy_from_slice(&v.to_le_bytes()); w += 4; } }
    b
}

fn rb_quat(o: &nf_game::rigid_body::RbObj) -> [f32; 4] { o.rb.quat }

/// The idle lamps drift and never settle: with no input at all the tilt keeps changing and
/// carries through the live 2-4° band repeatedly (the gust cycle: strength 0.6 → 1.2 → decay
/// → 0.15 → grow, [M 0x17bc88]). The first-cycle spike (K starts AT the target, so the first
/// grow arm doubles it past 0.6 — the game's own create values [M 0x17b968]) runs the tilt
/// higher than the live band for the first seconds; the window below starts after it.
#[test]
fn idle_lamps_drift_through_the_live_band_and_never_settle() {
    let mut scene = load_04();
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.rbodies.list.len(), 2, "07000004 places lamps #39/#40 [D the census]");
        assert_eq!(sim.drones.rbodies.winds.len(), 1, "the 0xDF #5 rain/wind placement builds the cel wind [D; M Env_Create 0x17b968]");
        assert!(sim.drones.rbodies.list.iter().all(|o| o.wind == Some(0)), "both lamps sit in the wind cel [D: cel #1 holds the 0xDF pos and both lamps]");
        for _ in 0..1500 { sim.step_bond(Input::default(), false, false, false); }
    }
    // The sampled window: 1200 ticks (20 s), every 100.
    let mut samples: Vec<f32> = Vec::new();
    for t in 0..1200 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if t % 100 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            samples.push(tilt_deg(&sim.drones.rbodies.list[0]));
        }
    }
    let (min, max) = (samples.iter().cloned().fold(f32::MAX, f32::min), samples.iter().cloned().fold(f32::MIN, f32::max));
    assert!(max - min > 2.0, "the lamp must DRIFT, not sit at one tilt: samples {samples:?}");
    let in_band = samples.iter().filter(|t| **t >= 1.9 && **t <= 4.1).count();
    assert!(in_band >= 2, "the drift must carry the tilt through the live 2-4° band [L the task brief]: samples {samples:?}");
    // Never settles: the last third of the window still spans over a degree.
    let tail = &samples[samples.len() * 2 / 3..];
    let (tmin, tmax) = (tail.iter().cloned().fold(f32::MAX, f32::min), tail.iter().cloned().fold(f32::MIN, f32::max));
    assert!(tmax - tmin > 1.0, "the lamp is still moving at the end of the window (never settles): tail {tail:?}");
    // The twin lamp moves with it (both lamps share the cel wind; their z — the only axis the
    // z-dir wind's dot reads — is identical [D the placements]).
    let sim = scene.sim.as_ref().unwrap();
    let t1 = tilt_deg(&sim.drones.rbodies.list[1]);
    assert!(t1 > 0.5, "lamp #40 drifts too, got {t1}°");
}

/// Sister: zero wind keeps the lamps bit-still — the game's `Env_GetWindVel` returns the zero
/// vector when the cel's wind target is 0 [M 0x17c048's `+0x34 == 0` gate; `Env_Update`
/// 0x17bc88 skips the cel entirely], the buoyancy gate never opens (the cel's water surface
/// is the −500 default [M build_alloc_cel 0x1c93f8]) so no RNG draw fires, and a type-1 body
/// with no torque and zero omega never integrates [M 0x1d7810/0x1d79ac]. With the wind gated
/// off the lamps' rotation state is bit-identical over the window — the pre-wind port model,
/// which is what a level without a wind-bearing 0xDF gets all the time.
#[test]
fn zero_wind_keeps_the_lamps_bit_still() {
    let mut scene = load_04();
    {
        let sim = scene.sim.as_mut().unwrap();
        // The [M] gate: target 0 → Env_GetWindVel's Vec_Zero, Env_Update skips the cel.
        for w in &mut sim.drones.rbodies.winds { w.target = 0.0; w.strength = 0.0; }
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    }
    let before: Vec<(u16, [u8; 76])> = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.rbodies.list.iter().map(|o| (o.placement, rotation_slots(o))).collect()
    };
    for _ in 0..1200 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let after: Vec<(u16, [u8; 76])> = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.rbodies.list.iter().map(|o| (o.placement, rotation_slots(o))).collect()
    };
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.0, a.0);
        assert_eq!(b.1, a.1, "lamp at placement {} moved with the wind gated off — a zero-wind body must be bit-still [M 0x17c048, 0x1d79ac]", b.0);
    }
}
