//! start-corridor-guard-11: the 07000011 start-corridor guards (level-11-start-corridor-guard).
//!
//! Symptom: on a fresh 07000011 load with Bond idle at the start, the port walks a red-suited
//! guard into the corridor south of the spawn from ~tick 200 and he kills Bond; the PS2 capture
//! shows the same corridor empty over f200-f3172 [L].
//!
//! What this test pins (every assertion cites its evidence):
//!  - The two switch-less mode-7 office guards exist at load — placements #126 (skin 0x05000038,
//!    disc p2 = 1) and #127 (skin 0x0500003A, p2 = 2) [D 07000011.bin map 43]; Drone_Create's
//!    difficulty gate creates a drone iff `placement param2 <= GameState+0x40` [M 0x136be0:49],
//!    so both exist at difficulty 3 (00 Agent). THE SISTER: they are present in drones.list.
//!  - Their wake chain is NOT the divergence: param 3 = 0 (no switch channel) [D] and param 1 =
//!    0x06000000, which NDrone2_DefaultInit zeroes at Drone+0x554 [M 0x14b300:611-613], so
//!    DSTATE_Global's ENTER sends them to +0x5A2 — mode 7's start state 0x85 EnemyRunToPoint
//!    [M 0x15b830:22-43; M 0x14cbd8 (DroneTypeSettings row 7 start = 0x85 [D ACTION.ELF])]
//!    — the game activates them at load exactly as the port does.
//!  - THE OWNER RULE (this task's fix): ClosestAIPoint prefers the point whose +0x0c owner equals
//!    the drone's +0x144, and DefaultInit writes +0x144 = placement PARAM 4 (piVar19[0x11] =
//!    params[4]; params start at piVar19[13] = DIVars+0x34, the level_tag's flat param array)
//!    [M 0x14b300:472; M 0x151b18:151-181]. The disc pairs the guards to their run-to points:
//!    #126's p4 = 31 ↔ the group-0 AIPoint with owner 31 at (1.40, 46.8, 16.70); #127's p4 = 30 ↔
//!    the group-0 AIPoint with owner 30 at (−1.51, 46.8, 18.21) [D]. The port used to read
//!    property BIT 0x11 of the props words (set by the alert block on these guards) — the wrong
//!    field — which matched the owner-1 point at (−1.51, 0.40) instead.
//!  - The Tower-mission run-to-point override [M 0x168910:24-31]: on 0x07000011/12/13/41 a fresh
//!    EnemyRunToPoint drone (alertness < 0.66) resumes to IDLE, and with no eligible run-to point
//!    EnemyRunToPoint's ENTER parks it there [M 0x168910:32-37].
//!
//! What is NOT pinned here (see docs/reports/level-11-start-corridor-guard.md "What's left"):
//! with the owner fix the port's guards still approach an idle Bond; the remaining divergence
//! sits between the game's ClosestAIPoint/emit inputs and the port's (the emit's ±2 start-box /
//! NearestNodeDCV semantics / the GoToGoalPosition look gate +0x228&4), and needs a live check.

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
    let path1 = 1usize; // both guards and every office AIPoint sit on nav path 1 [probe; the office path]
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
        let _ = path1;
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
