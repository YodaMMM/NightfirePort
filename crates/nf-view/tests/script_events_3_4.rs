//! script-events-3-4: pin `Script_EventHandler` (0x1d94c0) Event 3 and Event 4 on 07000013.
//!
//! Event 4 [M 0x1d94c0:118-121 = `Player_Disable(glb_players, 0)` + `PlarStat_LogTimerPause(0)`]:
//! the intro script 060006d7 fires it from its first tick (stream 7: `05` wait 0 → `12 00 04`
//! [D script_events_dump --ignored dump]). Test 1 drives the REAL input path (Walker →
//! `Sim::step_bond` + `Input`) and asserts input cannot move Bond while the intro runs, and can
//! after the hand-back. `Player_Disable` 0x195f70 zeroes `BL+0x94a`, which `Player_Update`
//! 0x193600:80 and `Player_CollisionHandler` 0x1949a8:24 gate on — the port's `disabled`.
//!
//! Event 3 [M 0x1d94c0:98-116]: `SCRIPTINFO+0x9C = p0 − 1` (the delay; every 06000959 use passes
//! p0 = 0 → the −1 hold [D]) plus the spawn-transform bake — every stream's entity/anim object
//! gets `Vec_Copy2(obj+0xC0, obj+0x30, obj+0x40)` [M call-site disasm 0x1d95e0-0x1d9614;
//! `Vec_Copy2` 0x1e2cf8 disasm: a0 read, a1 AND a2 written], i.e. position (+0x30) and previous
//! position (+0x40) snap to the matrix translation row (+0xC0). `Script_EntityStart`'s create
//! tail runs the identical copy (disasm 0x1d8f00-0x1d8f0c), so a fresh spawn is born baked. The
//! port's `Obj::Entity` carries only that matrix (position == translation by construction), so
//! the pinned observable is the baked transform itself: 06000959's t0 Entity op must spawn
//! SCRIPT_SHUTTER_CLOSE at the SP-base composed key transform — the disc keys are all identity
//! [D script_events_dump dump_06000959_keys_and_sp427], so the baked position IS the SP
//! placement translation (−1.4095, 0, 0) [D the 07000013 parse] — and the script must HOLD at
//! t0 (the −1 delay) with the t16 Entity op (SCRIPT_SHUTTER_OPEN) not yet spawned.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The intro's Event 4: input does not move Bond while 060006d7 runs, and does after the
/// hand-back. [M 0x1d94c0:118-121, 0x195f70:16-18, 0x193600:80, 0x1949a8:24; D stream 7 t0.]
#[test]
fn intro_060006d7_event_4_stops_bond_until_the_handback() {
    let scene = nf_view::load_with_carry(&iso(), "07000013.bin", false, None).expect("load 07000013");
    let mut w = Walker::new(scene);

    // Tick until the intro's Event 4 lands (the script plays at once, mode 1 [D SP #65 p2=1]).
    let mut disabled_at = None;
    for t in 0..400u32 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().disabled {
            disabled_at = Some(t);
            break;
        }
    }
    let t0 = disabled_at.expect("the intro never disabled Bond (event 4 missing)");
    let frozen = w.scene.sim.as_ref().unwrap().bond.pos;

    // Full-forward input through the real path while disabled: Bond must not move.
    for _ in 0..90 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.disabled, "the disable dropped before the intro ended");
        let d = walker::dxz(sim.bond.pos, frozen);
        assert!(d < 0.05, "input moved Bond {d:.3} units during the intro's disable (from tick {t0})");
    }

    // The hand-back (stream 11's event 6 at t349 → `Script_Free`'s enable) restores control.
    assert!(w.wait_enabled(3000, "the intro hand-back"), "Bond still disabled");
    w.wait(30); // settle after the hand-back drop
    let start = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..90 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    }
    let d = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, start);
    assert!(d > 0.3, "Bond still cannot move after the hand-back (moved only {d:.3})");
}

/// The shutters: 06000959's preload spawns SCRIPT_SHUTTER_CLOSE at the Event-3 baked transform
/// and holds at t0; SCRIPT_SHUTTER_OPEN (the t16 op) must not exist while the hold stands.
/// [M 0x1d94c0:98-116, 0x1d8f00-0x1d8f0c; D script_events_dump: keys identity, SP translation.]
#[test]
fn shutters_06000959_spawn_at_the_event_3_transform_and_hold() {
    let scene = nf_view::load_with_carry(&iso(), "07000013.bin", false, None).expect("load 07000013");
    let mut w = Walker::new(scene);
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); } // the SP priming beat

    let sim = w.scene.sim.as_ref().unwrap();
    // The SP that plays 06000959 (mode 0x40 preload [D placement p2=0x40]).
    let sp = sim.drones.scripts.players.iter()
        .find(|p| p.scripts[0] == 0x0600_0959)
        .expect("the 06000959 ScriptPlayer is missing");
    let script = sp.info[0].as_ref().expect("the shutter script is not loaded");
    // Mode 0x40's prime [M 0x1dbc08:41-55]: load, run one update, stop → state 1. The t0
    // commands ran inside that update; the Event-3 hold stands.
    assert_eq!(sp.state, 1, "the primed SP should sit in state 1");
    assert!(script.play & 1 == 0, "the primed script should be stopped (play = {:#x})", script.play);

    // The Event-3 hold: p0 = 0 [D] → delay = −1, the clock frozen at the first tick.
    assert!(script.delay != 0, "the shutter script is not held (delay = 0): event 3's delay write is missing");
    assert!(script.delay < 0, "expected the −1 hold, delay = {}", script.delay);

    // The baked spawn transform: the t0 Entity op's object at the SP-base composed key matrix —
    // identity keys [D] × the SP placement (−1.4095, 0, 0) [D].
    let close = sim.drones.scripts.objects.list.iter().flatten()
        .find(|o| matches!(o, nf_game::nis::Obj::Entity { hash: 0x0200_0a2c, src: (0x0600_0959, 0), .. }))
        .expect("SCRIPT_SHUTTER_CLOSE was not spawned by the preload");
    if let nf_game::nis::Obj::Entity { matrix, .. } = close {
        let t = matrix[3];
        assert!((t[0] - (-1.4095)).abs() < 0.01 && t[1].abs() < 0.01 && t[2].abs() < 0.01,
            "the shutter spawned off the baked transform: ({:.4},{:.4},{:.4})", t[0], t[1], t[2]);
    }

    // The t16 Entity op sits behind the hold: no OPEN shutter yet.
    assert!(!sim.drones.scripts.objects.list.iter().flatten()
        .any(|o| matches!(o, nf_game::nis::Obj::Entity { hash: 0x0200_0a2b, .. })),
        "SCRIPT_SHUTTER_OPEN spawned while the event-3 hold still stands");
}
