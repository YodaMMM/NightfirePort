//! walkreal probe 12g: the alarm-charge wedges through the REAL alarm path —
//! level-12-invisible-wall-spots. Difference from probe12f: no hand-sent msg::ALARM. The alarm is
//! raised the way the game's own press lands: switch channel 0x96 goes on through the switch layer
//! (`Drones::set_channel`, the write `Switch_GetSwitchChannel` 0x1b0430's consumer performs at
//! `DroneFunc_SetMissionFailReason` 0x13e560 case 0x10), and the port's own rebroadcast loop
//! delivers msg 0x1E to every drone every 30 frames [M drone.rs 18601-18602, `Drone_InitComms`].
//! Bond then walks the real route (no position writes) through the corridor touch plc 30 into the
//! cafeteria and STANDS at the reported wedge, while every guard's route is logged and every wedge
//! gets a stop-cause sample (goal, route flags, nav walls, static segment to goal, floors, doors).
//!
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12g -- --ignored --nocapture --test-threads=1

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The reported wedge spot.
const WEDGE: [f32; 3] = [22.97, 1.06, 36.21];

fn drone_report(sim: &nf_view::scene::Sim, i: usize) -> String {
    let d = &sim.drones.list[i];
    format!(
        "drone#{i} plc {} skin {:#08x} state {:#04x} anim {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} route_blocked {} watching {} hp {:.1} bounded {}",
        d.placement, d.skin, d.state, d.anim.state, d.pos[0], d.pos[1], d.pos[2],
        d.goal.map(|g| (g[0], g[1], g[2])), d.route_blocked, d.watching, d.health, d.bounded)
}

/// The stop-cause sample at a wedge tick: everything that could be holding the drone.
fn stop_cause(sim: &nf_view::scene::Sim, i: usize) {
    let d = &sim.drones.list[i];
    let p = d.pos;
    eprintln!("---- STOP CAUSE drone#{i} ----");
    eprintln!("  {}", drone_report(sim, i));
    // 1. Nav walls within 3 u (the BLINK boundaries — the invisible-wall suspect).
    for (wi, wall) in sim.nav.walls.iter().enumerate() {
        let near = |q: [f32; 3]| ((q[0] - p[0]).powi(2) + (q[2] - p[2]).powi(2)).sqrt() < 3.0;
        if near(wall.a) || near(wall.b) {
            eprintln!("  NAVWALL#{wi} a({:.2},{:.2},{:.2}) h {:.2} b({:.2},{:.2},{:.2}) h {:.2} flags {:#x}",
                wall.a[0], wall.a[1], wall.a[2], wall.ha, wall.b[0], wall.b[1], wall.b[2], wall.hb, wall.flags);
        }
    }
    // 2. Static world between the drone and its goal (collision in the way).
    if let Some(g) = d.goal {
        let hit = sim.world.line_of_sight_hit([p[0], p[1] + 1.0, p[2]], [g[0], g[1] + 1.0, g[2]], nf_game::world::LOS_PROBE_8);
        eprintln!("  SEGMENT drone→goal (eye +1.0): {}", match hit { None => "CLEAR".to_string(), Some(t) => format!("HIT t {:.2}", t) });
        let fb_goal = sim.world.floor_below(g, 12.0);
        eprintln!("  FLOOR at goal {:?}, at drone {:?}", fb_goal.map(|f| (f * 1000.0).round() / 1000.0),
            sim.world.floor_below(p, 12.0).map(|f| (f * 1000.0).round() / 1000.0));
    }
    // 3. Doors within 6 u and their states.
    for (di, door) in sim.drones.doors.list.iter().enumerate() {
        if ((door.pos[0] - p[0]).powi(2) + (door.pos[2] - p[2]).powi(2)).sqrt() < 6.0 {
            eprintln!("  DOOR[{di}] plc {} pos {:.2?} state {:#x}", door.placement, door.pos, door.state);
        }
    }
    // 4. Bond's position and the sight line to him.
    let bp = sim.bond.pos;
    let hit = sim.world.line_of_sight_hit([p[0], p[1] + 1.5, p[2]], [bp[0], bp[1] + 1.5, bp[2]], nf_game::world::LOS_PROBE_8);
    eprintln!("  BOND at ({:.2},{:.2},{:.2}) sight drone→bond: {}", bp[0], bp[1], bp[2],
        match hit { None => "CLEAR".to_string(), Some(t) => format!("HIT t {:.2}", t) });
}

#[test]
#[ignore = "probe: the real-path alarm charge with per-tick wedge tracking + stop causes; run explicitly"]
fn repro_the_real_alarm_charge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 1).unwrap());
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120);

    // The level's alarm points (aipoint group 8) near the wedge — the RunToAlarm goals.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("==== AI POINTS within 14u of the wedge ====");
        for (pi, ap) in sim.ai_points.iter().enumerate() {
            let d2 = (ap.pos[0] - WEDGE[0]).powi(2) + (ap.pos[2] - WEDGE[2]).powi(2);
            if d2 < 196.0 {
                eprintln!("AIPOINT#{pi} group {} pos ({:.2},{:.2},{:.2}) yaw {:.3} d {:.2}",
                    ap.group, ap.pos[0], ap.pos[1], ap.pos[2],
                    f32::from_bits(ap.yaw_bits), d2.sqrt());
            }
        }
    }

    // The REAL alarm path, minus only the guard-walk-to-panel leg: switch channel 0x96 rises
    // through the switch layer (the write the game's alarm press performs [M 0x13e560 case 0x10
    // ← Switch_GetSwitchChannel 0x1b0430]), then the port's own rebroadcast loop [M 18601] does
    // the delivery. Log the 0x96 edge.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.set_channel(0x96);
    }

    // Bond walks the real route north through the corridor touch plc 30 into the cafeteria and
    // stands at the wedge's room. Route legs = the corridor jog needs the nav graph.
    w.walk_nav_to([13.4, 1.1, 28.5], "through the corridor touch #30", 0.9);
    w.walk_nav_to([22.97, 1.06, 36.21], "onto the reported wedge", 0.6);
    {
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("ALARM ch 0x96 on {} — BOND IN PLACE at ({:.2},{:.2},{:.2})",
            s.drones.channels.on(0x96), s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]);
    }

    let chase = |st: u8| matches!(st,
        nf_game::drone::state::ATTACK
        | nf_game::drone::state::COMBAT
        | nf_game::drone::state::COMBAT_OUT_OF_RANGE
        | nf_game::drone::state::COMBAT_NEW_SIGHTING
        | nf_game::drone::state::GO_TO_GOAL
        | nf_game::kiko::RUN_TO_ALARM
        | nf_game::drone::state::AIM_STAND
        | nf_game::drone::state::AIM_STAND_FIRE
        | nf_game::drone::state::AIM_CROUCH
        | nf_game::drone::state::AIM_CROUCH_FIRE);
    let n = w.scene.sim.as_ref().unwrap().drones.list.len();
    let mut last: Vec<Option<([f32; 3], u8)>> = vec![None; n];
    let mut still: Vec<u32> = vec![0; n];
    let mut wedged: Vec<Option<(u32, [f32; 3], u8)>> = vec![None; n];
    let mut cause_done: Vec<bool> = vec![false; n];
    for t in 0..1500u32 {
        let inp = Input::default(); // Bond stands his ground in the cafeteria
        w.tick(inp, false);
        let sim = w.scene.sim.as_ref().unwrap();
        for i in 0..n {
            let d = &sim.drones.list[i];
            if !d.present || d.corpse || d.health <= 0.0 { continue; }
            match last[i] {
                Some((p0, st0)) if chase(d.state) => {
                    let moved = ((d.pos[0] - p0[0]).powi(2) + (d.pos[2] - p0[2]).powi(2)).sqrt();
                    if moved < 0.02 && chase(st0) { still[i] += 1; } else { still[i] = 0; }
                    if still[i] == 60 && wedged[i].is_none() {
                        wedged[i] = Some((t, d.pos, d.state));
                        eprintln!("WEDGE t={t} {}", drone_report(sim, i));
                    }
                    // The stop cause on the first and the 300th still tick.
                    if still[i] > 0 && (still[i] == 60 || still[i] == 300) && !cause_done[i] {
                        stop_cause(sim, i);
                        if still[i] == 300 { cause_done[i] = true; }
                    }
                    last[i] = Some((d.pos, d.state));
                }
                _ => { last[i] = Some((d.pos, d.state)); still[i] = 0; }
            }
        }
        if t % 150 == 149 {
            eprintln!("t={t} bond ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
            for i in 0..n {
                let d = &sim.drones.list[i];
                if d.present && !d.corpse && d.health > 0.0
                    && (d.state != nf_game::drone::state::WAIT_SWITCH || d.pos[2] > 20.0) {
                    eprintln!("  t={t} {}", drone_report(sim, i));
                }
            }
        }
    }
    eprintln!("==== WEDGES (>=60 ticks stationary while chasing) ====");
    for i in 0..n {
        if let Some((t, p, st)) = wedged[i] {
            let near_wedge = ((p[0] - WEDGE[0]).powi(2) + (p[2] - WEDGE[2]).powi(2)).sqrt() < 3.0;
            eprintln!("WEDGED drone#{i} at t={t} pos ({:.2},{:.2},{:.2}) state {st:#04x}{}",
                p[0], p[1], p[2], if near_wedge { "  <== AT THE REPORTED WEDGE" } else { "" });
        }
    }
}
