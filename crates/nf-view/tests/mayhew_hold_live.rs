//! mayhew-hold: 07000001's escort holds at the route's ally-wait node instead of leading through
//! the door ahead of Bond, and the hold releases through the game's own chain.
//!
//! The chain, every link on disc data or the decompile:
//! 1. The kind-8 "Mayhew Route" (nav path 0 [D the level census]) carries node 33
//!    (2.91, 5.74, 7.57) with flag 0x20 — the escort's hold. `NDrone2_ReachedDestNode`
//!    (0x153358:85-94) parks a drone reaching a 0x20 node in `DSTATE_AllyLeadMissionWait` (0x21)
//!    while `switch_channels[0x99]` is off, saving the return state in +0x5A2. The real-game
//!    recording tests/traces/01-doors.trace holds the contact in 0x21 at (2.94, 7.22) for 1262
//!    frames — west of door #1's plane (nodes 5/6 at z 10.17/11.78, the shoji at (7.92, 11.14)).
//! 2. The release: the TOUCH_ONCE plate at (6.79, 6.04, 8.91) raises ch 0x32 and the OUT mux
//!    (0xEE, out 0x32, list [0x33]) mirrors it into ch 0x33 every tick [M Trigger_Update
//!    0x1b1a78 case 5]; SP #276's start channel is its param 6 out = 0x33 [D placement;
//!    M SP_Update 0x1dbda8 state 0 `flags & mask`], so the plate starts script 0x06000086, whose
//!    stream-9 event 0xd writes ch 0x0e [D the lib dump] — the courtyard guards 6/7/11 at
//!    (5.42,14.44)/(9.05,17.01)/(5.93,17.86) enable. Their death channels 0x11/0x12/0x13
//!    (`DroneFunc_SetDeathChannel` 0x13e660) feed the AND mux pl 318 (0xEC, out 0x99) [D], and
//!    0x21's UPDATE (0x15efc8 case 3) returns the escort to 0x1d AllyLead when 0x99 rises.
//!
//! The test drives the real input path only (the walker: Bond's `Input`), and pins:
//! - the escort holds at the node (0x21, within 1.5 of (2.91, 7.57)) and never crosses door #1's
//!   plane while held (z < 10.0, the door still closed) with Bond walking back at the corridor start;
//! - walking over the plate raises 0x33 and enables the ambush guards;
//! - killing them raises 0x99 and the escort resumes the lead.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The ally-wait node [D nav path 0 node 33] and door #1's plane [D the shoji at (7.92, 11.14)].
const HOLD: [f32; 2] = [2.91, 7.57];
const DOOR1_Z: f32 = 10.0;
/// The ambush guards' positions [D placement census] and the plate [D trigger #2].
const AMBUSH: [[f32; 2]; 3] = [[5.42, 14.44], [9.05, 17.01], [5.93, 17.86]];
const PLATE: [f32; 2] = [6.79, 8.91];

/// Open any shut door near the Bond→target line (the real player's E), as fix_npc_doors' helper.
fn open_doors_on_the_way(w: &mut Walker, target: [f32; 3]) {
    let (bpos, doors): ([f32; 3], Vec<(usize, [f32; 3])>) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.drones.doors.list.iter().enumerate()
            .filter(|(_, d)| d.state & 0x12 != 0)
            .map(|(i, d)| (i, d.pos)).collect())
    };
    let mut doors = doors;
    doors.sort_by(|a, b| walker::dxz(a.1, bpos).total_cmp(&walker::dxz(b.1, bpos)));
    for (i, dp) in doors.into_iter().take(2) {
        if w.scene.sim.as_ref().unwrap().drones.doors.list[i].state & 0x12 == 0 { continue; }
        let (px, pz, ax, az, bx, bz) = (dp[0], dp[2], bpos[0], bpos[2], target[0], target[2]);
        let (dx, dz) = (bx - ax, bz - az);
        let l2 = dx * dx + dz * dz;
        let t = if l2 > 0.0 { ((px - ax) * dx + (pz - az) * dz) / l2 } else { 0.0 };
        let t = t.clamp(0.0, 1.0);
        let (qx, qz) = (ax + dx * t, az + dz * t);
        if ((px - qx).powi(2) + (pz - qz).powi(2)).sqrt() > 2.5 { continue; }
        w.walk_nav_to(dp, "to the shut door", 2.2);
        w.walk_to(Wp { at: dp, act: Act::Use { use_ticks: 120 }, arrive: 3.0 }, "open the way");
    }
}

#[test]
fn the_escort_holds_at_the_ally_wait_node_until_the_ambush_clears() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // The intro's contact-enable channel, then the tail that re-places Bond (the escort tests' setup).
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro must write ch 0xB");
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);

    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);

    // The escort hides while an armed started guard stands within 14 [M 0x15ec08]; clear the
    // meet-up guards through the real fire path (the escort tests' choreography).
    let mut cleared = std::collections::HashSet::new();
    for _round in 0..12 {
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, sim.drones.list[ci].pos) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() { break; }
        for g in guards {
            cleared.insert(g);
            let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 { w.walk_nav_to(gpos, "approach guard", 3.0); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            if !w.shoot(g, "meet-up guard") { cleared.remove(&g); }
        }
    }

    // ── the hold: Bond follows at walk pace; the escort must park at the ally-wait node ─────────
    w.keep_alive.push(ci);
    let mut parked_at = None;
    let mut hold_pos = [0.0f32; 3];
    'follow: for _round in 0..8 {
        let start = w.total_ticks;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[ci];
            let (dstate, dpos) = (d.state, d.pos);
            if dstate == state::LEAD_MISSION_WAIT {
                hold_pos = dpos;
                parked_at = Some(w.total_ticks);
                break 'follow;
            }
            assert!(dpos[2] < DOOR1_Z || walker::dxz(dpos, [HOLD[0], 0.0, HOLD[1]]) < 3.0,
                "the escort passed door #1's plane ({:.2}) before the hold: state {:#x} at ({:.1},{:.1},{:.1})",
                DOOR1_Z, dstate, dpos[0], dpos[1], dpos[2]);
            if w.total_ticks - start > 5400 { break; } // stall → the round loop closes in
            let bond = sim.bond.pos;
            let desired = (dpos[0] - bond[0]).atan2(dpos[2] - bond[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let dy = dpos[1] - bond[1];
            let d3 = (walker::dxz(bond, dpos).powi(2) + dy * dy).sqrt();
            let input = Input {
                turn: err.clamp(-0.07, 0.07),
                forward: if d3 > 10.0 { 1.0 } else if d3 > 4.5 { 0.55 } else { 0.0 },
                ..Default::default()
            };
            drop(sim);
            w.tick(input, false);
        }
        // stalled short of the hold: close the gap and retry
        let cpos = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
        w.walk_nav_to(cpos, "close on the contact", 4.0);
    }
    let parked = parked_at.expect("the escort never parked in AllyLeadMissionWait 0x21");
    eprintln!("[hold] parked 0x21 at ({:.2},{:.2},{:.2}) tick {parked}", hold_pos[0], hold_pos[1], hold_pos[2]);
    // The hold point IS the ally-wait node [D node 33 (2.91, 7.57)].
    assert!(walker::dxz(hold_pos, [HOLD[0], 0.0, HOLD[1]]) < 1.5,
        "the escort parked away from the ally-wait node: ({:.2},{:.2},{:.2})",
        hold_pos[0], hold_pos[1], hold_pos[2]);
    assert!(hold_pos[2] < DOOR1_Z, "the escort held past door #1's plane at z {:.2}", hold_pos[2]);

    // He STAYS while Bond stands back at the corridor start (the game's park: 1262 frames [L trace]).
    let bond_back = w.scene.sim.as_ref().unwrap().bond.pos;
    for _ in 0..300 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        assert_eq!(d.state, state::LEAD_MISSION_WAIT, "the escort left the hold with ch 0x99 off");
        assert!(d.pos[2] < DOOR1_Z, "the escort crept past door #1's plane while held");
        assert!(sim.drones.doors.list[1].state & 0x12 != 0,
            "door #1 opened while the escort was held — he worked the door ahead of Bond");
    }
    eprintln!("[hold] held 300 ticks; bond stands ({:.2},{:.2},{:.2})", bond_back[0], bond_back[1], bond_back[2]);

    // ── the release: Bond walks over the plate, the ambush arms, the guards die, ch 0x99 ─────────
    w.walk_nav_to([PLATE[0], 0.0, PLATE[1]], "onto the corridor plate", 0.4);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let t2 = &sim.drones.triggers[2];
        eprintln!("[hold] after the walk bond ({:.2},{:.2},{:.2}) ch33 {} | trigger #2 kind {:#x} out {:#x} mask {:#x} lock {:#x} gone {} tris {} bounds ({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.channels.on(0x33),
            t2.kind, t2.out, t2.mask, t2.lock, t2.gone, t2.tris.len(), t2.lo[0], t2.lo[1], t2.lo[2], t2.hi[0], t2.hi[1], t2.hi[2]);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x33),
        "the corridor plate never raised ch 0x33 (SP #276's start channel)");
    eprintln!("[hold] plate fired ch 0x33");

    // The ambush guards are enabled by 06000086's ch 0x0e; kill them through the real fire path.
    for _round in 0..10 {
        let ambush: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                d.team == 1 && d.started && !d.corpse && d.enabled && AMBUSH.iter()
                    .any(|a| walker::dxz(d.pos, [a[0], 0.0, a[1]]) < 2.5)
            }).collect()
        };
        if ambush.is_empty() { break; }
        for g in ambush {
            let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
            open_doors_on_the_way(&mut w, gpos);
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 { w.walk_nav_to(gpos, "approach ambush guard", 3.0); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            w.shoot(g, "ambush guard");
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let ch = &sim.drones.channels;
        eprintln!("[hold] after kills: ch0e {} ch11 {} ch12 {} ch13 {} ch99 {}", ch.on(0xe), ch.on(0x11), ch.on(0x12), ch.on(0x13), ch.on(0x99));
        for (i, d) in sim.drones.list.iter().enumerate() {
            if AMBUSH.iter().any(|a| walker::dxz(d.pos, [a[0], 0.0, a[1]]) < 2.5) || (d.team == 1 && d.started && d.enabled && !d.corpse) {
                eprintln!("[hold] drone #{i} ({:.1},{:.1},{:.1}) team {} state {:#x} started {} enabled {} corpse {} hp {:.0}",
                    d.pos[0], d.pos[1], d.pos[2], d.team, d.state, d.started, d.enabled, d.corpse, d.health);
            }
        }
    }
    // The AND mux (trigger #10: inputs [0x11,0x12,0x13] → out 0x99) updates in the triggers'
    // pass — give the sim a few ticks after the last death before reading its output.
    w.wait(5);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x99),
        "the ambush guards died but ch 0x99 never rose (the AND mux)");

    // The hold releases: 0x21's UPDATE (0x15efc8 case 3) returns the escort to AllyLead 0x1d.
    // [M 0x15efc8 case 3]: 0x21's UPDATE returns to 0x1d AllyLead when the channel rises; the
    // lead's own bands then govern (Bond off-route parks 0x20 again) — the proven behaviour is
    // that the HOLD ITSELF releases, so pin the 0x21 exit, not the Lead state byte.
    let mut resumed = false;
    for _ in 0..600 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.list[ci].state != state::LEAD_MISSION_WAIT { resumed = true; break; }
    }
    assert!(resumed, "the escort never left the hold after ch 0x99 rose");
    eprintln!("[hold] released on ch 0x99 — the escort leads again");
}
