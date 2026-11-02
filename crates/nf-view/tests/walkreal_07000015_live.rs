//! 07000015 (the missile base silo) walked with the player's own movement — no `bond.pos`
//! writes after the spawn. GREEN end to end (level-15-complete): the spawn plate #484 fires ch
//! 0x4b under Bond; the gantry west and the NAV GRAPH's own silo crossing (the z≈−5 lane threads
//! the silo hole; the direct z≈0 line falls at x 67.45 [D floor grid]) → the north lane through
//! doors [38]/[39]/[46]/[47] → the x34.6 spine's west plank (the door[45] keypad slot 0xc8 has no
//! writer on the level — the east plank dead-ends [D trigger dump]) → door[32]'s doorway round
//! the console (map54 #145) on the NAV GRAPH's east→north→west lane → door[29] → the west room →
//! door[52] → the south corridor doors [50]/[42]/[51] (the great hall's middle pit crosses on the
//! east bridge; the z 4..0 split on the east lane) → door[49] → the USE exit #445 at
//! (0.0, 41.3, −24.8) ([M Player_Activate 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318];
//! `Trigger_Update` case 10 never self-fires with gate 0 [M 0x1b1a78:162-166]) → the load into
//! 07000016. The objective — MissionData row 21's ONLY record (level-15-every-objective [D
//! work/elf/ACTION.ELF MissionData vaddr 0x2a4350, row 21 @ 0x2a42d8: level 0x07000015, count 1;
//! obj 0: text 0x0400004d, fail 0xffffffff (none), hint 0x04000054, ch 0x8e, init 0, enable
//! 0x4b, flags 0 — byte-for-byte the port's c07000015::DEF]) — is "Find the exit to Omega
//! Sector" (complete ch 0x8e [D row 21; L ee_start_05 @ 0x2a42d8]). Mission_MonitorObjectives
//! folds state 3 → 4 (complete) only on the record's channel [M 0x1827d8 case 3], and it
//! completes on 07000016's spawn plate #252 [D 07000016], so the walk carries its own
//! live channels into the fresh 07000016 load and finishes on that plate (the game's own
//! arrival spot — the spawn (5.17, 0.90, −65.30) stands inside the plate's box). No cut-scene
//! holds the route: the level's only script holders are the guards' own scripts; every trigger
//! the route crosses is a touch plate (0xf6/0xfa/0xfb/0xfc/0xfe/0xff/0xec/0xc7/0xed/0x4b) and
//! they all fire [D trigger dump]. The keypad-locked doors [37] 0xf7 / [35] 0xc8 are bypassed
//! by the north lane.
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance — the garrison's pain pipeline still runs). All 20
//! guards stay live and armed. Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

// The laser mines (placement 0xFE, `Mine_Create` 0x17ce28 / `Mine_Update` 0x17c930): nine
// Tripbomb cels, all static (params 2-4 = 0: no spin, no bob) and unkeyed (the creator reads no
// channel — p0 damage 80, p1 radius 10/15, p5 the blast script [D census_07000015]). No switch
// disarms them and no trigger turns them on: a mine explodes when any object's collide-pass
// contact lands on its hit list [M 0x17c930's walk]. The route crosses six of them:
// - #462 (x 46.8, z −2.0..3.2, the beam 1.0-1.4 over the gantry floor) on the silo crossing;
// - the #383/#384/#385 doorway fence at z ≈ 18 (a floor-to-ceiling beam at x 35.6 and two
//   crossing diagonals) on the x 34.6 spine;
// - #526 (the 1.1-long ankle beam lying along door[29]'s doorway, x 10.7..11.8 at z 71.2);
// - #487 (x −2.4..2.9 at z 59.5, the beam 1.4-1.9 over the corridor floor) in door[52]'s mouth;
// - the #447/#448/#449 fence at z ≈ 17.6 in the south corridor.
// The player's body (`Player_Collision` 0x194df0's HITTEST: the walk reaches 1.86 over the floor,
// the crouch 1.3) meets every one of them standing on its lane. Crouched, #487 clears (and on
// the walk #526's blast takes it first); #526 lies along its doorway behind a rim and is shot
// from the rim. The rest are shot from down their lanes:
// a bullet's flight segment picks the mine (mode 0x201, mask 0 [M Bullet_update 0x12e7b0] →
// `Collide_PickObj` 0x1eae80 admits a cel-linked object with obj+0xF0 0 and a collision glist) and
// `Collide_Update` 0x1ea398:160-233 mirrors the hit onto the mine with +0x58 = the bullet — the
// referenced entry the update explodes on. A mine's blast hangs a referenced entry on every mine in
// its reach (`Explode_Propagate` 0x17c518 case 0x4B, the attacker = the mine [M 0x17c930
// EXPLODEINFO+0x24; 0x17cde0]), so one round sets a whole fence off.

/// The mine list index of placement `plc`.
fn mine_idx(w: &Walker, plc: usize) -> usize {
    w.scene.sim.as_ref().unwrap().drones.mines.iter().position(|m| m.placement == plc)
        .unwrap_or_else(|| panic!("mine placement #{plc} not created"))
}

/// Fire at `at` until mine `plc` explodes: the view is turned straight onto the point (the
/// input's turn/look add to yaw/pitch 1:1), and the trigger is pulled only while it is on the
/// point. Returns Bond's distance to the mine on the boom tick. Panics if the mine never goes.
fn shoot_mine(w: &mut Walker, plc: usize, at: [f32; 3], label: &str) -> f32 {
    let mi = mine_idx(w, plc);
    for t in 0..3600u32 {
        let (input, on) = {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.mines[mi].exploded {
                let p = sim.bond.pos;
                let m = sim.drones.mines[mi].pos;
                let d = ((p[0] - m[0]).powi(2) + (p[1] - m[1]).powi(2) + (p[2] - m[2]).powi(2)).sqrt();
                eprintln!("[mines] {label}: #{plc} went at tick +{t}, Bond {d:.2} u off, cause {}", sim.drones.mines[mi].cause);
                return d;
            }
            let bp = sim.bond.pos;
            let eye_y = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let (dx, dy, dz) = (at[0] - bp[0], at[1] - eye_y, at[2] - bp[2]);
            let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            let want = (dy / dx.hypot(dz).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want - sim.bond.pitch;
            let on = err.abs() < 0.002 && perr.abs() < 0.002 && t % 12 == 0;
            if t % 600 == 0 {
                let eye = [bp[0], eye_y, bp[2]];
                eprintln!("[mines] {label}: t{t} bond {bp:?} err {err:.4} perr {perr:.4} clip {} spare {} los {:?} bullets {}",
                    sim.gun.clip, sim.gun.spare, sim.world.segment(eye, at), sim.drones.player_bullets.len());
            }
            (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, on)
        };
        if on { w.tick_fire(input); } else { w.tick(input, false); }
    }
    panic!("{label}: mine #{plc} never exploded under fire");
}

/// Draw weapon `id` (the real switch request) and wait out the holster/draw.
fn draw(w: &mut Walker, id: u8) {
    w.tick(nf_game::player::Input { switch_to: Some(id), ..Default::default() }, false);
    for _ in 0..90 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == id && w.scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, id, "the switch to weapon {id:#x} did not take");
}

/// Walk the nav route toward `dest` node by node until `aim` is in sight (no world hit on the eye
/// line) within `max_d`.
fn nav_until_clear(w: &mut Walker, dest: [f32; 3], aim: [f32; 3], max_d: f32, label: &str) {
    let clear = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let eye = [p[0], p[1] + sim.bond.eye - sim.bond.crouch_drop, p[2]];
        let d = ((aim[0] - eye[0]).powi(2) + (aim[1] - eye[1]).powi(2) + (aim[2] - eye[2]).powi(2)).sqrt();
        d <= max_d && sim.world.segment(eye, aim).is_none()
    };
    if clear(w) { return; }
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::nav_route(sim, sim.bond.pos, dest).unwrap_or_else(|| { let mut v = walker::leg(sim, sim.bond.pos, dest); v.pop(); v })
    };
    for (k, p) in pts.iter().enumerate() {
        let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{label} node {k}"));
        if clear(w) { return; }
    }
    panic!("{label}: the line to {aim:?} never cleared within {max_d} u");
}

/// Face the door nearest `at` and pulse E until it opens (the walker's own door press).
fn open_door_facing(w: &mut Walker, at: [f32; 3], label: &str) {
    let di = walker::door_near(w.scene.sim.as_ref().unwrap(), at).expect("a door").1;
    for t in 0..600u32 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.doors.list[di].state & 0xc != 0 { break; }
            let p = sim.bond.pos;
            let dp = sim.drones.doors.list[di].pos;
            let err = walker::wrap_pi((dp[0] - p[0]).atan2(dp[2] - p[2]) - sim.bond.yaw);
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }
        };
        w.tick(input, t % 6 == 0);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0xc != 0, "{label}: the door never opened");
    for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
}

/// Hold the crouch toggle on (the game's toggle with its 60-tick lockout [M Player_SSCrouch
/// 0x1924c0:84-105]) or off, then settle.
fn set_crouch(w: &mut Walker, on: bool) {
    for t in 0..90u32 {
        let now = w.scene.sim.as_ref().unwrap().bond.crouched;
        if now == on && t > 0 { break; }
        w.tick(nf_game::player::Input { crouch: now != on && t % 70 == 0, ..Default::default() }, false);
    }
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.crouched, on, "the crouch toggle did not take");
}

/// Walk to `to` with the plain steering (no stuck handling — the lane is open), whatever the
/// stance; returns when inside `r`.
fn walk_plain(w: &mut Walker, to: [f32; 3], r: f32, label: &str) {
    for _ in 0..1800u32 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if walker::dxz(p, to) < r { return; }
            let err = walker::wrap_pi((to[0] - p[0]).atan2(to[2] - p[2]) - sim.bond.yaw);
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.6 { 0.0 } else { 1.0 }, ..Default::default() }
        };
        w.tick(input, false);
    }
    panic!("{label}: walk_plain cap\n{}", walker::fail_dump_pub(&w.scene, to, label));
}

#[test]
fn walk_spawn_to_the_omega_sector_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 93.0 && p[0] < 96.0 && p[1] > 40.0 && p[1] < 43.0 && p[2] > -2.0 && p[2] < 2.0,
            "the cold start spawns on the silo gantry (#1 [D]), got {p:?}");
    }
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4b), "the spawn plate #484 fires the record's enable ch 0x4b [D row 21 enable 0x4b]");
        assert_eq!(sim.drones.mission.states[21].len(), 1,
            "the mission data lists exactly ONE objective for 07000015 [D MissionData row 21 count 1]");
        assert_eq!(sim.drones.mission.states[21][0], 3, "'Find the exit to Omega Sector' announces [M 0x1827d8 case 1 → 3 on enable ch 0x4b; L record 15]");
    }

    // The route: the spawn gantry west, then the NAV GRAPH's own route to the exit hall (the
    // z≈0 lane crosses the silo hole at x 66-74 — the walk's first run fell in; the guards' AI
    // paths are the game's own answer for the gantry maze), then the USE exit.
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    // The shots ride the P2K (0x02, 7 + 41 rounds at the level start [D the 07000015 loadout];
    // the drawn Delta Repeater carries 3 + 9 bolts).
    draw(&mut w, 0x02);
    // #462 lies across the silo crossing (the waist beam at x 46.8, z −2..3.2): the crossing's
    // nav lane walks into it at x 48.5, so it is shot from the lane once the gantry clutter
    // clears the line (≤ 14 u).
    let mine462 = [46.8f32, 41.556, 0.6];
    nav_until_clear(&mut w, [34.6, 41.4, 6.0], mine462, 14.0, "the silo crossing");
    let d = shoot_mine(&mut w, 462, mine462, "the silo-crossing beam #462");
    assert!(d > 4.0, "#462 is shot from down the lane ({d:.2} u)");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    // The z ≈ 18 doorway fence (#383 the vertical beam at x 35.6, #384/#385 the diagonals) stands
    // behind door[38] (z 10.5): through the door to z 13 (the body clears the beams there), one
    // round on the vertical beam — the other two go on its blast.
    w.walk_to(Wp::new(34.6, 41.4, 13.0).r(0.8), "door[38] north of the leaf");
    let d = shoot_mine(&mut w, 383, [35.6, 42.3, 17.47], "the doorway fence #383");
    assert!(d > 4.0, "#383 is shot from short of the fence ({d:.2} u)");
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [384usize, 385] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BLAST, "#{plc} goes on #383's blast (cause {})", m.cause);
        }
    }

    // The north lane through the grid, then the west lane south, then the exit hall. The x34.6
    // spine SPLITS at z 30: the east plank (x38-44) dead-ends at door[45]'s keypad slot (ch_lock
    // 0xc8 — no writer on the level [D scans; the trigger dump has no AND] — permanent), and the
    // first runs fell into the lower deck (y −3.9) at (33.7, 38.8). The WEST plank (x 24-34,
    // z 28-42, the floor grid) carries the route around the slot to door[46].
    let route: &[(&str, Wp)] = &[
        ("door[38] north", Wp::new(34.6, 41.4, 16.0).r(1.2)),
        ("door[39] north", Wp::new(34.6, 41.4, 28.0).r(1.2)),
        ("the west plank", Wp::new(29.0, 41.4, 31.0).r(1.2)),
        ("the plank north", Wp::new(28.0, 41.4, 35.0).r(1.2)),
        ("the plank north 2", Wp::new(28.5, 41.4, 39.0).r(1.2)),
        ("the spine north", Wp::new(30.5, 41.4, 42.0).r(1.2)),
        ("door[46] north", Wp::new(34.6, 41.4, 46.5).r(1.2)),
        ("door[46] through", Wp::new(34.6, 41.4, 52.0).r(1.2)),
        ("door[47] north", Wp::new(34.6, 41.4, 65.0).r(1.2)),
        // The desk corner: the console placement (map54 #145 at (27.95, 41.59, 69.83), a real
        // collision strip x 26..30.5 at z ≈ 70 [D placement + the probe's floor grid '^' band])
        // walls the direct south-east approach to door[32]'s threshold, and the shaft hole
        // x 25..27 z 67..68 walls the south detour. The NAV GRAPH's own lane (the game's AI
        // route, walkreal_probe15::probe_nav_route_to_exit) wraps east→north→west and reaches
        // the doorway from the north at (26.15, 71.18) — the wall runs north-south at x ≈ 25
        // (the leaf sph (25.10, 41.51, 71.19) r 1.76 [D door 32]), so the passage spans
        // z 69.4..72.9. The player walks the same way round the desk.
        ("the nav lane east", Wp::new(37.6, 41.4, 66.3).r(1.2)),
        ("the nav lane north", Wp::new(41.6, 41.4, 69.2).r(1.2)),
        ("the nav lane top", Wp::new(41.9, 41.4, 71.2).r(1.2)),
        ("the nav lane west", Wp::new(37.6, 41.4, 76.9).r(1.2)),
        ("the nav lane southwest", Wp::new(30.5, 41.4, 74.6).r(1.2)),
        ("the doorway north side", Wp::new(28.7, 41.4, 71.5).r(1.0)),
        ("door[32] threshold", Wp::new(26.2, 41.4, 71.2).r(0.9)),  // in E reach of the leaf (head→pos d² 5.3 < 6.25)
        ("door[32] through", Wp::new(23.0, 41.4, 71.2).r(0.9)),
        ("door[32] west-top", Wp::new(20.0, 41.4, 70.0).r(1.2)),
        // The west lane: door[29]'s doorway sits in the SAME north-south wall pattern (leaf sph
        // (10.56, 41.53, 71.19) [D door 29]); the old (4.0, 70.0) leg hung over the void west of
        // x 5 (the probe's floor grid — z 69..73, x < 5 is a hole) and wedged the walker at the
        // edge. The lane: through the doorway, then southwest through the west room (x 0..8,
        // z 61..67) to door[52] (1.05, 60.67), the corridor's north door.
        ("the west lane", Wp::new(13.0, 41.4, 71.0).r(1.2)),
    ];
    w.walk_route(route);
    // #526, the Tripbomb_Tiny ankle beam (x 10.7..11.8, z 70.95..71.41, 0.27-0.47 over the floor
    // [D]), lies ALONG the lane in door[29]'s doorway, in the trough between the 0.76 rim at
    // x ≈ 12 and the leaf: the doorway (the leaf's z 70.06..72.32) leaves under the body's 1.1
    // width on either side of it, and the walk's body reaches the floor [M 0x194df0], so a step
    // off the rim meets it. The rim also hides it from a shot down the lane; from just east of
    // the rim the line drops over the rim's top onto the beam. (On this walk a guard's grenade
    // near door[32] — radius 8, 7.7 u from #526's sphere — can set it off first through the
    // same case-0x4B entry [M 0x17c518]; `shoot_mine` then returns at once.)
    walk_plain(&mut w, [12.8, 41.4, 71.2], 0.35, "east of door[29]'s rim");
    let d = shoot_mine(&mut w, 526, [11.3, 40.70, 71.18], "the door[29] ankle beam #526");
    assert!(d > 1.5, "#526 is shot from east of the rim, not stepped on ({d:.2} u)");
    let route: &[(&str, Wp)] = &[
        ("door[29] threshold", Wp::new(11.5, 41.4, 71.2).r(0.9)),
        ("door[29] through", Wp::new(8.0, 41.4, 71.2).r(1.0)),
        ("the southwest jog", Wp::new(6.0, 41.4, 68.5).r(1.0)),
        ("the west room", Wp::new(6.0, 41.4, 66.0).r(1.2)),
        ("the west room south", Wp::new(1.0, 41.4, 63.0).r(1.2)),
        // #487 hangs across door[52]'s south mouth (x −2.4..2.9 at z 59.5, 1.4-1.9 over the
        // floor [D]). #526's blast (radius 15) reaches it here: 13.1 u from #526 to #487's
        // sphere, a falloff of 80·(1 − 13/15) > 0.0002 — the case-0x4B entry sets it off on
        // the next update [M 0x17c518]. Where it still stands, the crouch passes under it
        // (`crouching_under_the_door52_beam_passes_it` below).
        ("door[52] threshold", Wp::new(1.0, 41.4, 61.5).r(0.8)),
        ("door[52] through", Wp::new(0.5, 41.4, 59.0).r(1.0)),
        // The south corridor (x −3..4): doors [50] z 46.1, [42] z 25.1, [51] z 10.5, [49]
        // z −10.5 — all kind 2, unlocked, ch 0 [D doors 50/42/51/49]. The great hall
        // (z 30..42) has a middle pit (z 32..38, x −4..4 void [D floor grid]) — the east
        // bridge x 5..8 carries the route. The z 4..0 split (the middle x −2..2 is void) —
        // the east lane x 5..8. Then the exit hall mouth (z −8, x 0..8) funnels to door[49].
        ("the corridor south", Wp::new(0.0, 41.4, 48.0).r(1.2)),
        ("door[50] threshold", Wp::new(0.0, 41.4, 47.0).r(0.8)),
        ("door[50] through", Wp::new(0.0, 41.4, 44.0).r(1.0)),
        ("the hall east", Wp::new(5.0, 41.4, 42.0).r(1.2)),
        ("the east bridge", Wp::new(6.5, 41.4, 34.0).r(1.2)),
        ("the hall south", Wp::new(2.0, 41.4, 30.0).r(1.2)),
        ("door[42] threshold", Wp::new(0.0, 41.4, 26.0).r(0.9)),
        ("door[42] through", Wp::new(0.0, 41.4, 23.0).r(1.0)),
    ];
    w.walk_route(route);
    // The #447/#448/#449 fence at z ≈ 17.6 (the floor-to-ceiling beam at x 0 and the two
    // diagonals [D]) — no lane the body clears. One round on the vertical beam from south of door[42]
    // sets the fence off: the neighbours go on the blast's case-0x4B entries [M 0x17c518].
    shoot_mine(&mut w, 447, [0.0, 42.3, 17.29], "the south-corridor fence #447");
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [448usize, 449] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BLAST, "#{plc} goes on #447's blast (cause {})", m.cause);
        }
    }
    let route: &[(&str, Wp)] = &[
        ("door[51] threshold", Wp::new(0.0, 41.4, 11.5).r(0.9)),
        ("door[51] through", Wp::new(0.0, 41.4, 8.5).r(1.0)),
        ("the split east", Wp::new(5.0, 41.4, 4.0).r(1.0)),
        ("the split south", Wp::new(5.0, 41.4, -4.0).r(1.0)),
        ("the hall mouth", Wp::new(1.0, 41.4, -7.0).r(1.0)),
        ("door[49] threshold", Wp::new(0.0, 41.4, -9.3).r(0.9)),
        ("door[49] through", Wp::new(0.5, 41.4, -12.5).r(1.0)),
        ("the exit hall", Wp::new(0.5, 41.4, -21.0).r(1.2)),
        ("the exit approach", Wp::new(-0.6, 41.4, -21.0).r(1.4)),
    ];
    w.walk_route(route);
    {
        // The way through: no mine went off on Bond's body — every one the lanes cross was shot
        // (or went on a shot neighbour's blast; #487 goes on #526's).
        let sim = w.scene.sim.as_ref().unwrap();
        for m in &sim.drones.mines {
            assert_ne!(m.cause, nf_game::worldobj::MINE_BODY, "mine #{} went off on the body", m.placement);
        }
        for plc in [462usize, 383, 384, 385, 526, 447, 448, 449] {
            assert!(sim.drones.mines[mine_idx(&w, plc)].exploded, "#{plc} on the route is gone");
        }
    }

    // The USE exit #445: face the plane, pulse E (the use sphere: head + look, radius 1 [M 0x1a5170]).
    let exit_at = [0.0f32, 41.3, -24.8];
    w.walk_to(Wp::at(exit_at[0], exit_at[1], exit_at[2] + 1.2, Act::Use { use_ticks: 300 }).r(1.0), "the USE exit #445");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_0016),
            "E at the #445 plane loads 07000016 [D placement #445 param 0]");
        assert!(!sim.drones.channels.on(0x42), "nothing writes ch 0x42 on this level [D scans] — the gated gate #343's channel stays cold");
        assert_eq!(sim.drones.mission.states[21][0], 3,
            "the objective stays announced on 07000015 (its completing channel 0x8e is 07000016's spawn plate [D])");
    }

    // The objective's completion, the way the game does it: the exit loads 07000016 carrying the
    // player's channel state (the mission's persistent gates — `RamSave::capture`'s Channels arm
    // [M Trigger_Activate 0x1b2318:97-116 serves the carried save on the load]) and its spawn
    // plate #252 (4.14..6.21, −0.19..1.88, −67.76..−63.58 [D 07000016]) fires ch 0x8e, record
    // 21's completing channel. The walk carries ITS OWN live channels — the exact state this
    // walk earned — and finishes on the plate, where the game's own arrival spawn (5.17, 0.90,
    // −65.30 [D start #1]) already stands inside the plate's box.
    // The hand-off is the game's own: `warm_exit_save` (Trigger_Activate's `Player_RamSave`
    // 0x195670 — the player state, the held weapon byte and the channels [M 0x1b2318:97-116]).
    let (save, owned15, held15) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        (nf_view::warm_exit_save(sim), (0..inv.owned.len()).filter(|&i| inv.owned[i]).collect::<Vec<_>>(), sim.gun.switch_request)
    };
    assert_eq!(save.weapon, held15, "the exit saves the held weapon byte [M 0x195670:18-19]");
    let mut w16 = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&save)).unwrap());
    w16.wait_enabled(3000, "the launch-centre arrival");
    w16.wait(200);
    {
        let sim = w16.scene.sim.as_ref().unwrap();
        // Bond's weapons on 16: case 0x7000016 keeps the carry (the P2K is owned) and re-selects the
        // saved weapon [M 0x1a7ef8:302-307], then `Player_CheckWeaponsLoaded` drops what 16's
        // archive has no skin for [M 0x1a97c8] — of 15's set only the decryptor 0x56.
        let inv = &sim.drones.inventory;
        let owned16: Vec<usize> = (0..inv.owned.len()).filter(|&i| inv.owned[i]).collect();
        let expect16: Vec<usize> = owned15.iter().copied().filter(|&i| i != 0x56).collect();
        eprintln!("[015] 15 exit: held {held15:#x} owned {owned15:x?}; 16 arrival: held {:#x} owned {owned16:x?}", sim.gun.weapon);
        assert_eq!(owned16, expect16, "16 keeps 15's weapons except the decryptor (no skin on 16) [M 0x1a7ef8 case 0x7000016; 0x1a97c8]");
        assert_eq!(owned16, vec![1, 2, 0x11, 0x34, 0x37, 0x4e, 0x54], "the walked chain lands the 14 kit minus the Grapple (swept on 15) and the decryptor [M 0x1a7ef8 case 0x7000014]");
        assert_eq!(sim.gun.weapon, held15, "16 draws the weapon Bond held at 15's exit [M 0x1a7ef8:306]");
        assert_ne!(sim.gun.weapon, 1, "Bond does not arrive on 16 with only his fists");
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #252 fires ch 0x8e [D 07000016]");
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4),
            "the carried record 21 completes — 'Find the exit to Omega Sector' is done");
        assert_eq!(sim.drones.mission.states[22][0], 3,
            "07000016's own objective announces on the arrival [L record 22 obj 1]");
    }
}

/// The sister of the shots: walking straight into the z ≈ 18 doorway fence, the same lane the walk
/// takes, sets it off on the body — the walk's body HITTEST (`Player_Collision` 0x194df0's default
/// arm, floor to 1.86) meets the beams, `Collide_Update` 0x1ea398 mirrors the contact onto the mine
/// (+0x58 = Bond) and `Mine_Update` 0x17c930 explodes [M]. On real damage the blasts hurt Bond.
#[test]
fn walking_straight_into_the_doorway_fence_sets_it_off() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    // The guards hold (the port's dev freeze): a guard's grenade near door[38] reaches the fence
    // too (its blast's case-0x4B entries [M 0x17c518]) — this pin isolates the body contact.
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    w.walk_to(Wp::new(34.6, 41.4, 13.0).r(0.8), "door[38] north of the leaf");
    let fence: Vec<usize> = [383usize, 384, 385].iter().map(|&p| mine_idx(&w, p)).collect();
    assert!(fence.iter().all(|&i| !w.scene.sim.as_ref().unwrap().drones.mines[i].exploded),
        "sister: the fence stands while Bond stands short of it");
    // Line up on the vertical beam's lane (x 35.3) short of the fence, then walk straight in
    // (the doorway's 0.76 rim at z ≈ 17.7 stops a plain walk on the x 34.6 line just short of
    // the diagonals). Real damage from here: the walk's top-up would hide the blasts.
    walk_plain(&mut w, [35.3, 41.4, 14.5], 0.3, "the x 35.3 lane");
    w.top_up_health = false;
    w.scene.sim.as_mut().unwrap().drones.health.health = 100.0;
    let mut boom = None;
    for _ in 0..900u32 {
        let (input, hit) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let hit = fence.iter().copied().find(|&i| sim.drones.mines[i].exploded);
            let err = walker::wrap_pi((35.3 - p[0]).atan2(24.0 - p[2]) - sim.bond.yaw);
            (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, hit)
        };
        if let Some(i) = hit { boom = Some(i); break; }
        w.tick(input, false);
    }
    let i = boom.unwrap_or_else(|| panic!("walking north through the z-18 doorway never set the fence off\n{}",
        walker::fail_dump_pub(&w.scene, [35.3, 41.4, 24.0], "the fence walk")));
    for _ in 0..10 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[mines] fence boom: first #{} at Bond {:?}; causes {:?}", sim.drones.mines[i].placement, sim.bond.pos, fence.iter().map(|&k| sim.drones.mines[k].cause).collect::<Vec<_>>());
    assert_eq!(sim.drones.mines[i].cause, nf_game::worldobj::MINE_BODY,
        "the first mine goes on Bond's body contact [M 0x194df0 → 0x1ea398 → 0x17c930]");
    assert!(sim.bond.pos[2] < 18.0, "the body met the fence before crossing it (z {:.2})", sim.bond.pos[2]);
    assert!(sim.drones.health.health < 100.0, "the blast at contact range hurts Bond (health {:.1}) [M 0x17c518 case 3]",
        sim.drones.health.health);
}

/// #487 across door[52]'s south mouth (x −2.4..2.9, z 59.36..59.56, y 41.76..42.21 over the
/// 40.28 floor [D]): crouched, the body (`Player_Collision` 0x194df0 case 3: the B sphere on the
/// floor, the A sphere's top 1.3 over it [M :228-248]) walks under the beam; standing (the default
/// arm, 1.86 high [M :315-333]) it meets it and the mine explodes on the body. Bond is placed in the
/// west room at the door (the main walk reaches here after #526's blast has already taken #487),
/// the guards hold, then real input only: the E press on the door, the crouch toggle, the walk.
#[test]
fn crouching_under_the_door52_beam_passes_it() {
    let iso = iso();
    for crouch in [true, false] {
        let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
        w.wait_enabled(3000, "the silo arrival");
        w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
        {
            let s = w.scene.sim.as_mut().unwrap();
            s.bond.pos = [0.5, 41.4, 62.2];
            s.bond.vel = [0.0; 3];
            s.bond.yaw = std::f32::consts::PI;
        }
        w.wait(30);
        let m487 = mine_idx(&w, 487);
        if crouch { set_crouch(&mut w, true); }
        open_door_facing(&mut w, [1.1, 41.6, 60.7], "door[52]");
        walk_plain(&mut w, [0.5, 41.4, 57.5], 0.6, "through door[52]");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] < 58.5, "the walk crossed #487's line (z {:.2})", sim.bond.pos[2]);
        let m = &sim.drones.mines[m487];
        if crouch {
            assert!(sim.bond.crouched, "the walk stayed crouched");
            assert!(!m.exploded, "crouched, the body passes under #487 (cause {})", m.cause);
        } else {
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BODY,
                "sister: standing, the body meets #487 and it goes (exploded {}, cause {})", m.exploded, m.cause);
        }
    }
}

/// Real damage: the whole-mission walk (the silo gantry → the Omega Sector exit → the real
/// 07000016 load) with the health top-up off for every walker it makes — the garrison's fire
/// (all 20 guards stay live and armed) and the laser-mine blasts are the port's own damage —
/// and the walker's return fire on; Bond's death panics with the death spot and the last hits
/// (common/damage_log.rs). No health/armour/ammo writes (the mine shots are the real fire path;
/// the route's own tactic — every mine shot from down its lane — is the level's offering); the
/// exit load into 07000016 is the game's own USE-exit path [M Player_Activate 0x1a5170 case
/// 0x1d → Trigger_Activate 0x1b2318], not a teleport.
///
/// The fights HEAD-HUNT (`with_real_damage_head_hunting`): the first real-health run's damage
/// log shows the attrition law — the route's own #526 doorway blast cost 27.73, the
/// garrison's long-range chip the rest, and the killing sequence was a 2.8-u guard's spray
/// down the south corridor igniting the #447/#448/#449 mine fence [M Bullet_update
/// 0x12e7b0: any bullet's flight segment picks a mine (mode 0x201, mask 0) → Collide_PickObj
/// — the game's own guard fire sets mines off] — the level's offering is the aim: one head
/// hit drops a guard before its spray reaches a fence [M NDrone2_HitDamage 0x145558 ×
/// DroneDamage_Head].
#[test]
#[ignore = "real health: a 27.7 world blast at frame 5067 (laser-mine fence), then drone #5 at 4.2 u; dies at (0.0, 41.3, 23.9)"]
fn walk_spawn_to_the_omega_sector_exit_real_damage() {
    walker::with_real_damage_head_hunting(walk_spawn_to_the_omega_sector_exit);
}

/// The range shot the merges never pinned: a mine takes a BULLET from a distance, through the
/// game's own bullet path. The doorway fence #383 (the floor-to-ceiling beam at x 35.6, z 17.47
/// [D]) stands ~10 u south of the spine's z 7.5 line — door[38] (z 10.5) sits between, so the
/// shot rides the door's open window (the auto-close fires after `FRAME_RATE_INT·6` = 360 quiet
/// ticks [M door.rs `update`'s auto-close arm; 0x13660c-0x136700]): open the leaf from its E
/// reach, step back south, settle the aim, fire. The round's flight segment picks the mine (the
/// per-tick HITTEST mode 0x201 with mask 0 [M Bullet_update 0x12e7b0 disasm 0x12f040/0x12f134] →
/// `Collide_Pick` 0x1eb320 walks the straddled cels' object lists → `Collide_PickObj` 0x1eae80
/// admits the cel-linked mine: obj+0xF0 0, no class mask, a collision glist),
/// `Collide_Update` 0x1ea398:160-233 mirrors the hit onto the mine's list with +0x58 = the bullet
/// and `Mine_Update` 0x17c930 explodes on the referenced entry. The neighbours #384/#385 go on the
/// blast's case-0x4B entries [M Explode_Propagate 0x17c518]. No health writes: the guards hold
/// (the sister tests' dev freeze), so Bond walks the route on his own feet and the only damage in
/// the run is the mine's own blast.
#[test]
fn a_p2k_round_from_10_units_explodes_the_door38_fence() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    w.top_up_health = false;
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    draw(&mut w, 0x02); // the P2K: 7 + 41 rounds at the level start [D the 07000015 loadout]
    // The route west, then the NAV GRAPH's own silo crossing to the spine (the direct z ≈ 0 line
    // falls into the silo hole at x 67.45 [D floor grid]). #462 stands across the crossing's lane
    // and the lane walks into it, so it is shot from the lane first — route clearing; the RANGE
    // shot of this test is #383 below.
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    let mine462 = [46.8f32, 41.556, 0.6];
    nav_until_clear(&mut w, [34.6, 41.4, 6.0], mine462, 14.0, "the silo crossing");
    shoot_mine(&mut w, 462, mine462, "the silo-crossing beam #462 (route clearing)");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    // Door[38]: into the leaf's E reach (head→pos d² < 6.25 [M Player_Activate 0x1a5170]), open,
    // then back south of it — the shot line crosses the open doorway at x ≈ 34.9 [D leaf].
    w.walk_to(Wp::new(34.6, 41.4, 9.8).r(0.6), "door[38] E reach");
    open_door_facing(&mut w, [34.6, 41.4, 10.5], "door[38]");
    walk_plain(&mut w, [34.6, 41.4, 7.5], 0.35, "south of door[38], the shot line");
    let at = [35.6f32, 42.3, 17.47];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = &sim.drones.mines[mine_idx(&w, 383)];
        assert!(!m.exploded, "#383 still hangs before the shot");
        let bp = sim.bond.pos;
        let eye_y = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let d = ((at[0] - bp[0]).powi(2) + (at[1] - eye_y).powi(2) + (at[2] - bp[2]).powi(2)).sqrt();
        eprintln!("[mines] the range shot: bond {bp:?} eye_y {eye_y:.2} → #383 {at:?}, d {d:.2}");
        assert!(d > 8.0, "the shot is from RANGE ({d:.2} u — the task's > 8)");
        let di = walker::door_near(sim, [34.6, 41.4, 10.5]).expect("door[38]").1;
        assert!(sim.drones.doors.list[di].state & 0xc != 0,
            "door[38] is still open inside the auto-close window [M 0x13660c-0x136700]");
    }
    let d = shoot_mine(&mut w, 383, at, "the doorway fence #383 from 10 u");
    assert!(d > 8.0, "#383 takes the round from RANGE ({d:.2} u)");
    w.wait(10); // the blast's case-0x4B entries land on the neighbours' NEXT update [M 0x17c518]
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = &sim.drones.mines[mine_idx(&w, 383)];
        assert!(m.exploded, "#383 is gone (obj+0xFE bit 0 [M 0x17cb7c-0x17cb88])");
        assert_eq!(m.cause, nf_game::worldobj::MINE_BULLET,
            "the cause is the bullet's mirrored contact [M 0x1ea398:160-233]");
        // The blast-played proof is the neighbours' chain: #384/#385 can only explode through the
        // blast's own `Explode_Propagate` case-0x4B entries (a referenced HITDATA per mine in
        // reach, attacker = the exploding mine [M 0x17c518 case 0x4B; 0x17cde0]) — there is no
        // other writer of MINE_BLAST. (The `drones.explosions` list is the view's per-tick event
        // feed — the grenade section clears it every tick [M drone.rs `explosions.clear()`] — so
        // it is empty by the time the boom is observed and proves nothing here.)
        for plc in [384usize, 385] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BLAST,
                "#{plc} goes on #383's blast (cause {})", m.cause);
        }
        eprintln!("[mines] #383's blast chain: causes {:?}, bond health {:.1}",
            [384usize, 385].map(|p| sim.drones.mines[mine_idx(&w, p)].cause), sim.drones.health.health);
    }
}
