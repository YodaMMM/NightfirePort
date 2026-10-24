//! level-4a-lift-4: the lift ride's timed satchel charges, defused with the laser watch.
//!
//! The chain, every fact tagged:
//! - **[D 0700004a.bin]** the two 0xD2 shooters "ShootyBangBang": #286 at (3.04, −48.40, −1.67)
//!   watches ch 0x22, #287 at (2.09, −117.78, −1.91) watches ch 0x4f, both weapon 0x3d.
//! - **[M Shooter_Update 0x17d0c0]** the watch channel high → one `Bullet_init` (the 0x3b-family
//!   weapon fires STRAIGHT DOWN, the matrix dir replaced by (0, −1, 0)), then `obj+0xFE |= 1`
//!   (one charge per shooter).
//! - **[M Bullet_init 0x12c5a0:199-211]** the 0x3b family's fuse = `((kind − 0x3b)·5 + 5)·60`
//!   ticks (weapon 0x3d → 900) and the burn counter BU+0x102 = 100.
//! - **[M Bullet_DoTrails 0x12d890 disasm 0x12dcc4-0x12dd5c]** the fuse ticks −1.0 per tick
//!   (`+0x70 & 0x200`, FRAME_RATE_MUL [L ee_start_05.bin @0x30d0d8]); at `fuse < 0` the round
//!   dies with `Explode_Create(weapon+0x08 radius = 8, weapon+0x0C damage = 2000, script
//!   0x6000052)` [L the weapon records].
//! - **[M Bullet_CollisionHandler 0x12ce80 @0x12d1fc]** the world hit for the `+0x70 & 0x100`
//!   family with no struck object → state 3 (landed) + the 0x4a7 rest clank; the landed charge
//!   rides the cab [L the live run: the charge on the roof at Bond's (1.799, −73.418, −3.453)].
//! - **[M Bullet_DoTrails 0x12e2f0-0x12e390]** the laser burn: a 0x4E watch round in the
//!   charge's contact list takes 1/tick off BU+0x102, the 0x4F takes 10; at ≤ 0 the charge
//!   never explodes (the game re-randomizes the fuse every tick [M 0x12e3cc]; the port latches).
//! - **[L fresh boot, PCSX2]** with no defusing, ch 0x62 rises at lift root ≈ −95.85 with the
//!   health already 0 — the fail rides the charge's 900-tick fuse (the shot at the ch-0x22
//!   hand-back, the blast ≈ +901, the death sequence to ch 0x62 ≈ +477 more [M death.rs's
//!   pane chain; the level-16 parity measured +451..454]) ≈ +1377 after the hand-back.
//!
//! Both tests ride with REAL INPUT ONLY (the board walk + the aim/fire inputs; no bond.pos
//! writes after the spawn). Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use nf_game::world::V3;
use walker::{wrap_pi, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The charge a landed satchel bullet carries [D #286/#287 + M the 0x3b family].
fn landed_charge(w: &Walker) -> Option<(usize, V3, f32, i16)> {
    w.scene.sim.as_ref().unwrap().drones.bullets.iter().enumerate()
        .find(|(_, b)| b.landed && !b.defused)
        .map(|(i, b)| (i, b.pos, b.fuse, b.burn))
}

/// The first landed charge, any state (the defused one included).
fn any_landed(w: &Walker) -> Option<(V3, f32, i16, bool)> {
    w.scene.sim.as_ref().unwrap().drones.bullets.iter()
        .find(|b| b.landed)
        .map(|b| (b.pos, b.fuse, b.burn, b.defused))
}

/// The aim input toward `target` from Bond's eye — real look/turn only.
fn aim_input(w: &Walker, target: V3) -> Input {
    let sim = w.scene.sim.as_ref().unwrap();
    let b = &sim.bond;
    let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    let want_yaw = d[0].atan2(d[2]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    let want_pitch = (d[1] / len).asin() / (std::f32::consts::FRAC_PI_2);
    let yaw_err = wrap_pi(want_yaw - b.yaw);
    let pitch_err = (want_pitch - b.pitch).clamp(-0.06, 0.06);
    Input { turn: yaw_err.clamp(-0.06, 0.06), look: pitch_err, ..Default::default() }
}

/// Board the lift (the walkreal_0700004a's first phase, the reveal plate under the first steps),
/// select the laser watch (0x4E [M 0x1a7ef8:318-319 the loadout tail equips it]), then ride with
/// no input. Returns (y_board, the ch-0x22 rise tick offset from the ride loop's start).
fn board_and_arm(w: &mut Walker) -> (f32, u32) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 2.0 && p[0] < 6.0 && p[1] > -9.0 && p[1] < -6.0 && p[2] > -6.0 && p[2] < -3.0,
            "the arrival spawns at the shaft's top, on the platform (#1 [D]), got {p:?}");
    }
    let mut fired_plate = false;
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { fired_plate = true; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(fired_plate && sim.drones.channels.on(0x4d), "the shaft's first plate #277 fires ch 0x4d under the first steps [D]");
    }
    let y_board = w.scene.sim.as_ref().unwrap().bond.pos[1];
    // The laser watch into the hand (the draw completes during the pre-descent window).
    w.tick(Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..120 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the laser watch 0x4E is drawn [M 0x1a7ef8:318-319]");
    w.skip_cutscenes = false; // the landing cut-scene plays (its end event writes ch 0x22 [D s27])
    (y_board, w.total_ticks)
}

#[test]
fn ride_the_lift_burn_both_satchels_with_the_laser_watch_and_reach_the_lobby_alive() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    let (y_board, t_board_end) = board_and_arm(&mut w);

    // The ride to the hand-back: the landing cut-scene's s27 writes ch 0x22 at its t575 [D] —
    // the lift SP #42's out channel AND shooter #286's watch [D]. No input while riding.
    let mut t22: Option<u32> = None;
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        if t22.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(0x22) { t22 = Some(w.total_ticks); }
        if t22.is_some() { break; }
    }
    let t22 = t22.expect("ch 0x22 rises at the landing scene's end [D 0x060006cf s27]");
    assert!(t22 > t_board_end, "the hand-back is after the board");

    // Shooter #286 fires on the rise [M 0x17d0c0]; the charge falls (0.0667 u/tick [L record])
    // onto the cab and LANDS (state 3 [M 0x12d1fc]) at #286's x/z [D].
    let mut landed: Option<(V3, f32, i16, bool)> = None;
    for _ in 0..600u32 {
        w.tick(Default::default(), false);
        if let Some(l) = any_landed(&w) { landed = Some(l); break; }
    }
    let (pos, fuse0, burn0, _) = landed.expect("the first charge lands on the cab [M 0x17d0c0 + 0x12d1fc]");
    assert!((pos[0] - 3.04).abs() < 1.2 && (pos[2] - (-1.67)).abs() < 1.2,
        "the charge rests at shooter #286's x/z (3.04, −1.67) [D], got {pos:?}");
    assert_eq!(burn0, 100, "the burn counter starts at 100 [M Bullet_init 0x12c5a0:211]");
    assert!(fuse0 < 900.0 && fuse0 > 0.0, "the fuse ticks down from the 900 init [M 0x12c5a0:203-207], got {fuse0}");

    // Burn it: hold the beam on the charge until the burn counter runs out [M 0x12e2f0-0x12e390,
    // 1/tick for the 0x4E]. The ride continues under the beam (the cab's key line).
    let mut defused = false;
    for k in 0..900u32 {
        let Some((cpos, cfuse, burn, def)) = any_landed(&w) else { break };
        if def || burn <= 0 { defused = true; break; }
        if k % 60 == 0 { eprintln!("[burn] k{k} fuse{cfuse:.0} burn{burn} charge_y{:.2} bond{:?} yaw{:.2} pitch{:.2}", cpos[1], w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().bond.yaw, w.scene.sim.as_ref().unwrap().bond.pitch); }
        let input = aim_input(&w, cpos);
        w.tick_fire(input);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(defused, "the watch's beam defuses the charge (100 contact ticks [M]) — burn {:?}",
            sim.drones.bullets.iter().find(|b| b.landed).map(|b| b.burn));
        assert!(sim.drones.health.health > 0.0, "the ride is alive through the first defuse");
        assert!(!sim.drones.channels.on(0x62), "no death channel through the first defuse");
    }

    // The ride on: plate #281 (y −124.10) writes ch 0x4f [D] → shooter #287 drops the second
    // charge at (2.09, −1.91) [D]. Burn it the same way.
    let mut t4f: Option<u32> = None;
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4f) { t4f = Some(w.total_ticks); break; }
        if w.scene.sim.as_ref().unwrap().bond.pos[1] < y_board - 147.0 { break; }
    }
    assert!(t4f.is_some(), "ch 0x4f fires at the −124.1 plate [D #281]");
    let mut landed2: Option<V3> = None;
    for _ in 0..600u32 {
        w.tick(Default::default(), false);
        if let Some((_, pos, _, _)) = landed_charge(&w) {
            if (pos[0] - 2.09).abs() < 1.2 && (pos[2] - (-1.91)).abs() < 1.2 { landed2 = Some(pos); break; }
        }
    }
    let pos2 = landed2.expect("the second charge lands at #287's x/z [D]");
    let mut defused2 = false;
    for _ in 0..900u32 {
        let Some((cpos2, _, burn, def)) = any_landed(&w) else { break };
        if def || burn <= 0 { defused2 = true; break; }
        let input = aim_input(&w, if (cpos2[0] - 2.09).abs() < 1.2 { cpos2 } else { pos2 });
        w.tick_fire(input);
    }
    assert!(defused2, "the second charge defuses under the beam [M]");

    // The ride's end: the platform parks 150 down (the keyed drop [D]); the rider's root parks
    // ≈147.6 below the board root. Walk off into the lobby (the walkreal's route). Alive, no
    // death channel, and the bottom plate's ch 0x84 is on [D #62].
    for _ in 0..1500u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.pos[1] < y_board - 146.5 { break; }
        w.tick(Default::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] < y_board - 145.0, "the ride covered the shaft (y {:.2} from {y_board})", sim.bond.pos[1]);
        assert!(sim.drones.health.health > 0.0, "alive at the ride's end");
    }
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, -2.5).r(1.0), "off the platform");
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, 2.0).r(1.2), "into the lobby");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x84), "the bottom plate #62 fired ch 0x84 [D] — the lobby scene's trigger");
        assert!(sim.drones.health.health > 0.0, "alive in the lobby");
        assert!(!sim.drones.channels.on(0x62), "no death channel on the defused ride");
        assert_ne!(sim.drones.mission.status(), 3, "the mission did not fail");
    }
}

#[test]
fn an_undefused_satchel_explodes_and_fails_the_mission_like_the_game() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    w.top_up_health = false; // the blast must be able to kill
    let (_y_board, _) = board_and_arm(&mut w);
    // (the watch stays drawn but never fires — the sister is the NO-defuse ride)

    let mut t22: Option<u32> = None;
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        if t22.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(0x22) { t22 = Some(w.total_ticks); break; }
    }
    let t22 = t22.expect("ch 0x22 rises at the landing scene's end [D 0x060006cf s27]");

    // No input, no defusing: the charge's 900-tick fuse runs out [M], the blast (radius 8,
    // damage 2000 [L records]) kills Bond, the death sequence runs, Player_HandleDeath writes
    // ch 0x62 [M 0x190bb0:32] and Mission_Update's ch-0x62 arm fails the mission [M 0x182c60].
    let mut health0_tick: Option<u32> = None;
    let mut t62: Option<u32> = None;
    let mut y_at_62: f32 = 0.0;
    for _ in 0..2600u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if health0_tick.is_none() && sim.drones.health.health <= 0.0 { health0_tick = Some(w.total_ticks); }
        if sim.drones.channels.on(0x62) {
            t62 = Some(w.total_ticks);
            y_at_62 = sim.bond.pos[1];
            break;
        }
    }
    let t62 = t62.expect("ch 0x62 rises after the undefused charge's blast [M 0x190bb0:32]");
    let health0 = health0_tick.expect("the health ran out before the death channel");
    assert!(health0 <= t62, "the kill precedes the death channel (health 0 at t{health0}, ch 0x62 at t{t62})");
    let since22 = t62 - t22;
    assert!((1200..=1600).contains(&since22),
        "the fail lands at the fuse (900 [M 0x12c5a0:203-207]) + the death sequence — {since22} ticks after the hand-back [L 1377]");
    assert!((-99.5..=-91.5).contains(&y_at_62),
        "the fail comes at the live's ride depth (y {y_at_62:.2} [L −95.85])");

    // Mission_Update's ch-0x62 arm: state 3, the fail flow [M 0x182c60:35-42].
    for _ in 0..120u32 { w.tick(Default::default(), false); if w.scene.sim.as_ref().unwrap().drones.mission.status() == 3 { break; } }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.status(), 3, "the mission fails [M 0x182c60]");
        assert!(sim.drones.health.health <= 0.0, "the health is gone");
    }
}
