//! guards-shut-door (07000001): do alerted guards that heard Bond's shots open a SHUT door to
//! reach him? The game's answer, from the decompile and the level's own nav data:
//!
//! The door-opening machinery is unconditional ON A ROUTE: every followed route runs
//! `NDrone2_ReachedDestNode` 0x153358 (from `LinkCreep_Handler` 0x15a2c8:58-65), whose
//! door-node case binds the door and enters DSTATE_OpenDoor 0x81 with no seen/heard gate
//! [M 0x153358:144-174]; `NDrone2_OpenDoor` 0x14aa38:8-13 calls `Door_Activate(door, drone, 0)`;
//! `LinkCreep_Handler` 0x15a2c8:66-68 skips the blocked test while a door is bound in
//! Drone+0xBA8; `NDrone2_MoveToGoalPosition` 0x1517d8:61-63 returns 10 (door pending) and
//! AlertToPosition 0x165270:72-75 waits; `NDrone2_ControlSTANDARD` 0x1490f0:233-245 re-opens
//! every 60 ticks in any state.
//!
//! But the route itself is confined to ONE path record [M 0x1571b0 `AINetwork_DoAStarPath` —
//! `route+0xF4`'s own node/link arrays (+0x2C/+0x34/+0x3C/+0x40)], and the path records
//! eligible for node searches are kind & 0xD == 0 [M 0x1d2598 `AIPath_Parse`: in-memory
//! path+0x08 = the record's kind word (disc +0x80); the filter reads it at 0x1586a8
//! `AINetwork_NodesForPosition` and 0x158488 `AINetwork_NavPathForPosition`]. On 07000001 the
//! kind-8 record (port path 0) is the Mayhew mission route carrying the south door's only
//! flagged pair (n5 (7.1, 10.2) / n6 (7.0, 11.8) at the (7.92, 5.34, 11.14) shoji [D
//! placement; L the port nav census]) — the hall guards' NavPath (port path 1, 28 nodes)
//! carries only the west door #2 pair (n21/n22 at (−14.x, 14.1)). So the hall guards can never
//! acquire a route through the shut SOUTH door — in the game either — and the game's own combat
//! logic parks them at the standoff bands: `DroneFunc_CombatState` 0x148338:45-49 approaches to
//! `min(engage_range, 6.0)`, arrival 2.0, out-of-range → 0x5a, no route → 0x5f, whose update
//! rolls `NDrone2_ChooseCombatMove` 0x150c18 side-steps (step 1.8 [M 0x14f640:63], dodge 3.6,
//! roll 5.2).
//!
//! This test pins the south-door half on the real input path (load → intro → the meet walk →
//! the hold; no teleports): 40 s of pistol fire from the meet-room side and the door STAYS
//! shut, no guard crosses — the game's own law, from its own nav data. The pair-crossing
//! machinery itself (route → 0x81 → Door_Activate → cross) is pinned by the existing
//! npc_doors_live drives (the 01 escort through this very shoji, the 06 chase) — the hearing
//! chain ends in those same route states (Suspect 0xA4 → AlertToPosition 0x62 on prop 0x21, a
//! second loud shot → Attack 0x56 [M 0x16b0e0:120-126, 0x177800:62-78]).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The real-input path to the meet scene's tail: load → intro (ch 0xB) → the meet plate walk
/// (ch 0x32 → 0x33 → SP #276) → ch 0xE (the four hall guards enable). The walkreal route.
fn reach_meet_scene() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    w.prefer_head = true;
    w.sidestep_decays = true;
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { eprintln!("[sdoor] intro ch 0xB at {t}"); break; }
    }
    w.wait_enabled(6000, "the intro tail (event 6 re-places Bond)");
    w.wait(300);
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let plate = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x32)
            .map(|t| t.centre())
            .expect("the meet plate");
        drop(sim);
        w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.25 }, "onto the meet plate");
        for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: [plate[0] + dx, plate[1], plate[2] + dz], act: Act::None, arrive: 0.2 },
                &format!("the meet plate sweep ({dx},{dz})"));
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.2 }, "back across the meet plate");
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x32), "walking the meet plate must fire ch 0x32 [D placement]");
    let mut e_at = None;
    for t in 0..300u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    assert!(e_at.is_some(), "the meet scene must write ch 0xE (the hall guards' enable channel)");
    w.wait(120);
    w
}

fn hall_guards(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len())
        .filter(|&i| { let d = &sim.drones.list[i]; d.switch_channel == 0xe && !d.corpse && d.enabled })
        .collect()
}

fn door_near(w: &Walker, at: [f32; 3]) -> usize {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.doors.list.iter().enumerate()
        .min_by(|(_, a), (_, b)| walker::dxz(a.pos, at).total_cmp(&walker::dxz(b.pos, at)))
        .expect("a door near the spot").0
}

/// Fire `ticks` taps (every 16th tick) toward `at`, reload on dry; census every 60 ticks.
/// Returns (door_opened_at, crossed set at the end) — `crosses` decides a guard crossing the
/// plane (signed test on the guard's coordinate against `plane`).
fn fire_and_watch(w: &mut Walker, at: [f32; 3], ticks: u32, door_i: usize,
                  crosses: &dyn Fn(&nf_game::drone::Drone, f32) -> bool, plane: f32) -> (Option<u32>, Vec<usize>) {
    let mut door_opened_at = None;
    for t in 0..ticks {
        let (input, trigger, dry) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let aim = [at[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, at[2]];
            let dx = aim[0] - bp[0]; let dz = aim[2] - bp[2];
            let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            let dry = sim.gun.clip == 0;
            (Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, err.abs() < 0.03 && t % 16 == 0 && !dry, dry)
        };
        w.scene.sim.as_mut().unwrap().step_bond(input, trigger, dry, false);
        w.total_ticks += 1;
        w.after_step();
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.health.health <= 0.0 { eprintln!("[sdoor] Bond died at tick {t}"); break; }
        if sim.drones.doors.list[door_i].state != nf_game::door::CLOSED && door_opened_at.is_none() {
            door_opened_at = Some(t);
            eprintln!("[sdoor] DOOR {door_i} OPENED at fire-tick {t} state {:#x}", sim.drones.doors.list[door_i].state);
        }
        if t % 60 == 0 {
            let gs: Vec<String> = hall_guards(w).iter()
                .map(|&i| { let d = &sim.drones.list[i];
                    format!("#{} s{:#x} ({:.1},{:.1})", i, d.state, d.pos[0], d.pos[2]) })
                .collect();
            eprintln!("[sdoor] t{t} door{door_i} {:#x} | {}", sim.drones.doors.list[door_i].state, gs.join(" | "));
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let crossed: Vec<usize> = hall_guards(w).into_iter()
        .filter(|&i| crosses(&sim.drones.list[i], plane)).collect();
    (door_opened_at, crossed)
}

/// The south door: Bond holds south of the SHUT shoji and fires into the hall for 40 s.
/// The hall guards hold the standoff and the door STAYS SHUT — the game's own law, because the
/// only flagged pair for this door lives on the kind-8 Mayhew route (excluded from the A*,
/// [M 0x1d2598 + 0x1586a8]), never on the hall NavPath the guards route on [M 0x1571b0].
#[test]
fn south_shut_door_the_hall_guards_hold_the_standoff_like_the_game() {
    let mut w = reach_meet_scene();
    // The hold line south of the shut door — every leg stays z <= 10.0, and the door auto-opens
    // at ~1.1 u [L the stage-walk trace], so the door can only open by a guard's own door use.
    w.walk_nav_to([7.2, 6.4, 8.8], "the hold line south of the shut door", 1.0);
    let door_i = door_near(&w, [7.92, 5.34, 11.14]);
    let door_z = w.scene.sim.as_ref().unwrap().drones.doors.list[door_i].pos[2];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[door_i].state, nf_game::door::CLOSED, "the south door must still be shut at the hold line");
        let bz = sim.bond.pos[2];
        assert!(bz < door_z - 2.0, "Bond holds south of the plane (z {bz:.1} vs {door_z:.1})");
        let g = hall_guards(&w);
        assert_eq!(g.len(), 4, "four ch-0xE hall guards [D placements]");
        for &i in &g {
            let d = &sim.drones.list[i];
            eprintln!("[sdoor] guard {i} skin {:#x} state {:#x} at ({:.1},{:.1})", d.skin, d.state, d.pos[0], d.pos[2]);
        }
    }
    let (opened, crossed) = fire_and_watch(&mut w, [7.9, 6.4, 14.0], 2400, door_i,
        &|d: &nf_game::drone::Drone, plane: f32| d.pos[2] < plane - 0.3, door_z);
    eprintln!("[sdoor] south final: opened {opened:?} crossed {crossed:?}");
    assert!(opened.is_none() && crossed.is_empty(),
        "the game's law: the hall guards never acquire a route through the south door's pair (kind-8 Mayhew route only [M 0x1d2598]) — door opened {opened:?}, crossed {crossed:?}");
}
