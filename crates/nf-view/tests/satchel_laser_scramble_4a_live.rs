//! satchel-laser-scramble: the PLAYER's placed satchel under the laser watch — the burn sparks at
//! the contact, the 100-contact burn-out, and the scrambled countdown after it.
//!
//! The merged satchel_laser_4a_live pins the burn walk for the shooters' charges (drone bullets of
//! the 0x3b family). The player's own satchel is the SAME game object — a fired `Bullet_init`
//! round [M 0x12c5a0:199-211: the 0x3b family's fuse `((id − 0x3b)·5 + 5)·60` = 300 for the fired
//! 0x3b and the burn counter BU+0x102 = 100] — but the port routes the fired gadget into the
//! thrown-grenade pool (scene.rs's shot block), which the burn walk used to skip: the placed
//! satchel showed no sparks and its digits kept counting normally. The port now burns it the same
//! way (drone.rs's 0x4E/0x4F pass over the grenade pool), and the burned-out tail redraws the fuse
//! to `Rand_Rand(0x1518) + 0x3c` every tick and never explodes [M Bullet_DoTrails 0x12d890:
//! 424-433 — the decompile's burn arm, mirrored from the bullet path's `tick_charge_fuse`].
//!
//! The ride (every step real input; no bond writes after the load): the spawn stands on the shaft
//! lift, which auto-descends ~0.0303 u/tick from ~t100 [L probe run]. The two 0xD2 shooters drop a
//! charge each on the cab [D #286/#287; M Shooter_Update 0x17d0c0] and their 900-tick fuses would
//! blast Bond off the ride (the walkreal_0700004a_satchel sister dies at the −95.85 root), so both
//! are burned under the beam like the merged test does. At the park the platform is static: the
//! lobby scene (ch 0x84 [D #62]) is skipped the Walker-default way (`scripts.skip_request`, the
//! game's own skip action [M nis.rs Input_Action(-1, 0x19, 1)]), then Bond draws the satchel,
//! throws it at the platform floor (the stick), draws the watch and burns it — 100 contact ticks
//! [M :452-466], sparks `Emitter_Create(0xc00000a)` every contact [M :467-468], the 0x5dd burn-out
//! [M :470-474], then the digits scramble.
//!
//! Opt-in: NF_ISO (the slim ISO is enough). NF_SILENT.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const SPARKS: u32 = 0x0c00_000a;

fn charge_grenade(scene: &nf_view::scene::Scene) -> nf_game::grenade::Grenade {
    scene.sim.as_ref().unwrap().drones.grenades.iter()
        .find(|g| g.weapon_type == 0x3b && g.state < 2).cloned()
        .expect("the player's placed satchel")
}

/// The real aim input toward `target` from Bond's eye (turn/look only).
fn aim(scene: &nf_view::scene::Scene, target: [f32; 3]) -> Input {
    let s = scene.sim.as_ref().unwrap();
    let h = s.bond.head();
    let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    let mut yaw_err = d[0].atan2(d[2]) - s.bond.yaw;
    while yaw_err > std::f32::consts::PI { yaw_err -= std::f32::consts::TAU; }
    while yaw_err < -std::f32::consts::PI { yaw_err += std::f32::consts::TAU; }
    let pitch_err = (d[1] / len).asin() / std::f32::consts::FRAC_PI_2 - s.bond.pitch;
    Input { turn: yaw_err.clamp(-0.06, 0.06), look: pitch_err.clamp(-0.06, 0.06), ..Default::default() }
}

fn sparks_near(scene: &nf_view::scene::Scene, p: [f32; 3]) -> usize {
    scene.sim.as_ref().unwrap().drones.fx.emitters.iter()
        .filter(|e| e.def == SPARKS)
        .filter(|e| ((e.pos[0] - p[0]).powi(2) + (e.pos[1] - p[1]).powi(2) + (e.pos[2] - p[2]).powi(2)).sqrt() < 0.6)
        .count()
}

/// Draw `id` through the switch machine (real switch input until the gun settles).
fn draw(scene: &mut nf_view::scene::Scene, id: u8) {
    for _ in 0..300 {
        let i = Input { switch_to: Some(id), ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
        let gun = &scene.sim.as_ref().unwrap().gun;
        if gun.weapon == id && gun.switch_state == 0 && gun.reload_state == 0 { return; }
    }
    panic!("the weapon {id:#04x} never drew");
}

/// Burn the landed shooter charge under the watch's beam: hold the trigger with the per-tick
/// re-aim until the walk defuses it (100 contacts [M 0x12d890:452-466]).
fn burn_shooter_charge(scene: &mut nf_view::scene::Scene) {
    for _ in 0..900u32 {
        let s = scene.sim.as_ref().unwrap();
        let target = s.drones.bullets.iter()
            .find(|b| b.landed && !b.defused && b.family(&s.weapons) == 0x3b)
            .map(|b| b.pos).expect("the shooter's landed charge");
        let i = aim(scene, target);
        scene.sim.as_mut().unwrap().step_bond(i, true, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.drones.bullets.iter().filter(|b| b.landed && b.family(&s.weapons) == 0x3b).all(|b| b.defused) { return; }
    }
    panic!("the shooter charge never burned out");
}

#[test]
fn the_placed_satchel_sparks_under_the_beam_then_scrambles_its_digits_and_never_blows() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_weapons(&iso(), "0700004a.bin", false, &[0x3b, 0x4e]).expect("load 0700004a");
    // The watch into the hand (the 4a loadout carries it [M 0x1a7ef8:318-319]).
    draw(&mut scene, 0x4e);

    // The ride: no input until the first shooter charge lands on the cab, then burn every charge
    // the shooters drop (their 900-tick fuses would blast Bond off the ride), down to the park.
    let y0 = scene.sim.as_ref().unwrap().bond.pos[1];
    for _ in 0..4000u32 {
        let s = scene.sim.as_ref().unwrap();
        if s.drones.bullets.iter().any(|b| b.landed && b.family(&s.weapons) == 0x3b) { break; }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    {
        let s = scene.sim.as_ref().unwrap();
        assert!(s.drones.bullets.iter().any(|b| b.landed && b.family(&s.weapons) == 0x3b),
            "the first shooter charge lands on the cab within 4000 ticks");
    }
    burn_shooter_charge(&mut scene);
    let mut park_ticks = 0u32;
    for _ in 0..6000u32 {
        if scene.sim.as_ref().unwrap().drones.bullets.iter().any(|b| b.landed && !b.defused) {
            burn_shooter_charge(&mut scene);
            park_ticks = 0;
            continue;
        }
        let y = scene.sim.as_ref().unwrap().bond.pos[1];
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let y2 = scene.sim.as_ref().unwrap().bond.pos[1];
        if y2 < y0 - 140.0 && (y2 - y).abs() < 1e-4 { park_ticks += 1; } else { park_ticks = 0; }
        if park_ticks > 120 { break; }
    }
    {
        let s = scene.sim.as_ref().unwrap();
        assert!(s.bond.pos[1] < y0 - 140.0, "the ride reached the park (y {:.2} from {y0})", s.bond.pos[1]);
        assert!(s.drones.health.health > 0.0, "alive at the park");
    }
    // The lobby scene (ch 0x84 is its trigger [D #62]) holds the player at the park — skip it the
    // Walker-default way (the game's skip action), then step until the hand-back.
    for _ in 0..600u32 {
        {
            let s = scene.sim.as_mut().unwrap();
            if !s.disabled { break; }
            s.drones.scripts.skip_request = true;
        }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    for _ in 0..600u32 {
        if !scene.sim.as_ref().unwrap().disabled { break; }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    assert!(!scene.sim.as_ref().unwrap().disabled, "the lobby scene hands back control");

    // The satchel into the pocket (the panel give — the real give path, after the loadout init),
    // then into the hand.
    let (_, _) = nf_view::panel::apply(&mut scene, &[
        ("do".to_string(), "weapon".to_string()), ("id".to_string(), "0x3b".to_string()), ("rounds".to_string(), "5".to_string())]);
    draw(&mut scene, 0x3b);
    // Throw at a spot 1 unit ahead of the feet (real aim), one trigger tick, wait for the stick on
    // the parked platform floor.
    let target = {
        let s = scene.sim.as_ref().unwrap();
        let (px, pz) = (s.bond.pos[0], s.bond.pos[2]);
        let (fy, cy) = (s.bond.yaw.sin(), s.bond.yaw.cos());
        [px + fy * 1.0, s.bond.pos[1], pz + cy * 1.0]
    };
    for _ in 0..30 { let i = aim(&scene, target); scene.sim.as_mut().unwrap().step_bond(i, false, false, false); }
    scene.sim.as_mut().unwrap().step_bond(Input { fire: true, ..Default::default() }, true, false, false);
    let mut spawned = false;
    let mut stuck = false;
    for _ in 0..300u32 {
        let i = aim(&scene, target);
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.drones.grenades.iter().any(|g| g.weapon_type == 0x3b) { spawned = true; }
        if s.drones.grenades.iter().any(|g| g.weapon_type == 0x3b && g.stuck) { stuck = true; break; }
    }
    assert!(spawned && stuck, "the thrown satchel sticks on the platform (spawned {spawned})");
    let c0 = charge_grenade(&scene);
    // Sister premise: whole counter, no sparks, the digits count the fuse.
    assert_eq!(c0.burn, 100, "BU+0x102 starts at 100 [M Bullet_init 0x12c5a0:211]");
    assert!(!c0.defused);
    assert_eq!(sparks_near(&scene, c0.pos), 0, "no burn sparks before the beam");
    let sec = ((c0.range - c0.travelled) * 0.016_666_668) as i32;
    assert_eq!(c0.phase.as_ref().unwrap().digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "the display counts the fuse");

    // The watch back into the hand, then hold the beam on the charge until the burn-out.
    draw(&mut scene, 0x4e);
    let mut first_contact: Option<u32> = None;
    let mut defused_at: Option<u32> = None;
    let mut max_sparks = 0usize;
    let mut fizzle = false;
    for tick in 1..=400u32 {
        let target = charge_grenade(&scene).pos;
        let i = aim(&scene, target);
        scene.sim.as_mut().unwrap().step_bond(i, true, false, false);
        let g = charge_grenade(&scene);
        if first_contact.is_none() && g.burn < 100 { first_contact = Some(tick); }
        if let Some(t0) = first_contact {
            if defused_at.is_none() {
                assert_eq!(g.burn as i32, 100 - (tick - t0 + 1) as i32, "the counter loses 1 per beam tick (t{tick})");
                let n = sparks_near(&scene, g.pos);
                assert!(n > 0, "burn sparks at the placed satchel on beam tick {} (burn {})", tick - t0 + 1, g.burn);
                max_sparks = max_sparks.max(n);
                if g.burn == 1 {
                    assert!(!g.defused, "99 contacts do not burn the charge out");
                    let sec = ((g.range - g.travelled) * 0.016_666_668) as i32;
                    assert_eq!(g.phase.as_ref().unwrap().digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "99 contacts: digits still = fuse/60");
                }
            }
            if defused_at.is_none() && g.defused {
                defused_at = Some(tick);
                assert!(g.burn <= 0, "burned out at the walk's end");
                fizzle = scene.sim.as_ref().unwrap().drones.audio.active_ids().contains(&0x5dd);
            }
        }
        if let Some(td) = defused_at { if tick > td + 30 { break; } }
    }
    let t0 = first_contact.expect("the beam reaches the placed satchel");
    let td = defused_at.expect("the placed satchel burns out under the beam");
    assert_eq!(td - t0 + 1, 100, "the 0x4E burn-out takes 100 beam ticks [M 0x12d890:452-466]");
    assert!(max_sparks >= 2, "the per-tick bursts overlap while the beam holds (max {max_sparks})");
    assert!(fizzle, "the burn-out plays 0x5dd at the charge [M :470-474]");

    // After the burn-out: no new sparks (the walk is skipped [M :424-433]); the old bursts die
    // within their life + the dead ticks. The beam is still held here.
    for _ in 0..30 {
        let target = charge_grenade(&scene).pos;
        let i = aim(&scene, target);
        scene.sim.as_mut().unwrap().step_bond(i, true, false, false);
    }
    let g = charge_grenade(&scene);
    assert_eq!(sparks_near(&scene, g.pos), 0, "no sparks once burned out, beam still on");

    // The scramble: the digits show the previous tick's redraw — a fresh 60..5459 fuse every tick.
    let mut seen = std::collections::HashSet::new();
    let mut prev_fuse = g.range - g.travelled;
    for _ in 0..12 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let g = charge_grenade(&scene);
        let fuse = g.range - g.travelled;
        assert!(fuse >= 60.0 && fuse <= 5459.0 && fuse.fract() == 0.0, "redrawn fuse {fuse} in 0x3c + Rand_Rand(0x1518) [M :425-432]");
        let sec = ((prev_fuse - 1.0) * 0.016_666_668) as i32;
        assert_eq!(g.phase.as_ref().unwrap().digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "the digits read the previous redraw after the decrement [M :422]");
        seen.insert(g.phase.as_ref().unwrap().digits);
        prev_fuse = fuse;
    }
    assert!(seen.len() >= 6, "the display scrambles — {} distinct readings over 12 ticks", seen.len());

    // The outcome: the charge outlives its original fuse by far and never explodes; Bond lives and
    // the mission stands.
    let fuse_left = (g.range - g.travelled) as u32;
    for _ in 0..fuse_left + 400 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if !s.drones.grenades.iter().any(|g| g.weapon_type == 0x3b && g.state < 2) { break; }
    }
    let s = scene.sim.as_ref().unwrap();
    let g = s.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).expect("the charge survives");
    assert!(g.state < 2 && g.defused, "the burned-out charge is still placed, unexploded, 400 ticks past its redrawn fuse");
    assert!(s.drones.health.health > 0.0, "Bond is alive");
    assert!(!s.drones.channels.on(0x62), "no death channel");
    assert_ne!(s.drones.mission.status(), 3, "the mission did not fail");
}
