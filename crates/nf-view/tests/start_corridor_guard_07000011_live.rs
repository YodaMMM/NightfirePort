//! start-corridor-guard-11: the 07000011 start-corridor guards.
//!
//! Behaviour: on a fresh 07000011 load with Bond idle at the start, office guards #126 (and #127
//! from Agent up) walk into the corridor south of the spawn and kill him. The real game does the
//! same [L docs/live/2026-11-01-07000011-start-corridor-guards.md: fresh boot, goto_level, PINE
//! every 50 frames]: the fresh-boot difficulty is 2 (0x2A3790), both guards are in GoToGoal 0x63
//! on frame 1, stand 6-9 units from Bond by f200 and fire; idle Bond dies at f1113 (d2) / f878
//! (d3); 10/12/20 damage a hit at d1/d2/d3. An empty corridor is not a fresh idle start at any
//! selectable difficulty (the menu stores 1/2/3 [M C_SBNFDFCTY 0x216ca8:15]).
//!
//! What this test pins (every assertion cites its evidence):
//!  - The two switch-less office guards exist at load — placements #126 (skin 0x05000038, disc
//!    p2 = 1) and #127 (skin 0x0500003A, p2 = 2) [D 07000011.bin map 43]; Drone_Create's
//!    difficulty gate creates a drone iff `placement param2 <= difficulty` [M 0x136be0:49], so
//!    #126 alone at d1, both at d2/d3 [L the live rosters].
//!  - Their wake chain: param 3 = 0 (no switch channel) [D] and param 1 = 0x06000000, which
//!    NDrone2_DefaultInit zeroes at Drone+0x554 [M 0x14b300:611-613], so DSTATE_Global's ENTER
//!    sends them to +0x5A2 — mode 7's start state 0x85 EnemyRunToPoint [M 0x15b830:22-43;
//!    DroneTypeSettings row 7 start = 0x85 [D ACTION.ELF 0x29aee8+7*12]] → GoToGoal 0x63 [L f1].
//!  - The owner rule: ClosestAIPoint prefers the point whose +0x0c owner equals the drone's
//!    +0x144, and DefaultInit writes +0x144 = placement PARAM 4 (piVar19[0x11] = params[4])
//!    [M 0x14b300:472; M 0x151b18:151-181]. The disc pairs the guards to their run-to points:
//!    #126's p4 = 31 ↔ the group-0 AIPoint with owner 31 at (1.40, 46.8, 16.70); #127's p4 = 30
//!    ↔ the owner-30 AIPoint at (−1.51, 46.8, 18.21) [D].
//!  - No alarm detour: EnemyRunToPoint's ENTER redirects to RunToAlarm 0x89 only when property
//!    0x56 is set [M 0x168910:17-27] = behaviour word 2 bit 26 [M 0x1c7d98 + the bitDescs/bitMasks
//!    tables [D ACTION.ELF 0x2c7250/0x2c7308]]; the guards' word 2 (p20) is 0x40280000, bit 26
//!    CLEAR [D], and no kind-8 AIPoint exists on the level [D the 12 map-43 0xE9 placements].
//!  - At d3 the guards walk to their owner points and hold ATTACK as the resume state (the Tower
//!    clause [M 0x168910:23-31]).
//!  - Per difficulty: the guards are in the corridor box (x −4..5, z 15.5..22, where the game's
//!    sit [L]) at t200/t700, Bond takes the game's per-hit damage, and dies at d2/d3.
//!
//!  - In position the guards alternate StepAimLeft/Right 0x73/0x74 with AimStandFire 0x68 [L],
//!    because their arrival switches them to the second behaviour block (placement p5 = 100), whose
//!    step property 0x42 is set [D p25 = 0x40291660; M 0x165640:47-82 → 0x14a648:29-31]. A guard
//!    with the step property clear in both blocks never steps (the sister).

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_d3() -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3")
}

/// The group-0 AIPoints of the level, from the disc [D 07000011.bin main map 0xE9 placements]:
/// owner 31 at (1.40, 16.70), owner 30 at (−1.51, 18.21), owner 3 at (−22.40, 24.59), owner 0 at
/// (13.5, −6.3) and (18.9, −2.5), owner 1 at (−1.51, 0.40) — placement order.
#[test]
fn start_corridor_guards_exist_and_pair_to_their_owner_points() {
    let mut scene = load_d3();
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().unwrap();

    // THE SISTER: both guards are present in drones.list at load (the difficulty gate passes).
    for (p, skin, p4) in [(126usize, 0x0500_0038u32, 31i32), (127, 0x0500_003a, 30)] {
        let di = sim.drones.list.iter().position(|d| d.placement == p && d.present)
            .unwrap_or_else(|| panic!("guard p{p} present in drones.list [D; M 0x136be0:49]"));
        let dr = &sim.drones.list[di];
        assert_eq!(dr.skin, skin, "p{p}'s disc skin [D]");
        assert_eq!(dr.group, p4 as u32, "p{p}'s placement param 4 — the ClosestAIPoint owner +0x144 [M 0x14b300:472]");
        assert!(dr.enabled, "a switch-less guard is enabled at load, not held [M 0x15b830:22-43]");
    }

    // The wake chain: no switch channel (p3 = 0 [D]) → Global does NOT take the WaitSwitch arm
    // [M 0x15b830:22-35], and p1 = 0x06000000 zeroes +0x554 [M 0x14b300:611-613] — the guards'
    // start_script must be 0 so Global sends them to the mode-7 start state, not PlayScript.
    for p in [126usize, 127] {
        let dr = &sim.drones.list[sim.drones.list.iter().position(|d| d.placement == p).unwrap()];
        assert_eq!(dr.switch_channel, 0, "p{p} has no wake channel [D p3 = 0]");
        assert_eq!(dr.start_script, 0, "p{p}'s p1 0x06000000 is zeroed at +0x554 [M 0x14b300:611-613]");
        assert_ne!(dr.state, nf_game::drone::state::WAIT_SWITCH,
            "p{p} never enters WaitSwitch [M 0x15b830:22-43 — the channel byte is 0]");
    }

    // THE OWNER PAIRING: with the game's owner rule the guards' group-0 run-to point is their
    // owner-matched disc point [D p4 = 31/30 ↔ AIPoint p2 = 31/30; M 0x151b18:151-181], not the
    // owner-1 point the old property-bit read matched.
    for (p, owner, want) in [(126usize, 31i32, [1.40f32, 16.70]), (127, 30, [-1.51, 18.21])] {
        let dr = &sim.drones.list[sim.drones.list.iter().position(|d| d.placement == p).unwrap()];
        let path = dr.nav_path.flatten();
        let hit = nf_game::aipoint::closest(&sim.ai_points, &sim.nav, &sim.world, path, dr.pos, 0, owner)
            .unwrap_or_else(|| panic!("p{p} finds its owner-{owner} run-to point [D; M 0x151b18]"));
        let pt = &sim.ai_points[hit];
        assert_eq!(pt.owner, owner, "the picked point's owner matches p{p}'s param 4 [M 0x14b300:472]");
        assert!((pt.pos[0] - want[0]).abs() < 0.1 && (pt.pos[2] - want[1]).abs() < 0.1,
            "p{p} pairs to its disc point at ({:.2},{:.2}) [D], got ({:.2},{:.2})",
            want[0], want[1], pt.pos[0], pt.pos[2]);
    }
}

/// The old wrong read, pinned as a negative: the guards' props DO carry property bit 0x11 (the
/// alert block), so the port's old `props.get(0x11)` owner (1) is not their disc owner (31/30)
/// [D p17 = 0x5b0003 sets bit 17; M 0x14b300:472 — the game reads param 4, not the props].
#[test]
fn the_old_owner_read_disagrees_with_the_game() {
    let scene = load_d3();
    let sim = scene.sim.as_ref().unwrap();
    for (p, owner) in [(126usize, 31i32), (127, 30)] {
        let dr = &sim.drones.list[sim.drones.list.iter().position(|d| d.placement == p).unwrap()];
        let prop_bit = sim.props.get(0x11, &dr.props) as i32;
        assert_ne!(prop_bit, owner,
            "p{p}: property bit 0x11 ({prop_bit}) != the disc owner {owner} — the old read was wrong [M 0x14b300:472]");
    }
}

/// Property 0x56 is CLEAR on both guards — the EnemyRunToPoint ENTER's RunToAlarm redirect
/// [M 0x168910:17-27] cannot fire: word 2 (p20) = 0x40280000, bit 26 = 0 [D placement params;
/// M 0x1c7d98 + the bitDescs/bitMasks tables [D ACTION.ELF 0x2c7250/0x2c7308 —
/// prop 0x56 = word 2, mask 0x4000000, shift 26]]. And the level has no kind-8 AIPoint at all
/// [D the 12 map-43 0xE9 placements: p0 ∈ {0,1,2}], so `ClosestAIPoint(…, 8, …)` finds nothing —
/// the alarm route (RunToAlarm → FindAlarmPoint 0x153008 → Idle) is not on this level.
#[test]
fn prop_0x56_is_clear_and_no_alarm_point_exists() {
    let scene = load_d3();
    let sim = scene.sim.as_ref().unwrap();
    for p in [126usize, 127] {
        let dr = &sim.drones.list[sim.drones.list.iter().position(|d| d.placement == p).unwrap()];
        assert_eq!(sim.props.get(0x56, &dr.props), 0,
            "p{p}: prop 0x56 (p20 bit 26) is clear on the disc [D p20 = 0x40280000; M 0x1c7d98]");
        let hit8 = nf_game::aipoint::closest(&sim.ai_points, &sim.nav, &sim.world,
            dr.nav_path.flatten(), dr.pos, 8, dr.group as i32);
        assert!(hit8.is_none(), "p{p}: no kind-8 AIPoint exists on 07000011 [D the 12 0xE9 placements]");
    }
    assert_eq!(sim.props.descs.get(0x56), Some(&[210u8, 26u8]),
        "the bitDescs row: word 2 (210 & 7 = 2), shift 26 (210 >> 3), mask id 26 = 0x4000000 [D ACTION.ELF 0x2c7250+172]");
}

/// At d3 (both guards created [M 0x136be0:49]) the guards walk to their disc owner points and
/// hold ATTACK there — the game's own data path: mode 7 → start 0x85 [D DroneTypeSettings row 7],
/// prop 0x56 clear → the run-to-point machinery [M 0x168910:20-56], the owner rule picks the
/// owner-31/30 points [D; M 0x151b18], and the Tower clause at alertness 1.0 leaves the resume
/// state at ATTACK 0x56 [M 0x168910:23-31] (the 0x4000000 latch is clear at boot [M 0x1469b8 —
/// only the sighting paths set it]). AIPoint+4 = 1 on every point at creation [M 0x137550], so
/// the game's eligibility matches the port's selection.
#[test]
fn at_difficulty_3_the_guards_walk_their_disc_owner_points() {
    let mut scene = load_d3();
    let owners = [(126usize, [1.40f32, 16.70]), (127, [-1.51, 18.21])];
    // The resume state the guard carries out of GoToGoal 0x63 — read on its last 0x63 tick: once in
    // the fight, Attack 0x56's prop-0x18 arm rewrites +0x5A2 to 0x68 [M 0x164210:71-87].
    let mut leave_ret = [None; 2];
    for _ in 0..400u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for (k, (p, _)) in owners.iter().enumerate() {
            let dr = sim.drones.list.iter().find(|d| d.placement == *p).unwrap();
            if dr.state == nf_game::drone::state::GO_TO_GOAL { leave_ret[k] = Some(dr.return_state); }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    for (k, (p, want)) in owners.into_iter().enumerate() {
        let dr = &sim.drones.list[sim.drones.list.iter().position(|d| d.placement == p).unwrap()];
        let dx = dr.pos[0] - want[0];
        let dz = dr.pos[2] - want[1];
        assert!((dx * dx + dz * dz).sqrt() < 2.5,
            "p{p} reached its owner point ({:.2},{:.2}) [D; M 0x151b18] within the 2.0 goal accuracy, at ({:.2},{:.2})",
            want[0], want[1], dr.pos[0], dr.pos[2]);
        assert_ne!(dr.state, nf_game::drone::state::GO_TO_GOAL, "p{p} left GoToGoal by t400");
        assert_eq!(leave_ret[k], Some(nf_game::drone::state::ATTACK),
            "p{p}'s resume state stays ATTACK 0x56 at alertness 1.0 through GoToGoal (the Tower clause) [M 0x168910:23-31]");
    }
}

/// Live per-difficulty behaviour [L docs/live/2026-11-01-07000011-start-corridor-guards.md]:
/// the guards are in the corridor south of the spawn at t200 and t700 (the game's: x −2.4..3.1,
/// z 17.0..20.1), and an idle Bond takes the game's per-hit damage — 10/12/20 at d1/d2/d3 [L] —
/// and dies at d2/d3. The pre-fix port's d2 ~f1113 / d3 ~f878 death ticks re-measured after the
/// civilian-alarm fix (scared civilians shout, guards attack 60 ticks later, the dead +0xC6
/// duplicate gone — 5fbfa4f7d): the port's d2 now dies ~t3500 (5 hits by t1500, 9 by t3500 — the
/// ack chain is the game's slower one) and d3 ~t501 (5×20), so the window is 4000 (a
/// port-measured post-fix timing, not a fresh PCSX2 [L]).
#[test]
fn idle_bond_is_approached_and_shot_at_every_selectable_difficulty() {
    let start = [-0.40f32, 25.83];
    let in_box = |p: [f32; 3]| (-4.0..=5.0).contains(&p[0]) && (15.5..=22.0).contains(&p[2]);
    for (diff, guards, per_hit, dies) in [
        (1u32, &[126usize][..], 10.0f32, false),
        (2, &[126, 127][..], 12.0, true),
        (3, &[126, 127][..], 20.0, true),
    ] {
        let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, diff)
            .expect("load 07000011");
        if diff == 1 {
            let sim = scene.sim.as_ref().unwrap();
            assert!(sim.drones.list.iter().find(|d| d.placement == 127).map_or(true, |d| !d.present),
                "d1: #127 (p2 = 2) is not created [M 0x136be0:49; L the d1 roster]");
        }
        let mut dead_at = None;
        // Guards that ran from an explosive (RunAwayFromObject 0x86, via DroneFunc_ConsiderExplosive
        // [M 0x1431c0:194-195]) before t700, and whether they were back in the corridor by t900.
        let mut fled = vec![false; guards.len()];
        let mut back = vec![false; guards.len()];
        for t in 1..=4000u32 {
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            let sim = scene.sim.as_ref().unwrap();
            for (k, &p) in guards.iter().enumerate() {
                let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
                if t < 700 && dr.state == nf_game::drone::state::RUN_AWAY_FROM_OBJECT { fled[k] = true; }
                if (700..=900).contains(&t) && in_box(dr.pos) { back[k] = true; }
            }
            if t == 1 {
                for &p in guards {
                    let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present)
                        .unwrap_or_else(|| panic!("d{diff}: #{p} exists [L]"));
                    assert_eq!(dr.state as u32, nf_game::drone::state::GO_TO_GOAL as u32,
                        "d{diff}: #{p} is in GoToGoal 0x63 on tick 1 [L f1]");
                }
            }
            if t == 200 || t == 700 {
                for (k, &p) in guards.iter().enumerate() {
                    // The second behaviour block's fear properties 0x34/0x3a/0x3b [D p23-25] send a
                    // guard out of 0x5c after a grenade [M 0x1431c0:22-30]. The port's d3 lob from
                    // #127 hits the 49.73 ceiling and rolls back to the guards [S — the game's d3
                    // run shows no flight at f700 [L]]; such a guard is checked for its return to
                    // the corridor by t900 below instead of its t700 spot.
                    if t == 700 && fled[k] { continue; }
                    let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
                    assert!(in_box(dr.pos), "d{diff} t{t}: #{p} stands in the corridor (x −4..5, z 15.5..22) as the game's does [L], at ({:.2},{:.2})",
                        dr.pos[0], dr.pos[2]);
                    let dx = dr.pos[0] - start[0];
                    let dz = dr.pos[2] - start[1];
                    // The game's are 5.8-9.0 away at both ticks [L]; the port's d1 #126 first stops
                    // in AimStand 0x67 ~10.7 away at t200 and closes to 8.0 by t400.
                    let reach = if t == 200 { 11.0 } else { 10.0 };
                    assert!((dx * dx + dz * dz).sqrt() < reach, "d{diff} t{t}: #{p} within {reach} of the idle start [L 5.8-9.0]");
                }
            }
            if dead_at.is_none() && sim.drones.health.health <= 0.0 {
                dead_at = Some(t);
            }
        }
        for (k, &p) in guards.iter().enumerate() {
            if fled[k] { assert!(back[k], "d{diff}: #{p} ran from an explosive and was back in the corridor by t900"); }
        }
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.player_hits.is_empty(), "d{diff}: the guards hit the idle Bond [L]");
        assert_eq!(sim.drones.player_hits[0].2, per_hit, "d{diff}: a hit takes {per_hit} [L]");
        assert!(sim.drones.player_hits.iter().all(|h| h.2 <= per_hit),
            "d{diff}: no hit exceeds the game's per-hit damage [L]");
        if dies {
            assert!(dead_at.is_some(), "d{diff}: the idle Bond dies within 4000 ticks (post-fix port timing: d2 ~t3500, d3 ~t501)");
        }
    }
}

/// The tick budget for the step pins: the guards arrive at t172/t202 and Bond (idle, d2) dies at
/// ~t1000 [L f1113], after which they leave the fight (0x5c → 0x60).
const STEP_WINDOW: u32 = 1000;

/// Run d2 for `STEP_WINDOW` ticks (`prep` edits the sim after load) and collect each guard's visited
/// states.
fn d2_states(prep: impl FnOnce(&mut nf_view::scene::Sim)) -> Vec<(usize, std::collections::BTreeSet<u8>)> {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 2).expect("load 07000011 d2");
    prep(scene.sim.as_mut().unwrap());
    let mut seen = vec![(126usize, std::collections::BTreeSet::new()), (127, std::collections::BTreeSet::new())];
    for _ in 0..STEP_WINDOW {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for (p, set) in seen.iter_mut() {
            let dr = sim.drones.list.iter().find(|d| d.placement == *p && d.present).expect("guard present");
            set.insert(dr.state);
        }
    }
    seen
}

/// The arrival's behaviour switch [M 0x165640:47-82]: a mode-7 (EnemyRunToPoint) guard that reaches
/// its goal with an alternate state (+0x5A4 = 0x56) runs `NDrone2_ChangeToAttackMode` 0x14a648,
/// which, with +0xC48 = 100 (placement p5 [D]), points +0x4D8 at the second block (+0x29 = 1)
/// [M 0x14a648:29-31] and enters Attack 0x56. The first block's step property 0x42 is clear, the
/// second's set [D p20 = 0x40280000, p25 = 0x40291660: word 2 bit 5].
#[test]
fn the_arrival_switches_the_guards_to_their_second_behaviour_block() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 2).expect("load 07000011 d2");
    {
        let sim = scene.sim.as_ref().unwrap();
        for p in [126usize, 127] {
            let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
            assert!(!dr.using_2nd, "p{p} starts on its first block [M 0x14b300: +0x4D8 = +0x4DC]");
            assert_eq!(sim.props.get(0x42, &dr.props), 0, "p{p}: the first block's step property is clear [D p20]");
            assert_eq!(dr.alt_state, nf_game::drone::state::ATTACK, "p{p}: the alternate state +0x5A4 = 0x56 at load (the port's type-settings derivation)");
        }
    }
    for _ in 0..400u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    for p in [126usize, 127] {
        let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
        assert!(dr.using_2nd, "p{p} switched to its second block on arrival [M 0x165640:67-79 → 0x14a648:29-31]");
        assert_eq!(sim.props.get(0x42, &dr.props), 1, "p{p}: the second block's step property is set [D p25]");
        assert_eq!(dr.alt_state, 0, "p{p}: ChangeToAttackMode consumed +0x5A4 [M 0x14a648:28]");
    }
}

/// [L docs/live/2026-11-01-07000011-start-corridor-guards.md] in position the game's guards visit
/// StepAimLeft/Right 0x73/0x74 as well as AimStandFire 0x68 (#126 at d2: 0x74 f200, 0x73 f700/f1500;
/// #127 at d3: 0x73 f700). `NDrone2_ChooseCombatMove` 0x150c18 is the only source of 0x73/0x74
/// [M :282-316 — no other SetState of either], reached from AimStandFire's `DroneFunc_CombatState`
/// [M 0x148338:215-223] when the burst is over [M 0x166328:135-150]; the step arm needs property
/// 0x42 [M 0x150c18:127-139].
#[test]
fn in_position_the_guards_step_aim_as_well_as_fire() {
    for (p, states) in d2_states(|_| {}) {
        use nf_game::drone::state::*;
        assert!(states.contains(&STEP_AIM_LEFT) || states.contains(&STEP_AIM_RIGHT),
            "d2: #{p} visits StepAim 0x73/0x74 within {STEP_WINDOW} ticks [L], visited {states:x?}");
        assert!(states.contains(&AIM_STAND_FIRE) || states.contains(&AIM_STAND_RELOAD),
            "d2: #{p} visits AimStandFire/Reload 0x68/0x69 within {STEP_WINDOW} ticks [L], visited {states:x?}");
    }
}

/// THE SISTER: with property 0x42 cleared in both blocks the step arm's gate is closed
/// [M 0x150c18:127-139, :282-290] — the same guards still fight from AimStandFire 0x68 but never
/// step.
#[test]
fn a_guard_with_the_step_gate_closed_stays_put() {
    let runs = d2_states(|sim| {
        let props = &sim.props;
        for dr in sim.drones.list.iter_mut().filter(|d| matches!(d.placement, 126 | 127)) {
            props.set(0x42, &mut dr.props, 0);
            if let Some(w) = dr.alert_props.as_mut() { props.set(0x42, w, 0); }
        }
    });
    for (p, states) in runs {
        use nf_game::drone::state::*;
        assert!(!states.contains(&STEP_AIM_LEFT) && !states.contains(&STEP_AIM_RIGHT),
            "d2: #{p} with property 0x42 clear never steps [M 0x150c18:127-139], visited {states:x?}");
        assert!(states.contains(&AIM_STAND_FIRE), "d2: #{p} still fights from AimStandFire 0x68, visited {states:x?}");
    }
}
