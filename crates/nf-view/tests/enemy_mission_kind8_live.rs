//! enemy-mission-kind8: the kind-8 mission walks of `NDrone2_DSTATE_EnemyMission` 0x15e1e8
//! (`NDrone2_StateFuncs[0x1b]` [L 0x29b300 + symtab]).
//!
//! The ENEMY side of the prop-0x1D mission paths: Idle/Alert ENTER assigns the kind-8 AI path
//! (`AssignAIPath(8)` 0x154190) and — with pending route nodes (+0x9f2 != 0) — SetStates 0x1b
//! [M 0x15bf18:32-56, 0x15c248:21-29]; the state walks the route (`NDrone2_UpdateMissionRoute`
//! 0x154a80), keeps the ENTER walk while the route moves, aims when the route is blocked at
//! alert ≥ 0.66 (the +0x3b fire-request + `SetAngleToObj` + anim 10) or stands idle hurt, and on
//! arrival (code 3) clears prop 0x1D, invalidates the route and falls back to IDLE — or, with an
//! alt state (+0x5A4) pending and no second switch channel (+0x135) outside level 0x7000006, to
//! `NDrone2_ChangeToAttackMode` 0x14a648's return [M 0x15e1e8:42-82].
//!
//! The disc users [D the kind-8 census, 2026-10-23: per level, the drone placements (class 0x0f)
//! whose behaviour word carries prop 0x1d, with a kind-8 path node within the assign's 0.3 2D
//! bind, whose type maps through `GetDTYPENEW` 0x14c938 to the start-by-mode rows (4 Idle / 6
//! InitPatrol — no civilian init)]: 07000001 dtype-2 "Mayhew Route" node 31 (0.129),
//! 07000006 dtype-1 "SnowTalkDronePath - (Dr_13)" (0.013), 0700000d dtype-1 "Mission_G6" node 2
//! (0.254), 07000012 dtype-3 "MissionPath_Drones enter Office 7" ×2 (0.027/0.069), 07000014
//! dtype-8 "MissionPath" ×3 (0.023-0.043), 07000016 dtype-2 "Kiko Walk" (0.152). The party
//! guests (07000007 dtype-0xa/0xb) and the civilian walkers (07000011/12 dtype-0xe) route through
//! CivilianInit/PartyGirlInit → CivilianMission 0x16 instead [M 0x172730/0x1728c0]; the escort
//! lead (07000001 dtype-0xc) starts 0x1c AllyLeadInit [L DroneTypeSettings row 12].
//!
//! Level 07000006 carries the only census bind whose guard spawns at level start: the dtype-1
//! guard placed 0.013 from "SnowTalkDronePath - (Dr_13)" node 0 (68.88, 38.54, 27.33) [D]. Its
//! start script plays first (the probe's state arc 0x3 → 0x1b), then Idle ENTER lands 0x1b and
//! the guard walks the 10-node path. Sister: 07000005 places 14 prop-0x1d guards but the level's
//! nav carries NO kind-8 path at all, so no guard can ever enter the state.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn dist(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The mission walker on 07000006: the live drone placed at "SnowTalkDronePath - (Dr_13)" node 0
/// ([D the census bind: placement (68.89, 39.29, 27.33), node (68.88, 38.54, 27.33)]).
fn find_walker(sim: &nf_view::scene::Sim) -> Option<usize> {
    let node = [68.88_f32, 38.54, 27.33];
    sim.drones.list.iter().position(|dr| {
        dr.present && dist(&dr.pos, &node) < 0.4
    })
}

/// The real path on 07000006: the SnowTalk guard's start script plays, Idle ENTER assigns the
/// kind-8 path and enters 0x1b, the guard follows the mission path (the position advances, the
/// route index climbs the 10-node chain [D the path's node count]), and the arrival arm fires:
/// state 0x1b → IDLE with the route dropped [M 0x15e1e8:63-82].
#[test]
fn the_snow_talk_guard_follows_its_kind8_mission_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let walker = find_walker(scene.sim.as_ref().unwrap()).expect("the SnowTalk guard spawns at level start");

    // Idle → (the script) → EnemyMission → the walk → IDLE. The probe measured the arc:
    // 0x1b at t≈498 (the script ran first), the walk ≈ 16 s, arrival ≈ t1483. 40 s of headroom.
    let mut entered_frame: Option<u32> = None;
    let mut route_len = 0usize;
    let mut path_kind = 0u32;
    let mut start_pos = [0.0_f32; 3];
    let mut max_progress = 0.0_f32;
    let mut saw_arrival = false;
    let mut arrival_route: Option<bool> = None;
    for t in 0..2400u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[walker];
        if entered_frame.is_some() {
            if dr.state == state::ENEMY_MISSION {
                let prog = dist(&dr.pos, &start_pos);
                max_progress = max_progress.max(prog);
            } else if dr.state == state::IDLE {
                // [M 0x15e1e8:63-82] the arrival: the route slot invalidated, the state IDLE.
                saw_arrival = true;
                arrival_route = Some(dr.route.is_some());
                break;
            }
        } else if dr.state == state::ENEMY_MISSION {
            entered_frame = Some(sim.drones.frame);
            start_pos = dr.pos;
            let r = dr.route.as_ref().expect("EnemyMission ENTER carries the assigned kind-8 route");
            route_len = r.nodes.len();
            path_kind = sim.nav.paths[r.path].kind;
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let dr = &sim.drones.list[walker];
    assert_eq!(dr.skin, 0x0500_002e, "the census guard [D placement skin]");
    assert!(entered_frame.is_some(), "the guard enters 0x1b through the Idle/Alert ENTER arm");
    assert_eq!(path_kind & 8, 8, "the route is a kind-8 mission path [D the census bind]");
    assert_eq!(route_len, 10, "SnowTalkDronePath - (Dr_13) is a 10-node path [D the census]");
    assert!(max_progress > 10.0, "the guard walks its mission path (walked {:.1} u)", max_progress);
    assert!(saw_arrival, "the guard arrives and takes the code-3 arm [M 0x15e1e8:63]");
    assert_eq!(arrival_route, Some(false), "the arrival invalidates the route (+0x9f2 = 0) [M 0x15e1e8:64-65]");
    assert_eq!(dr.state, state::IDLE, "the fallback state is 4 IDLE [M 0x15e1e8:81]");
}

/// Sister: 07000005 places 14 prop-0x1d guards but its nav carries NO kind-8 path, so
/// `AssignAIPath(8)` cannot bind and the Idle/Alert ENTER never reaches 0x1b — the prop clears
/// [M 0x15bf18:37-39] and no guard of the level enters the state.
#[test]
fn a_level_without_kind8_paths_never_enters_enemy_mission() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.nav.paths.iter().any(|p| p.kind & 8 == 0), "the level has paths");
        assert!(sim.nav.paths.iter().all(|p| p.kind & 8 == 0),
            "07000005's nav carries no kind-8 path [D the kind-8 census, 2026-10-23]");
    }
    // 30 s: no drone of the level ever enters 0x1b.
    for _ in 0..1800u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for dr in &sim.drones.list {
            assert_ne!(dr.state, state::ENEMY_MISSION,
                "a guard without a kind-8 path entered EnemyMission at ({:.2},{:.2},{:.2})",
                dr.pos[0], dr.pos[1], dr.pos[2]);
        }
    }
    // And the Idle/Alert population is alive (the state the prop-0x1d guards pass through):
    // at least one guard sits in IDLE or ALERT during the run — the arm ran for them.
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.list.iter().any(|dr| dr.present
        && matches!(dr.state, state::IDLE | state::ALERT)),
        "the level keeps unaware guards in Idle/Alert (the arm's population)");
}
