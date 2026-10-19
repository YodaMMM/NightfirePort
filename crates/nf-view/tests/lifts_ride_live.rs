//! lifts-buttons-all-levels: the ride mechanisms through the real path.
//!
//! Two live tests, one per mechanism this task adds a pin for (docs/systems/lifts.md §6):
//!
//! 1. The paternoster (0700000b lift_scriptplayer #232): SP mode 8 is PRIMED at create —
//!    `SP_CreateScriptPlayer` 0x1db9a8's tail calls `SP_LoadScript` for every mode ∉ {0, 1}
//!    regardless of the trigger mask, and `SP_LoadScript` 0x1dbc08's mode-8 arm runs
//!    `Script_Play(si, 1)` (looping) and sets the state word 6 — the cab cycles FROM LEVEL LOAD.
//!    The script (0x06000048 [D nis dump]) creates entity 0x2000367 and keys it y 11 → 48.44 →
//!    11 over 1239 ticks, holding itself 499 ticks at each end (stream 3: `Event{kind 3, params
//!    [0x1f4]}` at t0 and t630; event 3 → the SCRIPTINFO delay = p0 − 1 [M 0x1da998's
//!    negative-delay arm]). Bond standing on the cab is carried [M `Control_InheritVelocity`
//!    0x133e88]. `lift_stop_trigger` #245 (the roof-deck touch volume, out ch 0x75 [D trigger
//!    dump]) is the channel SP_Update 0x1dbda8's case-2 rule consumes: mask bit 0 + the out
//!    channel ON → `SP_UnPause` 0x1dca58 clears SCRIPTINFO+0x9C every tick — the self-holds
//!    release while Bond is on the roof.
//! 2. The 07000009 lobby cab pair (#21/#22 `SCRIPT_liftSCRIPTPLAYER`, mode 8, mask 4, out 0
//!    [D placements]): the same prime — the cab cycles from load as idle scenery; out 0 means
//!    the use key can never write a channel through them (elevator-button.md).
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test lifts_ride_live -- --test-threads=1`

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn cab_y(scene: &nf_view::scene::Scene) -> Option<f32> {
    let sim = scene.sim.as_ref().unwrap();
    sim.drones.scripts.objects.list.iter().find_map(|o| match o {
        Some(nf_game::nis::Obj::Entity { hash: 0x2000367, matrix, .. }) => Some(matrix[3][1]),
        _ => None,
    })
}

fn lift_0b(scene: &nf_view::scene::Scene) -> &nf_game::nis::Player {
    scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .find(|p| p.scripts[0] == 0x0600_0048)
        .expect("lift_scriptplayer #232 [D placements]")
}

/// The paternoster: primed looping at load, Bond aboard is carried, and the roof channel
/// releases the self-holds. Sisters: the bottom hold EXISTS while ch 0x75 is off (the delay
/// counts down over ~499 ticks), and the hold RELEASES once the channel is on.
#[test]
fn the_paternoster_carries_bond_from_load() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b");
    // The prime runs on the first update tick (Scripts::update's prime pass [M 0x1db9a8 tail]).
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    // Primed at create: the state word is 6 with the loop bit set — BEFORE any input [M 0x1dbc08].
    {
        let sp = lift_0b(&scene);
        assert_eq!(sp.state, 6, "the mode-8 prime landed the SP in state 6 [M 0x1dbc08]");
        let info = sp.info[sp.cur as usize].as_ref().expect("script 0x06000048 loaded at create");
        assert!(info.play & 8 != 0 && info.play & 1 != 0, "Script_Play(si, 1): playing + looping [M 0x1dbc08 mode-8 arm]");
    }
    // The cab exists at its key-0 pose (y 11 [D nis dump]) and HOLDS there: the t0 self-hold
    // (event 3, p0 = 0x1f4 → delay 499 [M 0x1da998]) counts down before the first key moves.
    // The negative sister: the delay is NOT released while ch 0x75 is off (no writer on this
    // level fires it before the roof).
    {
        let sp = lift_0b(&scene);
        let info = sp.info[sp.cur as usize].as_ref().unwrap();
        assert!(info.delay > 400, "inside the t0 self-hold at load, delay = {} [D event 3 p0 0x1f4]", info.delay);
    }
    for tick in 0..250 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if tick % 60 == 0 {
            let sp = lift_0b(&scene);
            let info = sp.info[sp.cur as usize].as_ref().unwrap();
            assert!(info.delay > 150, "the bottom hold is still counting at tick {tick} (delay {}), not released [S sister: ch 0x75 is off]", info.delay);
        }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!((cab_y(&scene).unwrap() - 11.0).abs() < 0.05, "the cab holds key 0 (y 11) through the bottom hold [D keys]");
    }

    // Bond drops onto the cab (the cab's stand surface ≈ 15.1 at key 0; x 0, z ≈ 25 [D placement
    // box 23.19..26.56]). He grounds and rides: the carry [M 0x133e88].
    scene.sim.as_mut().unwrap().bond.pos = [0.0, 16.0, 25.0];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    let y0 = scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(scene.sim.as_ref().unwrap().bond.grounded, "Bond stands on the held cab");
    assert!((y0 - 16.11).abs() < 0.6, "grounded on the cab floor at key 0, got {y0}");

    // The ride up: by the top of the cycle the cab is at its 48.44 key and Bond rose with it.
    let mut top_at = None;
    for t in 1..=1400u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let cy = cab_y(&scene).unwrap();
        if cy > 48.0 { top_at = Some(t); break; }
    }
    let top_at = top_at.expect("the cab reaches its 48.44 key [D nis dump]");
    let _ = top_at;
    let y1 = scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(y1 > y0 + 30.0, "Bond carried up with the cab: {y0} → {y1} [M Control_InheritVelocity 0x133e88]");
    assert!(scene.sim.as_ref().unwrap().bond.grounded, "still standing on the cab at the top");

    // The top self-hold engages at the t630 key (the event fires as the stream reaches it).
    // The sister contrast: WITHOUT ch 0x75 the hold sits; WITH ch 0x75 (lift_stop_trigger
    // #245's out channel [D]) the case-2 rule releases it and the cab resumes downward.
    let mut held_at = None;
    for t in 1..=120u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if lift_0b(&scene).info[lift_0b(&scene).cur as usize].as_ref().unwrap().delay > 100 { held_at = Some(t); break; }
    }
    let held_at = held_at.expect("the top self-hold engages at the 48.44 key [D event 3 at t630]");
    let _ = held_at;
    let frame = scene.sim.as_ref().unwrap().drones.frame;
    scene.sim.as_mut().unwrap().drones.channels.set(0x75, 1, frame);
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    {
        let sp = lift_0b(&scene);
        let info = sp.info[sp.cur as usize].as_ref().unwrap();
        assert_eq!(info.delay, 0, "SP_UnPause cleared the delay the tick the hold engaged [M 0x1dbda8 case 2 → 0x1dca58]");
    }
    let mut down_at = None;
    for t in 1..=400u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if cab_y(&scene).unwrap() < 47.5 { down_at = Some(t); break; }
    }
    let down_at = down_at.expect("the cab resumes its descent once the hold is released");
    assert!(down_at <= 400, "descends within the window (tick {down_at})");
    let y2 = scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(y2 < y1 - 1.0, "Bond carried back down with the cab: {y1} → {y2} [M 0x133e88]");
}

/// The 07000009 lobby cab pair: mode 8, mask 4, out 0 [D placements] — primed looping at load
/// (state 6), and the loop scripts pose the cab: script 0x0600007E's stream 6 creates the cab
/// root entity 0x2000506 (unparented) keyed at t 0/124/180/338/710/869/1080 — the 1080-tick
/// loop [D nis dump]. Out 0 means the use key can never write a channel through them.
#[test]
fn the_09_cab_pair_loops_from_load() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    let cab_ys = |scene: &nf_view::scene::Scene| -> Vec<f32> {
        scene.sim.as_ref().unwrap().drones.scripts.objects.list.iter().filter_map(|o| match o {
            Some(nf_game::nis::Obj::Entity { hash: 0x2000506, matrix, src, .. }) if src.0 == 0x0600_007E => Some(matrix[3][1]),
            _ => None,
        }).collect()
    };
    let mut ys = Vec::new();
    for tick in 0..1200 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if tick % 200 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            let pair: Vec<&nf_game::nis::Player> = sim.drones.scripts.players.iter()
                .filter(|p| p.scripts[0] == 0x0600_007E || p.scripts[0] == 0x0600_007D)
                .collect();
            assert_eq!(pair.len(), 2, "both cab SPs present [D placements #21/#22]");
            for p in &pair {
                assert_eq!(p.state, 6, "the mode-8 prime: state 6 [M 0x1dbc08]");
                let info = p.info[p.cur as usize].as_ref().expect("the loop script loaded at create");
                assert!(info.play & 8 != 0, "looping [M Script_Play(si, 1)]");
            }
            ys.extend(cab_ys(&scene));
        }
    }
    // The cab root's keyed y changes over the loop (the art is positioned by the loop scripts [D]).
    let moved = ys.windows(2).any(|w| (w[0] - w[1]).abs() > 0.05);
    assert!(moved, "the loop scripts pose the cab: y samples {ys:?}");
}
