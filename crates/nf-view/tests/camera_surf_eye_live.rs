//! audit-player-camera-modes: the camera eye's surface offset (`BL+0x908`) — underwater the
//! game's eye eases 0.7 units DOWN to the body root and back up on surfacing, through the REAL
//! viewer path (`nf_view::load` + `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! Game evidence (ACTION.ELF):
//! - `Player_PositionCamera` 0x18f5d8 case 0 [M :113-128]: `BL+0x908 += (target − BL+0x908)·0.1`
//!   with target 0.7 while the `surfaced` global is set and 0.0 when it is clear, clamped at the
//!   target in the lerp's direction. `surfaced` is written by `Player_MonitorAir` 0x191a70
//!   [M :110 surfaced=1 in the head-above-surface refill branch, :124 surfaced=0 in the drain
//!   branch] — every tick, after the substate arm.
//! - The camera head reads it [M `Player_GetHeadPos` 0x195aa0:39-51: eye.y = obj+0x34 +
//!   BL+0x908 − BL+0x90c] — so underwater the first-person view sinks to the body root and the
//!   aim origin (`Player_GetAimingPoint` 0x1a4a98:52 calls `Player_GetHeadPos`) rides it.
//! - The ease runs only while the first-person camera is live (case 0/0xe; the wire/creep
//!   tracking cam 0xb and the script/missile/turret cams never touch BL+0x908) [M the switch].
//! - The port pinned `eye` at the standing 0.7 — the pre-fix symptom: diving on 07000002 kept
//!   the view 0.7 above the game's (the body root never became the eye).
//! - The drown head stays on the standing eye: `Player_MonitorAir` reads the ANIM head bone
//!   [M 0x191a70:50-53], which never sees BL+0x908 — the port's air path uses the standing eye
//!   and is untouched by this fix (the air/underwater-flag asserts in fix_swim* still pin it).

use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The 07000002 garden pond, exactly the fix_swim route: the fresh boot holds Bond in the intro
/// cut-scene SP #400 and releases him at frame ~1531 [L 2026-10-27 PCSX2]; the game's own skip
/// action resolves the scene so the placements below stick [M Script_FFwd 0x1dac78].
fn load() -> Sim {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let mut sim = scene.sim.take().expect("sim");
    let mut saw_scene = false;
    let mut n = 0u32;
    while n < 400 {
        if sim.disabled { saw_scene = true; } else if saw_scene { break; }
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        n += 1;
    }
    assert!(saw_scene && !sim.disabled,
        "the intro cut-scene never released Bond ({n} ticks, saw_scene {saw_scene})");
    sim
}

/// One step through the real path; the level's scripts grab Bond into cut-scenes within seconds,
/// so each tick carries the game's own skip action like the walkreal walker [M nis.rs].
fn step(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

/// The pond's bank [D probe, the fix_swim repro spot], yaw 1.57 walks +X into the pond.
/// Level-swim ticks out from the bank before the dive (over the pond's deeper floor).
const SWIM_OUT: usize = 60;
const BANK: [f32; 3] = [-82.3, 5.3, 32.05];

/// Walk Bond from the bank into the pond on the real path (the fix_swim recipe).
fn swim_in(sim: &mut Sim) {
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, BANK, 1.57);
    let walk = nf_game::player::Input { forward: 1.0, ..Default::default() };
    for t in 0..600 {
        step(sim, walk);
        if sim.bond.substate == nf_game::player::SS_SWIM { return; }
        let _ = t;
    }
    panic!("walking off the bank never entered the swim (pos {:?})", sim.bond.pos);
}

/// Diving lowers the eye: `BL+0x908` eases 0.1/tick toward 0.0 once the head is under the water
/// line [M 0x18f5d8:113-128 + 0x191a70:124], so the camera eye drops from the standing 0.7
/// toward the body root — visibly, about half the dip inside a second.
#[test]
fn diving_eases_the_camera_eye_down_to_the_body_root() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    swim_in(&mut sim);
    // At the surface the head (pos + 0.7) sits above the water line: surfaced, eye 0.7 [M 0x191a70:110].
    assert!((sim.bond.eye - 0.7).abs() < 1e-4, "the surfaced eye is the standing 0.7 (got {})", sim.bond.eye);
    // Swim out over deeper water first: with the pitched stroke easing from cos(pitch)·lane [M
    // Player_Move 0x191dd0:149-165] a dive straight off the bank reaches the shallow floor and
    // bobs the standing head back over the water line (surfaced again, the eye rightly rises).
    let level = nf_game::player::Input { forward: 1.0, ..Default::default() };
    for _ in 0..SWIM_OUT { step(&mut sim, level); }
    // Dive: pitch down over two seconds while holding forward — the swim stroke rides the look
    // direction [M Player_Move 0x191dd0:149-165], so Bond sinks past the head-under line.
    let dive = nf_game::player::Input { forward: 1.0, look: -0.03, ..Default::default() };
    let mut eye_a = sim.bond.eye;
    let mut dipped = false;
    for _ in 0..240 {
        step(&mut sim, dive);
        let head = sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop;
        let surface = sim.cel_at(sim.bond.pos).map(|i| sim.cels[i].water).filter(|w| *w > -400.0);
        if let Some(s) = surface {
            if head < s { dipped = true; } // the game's surfaced=0 branch [M 0x191a70:124]
        }
        if dipped {
            assert!(sim.bond.eye < eye_a + 1e-6,
                "the eye rose while the head was under the surface ({} -> {})", eye_a, sim.bond.eye);
            eye_a = sim.bond.eye;
        }
    }
    assert!(dipped, "the dive never put the head under the water line (pos {:?})", sim.bond.pos);
    assert!(eye_a < 0.35, "the eye did not ease toward the body root underwater (stopped at {})", eye_a);
    // Sister: the drown head did NOT ride the dip — the air drains from the standing eye
    // ([M 0x191a70:50-53, the anim head bone]; the port's standing-eye pin is the live check).
    assert!(sim.bond.air < 100.0, "the submerged Bond is not draining air");
}

/// Surfacing restores the standing eye: the head crosses the water line, `surfaced` flips to 1
/// [M 0x191a70:110] and `BL+0x908` eases 0.1/tick back to 0.7 [M 0x18f5d8:119-128's clamp arm].
#[test]
fn surfacing_eases_the_camera_eye_back_to_the_standing_height() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return; };
    let mut sim = load();
    swim_in(&mut sim);
    // Dive deep enough to dip the eye first.
    let dive = nf_game::player::Input { forward: 1.0, look: -0.03, ..Default::default() };
    for _ in 0..180 { step(&mut sim, dive); }
    assert!(sim.bond.eye < 0.7, "the dive did not lower the eye ({}); the surfacing arm is unpinned", sim.bond.eye);
    let eye_deep = sim.bond.eye;
    // Surface: pitch up — the rise clamps at surface − 0.3 [M 0x193600:201-220], the head crosses
    // the line and the eye eases back up.
    let up = nf_game::player::Input { forward: 1.0, look: 0.03, ..Default::default() };
    for _ in 0..240 { step(&mut sim, up); }
    assert!(sim.bond.eye > eye_deep + 0.2, "the eye did not rise on surfacing ({} -> {})", eye_deep, sim.bond.eye);
    assert!((sim.bond.eye - 0.7).abs() < 0.05, "the eye did not settle at the standing 0.7 (got {})", sim.bond.eye);
}
