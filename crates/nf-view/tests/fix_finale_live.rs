//! fix-finale-blockers: the mission blockers the per-level walkthroughs could not pass. Drives the
//! REAL path (`nf_view::load` / `load_with_carry` + `Sim::step_bond`) on the finale levels:
//!
//! - 07000016: the launch chain spawns the chaser (ch 0x81 → SP #262's pack cinema 0x06000631 →
//!   ch 185 → SP #272 "ShuttleLaunch" 0x0600090a → ch 240/242 → spawners #257/#293 [D]); the
//!   mission's defeat is the KikoDeathButton #693 (kiko-16-defeat: her death channel is param 12
//!   = 0 [D] — the old init-forced 0x90 stand-in is gone; docs/levels/07000016.md §3/§6).
//! - 0700001b: ch 0x0a (0x060008f0 t104 [D] — SP #211's second script, behind the missile
//!   choreography) spawns Drake (spawner #187, trigger 10 [D]) and enables him (sw_ch 10 [D]);
//!   his death (ch 0x0e [D]) fires SP #198 → NIS 0x06000a20 → ch 0x0b (t600) + ch 100 (t720).
//! - 0700004a: the lift ride (SP #42 → 0x06000571: the platform entity descends 150 units over
//!   4950 ticks [D keys]) and the lobby arrival (the nis.rs stand-in at the script's end).
//! - the unported start states 0x85 / 0xa7 / 0xb4 (kiko.rs).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::kiko as kiko_states;
use nf_game::player::{Input, RamSave};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn free(sim: &mut nf_view::scene::Sim, ticks: u32) -> bool {
    for _ in 0..ticks { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { return true; } }
    !sim.disabled
}

fn grenade_at(sim: &mut nf_view::scene::Sim, pos: [f32; 3]) {
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        pos, velocity: [0.0, 0.0, 0.0], speed: 0.0,
        // One tick from its fuse (travelled 59 of range 60): a grenade explodes only on its fuse, never on contact
        // (fix-weapons-all, `Bullet_CollisionHandler` 0x12ce80), so the drop goes off where it lands.
        travelled: 59.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
    });
}

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn wrap_pi(a: f32) -> f32 {
    const TAU: f32 = std::f32::consts::TAU;
    (a + std::f32::consts::PI).rem_euclid(TAU) - std::f32::consts::PI
}

/// The pack flight on real input (level-16-jetpack-flight), the walker-less shape of the walkreal
/// `fly_to`: the toggle edge once (the game's `Input_Action(3, 39, 4)` → `debug_scanmode ^= 1` →
/// ChangeSubState(5) [M Player_Update 0x193600:142-158]), the held (action 7 − action 8) thrust
/// to `ty` [M Player_SSScanMode 0x193098:121-124], the eased forward cruise to (`tx`,`tz`)
/// [M :64-110 + the 0x193600:284-295 integration], the toggle off, the landing wait. Every tick
/// is `Input` through `step_bond` — no pos writes; the motion is the substate's own.
fn fly_to(scene: &mut nf_view::Scene, tx: f32, ty: f32, tz: f32, r: f32, max: u32, label: &str) -> bool {
    let mut toggled = false;
    let mut off_sent = false;
    for t in 0..max {
        if scene.sim.as_ref().unwrap().disabled {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            continue;
        }
        if off_sent {
            let (grounded, d) = {
                let b = &scene.sim.as_ref().unwrap().bond;
                (b.grounded, ((b.pos[0] - tx).powi(2) + (b.pos[2] - tz).powi(2)).sqrt())
            };
            if grounded && d < r {
                let p = scene.sim.as_ref().unwrap().bond.pos;
                eprintln!("[fly] {label}: landed at ({:.2},{:.2},{:.2}) +{t} ticks", p[0], p[1], p[2]);
                return true;
            }
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            continue;
        }
        let (x, y, z, yaw, pitch, sub) = {
            let b = &scene.sim.as_ref().unwrap().bond;
            (b.pos[0], b.pos[1], b.pos[2], b.yaw, b.pitch, b.substate)
        };
        if !toggled {
            let mut input = Input::default();
            input.fly_toggle = true; // the one edge
            toggled = true;
            scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            continue;
        }
        if sub != nf_game::player::SS_FLY {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            continue;
        }
        let mut input = Input::default();
        if y < ty {
            input.turn = wrap_pi((tx - x).atan2(tz - z) - yaw).clamp(-0.07, 0.07);
            input.lift = 1.0;
        } else {
            let d = ((x - tx).powi(2) + (z - tz).powi(2)).sqrt();
            if d < r {
                input.fly_toggle = true; // the toggle off: the look re-derive + ChangeSubState(0) [M]
                off_sent = true;
            } else {
                input.turn = wrap_pi((tx - x).atan2(tz - z) - yaw).clamp(-0.07, 0.07);
                input.look = (0.0 - pitch).clamp(-0.02, 0.02);
                input.forward = 1.0;
            }
        }
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let p = scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[fly] {label}: {max}-tick cap at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    false
}

/// 07000016, the real defeat → objective path (kiko-16-defeat): the launch chain spawns the
/// chaser (Kiko, #261 — skin 0x0500001e, dtype 0xf, death channel param 12 = 0 [D]) and Bond
/// kills her: her death raises NOTHING (she is not the objective's writer — the pre-2026-10-20
/// port forced ch 0x90 onto her at init; that stand-in is removed). The mission's defeat is the
/// **KikoDeathButton #693** — the level's kind-0x29 switch, param 0 = ch 0x90, param 5 = the 0x8d
/// gate [D hash 020008c1; docs/levels/07000016.md §7]: Bond's own E on it → `Switch_Activate`
/// 0x1b02a8 toggles ch 0x90 (the gate ch 0x8d off [M 0x1b02a8:22]) → mux #618 (out 0x64 = ch 100
/// ← 0x90 [D]) → Mission_Update state 6 → the driving hand-off 0x7300017 [M 0x182c60:111-112].
/// Sisters: a pit-squad death (channels 23..26 [D]) never raises ch 0x90, and the chaser's OWN
/// death never does either — FAIL-FIRST: on the stand-in the kill assertion failed (the init
/// write raised 0x90 on her corpse).
#[test]
fn the_07000016_kiko_death_button_defeat_completes_the_final_mission() {
    let iso = iso();
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load 07000016");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free after the intro");
        // The silo plate #253 (0.45-4.62, -0.20..1.88, -154.93..-152.85 [D]) reveals the Kiko
        // objective (ch 0x8f [D]) — the walkthrough's descent.
        sim.bond = nf_game::player::Bond::new([2.5, 1.2, -153.9], std::f32::consts::PI);
        let mut input = Input::default();
        input.forward = 1.0;
        for _ in 1..=120 { sim.step_bond(input, false, false, false); if sim.drones.channels.on(0x8f) { break; } }
        // The launch chain's head: ch 0x81 (no writer on the disc data [D scans]) → SP #262 plays
        // the pack cinema 0x06000631 → ch 185 → SP #272 → 0x0600090a → ch 240/242 → the spawners.
        sim.drones.channels.set(0x81, 1, sim.drones.frame);
    }
    // Play the chain out: the cinema (750 ticks) + the shuttle script + the spawner waves.
    // The launch script's own exhaust gate: stream 6's event-0xd writes raise ch 0xf3 (243) at
    // script tick 1611 and drop it at 2170 [D 0x0600090a scriptlog + params] — the
    // HurtPoly-ShuttleLaunch #309 (gate 0xf3, 1000 damage [D]) is LIVE for those ~560 ticks, and
    // the pack cinema's snap leaves Bond on the silo floor INSIDE the volume (3.57, −23.18,
    // −179.55). The disc's order puts the pulse BEFORE the spawner writes (ch 0xf0 at tick 2180
    // [D]) — the chaser fight never overlaps it — and the mission can only complete from a live
    // Bond (Mission_Update 0x182c60 case 1 folds ch 100 → state 6, but its ch-0x62 fail arm —
    // Player_SetHealth(0), MissionState 3, LevelToEndTo 0x7000048 [M 0x182c60:21-43] — outranks
    // it). U-051 ANSWERED (docs/systems/hurt-volumes.md §U-051): the game's re-enable is the pit
    // floor — the cinema's stream-8 single key (3.57, −22.76, −179.55) [D pack_cinema_16_live]
    // teleported by Player_Enable 0x195498 [M] — and the survival is the PACK FLIGHT in the
    // 861-tick window. level-16-jetpack-flight ported the flight (the fly substate 5,
    // `Player_SSScanMode` 0x193098's bit-0-clear arm [M]): the test now FLIES the escape on real
    // input — the toggle edge, the held thrust out of the #309 stack, the eased cruise to the
    // silo plate, the toggle off — and the old documented [S] re-seat (2.5, 1.2, −153.9) is
    // REPLACED. The test still fires ch 0x81 from the upper floor instead of walking Bond to the
    // pack (the walkreal walk covers the walked reach).
    let mut ch243_y: Option<f32> = None;
    let chaser = {
        let mut found = None;
        let mut flown = false;
        for _ in 0..6000u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            if !flown && !scene.sim.as_ref().unwrap().disabled {
                // The PACK FLIGHT on real input: out of the #309 stack, over to the silo plate,
                // landed — the re-seat stand-in's replacement (no pos writes in the flight).
                flown = fly_to(&mut scene, 2.5, 2.2, -153.9, 1.2, 2600, "the pack flight out of the pit");
                assert!(flown, "the pack flight clears the pit and reaches the silo plate on real input (no pos writes) [M SS_FLY]");
            }
            let sim = scene.sim.as_ref().unwrap();
            if ch243_y.is_none() && sim.drones.channels.on(243) {
                ch243_y = Some(sim.bond.pos[1]);
            }
            // The chaser is the skin-0x1e dtype-0xf drone NOT in the death-ch 23..26 pit squad
            // (#294–297 [D]) — her param 12 is 0 [D #261], so this finds her with or without the
            // (removed) stand-in, which had made it 0x90.
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)) {
                found = Some(i);
                break;
            }
        }
        assert!(flown, "the flight ran");
        let y = ch243_y.expect("ch 0xf3 (243) rose in the launch chain [D 0x0600090a s6]");
        assert!(y > -21.3, "Bond flew above the #309 stack before the pulse (y {y:.2} at the ch-0xf3 rise)");
        assert!(scene.sim.as_ref().unwrap().drones.mission.status() != 3, "the flight survived the launch pulse (no MissionState 3)");
        found.expect("the launch chain spawns the chaser (spawner #257, group 6 [D])")
    };
    // Sister first: a non-chaser drone's death (the pit squad's ch 23..26 [D]) never raises ch 0x90
    // on its own. Re-baselined for fix-spawner-p7: the launch chain's ch 0x15 is the group-1
    // spawner's KILL channel [L-verified 07000005], and 07000016 is a keep level, so the chain's
    // own cleanup removes its undrawn spawns before this point — the sister throws at the nearest
    // remaining non-chaser (any present guard that is not the chaser herself, index-excluded),
    // re-reading his position per attempt like the chaser kill below.
    {
        let sim = scene.sim.as_mut().unwrap();
        let squad = sim.drones.list.iter().enumerate()
            .position(|(i, d)| i != chaser && d.present && !d.corpse)
            .expect("a non-chaser guard exists");
        let mut died = false;
        for _ in 0..4 {
            let p = sim.drones.list[squad].pos;
            grenade_at(sim, p);
            for _ in 0..200 {
                sim.step_bond(Input::default(), false, false, false);
                if sim.drones.list[squad].corpse { died = true; break; }
            }
            if died { break; }
        }
        assert!(died, "the grenade reaches the non-chaser guard");
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[squad].corpse, "the guard died");
        assert!(!sim.drones.channels.on(0x90), "a non-chaser death never completes the objective [D AND #271]");
    }
    // The chaser's own defeat through the game's explosive path (the ninja chain walks her, so
    // the throw re-reads her position each attempt) — and it raises NOTHING: her death channel is
    // param 12 = 0 [D #261]; ch 0x90's disc writer is the KikoDeathButton switch #693 [D], not
    // her health. FAIL-FIRST vs the removed stand-in: with the init-forced 0x90 this assert
    // failed on her corpse.
    {
        let sim = scene.sim.as_mut().unwrap();
        let mut died = false;
        for _ in 0..3 {
            let p = sim.drones.list[chaser].pos;
            grenade_at(sim, p);
            for _ in 0..300u32 {
                sim.step_bond(Input::default(), false, false, false);
                if sim.drones.list[chaser].corpse { died = true; break; }
            }
            if died { break; }
        }
        assert!(died, "the chaser dies at the silo");
    }
    // Death_Anim raises the death channel on its first UPDATE after the kill [M 0x1637f8 case 2]
    // — a few ticks after the corpse appears. Settle 120 ticks before reading the channels.
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[chaser].corpse, "the chaser is down");
        assert!(!sim.drones.channels.on(0x90), "the chaser's death raises ch 0x90 — her death channel is param 12 = 0 [D #261]; the writer is the switch #693 [D]");
        assert!(sim.drones.mission.status() != 6, "sister: the mission does not complete on the kill [D AND #271 waits for the button]");
    }
    // THE REAL DEFEAT: Bond's own E on the KikoDeathButton #693 — the switch's own position
    // (7.02, 1.16, −155.61) [D] is the pick target (`Switch_GetNearest`/the use sphere centre on
    // the placement origin), so the aim points AT it: the game's pick is a 1.0-radius sphere at
    // head + 1 unit forward [M 0x1a5170:100-108], which swallows any eye within 2.0 of the
    // origin. Teleport-reach only (the walkreal's full walk is parked on the pit geometry); the
    // press itself is the game's Player_Activate dispatch like the interactables test. The gate
    // ch 0x8d (the corridor touch #616) is OFF on this route — the switch is usable [M
    // Switch_Activate 0x1b02a8:22]. The tower-room slab at this spot leaks (the probe run: Bond
    // is shoved to (7.41, −153.78) and slides), so each press beat re-seats him 1.2 m north of
    // the button facing it — the walkreal-lift re-seat pattern for reach, with the E itself the
    // real input (the interactables test's shape, its aim pinned).
    let at = [7.02f32, 1.16, -155.61];
    let mut pressed = false;
    for k in 0..240u32 {
        if scene.sim.as_ref().unwrap().drones.channels.on(0x90) { pressed = true; break; }
        let (desired, want_pitch, input) = {
            let sim = scene.sim.as_ref().unwrap();
            let desired = (at[0] - sim.bond.pos[0]).atan2(at[2] - sim.bond.pos[2]);
            let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let to = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
            let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = [7.02, 1.2, -154.4];
            sim.bond.vel = [0.0; 3];
            sim.bond.yaw = desired;
            sim.bond.pitch = want_pitch;
            (desired, want_pitch, Input::default())
        };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, k % 6 == 0);
    }
    assert!(pressed, "the E on the KikoDeathButton never wrote ch 0x90 — Switch_Activate 0x1b02a8 must toggle param-0 [D #693 p0 = 0x90]");
    // ch 0x90 → mux #618 → ch 100 → Mission_Update state 6 [M 0x1827d8:155-160, 0x182c60:85-122].
    let mut complete = false;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x90), "the KikoDeathButton raises ch 0x90 [D #693 p0]");
        assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 ← 0x90) fires ch 100 [D #618]");
        assert!(complete, "the final mission completes (MissionState 6 [M 0x182c60:85-116])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
    }
}

/// 0700001b: ch 0x0a (the fight gate: NIS 0x060008f0 t104 [D]) spawns Drake (spawner #187,
/// trigger 10 [D]) and enables him (his wait-switch 10 [D]); his death (ch 0x0e [D]) fires
/// SP #198 → NIS 0x06000a20: t600 event-11 writes ch 0x0b ("Defeat Drake"), t720 ch 0x64 = 100
/// [D decode; Script_EventHandler 0x1d94c0 case 0x11] → MissionState 6 → 0x0730001b [M 0x182c60].
/// Sister: ch 0x0b waits for the NIS's t600, not the death tick.
#[test]
fn the_0700001b_drakes_defeat_ends_the_space_station() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free after the shuttle arrival");
        // The fight gate (the head of its chain is the missile choreography [D]; the port drives
        // the gate channel the way 0x060008f0's t104 event does).
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    let drake = {
        let mut found = None;
        for _ in 0..600u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            let sim = scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                found = Some(i);
                break;
            }
        }
        found.expect("the gate spawns Drake (spawner #187 [D])")
    };
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[drake].enabled, "the gate enables Drake (wait-switch 10 [D])");
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        let p = sim.drones.list[drake].pos;
        grenade_at(sim, p);
        let mut died = false;
        for _ in 0..300u32 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.drones.list[drake].corpse { died = true; break; }
        }
        assert!(died, "Drake dies");
    }
    // Death_Anim's msg-2 raises the channel on its first update [M 0x1637f8 case 2] — a few ticks
    // after the kill; the defeat NIS's own t600 write comes later.
    for _ in 0..100 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0e), "Drake's death channel fires ch 0x0e [D param 12 = 14]");
        assert!(!sim.drones.channels.on(0x0b), "ch 0x0b waits for the defeat NIS's t600 [D 0x06000a20] — sister");
    }
    // SP #198 plays 0x06000a20: t600 ch 0x0b, t720 ch 0x64 (100) [D].
    let mut complete = false;
    for _ in 0..1500u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0b), "the NIS's t600 event-11 writes ch 0x0b [D]");
        assert!(sim.drones.channels.on(100), "the NIS's t720 event-11 writes ch 0x64 = 100 [D]");
        assert!(complete, "the station mission completes [M 0x182c60]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    }
}

/// 0700004a: the lift (SP #42 "SCRIPTPLAYER_Lift", mode 1 [D]) plays 0x06000571 — the platform
/// entity descends 150 units over 4950 ticks [D keys] — and the ride's END is the game's own
/// chain (docs/systems/tower2-lift-ride.md §2): the script writes ch 0x40 at t1610 [D s5] (the
/// −52 doors unlock), which triggers the −52 landing cut-scene SP #289 (mask 1 = SP_Hit's
/// out-channel bit, out 0x40 [D placement; M 0x1dbda8 case 0 + 0x1dc470:17-19]); the scene
/// (pack 0780004a, 0x060006cf, 575 ticks [D]) plays with Bond disabled and its LAST event writes
/// ch 0x22 = 1 at t575 [D s27], which releases the lift's t1611 hold (mask&1 → SP_UnPause
/// [M 0x1dbda8 case 2; 0x1dca58]) and the platform rides on to the lobby. The script's end writes
/// NO channel: ch 0x4a completes on 07000013's spawn touch #153 [D 07000013.bin;
/// walk_07000013_live]. Sisters: the t1610 writer fires first (the doors' own consumer is pinned
/// in interact_0700004a_live), and the reveal (ch 0x4d) stays on from the descent.
/// Fails on the removed stand-ins: the old code wrote ch 0x22 two ticks after the hold and
/// ch 0x4a at the script's end.
#[test]
fn the_0700004a_lift_ride_reaches_the_lobby() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).expect("load 0700004a");
    // Bond boards the platform at once — the lift script starts at t0 (mode 1 [D]).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
    }
    let y0 = scene.sim.as_ref().unwrap().bond.pos[1];
    // Ride the whole script (5040 ticks) — Bond is re-seated onto the PLATFORM each segment
    // (the platform's own keyed matrix — the lift SP's live_matrix, Script_Interp 0x1d9a08's
    // stream-0 keying), so the plates fire as the car passes them and the shaft garrison's fire
    // is shrugged off (the ride is the system under test, not the fight). The health top covers
    // the ride; the re-seat is the walkreal-c documented shape for the unported shaft-door
    // auto-open (docs/levels/0700004a.md §6).
    let mut fired = Vec::new();
    let (mut t40, mut t22, mut t_disabled) = (None, None, None);
    let mut resumed_at: Option<u32> = None;
    let mut py_start: Option<f32> = None; // the platform entity's keyed y at the ride's start
    // 5400 + the landing hold's 576 ticks: the script's clock freezes during the scene, so its
    // wall-clock end lands ~t5616.
    for t in 0..5900u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.drones.health.health = 100.0;
            if t % 30 == 0 {
                if let Some(y) = platform_y(sim) {
                    sim.bond.pos = [3.0, y + 1.2, -3.4];
                }
            }
        }
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x40) && t40.is_none() { t40 = Some(t); }
        if sim.drones.channels.on(0x22) && t22.is_none() { t22 = Some(t); }
        if sim.drones.channels.on(0x4d) && !fired.contains(&0x4du32) { fired.push(0x4d); }
        for ch in [0x3du32, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f, 0x84] {
            if sim.drones.channels.on(ch) && !fired.contains(&ch) { fired.push(ch); }
        }
        if sim.disabled && t_disabled.is_none() && t40.is_some() { t_disabled = Some(t); }
        // The resume: after ch 0x22 the platform's keyed entity falls again (the stream-0 keys
        // of the platform's own stream — the SP's live_matrix is a helper stream at the origin).
        if py_start.is_none() { py_start = platform_y(sim); }
        if t22.is_some() && resumed_at.is_none() {
            if let Some(y) = platform_y(sim) {
                RESUME_Y.with(|c| {
                    let mut b = c.borrow_mut();
                    if let Some(prev) = *b { if y < prev - 0.005 { resumed_at = Some(t); } }
                    *b = Some(y);
                });
            }
        }
        if t == 3000 {
            assert!(!sim.drones.channels.on(0x4a),
                "ch 0x4a waits for 07000013's spawn touch #153 — the ride's end writes NO channel [D 07000013.bin] — sister");
        }
        if sim.drones.channels.on(0x4a) { panic!("ch 0x4a fired on 0700004a — the stand-in is back"); }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let t40 = t40.expect("the lift script's t1610 event writes ch 0x40 [D 0x06000571 s5]");
        let t22 = t22.expect("the landing cut-scene's end event writes ch 0x22 [D 0780004a.bin 0x060006cf s27]");
        // The hold: ch 0x22 comes from the 575-tick scene's LAST event, not from the hold itself.
        assert!((1600..=1640).contains(&t40), "ch 0x40 at the script's t1610 [D], got t{t40}");
        let gap = t22 - t40;
        assert!((500..=760).contains(&gap),
            "ch 0x22 rides the landing scene's 575-tick end [D 0x060006cf s27 t575], got +{gap} after ch 0x40 — the old stand-in wrote it at +2");
        // The scene took control: Bond disabled somewhere in the scene's window [D s9 t0 event 0x04].
        let td = t_disabled.expect("the landing cut-scene disables Bond [D 0x060006cf s9 t0]");
        assert!(td > t40 && td < t22, "the disable sits inside the scene window (t{td} vs {t40}..{t22})");
        // The lift resumed after the scene's end and parked at the lobby.
        let resumed = resumed_at.expect("the mask&1 rule releases the hold once ch 0x22 is on [M 0x1dbda8 case 2]");
        assert!(resumed >= t22, "the resume follows the ch-0x22 write (t{resumed} vs t{t22})");
        assert!(fired.contains(&0x3d) || fired.contains(&0x17), "descent plates fire during the ride ([D] floors), got {fired:?}");
        let lift = sim.drones.scripts.players.iter().find(|p| p.scripts[0] == 0x0600_0571).expect("the lift SP");
        assert!(lift.deleted, "the lift SP deletes at the script's end (stay=1 → state 5 [M 0x1dbda8 case 2])");
        let py = platform_y(sim).expect("the platform entity (0x020005b4 [D 0x06000571]) still exists");
        let p0 = py_start.expect("the platform entity existed during the ride");
        assert!((py - (p0 - 150.0)).abs() < 3.0,
            "the platform parked 150 units down at the lobby (y {py:.2}, started {p0:.2}) [D 0x06000571 keys: 150 units t38→t5039]");
        assert!(!sim.drones.channels.on(0x4a), "ch 0x4a is OFF at the ride's end — the real writer is 07000013's spawn touch [D]");
        assert_eq!(sim.drones.mission.states[17][0], 3, "'Gain access to lobby' stays revealed, not complete, on 0700004a [L record 4a]");
        assert!(sim.drones.channels.on(0x4d), "the reveal (ch 0x4d) holds from the descent [D plate #277]");
    }
}

thread_local! {
    static RESUME_Y: std::cell::RefCell<Option<f32>> = const { std::cell::RefCell::new(None) };
}

/// The lift platform's keyed entity y — the Entity stream's object 0x020005b4, obj_type 31
/// [D 0x06000571; docs/levels/0700004a.md §6], whose keys run t38..t5039 (the 150-unit descent).
fn platform_y(sim: &nf_view::scene::Sim) -> Option<f32> {
    sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_05b4, matrix, .. } => Some(matrix[3][1]),
        _ => None,
    })
}

/// The unported start states (fix-levels-sweep's table): 0xb4 (07000007) despawns at once
/// [M 0x173f00], the mode-7 0x85 drones on 07000011/12/13/15 resolve to Idle/Alert/Attack — never
/// stuck [M 0x168910:32-36] — and 07000004's assassin (dtype 0xf → 0xd → 0xa6/0xa7 [M 0x14c938])
/// enters the ninja chain at alert 1.0.
#[test]
fn the_unported_start_states_behave_like_the_game() {
    let iso = iso();
    // 07000007: the k16=5 placement (mode 21 → DeleteMe [L DroneTypeSettings[21]]) is gone.
    {
        let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 1200), "Bond is free");
        let deleted = sim.drones.list.iter().filter(|d| d.mode == 21).count();
        let gone = sim.drones.list.iter().filter(|d| d.mode == 21 && !d.present).count();
        assert!(deleted > 0 && gone == deleted, "every mode-21 (0xb4) party placement despawned [M 0x173f00], {gone}/{deleted}");
    }
    // 07000011: the mode-7 drones (GetDTYPENEW case 3 → 7 [M 0x14c938]) stand in a game state.
    {
        let mut scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 1800), "Bond is free");
        let m7: Vec<u8> = sim.drones.list.iter().filter(|d| d.present && d.mode == 7).map(|d| d.state).collect();
        assert!(!m7.is_empty(), "07000011 has mode-7 drones [D sweep]");
        // With `FindRunToPoint` wired (ai-points): EnemyRunToPoint's found arm goes
        // to GoToGoalPosition 99 (class byte +0x44 == 1) or HostageGoToGoalPosition 100, the
        // none-found arm to the return state [M 0x168910:32-45 — ACTION.ELF decompile read at the
        // merge]; the old pin assumed none-found. Either way never stuck in 0x85.
        assert!(m7.iter().all(|&s| matches!(s, nf_game::drone::state::IDLE | nf_game::drone::state::ALERT | nf_game::drone::state::ATTACK | nf_game::drone::state::WAIT_SWITCH | nf_game::drone::state::GO_TO_GOAL | nf_game::drone::state::HOSTAGE_GO_TO_GOAL)),
            "the 0x85 drones resolved per [M 0x168910:32-36] (the gate-waiters sit in WaitSwitch 0x01 [M]), got {m7:?}");
    }
    // 07000004: the assassin (dtype 0xf [D]) is in the ninja chain at alert 1.0.
    {
        let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 1800), "Bond is free");
        let ninja = sim.drones.list.iter().find(|d| d.present && d.dtype == 0xf).expect("the assassin exists [D]");
        // ninja-melee: the full melee chain is ported — the fight may sit in any of its states
        // (kiko.rs documents the addresses; the set is the StateFuncs table [L 0x29b300]).
        assert!(kiko_states::NINJA_FIGHT_STATES.contains(&ninja.state),
            "the assassin runs the ninja chain [M 0x16bb50], state {:#x}", ninja.state);
        assert!((ninja.alert - 1.0).abs() < 1e-4, "NinjaAttack sets alert 1.0 [M 0x16bb50]");
    }
}
