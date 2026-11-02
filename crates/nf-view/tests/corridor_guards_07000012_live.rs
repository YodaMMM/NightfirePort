//! corridor-guards-12: the two corridor guards by the exit sign near the dropped case.
//!
//! At load the only enabled watching guards on Bond's corridor are drone 6 (13.09, 1.00, 18.55,
//! ALERT 0x05) and drone 7 (12.98, 1.00, 16.43, PLAY_SCRIPT 0x03) [L enemies_07000012_live
//! dump_every_drone_at_load]; Bond starts at (22.49, 1.09, 19.16) facing yaw −1.56 [D the level's
//! start placement], i.e. straight at them down the corridor. The symptom: they stay bunched
//! at the far end and "walk into nothing" instead of coming down the corridor.
//!
//! Test 1 (log-only repro): tick from the start spot, then wake through the game's OWN alarm path
//! (msg 0x1e [M DroneVision_EnemyAlerts 0x177c18:113-114 — +0x4F8 |= 0x4000000|0x40000000,
//! SetState(0x56)]) and log both guards' state/pos/anim/goal/route_blocked every 30 ticks for
//! 600 ticks. Diagnostic — the pin follows once the blocker is named.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn drone_report(sim: &nf_view::scene::Sim, i: usize) -> String {
    let d = &sim.drones.list[i];
    format!(
        "drone#{i} skin {:#08x} state {:#04x} mode {:#04x} anim {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} route_blocked {} watching {} hp {:.1}",
        d.skin, d.state, d.mode, d.anim.state, d.pos[0], d.pos[1], d.pos[2],
        d.goal.map(|g| (g[0], g[1], g[2])), d.route_blocked, d.watching, d.health)
}

#[test]
fn repro_logs_the_two_corridor_guards() {
    let iso = iso();
    let mut w = nf_view::load(&iso, "07000012.bin", false).map(Walker::new).expect("load 07000012");
    assert!(w.wait_enabled(6000, "level start"), "the level never enabled");
    w.wait(120); // the first two seconds settle (scripts, spawners)

    // Which drones are the two corridor guards? Log every present living drone near the corridor.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.present && !d.corpse && d.health > 0.0
                && d.pos[0] > 5.0 && d.pos[0] < 30.0 && d.pos[2] > 10.0 && d.pos[2] < 25.0 {
                eprintln!("AT-START {}", drone_report(sim, i));
            }
        }
        // LOS probes: the guards' sight path to Bond's start head. The port's sight consumes
        // world.line_of_sight [S stand-in for AINetwork_TestRayCels + Collide_LineOfSight — see
        // team2_opponent_picks]. Probe from each guard's head to Bond's head (22.49, ~2.6, 19.16),
        // then bisect the corridor east of the guards to find WHERE a hit sits.
        for by in [1.4f32, 1.62, 1.9, 2.2, 2.6] {
            let head = [22.49f32, by, 19.16];
            for i in [6usize, 7] {
                let a = sim.drones.list[i].head();
                let seen = sim.world.line_of_sight(a, head, nf_game::world::LOS_GUARD_VISION);
                let hit = sim.world.line_of_sight_hit(a, head, nf_game::world::LOS_GUARD_VISION);
                eprintln!("LOS y={:.2} guard#{i} -> bond {:?}: seen {} hit_frac {:?}", by, head, seen, hit);
            }
        }
        // A dense x-walk at guard-6 head height, z 18.55 → 19.16 (the corridor axis): which x hits?
        for y in [1.4f32, 1.62, 2.0, 2.4] {
            for x in [13usize, 14, 15, 16, 17, 18, 19, 20, 21, 22] {
                let z = |xi: f32| 18.55 + (19.16 - 18.55) * (xi - 13.0) / 9.0;
                let a = [x as f32, y, z(x as f32)];
                let b = [(x + 1) as f32, y, z((x + 1) as f32)];
                let hit = sim.world.line_of_sight_hit(a, b, nf_game::world::LOS_GUARD_VISION);
                if hit.is_some() { eprintln!("LOS-SEG y={:.2} x {}..{}: HIT frac {:?}", y, x, x + 1, hit); }
            }
        }
        // The visibility factors: every kind-0 AI volume containing Bond's start and the guards'
        // positions — visibility_factor multiplies them into every guard's sight [M
        // DroneVision_OpponentVisibility 0x175af0 x +0x22C; the port's aivolume::visibility_factor].
        {
            let pts: [(&str, [f32; 3]); 3] = [
                ("bond-start", [22.49, 1.09, 19.16]),
                ("guard6", sim.drones.list[6].pos),
                ("guard7", sim.drones.list[7].pos),
            ];
            for (name, pt) in pts {
                for (vi, v) in sim.volumes.iter().enumerate() {
                    if v.kind == 0 && v.contains(pt) {
                        eprintln!("VIS volume#{vi} placement {} factor {:.3} half {:?} centre ({:.2},{:.2},{:.2}) contains {}",
                            v.placement, v.factor, v.half, v.m[3][0], v.m[3][1], v.m[3][2], name);
                    }
                }
            }
            eprintln!("VIS total at bond-start = {}", nf_game::aivolume::visibility_factor(&sim.volumes, [22.49f32, 1.09, 19.16], false));
            // The sight gates per guard: the stats the visibility score reads.
            for i in [6usize, 7] {
                let dr = &sim.drones.list[i];
                eprintln!("SIGHT drone#{i} sight_range {:.2} half_fov {:.3} alert {:.2} alert_bits {:x} engage {:.2} los {} seen {} dist {:.2} yaw {:.2} yaw_to_opp {:.2} prop46 {}",
                    dr.sight_range, dr.half_fov, dr.alert, dr.alert_bits, dr.engage_range, dr.los, dr.seen, dr.dist_to_player[0], dr.yaw, dr.yaw_to_opp, dr.props.get(0x46).is_some());
                eprintln!("SCRIPT drone#{i} start_script {:?} placement {} goal {:?}", dr.start_script, dr.placement, dr.goal);
            }
        }
        // Nav: which AI path holds each guard and Bond's start, and can the guards' path reach
        // Bond's start? (The port's route machinery is per NavPath; a missing cross-path hop is an
        // invisible wall to a route walker.)
        let g6 = sim.drones.list[6].pos;
        let g7 = sim.drones.list[7].pos;
        let bond = [22.49f32, 1.09, 19.16];
        let p6 = sim.nav.nav_path_for(g6);
        let p7 = sim.nav.nav_path_for(g7);
        let pb = sim.nav.nav_path_for(bond);
        eprintln!("NAV path guard6 {:?} guard7 {:?} bond {:?}", p6, p7, pb);
        // The nav boundary walls near the corridor (the BLINK walls — an over-tall one is a
        // see-through blocker): print every wall with an endpoint inside the corridor box.
        for (wi, w2) in sim.nav.walls.iter().enumerate() {
            let inbox = |p: [f32; 3]| p[0] > 11.0 && p[0] < 26.0 && p[2] > 11.0 && p[2] < 26.0;
            if inbox(w2.a) || inbox(w2.b) {
                eprintln!("WALL #{wi} a({:.2},{:.2},{:.2}) h {:.2}  b({:.2},{:.2},{:.2}) h {:.2} flags {:#x}",
                    w2.a[0], w2.a[1], w2.a[2], w2.ha, w2.b[0], w2.b[1], w2.b[2], w2.hb, w2.flags);
            }
        }
    }

    // Phase 1: Bond WALKS from the spawn down the corridor toward the guards (the real input
    // path — the reported scenario). Log both guards every 20 ticks and Bond's pos.
    for t in 0..360u32 {
        let mut inp = Input::default();
        // The spawn yaw is −1.56 [D the start placement — the task's spot]; forward walks him
        // straight down the corridor toward the guards.
        inp.forward = 1.0;
        w.tick(inp, false);
        if t % 20 == 19 {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            eprintln!("P1 t={:3} bond ({:.2},{:.2},{:.2})", t, bp[0], bp[1], bp[2]);
            for i in [6usize, 7] {
                eprintln!("P1 t={:3} {}", t, drone_report(sim, i));
            }
        }
    }

    // Phase 2: the game's own alarm path [M 0x177c18:113-114] on every live watching guard.
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
                d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                    && matches!(d.state, nf_game::drone::state::IDLE | nf_game::drone::state::ALERT)
            };
            if wake {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, nf_game::drone::msg::ALARM);
                eprintln!("ALARMED drone#{i}");
            }
        }
    }
    for t in 0..300u32 {
        w.tick(Input::default(), false);
        if t % 30 == 29 {
            let sim = w.scene.sim.as_ref().unwrap();
            for i in [6usize, 7] {
                eprintln!("P2 t={:3} {}", t, drone_report(sim, i));
            }
        }
    }

    // End state: the full near-corridor picture again.
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.present && !d.corpse && d.health > 0.0
            && d.pos[0] > 5.0 && d.pos[0] < 30.0 && d.pos[2] > 10.0 && d.pos[2] < 25.0 {
            eprintln!("AT-END {}", drone_report(sim, i));
        }
    }
}
