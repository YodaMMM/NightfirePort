//! guard-capsule-radii: the guard body capsule's per-state radii, reached on the real level
//! through the player's own inputs — no teleports after the load.
//!
//! The game's collision body (`NDrone2_Collision` 0x140e90) has three arms [M disasm]:
//! • DOWN (+0x2f, the tase/grenade/dart stun states): radius 0.3, both +0x3F0/+0x400 stacked at
//!   up·(0.3 − hip) — the collapsed body's single low ball (0x140fd4-88);
//! • voice set 0xC (+0xD8): radius 0.55 (0x141048-68);
//! • else 0.4 (0x14104c-58).
//! The player-vs-guard bump (`Drone_CollisionHandler` 0x140968's hit list → DealWithObjHit case 3,
//! 0x1405c8) fires while the player's body HITTEST (the r-0.55 walk arm, Player_Collision
//! 0x194df0) overlaps the guard's capsule. With the OLD port the down guard kept the standing
//! 0.4 capsule, so the bump lit at 0.55 + 0.4 = 0.95 u; the game's 0.3 ball lights it at
//! 0.55 + 0.3 = 0.85 u.
//!
//! THE PLAN (the test below is the harness; see the #[ignore] reason): walk Bond (real input) to
//! the nearest placed guard on 07000005, taser him through the held Stunner beam, wait for the
//! down state, then creep in with a partial stick (the analog walk is ~0.02 u/tick — finer than
//! the 0.10 u band) and record the guard's own +0x26 body-contact latch (the port's
//! `objhit_latch`) first firing distance. The game's ball fires it below 0.85 + a step; the old
//! standing capsule fired at ~0.95. The sister
//! `capsule_radii_follow_the_voice_set_and_the_down_flag` (nf-game) pins the 0.55/0.4 arms —
//! 07000016's #634 (skin 0x500005e → voice 0x13, dtype 0xc) is the game's 0.4 and stood 0.55 on
//! the old `dtype` key.
//! Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;
use walker::{wrap_pi, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// BLOCKED (2026-11-09, the first pass — the gatehouse route, answered this session): the walk
/// legs worked but (1) the gatehouse chase never closed the dy (the closing legs chased #6 from
/// y 26.2 to 18.7 — the courtyard guards flee downhill toward the gatehouse approach) and (2)
/// the Stunner beam's wall gate refuses a target across a ledge drop. THE CENSUS ANSWER (this
/// session): the Level::parse census of 07000005's 16 kind-0x0F placements [D] puts two guards
/// ON the courtyard west band — #5 skin 0x500002e at (61.34, 27.13, 99.50) and #6 skin
/// 0x5000030 at (50.85, 28.27, 93.63) — the same floor as the established courtyard-west nav
/// route's end (60.1, 28.4, 96.4) [D; the interact_07000005 ladder/pickup walks run that route
/// clean], and the walkreal_07000005_live fights engage these same guards at Bond's floor (the
/// nav 19/20 return-fire kills at (73.6, 25.0, 98.4) / (63.8, 26.7, 98.1) [L the walkreal run])
/// — they hold their post and fight; no downhill flee on the west approach. The harness walks
/// THAT route on real input, picks a same-floor guard with the |dy| < 1.5 finder, and runs the
/// 07000006 sister's proven tase + creep + latch sequence unchanged.
#[test]
fn a_down_guard_bumps_at_the_games_03_ball() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000005.bin", false).expect("load 07000005"));
    w.dmg.return_fire = false; // the walk scripts its own fight: the Stunner only
    w.wait_enabled(3000, "the Mayhew A start");
    w.wait(30);

    // The courtyard west route (the interact_07000005 walks' proven legs — the spawn floor is
    // the courtyard [D start #26 y 25.1]) — NOT the first pass's gatehouse leg: the two
    // same-floor subjects defend the west band, and the gatehouse chase ran them downhill.
    w.walk_nav_to([60.1, 28.4, 96.4], "courtyard west", 1.2);
    let gi = {
        let mut found = None;
        for _ in 0..8 {
            let sim = w.scene.sim.as_ref().unwrap();
            found = sim.drones.llist.iter().copied().find(|&i| {
                let dr = &sim.drones.list[i];
                dr.present && dr.enabled && !dr.corpse && dr.health > 0.0
                    && (dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2) < 900.0
                    // Same floor: the beam's wall gate refuses a guard across a ledge drop.
                    && (dr.pos[1] - sim.bond.pos[1]).abs() < 1.5
            });
            if found.is_some() { break; }
            drop(sim);
            // Nobody reachable yet: walk a short leg toward the nearest live guard (he may be
            // closing in himself) and look again.
            let sim = w.scene.sim.as_ref().unwrap();
            let Some(near) = sim.drones.llist.iter().copied().min_by(|a, b| {
                let key = |i: usize| {
                    let dr = &sim.drones.list[i];
                    if !(dr.present && dr.enabled && !dr.corpse && dr.health > 0.0) { return f32::MAX; }
                    let d2 = (dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2);
                    if d2 >= 2500.0 { f32::MAX } else { d2 + (dr.pos[1] - sim.bond.pos[1]).abs() }
                };
                key(*a).total_cmp(&key(*b))
            }) else { break };
            let gp = sim.drones.list[near].pos;
            drop(sim);
            w.walk_to(Wp::new(gp[0], gp[1], gp[2]).r(2.5), &format!("the closing leg toward #{near}"));
        }
        found.expect("a same-floor guard reached on 07000005's courtyard west band")
    };
    w.keep_alive.push(gi);
    eprintln!("[capsule] guard #{gi} at spawn dxz {:.2}", dxz(w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.list[gi].pos));

    // Walk to him — short legs re-aimed at his live position (he may patrol or close in), then
    // creep into the beam's band (the drop harness's 2.5 u setup) with the partial stick.
    for leg in 0..14 {
        let (bp, gp) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.drones.list[gi].pos) };
        if dxz(bp, gp) <= 2.6 { break; }
        w.walk_to(Wp::new(gp[0], gp[1], gp[2]).r(2.4), &format!("the approach leg {leg}"));
    }
    for _ in 0..1200 {
        let sim = w.scene.sim.as_ref().unwrap();
        let (bp, gp) = (sim.bond.pos, sim.drones.list[gi].pos);
        if dxz(bp, gp) <= 2.2 { break; }
        let turn = wrap_pi((gp[0] - bp[0]).atan2(gp[2] - bp[2]) - sim.bond.yaw).clamp(-0.08, 0.08);
        w.tick(Input { forward: 0.4, turn, ..Default::default() }, false);
    }
    let near = { let sim = w.scene.sim.as_ref().unwrap(); dxz(sim.bond.pos, sim.drones.list[gi].pos) };
    assert!(near <= 3.2, "Bond closed to the tase band, got {near:.2}");

    // The Stunner through the switch input (the 05 kit owns it), the draw finished before the
    // trigger press [M Player_WeaponFiring's view-state gate — the drop harness's equip].
    let equipped = (0..60).find(|_| {
        w.tick(Input { switch_to: Some(0x4a), ..Default::default() }, false);
        w.scene.sim.as_ref().unwrap().gun.weapon == 0x4a
    }).is_some();
    assert!(equipped, "the Stunner 0x4a equipped");
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.switch_state, 0, "the draw ended");

    // === the tase delivery (the 06 sister's proven shape — the held beam fires on the walked
    // Bond at this range; the first pass's blocker was the ledge-crossing aim, not the walk) ===
    // Hold the beam until the guard falls into TASER, then the 30-tick fall wait into STUNNED
    // [M 0x162b98 case 0xc — the drop harness's stun_to_taser, on the raw scene step].
    w.keep_alive.clear();
    { let mut sim = w.scene.sim.as_mut().unwrap(); let dr = &mut sim.drones.list[gi]; dr.health = dr.max_health; }
    let steer = |w: &mut Walker, fire: bool| {
        let (turn, look) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let g = sim.drones.list[gi].pos;
            let dy = g[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = (g[0] - bp[0]).atan2(g[2] - bp[2]);
            let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            ((desired - sim.bond.yaw).clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
        };
        w.scene.sim.as_mut().unwrap().step_bond(Input { turn, look, ..Default::default() }, fire, false, false);
    };
    for _ in 0..40 { steer(&mut w, false); } // the drop harness's aim settle
    let fell = (0..500).find(|t| {
        steer(&mut w, true);
        if t % 40 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            eprintln!("[capsule] tase t{t}: dxz {:.2} pitch {:.3} yaw {:.3} guard {:#x} accum {} clip {}", dxz(sim.bond.pos, dr.pos), sim.bond.pitch, sim.bond.yaw, dr.state, dr.tase_accum, sim.gun.clip);
        }
        w.scene.sim.as_ref().unwrap().drones.list[gi].state == state::TASER
    }).expect("the guard falls into DSTATE_Taser 0x4a under a held tase [M 0x1469b8 case 7]");
    for _ in 0..30 { steer(&mut w, true); }
    steer(&mut w, false);
    eprintln!("[capsule] guard #{gi} fell at hold tick {fell}");
    (0..240).find(|_| {
        steer(&mut w, false);
        let dr = &w.scene.sim.as_ref().unwrap().drones.list[gi];
        dr.state == state::STUNNED && dr.down
    }).expect("the fall ends into DSTATE_Stunned 0x4b with the +0x2f down flag [M 0x162eb0 case 1]");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[gi].down, "the down flag rides STUNNED [M 0x162eb0 case 1]");

    // THE PROBE: creep in with the partial stick (the analog walk ~0.02 u/tick — finer than the
    // 0.55+0.3 = 0.85 band) and record the guard's own body-contact latch (+0x26, DealWithObjHit
    // case 3, 0x1405c8) first firing distance. The AnyoneNear 4.0 band keeps him down while Bond
    // closes [M 0x162eb0 case 0xc / DroneFunc_AnyoneNear 0x14a718].
    let mut first_fire: Option<f32> = None;
    for _ in 0..1500 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            if dr.objhit_latch {
                first_fire = Some(dxz(sim.bond.pos, dr.pos));
                break;
            }
            if !dr.down { break; } // he recovered early: the window closed
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let (bp, gp) = (sim.bond.pos, sim.drones.list[gi].pos);
        let turn = wrap_pi((gp[0] - bp[0]).atan2(gp[2] - bp[2]) - sim.bond.yaw).clamp(-0.08, 0.08);
        w.tick(Input { forward: 0.4, turn, ..Default::default() }, false);
    }
    let d = first_fire.expect("the bump fired while Bond crept into the downed guard");
    eprintln!("[capsule] the down guard's bump first fired at dxz {d:.3}");
    // The game's 0.3 ball + the 0.55 walk arm fires within 0.85 + a step; the old standing
    // 0.4 capsule fired at ~0.95 (a 0.10 u gap, the walk's ~0.02 u/tick cannot skip).
    assert!(d < 0.90, "the down guard's bump fired at {d:.3} — the game's 0.3 ball fires it below 0.85 + a step (the old 0.4 capsule lit at ~0.95)");
    assert!(d > 0.5, "the latch fired inside the contact band, not at overlap: {d:.3}");
}

/// SPRINT-3 (2026-11-10): the 06 lead — the same proof on 07000006's START guards, the drop
/// harness's tase subjects ON THE SAME FLOOR as the walked approach [L the probe: Bond spawns
/// (57.18, 39.27, 22.61); #12 idles at (68.89, 39.0, 27.33), d 12.6, dy −0.27, hp 10, the SG5;
/// #13/#14 the same floor at d 25.8/26.7]. The 05 blockers were both terrain: the courtyard
/// guards sat below the plateau (the chase never closed the dy) and the beam's wall gate
/// refused across the ledge — 06's start has neither. The walk legs are the 05 harness's
/// proven pattern (the same-floor finder, the closing legs re-aimed at the live position, the
/// partial-stick creep into the 2.2 u band); the tase holds the beam through the real trigger
/// with the view-state diagnostics printed at every sample (the 05 first-pass blocker's
/// remaining probes, called for in that doc). The measurement is unchanged: the downed
/// guard's own +0x26 latch (DealWithObjHit case 3, 0x1405c8) first fires at the game's
/// 0.55 + 0.3 = 0.85 band (the old standing 0.4 capsule lit at ~0.95) while the AnyoneNear
/// 4.0 band holds him down [M 0x162eb0 case 0xc].
/// SPRINT-3 RUN-1 RESULT (green, 0.99 s): the walked approach closed to 2.23 u, the held beam
/// FIRED on the walked Bond (clip 100 → 99, tase_accum 1 — the 05 first-pass blocker never
/// bit on 06; the pre-hold view state printed view_script 100664459 the level's own idle,
/// fire_state false, switch 0), #12 fell TASER at hold tick 20 → STUNNED+down, and the
/// creep's latch first fired at dxz 0.771 — the game's 0.55 + 0.3 = 0.85 band, where the old
/// standing 0.4 capsule lit at ~0.95 [L cap06_run1].
#[test]
fn a_down_guard_bumps_at_the_games_03_ball_07000006() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000006.bin", false).expect("load 07000006"));
    w.dmg.return_fire = false; // the walk scripts its own fight: the Stunner only
    w.wait_enabled(3000, "the 06 start");
    w.wait(30);

    // The same-floor finder (the 05 harness's, with the 06 start's 30 u box — #12 idles 12.6 u
    // out; #13/#14 the same floor at ~26 u are the fallbacks).
    let gi = {
        let mut found = None;
        for _ in 0..8 {
            let sim = w.scene.sim.as_ref().unwrap();
            found = sim.drones.llist.iter().copied().find(|&i| {
                let dr = &sim.drones.list[i];
                dr.present && dr.enabled && !dr.corpse && dr.health > 0.0
                    && (dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2) < 900.0
                    && (dr.pos[1] - sim.bond.pos[1]).abs() < 1.5
            });
            if found.is_some() { break; }
            drop(sim);
            let sim = w.scene.sim.as_ref().unwrap();
            let Some(near) = sim.drones.llist.iter().copied().min_by(|a, b| {
                let key = |i: usize| {
                    let dr = &sim.drones.list[i];
                    if !(dr.present && dr.enabled && !dr.corpse && dr.health > 0.0) { return f32::MAX; }
                    let d2 = (dr.pos[0] - sim.bond.pos[0]).powi(2) + (dr.pos[2] - sim.bond.pos[2]).powi(2);
                    if d2 >= 2500.0 { f32::MAX } else { d2 + (dr.pos[1] - sim.bond.pos[1]).abs() }
                };
                key(*a).total_cmp(&key(*b))
            }) else { break };
            let gp = sim.drones.list[near].pos;
            drop(sim);
            w.walk_to(Wp::new(gp[0], gp[1], gp[2]).r(2.5), &format!("the closing leg toward #{near}"));
        }
        found.expect("a same-floor guard reached on 07000006's start")
    };
    w.keep_alive.push(gi);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[gi];
        eprintln!("[capsule06] guard #{gi} at ({:.2},{:.2},{:.2}) hp {:.0} state {:#x} w {:#x} spawn dxz {:.2} dy {:.2}",
            dr.pos[0], dr.pos[1], dr.pos[2], dr.health, dr.state, dr.weapon,
            dxz(sim.bond.pos, dr.pos), dr.pos[1] - sim.bond.pos[1]);
    }

    // The approach: short legs re-aimed at his live position, then the partial-stick creep
    // into the beam's band (the drop harness's 2.5 u setup).
    for leg in 0..14 {
        let (bp, gp) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.drones.list[gi].pos) };
        if dxz(bp, gp) <= 2.6 { break; }
        w.walk_to(Wp::new(gp[0], gp[1], gp[2]).r(2.4), &format!("the approach leg {leg}"));
    }
    for _ in 0..1200 {
        let sim = w.scene.sim.as_ref().unwrap();
        let (bp, gp) = (sim.bond.pos, sim.drones.list[gi].pos);
        if dxz(bp, gp) <= 2.2 { break; }
        let turn = wrap_pi((gp[0] - bp[0]).atan2(gp[2] - bp[2]) - sim.bond.yaw).clamp(-0.08, 0.08);
        w.tick(Input { forward: 0.4, turn, ..Default::default() }, false);
    }
    let near = { let sim = w.scene.sim.as_ref().unwrap(); dxz(sim.bond.pos, sim.drones.list[gi].pos) };
    assert!(near <= 3.2, "Bond closed to the tase band, got {near:.2}");

    // The Stunner through the switch input; the draw finished before the trigger press
    // [M Player_WeaponFiring's view-state gate — the drop harness's equip].
    let equipped = (0..60).find(|_| {
        w.tick(Input { switch_to: Some(0x4a), ..Default::default() }, false);
        w.scene.sim.as_ref().unwrap().gun.weapon == 0x4a
    }).is_some();
    assert!(equipped, "the Stunner 0x4a equipped");
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.switch_state, 0, "the draw ended");

    // The tase: the guard's health reset (the approach's return fire is off, but the 05
    // harness's pattern kept the subject full), then the held beam through the REAL trigger
    // with the view-state diagnostics printed (the 05 doc's probes).
    w.keep_alive.clear();
    { let mut sim = w.scene.sim.as_mut().unwrap(); let dr = &mut sim.drones.list[gi]; dr.health = dr.max_health; }
    let steer = |w: &mut Walker, fire: bool| {
        let (turn, look) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let g = sim.drones.list[gi].pos;
            let dy = g[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = (g[0] - bp[0]).atan2(g[2] - bp[2]);
            let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            ((desired - sim.bond.yaw).clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
        };
        w.scene.sim.as_mut().unwrap().step_bond(Input { turn, look, ..Default::default() }, fire, false, false);
    };
    for _ in 0..40 { steer(&mut w, false); } // the aim settle
    let sim0 = w.scene.sim.as_ref().unwrap();
    eprintln!("[capsule06] pre-hold: bond ({:.2},{:.2},{:.2}) view_script {:?} view_time {:.1} aim_anim {:?} fire_state {} switch {} clip {}",
        sim0.bond.pos[0], sim0.bond.pos[1], sim0.bond.pos[2], sim0.view_script, sim0.view_time,
        sim0.view_aim_anim, sim0.gun.fire_state, sim0.gun.switch_state, sim0.gun.clip);
    drop(sim0);
    let fell = (0..500).find(|t| {
        steer(&mut w, true);
        if t % 40 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            eprintln!("[capsule06] tase t{t}: dxz {:.2} pitch {:.3} yaw {:.3} guard {:#x} accum {} clip {} fire_state {} view_script {:?} view_time {:.1} aim_anim {:?}",
                dxz(sim.bond.pos, dr.pos), sim.bond.pitch, sim.bond.yaw, dr.state, dr.tase_accum,
                sim.gun.clip, sim.gun.fire_state, sim.view_script, sim.view_time, sim.view_aim_anim);
        }
        w.scene.sim.as_ref().unwrap().drones.list[gi].state == state::TASER
    }).expect("the guard falls into DSTATE_Taser 0x4a under a held tase [M 0x1469b8 case 7]");
    for _ in 0..30 { steer(&mut w, true); }
    steer(&mut w, false);
    eprintln!("[capsule06] guard #{gi} fell at hold tick {fell}");
    (0..240).find(|_| {
        steer(&mut w, false);
        let dr = &w.scene.sim.as_ref().unwrap().drones.list[gi];
        dr.state == state::STUNNED && dr.down
    }).expect("the fall ends into DSTATE_Stunned 0x4b with the +0x2f down flag [M 0x162eb0 case 1]");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[gi].down, "the down flag rides STUNNED [M 0x162eb0 case 1]");

    // THE PROBE: creep in with the partial stick and record the guard's own body-contact
    // latch (+0x26) first firing distance. The AnyoneNear 4.0 band keeps him down while Bond
    // closes [M 0x162eb0 case 0xc / DroneFunc_AnyoneNear 0x14a718].
    let mut first_fire: Option<f32> = None;
    for _ in 0..1500 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[gi];
            if dr.objhit_latch {
                first_fire = Some(dxz(sim.bond.pos, dr.pos));
                break;
            }
            if !dr.down { break; } // he recovered early: the window closed
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let (bp, gp) = (sim.bond.pos, sim.drones.list[gi].pos);
        let turn = wrap_pi((gp[0] - bp[0]).atan2(gp[2] - bp[2]) - sim.bond.yaw).clamp(-0.08, 0.08);
        w.tick(Input { forward: 0.4, turn, ..Default::default() }, false);
    }
    let d = first_fire.expect("the bump fired while Bond crept into the downed guard");
    eprintln!("[capsule06] the down guard's bump first fired at dxz {d:.3}");
    // The game's 0.3 ball + the 0.55 walk arm fires within 0.85 + a step; the old standing
    // 0.4 capsule fired at ~0.95 (a 0.10 u gap, the walk's ~0.02 u/tick cannot skip).
    assert!(d < 0.90, "the down guard's bump fired at {d:.3} — the game's 0.3 ball fires it below 0.85 + a step (the old 0.4 capsule lit at ~0.95)");
    assert!(d > 0.5, "the latch fired inside the contact band, not at overlap: {d:.3}");
}
