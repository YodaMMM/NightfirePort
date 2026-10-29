//! escort-01: 07000001's escort walked from the meet-up to the exit with BOND AT WALKING SPEED
//! (the analog half-press, `forward ≤ 0.55` — the walkreal-a harness runs full-stick). The two
//! game rules under test:
//!
//! 1. **The follow distance / catch-up** [M 0x15e698 case 3 + 0x15eda8 case 3 +
//!    `DroneMove_ObjectMissionPathDistance` 0x153f50]: while Mayhew LEADS he keeps within the
//!    game's distance — Bond more than 9 route-units behind (path < −9.0) parks him in
//!    AllyLeadWait, and he resumes when Bond is back within 7 (path > −7.0). The signed path
//!    value is the drone's `mission_path_dist` (the game's SM+0x20 scratch).
//! 2. **The exit's condition**: the transition is NOT gated on Mayhew reaching the door — the
//!    exit doorway (gallery_secretdoor #224/#225) has no channels at all and Bond can E-open it
//!    with Mayhew anywhere [D docs/atlas/door.md]; what fires 07000002 is the AND plate (Touch
//!    out 0x10, mask 0x18, filter player 0xE + skin 0x4F): BOTH bodies inside the volume → ch
//!    0x10 → the LOAD_LEVEL gate [M Trigger_Hit 0x1b1fc8 / Trigger_Update case 10 0x1b1a78].

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_game::drone::state;

/// The states the escort machine legitimately parks in while Bond walks behind him
/// ([M 0x15e698/0x15ea28/0x15ec08/0x15efc8/0x15f120/0x15f2e8] + the route's own OpenDoor 0x81).
const ESCORT_STATES: [u8; 10] = [
    state::ALLY_LEAD_INIT, state::ALLY_LEAD, state::ALLY_LEAD_WAIT, state::PLAYER_IN_WAY,
    state::LEAD_HIDE, state::LEAD_MISSION_WAIT, state::LEAD_BOND_COMBAT, state::ALLY_LEAD_DONE,
    state::OPEN_DOOR,
    // the cover run [M 0x15e698:30-36 → 0x15f938]
    state::GO_TO_GOAL_ALLY,
];

#[test]
fn escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // The intro cut-scene writes ch 0xB (the contact's enable) ~tick 2119; its tail re-places
    // Bond — the same wait-out walkreal-a runs.
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro must write ch 0xB");
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);

    // Mayhew (skin 0x4F) and the AND plate.
    let ci = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate");

    // The AllyLead machine hides him while an armed, started guard stands within 14 [M 0x15ec08];
    // the real player clears them through the real fire path (walkreal-a's choreography) BEFORE
    // the escort leg — so the walking-speed leg starts with the lead actually leading.
    let mut cleared = std::collections::HashSet::new();
    for _round in 0..12 {
        let (cp, cstate) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[ci].pos, sim.drones.list[ci].state)
        };
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() && cstate != state::LEAD_HIDE { break; }
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.0);
            }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            let killed = w.shoot(g, &format!("guard {g}")) || { w.resupply("ammo retry"); w.shoot(g, &format!("guard {g}")) };
            let sim = w.scene.sim.as_ref().unwrap();
            if killed || sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0 { cleared.insert(g); }
        }
    }
    let cpos = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    w.walk_nav_to(cpos, "to the meet-up", 2.5);

    // ── the walking-speed escort: Bond follows at the analog half-press ────────────────────────
    // The lead RUNS (anim 0x1e [M 0x15e698 case 3]) — he pulls ahead until Bond is more than 9
    // route-units behind, then waits (AllyLeadWait); Bond's walk closes to 7 and he resumes. The
    // follower keeps 4.5..10 behind, out of the PlayerInWay band, and every second asserts the
    // game's own bands off the drone's `mission_path_dist` (SM+0x20).
    w.keep_alive.push(ci);
    let mut checked = 0u32;
    let mut last_round = [0.0f32; 3];
    let mut last_bond: Option<[f32; 3]> = None;
    let mut last_flee = 0u32;
    let settled = 'escort: {
        for round in 0..12 {
            let mp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
            if walker::dxz(mp, plate) < 5.0 { break 'escort mp; }
            let mut last = mp;
            let mut still = 0u32;
            let start = w.total_ticks;
            loop {
                // The walker's allowance extends to the escort: this leg pins the follow bands, not
                // the firefight — guards still alive along the route can catch the hiding contact
                // in their fire at Bond (msg 8 through the GLOBAL base arm [M 0x1469b8:134-166]),
                // which the game allows too. Keep him standing so the bands stay observable.
                // (`Walker::keep_alive`, armed before the leg — every tick, door sub-loops included.)
                if w.total_ticks - start >= 9000
                    || w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[ci];
                let (dstate, dpos, pd) = (d.state, d.pos, d.mission_path_dist);
                last_round = dpos;
                let bond = sim.bond.pos;
                let db = walker::dxz(bond, dpos);
                // The distance asserts — the game's own bands, evaluated whenever the escort
                // machine has computed the path value this tick (Lead/Wait update it).
                // The bands hold outside the scare's grace: the recovery's SetState(0x20) carries no
                // band [M 0x173658 case 1] and the re-anchor walk takes a moment, so within 600 ticks
                // of the last flee sighting the resumed Wait may sit outside the -7/0 band [M
                // 0x15eda8 case 3 pins the ALLY_LEAD-case-3 door, not the recovery's].
                let grace = w.total_ticks.saturating_sub(last_flee) <= 600;
                if matches!(dstate, state::ALLY_LEAD | state::ALLY_LEAD_WAIT) {
                    checked += 1;
                    if dstate == state::ALLY_LEAD {
                        assert!(pd >= -9.0, "the lead abandoned Bond: path {pd:.1} < -9.0 at ({:.1},{:.1},{:.1}) [M 0x15e698 case 3]", dpos[0], dpos[1], dpos[2]);
                    } else if !grace {
                        assert!(pd <= -7.0 || pd == 0.0,
                            "the lead resumed outside the game's band: path {pd:.1} in AllyLeadWait [M 0x15eda8 case 3]");
                    }
                    if !grace {
                        assert!(db < 24.0, "the escort left Bond behind: {db:.1} straight-line at ({:.1},{:.1},{:.1})", dpos[0], dpos[1], dpos[2]);
                    }
                }
                                // The grenade scare's flee chain is the game's own answer for the escorted contact:
                // the ally states' case 0x21 arms call ConsiderExplosive [M 0x15e698:131 et al.] and
                // Drone_SM_SetState(0x86) [M 0x1431c0:194-195] - the escort leaves the ally machine
                // for the run-away/hide/recover chain and the recovery re-anchors the route and
                // returns to AllyLeadWait [M 0x173658 case 1]. level-13-guards-7: the -6 corpse fix
                // shifted the shared-RNG stream so the guard-15 fight's grenade now lands inside the
                // escort's explosive scan. The resume is pinned below (the grace window + the round
                // loop), not the absence of the scare.
                if matches!(dstate, state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT | state::RECOVER_FROM_SCARY_OBJECT) {
                    last_flee = w.total_ticks;
                }
                assert!(ESCORT_STATES.contains(&dstate)
                    || matches!(dstate, state::RUN_AWAY_FROM_OBJECT | state::HIDE_FROM_SCARY_OBJECT | state::RECOVER_FROM_SCARY_OBJECT),
                    "the escort left the ally machine: state {dstate:#x}");
                drop(sim);
                still = if walker::dxz(dpos, last) < 0.0025 { still + 1 } else { 0 };
                last = dpos;
                if walker::dxz(dpos, plate) < 5.0 { break 'escort dpos; }
                if still > 900 { break; } // 15 s parked — the round loop recovers (as walkreal's)
                // Steer BEHIND the lead at the walk pace: full press only past 10 (the catch-up
                // closing), the 0.55 half-press (the walk) in 4.5..10, stand inside 4.5. The band
                // is 3D — a lead waiting at a stair head must be FOLLOWED (the 2D dxz is blind to
                // the floors and would stand Bond under him forever).
                let sim = w.scene.sim.as_ref().unwrap();
                let desired = (dpos[0] - sim.bond.pos[0]).atan2(dpos[2] - sim.bond.pos[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let dy = dpos[1] - sim.bond.pos[1];
                let d3 = (walker::dxz(sim.bond.pos, dpos).powi(2) + dy * dy).sqrt();
                let input = Input {
                    turn: err.clamp(-0.07, 0.07),
                    forward: if d3 > 10.0 { 1.0 } else if d3 > 4.5 { 0.55 } else { 0.0 },
                    ..Default::default()
                };
                drop(sim);
                w.tick(input, false);
            }
            // Stall recovery — the walkreal-a rules: clear armed started guards within 14, step
            // out of the way band, or walk past the wedged lead so he re-paths.
            // The ally-wait hold ( ReachedDestNode 0x153358:85-94 parks the contact in 0x21 at the
            // 0x20 node until ch 0x99) needs the corridor plate crossed first: the TOUCH plate at
            // (6.79, 6.04, 8.91) raises ch 0x32 → the OUT mux mirrors 0x33 → SP #276 starts →
            // 06000086 writes ch 0x0e → the ambush guards enable; their deaths raise 0x99 through
            // the AND mux and the hold releases [D the 07000001 placements + lib dump]. The plate
            // is ON the escort route between the hold and door #1 — the walking player crosses it.
            if !w.scene.sim.as_ref().unwrap().drones.channels.on(0x33) {
                w.walk_nav_to([6.79, 0.0, 8.91], "onto the corridor plate", 0.4);
            }
            let (cstate, cp, cpd) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[ci];
                (d.state, d.pos, d.mission_path_dist)
            };
            eprintln!("[escort] round {round} stall: contact ({:.1},{:.1},{:.1}) state {cstate:#x} path {cpd:.1} bond ({:.1},{:.1},{:.1}) db {:.1}",
                cp[0], cp[1], cp[2], w.scene.sim.as_ref().unwrap().bond.pos[0], w.scene.sim.as_ref().unwrap().bond.pos[2], w.scene.sim.as_ref().unwrap().bond.pos[1], walker::dxz(cp, w.scene.sim.as_ref().unwrap().bond.pos));
            // The un-wedge (the player's stick move): a round that ended where it started means
            // Bond is wedged against a wall corner — the direct steer and the nav legs all slide
            // into the same wall. The game's player rounds it with the stick's LATERAL component
            // (the analog strafe the walk uses everywhere else, e.g. the wire ride): jog laterally
            // both ways with a little forward until Bond actually moves, then let the recovery
            // below re-aim from the freed spot.
            let bond_now = w.scene.sim.as_ref().unwrap().bond.pos;
            if last_bond.is_some_and(|p| walker::dxz(p, bond_now) < 0.5) {
                eprintln!("[escort] round {round}: bond wedged at ({:.1},{:.1},{:.1}) — the stick's lateral jog",
                    bond_now[0], bond_now[1], bond_now[2]);
                for side in [1.0f32, -1.0] {
                    let jog_start = w.total_ticks;
                    while w.total_ticks - jog_start < 45 {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let desired = (cp[0] - sim.bond.pos[0]).atan2(cp[2] - sim.bond.pos[2]);
                        let err = walker::wrap_pi(desired - sim.bond.yaw);
                        let input = Input { turn: err.clamp(-0.07, 0.07), strafe: side, forward: 0.2, ..Default::default() };
                        drop(sim);
                        w.tick(input, false);
                    }
                    if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, bond_now) > 0.5 { break; }
                }
            }
            last_bond = Some(bond_now);
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[ci];
                if let Some(r) = &d.route {
                    let nodes: Vec<String> = r.nodes.iter().take(r.index.max(0) as usize + 3).rev().take(4)
                        .map(|&n| format!("({:.1},{:.1},{:.1})", sim.nav.paths[r.path].nodes[n as usize].pos[0], sim.nav.paths[r.path].nodes[n as usize].pos[1], sim.nav.paths[r.path].nodes[n as usize].pos[2]))
                        .collect();
                    eprintln!("[escort] route idx {} of {}: behind {:?} target {:?}", r.index, r.nodes.len(), nodes, r.target);
                }
            }
            let guards: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                        && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
                }).collect()
            };
            if !guards.is_empty() {
                for g in guards {
                    let (gpos, los_clear) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let d = &sim.drones.list[g];
                        let bp = sim.bond.pos;
                        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                        (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
                    };
                    let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
                    if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                        w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.0);
                    }
                    if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
                    let killed = w.shoot(g, &format!("guard {g}")) || { w.resupply("ammo retry"); w.shoot(g, &format!("guard {g}")) };
                    let sim = w.scene.sim.as_ref().unwrap();
                    if killed || sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0 { cleared.insert(g); }
                }
            } else if walker::dxz(cp, w.scene.sim.as_ref().unwrap().bond.pos) > 2.0
                && (w.scene.sim.as_ref().unwrap().world.segment(
                    [w.scene.sim.as_ref().unwrap().bond.pos[0], w.scene.sim.as_ref().unwrap().bond.pos[1] + 1.0, w.scene.sim.as_ref().unwrap().bond.pos[2]],
                    [cp[0], cp[1] + 1.0, cp[2]]).is_some()
                    || cstate == state::ALLY_LEAD_WAIT) {
                // ... or the lead is simply PARKED waiting (AllyLeadWait): the real player walks to
                // him — the resume band is route-relative, so standing outside the route's fence
                // strip (db 2.6, the scan's far-node −9.8) never un-parks him; closing through the
                // nav does.
                // The straight line to the lead is BLOCKED (a wall corner / a closed shoji between
                // them): the direct steer wedges against it and holds the route gap open. Close
                // through the nav route — the walker's stuck handler opens the route's doors under
                // Bond's feet [the walkreal-a guard-approach rule].
                // The gallery doorway wedge (the stream-shifted park at (3.4, 6.3, 6.6) puts this
                // recovery to work every round): Bond's west approach at (4-5.5, 11.5) clips the
                // slide door's OPEN panel — the auto-close [M 0x13660c-0x136700] re-closes it every
                // 360 quiet ticks and the re-open slides the panel back WEST across the approach.
                // The player walks the frame's OPEN east half instead: along the south wall face,
                // through the frame's east side, then re-aim from the lead's side of the wall.
                let bp = w.scene.sim.as_ref().unwrap().bond.pos;
                if bp[0] > 2.5 && bp[0] < 7.0 && bp[2] > 10.5 && bp[2] < 12.5 && cp[2] < bp[2] - 2.0 {
                    eprintln!("[escort] round {round}: the gallery doorway wedge — walking the frame's open east half");
                    w.try_walk_to(Wp::new(8.4, 1.3, 11.6).r(1.0), "the doorway's east jamb");
                    w.try_walk_to(Wp::new(8.4, 1.3, 10.2).r(1.0), "through the open half");
                }
                w.walk_nav_to(cp, "the line to the lead is blocked — close through the nav", 2.0);
            } else if (cp[1] - w.scene.sim.as_ref().unwrap().bond.pos[1]).abs() > 1.5 {
                // A floor gap (the lead parked at a stair head, Bond below): the direct steer is
                // blind to the vertical — close through the nav route (the stairs) [the walker's
                // stall-recovery close, as walkreal-a's].
                w.walk_nav_to(cp, "close the floor gap to the lead", 2.0);
            } else if cstate == state::PLAYER_IN_WAY {
                let back = [cp[0] + (cp[0] - plate[0]) * 0.5, cp[1], cp[2] + (cp[2] - plate[2]) * 0.5];
                w.walk_nav_to(back, "step out of the lead's way", 3.0);
            } else {
                // A closed door on the route: the escort works his own door nodes (DSTATE_OpenDoor
                // 0x81), but the courtyard slide door is BOND's to open — the game's other
                // activator [M Player_Activate 0x1a5170 case 0x1c → Door_Activate(obj, player, 1)].
                // The fix_mission patience: press only while the door is CLOSED/CLOSING (& 0x12), a
                // press on an OPENING door re-closes it [M 0x135be0's state-4/8 arm]; stand behind
                // the contact on his line to the door.
                let door_i = if cstate == state::OPEN_DOOR {
                    // He is working his own door node [M 0x1681f8] — pressing over him mashes the
                    // door (the fix_mission deadlock: an activation every tick sets the tick-busy
                    // marker and re-closes what he opened). Wait for his work instead.
                    None
                } else {
                    // The BLOCKING door sits between Bond and the contact — pick it by the
                    // midpoint of the pair, not by the contact alone (a closed door behind the
                    // contact's back would win the wrong way).
                    let mid = [(cp[0] + w.scene.sim.as_ref().unwrap().bond.pos[0]) * 0.5,
                               cp[1],
                               (cp[2] + w.scene.sim.as_ref().unwrap().bond.pos[2]) * 0.5];
                    w.scene.sim.as_ref().unwrap().drones.doors.list.iter().enumerate()
                        .filter(|(_, dr)| dr.state & 0x12 != 0 && walker::dxz(dr.pos, mid) < 8.0)
                        .min_by(|(_, a), (_, b)| walker::dxz(a.pos, mid).total_cmp(&walker::dxz(b.pos, mid)))
                        .map(|(i, _)| i)
                };
                if let Some(door_i) = door_i {
                    let (dpos, dl0) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        (sim.drones.doors.list[door_i].pos, sim.drones.doors.list[door_i].state)
                    };
                    eprintln!("[escort] round {round}: the contact parks at a closed door {door_i} (state {dl0:#x}) at ({:.1},{:.1}) — Bond E-opens it", dpos[0], dpos[2]);
                    let (dx, dz) = (dpos[0] - cp[0], dpos[2] - cp[2]);
                    let l = (dx * dx + dz * dz).sqrt().max(1e-4);
                    // door-react-path5: the use fires once per press edge [M 0x1a5ea8-0x1a5ee8 +
                    // 0x17f084-0x17f090] — a held key whose first activation is refused (the
                    // 2-tick busy marker from the contact's own door work [M 0x135be0's tail])
                    // would deadlock the hold. Tap: press, release, press — every press fresh,
                    // the top-of-loop break still stops on OPENING/OPEN before a re-close.
                    let mut flip = false;
                    for _ in 0..1500u32 {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let st = sim.drones.doors.list[door_i].state;
                        if st & 0x12 == 0 { break; } // opening/open — stop pressing
                        let bp = sim.bond.pos;
                        let stand = [dpos[0] - dx / l * 1.6, dpos[1], dpos[2] - dz / l * 1.6];
                        let d = walker::dxz(bp, stand);
                        let desired = (stand[0] - bp[0]).atan2(stand[2] - bp[2]);
                        let err = walker::wrap_pi(desired - sim.bond.yaw);
                        drop(sim);
                        if d > 1.8 {
                            w.tick(Input { turn: err.clamp(-0.07, 0.07), forward: 0.55, ..Default::default() }, false);
                        } else {
                            w.tick(Input { turn: err.clamp(-0.07, 0.07), ..Default::default() }, flip);
                            flip = !flip;
                        }
                    }
                } else {
                    let (dx, dz) = (plate[0] - cp[0], plate[2] - cp[2]);
                    let l = (dx * dx + dz * dz).sqrt().max(1e-3);
                    let ahead = [cp[0] + dx / l * 4.0, cp[1], cp[2] + dz / l * 4.0];
                    w.walk_nav_to(ahead, "walk past the wedged lead", 3.0);
                }
            }
        }
        panic!("the escort never reached the plate in 12 rounds (last at {last_round:?})");
    };
    assert!(checked > 0, "the walking-speed leg must have evaluated the game's distance bands");
    assert!(walker::dxz(settled, plate) < 6.0, "the contact must walk his route to the LoadPointActivator plate (at {settled:?})");
    eprintln!("[escort] the contact settled at ({:.1},{:.1},{:.1}) after {} distance checks",
        settled[0], settled[1], settled[2], checked);

    // break-debris-shatter-2: the harness must not end the level before the final walk. The
    // game's plate is a kind-0xEB TOUCH trigger, out 0x10, mask 0x18, filter [0xE, 0x0500004F]
    // [D the 07000001 placement — player AND the 0x0500004F contact both inside its volume],
    // box x −33.8..−32.4, y 0.64..3.96, z 12.17..18.68; the LOAD_LEVEL (0xE8) trigger's gate is
    // ch 0x10 [M Trigger_Update case 10 0x1b1a78:162-166] — the contact STANDS in the box once
    // settled, so the level ends the moment Bond's clearing-phase walk crosses it [M Trigger_Hit
    // 0x1b1fc8]. 785's kill draws are the game's own (Debris_Create's eight shared-stream draws
    // per piece [M 0x1aaa00:30-50] and the shatter's split draws [M 0x12b270:348-372], the same
    // global SEED_X/SEED_Y pair every AI draw uses [M 0x1e36f0/0x1e3738; Bullet_init's spread
    // 0x12c5a0]) — the escort's timeline legitimately moves with every breakable the walk kills,
    // so the choreography keeps Bond out of the plate: the reserve is topped up here (harness
    // setup, not game behaviour) so the low-ammo resupply — whose only route descends the
    // stairwell THROUGH the plate — never runs. The trail sister below pins the avoidance.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // The durable pool (the reload source; gun.spare resyncs from it on the weapon events)...
        let at = sim.weapons.ammo_type(sim.gun.weapon as usize);
        if at < sim.drones.inventory.carried.len() {
            sim.drones.inventory.carried[at] = sim.drones.inventory.carried[at].max(120);
        }
        // ...and the synced view the harness's low-ammo check reads before the next sync.
        sim.gun.spare = sim.gun.spare.max(120);
    }
    let trail_at_clear = w.trail_points().len();

    // The last stretch's guards: the stair-head/gallery trigger starts the gallery's window guard
    // (and the stairwell pair) only when the lead is already at the route's end, AFTER the leg's
    // own stall-recovery stopped running — the game's AllyLeadHide parks the contact while an
    // armed, started guard stands within 14 [M 0x15ec08 case 3 → NDrone2_NotNearArmedDrone(14.0)],
    // and only a dead guard un-parks him. The real player kills them before walking in (the
    // walkreal-a clearing rule the leg already runs); the same rule here, for the same reason.
    for _round in 0..6 {
        let cp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() { break; }
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.0);
            }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            let killed = w.shoot(g, &format!("guard {g}")) || { w.resupply("ammo retry"); w.shoot(g, &format!("guard {g}")) };
            let sim = w.scene.sim.as_ref().unwrap();
            if killed || sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0 { cleared.insert(g); }
        }
    }

    // ── the exit's condition: BOTH bodies on the plate fire the transition ─────────────────────
    // Sister (break-debris-shatter-2): the clearing phase stayed out of the plate's touch box —
    // the contact is inside it, so one crossing ends the level (can-fail: the pre-785 harness's
    // low-ammo resupply descended the stairwell through the box and fired the exit mid-clearing).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let box_ = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH && t.out == 0x10 && !t.tris.is_empty())
            .map(|t| (t.lo, t.hi)).expect("the AND-touch plate");
        let trail = w.trail_points();
        let from = trail_at_clear.min(trail.len());
        let crossed = trail[from..].iter().position(|p| (0..3).all(|ax| p[ax] >= box_.0[ax] && p[ax] <= box_.1[ax]));
        assert!(crossed.is_none(), "the clearing phase crossed the plate's touch box at trail +{crossed:?} ({:?}) — the game would end the level there", trail[from..].get(crossed.unwrap_or(0)));
    }
    // Bond walks the last leg onto the plate (the stairwell descent anchor walkreal-a uses).
    w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
    w.walk_nav_to(plate, "the plate", 1.2);
    for _ in 0..120 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let on_plate = sim.bond.pos[0] < -32.0 && sim.bond.pos[2] > 12.0;
    assert!(on_plate, "Bond must walk onto the LoadPointActivator plate (at {:?})", sim.bond.pos);
    if std::env::var_os("NF_ESCORT_DUMP").is_some() {
        eprintln!("[escort-dump] plate centre {:?}", walker::trigger_centre(&sim, walker::trigger_kind::TOUCH, 0x10));
        for (i, d) in sim.drones.list.iter().enumerate() {
            eprintln!("[escort-dump] drone {i}: skin {:#x} team {} state {:#x} corpse {} enabled {} started {} health {:.1} pos ({:.1},{:.1},{:.1})",
                d.skin, d.team, d.state, d.corpse, d.enabled, d.started, d.health, d.pos[0], d.pos[1], d.pos[2]);
        }
    }
    assert!(sim.drones.channels.on(0x10), "the AND plate must fire with Bond AND the contact inside [M Trigger_Hit 0x1b1fc8]");
    assert_eq!(sim.drones.level_exit, Some(0x0700_0002), "the exit fires to 07000002 under the game's condition");
    eprintln!("[escort] level_exit 07000002 at tick {}", w.total_ticks);
    for f in &w.report.falls { eprintln!("[escort] fall: {f}"); }
}
