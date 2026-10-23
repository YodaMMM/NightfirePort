//! garage-elevator-buttons-09: the 07000009 car-park lift-call stations through the real input path.
//!
//! The underground car park has TWO lift shafts: the exit lift (the LIFTDOORS_*_ENDLEVELDOORS SS
//! pair + the LIFT_(FAKE)/liftswitch_fake static art, docs/levels/07000009.md) and the script
//! lift — the `SCRIPT_liftSCRIPTPLAYER` pair #21/#22 (0xD9, scripts 0x0600007E/0x0600007D, mode 8,
//! mask 4, out 0 [D placements]), whose 1080-tick loop scripts animate a cab (entity 0x02000506
//! `SCRIPT_LIFT_IN`) between the lobby (y −368.19) and the car park (y −375.84) with outer doors
//! at both floors (0x020006a8/a9 `SCRIPT_LIFTDOOROUTER_L/Rcarpark`), plus the call-station art:
//! the sub-script 0x0600064F creates `liftswitch_off` (0x020006b2) at t0 and `liftswithch_on`
//! (0x020006b3) at t7 of the sub stream 2, baked at (−4.62, −375.52, 25.91) [D nis dump] — the
//! car-park call button; its twin instances (the host streams' keyed bases) cover the lobby
//! station and both lifts' second column.
//!
//! The mechanism [M]: the mode-8 prime leaves the SP in state 6 with the script PLAYING but
//! HELD — stream 9's `Event{kind 3, p0 = 0}` at t1 sets SCRIPTINFO+0x9C = p0 − 1 = −1
//! [M 0x1d94c0:98-100], and `SP_Update` case 6 [M 0x1dbda8:193-216] only releases it when the
//! latch (`SP_Activate` 0x1dc338:82 `ext+0x2C = 1`) arrives: the use pick hits one of the SP's
//! class-0 entities (`obj+0xE4` — stamped from the SP handle for the TOP-LEVEL script's entities
//! [M SP_LoadScript 0x1dbc08 param_2 → Script_Load 0x1d8248:135 `+0x80 = param_5` →
//! Script_EntityStart 0x1d8d78:108-110 `entity+0xE4 = SCRIPTINFO+0x80`]; sub-script entities get
//! owner 0 [M Script_SubScriptStart 0x1db050:28 passes param_5 = 0] and are NOT use targets — the
//! station press lands on the outer-door entities that flank the button art). The latch clears
//! +0x9C [M 0x1dc338:83-85] and the script resumes: the button art swaps to the ON variant at
//! t7 (the indicator light, [L 2026-10-17: dim → bright white]), the cab descends (stream 6 keys
//! t180 → t338), the car-park outer doors slide open (stream 7/8 keys t422-531) and shut (t652),
//! the lift sounds play (stream 0: 0x28a t2, 0x28c t124, 0x288 t152, 0x289 t422… [D nis dump]),
//! and stream 9's second hold (t214) pauses the ride until the next call. Out 0: no channel ever
//! changes — ch 0x39 (the exit lift) stays off [L: no alarm, ch 0x39 unchanged; M 0x1dbda8:221-229
//! skips the write for out 0].
//!
//! The port bug: the use pick ran from the RENDER CAMERA (`player_use_doors`'s head arg), while
//! the game builds it at Bond's head (`Player_GetHeadPos` [M 0x1a5170:100-108]); the camera hangs
//! ~2.5 behind Bond, and the station entities (door spheres r 1.451 at z 26.32, the button art
//! r 0.20) sat outside the camera-based sphere — the buttons never responded.
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test garage_lift_calls_live -- --test-threads=1`

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_game::nis::World;

const SCRIPT_21: u32 = 0x0600_007E;
const SCRIPT_22: u32 = 0x0600_007D;

fn sp<'a>(sim: &'a nf_view::scene::Sim, hash: u32) -> &'a nf_game::nis::Player {
    sim.drones.scripts.players.iter().find(|p| p.scripts[0] == hash)
        .expect("the SCRIPT_liftSCRIPTPLAYER placement [D #21/#22]")
}

/// All world translations of live script entities with (src, hash) — the sub-script creates
/// SEVERAL instances (the four call stations + the cab-riding panels) [D host streams 13-18].
fn ents(sim: &nf_view::scene::Sim, src: (u32, u8), hash: u32) -> Vec<[[f32; 4]; 4]> {
    sim.drones.scripts.objects.list.iter().flatten()
        .filter_map(|o| match o {
            nf_game::nis::Obj::Entity { hash: h, matrix, src: s, .. } if *s == src && *h == hash => Some(*matrix),
            _ => None,
        }).collect()
}

/// The instance nearest `near` (the station spot).
fn ent_near(sim: &nf_view::scene::Sim, src: (u32, u8), hash: u32, near: [f32; 3]) -> Option<[[f32; 4]; 4]> {
    let mut best: Option<([[f32; 4]; 4], f32)> = None;
    for m in ents(sim, src, hash) {
        let d = (m[3][0] - near[0]).powi(2) + (m[3][1] - near[1]).powi(2) + (m[3][2] - near[2]).powi(2);
        if best.map_or(true, |(_, bd)| d < bd) { best = Some((m, d)); }
    }
    best.map(|(m, _)| m)
}

fn any_ent(sim: &nf_view::scene::Sim, src: (u32, u8), hash: u32) -> bool {
    !ents(sim, src, hash).is_empty()
}

/// Bond placed at the car-park call station of the left lift (SP #22), facing the button art.
/// The station button (sub-entity 0x020006b2) sits at (−4.62, −375.88, 25.91) [D 0x0600064F
/// stream 2 keys, composed]; Bond stands ~1.3 units in front of it (z 24.6), eye ~1.5 above the
/// floor, pitched down at it, yaw 0 = +z (`Bond::look` (cp·sin yaw, sin pitch, cp·cos yaw)).
fn stand_at_station(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [-4.6, -375.98, 24.6];
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = 0.0;
    sim.bond.pitch = -0.19; // down at the button (−1.4 below the eye, 1.3 ahead)
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
}

#[test]
fn the_car_park_call_button_lights_and_calls_the_lift() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    // The mode-8 prime runs on the first tick [M 0x1db9a8 tail + 0x1dbc08].
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = sp(sim, SCRIPT_22);
        assert_eq!(p.state, 6, "the mode-8 prime left the SP in state 6 [M 0x1dbc08 mode-8 arm]");
        assert!(p.info[p.cur].as_ref().unwrap().delay < 0,
            "the script holds itself at t1 (stream 9 event 3 p0 = 0 → delay −1) [M 0x1d94c0:98-100]");
        // The station art exists from the prime (the sub-script entities are created and posed).
        let btn = ent_near(sim, (0x0600_064F, 2), 0x0200_06b2, [-4.62, -375.88, 25.91])
            .expect("the liftswitch_off call-button art exists at the station [D 0x0600064F stream 2 t0]");
        assert!(f32::max(f32::max((btn[3][0] + 4.62).abs(), (btn[3][1] + 375.88).abs()), (btn[3][2] - 25.91).abs()) < 0.06,
            "the button art stands at the composed station spot (−4.62, −375.88, 25.91), got {:?}", btn[3]);
        assert!(!any_ent(sim, (0x0600_064F, 2), 0x0200_06b3),
            "the ON variant is not created before the call [D 0x0600064F stream 2 t7 — the script is held at t1]");
        // The cab is parked at the lobby deck [D stream 6 keys t0/t124; L probe −368.19].
        let cab = ent_near(sim, (SCRIPT_22, 6), 0x0200_0506, [-2.77, -368.19, 28.08]).expect("the cab entity [D stream 6]");
        assert!((cab[3][1] + 368.19).abs() < 0.05, "the cab parked at the lobby y −368.19, got {}", cab[3][1]);
        // The exit-lift channel is off [L: ch 0x39 unchanged].
        assert!(!sim.drones.channels.on(0x39), "ch 0x39 off before anything");
    }

    stand_at_station(&mut scene);

    // FIRST PRESS — releases hold #1.
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, true);
    // The latch cleared the hold and the script resumed: the ON art appears at t7 of the sub
    // (the press lands the script at t1 → 6 ticks later), and the lift sound 0x28a (stream 0 t2)
    // plays within the first ticks.
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sp(sim, SCRIPT_22).info[sp(sim, SCRIPT_22).cur].as_ref().unwrap().delay, 0,
            "SP_Activate's +0x9C = 0 cleared the hold on the press tick [M 0x1dc338:83-85]");
        assert!(!sp(sim, SCRIPT_21).activated, "only the aimed SP (#22) latched");
    }
    let mut lit_at = None;
    for t in 1..=60u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if lit_at.is_none() && any_ent(sim, (0x0600_064F, 2), 0x0200_06b3) { lit_at = Some(t); }
    }
    let lit_at = lit_at.expect("the button's ON art (0x020006b3) appears within 60 ticks of the call — the indicator light [L dim → bright; D 0x0600064F t7]");
    assert!(lit_at <= 10, "the light lands at sub t7 (the press resumed the script at t1), got tick {lit_at}");

    // The cab rides down and the script's SECOND self-hold (stream 9 Wait(0x213) → t539)
    // parks it at the car-park deck with the outer doors open, waiting for the rider.
    let mut down_at = None;
    let mut open_at = None;
    let mut held2_at = None;
    for t in 1..=700u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if down_at.is_none() {
            if let Some(cab) = ent_near(sim, (SCRIPT_22, 6), 0x0200_0506, [-2.77, -375.8, 28.08]) {
                if (cab[3][1] + 375.84).abs() < 0.1 { down_at = Some(t); }
            }
        }
        if open_at.is_none() {
            if let Some(door) = ent_near(sim, (SCRIPT_22, 7), 0x0200_06a8, [-2.22, -375.92, 26.32]) {
                if door[3][0] > -1.9 { open_at = Some(t); } // the key slides x −2.22 → −1.32 [D t531]
            }
        }
        if held2_at.is_none() && down_at.is_some() && open_at.is_some() {
            if sp(sim, SCRIPT_22).info[0].as_ref().unwrap().delay < 0 { held2_at = Some(t); break; }
        }
    }
    let down_at = down_at.expect("the cab reached the car-park deck y −375.84 [D stream 6 t338]");
    let open_at = open_at.expect("the car-park outer door slid open (x −2.22 → −1.32) [D stream 7 t531]");
    let held2_at = held2_at.expect("the t539 hold parks the cab at the open car-park doors [D stream 9]");
    assert!(open_at > down_at, "the doors open after the arrival [D key order]");
    let _ = held2_at;

    // SECOND PRESS — releases hold #2: the doors close (t652) and the cab carries back up
    // to the lobby deck (t869). The station press works while the cab is down (Bond stays
    // at the station, the pick is his own).
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, true);
    let mut back_at = None;
    for t in 1..=600u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(cab) = ent_near(sim, (SCRIPT_22, 6), 0x0200_0506, [-2.77, -368.2, 28.08]) {
            if (cab[3][1] + 368.19).abs() < 0.1 { back_at = Some(t); break; }
        }
    }
    let back_at = back_at.expect("the cab returned to the lobby deck [D stream 6 t869]");
    let _ = back_at;
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x39), "ch 0x39 still off — the call writes no channel (out 0) [M 0x1dbda8:221-229]");
        assert!(!sim.drones.channels.on(0x61) , "no alarm [L]");
    }
}

/// Sister: with no use press the script stays held at t1 — no light, no ride, doors shut.
#[test]
fn without_the_call_the_lift_stays_parked() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    stand_at_station(&mut scene);
    for _ in 0..1200 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sp(sim, SCRIPT_22).info[0].as_ref().unwrap().delay < 0,
        "the script is still held at t1 after 1200 ticks (no call, no release) [M 0x1d94c0:98-100]");
    assert!(!any_ent(sim, (0x0600_064F, 2), 0x0200_06b3), "no ON art — the indicator never lit [D]");
    let cab = ent_near(sim, (SCRIPT_22, 6), 0x0200_0506, [-2.77, -368.19, 28.08]).expect("the cab");
    assert!((cab[3][1] + 368.19).abs() < 0.05, "the cab stayed parked at the lobby deck [D t0 keys; L probe]");
    let door = ent_near(sim, (SCRIPT_22, 7), 0x0200_06a8, [-2.22, -375.92, 26.32]).expect("the car-park door");
    assert!((door[3][0] + 2.22).abs() < 0.02, "the outer door never moved (x −2.22) [D t0 key]");
    // The lift writes no channel (out 0). Ch 0x61 is not the lift's: idling 1200 ticks beside the
    // security-room panel, a guard raises the alarm — the game does that too (774: fresh-boot PCSX2,
    // Bond at the panel, a guard raised ch 0x61 within seconds [L 2026-10-17]; 774b's rule).
    assert!(!sim.drones.channels.on(0x39), "no channels moved [M out 0]");
}
