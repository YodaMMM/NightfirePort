//! escort-lead-substates (835): the escort lead's armed-drone branch [M 0x15e698:30-38 — and the
//! same arm in LeadWait 0x15eda8:57-66, LeadPlayerInWay 0x15ea28:53-62, LeadBondCombat
//! 0x15f120:41-50]: `NDrone2_NearArmedDrone(10, flag 1)` → `NDrone2_FindRunToPoint` 0x153168
//! found → `+0x5A2 = 0x1f` and AllyGoToGoalPosition 0x28, which runs `MoveToGoalPosition` and
//! returns to the +0x5A2 state (LeadHide 0x1f) on arrival [M 0x15f938 case 3]; none found →
//! LeadHide 0x1f directly.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The real path on 07000001: the intro releases Mayhew (ch 0xB), Bond stands by him, and the
/// courtyard's started guards are the armed drones within 10. The escort (team 2 → run-to group 9
/// [M 0x153168:14-30]) takes the cover run 0x28 before he hides — the old stand-in cowered in place.
#[test]
fn the_escort_runs_to_cover_then_hides() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000u32 { sim.step_bond(Input::default(), false, false, false); if sim.drones.channels.on(0xb) { break; } }
    assert!(sim.drones.channels.on(0xb), "the intro releases the escort");
    let escort = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the skin-0x4F escort exists");
    let ep = sim.drones.list[escort].pos;
    sim.bond.pos = [ep[0], ep[1], ep[2] + 2.0];
    let mut seq: Vec<u8> = vec![sim.drones.list[escort].state];
    let mut run_start = None;
    let (mut ran, mut run_ticks) = (0.0f32, 0u32);
    let mut ret = 0u8;
    for _ in 0..600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let d = &sim.drones.list[escort];
        if seq.last() != Some(&d.state) { seq.push(d.state); }
        if d.state == state::GO_TO_GOAL_ALLY {
            let s = *run_start.get_or_insert(d.pos);
            ran = ran.max(((d.pos[0] - s[0]).powi(2) + (d.pos[2] - s[2]).powi(2)).sqrt());
            run_ticks += 1;
            ret = d.return_state;
        }
        if d.state == state::LEAD_HIDE && seq.contains(&state::GO_TO_GOAL_ALLY) { break; }
    }
    eprintln!("escort team {} states {:x?} ran {ran:.2} in {run_ticks} ticks", sim.drones.list[escort].team, seq);
    let gi = seq.iter().position(|&s| s == state::GO_TO_GOAL_ALLY).expect("FindRunToPoint found a cover point → AllyGoToGoalPosition 0x28");
    let hi = seq.iter().rposition(|&s| s == state::LEAD_HIDE).expect("the cover run returns to LeadHide");
    assert!(gi < hi, "0x28 then 0x1f: {seq:x?}");
    assert_eq!(ret, state::LEAD_HIDE, "+0x5A2 = 0x1f [M 0x15e698:34 / 0x15eda8:60]");
    // Sister: the run moves him toward the point (the stand-in cowered where he stood).
    assert!(ran > 0.5, "the cover run moves the escort (ran {ran:.2} in {run_ticks} ticks)");
}

/// Blasts reach the escort in his own states: LeadHide's case 9 → `HandleImpact(SM+4, 1)` [M
/// 0x15ec08] — the port had no ally arm, so a grenade at the hiding contact's feet did nothing.
/// Sister: the same blast 20 units away leaves him whole.
#[test]
fn a_blast_hurts_the_hiding_escort() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000u32 { sim.step_bond(Input::default(), false, false, false); if sim.drones.channels.on(0xb) { break; } }
    let escort = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the escort");
    let ep = sim.drones.list[escort].pos;
    sim.bond.pos = [ep[0], ep[1], ep[2] + 2.0];
    let mut hid = false;
    for _ in 0..300u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[escort].state == state::LEAD_HIDE { hid = true; break; }
    }
    assert!(hid, "the courtyard guards hide the escort");
    let blow = |sim: &mut nf_view::scene::Sim, off: f32| {
        let p = sim.drones.list[escort].pos;
        sim.drones.grenades.push(nf_game::grenade::Grenade {
            pos: [p[0] + off, p[1] + 0.5, p[2]], velocity: [0.0; 3], speed: 0.0, travelled: 0.0, range: 0.0,
            damage: 5.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0,
            smoke_popped: false, resting: false, stuck: false, seek_target: None, phase: None,
        });
        for _ in 0..30u32 { sim.step_bond(Input::default(), false, false, false); if sim.drones.grenades.is_empty() { break; } }
        assert!(sim.drones.grenades.is_empty(), "the zero-range grenade blows at once");
    };
    let h0 = sim.drones.list[escort].health;
    blow(sim, 20.0);
    assert_eq!(sim.drones.list[escort].health, h0, "sister: a blast 20 away leaves him whole");
    blow(sim, 0.0);
    assert!(sim.drones.list[escort].health < h0, "a blast at his feet hurts the hiding escort [M 0x15ec08 case 9] ({h0} → {})", sim.drones.list[escort].health);
}

/// After the cover run and the hide, the fight ends (every started hostile moved beyond the
/// 14-unit clear band [M 0x15ec08 case 3]) and the escort returns to the lead (AllyLead case 1
/// re-points +0x854 at the mission route +0x970 [M 0x15e698]) — and LEADS: he must move off the
/// cover point along his route. Sister: he reached the cover point first (0x28 ran).
#[test]
fn the_escort_resumes_the_route_after_cover() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000u32 { sim.step_bond(Input::default(), false, false, false); if sim.drones.channels.on(0xb) { break; } }
    let escort = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the escort");
    let ep = sim.drones.list[escort].pos;
    sim.bond.pos = [ep[0], ep[1], ep[2] + 2.0];
    let mut ran = false;
    for _ in 0..600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let st = sim.drones.list[escort].state;
        if st == state::GO_TO_GOAL_ALLY { ran = true; }
        if ran && st == state::LEAD_HIDE { break; }
    }
    assert!(ran, "the cover run happened");
    for g in sim.drones.list.iter_mut().filter(|g| g.team == 1 && g.started && !g.corpse) {
        g.pos = [g.pos[0] + 60.0, g.pos[1], g.pos[2] + 60.0];
    }
    let mut back = false;
    for _ in 0..300u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[escort].state == state::ALLY_LEAD { back = true; break; }
    }
    assert!(back, "the hide clears back to the lead");
    let p0 = sim.drones.list[escort].pos;
    let mut best = 0.0f32;
    for t in 0..400u32 {
        // Bond keeps close behind (inside the follow band, out of the in-way cone).
        let d = &sim.drones.list[escort];
        let (p, yaw) = (d.pos, d.yaw);
        sim.bond.pos = [p[0] - yaw.sin() * 3.0, p[1], p[2] - yaw.cos() * 3.0];
        sim.step_bond(Input::default(), false, false, false);
        let d = &sim.drones.list[escort];
        best = best.max(((d.pos[0] - p0[0]).powi(2) + (d.pos[2] - p0[2]).powi(2)).sqrt());
        if t % 50 == 0 { eprintln!("t{t} state {:#x} pos {:.2?} path {:.1}", d.state, d.pos, d.mission_path_dist); }
    }
    assert!(best > 2.0, "the escort leads on after the cover (moved {best:.2} from {p0:.2?})");
}
