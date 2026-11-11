//! port-map-bullets / homing-copter-2: the `Bullet_heatseekable` 0x12f248 class-0 arm on the
//! real level — Bond's AT-600 Scorpion 'Heatseeker' (weapon 0x2e, record `+0x6C` = 0x32200, the
//! homing bit [L ee_start_05.bin @0x2bf150 + 0x2e·0x10C]) locks a COPTER body. The disc reaches
//! the arm here: 07000012 carries the kind-0 copter placement #707 (64.44, 4.30, 19.53) [D] and
//! grants the 0x2e through pickups #2aa/#2f0 [D the placement sweep] — the old code's homing
//! scan knew only the drones (the class-2 arm), so the round flew straight past the copter and
//! the lock never happened (the red run).
//!
//! homing-copter-2: the equip goes through the LEVEL'S OWN grant path — the placed pickup
//! (`Pickup_Update`'s take test on the real tick → `Pickup_Handler` 0x18e650 →
//! `Player_EquipWeapon`'s loaded-model check), not the harness give; the first-person model is
//! pre-merged by `load_weapons(&[0x2e])`, the in-memory state the disc's own equip requires
//! [M the +0xDC hashtable read]. The body's world position reads the copter matrix's
//! TRANSLATION ROW (row 3 — the SP_SetPosRot rides obj+0x90 [M Copter_Move 0x17d628:115]); the
//! first cut read column 3 of rows 0-2, (0,0,0) for every rigid matrix, which both hid the
//! pose from this harness ("the copter never poses") and steered every lock at the world
//! origin. The dormant half pins the `+0xFC` gate: a copter still waiting on its gate channel
//! is SP_HideObj'd [M Copter_Create 0x17d340:73-76], never view-added, and its in-view counter
//! decays to 0 [M 0x134630:16 / 0x1336f8:242-246 / 0x1e62d8:71] — `Bullet_heatseekable`'s
//! class-0 arm reads it [M 0x12f248:29] — so no round may lock it.
//!
//! Evidence tags: [M] decompile, [D] disc bytes, [L] EE snapshot, [S] suggested. The teleports
//! are the documented stand-in transit (the take, the equip, the switch, the aim and the fire
//! all run through the real input/tick paths); the 0700000b pairing (copter #148 + pickup #15d)
//! is the same arm's second level.

use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn aim_at(sim: &mut Sim, target: [f32; 3], fire: bool) {
    sim.drones.health.health = sim.drones.health.health.max(100.0);
    let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
    let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
    let desired = dx.atan2(dz);
    let input = nf_game::player::Input {
        turn: (desired - sim.bond.yaw).clamp(-0.08, 0.08),
        look: ((dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch).clamp(-0.08, 0.08),
        aiming: true,
        fire,
        ..Default::default()
    };
    sim.step_bond(input, fire, false, false);
}

fn copter_pose(sim: &nf_view::scene::Sim) -> [f32; 3] {
    sim.drones.copters[0].pos() // row 3 — the translation (see the module doc)
}

fn dot_to(b: &nf_game::drone::PlayerBullet, t: [f32; 3]) -> f32 {
    let to = [t[0] - b.pos[0], t[1] - b.pos[1], t[2] - b.pos[2]];
    let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
    (to[0] * b.dir[0] + to[1] * b.dir[1] + to[2] * b.dir[2]) / l
}

/// The level's own grant path: the placed Scorpion pickup (#2aa/#2f0 [D the placement sweep])
/// taken through the real `Pickup_Update` take test — the teleport is the documented stand-in
/// for the walk-over, the take itself is the game's (`Pickup_Handler` 0x18e650 →
/// `Player_EquipWeapon`, the loaded-model gate included).
fn take_and_draw_scorpion(scene: &mut nf_view::scene::Scene) {
    let (at, taken0) = {
        let sim = scene.sim.as_ref().unwrap();
        let k = sim.drones.pickups.iter()
            .position(|p| p.kind == nf_game::pickup::kind::WEAPON && p.weapon == 0x2e && !p.gone)
            .expect("07000012 carries the 0x2e pickup (#2aa/#2f0) [D the placement sweep]");
        let p = &sim.drones.pickups[k];
        ([p.m[3][0], p.m[3][1] + 0.4, p.m[3][2]], sim.drones.taken.len())
    };
    eprintln!("scorpion pickup at ({:.2},{:.2},{:.2})", at[0], at[1], at[2]);
    let mut taken = false;
    // The take test is radial; a placed pickup against geometry is still taken from an open
    // side (the fix_player2 approach pattern).
    let mut approaches = vec![at];
    for (dx, dz) in [(2.0f32, 0.0), (-2.0, 0.0), (0.0, 2.0), (0.0, -2.0)] {
        approaches.push([at[0] + dx, at[1], at[2] + dz]);
    }
    for a in approaches {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, a, 0.0);
        }
        for _ in 0..30 {
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.taken.len() > taken0 || sim.drones.pickups.iter().all(|p| p.weapon != 0x2e || p.gone) { taken = true; break; }
        }
        if taken { break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let base = sim.weapons.base(0x2e as usize) as usize;
        assert!(sim.drones.inventory.owned.get(base).copied().unwrap_or(false),
            "the level's own pickup must grant the Scorpion (owned[base {base:#x}]) — the real \
             Pickup_Handler 0x18e650 → Player_EquipWeapon path");
    }
    eprintln!("scorpion taken through the level's pickup path");
    // The real switch request (`M_CONTROL+0x63` → `Player_WeaponSelect` 0x1a2cc0); the fp model
    // is in memory (the merge), so the draw runs — the disc's own equip state.
    scene.sim.as_mut().unwrap().gun.switch_request = 0x2e;
    let mut equipped = false;
    for _ in 0..400 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().gun.weapon == 0x2e && scene.sim.as_ref().unwrap().gun.switch_state == 0 { equipped = true; break; }
    }
    assert!(equipped, "the Scorpion (0x2e) must equip; weapon now {:#x}", scene.sim.as_ref().unwrap().gun.weapon);
}

/// Drive the copter's gate channel (the game's own activation — `Copter_Update`'s head [M
/// 0x17dbb8:91-97]). The body's pose is the placement matrix from create (row 3), so "posed"
/// cannot be read off a non-zero translation — the gate's effect is the SHOWN state, and the
/// flight follows on the next ticks (case 0 → +0xF4 = 1 [M :136], case 1's Copter_Move).
fn raise_gate(scene: &mut nf_view::scene::Scene) {
    let gate = scene.sim.as_ref().unwrap().drones.copters[0].gate;
    assert!(gate != 0, "the disc's copter placement #707 is gate-channel'd [D the placement sweep]");
    let f = scene.sim.as_ref().unwrap().drones.frame;
    scene.sim.as_mut().unwrap().drones.channels.set(gate as u32, 1, f);
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert!(scene.sim.as_ref().unwrap().drones.copters[0].body_shown(),
        "after the gate's first ON tick the SP is unhidden [M 0x17dbb8:95-96] — the port's \
         shown flag must read true (the +0xFC counter rides the view-add from here)");
}

#[test]
/// (homing-copter-2 measured the self-block this test used to carry an ignore for; fixed in
/// homing-copter-3: the homing LOS excludes the candidate's own SP stream shapes — the port of
/// `Collide_LineOfSight`'s subject-argument rule [M Bullet_homing 0x12f358:92-93].)
fn the_scorpions_homing_round_locks_the_copter_body_on_07000012() {
    let mut scene = nf_view::load_weapons(&iso(), "07000012.bin", false, &[0x2e])
        .expect("load 07000012 with the AT-600 Scorpion");
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters.len(), 1, "07000012 carries exactly one kind-0 copter [D #707]");
    let ci = 0usize;
    take_and_draw_scorpion(&mut scene);

    // The body's rest pose is route key 0 — the placement translation (64.44, 4.30, 19.53)
    // [D #707, the path's own first frame]. The firing spots below are standable floors whose
    // eye has a clear BULLET segment to the route's opening stretch (the terrain probe: the
    // patrol loop itself is sealed behind walls from every floor, but keys 0-250 from the
    // (20, 4.4, 15) shelf are all clear). The teleport is the documented stand-in transit; the
    // fire loop's aim sits a distance-scaled ~10° off the LIVE body pose (inside the 0.66-dot
    // lock cone but not on it) — the round spawns off-target and the HOMING does the turning
    // (the thing under test; the old code flew it straight past).
    let c0 = copter_pose(scene.sim.as_ref().unwrap());
    let spot = {
        let sim = scene.sim.as_ref().unwrap();
        let mut found = None;
        for (sx, sz) in [(20.0f32, 15.0), (25.0, 35.0), (-15.0, 30.0), (10.0, 30.0), (5.0, 25.0)] {
            let Some(fy) = sim.world.floor_below([sx, 45.0, sz], 120.0) else { eprintln!("  spot ({sx},{sz}): no floor"); continue };
            if fy < -5.0 || fy > 20.0 { eprintln!("  spot ({sx},{sz}): floor {fy} out of band"); continue; }
            let eye = [sx, fy + 1.6, sz];
            // The game's subject rule: the candidate's own shapes don't block its lock ray
            // [M Bullet_homing 0x12f358:92-93] — the same exclusion the homing scan now runs.
            let ex = sim.drones.copter_stream_hashes(ci);
            let los = sim.world.segment_bullet_clear_excluding(eye, c0, &ex);
            eprintln!("  spot ({sx},{sz}): floor {fy:.1} eye {:?} → c0 {:?} los {los}", eye, c0);
            if !los { continue; }
            found = Some([sx, fy, sz]);
            break;
        }
        let spot = found.expect("a standable spot with clear bullet-LOS to the copter's opening route exists on 07000012");
        eprintln!("lock firing spot ({:.1},{:.1},{:.1})", spot[0], spot[1], spot[2]);
        spot
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
    }
    for _ in 0..90 { aim_at(scene.sim.as_mut().unwrap(), [c0[0], c0[1] + 0.5, c0[2] + 8.0], false); }
    raise_gate(&mut scene);

    // Hold fire through the real trigger; the aim tracks the flying body with a ~10° lateral
    // offset (0.18·dist, the atan of the cone margin). The proof is the LOCK (only the class-0
    // arm sets a copter target) plus the CONVERGENCE: every round spawns ~10° off (dot ≈ 0.985
    // at 45 units), and only a steered round reaches the body (dot ≥ 0.999) — the game's steer
    // is `dir += clamp((to−dir)·0.0333, ±0.01)` per component [M 0x12f358:126-165], a straight
    // round's dot only ever drops as the lateral gap closes.
    let mut early = f32::NAN;
    let mut best_dot = 0.0f32;
    let mut locked = false;
    for t in 0..360 {
        let Some(cp0) = scene.sim.as_ref().unwrap().drones.copters.first().map(|c| c.pos()) else {
            panic!("the copter left the list mid-window (landed/deleted) — the window must hold a flying body");
        };
        // The aim point: the live body pose pushed ~10° sideways (0.18·dist lateral), so the
        // round spawns off-target and the homing turns it on.
        {
            let sim = scene.sim.as_ref().unwrap();
            let eye = [sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]];
            let dist = ((cp0[0] - eye[0]).powi(2) + (cp0[1] - eye[1]).powi(2) + (cp0[2] - eye[2]).powi(2)).sqrt();
            let (fx, fz) = (cp0[0] - eye[0], cp0[2] - eye[2]);
            let fl = (fx * fx + fz * fz).sqrt().max(1e-3);
            let lat = (0.18 * dist).clamp(3.0, 24.0);
            let aim = [cp0[0] - fz / fl * lat, cp0[1] + 0.5, cp0[2] + fx / fl * lat];
            aim_at(scene.sim.as_mut().unwrap(), aim, true);
        }
        let sim = scene.sim.as_ref().unwrap();
        let Some(b) = sim.drones.player_bullets.iter().rev().find(|b| b.weapon == 0x2e && b.homing) else { continue };
        let cpos = copter_pose(sim);
        let d = dot_to(b, cpos);
        if early.is_nan() && t > 20 { early = d; }
        best_dot = best_dot.max(d);
        // The lock itself [M 0x12f248's class-0 arm]: the copter body is the bullet's target.
        if b.target == Some(ci) && b.target_copter { locked = true; }
        if locked && best_dot >= 0.999 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.player_bullets.iter().any(|b| b.weapon == 0x2e && b.homing),
        "the Scorpion fired homing rounds [L the 0x20000 record bit]");
    assert!(locked, "the homing round must lock the copter body [M Bullet_heatseekable 0x12f248 class 0] — \
        the red run (old code, drones-only scan) never locked it");
    assert!(best_dot >= 0.999,
        "a steered round must converge onto the body: spawn dot {early:.4} → best {best_dot:.4} — \
         the old code's rounds flew straight past (their dot only drops)");
    eprintln!("lock proof: spawn dot {early:.4} → best {best_dot:.4}, locked");
}

/// The wall half: the subject rule excludes only the CANDIDATE's own shapes — the static and
/// missile sets never exclude (`Collide_LineOfSight`'s subjects are the bullet and the
/// candidate obj, not the world [M Bullet_homing 0x12f358:92-93]), so a wall between the
/// shooter and the body still stops the lock. The firing spot is picked with the EXCLUDING
/// clear reading FALSE (the body's own cluster removed, a real occluder left), the fire loop
/// releases the trigger only while the live eye→body ray stays blocked, and no round may ever
/// carry a copter target.
#[test]
fn a_wall_between_still_stops_the_lock_on_07000012() {
    let mut scene = nf_view::load_weapons(&iso(), "07000012.bin", false, &[0x2e])
        .expect("load 07000012 with the AT-600 Scorpion");
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters.len(), 1, "07000012 carries exactly one kind-0 copter [D #707]");
    take_and_draw_scorpion(&mut scene);
    raise_gate(&mut scene);
    let c0 = copter_pose(scene.sim.as_ref().unwrap());

    // The wall spot: a standable cell whose STATIC bullet path to the body's rest pose is
    // blocked (the candidate's own shapes excluded — with them gone, whatever blocks is the
    // world: walls, floors, terrain [D the level's own geometry]).
    let spot = {
        let sim = scene.sim.as_ref().unwrap();
        let ex = sim.drones.copter_stream_hashes(0);
        let mut found = None;
        for (sx, sz) in [(20.0f32, 15.0), (25.0, 35.0), (-15.0, 30.0), (10.0, 30.0), (5.0, 25.0), (30.0, 20.0), (-10.0, 10.0), (0.0, 20.0), (15.0, 35.0)] {
            let Some(fy) = sim.world.floor_below([sx, 45.0, sz], 120.0) else { continue };
            if fy < -5.0 || fy > 20.0 { continue; }
            let eye = [sx, fy + 1.6, sz];
            if sim.world.segment_bullet_clear_excluding(eye, c0, &ex) { continue; }
            eprintln!("wall spot ({sx},{sz}): floor {fy:.1} — the static path to the body is blocked");
            found = Some([sx, fy, sz]);
            break;
        }
        found.expect("a standable spot with a static occluder toward the body's route exists on 07000012")
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
    }

    // Fire only while the live eye→body ray is statically blocked (the wall half's premise);
    // the homing scan runs the same predicate per acquisition tick, so a blocked ray can
    // never yield the copter target.
    let mut fired = 0usize;
    let mut locked = false;
    for t in 0..420 {
        let blocked = {
            let sim = scene.sim.as_ref().unwrap();
            let Some(cp0) = sim.drones.copters.first().map(|c| c.pos()) else { break };
            let eye = [sim.bond.pos[0], sim.bond.pos[1] + 1.6, sim.bond.pos[2]];
            let ex = sim.drones.copter_stream_hashes(0);
            !sim.world.segment_bullet_clear_excluding(eye, cp0, &ex)
        };
        if blocked {
            aim_at(scene.sim.as_mut().unwrap(), [c0[0], c0[1] + 0.5, c0[2] + 8.0], true);
            fired += 1;
        } else {
            aim_at(scene.sim.as_mut().unwrap(), [c0[0], c0[1] + 0.5, c0[2] + 8.0], false);
        }
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.player_bullets.iter().any(|b| b.weapon == 0x2e && b.homing && b.target_copter) {
            locked = true;
        }
        let _ = t;
    }
    assert!(fired > 0, "the wall window must have fired (the body's route passes behind the occluder)");
    assert!(!locked, "no homing round may lock the copter while a static wall blocks the ray \
        [M Bullet_homing 0x12f358:92 — the LOS bit-0x8 drop trace gates the class-0 arm]");
    eprintln!("wall proof: {fired} rounds fired behind the occluder, none locked");
}

/// The dormant half [M Bullet_heatseekable 0x12f248:29 — the class-0 arm's `+0xfc != 0` gate]:
/// the copter still waits on its gate channel, its SP is `SP_HideObj`'d since create
/// [M 0x17d340:73-76], the view never adds the hidden body (the Script_HideObj cascade sets
/// obj+0xF0 bit 0x10 [M 0x1db808:27-32]) and the in-view counter decays to 0 within three
/// ticks [M 0x134630:16 / 0x1336f8:242-246] — so no round may lock it, whatever it aims at.
/// The first cut made every created copter a candidate with its position read at (0,0,0), so a
/// round turned toward the world origin (the red run).
#[test]
fn a_gated_copter_body_is_never_a_homing_candidate_on_07000012() {
    let mut scene = nf_view::load_weapons(&iso(), "07000012.bin", false, &[0x2e])
        .expect("load 07000012 with the AT-600 Scorpion");
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters.len(), 1, "07000012 carries exactly one kind-0 copter [D #707]");
    take_and_draw_scorpion(&mut scene);

    // The premise: the gate channel is still off — nothing on this path raised it.
    let gate = scene.sim.as_ref().unwrap().drones.copters[0].gate;
    assert!(gate != 0, "the copter must still be gated on this path (the channel only fires through the level's own script trigger)");
    assert!(!scene.sim.as_ref().unwrap().drones.copters[0].body_shown(),
        "a gated copter's body is SP_HideObj'd [M 0x17d340:73-76] — the port's shown flag must read false");

    // Fire from the open ground by the pickup (−0.04, 0.78, −4.06 — the level's own floor),
    // aimed straight at the WORLD ORIGIN: the first cut's candidate position (its `copter_body_pos`
    // read column 3 of rows 0-2 — (0,0,0) for every rigid matrix), so the old code's rounds saw a
    // seekable "body" at the origin and steered onto it. The origin is in the open here, so the
    // red run's lock is not LOS-limited. No round may lock the dormant body.
    let spot = {
        let sim = scene.sim.as_ref().unwrap();
        let mut cands: Vec<(f32, [f32; 3])> = Vec::new();
        'outer: for dx in (-60..=60i32).step_by(4) {
            for dz in (-40..=40i32).step_by(4) {
                let xz = [-0.04 + dx as f32, -4.06 + dz as f32];
                let d = (xz[0] * xz[0] + xz[1] * xz[1]).sqrt();
                if !(4.0..=60.0).contains(&d) { continue; }
                let Some(fy) = sim.world.floor_below([xz[0], 6.0, xz[1]], 12.0) else { continue };
                let eye = [xz[0], fy + 1.6, xz[1]];
                if !sim.world.segment_bullet_clear(eye, [0.0, 1.0, 0.0]) { continue; }
                cands.push((d, [xz[0], fy, xz[1]]));
            }
        }
        // The farthest clear cell: the longest flight the pocket allows at the origin.
        cands.sort_by(|a, b| b.0.total_cmp(&a.0));
        let spot = cands.first().map(|(_, s)| *s)
            .expect("open ground with bullet-LOS to the origin exists by the 07000012 pickup");
        eprintln!("dormant firing spot ({:.1},{:.1},{:.1}) — {} clear cells", spot[0], spot[1], spot[2], cands.len());
        spot
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, 0.0);
    }
    for _ in 0..90 { aim_at(scene.sim.as_mut().unwrap(), [0.0, 1.0, 0.0], false); }
    let mut seen = 0usize;
    for t in 0..240 {
        aim_at(scene.sim.as_mut().unwrap(), [0.0, 1.0, 0.0], true);
        let sim = scene.sim.as_ref().unwrap();
        for b in sim.drones.player_bullets.iter().filter(|b| b.weapon == 0x2e && b.homing) {
            seen += 1;
            assert!(!b.target_copter,
                "tick {t}: a homing round locked the DORMANT copter (target {:?}) — the +0xFC gate \
                 [M Bullet_heatseekable 0x12f248:29] must refuse a hidden body (a drone lock is the \
                 class-2 arm working and stays legal)", b.target);
        }
    }
    assert!(seen > 0, "the Scorpion must have fired homing rounds on this path [L the 0x20000 record bit]");
    // The premise held to the end: the gate never self-fired under the fire.
    assert!(scene.sim.as_ref().unwrap().drones.copters[0].gate != 0, "the gate channel stayed off for the whole window");
    eprintln!("dormant window clean: {seen} homing rounds, none locked the gated body");
}
