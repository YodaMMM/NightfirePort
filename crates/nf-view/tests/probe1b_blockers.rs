//! probe1b-blockers (level-1b-interactables-walk): the three gap-ignored blockers measured on the
//! loaded disc level, assertion-free, run with --nocapture:
//! A) spawner #187's parsed fields + the min_dist screen around Drake's template spot;
//! B) the hub guard (drone #0): live pos/mode, hit boxes, and every hub-floor flank's segment
//!    under the BODY rules vs the BULLET rules (the game's shot ray drops pass-flagged hits
//!    [M Collide_Intersect 0x1ebaa8 disasm 0x1ebff0-0x1ec010 — HITTEST bit 0x8]);
//! C) the kind-6 bonus pickup's walk-over take with per-tick dumps.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
fn p3(v: [f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", v[0], v[1], v[2]) }

#[test]
fn probe_blockers() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry_medals(&iso, "0700001b.bin", false, None, 1, u64::MAX).expect("load 1b"); // kind-6 pickups need the gold medal [M 0x18dda0:93]
    // --- A) spawners + grouped drones ---
    {
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("=== A: spawners {} ===", sim.drones.spawners.len());
        for (i, sp) in sim.drones.spawners.iter().enumerate() {
            eprintln!("spawner[{i}] placement={} mode={} group={} max_alive={} waves={} start={} stop={} done={} kill={} min_dist={:.2} immediate={} templates={:?} active={} spawned={:?}",
                sp.placement, sp.mode, sp.group, sp.max_alive, sp.waves, sp.start, sp.stop, sp.done,
                sp.kill, sp.min_dist, sp.immediate, sp.templates, sp.active, sp.spawned);
        }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.group != 0 {
                eprintln!("drone[{i}] skin={:#x} group={} present={} enabled={} at={} health={:.1} mode={:#x} state={:#x}",
                    d.skin, d.group, d.present, d.enabled, p3(d.pos), d.health, d.mode, d.state);
            }
        }
    }
    // --- B) the hub guard ---
    let (guard_boxes, guard_pos) = {
        let sim = scene.sim.as_ref().unwrap();
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
            ai_points: &sim.ai_points,
        };
        let g = &sim.drones.list[0];
        eprintln!("=== B: hub guard ===");
        eprintln!("guard[0] skin={:#x} at={} present={} enabled={} corpse={} health={:.1} mode={:#x} state={:#x} team={}",
            g.skin, p3(g.pos), g.present, g.enabled, g.corpse, g.health, g.mode, g.state, g.team);
        let boxes: Vec<[f32; 3]> = g.hit_boxes(&data).into_iter().map(|b| [b.m[3][0], b.m[3][1], b.m[3][2]]).collect();
        eprintln!("guard hit boxes {}:", boxes.len());
        for (k, c) in boxes.iter().enumerate() { eprintln!("  box[{k}] {}", p3(*c)); }
        eprintln!("floor_below guard (+6): {:?}", sim.world.floor_below(g.pos, 6.0));
        eprintln!("floor_below hub floor at (2.86,-1.0,0) (+4): {:?}", sim.world.floor_below([2.86, -1.0, 0.0], 4.0));
        (boxes, g.pos)
    };
    {
        let sim = scene.sim.as_ref().unwrap();
        // The hub-floor flanks (the walker's 6 + the pit rim) — eye height 1.6 above the feet.
        let flanks: [(&str, [f32; 3]); 8] = [
            ("east rim", [12.41, -1.9, 2.40]),
            ("west rim", [-6.0, -1.9, 0.0]),
            ("north rim", [2.86, -1.9, 6.0]),
            ("south rim", [2.86, -1.9, -6.0]),
            ("east deck", [8.0, -1.9, 0.0]),
            ("spawn side", [18.0, -1.9, 3.0]),
            ("near pit", [5.0, -1.9, 1.0]),
            ("above floor centre", [2.86, -1.9, 0.5]),
        ];
        for (label, from) in flanks {
            let eye = [from[0], from[1] + 1.6, from[2]];
            for (k, c) in guard_boxes.iter().enumerate() {
                let body = sim.world.segment(eye, *c);
                let bullet = sim.world.segment_bullet(eye, *c);
                if body.is_some() != bullet.is_some() || k == 0 {
                    eprintln!("flank {label} eye {} box[{k}] {}: body {:?} bullet {:?}{}",
                        p3(eye), p3(*c), body, bullet,
                        if body.is_some() != bullet.is_some() { "  <-- DIFFERS" } else { "" });
                }
            }
        }
        // The same check from straight above the pit rim (looking down into the pit).
        for y in [-1.0, 0.0, 1.0, 2.0, 3.0] {
            let eye = [2.86, y, 3.0];
            for (k, c) in guard_boxes.iter().enumerate() {
                let body = sim.world.segment(eye, *c);
                let bullet = sim.world.segment_bullet(eye, *c);
                eprintln!("high eye y{y} box[{k}]: body {:?} bullet {:?}", body, bullet);
            }
        }
    }
    // --- C) the kind-6 bonus pickup walk-over ---
    {
        eprintln!("=== C: the bonus pickups ===");
        let sim = scene.sim.as_ref().unwrap();
        for (i, p) in sim.drones.pickups.iter().enumerate() {
            if p.kind == 6 {
                eprintln!("pickup[{i}] kind=6 weapon={:#x} state={} hidden={} gone={} m3={} start={} radius={:.2}",
                    p.weapon, p.state, p.hidden, p.gone, p3([p.m[3][0], p.m[3][1], p.m[3][2]]), p3(p.start), p.bounds.radius);
            }
        }
    }
    // Free Bond (the arrival), then variant A: the ignored test's exact teleport + walk.
    let mut sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
    sim.bond.substate = nf_game::player::SS_WALK;
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    let (px, py, pz) = {
        let p = &sim.drones.pickups.iter().position(|p| p.kind == 6).map(|i| &sim.drones.pickups[i]);
        let p = p.unwrap();
        (p.m[3][0], p.m[3][1], p.m[3][2])
    };
    eprintln!("variant A: the ignored test's teleport [x+0.6, y+1.2, z] = {}", p3([px + 0.6, py + 1.2, pz]));
    sim.bond.pos = [px + 0.6, py + 1.2, pz];
    sim.bond.vel = [0.0; 3];
    let mut took = false;
    for t in 0..400u32 {
        let b = sim.bond.pos;
        let d = [px - b[0], py - b[1], pz - b[2]];
        let yaw = d[0].atan2(d[2]);
        sim.bond.yaw = yaw;
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = &sim.drones.pickups.iter().position(|p| p.kind == 6).map(|i| &sim.drones.pickups[i]).unwrap();
        took |= p.state == 2 || p.hidden || p.gone;
        if t % 20 == 0 || took {
            let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            eprintln!("t{t} bond {} grounded={} substate={:#x} floor {:?} dist {dist:.2} r {:.2} pickup state={} hidden={} gone={}",
                p3(sim.bond.pos), sim.bond.grounded, sim.bond.substate,
                sim.world.floor_below(sim.bond.pos, 6.0), p.bounds.radius + 1.1, p.state, p.hidden, p.gone);
        }
        if took { break; }
    }
    eprintln!("variant A took = {took}");
    // Variant B: stand still 0.3 above the pickup's spot (drop onto it).
    if !took {
        eprintln!("variant B: teleport straight above (y+0.3), no input");
        sim.bond.pos = [px, py + 0.3, pz];
        sim.bond.vel = [0.0; 3];
        for t in 0..240u32 {
            sim.step_bond(Input::default(), false, false, false);
            let p = &sim.drones.pickups.iter().position(|p| p.kind == 6).map(|i| &sim.drones.pickups[i]).unwrap();
            took |= p.state == 2 || p.hidden || p.gone;
            if t % 40 == 0 || took {
                let d = [px - sim.bond.pos[0], py - sim.bond.pos[1], pz - sim.bond.pos[2]];
                let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                eprintln!("t{t} bond {} grounded={} dist {dist:.2} r {:.2} state={} hidden={}",
                    p3(sim.bond.pos), sim.bond.grounded, p.bounds.radius + 1.1, p.state, p.hidden);
            }
            if took { break; }
        }
        eprintln!("variant B took = {took}");
    }
    // Variant C: walk slowly from 0.4 west along the deck.
    if !took {
        eprintln!("variant C: teleport 0.4 west, walk forward");
        sim.bond.pos = [px - 0.4, py + 0.1, pz];
        sim.bond.vel = [0.0; 3];
        for t in 0..240u32 {
            let b = sim.bond.pos;
            let d = [px - b[0], 0.0, pz - b[2]];
            sim.bond.yaw = d[0].atan2(d[2]);
            sim.step_bond(Input { forward: 0.4, ..Default::default() }, false, false, false);
            let p = &sim.drones.pickups.iter().position(|p| p.kind == 6).map(|i| &sim.drones.pickups[i]).unwrap();
            took |= p.state == 2 || p.hidden || p.gone;
            if t % 40 == 0 || took {
                let dist = ((px - sim.bond.pos[0]).powi(2) + (py - sim.bond.pos[1]).powi(2) + (pz - sim.bond.pos[2]).powi(2)).sqrt();
                eprintln!("t{t} bond {} grounded={} dist {dist:.2} r {:.2} state={} hidden={}",
                    p3(sim.bond.pos), sim.bond.grounded, p.bounds.radius + 1.1, p.state, p.hidden);
            }
            if took { break; }
        }
        eprintln!("variant C took = {took}");
    }
    // --- A2) the spawner's min_dist screen: ch 0x0a driven with Bond near Drake's spot vs far ---
    {
        eprintln!("=== A2: the spawner gate vs Bond's distance ===");
        // Reload fresh so the spawner state is clean.
        let mut scene2 = nf_view::load(&iso, "0700001b.bin", false).expect("reload");
        let s2 = scene2.sim.as_mut().unwrap();
        s2.disabled = false;
        for _ in 0..2 { s2.step_bond(Input::default(), false, false, false); }
        s2.bond.substate = nf_game::player::SS_WALK;
        // The green drake-fight test's 3000-tick free before the drive (any load-time script
        // holding the NPCs must end first).
        for _ in 0..3000 { s2.step_bond(Input::default(), false, false, false); }
        let drake_spot = s2.drones.list.iter().find(|d| d.group == 7).map(|d| d.pos).unwrap_or([0.0; 3]);
        eprintln!("Drake template spot {}", p3(drake_spot));
        // Far first: Bond at the spawn platform (the green test's shape). scene2 is dropped
        // before scene3 loads (two live scenes OOM this box).
        let far;
        {
            let frame = s2.drones.frame;
            s2.drones.channels.set(0x0a, 1, frame);
            let mut spawned = false;
            for t in 0..600u32 {
                s2.step_bond(Input::default(), false, false, false);
                if s2.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) { spawned = true; eprintln!("FAR: Drake spawned at t{t}"); break; }
            }
            far = spawned;
            if !spawned { eprintln!("FAR: Drake did NOT spawn in 600 ticks"); }
        }
        drop(scene2);
        if far {
            // Remove Drake and walk Bond onto the arena floor, then drive the gate again (needs a
            // fresh spawner — reload instead).
            let mut scene3 = nf_view::load(&iso, "0700001b.bin", false).expect("reload 2");
            let s3 = scene3.sim.as_mut().unwrap();
            s3.disabled = false;
            for _ in 0..2 { s3.step_bond(Input::default(), false, false, false); }
            s3.bond.substate = nf_game::player::SS_WALK;
            for _ in 0..3000 { s3.step_bond(Input::default(), false, false, false); }
            s3.bond.pos = [drake_spot[0] + 1.5, drake_spot[1], drake_spot[2]];
            s3.bond.vel = [0.0; 3];
            let frame = s3.drones.frame;
            s3.drones.channels.set(0x0a, 1, frame);
            let mut near = false;
            for t in 0..600u32 {
                s3.step_bond(Input::default(), false, false, false);
                if s3.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) { near = true; eprintln!("NEAR: Drake spawned at t{t} (no min_dist screen?!)"); break; }
            }
            if !near { eprintln!("NEAR (1.5 u): Drake did NOT spawn in 600 ticks (the min_dist screen [M 0x137cb0:30])"); }
        }
    }
}

/// The hub guard seen from the WALKER's real flank deck (y ~6.02 ring) and the walk-reachable
/// deck spots: body vs bullet segment per box — where is a BODY-clear part (the flight stops on
/// pass-flagged nodes [L guard cover trace], so only a body-clear flank can land the shot)?
#[test]
fn probe_guard_deck_flanks() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
    sim.bond.substate = nf_game::player::SS_WALK;
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let guard_pos = sim.drones.list[0].pos;
    let boxes: Vec<[f32; 3]> = sim.drones.list[0].hit_boxes(&data).into_iter()
        .map(|b| [b.m[3][0], b.m[3][1], b.m[3][2]]).collect();
    eprintln!("guard at {} boxes {}", p3(guard_pos), boxes.len());
    // The walker's flank ring on the deck (from the test log) + a dense ring around the pit.
    let eyes: [(&str, [f32; 3]); 14] = [
        ("flank2 (6.37,6.02,1.87)", [6.37, 6.02, 1.87]),
        ("flank1 (4.98,6.02,-3.51)", [4.98, 6.02, -3.51]),
        ("flank6 (12.41,3.78,2.38)", [12.41, 3.78, 2.38]),
        ("deck east", [7.0, 6.02, 0.0]),
        ("deck ne", [6.0, 6.02, 3.5]),
        ("deck se", [6.0, 6.02, -3.5]),
        ("rim north", [2.86, 6.02, 5.5]),
        ("rim south", [2.86, 6.02, -5.5]),
        ("deck above", [2.86, 6.02, 0.0]),
        ("low east", [7.0, 3.78, 0.0]),
        ("low ne", [6.0, 3.78, 3.0]),
        ("low se", [6.0, 3.78, -3.0]),
        ("low north", [2.86, 3.78, 5.0]),
        ("low south", [2.86, 3.78, -5.0]),
    ];
    for (label, from) in eyes {
        let eye = [from[0], from[1] + 1.6, from[2]];
        let mut body_clear = Vec::new();
        let mut bullet_clear = Vec::new();
        for (k, c) in boxes.iter().enumerate() {
            if sim.world.segment(eye, *c).is_none() { body_clear.push(k); }
            if sim.world.segment_bullet(eye, *c).is_none() { bullet_clear.push(k); }
        }
        eprintln!("{label} eye {} -> body-clear boxes {:?} bullet-clear {:?}", p3(eye), body_clear, bullet_clear);
    }
    // The deck's own edge: floor_below at the ring spots (is the deck there at all?).
    for (label, from) in eyes.iter().take(6) {
        eprintln!("floor_below {} ({}): {:?}", label, p3(*from), sim.world.floor_below(*from, 6.0));
    }
}
#[test]
fn probe_pickup_watch() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    eprintln!("=== pickup watch ===");
    let n = sim.drones.pickups.len();
    let mut last: Vec<(u8, bool, bool, u16, u16)> = sim.drones.pickups.iter()
        .map(|p| (p.state, p.hidden, p.gone, p.lifetime, p.respawn)).collect();
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        eprintln!("pickup[{i}] kind={} state={} hidden={} gone={} lifetime={} respawn={} switch_ch={} at={}",
            p.kind, p.state, p.hidden, p.gone, p.lifetime, p.respawn, p.switch_channel, p3(p.start));
    }
    for t in 0..4000u32 {
        sim.step_bond(Input::default(), false, false, false);
        for i in 0..n {
            let p = &sim.drones.pickups[i];
            let now = (p.state, p.hidden, p.gone, p.lifetime, p.respawn);
            if now != last[i] {
                eprintln!("t{t}: pickup[{i}] kind={} -> state={} hidden={} gone={} lifetime={} respawn_ticks={}",
                    p.kind, p.state, p.hidden, p.gone, p.lifetime, p.respawn_ticks);
                last[i] = now;
            }
        }
    }
}

/// Standing floors + guard exposure on the main floor around the pedestal.
#[test]
fn probe_guard_ground_spots() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
    sim.bond.substate = nf_game::player::SS_WALK;
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let boxes: Vec<[f32; 3]> = sim.drones.list[0].hit_boxes(&data).into_iter()
        .map(|b| [b.m[3][0], b.m[3][1], b.m[3][2]]).collect();
    for spot in [[2.86f32, -1.9, 5.0], [2.86, -1.9, -5.0], [0.0, -1.9, 3.0], [5.5, -1.9, 3.0],
                 [5.5, -1.9, -3.0], [-2.0, -1.9, 0.0], [2.86, -1.9, 4.0], [2.86, -1.9, -4.0],
                 [8.0, -1.9, 3.13], [16.0, -1.9, 3.13]] {
        let floor = sim.world.floor_below(spot, 6.0);
        let eye = [spot[0], spot[1] + 1.6, spot[2]];
        let clear: Vec<usize> = (0..boxes.len()).filter(|&k| sim.world.segment(eye, boxes[k]).is_none()).collect();
        eprintln!("spot {} floor {:?} body-clear {:?}", p3(spot), floor, clear);
    }
}

/// Spawn Drake far, dump his death fields, kill him with the walker's gun, dump the corpse state.
#[test]
fn probe_drake_death() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
    sim.bond.substate = nf_game::player::SS_WALK;
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); }
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x0a, 1, frame);
    let mut idx = None;
    for t in 0..600u32 {
        sim.step_bond(Input::default(), false, false, false);
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) { idx = Some(i); eprintln!("Drake spawned t{t}"); break; }
    }
    let i = idx.expect("spawn");
    {
        let d = &sim.drones.list[i];
        eprintln!("drake idx {i} mode={:#x} state={:#x} health={:.1} death_channel={} special_death={:#x} team={} enabled={}",
            d.mode, d.state, d.health, d.death_channel, d.special_death, d.team, d.enabled);
    }
    // Walk Bond near and shoot him through the real path (the walker's tick_fire at the exposed box).
    sim.bond.pos = [-4.0, -1.9, 0.2];
    sim.bond.vel = [0.0; 3];
    for t in 0..2000u32 {
        // aim + fire at Drake's torso via the sim's own aim (use the player's step with fire toward him)
        let dp = sim.drones.list[i].pos;
        let bp = sim.bond.pos;
        let dx = dp[0] - bp[0]; let dz = dp[2] - bp[2];
        let dy = dp[1] + 0.3 - (bp[1] + sim.bond.eye);
        let desired = dx.atan2(dz);
        let err = desired - sim.bond.yaw;
        let want_pitch = (dy / dxz_max(bp, dp)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let on = err.abs() < 0.03 && perr.abs() < 0.04 && t % 16 == 0;
        sim.step_bond(Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, on, false, false);
        let d = &sim.drones.list[i];
        if t % 100 == 0 { eprintln!("t{t} bond {} drake state={:#x} health={:.1} corpse={} ch0e={}", p3(sim.bond.pos), d.state, d.health, d.corpse, sim.drones.channels.on(0x0e)); }
        if d.corpse || d.health <= 0.0 { eprintln!("drake down at t{t}"); break; }
    }
    for t in 0..200u32 { sim.step_bond(Input::default(), false, false, false); }
    let d = &sim.drones.list[i];
    eprintln!("AFTER: drake state={:#x} corpse={} ch0e={} ch0b={} mission_status={}", d.state, d.corpse, sim.drones.channels.on(0x0e), sim.drones.channels.on(0x0b), sim.drones.mission.status());
}

fn dxz_max(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt().max(0.5) }
