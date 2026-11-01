//! satchel-laser: the 4a lift charge under the laser watch — the burn sparks at the contact, the
//! scrambled countdown after the game's burn time, and the outcome (the charge never blows).
//!
//! The game's rules, cited (`Bullet_DoTrails` 0x12d890, the 0x3b-family tail):
//! - **The burn counter** — `BU+0x102` = 100 at `Bullet_init` for the 0x3b family [M 0x12c5a0:206].
//! - **The contact walk** — each entry on the charge's `obj+0xD0` contact list whose picker is a
//!   class-5 object with weapon id 0x4E takes `FRAME_RATE_MUL` (1.0) off `BU+0x102` (0x4F: 10)
//!   [M :452-466] and spawns `Emitter_Create(0xc00000a, entry+0x40, entry+0x10, …)` — the burn
//!   sparks at the contact's hit point [M :467-468]. The walk that takes the counter to ≤ 0 plays
//!   `Sound_Play3D(…, 0x5dd, obj+0x30)` [M :470-474].
//! - **The burn-out** — with `BU+0x102 < 1` the tail redraws the fuse to `Rand_Rand(0x1518) + 0x3c`
//!   EVERY tick and jumps past the beep and the contact walk (`goto LAB_0012e41c`) [M :424-433]: the
//!   digits (taken from the fuse before the redraw, :422) show a new random number every tick, no
//!   more sparks spawn, and the fuse (−1/tick) never reaches < 0 — the charge never explodes.
//! - The watch fires one round per held tick and each round's first segment is its contact
//!   (docs/systems/laser-watch-contact.md) — one contact per tick, so the 0x4E burn-out takes
//!   **100 ticks** of beam on the charge.
//!
//! The ride: Bond on the lift's boarding plate (`--bond 3.2,-7.0,-3.8,0`), no input until shooter
//! #286's charge lands on the cab roof [D 0700004a #286; M Shooter_Update 0x17d0c0]; then real
//! turn/look input aims at the charge and the trigger holds the drawn watch. Opt-in: NF_ISO.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const SPARKS: u32 = 0x0c00_000a;


fn charge(scene: &nf_view::scene::Scene) -> nf_game::bullet::Bullet {
    let s = scene.sim.as_ref().unwrap();
    s.drones.bullets.iter().find(|b| b.landed && b.family(&s.weapons) == 0x3b).cloned().expect("the landed charge")
}

/// The real aim input toward the charge from Bond's eye (turn/look only).
fn aim(scene: &nf_view::scene::Scene) -> Input {
    let s = scene.sim.as_ref().unwrap();
    let c = charge(scene).pos;
    let h = s.bond.head();
    let d = [c[0] - h[0], c[1] - h[1], c[2] - h[2]];
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

fn ride_aim_and_draw() -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a");
    scene.sim.as_mut().unwrap().bond = nf_game::player::Bond::new([3.2, -7.0, -3.8], 0.0);
    let mut landed = false;
    for _ in 0..4000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.drones.bullets.iter().any(|b| b.landed && b.family(&s.weapons) == 0x3b) { landed = true; break; }
    }
    assert!(landed, "shooter #286's charge lands on the cab within 4000 ticks of the ride");
    // The watch into the hand (the 4a loadout carries it [M 0x1a7ef8:318-319]).
    scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(0x4e), ..Default::default() }, false, false, false);
    // Aim with the real sticks, trigger up, until the view settles on the charge.
    for _ in 0..90 {
        let i = aim(&scene);
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
    }
    assert_eq!(scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the laser watch 0x4E is drawn");
    scene
}

#[test]
fn the_watch_beam_sparks_on_the_4a_charge_then_scrambles_its_digits_and_the_charge_never_blows() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = ride_aim_and_draw();
    let c0 = charge(&scene);
    {
        let s = scene.sim.as_ref().unwrap();
        let h = s.bond.head();
        eprintln!("[satchel-laser] eye→charge {:.2} units", ((c0.pos[0] - h[0]).powi(2) + (c0.pos[1] - h[1]).powi(2) + (c0.pos[2] - h[2]).powi(2)).sqrt());
    }
    // Sister premise: no beam yet → the counter is whole, no sparks, the digits are the fuse's seconds.
    assert_eq!(c0.burn, 100, "BU+0x102 starts at 100 [M 0x12c5a0:206]");
    assert!(!c0.defused);
    assert_eq!(sparks_near(&scene, c0.pos), 0, "no burn sparks before the beam touches the charge");
    let sec = (c0.fuse * 0.016_666_668) as i32;
    assert_eq!(c0.digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "the display counts the fuse");

    // Hold the beam on it. Contact ticks are counted off the counter itself.
    let mut first_contact: Option<u32> = None;
    let mut defused_at: Option<u32> = None;
    let mut max_sparks = 0usize;
    let mut fizzle = false;
    for t in 1..=400u32 {
        let i = aim(&scene);
        scene.sim.as_mut().unwrap().step_bond(i, true, false, false);
        let c = charge(&scene);
        if first_contact.is_none() && c.burn < 100 { first_contact = Some(t); }
        if let Some(t0) = first_contact {
            if defused_at.is_none() {
                // One contact per tick, 1 off per contact for the 0x4E [M :452-466].
                assert_eq!(c.burn as i32, 100 - (t - t0 + 1) as i32, "the counter loses 1 per beam tick (t{t})");
                // Sparks at the contact while the beam is on it [M :467-468] — the burst lives
                // ~10 ticks (def 0xc00000a life 0.167 s), so they are present every contact tick.
                let n = sparks_near(&scene, c.pos);
                assert!(n > 0, "burn sparks at the charge on beam tick {} (burn {})", t - t0 + 1, c.burn);
                max_sparks = max_sparks.max(n);
                // Sister: one contact short of 100, the charge still counts down normally.
                if c.burn == 1 {
                    assert!(!c.defused, "99 contacts do not burn the charge out");
                    let sec = (c.fuse * 0.016_666_668) as i32;
                    assert_eq!(c.digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "99 contacts: digits still = fuse/60");
                }
            }
            if defused_at.is_none() && c.defused {
                defused_at = Some(t);
                assert_eq!(c.burn, 0, "burned out at exactly 0");
                fizzle = scene.sim.as_ref().unwrap().drones.audio.active_ids().contains(&0x5dd);
            }
        }
        if let Some(td) = defused_at { if t > td + 30 { break; } }
    }
    let t0 = first_contact.expect("the beam reaches the charge (a contact took BU+0x102 below 100)");
    let td = defused_at.expect("the charge burns out under the beam");
    assert_eq!(td - t0 + 1, 100, "the 0x4E burn-out takes 100 beam ticks [M 0x12c5a0:206, :452-466]");
    assert!(max_sparks >= 2, "the per-tick bursts overlap while the beam holds (max {max_sparks})");
    assert!(fizzle, "the burn-out plays 0x5dd at the charge [M :470-474]");

    // After the burn-out: no new sparks (the walk is skipped [M :424-433]) — the old bursts die
    // within their 0.2 s life + the 3 dead ticks; the beam is still held here.
    for _ in 0..30 {
        let i = aim(&scene);
        scene.sim.as_mut().unwrap().step_bond(i, true, false, false);
    }
    let c = charge(&scene);
    assert_eq!(sparks_near(&scene, c.pos), 0, "no sparks once the charge is burned out, beam still on");

    // The scramble: every tick the digits show the previous tick's redraw — a fresh 60..5459 fuse.
    let mut seen = std::collections::HashSet::new();
    let mut prev_fuse = c.fuse;
    for _ in 0..12 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let c = charge(&scene);
        assert!(c.fuse >= 60.0 && c.fuse <= 5459.0, "redrawn fuse {} in 0x3c + Rand_Rand(0x1518) [M :425-432]", c.fuse);
        let sec = ((prev_fuse - 1.0) * 0.016_666_668) as i32;
        assert_eq!(c.digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "the digits read the previous redraw after the decrement [M :422]");
        seen.insert(c.digits);
        prev_fuse = c.fuse;
    }
    assert!(seen.len() >= 6, "the display scrambles — {} distinct readings over 12 ticks", seen.len());

    // The outcome: the charge outlives its original fuse by far and never explodes; Bond lives.
    let fuse_left = c0.fuse as u32;
    for _ in 0..fuse_left + 300 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let alive = s.drones.bullets.iter().any(|b| b.landed && b.family(&s.weapons) == 0x3b && b.state < 2);
        if !alive { break; }
    }
    let s = scene.sim.as_ref().unwrap();
    let ch = s.drones.bullets.iter().find(|b| b.landed && b.family(&s.weapons) == 0x3b);
    assert!(ch.is_some_and(|b| b.state < 2 && b.defused), "the burned-out charge is still on the roof, unexploded, {} ticks past its fuse", 300);
    assert!(s.drones.health.health > 0.0, "Bond is alive");
    assert!(!s.drones.channels.on(0x62), "no death channel");
}

#[test]
fn the_charge_left_alone_counts_down_without_sparks_and_explodes() {
    // Sister: the same ride with the watch drawn but the trigger up — no sparks, the counter stays
    // at 100, the digits track the fuse and the fuse runs out into the blast (state 2).
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = ride_aim_and_draw();
    let fuse = charge(&scene).fuse as u32;
    let mut blew = false;
    for _ in 0..fuse + 60 {
        let i = aim(&scene);
        scene.sim.as_mut().unwrap().step_bond(i, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let Some(c) = s.drones.bullets.iter().find(|b| b.landed && b.family(&s.weapons) == 0x3b) else { blew = true; break };
        if c.state >= 2 { blew = true; break; }
        assert_eq!(c.burn, 100, "no beam → the counter stays whole");
        assert_eq!(sparks_near(&scene, c.pos), 0, "no beam → no burn sparks");
        let sec = (c.fuse * 0.016_666_668) as i32;
        assert_eq!(c.digits, Some(((sec.max(0) / 10) as u8, (sec.max(0) % 10) as u8)), "the digits track the fuse");
    }
    assert!(blew, "the unburned charge explodes when its fuse runs out [M 0x12dd44-0x12dd58]");
}
