//! swim-height-02: the swimmer floats ~0.5 too high in the 07000002 pond, through the REAL viewer
//! path (`nf_view::load` + `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! PCSX2 truth [L, fresh 07000002 boot 2026-10-16, Bond placed by PINE in the pond cel 0x1708870,
//! stick forward held — the same protocol ladder_underwater_02_live pins]: swimming into the hut
//! pocket under the hut ladder (centre (−74.84, ·, 22.22)) the PS2 root settles at y 3.17 at
//! (−76.27, 3.17, 22.14); in open water near (−71.1, ·, 27.9) it sits at y 3.48. The port,
//! swimming the same way, rises to 3.70 in the pocket and 3.60 in open water; its log shows the
//! swim capsule touching the bed and pushing up (grounded at tick 12 of the swim-in). The swim
//! root height animobj+0xCC is the same in both (PS2 1.0304, port 1.035), so the capsule SIZE is
//! not the cause — the capsule SHAPE is:
//!
//! [M Player_Collision 0x194df0's hittest capsule switch indexes substate−1 (disasm
//! 0x19506c-0x195098 `lhu 0xf6($s3)`, `addiu −1`, the jump table @0x2fc0d0)]: its DEFAULT arm
//! (radius +0x7CC = 0x3f0ccccd (0.55), A(+0x760) = pos + up·0x3e8ccccd (0.275), B(+0x770) =
//! pos + up·(0.55 − AnimObj+0xCC)) covers the walk 0 AND the swim 3 — the underwater path jumps
//! past the gravity straight into this switch [M disasm 0x194f4c-0x194f70: `cel+0x90 > obj+0x34`
//! → `bc1fl` past `WldGravity` to the switch]. Only case 3 — the crouch substate 4 — takes the
//! LOW arm (A = pos + up·(0.1 − (cc − 0.55)), B = pos + up·(0.45 − cc)). The port's
//! `Bond::capsule()` handed the SWIM substate the crouch LOW arm, hanging the spine 0.585 under
//! the root instead of 0.485: with r 0.55 the LOW capsule's lowest point sits cc + 0.10 below the
//! root against the game's cc, so the bed push-out parks the root ~0.1 high — the observed
//! 3.60 open-water settle (bed 2.4496 + 1.135) against the PS2's 3.48 (bed 2.4496 + 1.0304).
//! (The bed roots decode from the [L] settles: open bed 3.48 − 1.0304 = 2.4496; pocket bed
//! 3.17 − 1.0304 = 2.1396.) The port's own ladder pick already rides the default arm in swim
//! [M ladder.rs `capsule_hits_default` — "the switch value is substate−1, so the DEFAULT covers
//! the swim 3"]; only the collision capsule disagreed with itself.

use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> Sim {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    scene.sim.take().expect("sim")
}

/// One step through the real path; the level's scripts grab Bond into cut-scenes within seconds,
/// so each tick carries the game's own skip action like the walkreal walker — [S] unverified (nis.rs is the port's skip plumbing; no decompile line pinned here).
fn step(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

fn wrap_pi(d: f32) -> f32 {
    let mut d = d % std::f32::consts::TAU;
    if d >= std::f32::consts::PI { d -= std::f32::consts::TAU; }
    if d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
    d
}

/// The [L] settle protocol: 200 idle ticks (the level's scripts settle — the swim-out test's
/// run()), then the PINE placement in the pond cel, then stick forward held. Returns the mean
/// root over the last 60 ticks once the height stops moving.
fn settle(sim: &mut Sim, spot: [f32; 3], yaw: f32, what: &str) -> f32 {
    for _ in 0..200 { step(sim, nf_game::player::Input::default()); }
    sim.bond = nf_game::player::Bond::new(spot, yaw);
    let mut ys = Vec::new();
    for t in 0..240u32 {
        step(sim, nf_game::player::Input { forward: 1.0, ..Default::default() });
        if t % 30 == 0 {
            eprintln!("{what} t {t}: sub {} pos {:?} grounded {} water {:?} cc {:.4}",
                sim.bond.substate, sim.bond.pos, sim.bond.grounded,
                sim.cel_at(sim.bond.pos).map(|i| sim.cels[i].water), sim.bond.cc);
        }
        ys.push(sim.bond.pos[1]);
    }
    let tail = &ys[ys.len() - 60..];
    let mean = tail.iter().sum::<f32>() / tail.len() as f32;
    let spread = tail.iter().fold(0.0f32, |m, &y| m.max((y - mean).abs()));
    eprintln!("{what}: settled mean y {mean:.4} (spread {spread:.4}, sub {})", sim.bond.substate);
    mean
}

/// The open-water hold pins the PS2's 3.48 [L] within 0.1: placed at the [L] spot and
/// station-kept there (the [L] settle is a measurement AT the spot, not a glide), the root
/// holds 3.48 with no phantom rise — the free-water shape of the capsule fix. (The driver's
/// pre-fix port read 3.60 here — a bed-cap contact the [L] protocol reached from its own
/// placement; through this path the same contact binds only NE of the spot, z ≈ 36, past the
/// [L] swim-out's farthest coverage (z 27.94), so the cap is pinned at the pocket leg instead.)
#[test]
fn open_water_settles_at_the_ps2_height() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    for _ in 0..200 { step(&mut sim, nf_game::player::Input::default()); }
    // yaw 0.4 — the swim-out heading that crosses this spot [L ladder_underwater_02_live].
    sim.bond = nf_game::player::Bond::new([-71.1, 3.48, 27.9], 0.4);
    let mut ys = Vec::new();
    let mut swam = true;
    for t in 0..240u32 {
        // Station-keep: hold forward but steer back onto the spot, like a player idling there.
        let p = sim.bond.pos;
        let want = (-71.1 - p[0]).atan2(27.9 - p[2]);
        let diff = wrap_pi(want - sim.bond.yaw);
        let turn = (diff * 3.0).clamp(-1.0, 1.0);
        step(&mut sim, nf_game::player::Input { forward: 1.0, turn, ..Default::default() });
        let dxz = ((sim.bond.pos[0] + 71.1).powi(2) + (sim.bond.pos[2] - 27.9).powi(2)).sqrt();
        if dxz <= 0.6 {
            if t % 30 == 0 {
                eprintln!("open t {t}: sub {} pos {:?} grounded {} water {:?}",
                    sim.bond.substate, sim.bond.pos, sim.bond.grounded,
                    sim.cel_at(sim.bond.pos).map(|i| sim.cels[i].water));
            }
            ys.push(sim.bond.pos[1]);
            if sim.bond.substate != nf_game::player::SS_SWIM { swam = false; }
        }
    }
    assert!(!ys.is_empty(), "the swimmer never stayed within 0.6 of the [L] spot");
    let mean = ys.iter().sum::<f32>() / ys.len() as f32;
    let spread = ys.iter().fold(0.0f32, |m, &y| m.max((y - mean).abs()));
    eprintln!("open: held mean y {mean:.4} over {} samples (spread {spread:.4})", ys.len());
    assert!((mean - 3.48).abs() <= 0.1, "open-water hold y {mean:.3} vs PS2 3.48 [L]");
    // Sister: still swimming while held at the spot (the exit needs grounded AND feet 0.3 above
    // the surface [M 0x193600:189-194] — open water grounds nothing).
    assert!(swam, "the swim left the substate while held at the [L] spot");
}

/// The hut-pocket settle pins the PS2's 3.17 [L] within 0.1. The port rose to 3.70 here —
/// the same bed-riding capsule plus the pocket's hut/ladder contact above the bed.
#[test]
fn hut_pocket_settles_at_the_ps2_height() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    // yaw −π/2 faces −x, INTO the pocket (away from the ladder east of the spot) — "swimming
    // into the hut pocket" [L]; the swim-out test proved the spot itself grabs nothing.
    let y = settle(&mut sim, [-76.27, 3.17, 22.14], -std::f32::consts::FRAC_PI_2, "pocket");
    assert!((y - 3.17).abs() <= 0.1, "hut-pocket settle y {y:.3} vs PS2 3.17 [L]");
    // Sister: the swim holds — no spurious exit or ladder latch under the hut (the spot sits
    // 0.955 off the ladder axis, out of the pick's reach [L ladder_underwater_02_live]).
    assert_eq!(sim.bond.substate, nf_game::player::SS_SWIM, "the swim left the substate in the pocket");
    assert!(sim.bond.ladder.is_none(), "the pocket settle latched the hut ladder");
}
