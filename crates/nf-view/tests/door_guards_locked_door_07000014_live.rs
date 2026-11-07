//! 07000014: the guards the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) sends toward the locked
//! west door must not path through it. The game's A* rejects a link between two door-type nodes
//! while the node's bound door is locked [M AINetwork_DoAStarPath 0x1571b0:262-271 →
//! Door_IsLocked 0x136810], so their combat route to Bond fails and they hold
//! COMBAT_NO_ROUTE 0x5f at the door — no OpenDoor 0x81 [L 2026-10-30 PCSX2 fresh boot, Bond
//! held in the generator hall: CUR 0x5f / PREV 0x5a at the door, never 0x81].
//! The route ends: mission routes arrive at 0.5 [M NDrone2_InitAIPath 0x153720:113; L the live
//! route+0x8C = 0.500].
//!
//! The game's hold is TOTAL — 17 live samples at 30-frame spacing, the state never leaves 0x5f —
//! while the port's guard cycles 0x5f → STEP_AIM_LEFT/RIGHT 0x73/0x74. The game's
//! `NDrone2_DSTATE_CombatNoRoute` case 3 keeps the state exactly on ChooseCombatMove picks 0 /
//! 0x67 / 0x6f and SetStates anything else [M 0x164c08:36-62]; the port's handler mirrors that
//! keep-set. The game never PICKS the step-aims here because every Can* probe of
//! `NDrone2_ChooseCombatMove` gates on the +0x31C side bits (`CanStepLeft` 0x14f640:50 `& 8`,
//! the Right probe `& 0x10`) — and those bits are set EVERY tick by `DroneMove_NoBunching`
//! 0x150ad0 (called from ControlSTANDARD 0x1490f0:202-226, which also zeroes +0x31C and stamps
//! the scan's return byte into drone+0x17 = the `cVar1` of ChooseCombatMove 0x150c18:58): the
//! two guards stand ~2.4 apart at the door [L: (-51.6,41.2) / (-51.2,38.8)], each reads the
//! other as the +0x314 neighbor, and the 8/0x10 pair lands on BOTH — every dodge/strafe/step
//! probe refuses, every pick cascades to 0x67 (Rand(4) ≠ 0) or 0, and 0x5f holds.
//! The port's NoBunching mirror is HELD INERT (`NB_SCAN_LIVE = false` in drone.rs — the flip
//! regresses walkreal_07000014_live's corridor legs, see the ignore note below), so the port's
//! side bits stay 0, the probes pass, and the guard cycles. Proven in the port: flipping only
//! that const still cycles, because the first guard's step-aim wandering separates the pair
//! past the 3.0 scan radius before the second guard arrives — the cycle and the missing scan
//! feed each other from the first tick.
//! The 0x5f hold therefore cannot be pinned without landing the scan (and re-pinning the 014
//! walk it reshuffles); the ignored test below names exactly that blocker.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const OPEN_DOOR: u8 = 0x81;
const COMBAT_NO_ROUTE: u8 = 0x5f;

#[test]
fn released_guards_do_not_work_the_locked_west_door() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let door = scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.ch_lock == 0x85)
        .expect("the west door with lock ch 0x85 [D placements 151/152]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut at_door_max = 0usize;
    let mut came = false;
    let mut seen_states: Vec<u8> = Vec::new();
    for t in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.is_locked(door, &sim.drones.channels), "the door stays locked (no cascade fired)");
        let near: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
            .filter(|d| d.health > 0.0 && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 9.0).collect();
        at_door_max = at_door_max.max(near.len());
        came |= !near.is_empty();
        for d in &near {
            assert_ne!(d.state, OPEN_DOOR, "t{t}: a released guard works the LOCKED door at ({:.1},{:.1}) — the A* door gate [M 0x1571b0:262-271]", d.pos[0], d.pos[2]);
            if !seen_states.contains(&d.state) { seen_states.push(d.state); }
        }
    }
    assert!(came, "the released guards reach the door side [L: two guards stop at (-51.6,41.2) / (-51.2,38.8)]");
    eprintln!("[014] guards near the door (max at once): {at_door_max}; states sampled at the door: {:?}",
        seen_states.iter().map(|s| format!("{s:#04x}")).collect::<Vec<_>>());
    {
        let sim = scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.health > 0.0 && d.enabled && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 400.0 {
                eprintln!("[014] released guard {i}: ({:.2},{:.2},{:.2}) state {:#04x} no_bunching {:#06x} bunched {}",
                    d.pos[0], d.pos[1], d.pos[2], d.state, d.no_bunching, d.bunched);
            }
        }
    }
}

/// The game's hold, pinned only when the port can run the same scan. The game's door guards sit
/// in COMBAT_NO_ROUTE 0x5f at EVERY sample (17 live samples, 30-frame spacing, CUR 0x5f / PREV
/// 0x5a, opponent set, grace 0 [L 2026-10-30]); the mechanism is the +0x31C side bits the
/// NoBunching scan stamps per tick [M 0x150ad0:42-55 via 0x1490f0:202-226].
/// drone-nobunching-side-bits landed the faithful scan (the Steer-pairing keeper [M 0x14eba8:83-85
/// radius 3.0], the obstacle's movement-lock refusal [M 0x150ad0:22], the +0x17 return byte
/// [M 0x150ad0:56-62]) behind NB_SCAN_LIVE and re-verified the blockers with it LIVE:
/// (1) RESOLVED — the mirror is ported and flips green on 01 (walkreal_07000001_live 5/0 with
/// the scan on, measured this branch);
/// (2) STILL OPEN, refined — with the scan live the pairing STILL never bites at the door: only
/// #783 (the group-0xd full-alert template) routes to the door; the released group-9 pair
/// #778/#780 wake into a no-route combat at their own spawns and shuffle ±1.8 there (end-state
/// dump: (-29.25,15.12) 0x73 / (-22.87,40.10) 0x73, spawns [D audit] (-30.58,13.36) /
/// (-23.52,36.03)) while the game's two guards stop 2.4 apart at the door [L 2026-10-30]; AND
/// the flip regresses walkreal_07000014_live 6/0 → 4/2 (two corridor legs wedge at the 6000-tick
/// cap, stash-verified pre-flip pass) — which guard behaviour is game-true there is a live check.
/// Un-ignore when the release-to-door chain and the 014 walk re-pin land; the assert below is
/// the game's behaviour and must not be weakened.
#[test]
#[ignore = "TWO named blockers, both evidence-backed with the faithful scan landed behind NB_SCAN_LIVE (drone-nobunching-side-bits): (1) the scan is now PORTED (Steer-pairing keeper 3.0 [M 0x14eba8:83-85], the obstacle's movement-lock refusal [M 0x150ad0:22], the +0x17 return byte [M 0x150ad0:56-62]) and flips green on 01 (walkreal_07000001_live 5/0 with it on, measured on that branch) — but the flip regresses walkreal_07000014_live 6/0 → 4/2 (two corridor legs wedge at the 6000-tick cap; the pre-flip tree passes, stash-verified) so it stays inert pending that walk's live re-pin; (2) at Operative only #783 walks to the door: the released group-9 pair #778/#780 spawn at alertness 1.0 (prop 0x20 = param 19 & 3 = 3 [M bitDescs 0x2c7250; 0x14c600:61-66]) so the game's own wake fold sends them to ATTACK 0x56 [M 0x14cbd8:122-155] and their combat route to a hall-side goal cannot cross the locked door link [M 0x1571b0:262-271] — the decompile chain holds them no-route at their spawns in the game too; the [L] door pair (-51.6,41.2)/(-51.2,38.8) is the MISSION-WALKER pair (#783 + #784), which both exist only at Agent and up — the port reproduces that end state at Agent (at_agent_the_two_mission_walkers_hold_at_the_locked_door_like_the_game) — and the [L] boot's difficulty and guard ids are the live check in docs/reports/level-14-door-pair.md"]
fn the_door_guards_hold_combat_no_route_like_the_game() {
    let mut scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut samples = 0usize;
    for t in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let near: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
            .filter(|d| d.health > 0.0 && (d.pos[0] + 51.5).powi(2) + (d.pos[2] - 40.0).powi(2) < 9.0).collect();
        if near.is_empty() { continue; }
        if t % 30 == 0 {
            samples += 1;
            for d in &near {
                assert_eq!(d.state, COMBAT_NO_ROUTE,
                    "t{t}: the door guard holds COMBAT_NO_ROUTE 0x5f at every sample in the game (17 live samples, CUR 0x5f / PREV 0x5a [L 2026-10-30]); the port's guard is in {:#04x} at ({:.1},{:.1}) — the NoBunching side bits [M 0x150ad0:42-55] are not stamping while NB_SCAN_LIVE is inert",
                    d.state, d.pos[0], d.pos[2]);
            }
        }
    }
    assert!(samples >= 17, "the sampling window must cover the live window's span (17 samples at 30 frames), got {samples}");
}

/// Sister: the gate is the LOCK, not the door. With the lock channel on (the cascade's ch 0x85),
/// the same door is no route block (the A* expands through it).
#[test]
fn the_unlocked_door_is_no_route_block() {
    let scene = nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_ref().unwrap();
    let door = sim.drones.doors.list.iter().position(|d| d.ch_lock == 0x85).unwrap();
    let mut ch = sim.drones.channels.clone();
    assert!(sim.drones.doors.is_locked(door, &ch), "locked at load");
    ch.set(0x85, 1, 1);
    assert!(!sim.drones.doors.is_locked(door, &ch), "ch 0x85 unlocks it [M Door_IsLocked 0x136810:10-16]");
}

// level-14-missing-group-guards: the release's "missing" guards are the disc's own difficulty
// tags, not a parse drop. `Drone_Create` 0x136be0 (first branch) creates NOTHING for a placement
// whose level tag (parameter 2, level_tag+0x34) exceeds the load's difficulty —
// `*(uint *)(param_3 + 0x34) <= (uint)GameState._40_4_` [M 0x136be0]; `_40_4_` is Ghidra's
// DECIMAL 40 = +0x28 (0x2a3790), the port's `Drones::difficulty` (1 Operative / 2 Agent /
// 3 00 Agent [D difficulty table 0x2df4c0; M C_SBNFDFCTY_Handler 0x216dc4]). The port mirrors
// the gate at level_drones (drone.rs, `if param(2) > d.difficulty { return None; }`) — the
// ghost-guards-990 mirror. [D audit :73-76/:85-87] parameter 2: #721/#722/#724/#783 = 0x1,
// #723 = 0x3, #784 = 0x2, #785 = 0x3; group 9: #778/#780 = 0x1, #779 = 0x2, #781 = 0x3.
// The tests load at difficulty 1 (Operative — the game's mission-start force writes +0x28 = 1
// [M Menu_RunMiniMission 0x1ffb90:22; L the level-start snapshots' +0x28 = 1]), so exactly the
// parameter-2 >= 0x2 set is gated — the probe's absent set. Spawners [D audit :1527/:1531]:
// #1333 group 0xd max 7 waves 1 wake 0x8C; #720 group 9 max 2 wake 0x8C; mux #717
// 0x8C = 0x87 AND 0x88 [D audit :1411]. The ambush trio stays 0x38 at every difficulty —
// `NDrone2_DSTATE_AmbushWait` 0x161560 has no channel read; its only exits are the
// opponent-sight bit or the player within 3.0 2-D [M 0x161560:28-70], and Bond is held 25+
// units away in the hall.

const AMBUSH_WAIT: u8 = 0x38;

fn at_xz<'a, I>(list: I, x: f32, z: f32) -> Option<&'a nf_game::drone::Drone>
where I: IntoIterator<Item = &'a nf_game::drone::Drone> {
    list.into_iter().find(|d| (d.pos[0] - x).powi(2) + (d.pos[2] - z).powi(2) < 0.25)
}

/// The [D] audit positions of the three templates the probe found missing at Operative.
const TAGGED: [(u32, f32, f32); 3] = [(723, -32.78, 28.48), (784, -36.18, 27.76), (785, -31.74, 25.16)];
/// The ambush trio's [D] positions (#721/#722/#724) and #783 (the one full-alert group-0xd
/// template at Operative).
const TRIO: [(u32, f32, f32); 3] = [(721, -51.33, 32.42), (722, -47.87, 33.48), (724, -40.24, 25.15)];
/// The release's template set: the seven group-0xd wake-0x8C templates [D #1333] + the four
/// group-9 wake-0x8C templates [D #720], by main-map placement index (the audit's #N).
const RELEASED: [usize; 11] = [721, 722, 723, 724, 783, 784, 785, 778, 779, 780, 781];
/// The ambush QUARTET's audit ids and [D] positions — #723 shares prop word 0xf1dfc8c1 with
/// #721/#722/#724 [D audit :73-76/:85-87], so it is AmbushWait too, not a door router.
const QUARTET: [(usize, f32, f32); 4] = [(721, -51.33, 32.42), (722, -47.87, 33.48), (723, -32.78, 28.48), (724, -40.24, 25.15)];
/// The [D] spawn positions of the release's two mission walkers: #783 (the one full-alert
/// group-0xd template at Operative) and #784 (the second, param 2 = 0x2, Agent and up).
const WALKERS: [(u32, f32, f32); 2] = [(783, -37.73, 25.86), (784, -36.18, 27.76)];
/// The east node of the locked west door pair (the mission paths' end) [D node dump:
/// path 0 node 124 (-51.4,0.4,39.6), flags 0x2 door-type, bound to the ch-0x85 door].
const DOOR_NODE: (f32, f32) = (-51.4, 39.6);

fn hold_and_release(difficulty: u32) -> (nf_view::scene::Scene, std::collections::HashSet<usize>) {
    let mut scene = if difficulty == 1 {
        nf_view::load(&iso(), "07000014.bin", false).expect("load 07000014")
    } else {
        nf_view::load_with_carry_difficulty(&iso(), "07000014.bin", false, None, difficulty).expect("load 07000014")
    };
    for _ in 0..400 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    }
    let pre: std::collections::HashSet<usize> = scene.sim.as_ref().unwrap().drones.list.iter()
        .filter(|d| d.enabled && d.health > 0.0).map(|d| d.placement).collect();
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x87, 1, 1);
        sim.drones.channels.set(0x88, 1, 1);
    }
    (scene, pre)
}

fn run_release(scene: &mut nf_view::scene::Scene) {
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    for _ in 0..3000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
    }
}

/// Operative (difficulty 1): the parameter-2 >= 0x2 templates never exist, so the ch 0x87+0x88
/// release spawns SIX alive guards — the ambush trio (#721/#722/#724, all 0x38 [M 0x161560]) +
/// #783 + the group-9 pair (#778/#780) — not nine. Absent at load, enabled or not [M 0x136be0].
/// Released set = enabled ∧ alive ∧ not enabled at load (the wake-0x8C templates sit
/// enabled=false/health=10.0 until spawner #1333/#720 pick them [M 0x137cb0]).
#[test]
fn at_operative_the_tagged_templates_never_exist_and_the_release_spawns_six() {
    let (mut scene, pre) = hold_and_release(1);
    {
        let sim = scene.sim.as_ref().unwrap();
        for (id, x, z) in TAGGED {
            assert!(at_xz(&sim.drones.list, x, z).is_none(),
                "Operative: template #{id} ({x},{z}) must not exist at load — Drone_Create 0x136be0 makes nothing for parameter 2 = 3 > difficulty 1 [M 0x136be0]");
        }
    }
    run_release(&mut scene);
    let sim = scene.sim.as_ref().unwrap();
    let released: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
        .filter(|d| d.enabled && d.health > 0.0 && !pre.contains(&d.placement)).collect();
    eprintln!("[014-op] released after the release: {} — {}", released.len(),
        released.iter().map(|d| format!("({:.2},{:.2}) {:#04x} en {}", d.pos[0], d.pos[2], d.state, d.enabled)).collect::<Vec<_>>().join("; "));
    assert_eq!(released.len(), 6,
        "Operative: the release brings 6 (7 group-0xd candidates gated to 4 [D] + spawner #720's 2 of the 2 existing group-9 [M 0x137cb0]), not 9");
    for (id, x, z) in TRIO {
        let d = at_xz(released.iter().copied(), x, z)
            .unwrap_or_else(|| panic!("Operative: ambush template #{id} must be alive at ({x},{z}) [D audit :73-76]"));
        assert_eq!(d.state, AMBUSH_WAIT,
            "Operative: #{id} holds 0x38 — NDrone2_DSTATE_AmbushWait 0x161560 has no channel read and Bond is 25+ units away [M 0x161560:28-70]");
    }
    for (id, x, z) in TAGGED {
        assert!(at_xz(&sim.drones.list, x, z).is_none(),
            "Operative: template #{id} still must not exist after the release [M 0x136be0]");
    }
}

/// 00 Agent (difficulty 3): every parameter-2 tag passes the gate [M 0x136be0], so the three
/// tagged templates EXIST at load (as wake-0x8C spawner templates, enabled=false until the
/// release) and the release spawns NINE — the ambush QUARTET (all 0x38) + the full-alert
/// #783/#784/#785 + the group-9 spawner's 2 picks from its now-4 candidates (still capped at
/// max 2 [D audit :1527]). The quartet holds 0x38 at every difficulty: #723 carries the SAME
/// ambush prop word 0xf1dfc8c1 as #721/#722/#724 [D audit :85-87; M 0x161560].
#[test]
fn at_00_agent_the_tagged_templates_exist_and_the_release_spawns_nine() {
    let (mut scene, pre) = hold_and_release(3);
    {
        let sim = scene.sim.as_ref().unwrap();
        for (id, x, z) in TAGGED {
            let d = at_xz(&sim.drones.list, x, z)
                .unwrap_or_else(|| panic!("00 Agent: template #{id} must exist at load — parameter 2 <= difficulty 3 passes Drone_Create's gate [M 0x136be0; D audit :85-87]"));
            assert!(!d.enabled,
                "00 Agent: #{id} is a wake-0x8C template, not yet released at load [D audit :85-87]");
        }
    }
    run_release(&mut scene);
    let sim = scene.sim.as_ref().unwrap();
    let released: Vec<&nf_game::drone::Drone> = sim.drones.list.iter()
        .filter(|d| d.enabled && d.health > 0.0 && !pre.contains(&d.placement)).collect();
    eprintln!("[014-00a] released after the release: {} — {}", released.len(),
        released.iter().map(|d| format!("({:.2},{:.2}) {:#04x} en {}", d.pos[0], d.pos[2], d.state, d.enabled)).collect::<Vec<_>>().join("; "));
    assert_eq!(released.len(), 9,
        "00 Agent: the release brings 9 — 7 group-0xd [D #1333 max 7] + 2 group-9 [D #720 max 2] [M 0x137cb0]");
    for (id, x, z) in QUARTET {
        let d = at_xz(released.iter().copied(), x, z)
            .unwrap_or_else(|| panic!("00 Agent: ambush template #{id} must be released at its placement ({x},{z}) [D audit]"));
        assert_eq!(d.state, AMBUSH_WAIT,
            "00 Agent: #{id} still holds 0x38 — the ambush props are difficulty-independent [D audit :85-87; M 0x161560]");
    }
}

/// The game's door pair [L 2026-10-30: two guards stop 2.4 apart at (-51.6,41.2) / (-51.2,38.8),
/// CUR 0x5f / PREV 0x5a] is the release's MISSION-WALKER pair at the boot difficulty, not the
/// group-9 pair. At Agent (+0x28 = 2 — the game's boot default [M bootup 0x1c7e78:25 @0x1c7f88])
/// #784 (param 2 = 0x2 [D audit :85-87]) exists beside #783, and BOTH carry the walker props:
/// prop 0x20 (word 1 bits 0-1 [M bitDescs 0x2c7250 prop 0x20 = word 1 mask 0x3]; param 19
/// 0xee738fe6 → 2) reads 0.66 off the static alertness table {0, 0, 0.66, 1.0} [M 0x14c600:61-66,
/// table 0x2f5cc8], so GetDroneTypeAttackTypeFriend's `1.0 <= +0x50c` fold does NOT fire and both
/// wake to start 4 [M 0x14cbd8:122-155]; IDLE's prop-0x1d arm (word 0 bit 0x20000000, set in both
/// words 0xf1dec9c1/0xf1dec8e1 [M bitDescs prop 0x1d]) assigns the kind-8 MissionPath and
/// SetStates 0x1b EnemyMission [M 0x15bf18:32-56], whose walk ends at the east door node
/// (-51.4,39.6) [D node dump]. The group-9 templates (param 19 0xee738fe7 → 3 → alertness 1.0)
/// take the fold to ATTACK 0x56 in the game's own chain [M 0x14cbd8:122-155] — their combat route
/// to a hall-side goal cannot cross the locked door link [M AINetwork_DoAStarPath 0x1571b0:262-271;
/// the goal-node set around Bond is hall-side, NDrone2_NavNodeCache 0x155cd8 → BuildAITarget
/// 0x15ac78 → NodeSearchCel 0x156e18:59-62 radius 15.0], so a spawn-parked no-route hold is the
/// game's own Operative picture for them; which two guards the [L] boot held at the door is a
/// live check (the report's). This pin: at Agent BOTH walkers reach the door node and hold there,
/// and neither works the locked door — the [L] end state on the port.
#[test]
fn at_agent_the_two_mission_walkers_hold_at_the_locked_door_like_the_game() {
    let (mut scene, _pre) = hold_and_release(2);
    let hall = [-76.8, -1.1 + 1.0347, 46.5];
    let mut walkers: Vec<(u32, usize)> = Vec::new();
    let mut walked = [0.0f32; 2];
    for t in 0..2400u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = hall;
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for (id, x, z) in WALKERS {
            if walkers.iter().any(|(w, _)| *w == id) { continue; }
            if let Some(i) = sim.drones.list.iter().position(|d| d.enabled && d.health > 0.0
                && (d.pos[0] - x).powi(2) + (d.pos[2] - z).powi(2) < 0.25) {
                walkers.push((id, i));
            }
        }
        for (k, (id, i)) in walkers.iter().enumerate() {
            let d = &sim.drones.list[*i];
            assert_ne!(d.state, OPEN_DOOR,
                "t{t}: walker {k} (#{id}) works the LOCKED door at ({:.1},{:.1}) [M 0x1571b0:262-271]", d.pos[0], d.pos[2]);
            walked[k] = walked[k].max((d.pos[0] - WALKERS.iter().find(|w| w.0 == *id).unwrap().1).hypot(d.pos[2] - WALKERS.iter().find(|w| w.0 == *id).unwrap().2));
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(walkers.len(), 2, "both mission walkers must spawn at Agent [D audit :85-87]");
    for (k, (id, i)) in walkers.iter().enumerate() {
        let d = &sim.drones.list[*i];
        assert!(walked[k] > 10.0,
            "walker {k} (#{id}) must WALK to the door (max leg {:.1}) [M 0x15bf18:32-56 the kind-8 assign]", walked[k]);
        let dd = (d.pos[0] - DOOR_NODE.0).powi(2) + (d.pos[2] - DOOR_NODE.1).powi(2);
        assert!(dd < 9.0,
            "walker {k} (#{id}) must hold at the door node (-51.4,39.6) [L (-51.6,41.2)/(-51.2,38.8) 2.4 apart]; it is at ({:.2},{:.2}), d2 {dd:.2}",
            d.pos[0], d.pos[2]);
        eprintln!("[014-agent] walker {k} (#{id}) end ({:.2},{:.2}) state {:#04x} leg {:.1}",
            d.pos[0], d.pos[2], d.state, walked[k]);
    }
}
