//! walkreal probe 12f: the alarm-charge wedge near (22.97, 1.06, 36.21) — level-12-alarm-guard-walls.
//! Symptom (the driver, frame 1391 of a real play): after the alarm, guards running toward Bond
//! stop and press empty space near (22.97, 1.06, 36.21); nothing is drawn there.
//! Run: NF_ISO=… cargo test -p nf-view --test walkreal_probe12f -- --ignored --nocapture --test-threads=1
//! Log-only diagnostic; the pass/fail pin lives in walkreal_07000012_live.rs once the blocker is named.

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

#[test]
#[ignore = "probe: dumps the wedge region; run explicitly"]
fn dump_the_wedge_region() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120);

    let sim = w.scene.sim.as_ref().unwrap();
    let near = |p: [f32; 3], r: f32| ((p[0] - WEDGE[0]).powi(2) + (p[2] - WEDGE[2]).powi(2)).sqrt() < r;

    println!("==== DRONES at load (all present living) ====");
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.present && !d.corpse && d.health > 0.0 {
            println!("DRONE {}", drone_report(sim, i));
        }
    }

    println!("==== SPAWNERS (all) ====");
    for (i, s) in sim.drones.spawners.iter().enumerate() {
        println!("SPAWNER#{i} {:?}", s);
    }

    println!("==== NAV paths: count {} ====", sim.nav.paths.len());
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        let near_nodes = p.nodes.iter().filter(|n| near(n.pos, 18.0)).count();
        println!("PATH#{pi} name {:?} kind {:#x} nodes {} links {} (within 18u of wedge: {})",
            p.name, p.kind, p.nodes.len(), p.links.len(), near_nodes);
    }

    println!("==== NAV walls within 16u of the wedge ====");
    for (wi, wall) in sim.nav.walls.iter().enumerate() {
        if near(wall.a, 16.0) || near(wall.b, 16.0) {
            println!("WALL #{wi} a({:.2},{:.2},{:.2}) h {:.2} b({:.2},{:.2},{:.2}) h {:.2} flags {:#x}",
                wall.a[0], wall.a[1], wall.a[2], wall.ha, wall.b[0], wall.b[1], wall.b[2], wall.hb, wall.flags);
        }
    }

    println!("==== NAV nodes within 16u of the wedge ====");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        for (ni, n) in p.nodes.iter().enumerate() {
            if near(n.pos, 16.0) {
                println!("NODE path#{pi}[{ni}] ({:.2},{:.2},{:.2}) flags {:#x}", n.pos[0], n.pos[1], n.pos[2], n.flags);
            }
        }
    }

    println!("==== STATIC COLLISION scan around the wedge ====");
    let world = &sim.world;
    // Vertical: what floor is below/above each grid point? (The west wing row answers whether
    // the north wing is continuous from the west-room guards (x -33, z 33-35) to the cafeteria.)
    for z in [33.0f32, 34.5, 35.5, 36.2] {
        for x in [-40.0f32, -34.0, -28.0, -22.0, -16.0, -10.0, -4.0, 2.0, 8.0, 13.0, 16.0, 19.0, 22.97, 27.0] {
            let floor = world.floor_below([x, 4.0, z], 12.0);
            println!("VERT x {x:6.2} z {z:6.2}: floor_below(4.0,12) {:?}", floor.map(|f| (f * 1000.0).round() / 1000.0));
        }
    }
    // Horizontal rays THROUGH the wedge from eight directions at three heights.
    for y in [0.6f32, 1.06, 1.6] {
        for (dx, dz, name) in [(1.0f32, 0.0, "east→"), (-1.0, 0.0, "west→"), (0.0, 1.0, "north→"), (0.0, -1.0, "south→")] {
            let a = [WEDGE[0] - dx * 8.0, y, WEDGE[2] - dz * 8.0];
            let b = [WEDGE[0] + dx * 8.0, y, WEDGE[2] + dz * 8.0];
            let hit = world.line_of_sight_hit(a, b, nf_game::world::LOS_PROBE_8);
            println!("HOR y {y:4.2} {name} wedge: {}", match hit {
                None => "CLEAR".to_string(),
                Some(t) => format!("HIT t {:.2}", t),
            });
        }
    }

    println!("==== DOORS / TRIGGERS / SS within 16u ====");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if near(d.pos, 16.0) { println!("DOOR[{i}] plc {} pos {:.2?} state {:#x}", d.placement, d.pos, d.state); }
    }
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let mid = [(t.lo[0] + t.hi[0]) / 2.0, (t.lo[1] + t.hi[1]) / 2.0, (t.lo[2] + t.hi[2]) / 2.0];
        if near(mid, 16.0) { println!("TRIG[{i}] plc {} kind {:#x} out {:#x} lo {:.2?} hi {:.2?}", t.placement, t.kind, t.out, t.lo, t.hi); }
    }

    println!("==== CEL bounds with any corner in z 26..46 ====");
    for (ci, c) in w.scene.cels.iter().enumerate() {
        if c.max[2] > 26.0 && c.min[2] < 46.0 {
            println!("CEL#{ci} min {:.2?} max {:.2?}", c.min, c.max);
        }
    }
}

#[test]
#[ignore = "probe: the alarm-charge repro with per-tick wedge tracking; run explicitly"]
fn repro_the_alarm_charge_wedges() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120);

    // The game's own alarm path on EVERY present+enabled drone — the port's alarm rebroadcast
    // walks the whole list with no state filter [M 0x177c18 EnemyAlerts case 0x1e → +0x4F8 |=
    // 0x4000000|0x40000000, SetState(0x56); the port's tick loop sends msg::ALARM to every
    // present+enabled drone while switch_channels[0x96]].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let difficulty = sim.drones.difficulty;
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
            ai_points: &sim.ai_points,
        };
        let frame = sim.drones.frame;
        let mut woken = 0;
        for i in 0..sim.drones.list.len() {
            let wake = {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.team != 2 && d.health > 0.0
            };
            if wake {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, nf_game::drone::msg::ALARM);
                woken += 1;
            }
        }
        eprintln!("ALARMED {woken} guards (every present+enabled, the port's rebroadcast list)");
    }

    // Bond WALKS the route south the way a player does (forward held, the real input path) while
    // the alarmed guards charge. Track every guard per 20 ticks: a wedge = the horizontal position
    // moving < 0.02 u over 60 consecutive ticks while the state is chasing/attacking.
    // The player's route waypoints (tower2::walk_12 + walk_12_exit legs — the real walking path),
    // steered with the real input path per tick so the wedge tracker sees every tick.
    let route: [(f32, f32, f32); 16] = [
        (14.0, 1.1, 6.0), (10.0, 1.1, 2.0), (9.3, 1.1, -0.6), (6.2, 1.1, -0.86),
        (1.0, 1.1, 4.0), (-1.55, 0.5, 7.2), (1.1, 0.9, 7.4), (4.0, 1.1, 5.0),
        (6.0, 1.1, -0.86), (8.4, 1.1, -0.6), (10.75, 1.1, -0.5), (13.25, 1.1, 3.25),
        (21.0, 1.1, 3.5), (24.25, 1.1, 2.75), (24.0, 1.1, -1.0), (23.5, 1.1, -4.5),
    ];
    let mut wp_i = 0usize;
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
    let n = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.len()
    };
    let mut last: Vec<Option<([f32; 3], u8)>> = vec![None; n];
    let mut still: Vec<u32> = vec![0; n];
    let mut wedged: Vec<Option<(u32, [f32; 3], u8)>> = vec![None; n];
    for t in 0..2400u32 {
        let mut inp = Input::default();
        // Steer Bond along the route waypoints (the real input path: turn + forward).
        if wp_i < route.len() {
            let target = [route[wp_i].0, route[wp_i].1, route[wp_i].2];
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if ((p[0] - target[0]).powi(2) + (p[2] - target[2]).powi(2)).sqrt() < 1.2 {
                wp_i += 1;
            } else {
                let err = walker::wrap_pi((target[0] - p[0]).atan2(target[2] - p[2]) - sim.bond.yaw);
                inp.turn = err.clamp(-0.06, 0.06);
                inp.forward = 1.0;
            }
        }
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
                    last[i] = Some((d.pos, d.state));
                }
                _ => { last[i] = Some((d.pos, d.state)); still[i] = 0; }
            }
        }
        if t % 300 == 299 {
            eprintln!("t={t} bond ({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
            for i in 0..n {
                let d = &sim.drones.list[i];
                if d.present && !d.corpse && d.health > 0.0 {
                    eprintln!("  t={t} {}", drone_report(sim, i));
                }
            }
        }
    }
    eprintln!("==== WEDGES (>=60 ticks stationary while chasing) ====");
    for i in 0..n {
        if let Some((t, p, st)) = wedged[i] {
            eprintln!("WEDGED drone#{i} at t={t} pos ({:.2},{:.2},{:.2}) state {st:#04x}", p[0], p[1], p[2]);
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let _ = sim; // final picture already logged above
}

#[test]
#[ignore = "probe: Bond walks NORTH into the cafeteria (the z 32-36.6 room) and everyone charges; run explicitly"]
fn repro_bond_in_the_cafeteria() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120);

    // The alarm on every present+enabled drone (the port's rebroadcast list — the game's own
    // broadcast reaches every class != {1,3,0xf,0x47..0x48} drone [M 0x172128:106-142], and
    // WAIT_SWITCH ignores msg 0x1e [M 0x15b960:40-41 — only 0x0e ENABLE acts], so the
    // enabled filter and the class filter agree on who can charge).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let difficulty = sim.drones.difficulty;
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
            ai_points: &sim.ai_points,
        };
        let frame = sim.drones.frame;
        for i in 0..sim.drones.list.len() {
            let wake = {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.team != 2 && d.health > 0.0
            };
            if wake {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, nf_game::drone::msg::ALARM);
            }
        }
        // The stress mix: every WAIT_SWITCH guard also gets the game's own switch-channel enable
        // [M 0x15b960:21-25 — channel on → NDrone2_Enable(1) + the start state; the game's
        // broadcast delivers by class, no enabled gate — M 0x172128:106-142 — and WAIT_SWITCH
        // guards carry enabled=false in the port, so the filter must not require it], then the
        // alarm again on everyone enabled.
        for i in 0..sim.drones.list.len() {
            let wake = {
                let d = &sim.drones.list[i];
                d.present && !d.corpse && d.team != 2 && d.health > 0.0
                    && d.state == nf_game::drone::state::WAIT_SWITCH
            };
            if wake {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, nf_game::drone::msg::ENABLE);
            }
        }
        for i in 0..sim.drones.list.len() {
            let wake = {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.team != 2 && d.health > 0.0
            };
            if wake {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, nf_game::drone::msg::ALARM);
            }
        }
    }

    // Bond walks NORTH through the nav graph (the corridor's jog needs the routed path),
    // THROUGH the corridor touch trigger plc 30 (volume 11.10-15.72, 26.28-30.90, out ch 0x5
    // [D] — the writer that enables the two west-wing north-room guards, placements 21/22,
    // wait ch 5 [D params]), then into the cafeteria; then he STANDS at (22.5, 35.0) while the
    // charge runs — the tracker watches every tick after the arrival.
    w.walk_nav_to([13.4, 1.1, 28.5], "through the corridor touch #30", 0.9);
    {
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("ch 0x5 on {} after the touch", s.drones.channels.on(0x5));
    }
    w.walk_nav_to([22.5, 1.1, 35.0], "into the cafeteria", 1.0);
    {
        let s = w.scene.sim.as_ref().unwrap();
        eprintln!("BOND IN PLACE at ({:.2},{:.2},{:.2})", s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]);
    }
    let wp_i = 0usize; // unused marker for the loop below
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
    for t in 0..1500u32 {
        let inp = Input::default(); // Bond stands his ground in the cafeteria
        let _ = wp_i;
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
    eprintln!("==== WEDGES ====");
    for i in 0..n {
        if let Some((t, p, st)) = wedged[i] {
            eprintln!("WEDGED drone#{i} at t={t} pos ({:.2},{:.2},{:.2}) state {st:#04x}", p[0], p[1], p[2]);
        }
    }
}
