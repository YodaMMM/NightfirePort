//! gas-breath-cel: the real-play pins gas-cel-host left in "What's left" item 2, through the
//! real input path from the level start.
//!
//! - 07000004's wind cel 1 (the lamp cel — the 0xDF #5 radiator, target 0.6 [D the port's
//!   parse; the lamp-wind test pins its gust cycle]): Bond starts inside it, fires the P2K,
//!   and the muzzle smoke DRIFTS with the cel wind. `Player_MuzzleFlash` 0x1a4298 spawns the
//!   kind-7 puff with a NULL velocity, so `Gas_Init` takes the wind branch: the puff's vel =
//!   `Env_GetWindVel(cel, pos)` + spread, ·REC [M 0x1aacc8:108-115], the cel being the
//!   player's own (obj+0x20 [M the call's a0]). The pin diffs a wind-less twin load fired at
//!   the same RNG stream position (the wind's target zeroed — `Env_GetWindVel`'s Vec_Zero
//!   gate [M 0x17c048]; the wind itself consumes no draws [M 0x17c048 has no Rand call]): the
//!   per-component difference is the cel wind term — along the cel's wind dir [M 0x17c048's
//!   `dir·s` return], non-zero, inside the gust's bound (strength ≤ 2·target [M 0x17bc88's
//!   grow arm], the falloff factor ≤ 2 inside the radius).
//! - A flashbang's kind-0xb burst puff holds under a low ceiling: the pop arm passes the
//!   bullet's own cel [M Bullet_DoTrails 0x12d890's 0x12dd6c `*(iVar18 + 0x20)`] and
//!   `gases_smoke` clamps the puff DOWN to that cel's +0x54 every tick [M 0x1ab2d0 :200-203].
//!   Thrown at the ceiling through the real give/throw path, the white cloud must stop at the
//!   blast cel's max y instead of rising through it (the puff rises REC + 9.8·0.02·REC ≈
//!   0.0199/tick [M the 0xb case body], life 127 → a ~2.5 rise budget).
//!
//! Opt-in via NF_ISO (the slim ISO carries 07000004 and 07000005).

use nf_game::player::Input;

const REC: f32 = 1.0 / 60.0;

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn load(level: &str) -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    nf_view::load(&iso(), level, false).expect("load")
}

fn load_weapons(level: &str, ids: &[u8]) -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    nf_view::load_weapons(&iso(), level, false, ids).expect("load_weapons")
}


/// The settle gate: idle through the intro + story cut-scenes (the game's own path — the
/// PlayerDisable/PlayerEnable NIS events hold and re-enable the sim [M scene.rs's event arms])
/// until the AI is back. 07000004's full gate is the walkreal harness's own: `npc_enabled`,
/// `!disabled`, ch 0x53 (the castle-C hand-off [D script 0x06000082 stream 9]).
fn settle(scene: &mut nf_view::scene::Scene, ch_0x53: bool) -> u32 {
    for t in 0..6000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let ok = sim.drones.npc_enabled && !sim.disabled && (!ch_0x53 || sim.drones.channels.on(0x53));
        if ok { return t; }
    }
    panic!("the intro + story scenes never settled");
}

/// Give the panel weapon and wait for the draw.
fn give_and_switch(scene: &mut nf_view::scene::Scene, id: u8, rounds: u32) {
    let (msg, sw) = nf_view::panel::apply(scene, &[("do".into(), "weapon".into()), ("id".into(), id.to_string()), ("rounds".into(), rounds.to_string())]);
    eprintln!("[give] {msg} -> {sw:?}");
    for _ in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(id), ..Default::default() }, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if sim.gun.weapon == id && sim.gun.switch_state == 0 { return; }
    }
    panic!("the panel weapon {id:#04x} never drew");
}

/// ── Pin 1: the muzzle puff drifts with 07000004's wind cel 1 ────────────────────────────────
/// Bond settles from the level start inside the lamp cel (the 0xDF #5 radiator, target 0.6 [D
/// the port's parse]), fires the P2K (the panel take, the loadout's own weapon family), and
/// the kind-7 muzzle puff's velocity carries the cel wind: the pin fires a wind-less twin load
/// at the same RNG stream position (the wind's target zeroed — `Env_GetWindVel`'s Vec_Zero
/// gate [M 0x17c048]; the wind itself consumes no draws [M 0x17c048 has no Rand call]) and
/// diffs the two puffs. The difference must BE the cel wind term [M 0x1aacc8:108-115]:
/// non-zero, along the cel's wind dir, inside the gust bound (strength ≤ 2·target [M
/// 0x17bc88's grow arm], the falloff factor ≤ 2 inside the radius), ·REC.
#[test]
fn muzzle_puff_drifts_with_the_wind_cel_1() {
    // Run A: the wind live. Run B: the twin with the target zeroed right before the fire.
    let mut runs: Vec<[f32; 3]> = Vec::new();
    let mut spins = Vec::new();
    let mut draws = Vec::new();
    let mut positions = Vec::new();
    let mut wind_dir = [0.0f32; 3];
    for zeroed in [false, true] {
        let mut scene = load_weapons("07000004.bin", &[2]);
        give_and_switch(&mut scene, 2, 100);
        let settled = settle(&mut scene, true);
        let (bond, ci) = {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            (p, sim.drones.rbodies.cel_at(p))
        };
        let ci = ci.expect("the settled Bond stands inside a room cel");
        let wi = scene.sim.as_ref().unwrap().drones.rbodies.cels[ci].wind
            .expect("the settled cel is the wind cel (the 0xDF #5 radiator [D the census])");
        eprintln!("[A{}] settled tick {settled} bond {bond:?} cel {ci}", if zeroed { "B" } else { "A" });
        if zeroed {
            // The twin's control: target 0 reads as no wind [M 0x17c048's Vec_Zero gate] and
            // consumes no draws — every earlier tick is bit-identical to run A.
            scene.sim.as_mut().unwrap().drones.rbodies.winds[wi].target = 0.0;
        } else {
            wind_dir = scene.sim.as_ref().unwrap().drones.rbodies.winds[wi].dir;
        }
        // Idle a few ticks (the gust animates), then fire until the kind-7 muzzle puff lands.
        for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        let mut puff = None;
        for t in 0..15u32 {
            scene.sim.as_mut().unwrap().step_bond(Input { fire: true, aiming: true, ..Default::default() }, true, false, false);
            let sim = scene.sim.as_ref().unwrap();
            if let Some(g) = sim.drones.gas.puffs.iter().rev().find(|g| g.kind == 7) {
                eprintln!("[{}] kind-7 muzzle puff at fire tick {t}: pos {:?} vel {:?} (x60 = {:?}) draws {}",
                    if zeroed { "B" } else { "A" }, g.pos, g.vel, [g.vel[0]*60.0, g.vel[1]*60.0, g.vel[2]*60.0], sim.drones.rng.draws);
                puff = Some(((g.pos, g.vel, g.spin.to_bits()), sim.drones.rng.draws));
                break;
            }
        }
        let (g, d) = puff.expect("the P2K's muzzle smoke spawned (the record smokes [M 0x1a4298's gate])");
        positions.push(g.0); runs.push(g.1); spins.push(g.2); draws.push(d);
    }
    // The twins agree everywhere except the wind term. The puff is read after its first
    // gas tick (the pool updates within the same step), so its pos carries one tick of its
    // own motion — the wind-independent gravity term cancels in the diff, leaving
    // Δpos == Δvel [M the kind-7 case body's pos += vel].
    for k in 0..3 {
        let dp = positions[0][k] - positions[1][k];
        let dv = runs[0][k] - runs[1][k];
        assert!((dp - dv).abs() < 1e-6, "the spawn pos is wind-independent (the read carries one tick of motion), k={k} dp {dp} dv {dv}");
    }
    assert_eq!(spins[0], spins[1], "the spin draws are the same stream bits");
    assert_eq!(draws[0], draws[1], "the wind consumes no draws [M 0x17c048 has no Rand call]");
    let delta = [(runs[0][0] - runs[1][0]) * 60.0, (runs[0][1] - runs[1][1]) * 60.0, (runs[0][2] - runs[1][2]) * 60.0];
    eprintln!("[diff] velA−velB (x60) {delta:?} wind dir {wind_dir:?}");
    let mag = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
    assert!(mag > 0.05, "the cel wind actually moved the puff (the gust is live), delta {delta:?}");
    // The delta is the wind term: dir·s — parallel to the cel's wind dir [M 0x17c048's return].
    let dot = delta[0] * wind_dir[0] + delta[1] * wind_dir[1] + delta[2] * wind_dir[2];
    let cross2 = (mag * mag - dot * dot).abs();
    assert!(cross2 < 1e-3 * mag * mag + 1e-6, "the delta runs along the cel's wind dir [M 0x17c048], delta {delta:?} dir {wind_dir:?}");
    assert!(dot > 0.0, "the wind pushes along its own dir");
    assert!(dot <= 2.5, "inside the gust bound: strength <= 2·target = 1.2 [M 0x17bc88's grow], falloff factor <= 2, got {dot}");
    // And the REC multiply is the game's [M 0x1aacc8:108-115]: delta·REC == the term the host
    // resolution evaluates at the puff pos — the per-component bound the formula implies.
    for k in 0..3 {
        let d = (runs[0][k] - runs[1][k]).abs();
        assert!(d <= (2.5 / 60.0) + 1e-6, "per-component ·REC bound, k={k} got {d}");
    }
}

#[test]
fn flashbang_burst_puff_threads_the_blast_cel_and_never_rises_through_it() {
    let mut scene = load_weapons("07000012.bin", &[0x35]);
    give_and_switch(&mut scene, 0x35, 5);
    let settled = settle(&mut scene, false);
    let (bond, headroom) = {
        let sim = scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let hr = sim.drones.rbodies.cel_at(p).map(|ci| sim.drones.rbodies.cels[ci].max[1] - p[1]);
        (p, hr)
    };
    eprintln!("[fb] settled tick {settled} bond {bond:?} headroom {headroom:?}");
    // At the floor at Bond's feet (the fix-grenade harness's own at-feet pitch).
    scene.sim.as_mut().unwrap().bond.pitch = -0.9;
    let mut puff = None;
    for t in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t < 30, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(g) = sim.drones.gas.puffs.iter().rev().find(|g| g.kind == 0xb) {
            eprintln!("[fb] 0xb burst at tick {t}: pos {:?} vel {:?} ceil_y {:?}", g.pos, g.vel, g.ceil_y);
            puff = Some((g.pos, g.vel, g.ceil_y));
            break;
        }
    }
    let (bpos, _bvel, puff_ceil) = puff.expect("the flashbang's burst puff spawned [M Bullet_DoTrails 0x12d890's +0x70 & 0x2000 arm]");
    let sim = scene.sim.as_ref().unwrap();
    let bci = sim.drones.rbodies.cel_at(bpos).expect("the blast sits inside a room cel");
    let ceil = sim.drones.rbodies.cels[bci].max[1];
    assert_eq!(puff_ceil, Some(ceil), "the pop threads the blast cel's +0x54 [M 0x12dd6c + 0x1aacc8:84-87 + gases_smoke :200-203]");
    // The rise: vel (0, REC, 0) plus the case body's buoyancy, undamped — the measured rate
    // must stay inside the game's own band and never punch through the +0x54 word.
    assert!((_bvel[1] - REC).abs() < 1e-7, "the burst rises at the game's (0, REC, 0) [M 0x1aacc8:105-129]");
    let mut max_y = bpos[1];
    for t in 0..130u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let Some(cur) = sim.drones.gas.puffs.iter().find(|p| p.kind == 0xb) else { break };
        assert!(cur.pos[1] <= ceil + 1e-3, "the puff never rises through the ceiling word [M gases_smoke :200-203], tick {t} y {} ceil {ceil}", cur.pos[1]);
        max_y = max_y.max(cur.pos[1]);
    }
    let rise = (max_y - bpos[1]) / 130.0;
    eprintln!("[fb] blast {:.3} below the {:.3} ceil; measured rise {rise:.5}/tick", bpos[1], ceil);
    assert!(rise > 0.015 && rise < 0.025, "the rise is REC + the buoyancy term [M the 0xb case body], got {rise}");
}
