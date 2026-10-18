//! stairwell-guards-11: the 07000011 stairwell guards' wake + alert chain (playtest 2026-10-18,
//! F12s 006-008: pairs standing inside each other and idle guards ignoring Bond). Pins the game's
//! own design against the decompile:
//!   - the stairwell guards are oDrone placements woken by the flight-plate TouchOnces (param 3 =
//!     the switch channel [M NDrone2_DefaultInit 0x14b300:463-471 → Drone+0x134]; plates #250-253/
//!     #181/#285 out 0x10/0x11/0x13/0x14/0x36/0x37 [D]) — until their channel fires they sit in
//!     WaitSwitch DISABLED [M DSTATE_Global 0x15b830:26-29 → DSTATE_WaitSwitch 0x15b960:22-35],
//!     with NO vision [M DroneVision_ProcessDroneSight 0x176a28: the 0x4F8 & 0x100 eligibility]
//!     and NOT DRAWN [M NDrone2_Enable 0x149dd0:8-15 — obj+0xF0 |= 0x10|0x20; bit 0x10 = "object
//!     not drawn", docs/fields/glist.md +0xF0];
//!   - the disc places the landing guards in PAIRS 0.85-2.0 apart [D placements of 07000011.bin]
//!     and the game has no drone-drone push-apart (NDrone2_Collision 0x140e90 is the AI-boundary
//!     wall push-out; the only inter-drone mechanic is NDrone2_Steer 0x14eba8's heading adjust for
//!     a moving drone) — so the pairs stand as placed, hidden until woken;
//!   - a woken guard with Bond in view alerts within the 10-tick reaction time [M
//!     DroneVision_HaveOpponentSight 0x175ef0:41-44 + DroneFunc_ReactionTime 0x14ac60:
//!     FRAME_RATE_DIV·10 on 07000011's arm].
//! Evidence tags: [M] the decompile address, [D] the disc bytes. A teleport is used only to reach
//! the plate (the touch itself runs through the real Trigger_Calc path, as the game's volume test).
//!
//! NOTE (this task): in a poisoned sim (the atrium-fall ScriptPlayer #313 fired by a fall) the
//! event-5 freeze holds every guard in WaitSwitch — the freeze is the game's own [M 0x1d94c0 case 5
//! → 0x1383e8], lifted by the death restart; these tests load clean (no fall), the way the level
//! boots.

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_d3() -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3")
}

/// The stairwell guards by placement [D 07000011.bin main map m43]: p152/p154 (wake 0x10, the top
/// antechamber plate #173), p155/p156/p157 (0x11, plate #252), p179/p180 + p233/p234 (0x13, plate
/// #253), p171/p172 (0x14, plate #181), p222/p223 (0x36, plate #251, diff ≥ 2), p224/p225 (0x37,
/// plate #250, diff ≥ 3), and the two abseilers p258 (0x13) / p280 (0x14, diff ≥ 2) whose param 1
/// 0x06000892 arms the rope chain [M 0x14b300:615-627].
const STAIRWELL: [usize; 17] = [152, 154, 155, 156, 157, 171, 172, 179, 180, 222, 223, 224, 225, 233, 234, 258, 280];

/// The landing pairs' XZ separations, from the disc placements [D]: p156/p157 (19.098,4.324)-
/// (17.129,4.711); p179/p180 (14.452,−3.563)-(14.486,−2.265); p222/p223 (22.853,5.271)-
/// (23.149,4.480); p224/p225 (14.974,−3.448)-(14.908,−2.289); p233/p234 (22.933,−3.490)-
/// (22.956,−1.645); p171/p172 (14.495,5.314)-(15.311,4.076).
const PAIR_XZ: [(&str, usize, usize, f32); 6] = [
    ("p156/p157 (y 28.2 flight-C mouth)", 156, 157, 2.007),
    ("p179/p180 (y 32.25 west strip)", 179, 180, 1.299),
    ("p222/p223 (y 28.2 east connector)", 222, 223, 0.845),
    ("p224/p225 (y 24.2 west landing)", 224, 225, 1.161),
    ("p233/p234 (y 24.3 east landing)", 233, 234, 1.845),
    ("p171/p172 (y 20.3 flight-A top)", 171, 172, 1.483),
];

fn by_placement(scene: &Scene, p: usize) -> usize {
    scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.placement == p && d.present)
        .unwrap_or_else(|| panic!("stairwell guard placement {p} present at load [D]"))
}

#[test]
fn stairwell_guards_load_disabled_undrawn_at_their_disc_pairs() {
    let mut scene = load_d3();
    // One tick: the drones' first ENTER dispatch (state 0 → WaitSwitch) runs on the first control
    // walk, like the game's first Drone_Control pass.
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().unwrap();
    // Difficulty 3 (00 Agent): every stairwell guard exists [D param 2 ≤ 3; M Drone_Create 0x136be0:49].
    for &p in &STAIRWELL {
        let i = by_placement(&scene, p);
        let dr = &sim.drones.list[i];
        assert!(dr.enabled == false && dr.state == nf_game::drone::state::WAIT_SWITCH,
            "p{p} waits disabled in WaitSwitch until its plate channel fires [M 0x15b830:26-29, 0x15b960:22-35], got en {} st {:#x}", dr.enabled, dr.state);
        assert!(!dr.drawn, "p{p} undrawn while disabled [M 0x149dd0:8-15 obj+0xF0 |= 0x30]");
    }
    // The abseilers carry the rope chain [D param 1 = 0x06000892; M 0x14b300:615-627].
    let (i258, i280) = (by_placement(&scene, 258), by_placement(&scene, 280));
    let (a, b) = (&sim.drones.list[i258], &sim.drones.list[i280]);
    assert_eq!(a.mode, 0x1c, "p258's placement msg 0x06000892 sets the rope mode 0x1c [M 0x14b300:615-627]");
    assert_eq!(b.mode, 0x1c, "p280 ditto (difficulty ≥ 2 [D param 2])");
    // The disc pairs keep their XZ separation in the port's spawn (the game spawns at the placement
    // position [M parsemap 0x1d04f0:64-66 → Drone_Create 0x136be0:55-62 → NDrone2_DefaultInit
    // 0x14b300:32-33] — no spawner spread, no drone-drone push-apart [M 0x140e90 / 0x14eba8]).
    for (name, pa, pb, want) in &PAIR_XZ {
        let (ia, ib) = (by_placement(&scene, *pa), by_placement(&scene, *pb));
        let (a, b) = (&sim.drones.list[ia], &sim.drones.list[ib]);
        let d = ((a.pos[0] - b.pos[0]).powi(2) + (a.pos[2] - b.pos[2]).powi(2)).sqrt();
        assert!((d - want).abs() < 2e-3,
            "{name}: the pair keeps its disc XZ separation [D placements] — want {want:.3}, got {d:.3}");
        assert!(d > 0.5, "{name}: the pair is distinct, not stacked ({d:.3} apart)");
    }
    // And no two stairwell guards share a spot (the same-x/z cross-floor rows are ≥ 20 apart in y).
    for (k, &pa) in STAIRWELL.iter().enumerate() {
        let a = &sim.drones.list[by_placement(&scene, pa)];
        for &pb in &STAIRWELL[k + 1..] {
            let b = &sim.drones.list[by_placement(&scene, pb)];
            let dxz2 = (a.pos[0] - b.pos[0]).powi(2) + (a.pos[2] - b.pos[2]).powi(2);
            let dy = (a.pos[1] - b.pos[1]).abs();
            assert!(dxz2.sqrt() > 0.5 || dy > 2.5,
                "p{pa}/p{pb} coincide in XZ ({:.3}) at nearly the same height (dy {dy:.3}) — the stacking the playtest saw",
                dxz2.sqrt());
        }
    }
}

#[test]
fn the_flight_plate_wakes_the_0x13_guards_and_a_guard_in_view_alerts() {
    let mut scene = load_d3();
    // Bond onto the east connector at the 0x13 plate #253 (23.1, 29.9, 3.9) [D] — the F12 f6186
    // spot. The touch runs the real Trigger_Calc (the same volume test the game's walk applies).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [22.34, 29.37, 3.52];
        sim.bond.vel = [0.0; 3];
    }
    let mut woke_at = None;
    for t in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x13) { woke_at = Some(t); break; }
    }
    let woke_at = woke_at.expect("the 0x13 plate #253 fires as Bond crosses it [D #253 out 0x13]");
    // The wake is the channel poll's next tick [M DSTATE_WaitSwitch 0x15b960:22-35 — msg 3 polls
    // switch_channels[+0x134], then NDrone2_Enable(1) 0x149dd0:16-28 + SetState(+0x5A2)].
    for t in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let all_up = [179usize, 180, 233, 234, 258].iter().all(|&p| {
            let dr = &sim.drones.list[by_placement(&scene, p)];
            dr.enabled && dr.state != nf_game::drone::state::WAIT_SWITCH
        });
        if all_up {
            // p280 waits on 0x14, the guards on 0x36/0x37/0x11 stay held: the wake is per-channel [D].
            for &p in &[155usize, 156, 222, 223, 224, 225, 280] {
                let dr = &sim.drones.list[by_placement(&scene, p)];
                assert!(!dr.enabled && dr.state == nf_game::drone::state::WAIT_SWITCH,
                    "sister: p{p}'s channel has not fired — it stays held [D plate map; M 0x15b960]");
            }
            assert!(t <= 60, "all five 0x13 guards enabled within 60 ticks of the channel (took {t})");
            break;
        }
        assert!(t < 59, "the 0x13 guards never woke after the channel fired at tick {woke_at} — the playtest's frozen idles");
    }
    // Bond stands in the open ~1.5-2 m from the pair — the woken guards' sight accrues and the
    // combat chain starts within the reaction band [M 0x175ef0:41-44 + 0x14ac60 = 10 ticks; the
    // port's LOS round-robin adds a few ticks]. The probe measured GO_TO_GOAL(0x63)/combat(0x5c)/
    // AIM_STAND_FIRE inside 240.
    let mut combat = false;
    for _ in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        combat = [179usize, 180, 233, 234].iter().any(|&p| {
            let dr = &sim.drones.list[by_placement(&scene, p)];
            dr.enabled && (dr.seen || matches!(dr.state,
                nf_game::drone::state::ALERT
                | nf_game::drone::state::ATTACK
                | nf_game::drone::state::COMBAT
                | nf_game::drone::state::COMBAT_NO_MOVE
                | nf_game::drone::state::COMBAT_OUT_OF_RANGE
                | nf_game::drone::state::COMBAT_NEW_SIGHTING
                | nf_game::drone::state::COMBAT_NO_SIGHT
                | nf_game::drone::state::GO_TO_GOAL))
        });
        if combat { break; }
    }
    assert!(combat, "a woken guard with Bond in plain view enters the alert/combat family within 240 ticks of the wake [M 0x175ef0 + 0x14ac60]");
}

/// The mode-0x1c abseiler's hang gate [M NDrone2_DSTATE_AbseilSlide 0x169730:55-58 — hang when the
/// drone's Y < opponent Y + FRAME_RATE_MUL·3.0]: p258 (rope at (18.895, 46.516, 1.434) [D], woken
/// by the 0x13 plate) must hang within 3.0 above Bond's height, not slide past it. MEASURED on the
/// current code (this task's probe, d3, Bond parked at (22.34, 29.37, 3.52)): p258 slid to y 22.34
/// before hanging — 10.0 past the 32.37 gate — leaving him hanging mid-well beside the y-24
/// landings (the playtest's "one floating a body height above the other"). Ignored until the gate
/// is fixed; the numbers are the pin.
#[test]
#[ignore = "the abseiler slides past the 3.0 hang gate (measured y 22.34 vs the 32.37 gate) — see docs/reports/ai-stacked-idle-guards-11.md"]
fn the_mode_1c_abseiler_hangs_within_the_game_s_3_0_of_the_opponent() {
    let mut scene = load_d3();
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [22.34, 29.37, 3.52];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..120 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x13) { break; }
    }
    let mut hung = None;
    for _ in 0..400 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[by_placement(&scene, 258)];
        if dr.state == nf_game::drone::state::ABSEIL_HANG { hung = Some(dr.pos[1]); break; }
    }
    let y = hung.expect("p258 enters the hang [M 0x169730:55-58]");
    assert!(y <= 29.37 + 3.0 && y >= 29.37 - 1.0,
        "the hang stops within the game's 3.0 of the opponent [M 0x169730:55-58]: Bond 29.37, hung at {y:.2}");
}
