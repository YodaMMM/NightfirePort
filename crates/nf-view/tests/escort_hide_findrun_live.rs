//! escort-hide-findrun: the 07000001 escort contact's armed-hostile arm through the REAL path —
//! `nf_view::load` + `Walker` ticks, no state pokes. The game's rule [M 0x15e698:93-102, same
//! shape in 0x15eda8:57-66 / 0x15ea28:53-62 / 0x15f120:41-51]: while the lead machine's states
//! see an armed, started team-1 drone within 10 (`NDrone2_NearArmedDrone(10, flag=1)` 0x177420,
//! gated on the drone's own prop 0x50), `NDrone2_FindRunToPoint` 0x153168 runs — team 2 asks the
//! group-9 AI points (`run_to_point_group`), `NDrone2_ClosestAIPoint` 0x151b18 picks the cheapest
//! on the drone's nav path, and a hit means `+0x5A2 = 0x1f` (LeadHide) + SetState(0x28
//! AllyGoToGoalPosition 0x15f938) — the escort RUNS to the point and only then cowers; a miss
//! means SetState(0x1f) — the cower in place. 07000001 carries group-9 points around the intro
//! courtyard [D placements 0x(286..292), param 0 = 9], so the found arm is the live one here.
//! The old port arm ran the hide half only (the FindRunToPoint stand-in) — these pins fail on it.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

use nf_game::drone::state;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn escort_contact_runs_to_the_registry_point_then_hides() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    // The intro cut-scene writes ch 0xB (the contact's enable) — the same wait escort_follow runs.
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro must write ch 0xB");
    w.wait_enabled(6000, "the intro tail");

    let ci = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    // Bond stands off (the spawn) — the armed peers are the courtyard guards, not him
    // [M 0x177420:63-90: the team-1 armed-drone scan only].
    let mut found_at = None;
    let mut goal_observed = None;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        if d.state == state::GO_TO_GOAL_ALLY {
            found_at = Some(w.total_ticks);
            goal_observed = d.goal;
            break;
        }
        // The stand-in never left the hide arm: if he cowers before any GoToGoal the pin fails.
        if d.state == state::LEAD_HIDE && w.total_ticks > 300 { break; }
    }
    assert_eq!(found_at.map(|_| ()), Some(()),
        "the armed-hostile arm must run the contact to a group-9 point (state 0x28 [M 0x15e698:96-97]) — the old hide-only stand-in cowers instead");
    let (goal, cpos, cpath) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        (d.goal.expect("the found arm's goal"), d.pos, d.nav_path.flatten())
    };
    // The goal is the registry's answer for this drone (team 2 → group 9, its owner +0x144).
    let owner = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.props.get(0x11, &sim.drones.list[ci].props) as i32
    };
    let want = nf_game::aipoint::closest(&w.scene.sim.as_ref().unwrap().ai_points, &w.scene.sim.as_ref().unwrap().nav, &w.scene.sim.as_ref().unwrap().world, cpath, cpos, 9, owner)
        .map(|i| w.scene.sim.as_ref().unwrap().ai_points[i].pos);
    assert_eq!(goal, want.expect("01 carries eligible group-9 points [D placements]"),
        "the run point is `NDrone2_ClosestAIPoint`'s cheapest group-9 point [M 0x153168:38 + 0x151b18]");
    assert_eq!({ let sim = w.scene.sim.as_ref().unwrap(); sim.drones.list[ci].return_state }, state::LEAD_HIDE,
        "`+0x5A2 = 0x1f` before the 0x28 entry [M 0x15e698:96]");

    // He arrives and THEN cowers: 0x28's arrival returns to `+0x5A2` [M 0x15f938 case 3], and
    // LeadHide holds while an armed peer stays within 14 [M 0x15ec08 case 3].
    let mut hiding_at = None;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        let d = &w.scene.sim.as_ref().unwrap().drones.list[ci];
        if d.state == state::LEAD_HIDE { hiding_at = Some(w.total_ticks); break; }
        if d.state == state::ALLY_LEAD { break; } // the peer cleared before the run ended — re-loop
    }
    assert_eq!(hiding_at.map(|_| ()), Some(()),
        "the contact hides where the game sends him: at the run point (0x28 arrival → +0x5A2 [M 0x15f938:26-30])");
}

/// The sister: the lead's neighbouring behaviour survives the new arm — with the guards cleared
/// through the real fire path the escort LEADS again (the escort_follow choreography's opening,
/// pinned here against the 0x28 rewrite: Lead's route leg still runs and the exit plate opens).
#[test]
fn escort_still_leads_after_the_hide_arm() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro must write ch 0xB");
    w.wait_enabled(6000, "the intro tail");
    w.wait(200);
    let ci = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    // Clear the armed peers within 14 through the real fire path (escort_follow's opener).
    let mut cleared = std::collections::HashSet::new();
    for _round in 0..12 {
        let (cp, cstate) = { let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[ci].pos, sim.drones.list[ci].state) };
        let guards: Vec<usize> = { let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect() };
        if guards.is_empty() && cstate != state::LEAD_HIDE && cstate != state::GO_TO_GOAL_ALLY { break; }
        for g in guards {
            let (gpos, los_clear) = { let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none()) };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear { w.walk_nav_to(gpos, &format!("approach guard {g}"), 3.0); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            let killed = w.shoot(g, &format!("guard {g}")) || { w.resupply("ammo retry"); w.shoot(g, &format!("guard {g}")) };
            let sim = w.scene.sim.as_ref().unwrap();
            if killed || sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0 { cleared.insert(g); }
        }
    }
    // With no armed peer within 14 the hide exits [M 0x15ec08 case 3] and the lead leads: the
    // mission-route leg runs (AllyLead / AllyLeadWait on the −9/−7 bands) [M 0x15e698 case 3].
    let mut led = false;
    for _ in 0..1200u32 {
        w.tick(Default::default(), false);
        let d = &w.scene.sim.as_ref().unwrap().drones.list[ci];
        if matches!(d.state, state::ALLY_LEAD | state::ALLY_LEAD_WAIT) { led = true; break; }
    }
    assert!(led, "the escort resumes the lead once the peers are gone [M 0x15ec08:23-29 + 0x15e698 case 3]");
}
