//! walkreal-h: 07000008 (Mayhew D, the gondola) walked with the player's own movement — no
//! teleports after the spawn, NO channel or state pokes in the walk: the whole mission from one
//! fresh load, both objectives in player order, ending on the level exit firing
//! (level-08-every-objective). Route (docs/levels/07000008.md, the interactables census):
//!
//! 1. the mountain-top station #0 (139.5, 50.5, −76.4) [D placements kind 0x24] → the intro plate
//!    TouchOnce #237 (139.2, 49.5, −81.2) → **ch 0x49** (this completes 07000007's carried
//!    "Spy on secret meeting" objective [L ee_start_05 record 2]) + SP #294 (0x06000077, the
//!    station intro, 3880 ticks [M levels.md §5.1]) plays, skipped only with the game's own skip;
//! 2. down the station/walkway (the ch-0x2a zone plates wake the placed guards, sw_ch 42 [D]) to
//!    the fusebox #219 (FUSEBOX_GuardRoomDoorLock, group-0 channel 137 [D]) — Bond SHOOTS its
//!    wires (the gun contact survives the wire filter [M fusebox.rs Collide_FilterBullets]) →
//!    **ch 137** unlocks the Secure Room door #122/10 (lock ch 137 [D]) and starts SP #221
//!    (GuardRoomLights);
//! 3. inside, the LASER (0x4E, owned [D inventory]) burns the safe bolts #207/#208 (Lock_Create
//!    0x1ae1f8, out 134/135 [M Lock_Update 0x1ae3d8:194]) → mux #278 → **ch 136** opens the safe;
//! 4. the briefcase pickup #290 (kind 0xF0 type 4, switch channel 74 = 0x4a [D]) → the take writes
//!    **ch 0x4A** [M Pickup_Handler 0x18e650:330-333] → objective "Retrieve guidance chip" folds;
//! 5. back to the gondola station: the floor plate Touch #236 (out 4, mask 8 [D]) → ch 4; mux #235
//!    (out 1 = 0x4A AND 4) → **ch 1** starts SP #295 ScriptPlayer_CableCar (mode 64 loaded-armed
//!    [M SP_LoadScript 0x1dbc08]; the ride script 0x0600004C lives in the LEVEL BIN, 1420 ticks)
//!    — the carriage (112.0, 32.4, −126.3) → (112.0, 18.0, −179.4) [D stream 1] carries Bond
//!    (the game's riding [M Control_InheritVelocity 0x133e88]) — its stream 46 writes **ch 83
//!    (0x53)** at tick 1410;
//! 6. ch 83 starts SP #296 (0x06000078, the bottom-of-mountain cinematic, 342 ticks) whose
//!    stream 10 writes **ch 5**; the helicopter #45 (Copter_Create 0x17d340: gate ch 5, out
//!    ch 0x4B [M record+0x70/+0x72; Copter_Update 0x17dbb8 cases 4/5/6]) flies and writes
//!    **ch 0x4B** → objective "Escape with Zoe in gondola" folds → mux #276 → **ch 100** →
//!    Mission_Update ends the level into the DRIVING hand-off **0x0730008**
//!    [M 0x182c60:85-116 — LevelToEndTo; no LOAD_LEVEL placement exists on this level [D]].
//!
//! The level's mission data lists exactly TWO objectives for 07000008
//! [L ee_start_05.bin @ 0x2a3e88, 2 records; the states Mission_MonitorObjectives 0x1827d8 runs]:
//! 1. "Retrieve guidance chip from Safe Room" (ch 0x4A, enabled by ch 0x49) — completes at the
//!    briefcase take (leg 4);
//! 2. "Escape with Zoe in gondola" (ch 0x4B, enabled by ch 0x4A) — completes at the Hind's
//!    death tail (leg 6, the only ch-0x4B writer on this level [M Copter_Update's state-4 arm;
//!    D the route dump: no kind-2 arrival path]).
//!
//! The carried latches (05's 0x45/0x70, 06's 0x02, 07's 0x48) are the mission's ENTRY state — in
//! play they ride the RamSave [M Player_RamSave 0x195670 / Player_RamLoad 0x1957a8]; the cold
//! test load sets them once at spawn (the same set the cold stand-in walk_07000008_live pins),
//! never during the walk. The old separate ride test's ch-0x4A stand-in poke is gone: the real
//! ch 0x4A from the briefcase take feeds mux #235 (level-08-every-objective).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission objective states for 07000008 (LEVELS[3] [M mission.rs LEVELS table]).
fn obj08(w: &Walker) -> (u32, u32) {
    let m = &w.scene.sim.as_ref().unwrap().drones.mission;
    (m.states[3][0], m.states[3][1])
}

/// Face `target` from Bond's head (turn + look), then hold fire for `ticks` — the REAL fire input
/// drives the gun bullets (the fusebox wires) and the laser beam (the safe bolts' burn [M
/// Player_LaserPointer 0x1a65d8's hook; walker.rs tick_fire]).
fn aim_and_fire(w: &mut Walker, target: [f32; 3], weapon: u8, fire_ticks: u32) {
    aim_and_fire_until(w, target, weapon, fire_ticks, |_| false);
}

/// Hold the fire until `done` holds (a player lets go once the lock or wire goes) — the laser's
/// meter drains ~31/s while held [D docs/live/2026-10-14-weapon-measurements-pcsx2.md].
fn aim_and_fire_until(w: &mut Walker, target: [f32; 3], weapon: u8, fire_ticks: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) {
    // The switch through the real weapon-change input.
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == weapon { break; }
        w.tick(Input { switch_to: Some(weapon), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, weapon, "the switch to {weapon:#x} never landed");
    for t in 0..fire_ticks {
        let sim = w.scene.sim.as_ref().unwrap();
        if done(sim) { break; }
        let head = sim.bond.head();
        let to = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let desired_yaw = to[0].atan2(to[2]);
        // `bond.pitch` is BL+0x8A8 in units of π/2 (player.rs) — the angle converts.
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let pitch_err = desired_pitch - sim.bond.pitch;
        let input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick_fire(input);
        if t % 120 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 aim t{t}] bond {:?} yaw {:.2} pitch {:.2} weapon {:#x}", sim.bond.pos, sim.bond.yaw, sim.bond.pitch, sim.gun.weapon);
        }
    }
}

/// The shoot-through check for a rocket launch: the round flies the straight corridor its
/// muzzle opens [M Bullet_update 0x12e7b0's per-tick world segment + Collide_Pick 0x1eb320],
/// so a launch whose line the bottom-station's own geometry blocks explodes at the contact —
/// and at 0.65–5.5 u from Bond that blast (radius 8, ≈100 [L ee_start_05.bin @0x2bf150 +
/// 0x2C·0x10C], the (int)-truncated linear falloff × the level's 0.6 pain factor [M
/// Explode_Propagate 0x17c518:69-71; Player_HandlePain's table]) takes Bond with it: the
/// measured death run's four "world" hits −52.5/−7.5/−22.5/−3.7 at frames 26477/26580/26639/
/// 26696 are Bond's OWN rockets 1/3/4 (docs/reports/level-08-long-range-hits.md). A player
/// pulls the trigger only on a clear line — the eye ray AND both gun stand-ins (the view
/// model's low-right/left offsets; the lane-fights' rule — at a wall edge the eye ray clears
/// by a hair while the gun ray clips the corner [L the level-13 rf-trace]).
fn launch_line_clear(sim: &nf_view::scene::Sim, to: [f32; 3]) -> bool {
    let b = sim.bond.pos;
    let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
    let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
    let right = [cy, 0.0, -fy];
    let spots = [eye,
        [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
        [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]];
    spots.iter().all(|&from| {
        sim.world.segment_bullet_scoped(sim.world.find_cel(from), from, to)
            .is_none_or(|t| t >= 0.999)
    })
}

/// A player does not board the gondola with a burst-rifle pair alive at 4 u behind him: close
/// guards (team 1, within 12 u horizontal / 4 u vertical of Bond) are answered with the GUN
/// (`switch_to_gun` → `stand_and_fire`, the shared fight machinery) before the walk goes on.
/// REAL-DAMAGE ONLY (gated on `tops_up()`): the tactic exists for the run that has to survive
/// its own damage; the topped-up sister's fight timeline stays exactly as pinned.
fn fight_close_guards(w: &mut Walker, label: &str, max_ticks: u32) {
    if w.tops_up() { return; }
    let close = |sim: &nf_view::scene::Sim| -> Option<usize> {
        let b = sim.bond.pos;
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1)
            .filter(|(_, d)| ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < 12.0 && (d.pos[1] - b[1]).abs() < 4.0)
            .min_by(|x, y| {
                let dx = (x.1.pos[0] - b[0]).powi(2) + (x.1.pos[2] - b[2]).powi(2);
                let dy = (y.1.pos[0] - b[0]).powi(2) + (y.1.pos[2] - b[2]).powi(2);
                dx.total_cmp(&dy)
            })
            .map(|(i, _)| i)
    };
    let start = w.total_ticks;
    let mut skipped: Vec<usize> = Vec::new();
    loop {
        if w.total_ticks - start >= max_ticks {
            eprintln!("[08 guards] {label}: fight cap at tick {} (health {:.1})", w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            break;
        }
        let Some(di) = close(&w.scene.sim.as_ref().unwrap()) else { break };
        if skipped.contains(&di) && skipped.len() >= w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1).count() {
            break; // every close guard has had its stand; walk on
        }
        eprintln!("[08 guards] {label}: the close guard #{di} ({:?}) at tick {}, health {:.1} — the gun first",
            w.scene.sim.as_ref().unwrap().drones.list[di].pos, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        let killed = w.switch_to_gun(label) && w.stand_and_fire(di, 900);
        // Level the view again before walking on (a fight leaves the pitch on the guard).
        for _ in 0..90 {
            let p = w.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            w.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        eprintln!("[08 guards] {label}: guard #{di} {} (killed {killed}) at tick {}, health {:.1}",
            if killed { "down" } else { "held (out of sight or cap)" }, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        if !killed { skipped.push(di); }
    }
}

/// An owned FIREARM with rounds in the pool — the AT-420 launcher (0x2c) EXCLUDED: its rounds
/// are the Hind shoot-down's (the walk asserts the rockets on hand at the fight) and its blast
/// at a 9.8 u guard is Bond's own geometry (radius 8, ≈100 [L ee_start_05.bin + 0x2C·0x10C]).
/// The 25773 death's trap: `DamageLog::stand_and_fire` fires whatever is in hand, and the dry
/// pistol's `switch_to_armed` wheel (2..=0x47) counts the owned launcher armed — the stand drew
/// it instead of a gun [L the 2026-11-03 run log: cycling (0) 0xc 0/0, (1) 0x1 clip 1 → the next
/// owned stop is 0x2c].
fn firearm_armed(sim: &nf_view::scene::Sim) -> bool {
    let inv = &sim.drones.inventory;
    (2u8..=0x47).filter(|k| *k != 0x2c).any(|k| {
        inv.owned.get(k as usize).copied().unwrap_or(false) && {
            let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, k as usize);
            inv.clip.get(slot).copied().unwrap_or(0) > 0
                || inv.carried.get(sim.weapons.ammo_type(k as usize)).copied().unwrap_or(0) > 0
        }
    })
}

/// Put a FIREARM with rounds in hand through the real switch input (never the 0x2c launcher —
/// see `firearm_armed`). Returns whether a firearm is in hand with rounds.
fn switch_to_firearm(w: &mut Walker, label: &str) -> bool {
    for _ in 0..120 {
        let want = {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if g.weapon != 0x2c && (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0 { return true; }
            let inv = &sim.drones.inventory;
            (2u8..=0x47).filter(|k| *k != 0x2c).find(|k| {
                inv.owned.get(*k as usize).copied().unwrap_or(false) && {
                    let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, *k as usize);
                    inv.clip.get(slot).copied().unwrap_or(0) > 0
                        || inv.carried.get(sim.weapons.ammo_type(*k as usize)).copied().unwrap_or(0) > 0
                }
            })
        };
        let Some(k) = want else { break };
        w.tick(Input { switch_to: Some(k), ..Default::default() }, false);
    }
    eprintln!("[08] {label}: no firearm with ammo to put in hand");
    let sim = w.scene.sim.as_ref().unwrap();
    let g = &sim.gun;
    g.weapon != 0x2c && (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0
}

/// The whole mission on one fresh load: the intro, the fusebox, the safe, the briefcase
/// (objective 1 → 4), the descent back down, the boarding, the ride, the cinematic, the Hind
/// shoot-down on real aim and trigger (objective 2 → 4), and the mission end into 0x0730008.
#[test]
fn walk_the_whole_mission_on_real_input() { walk_the_whole_mission_on_real_input_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_the_whole_mission_on_real_input_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000008.bin", false).expect("load 07000008")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000008.bin", false, None, difficulty)
            .expect("load 07000008")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // 00 Agent (difficulty 3): the fights keep the 05/07 sisters' 4.0 back-off — the boarding
    // platform's burst-rifle pair closes under 2 u during the exchange and its interval-4
    // bursts land point-blank (the measured d3 ledger: #20's fight alone cost 54.0 from
    // 1.2-2 u). A player steps back while firing. D3-GATED: the Operative/Agent sisters'
    // timelines stay byte-exact (the flag reads only inside the shared stands).
    if difficulty >= 3 {
        w.dmg.back_off = Some(4.0);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 139.5).abs() < 2.0 && (p[2] + 76.4).abs() < 2.0, "the station spawn #0 [D], got {p:?}");
        // The carried latches (in play they ride the RamSave [M 0x195670]); the cold stand-in
        // walk_07000008_live pins the same set.
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    w.wait(30);

    // ── the intro plate: ch 0x49 + the station-intro scene ──────────────────────────────────────
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x49), "the intro plate #237 fired ch 0x49 [D placements]");
    }
    w.wait_enabled(4500, "the station-intro scene");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0077) && p.state >= 3),
            "the station-intro scene (0x06000077) ran through the real trigger [M levels.md §5.1]");
        assert_eq!(sim.drones.mission.states[2][1], 4, "07000007's carried objective 2 completes on ch 0x49 [L record 2]");
        assert_eq!(obj08(&w).0, 3, "objective 1 announced by ch 0x49 [L record 3]");
    }

    // ── down the station and the walkway toward the fusebox ─────────────────────────────────────
    // The ch-0x2a plate sits inside the station (139.1, 50.2, −93.6 [D]) — the placed guards
    // (sw_ch 42) wake as Bond passes. The walkway descends south (door 11 free at (145,45,−108.6)).
    //
    // THE SCOPED-FLIGHT WALK HARDENING (level-4a-every-objective-4): under the game's cel-scoped
    // drone bullet flight [M Bullet_update 0x12e7b0:343 + Collide_Pick 0x1eb320:125] the station's
    // guards' rounds reach Bond on this approach (228 PlayerImpact hits by the lower-landing wedge
    // in the measured run), and each hit's `Player_HandlePain` grunt roll draws the shared
    // bit-exact RNG [M 0x1902c8:111 — the port's bullet.rs apply()] that the guards' AI also
    // draws — so the guards' positions and the doors' approach states wobble run-to-run, and the
    // walk's stuck-mitigation sidesteps land on different spots. The dip crossing at the stair
    // down to the fusebox walkway (the nav node (147.30, 42.41, −120.40) [D nav]) is the fragile
    // step: a sidestep at its edge drops Bond into the station's pit (measured: y 44.85 → 37.13,
    // the walk then fires the laser from the pit and the 4.0-range segment [L weapon_data
    // +0x4E·0x10C+0x1C] tops 1.4 below the wire box at y 43.41 — ch 137 never rises; the -3
    // report's mechanism, reproduced). The game's own player walks this approach live (the -3
    // live check stands) — the walk therefore RETRIES the approach like a player re-climbing:
    // the stance is checked before the shot (on the fusebox level, beside the box), and a wrong
    // stance re-walks the approach from the lower landing (bounded). [S] unverified as game
    // behaviour: the retry is the walk's own robustness, not a game claim.
    let fusebox = [154.35f32, 43.41, -119.67];
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        // No "station doors" stop at (139.5, 49.5, −97.5) on this walk: its nav route runs east
        // through door (160.6, 45.0, −100.1) and back west along the lower deck (y 46.1), and the
        // last leg aims at a point 3.4 above that deck — the leg wedged there, and the stuck
        // ladder's sidesteps at the deck edge dropped Bond into the station pit and out of the
        // map (y 46.1 → 40.8 → −508). The walkway head's own route walks the same deck.
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    // THE SCOPED-FLIGHT APPROACH FIX (level-4a-every-objective-4). The nav route into the fusebox
    // walkway steers at the dip node (147.30, 42.41, −120.40 [D nav]) from the lower-landing
    // wedge spot — and the wedge spot WOBBLES with the guards' fire: under the game's cel-scoped
    // drone bullet flight [M Bullet_update 0x12e7b0:343 + Collide_Pick 0x1eb320:125] the station
    // guards' rounds reach Bond here (228 PlayerImpact hits by the lower-landing wedge in the
    // measured run), and every hit's `Player_HandlePain` grunt roll draws the shared bit-exact
    // RNG [M 0x1902c8:111; the port's bullet.rs apply()] the guards' AI also draws — the guards'
    // positions and the doors' approach states wobble, the walk's stuck-mitigation sidesteps
    // land on different spots, and a sidestep at the stair edge drops Bond into the station's
    // pit (measured y 44.85 → 37.13; the 4.0-range laser [L weapon_data +0x4E·0x10C+0x1C] then
    // tops 1.4 below the wire box at y 43.41 and ch 137 never rises — the -3 report's mechanism,
    // reproduced). The green run's crossing is the proof the stair is walked head-on: its nav-1
    // arrival (146.15, 43.26, −119.85) sits on the walkway's west end after a straight south
    // descent. So the approach squares UP first — explicit waypoints down the stair's measured
    // line, each with an arrive radius the wobble cannot outrun (a leg that arrives within its
    // first stuck sample never enters the mitigation ladder). The last leg walks the walkway
    // east to the firing spot. [S] unverified as game behaviour: the waypoints are the walk's
    // own robustness, the descent line is the measured green crossing.
    for (name, at, r) in [
        ("the stair head", [146.6f32, 44.4, -117.6], 2.2),
        ("the walkway west", [146.3, 43.3, -119.8], 2.0),
        // The stair foot: the green crossing's own arrival (146.15, 43.26, −119.85). The 2.0
        // radius above can count Bond arrived still ON the stair (measured (146.33, 44.15,
        // −117.85)); the diagonal leg east from there pressed the stair's side, and the stuck
        // ladder walked him back up to the walkway door (146.3, 46.1, −109.4) for the cap.
        ("the stair foot", [146.15, 43.26, -119.85], 0.7),
        ("the walkway line", [149.5, 43.2, -120.3], 2.2),
        ("by the fusebox", [153.0, 43.4, -118.5], 1.5),
    ] {
        w.walk_to(Wp::new(at[0], at[1], at[2]).r(r), name);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08] at the fusebox tick {} bond {:?} ch42 {} ch43 {} ch44 {}", w.total_ticks, sim.bond.pos,
            sim.drones.channels.on(0x2a) as u8, sim.drones.channels.on(0x2b) as u8, sim.drones.channels.on(0x2c) as u8);
    }

    // ── cut the fusebox wires with the LASER → ch 137 (the Secure Room door's lock channel) ─────
    // The wire filter DROPS guns (kind 0x0008) and keeps the Laser (0x0002) [M fusebox.rs's
    // Collide_FilterBullets 0x1ece30:24-31 with the disc category words] — gunfire never cuts a
    // wire; the player's laser does.
    aim_and_fire_until(&mut w, fusebox, 0x4e, 420, |s| s.drones.channels.on(137));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(137), "shooting the fusebox wires raises ch 137 [D #219 group-0; M FuseBox_Update 0x1adfe8]");
    }

    // ── the secure room: the laser burns the safe bolts → ch 136 ────────────────────────────────
    // Door #10/122 (the GuardRoomDoor, lock ch 137 [D]) sits at (154.5, 42, −121.3) — E it ON its
    // position, then step through into the room (the safe wall's face is east).
    w.walk_to(Wp { at: [154.6, 42.5, -121.0], act: Act::Use { use_ticks: 240 }, arrive: 1.1 }, "E the secure room door");
    w.walk_nav_to([158.0, 43.0, -119.0], "into the secure room", 1.4);
    // The bolts sit at (162.24, 43.88/42.70, −117.14) [D #207/#208] — inside, on the safe face.
    w.walk_nav_to([160.0, 43.4, -117.0], "before the safe", 1.2);
    let bolts = [[162.24f32, 43.88, -117.14], [162.24, 42.70, -117.14]];
    for (k, bolt) in bolts.iter().enumerate() {
        let ch = [134u32, 135][k];
        aim_and_fire_until(&mut w, *bolt, 0x4e, 240, |s| s.drones.channels.on(ch));
        eprintln!("[08] bolt {k} laser_charge {:.1}", w.scene.sim.as_ref().unwrap().laser_charge);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(ch), "the laser burn wrote the bolt's out channel {ch} [M Lock_Update 0x1ae3d8:194]");
    }
    // The mux folds 134|135 → 136; the safe door opens and SP #286 (SafePickUps) starts.
    w.wait(10); // the fire stops on the burn-through tick; the mux and the SP start on the following ticks
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(136), "mux #278 folded the bolt channels into ch 136 [D]");
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_09e2) && p.state >= 2), // 2 = playing, 3 = done (nis.rs SP states)
            "SP #286 SafePickUps started on ch 136 [D mask 1]");
    }

    // ── the briefcase: the take writes ch 0x4A (objective 1) ────────────────────────────────────
    w.walk_nav_to([172.7, 43.5, -116.2], "to the briefcase", 1.0);
    let mut got_case = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) { got_case = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(got_case, "taking the briefcase writes ch 0x4a [M Pickup_Handler 0x18e650:330-333]");
    for _ in 0..120 {
        if obj08(&w).0 == 4 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(obj08(&w).0, 4, "objective 'Retrieve guidance chip' completes on ch 0x4a [L record 3]");

    eprintln!("[08] the briefcase stretch done in {} ticks; falls {:?}", w.total_ticks, w.report.falls);

    // ── back out of the secure room: the same waypoints in reverse, real walking ────────────
    for (name, at) in [
        ("before the safe", [160.0f32, 43.4, -117.0]),
        ("the secure room door", [158.0, 43.0, -119.0]),
        ("at the door west side", [154.6, 42.5, -121.0]),
        ("by the fusebox", [153.0, 43.4, -118.5]),
        ("the lower landing", [150.0, 43.5, -116.0]),
    ] {
        if name == "at the door west side" {
            // The way out is the door Bond came in by (#10/122 at (154.5, 42, −121.3)): walk INTO
            // its gap and press E, as the way in did. The old 1.5 nav arrival counted (155.19,
            // −119.93) — still on the room side of the wall line x 155.19, beside the kind-2 door
            // at (154.3, −119.6) — as arrived; the straight leg west to the fusebox then pressed
            // that wall and slid into the corner at (155.19, −121.78), wedged there for every
            // later leg while two guards stood in GoToGoalPosition 0x63 just outside at (154.0,
            // 43.0, −120.1) (measured on the seven-bone sight build; the arrival spot depends on
            // the run's earlier fights).
            w.walk_to(Wp { at, act: Act::Use { use_ticks: 240 }, arrive: 0.5 }, name);
            continue;
        }
        w.walk_nav_to(at, name, 1.5);
    }

    // ── the gondola: the descent to the floor plate ─────────────────────────────────────────
    // The outbound descent follows path 2's own link chain ([D the path-2 dump]): the landing
    // east (147.3/149.3/151.2/153, y 42.4, z −120.5) → the north ramp (153.1, 41.15, −115.95) →
    // (153.16, 39.65, −111.63) → (148.29, 38.4, −110.84) → the mid level (143.32, 36.4, −111.43)
    // → west past the mid doors (135.1, −111/−113.5) → (132.1, −112.27) → the long link to
    // (120.5, 34.04, −112.51) → the y-33.9 hall.
    for (name, at) in [
        ("the landing east", [151.2f32, 42.5, -120.5]),
        ("through the door", [153.0f32, 42.5, -120.7]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("the ramp south", [153.1, 41.2, -116.0]),
        ("the ramp mid", [153.2, 39.7, -111.7]),
        ("the ramp turn", [148.3, 38.5, -110.9]),
        ("the mid level", [143.3, 36.5, -111.5]),
        ("the mid doors", [135.5, 36.5, -112.2]),
        ("the mid west", [132.1, 36.2, -112.3]),
        ("the long link", [120.5, 34.1, -112.5]),
        ("the hall y33", [118.2, 33.9, -111.0]),
        ("the hall y33 west", [118.2, 33.9, -107.9]),
        ("the hall y33 south", [113.2, 33.7, -106.4]),
        ("the stairwell top", [108.2, 31.5, -106.3]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        // 00 Agent (difficulty 3): the rifle pair (#15/#16, weapon 0x0, at (122.2/122.6,
        // 34.6/35.3, -112.8/-114.7), ~10 u east of the mid west stop [L the d3 ledger]) owns the
        // open long link from its east end — on the d3 run both answered fights ("the long link
        // nav 2" #16, "the hall y33 nav 1" #15) cost 40.0 of pool while Bond stood on the open
        // link. A player fights them from the mid-doors doorway (cover at the link's west end)
        // BEFORE stepping out: the pair is 10.2 u east, dy −1..−2 — inside the close window.
        // D3-GATED: the Operative/Agent sisters' timelines stay byte-exact.
        if difficulty >= 3 && name == "the mid west" {
            fight_close_guards(&mut w, "the mid west", 4200);
        }
        // 00 Agent (difficulty 3): #16/#17 are ambush spawners (their plates fire mid-leg) —
        // the walk-out stand catches whatever the link walk enabled before the hall legs.
        // D3-GATED.
        if difficulty >= 3 && name == "the long link" {
            fight_close_guards(&mut w, "the long link", 4200);
            // 00 Agent (difficulty 3): the mid-west and long-link stands left #15/#16/#17's guns
            // ON the walked path (#15 fell 1.6 u off this waypoint, (121.5, 34.5, −113.5) [L the
            // 2026-11-03 25773-death run]) — a player scoops the free rifle before the stairwell:
            // these fights drain the pistol pool dry (the death run's platform-mouth stand opened
            // on weapon 0xc clip 0 spare 0) and the post-climb reload_full has no spare to fill
            // from. REAL INPUT: the walk-over take [L the level-11 kill-drop pass]. D3-GATED: the
            // Operative/Agent sisters' timelines stay byte-exact.
            w.resupply("the long link");
        }
    }
    // Sister: the descent was WALKED — the walkway's zone plates (TouchOnces out 42/43/44, the
    // ambush spawners' start channels [D placements; M Trigger_Hit 0x1b1fc8 walk-in mask 8])
    // fired on the way down; a teleport to the landing would leave them off.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x2a) || sim.drones.channels.on(0x2b) || sim.drones.channels.on(0x2c),
            "the walked descent crossed the walkway ambush plates (42/43/44) [D]");
    }
    w.climb_stairs([107.25, 27.7, -119.5], "the stairwell down", 1.6, 4200);
    // 00 Agent (difficulty 3): the descent's return-fire stands drained the clip — the 05
    // sisters' rule: the fights must not open on a drained clip (a mid-fight reload is free
    // burst time for the platform pair). D3-GATED.
    if difficulty >= 3 {
        w.reload_full();
    }
    // The boarding ([D the floor probe]): the platform (y 27.25, z −121..−122.5) ends at the car
    // shaft — the middle (x 112) is open below (the cabin hangs there; its floor is the plate at
    // y 28, z −127.4), the side ledges (x 110/114) run south at y 26.75 past the parked cabin's
    // mouth. Walk the west ledge south; the plate's volume (12 tris = its box) touches Bond's body.
    for (name, at) in [
        ("the lower level east", [110.7f32, 27.7, -121.5]),
        ("the gondola mouth", [112.0, 27.4, -122.0]),
        ("the platform west", [110.3, 27.3, -122.2]),
        ("the west ledge south", [110.0, 27.0, -124.5]),
        ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
    ] {
        w.walk_nav_to(at, name, 1.4);
        // 00 Agent (difficulty 3): the burst-rifle pair (#19/#20, weapon 0x15, skin 0x500002f,
        // at (101.9-104.5, 28.2, -122.3/-122.6), 7-10 u west [L the d3 ledgers]) fires into
        // the boarding legs from the platform's west — the d3 runs died in the legs (ticks
        // 25164/25382/25426) before the `before the boarding` stand below ran. A player fights
        // the pair the moment the platform mouth opens (the "platform west" stand of the same
        // runs killed #19 from there) instead of walking the burst lane first. D3-GATED: the
        // Operative/Agent sisters' timelines stay byte-exact.
        if difficulty >= 3 && name == "the lower level east" {
            // 00 Agent (difficulty 3): the descent's fights can leave the FIREARMS dry while the
            // AT-420 sits owned with clip 4 — the 25773 death's stand cycled the dry pistol onto
            // the LAUNCHER (see firearm_armed) and stood into #20's interval-4 bursts (13 × 6.0,
            // health 78 → 0, ticks 25375–25773 [L the run log]). A player does not fight a 9.8 u
            // burst lane rocket-handed: grab the freshest drops when no firearm has ammo (#23 and
            // #19 fell ON this floor seconds before [L the run log]), put a firearm in hand,
            // THEN take the stand — kill #20 first, from the stand's own back-off cover, and only
            // then approach the car. D3-GATED: the Operative/Agent sisters' timelines stay
            // byte-exact.
            if !firearm_armed(&w.scene.sim.as_ref().unwrap()) { w.resupply("the platform mouth"); }
            switch_to_firearm(&mut w, "the platform mouth");
            fight_close_guards(&mut w, "the platform mouth", 4200);
        }
    }
    // 00 Agent (difficulty 3): the same firearm hygiene before the last pre-boarding stand —
    // the legs' return-fire stands may have spent the clip on the walk over. D3-GATED.
    if difficulty >= 3 {
        if !firearm_armed(&w.scene.sim.as_ref().unwrap()) { w.resupply("before the boarding"); }
        switch_to_firearm(&mut w, "before the boarding");
    }
    // THE CLOSE-GUARD FIGHT (level-08-real-health-rockets): the descent wave's burst-rifle
    // pair (#263/#264, weapon 0x15, skin 0x0500002f [D the audit's param dump]) closes to
    // 4.0–6.5 u on this platform and its interval-4 bursts land (the measured death run:
    // −1.80/−3.60/−3.60 at frames 24647/24663/24667, drone #19/#20 —
    // docs/reports/level-08-long-range-hits.md). A player switches to the gun for guards this
    // close before stepping into the car. REAL-DAMAGE ONLY (the fn gates on `tops_up()`).
    fight_close_guards(&mut w, "before the boarding", 4200);
    {
        // East into the open west doorway, polling for the plate.
        let mut last = [0.0f32, 0.0];
        let mut still = 0u32;
        for t in 0..1200u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(4) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let target = [112.0f32, 28.0, -127.0];
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[08 board t{t}] bond {:?} ch4 {}", sim.bond.pos, sim.drones.channels.on(4) as u8);
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08 gondola] bond {:?} ch4 {} falls {:?} stuck {}", sim.bond.pos,
            sim.drones.channels.on(4) as u8, w.report.falls, w.report.stuck_events.len());
        assert!(sim.drones.channels.on(4), "the cable-car floor plate fired ch 4 [D #236 mask 8]");
    }
    // mux #235 folds the REAL ch 0x4A (the briefcase take above) AND 4 → ch 1 → SP #295 (the
    // ride, 1420 ticks) starts. Watch the ride: the carriage carries Bond [M
    // Control_InheritVelocity 0x133e88] and writes ch 83 at tick 1410.
    let mut ride_started = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(1) { ride_started = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(ride_started, "mux #235 (0x4a AND 4) fired ch 1 [D]");
    let start_y = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let rockets_at = |w: &Walker| { let inv = &w.scene.sim.as_ref().unwrap().drones.inventory;
        (inv.owned.get(0x2c).copied().unwrap_or(false), inv.clip.get(0x2c).copied().unwrap_or(0), inv.carried.get(0xe).copied().unwrap_or(0)) };
    eprintln!("[08] rockets boarding (owned, clip, carry): {:?}", rockets_at(&w));
    let mut ch83 = false;
    for t in 0..3600u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { ch83 = true; break; }
        w.tick(Input::default(), false);
        if t % 300 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 ride t{t}] bond {:?} ch1 {} ch83 {} disabled {}", sim.bond.pos,
                sim.drones.channels.on(1) as u8, sim.drones.channels.on(0x53) as u8, sim.disabled);
        }
    }
    assert!(ch83, "the ride script wrote ch 83 (0x53) at its tick 1410 [D stream 46]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08] ride done: bond {:?} (started y {start_y:.1}); rockets (owned, clip, carry) {:?}", sim.bond.pos, rockets_at(&w));
        assert!(!sim.drones.channels.on(0x7c), "Bond rode the carriage — the plunge (out 0x7C) never fired [D §6]");
    }
    w.wait_enabled(1500, "the bottom-of-mountain cinematic");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0078) && p.state >= 3),
            "SP #296 (0x06000078) ran on ch 83 [D]");
        assert!(sim.drones.channels.on(5), "the cinematic wrote ch 5 [D stream 10]");
    }
    // The helicopter: gate ch 5 opens the route flight (state 1) [M Copter_Update 0x17dbb8 head].
    // The faithful machine writes the out channel (rec+0x72 = param 7 = ch 0x4B [D #45 p7; M
    // 0x17d4ac]) ONLY from the death tail — hp < damage → state 4, disasm 0x17e390-0x17e3c8 — or
    // cases 4/5/6, which only the death (or a kind-2 arrival path; none exists on 08 [D the route
    // dump]) reaches. The walk shoots the Hind down with REAL aim and trigger pulls: Bond stands
    // where the cinematic left him, the aim tracks the machine's live position every tick (the
    // real look input), the damage accumulates from the weapon contacts on the part spheres
    // [M 0x17dbb8:277-353] — no position or damage pokes.
    let mut flying = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.copters.iter().any(|c| c.out == 0x4b && (1..=2).contains(&c.state)) { flying = true; break; }
        w.tick(Input::default(), false);
        // hind-flight-pin: the arrival log — every 30 ticks from the gate opening [M Copter_Update
        // 0x17dbb8 head], the copter's live flight record.
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.frame % 30 == 0 {
            for c in sim.drones.copters.iter().filter(|c| c.out == 0x4b) {
                eprintln!("[08 flight] f{} pos ({:.2},{:.2},{:.2}) state {} path {:?} t {:.2} dmg {:.1}",
                    sim.drones.frame, c.pos()[0], c.pos()[1], c.pos()[2], c.state, c.path, c.t, c.damage);
            }
        }
    }
    assert!(flying, "the gate ch 5 opened the route flight (state 1/2) [M 0x17dbb8 head]");
    // 00 Agent (difficulty 3): SHOOT OUT the carriage's north and south panes before the fight.
    // The cabin box stops the eye rays 0.4-3.6 u out in every direction [L the run sweeps] and
    // there is NO floor outside to walk to (the probe: floor_below is None at every candidate
    // 2-8 u out — the terminal hangs over the valley, the kill floor y −24 [D #42/#43/#44]) —
    // the game's player fights from INSIDE, through the panes he has shot out. The measured
    // proof: the fix1 run's first rocket (t191) broke the north pane and the north ray then read
    // CLEAR 30 u for the rest of the fight (the t600-t2400 sweeps az0.00:1.00) while the intact
    // south stayed blocked (az3.14:0.08) [L the run logs]. With a pane broken the launch
    // corridor is open air — a round cannot detonate on the carriage's own wall band next to
    // Bond (the two measured d3 deaths: explosions at d 5.8 and d 2.2 [L NF_DMG_TRACE]). Real
    // aim and trigger, the in-hand firearm; the hold stops when the ray flies 30 u. D3-GATED:
    // the Operative/Agent sisters' timelines stay byte-exact (their fights run through the
    // doorway slots the RNG gives them and stay green).
    if difficulty >= 3 {
        // The cabin's glass: the live panes' boxes name the shoot-out targets (the gondola's
        // windows are script panes, kind 2, health 1.0 — one bullet contact kills [M
        // breakable.rs bullet_hit; the pane test]). The ends' panes (|dz| > 1.5) are the fight's
        // launch corridors: the machine circuits north and south of the terminal, and a broken
        // pane leaves 30 u of open air on its bearing [L the fix1 run — the first rocket broke
        // the north pane and the north ray read CLEAR 30 u for the rest of the fight].
        let bond_at = w.scene.sim.as_ref().unwrap().bond.pos;
        let panes: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter()
            .filter(|b| b.kind == 2 && !b.dead)
            .map(|b| [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0])
            .filter(|c| (c[0] - bond_at[0]).abs() < 3.0 && (c[1] - bond_at[1]).abs() < 2.5 && (c[2] - bond_at[2]).abs() > 1.5)
            .collect();
        eprintln!("[08 panes] live end panes near the cabin: {panes:?}");
        switch_to_firearm(&mut w, "the panes");
        // A player shoots the glass he can see — the aim-ray-to-bounds-centre gate CANNOT work
        // here: the captured centre is the pane bounds' midpoint, which sits a hair short of the
        // sheet plane (the north pane's centre ray eye→centre reads OPEN while a perpendicular
        // ray through the sheet hits at t 0.52 [L the probe raytest 2026-11-04]), so every ray
        // reads OPEN before a single round and the old cleared-gated tap fired nothing (the
        // orphaned-pane symptom of the hind session: 9 rounds were the earlier fights, dead rows
        // []). The pane damage law needs no new sim code — the port's contact already drains it:
        // the game's collision pass registers the round's contact on the pane's obj+0xD0 chain
        // whether or not the round stops (the mirrored contact) and `Break_Update` 0x12bfd8:36-37
        // drains health from that chain; the port's seg_box_t contact over the pane bounds does
        // the same (drone.rs's player-bullet pass). The kill drops the row and the script object
        // (Break_Kill's obj+0xFE delete → retain_script_panes → the dynamic quads leave the set),
        // which is what "breaks and clears" means for the launch corridors. So: aim at the pane
        // centre with the real look input, pull the trigger on a CONVERGED aim (the fire edge
        // every 12 ticks — a held trigger is one pull [L the pane hold]), until the pane's row
        // dies (the kill drops it from breaks.list).
        let pane_gone = |sim: &nf_view::scene::Sim, c: [f32; 3]| {
            sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead).all(|b| {
                let m = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
                (m[0] - c[0]).powi(2) + (m[1] - c[1]).powi(2) + (m[2] - c[2]).powi(2) > 0.7 * 0.7
            })
        };
        for (k, c) in panes.iter().enumerate() {
            let mut fired = 0u32;
            for t in 0..260u32 {
                if pane_gone(&w.scene.sim.as_ref().unwrap(), *c) { break; }
                let sim = w.scene.sim.as_ref().unwrap();
                let head = sim.bond.head();
                let to = [c[0] - head[0], c[1] - head[1], c[2] - head[2]];
                let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(0.01);
                let desired_yaw = to[0].atan2(to[2]);
                let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
                let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
                let pitch_err = desired_pitch - sim.bond.pitch;
                let input = Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.05, 0.05), ..Default::default() };
                // Fire on the edge, only on a converged aim — the pane's bounds box is thin
                // (0.9-1.2 u across, ~0.1 thick [L the rows' bounds]) and an off-aim round
                // misses it (the health 1.0 dies to one contact [D params [2,1,0]]).
                let fire = t % 12 == 0 && yaw_err.abs() < 0.01 && pitch_err.abs() < 0.02 && fired < 5;
                if fire { fired += 1; w.tick_fire(input); } else { w.tick(input, false); }
            }
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 panes] pane {k} at {c:?}: {} after {fired} rounds, clip {} spare {}",
                if pane_gone(sim, *c) { "SHOT OUT" } else { "STILL GLAZED" }, sim.gun.clip, sim.gun.spare);
        }
        {
            // The proof the shoot-out happened: every end pane's row is gone (killed rows drop),
            // and the two end bearings read open air 30 u out — a shot-out window breaks AND
            // clears for the player [L the fix1 run — the broken pane's bearing read CLEAR 30 u].
            let sim = w.scene.sim.as_ref().unwrap();
            let alive: Vec<[f32; 3]> = sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead)
                .map(|b| [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0]).collect();
            let unbroken = panes.iter().filter(|c| !pane_gone(sim, **c)).count();
            eprintln!("[08 panes] after the shoot-out: {unbroken} end panes still glazed, {} live kind-2 rows total", alive.len());
            let b = sim.bond.pos;
            let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
            for (name, c) in [("north", panes.iter().find(|c| c[2] < b[2] - 1.5)), ("south", panes.iter().find(|c| c[2] > b[2] + 1.5))] {
                let Some(c) = c else { continue };
                let dir = [(c[0] - eye[0]) / 3.0, (c[1] - eye[1]) / 3.0, (c[2] - eye[2]) / 3.0];
                let far = [eye[0] + dir[0] * 30.0, eye[1] + dir[1] * 30.0, eye[2] + dir[2] * 30.0];
                let t = sim.world.segment_bullet(eye, far);
                eprintln!("[08 panes] the {name} bearing 30 u out: {}", if t.is_none() || t.is_some_and(|t| t >= 0.999) { "CLEAR" } else { "blocked" });
            }
            assert_eq!(unbroken, 0, "the shoot-out must kill every end pane (the rows' mirrored contact drains [M Break_Update 0x12bfd8:36-37]) — {unbroken} still glazed");
        }
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b), "no arrival while the route flies");
    let ci = w.scene.sim.as_ref().unwrap().drones.copters.iter().position(|c| c.out == 0x4b).unwrap();
    let hp = w.scene.sim.as_ref().unwrap().drones.copters[ci].hp;
    // The game's way down: the AT-420 Sentinel. The level hands it out twice — the safe's launcher
    // #291 (kind 0, id 0x2C) beside the briefcase, its rounds forced to 4 on this level only
    // [M Pickup_Create 0x18dda0:116-118 `GameState+0xc == 0x7000008 && id == 0x2c`], and the cable
    // car's three pickups (a launcher and two rocket-ammo boxes, 4 each, parented to the carriage
    // [D 0x0600004C streams 37-39]) that Bond collects standing in the car on the ride. The copter's
    // hp is DiffMod·299 [M Copter_Create 0x17d340: 1.0 below difficulty 2]; the body (slot 2) is
    // the only part a pick reaches [M Copter_Register 0x17e530:7; Collide_PickObj 0x1eae80's
    // always-on 0x20 gate], so the P2K needs 86 rounds (3.5 [L weapon record 2]) — far past the
    // 48 the carry starts 07 with. A rocket on the body blasts at the contact [M
    // Bullet_CollisionHandler 0x12ce80's 0x4 arm] and `Explode_Propagate` 0x17c518 case 0 adds
    // ≈100 (radius 8, damage 100 [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C]) plus the round's
    // 1.0 contact: three rockets (303 > 299), one spare. The walk fires the 'Guided' mode the
    // launcher is picked up in and flies each rocket into the body with the real look input
    // (`Bullet_update` 0x12e7b0:177-244's steering) — no ammo or damage pokes.
    const ROCKET: u8 = 0x2c;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        let owned: Vec<u8> = (0x02u8..0x4e).filter(|&wid| inv.owned.get(wid as usize).copied().unwrap_or(false)).collect();
        eprintln!("[08] owned weapons at the fight: {owned:02x?} (current {:#x}); rocket clip {} carried {}", sim.gun.weapon,
            inv.clip.get(ROCKET as usize).copied().unwrap_or(0), inv.carried.get(ROCKET as usize).copied().unwrap_or(0));
        assert!(inv.owned.get(ROCKET as usize).copied().unwrap_or(false), "the walk took the safe's rocket launcher #291");
        let at = sim.weapons.ammo_type(ROCKET as usize);
        eprintln!("[08] rocket ammo type {at:#x}: carried {} gun.spare {}; rocket pickups {:?}", inv.carried.get(at).copied().unwrap_or(0), sim.gun.spare,
            sim.drones.pickups.iter().filter(|p| p.weapon == ROCKET as u16).map(|p| (p.kind, p.rounds, p.gone, p.from, p.m[3])).collect::<Vec<_>>());
    }
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == ROCKET { break; }
        w.tick(Input { switch_to: Some(ROCKET), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, ROCKET, "the switch to the rocket launcher never landed");
    let mut rockets = 0u32;
    let mut ch4b = false;
    let mut riding = false;
    let mut launch_damage = 0.0f32;
    let mut best_rocket = 0.0f32;
    // hind-flight-pin: the every-30-ticks flight record (pos, state, path, t) [M Copter_Move
    // 0x17d628 / Copter_Update 0x17dbb8].
    let mut flight_samples: Vec<([f32; 3], u16, Option<usize>, f32)> = Vec::new();
    // The reposition state: Bond arrives INSIDE the bottom terminal (the ride's end (112, 13.5,
    // −179.7) sits in a walled box — the eye ray stops at x ≈ 110.4/113.6 and z ≈ −176.6/−182.2
    // in every direction, eye height [D the measured stops]). The machine's circuit is outside,
    // and the game's player walks out of the terminal to fight it — the death run's rocket 2
    // reached the machine only because the GUIDED round's per-tick steering threaded the
    // opening [M Bullet_update 0x12e7b0:177-244] while rockets 1/3/4 exploded on the walls
    // 0.65–5.5 u from Bond. So when the corridor to the machine is blocked, Bond LOOKS for the
    // opening (a 16-ray azimuth sweep at eye height — the farthest clear ray, preferring the
    // machine's bearing) and WALKS out (real forward input, grounded only), then fights from
    // the open pad.
    let mut exit_ticks = 0u32;
    for t in 0..20000u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b) { ch4b = true; break; }
        let sim = w.scene.sim.as_ref().unwrap();
        // hind-flight-pin: the fight log — every 30 ticks to the death [M the death tail writes
        // ch 0x4B at 0x17e3bc-0x17e3c8].
        if sim.drones.frame % 30 == 0 {
            let c = &sim.drones.copters[ci];
            flight_samples.push((c.pos(), c.state, c.path, c.t));
            eprintln!("[08 flight] f{} pos ({:.2},{:.2},{:.2}) state {} path {:?} t {:.2} dmg {:.1}/{:.0} bullets {}",
                sim.drones.frame, c.pos()[0], c.pos()[1], c.pos()[2], c.state, c.path, c.t, c.damage, hp,
                sim.drones.bullets.iter().filter(|b| b.state != 2).count());
        }
        let cp = sim.drones.copters[ci].pos();
        let rocket = sim.drones.player_bullets.iter().find(|b| b.guided_cam && !b.dead).map(|b| (b.pos, b.dir));
        let (from, yaw, pitch) = match rocket {
            // Riding the rocket: the look input steers it [M 0x12e7b0:177-244; scene.rs's ride].
            Some((p, d)) => (p, d[0].atan2(d[2]), d[1].clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2),
            None => (sim.bond.head(), sim.bond.yaw, sim.bond.pitch),
        };
        let to = [cp[0] - from[0], cp[1] - from[1], cp[2] - from[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        // 00 Agent: the shot leaves through the CROSSHAIR, not the view centre — the aim law is
        // look + 0.76966·cx·right + 0.57724·cy·up, normalized [M the crosshair pass: the shot's
        // direction through the crosshair]. The walk's own tracker saturates the free-aim cursor
        // at its clamp while it converges (the fire dump at every launch: cross (−0.296, ±0.394),
        // the cursor pinned), which put every measured launch up to 0.23 rad off the tracker's
        // converged view — rocket 4's round crossed the south-east corner post 3 u out and the
        // blast took −62.5 at 2.2 u [L the diag run 2026-11-04 — the world hit at frame 28470].
        // The player aims the CROSSHAIR: the tracker's error (and the slit/corridor lanes below)
        // read the crosshair's own bearing, so a clamped cursor recenters before the shot. The
        // ride keeps the rocket-dir steering (the cursor is frozen mid-flight) [M the missile-cam
        // block zeroes the input the cursor arm would see]. D3-GATED: the sisters' timelines stay
        // byte-exact.
        let (aim_ref_yaw, aim_ref_pitch, aim_dir) = if difficulty >= 3 && rocket.is_none() {
            let (syy, cyy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            // `bond.pitch` is BL+0x8A8 in units of π/2, positive up (the walk's own aim code).
            let p = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let right = [cyy, 0.0, -syy];
            let up = [-p.sin() * syy, p.cos(), -p.sin() * cyy];
            let tx = 0.76966 * sim.cross[0];
            let ty = 0.57724 * sim.cross[1];
            let dd = [syy * p.cos() + tx * right[0] + ty * up[0],
                p.sin() + tx * right[1] + ty * up[1],
                cyy * p.cos() + tx * right[2] + ty * up[2]];
            let ll = (dd[0] * dd[0] + dd[1] * dd[1] + dd[2] * dd[2]).sqrt().max(1e-6);
            let ad = [dd[0] / ll, dd[1] / ll, dd[2] / ll];
            (ad[0].atan2(ad[2]), ad[1].clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2, ad)
        } else {
            let (sy, syp) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let pa = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let (sp, spp) = (pa.sin(), pa.cos());
            (yaw, pitch, [sy * spp, sp, syp * spp])
        };
        let yaw_err = walker::wrap_pi(desired_yaw - aim_ref_yaw);
        let pitch_err = desired_pitch - aim_ref_pitch;
        // REAL-HEALTH TACTIC (level-08-real-health-rockets) — pick weapon and range the way a
        // player would:
        // 1. NO ROCKET AT A TARGET INSIDE THE BLAST RADIUS: the AT-420's blast is radius 8,
        //    ≈100 [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C; M Explode_Propagate 0x17c518 case
        //    0] with the (int)-truncated linear falloff × the 0.6 level factor [M :69-71;
        //    Player_HandlePain's table] — the fire gate holds at 10 u, and Bond steps back
        //    (aim held) while the machine is inside that reach (its circuit closes; the flight
        //    time can close 2-3 u more).
        // 2. NO ROCKET DOWN A BLOCKED LINE: the round flies the straight corridor its muzzle
        //    opens [M Bullet_update 0x12e7b0], and the death run's four "world" hits
        //    −52.5/−7.5/−22.5/−3.7 (frames 26477/26580/26639/26696) are Bond's OWN rockets
        //    1/3/4 exploding on the bottom-station geometry 0.65–5.5 u from him — only rocket
        //    2 reached the machine (docs/reports/level-08-long-range-hits.md). The launch
        //    needs the clear corridor (`launch_line_clear`, eye + both gun stand-ins).
        // The slit gate: what kills Bond is a round whose FIRST flight segments run into the
        // cabin walls (the death run's 0.65–5.5 u detonations). The player-model is not a
        // straight sniper line — the round is GUIDED and steers toward the machine's live
        // position every tick [M Bullet_update 0x12e7b0:177-244] — it is "fire through the
        // doorway, then steer": the muzzle ray's first 3 u must leave the cabin clear, and the
        // machine must be beyond the blast's 10 u reach. A round that dies mid-flight after
        // that detonates on the far terrain (k > 8 of the falloff — no self-damage).
        // (`aim_dir` above: the crosshair's own bearing on 00 Agent, the view's on the sisters.)
        let b = sim.bond.pos;
        let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
        // The slit gate reads the FLIGHT query on 00 Agent (the round's own world trace — the
        // glass pass [M Collide_Update 0x1ea398:216-229; Bullet_update disasm 0x12f048-0x12f058])
        // so an aim through the live panes reads as the game's flight reads it: the mat-0x10
        // pane quads pass, the mat-0x06 frame walls stop. The Operative/Agent sisters keep the
        // plain segment_bullet — their timelines stay byte-exact.
        let slit_clear = if difficulty >= 3 {
            sim.world.segment_flight(eye, [eye[0] + aim_dir[0] * 3.0, eye[1] + aim_dir[1] * 3.0, eye[2] + aim_dir[2] * 3.0], true).is_none()
        } else {
            sim.world.segment_bullet(eye, [eye[0] + aim_dir[0] * 3.0, eye[1] + aim_dir[1] * 3.0, eye[2] + aim_dir[2] * 3.0]).is_none()
        };
        // 00 Agent (difficulty 3): the launch CORRIDOR — the first 12 u of the round's flight
        // must be clear air from the eye AND both gun stand-ins (the round spawns at the muzzle
        // [M the fire block]; the AT-420's blast is radius 8, ≈100 [L ee_start_05.bin
        // + 0x2C·0x10C]). The measured d3 deaths are rounds that cleared the 3 u slit but chased
        // the machine behind the terminal and died on its own wall band next to Bond — explosion
        // at (112.4, 15.0, −174.3) d 5.8 → −37.5 (frame 27680) and at (110.5, 14.7, −181.4)
        // d 2.2 → −40.5 (frame 30000) [L the NF_DMG_TRACE run — the wall stops the eye/gunR/gunL
        // rays at t0.17 ≈ 5 u on the first, ~2.4 u on the second]. A player fires only on a
        // corridor that cannot detonate on him. D3-GATED: the Operative/Agent sisters' timelines
        // stay byte-exact.
        let corridor_clear = difficulty < 3 || {
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            // The flight query with the glass pass: the round's own world trace — the live
            // panes' mat-0x10 quads pass, the mat-0x06 frames/walls stop [M 0x1ea398:216-229].
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight(from, [from[0] + aim_dir[0] * 12.0, from[1] + aim_dir[1] * 12.0, from[2] + aim_dir[2] * 12.0], true)
                    .is_none() })
        };
        let line_ok = slit_clear && launch_line_clear(sim, cp);
        // The doorway fight: Bond rode down INSIDE the carriage (the cabin floor is the origin's
        // −4.5 [D stream 1; the ride logs]) and the game's fight is fought from in here — the
        // cabin's west doorway (the opening he boarded through at the top; the wall band's
        // z gap −178.25..−180.6 in the measured tri dump) is the firing slit, its top edge
        // (the lintel at y ≈ 14.31 [D the tri dump]) just above the aiming eye (14.24). The
        // machine's circuit dips below the lintel's sight line (y ≈ 14.6 at 40 u) on its low
        // passes — THOSE are the fire windows. The launch gates: the corridor must be clear at
        // the eye AND both gun stand-ins (the round spawns at the muzzle [M the fire block] and
        // the death run's rockets exploded at 0.65–5.5 u on the walls a blocked launch
        // produces); the aim tolerance is loose (±0.10 rad) because the round is GUIDED — the
        // ride steers it toward the machine's live position every tick [M Bullet_update
        // 0x12e7b0:177-244] — and the launch only has to START through the slit.
        let mut input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            forward: if rocket.is_none() && l < 10.0 { -1.0 } else { 0.0 },
            aiming: rocket.is_none(),
            ..Default::default()
        };
        let _ = exit_ticks;
        if rocket.is_some() != riding {
            riding = rocket.is_some();
            let now = sim.drones.copters[ci].damage;
            if riding { rockets += 1; launch_damage = now; } else { best_rocket = best_rocket.max(now - launch_damage); }
            eprintln!("[08 rocket] t{t} {} — damage {:.1} hp {hp} dist {l:.1} from {from:?} copter {cp:?} clip {} bullets {}",
                if riding { "launched" } else { "gone" }, sim.drones.copters[ci].damage,
                sim.drones.inventory.clip.get(ROCKET as usize).copied().unwrap_or(0),
                sim.drones.player_bullets.iter().filter(|b| !b.dead).count());
        }
        // Fire when the corridor to the machine is open and it is beyond the blast's reach —
        // the aim tolerance is the guided round's, not a settled sniper's (see above): the
        // machine's low passes through the slit are a few ticks wide and the tracker's
        // ±0.08/±0.05 per-tick clamps cannot settle inside a sniper gate there.
        // 00 Agent: the player fires at what he can SEE. The eye's flight ray to the machine's
        // live position must reach (essentially) the machine — the stop, if any, is the cable
        // sag the guided round threads (the measured launches' stops t0.85-0.95 [L the diag
        // run 2026-11-04]) — NOT the cabin's own wall band: the t347 launch's eye ray stopped
        // at t0.048 (the machine hidden behind the band between the windows), the chase then
        // steered the round onto the window frame and the blast took −75 at 2.4 u [L the diag
        // run — the world hit at frame 28066]. And the CHASE's lane — the machine-bearing
        // corridor from the eye and both gun stand-ins — must reach the machine the same way
        // (the t747 launch's round clipped the south-east corner post 2.2 u from Bond: −62.5
        // [L the diag run — frame 28470]). D3-GATED: the sisters' timelines stay byte-exact.
        let eye_sees = difficulty < 3 || sim.world
            .segment_flight_scoped(sim.world.find_cel(eye), eye, cp, true)
            .is_none_or(|t| t >= 0.8);
        let machine_lane_clear = difficulty < 3 || {
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight_scoped(sim.world.find_cel(from), from, cp, true)
                    .is_none_or(|t| t >= 0.8) })
        };
        let ready = rocket.is_none() && sim.missile_cam.is_none()
            && yaw_err.abs() < if difficulty >= 3 { 0.10 } else { 0.25 }
            && pitch_err.abs() < if difficulty >= 3 { 0.10 } else { 0.25 }
            && l >= if difficulty >= 3 { 16.0 } else { 10.0 } && slit_clear && corridor_clear && eye_sees && machine_lane_clear;
        if ready { w.tick_fire(input); } else { w.tick(input, false); }
        if t % 100 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 hind t{t} f{}] dist {l:.1} line_ok {line_ok} exit {exit_ticks} damage {:.1} hp {hp} weapon {:#x} clip {} bond {:?} copter {:?} st {} bullets {:?}",
                w.total_ticks,
                sim.drones.copters[ci].damage, sim.gun.weapon, sim.gun.clip, sim.bond.pos, cp, sim.drones.copters[ci].state,
                sim.drones.player_bullets.iter().filter(|b| !b.dead).map(|b| [(b.pos[0]*10.0).round()/10.0, (b.pos[1]*10.0).round()/10.0, (b.pos[2]*10.0).round()/10.0]).collect::<Vec<_>>());
            if !line_ok {
                // Map the box: 16 azimuth rays + straight up — where does the eye see 30 u?
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                if exit_ticks == 1 {
                    // One-shot dump: the triangles around Bond name the box and its opening.
                    let lo = [b[0] - 5.0, b[1] - 3.0, b[2] - 6.5];
                    let hi = [b[0] + 5.0, b[1] + 4.0, b[2] + 6.5];
                    for (name, tris) in [("dyn", sim.world.dynamic_tris_near(lo, hi)), ("misl", sim.world.missile_tris_near(lo, hi))] {
                        eprintln!("[08 cabin] {name}: {} tris near bond", tris.len());
                        for (k, t) in tris.iter().enumerate().take(120) {
                            let c = [(t[0][0] + t[1][0] + t[2][0]) / 3.0, (t[0][1] + t[1][1] + t[2][1]) / 3.0, (t[0][2] + t[1][2] + t[2][2]) / 3.0];
                            eprintln!("[08 cabin] {name} t{k} c ({:.2},{:.2},{:.2})", c[0], c[1], c[2]);
                        }
                    }
                }
                let mut sweep = String::new();
                for k in 0..16 {
                    let az = k as f32 * std::f32::consts::TAU / 16.0;
                    let far = [eye[0] + az.sin() * 30.0, eye[1], eye[2] + az.cos() * 30.0];
                    let t = sim.world.segment_bullet(eye, far).unwrap_or(1.0);
                    sweep.push_str(&format!("az{az:.2}:{t:.2} "));
                }
                let tup = sim.world.segment_bullet(eye, [eye[0], eye[1] + 30.0, eye[2]]).unwrap_or(1.0);
                eprintln!("[08 hind t{t}] eye ({:.1},{:.1},{:.1}) UP t{tup:.2} sweep {sweep}", eye[0], eye[1], eye[2]);
                // Which ray stops, and where — the stop point names the blocker.
                let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
                let right = [cy, 0.0, -fy];
                for (name, from) in [("eye", eye), ("gunR", [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25]), ("gunL", [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25])] {
                    let tscope = sim.world.segment_bullet_scoped(sim.world.find_cel(from), from, cp);
                    let tflat = sim.world.segment_bullet(from, cp);
                    let desc = |t: Option<f32>| match t {
                        None => "CLEAR".to_string(),
                        Some(t) => {
                            let s = [from[0] + (cp[0] - from[0]) * t, from[1] + (cp[1] - from[1]) * t, from[2] + (cp[2] - from[2]) * t];
                            format!("stop t{t:.3} at ({:.1},{:.1},{:.1})", s[0], s[1], s[2])
                        }
                    };
                    eprintln!("[08 hind t{t}]   {name}: scoped {} flat {}", desc(tscope), desc(tflat));
                }
            }
        }
    }
    let damage = w.scene.sim.as_ref().unwrap().drones.copters[ci].damage;
    eprintln!("[08] the shoot-down: {rockets} rockets; damage {damage} vs hp {hp}; best single rocket {best_rocket}");
    // hind-flight-pin: the measured flight. The machine sweeps the valley the whole window —
    // the log's samples run (70.95, 22.16, −165.04) f25860 → (171.16, 16.32, −192.44) f26640 at
    // the path's 0.2533/t step, ≈120-170 u of travel per run, with NOT ONE 30-tick stretch below
    // 2 u — the task's pre-fix "holds one spot" is gone (Copter_Move's spline pose,
    // copter-path-flight). The case order is the game's [M Copter_Update 0x17dbb8]: 0 → 1 (fly)
    // with case 2 whenever the spot check lands (the 00 Agent sister's run samples a 2 — the
    // 45° cone off the path keys' facings points at Bond on her timeline) → the rockets cross hp
    // → state 4 (no kind-2 arrival path on 08 [D the route dump], rec+0x58 stays None) → 5, whose
    // ticks re-write ch 0x4B [M :255-275]. The death's 4/5 ticks fall between the 30-tick
    // samples, so the terminal state is pinned from the sim, not the samples.
    assert!(flight_samples.len() >= 10, "the flight was sampled every 30 ticks: {}", flight_samples.len());
    let mut travel = 0.0f32;
    for w2 in flight_samples.windows(2) {
        let d = ((w2[1].0[0] - w2[0].0[0]).powi(2) + (w2[1].0[1] - w2[0].0[1]).powi(2) + (w2[1].0[2] - w2[0].0[2]).powi(2)).sqrt();
        if w2[0].1 <= 2 && w2[1].1 <= 2 {
            // Two flying samples 30 ticks apart: the route's own speed carries it ≥ 2 u — a
            // held copter moves 0.
            assert!(d >= 2.0, "the flight never holds still: {:?} → {:?} moved only {d:.2}", w2[0], w2[1]);
        }
        travel += d;
    }
    eprintln!("[08 flight-pin] samples {} travel {travel:.1} u; states {:?}", flight_samples.len(),
        flight_samples.iter().map(|s| s.1).collect::<Vec<_>>());
    assert!(travel >= 100.0, "the flight travels its circuit, measured {travel:.1} u");
    // The case order: the flying samples are case 1, with case 2 wherever the spot check landed
    // [M :188-243]; the death (4) is pinned from the sim at the ch-0x4B break [M :358-370].
    let flying_states: Vec<u16> = flight_samples.iter().map(|s| s.1).collect();
    assert!(flying_states.iter().all(|&st| st == 1 || st == 2), "the flight is cases 1/2 [M :138-243]: {flying_states:?}");
    assert!(flying_states.contains(&1), "the flight flew [M :138]");
    let end_state = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.copters[ci].state
    };
    assert_eq!(end_state, 4, "the death state at the ch-0x4B break [M :358-370 — 08 is not an explode level]");
    assert!(ch4b, "the rockets pushed damage {damage} past hp {hp} → state 4 → rec+0x72 [M :358-370]");
    // The supply: a full clip (4 [L record 0x2C +0x92]) and the type-0xE carry cap (4 [L ammo_data
    // 0x2C69B8 + 0xE·12, +0x02]) — 8 on hand, the ride's three rocket pickups [D 0x0600004C streams
    // 37-39: two type-1 rocket ammo + one launcher, 4 rounds each, parented to the carriage] topping
    // the carry up. Aimed rockets miss now and then (the moving machine, the station's geometry).
    assert!(rockets <= 8, "the rockets on hand (clip 4 + carry cap 4) are all the walk had: {rockets} launched");
    // A rocket that lands on the body blasts there [M 0x12ce80's 0x4 arm → Explode_Propagate case 0]:
    // ≈100 from the blast + 1.0 contact. Sister: without the blast a body hit adds only 1.0.
    assert!(best_rocket >= 100.0, "a body hit carries the blast's ≈100, not just the 1.0 contact: best {best_rocket}");
    assert!(damage > hp, "the walk's accumulation, not a poke, crossed hp");
    for _ in 0..120 {
        if obj08(&w).1 == 4 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(obj08(&w).1, 4, "objective 'Escape with Zoe in gondola' completes on ch 0x4b [L record 3]");
    // ch 0x4B → mux #276 → ch 100 → the mission ends the level. The DRIVING hand-off is logged
    // (state 6 + LevelToEndTo 0x0730008), not a level_exit — the port skips driving codes
    // [M 0x182c60:85-116; mission.rs to_end match; docs/systems/mission.md §4].
    let mut end = (0u32, 0u32);
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        end = (sim.drones.mission.status(), sim.drones.mission.to_end);
        if end.0 == 6 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(end, (6, 0x0730_0008), "the mission completes into the driving hand-off 0x0730008 [M 0x182c60:85-116]");
    eprintln!("[08] the whole mission walked on real input in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
}

/// The ally rule (Zoe once shot at Bond at the gondola): the dtype-0xC ally
/// (KikoMission [M 0x14c938:54-59]) never targets or fires at Bond while Bond walks the whole
/// route. Sister: the placed guards DO engage (their combat states appear once the zone plates
/// fire) — the ally is not one of them.
#[test]
fn the_ally_never_shoots_bond_on_the_walk() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    // Walk the same first legs (the station, the walkway) — where the guards wake and fight.
    for (name, at) in [
        ("the intro plate", [139.2f32, 49.5, -81.2]),
        ("the station hall", [139.0, 49.5, -93.0]),
        ("the station doors", [139.5, 49.5, -97.5]),
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.wait(600);
    let sim = w.scene.sim.as_ref().unwrap();
    let ally = sim.drones.list.iter().enumerate()
        .find(|(_, d)| d.present && d.dtype == 0xc)
        .map(|(i, d)| (i, d.pos, d.state, d.team))
        .expect("the ally (dtype 0xC, Zoe) is placed");
    eprintln!("[08 ally] after the walk: {ally:?}");
    // The guards' ambush is up (the zone plates fired); the ally is NOT in a combat state against
    // Bond — KikoMission's target is never the player [M 0x14c938 dtype 0xC voice 0xC → 0x11].
    assert!(sim.drones.channels.on(0x2a) || sim.drones.channels.on(0x2b) || sim.drones.channels.on(0x2c),
        "the zone plates fired on the walk (the ambush is up)");
    let (ai, _, astate, _) = ally;
    assert!(!matches!(astate, 0x56..=0x7e | 0x85..=0x99),
        "the ally never enters a combat state on the walk [M KikoMission], state {astate:#x}");
    let _ = ai;
    assert!(!sim.drones.list.iter().any(|d| d.present && d.dtype == 0xc && d.hit_by_player),
        "the ally was never shot by Bond");
}

/// The heard-noise path (Zoe must never fire at Bond at the gondola): the dtype-0xC
/// ally rides the SAME walked route down to the gondola mouth and through the ride while the guards
/// fight Bond. The game's rule set (docs/systems/ally-heard-noise.md): FindOpponent never gives a
/// team-2 drone the player [M 0x1424e0:101-121], the heard-noise chain DOES walk her into Attack
/// 0x56 (Suspect's sound case 0x14 and Alert's timer case 0xc are unconditional SetState(0x56)
/// [M 0x16b0e0:118-124, 0x16b540:78-82]) but `DroneFunc_CombatState`'s opening gate sends an
/// opponentless drone to NoOpponent 0x60 [M 0x148338:20-31], CombatNoMove 0x59 has no 0x5b/0x5c
/// exit [M 0x1643e8:49-101] (the old port arm thrashed 0x59 ↔ 0x5c every tick), and every shot's
/// aim is the OPPONENT's position (`SetOpponentAimPos` per shot [M 0x179d40:130, 0x1422d0:70-73]) —
/// never Bond's chest. Sister: the team-1 guards on the same route still fight Bond.
#[test]
fn the_ally_fights_only_the_guards_through_the_noise_path() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
        // The briefcase channel stand-in (mux #235 ANDs it with the floor plate), as the ride walk does.
        sim.drones.channels.set(0x4a, 1, sim.drones.frame);
    }
    w.wait_enabled(3000, "the station start");
    w.wait(30);
    // The walked descent (the repro's route; the walkway plates wake the ambush).
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    w.wait_enabled(4500, "the station-intro scene");
    w.wait(60);
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        // No "station doors" stop (the whole-mission walk's note): its nav route runs east
        // through door (160.6, 45.0, −100.1) and back west along the lower deck, and the leg
        // aiming 3.4 above that deck wedged and dropped Bond into the station pit — out of the
        // map here too once the E reach changed the door timing (use-pick).
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("the ramp south", [153.1, 41.2, -116.0]),
        ("the ramp mid", [153.2, 39.7, -111.7]),
        ("the ramp turn", [148.3, 38.5, -110.9]),
        ("the mid level", [143.3, 36.5, -111.5]),
        ("the mid doors", [135.5, 36.5, -112.2]),
        ("the mid west", [132.1, 36.2, -112.3]),
        ("the long link", [120.5, 34.1, -112.5]),
        ("the hall y33", [118.2, 33.9, -111.0]),
        ("the hall y33 south", [113.2, 33.7, -106.4]),
        ("the stairwell top", [108.2, 31.5, -106.3]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.climb_stairs([107.25, 27.7, -119.5], "the stairwell down", 1.6, 4200);
    for (name, at) in [
        ("the lower level east", [110.7f32, 27.7, -121.5]),
        ("the gondola mouth", [112.0, 27.4, -122.0]),
        ("the platform west", [110.3, 27.3, -122.2]),
        ("the west ledge south", [110.0, 27.0, -124.5]),
        ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    // The briefcase channel stand-in again: the walked descent below runs ~16k ticks and some
    // writer clears ch 0x4a on the way (the ride walk's shorter leg set keeps it; the mux #235
    // needs it ON at the board). The real chain is walk_the_gondola_to_the_briefcase_and_back.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4a, 1, sim.drones.frame);
    }
    let (zi, zpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let hit = sim.drones.list.iter().enumerate()
            .find(|(_, d)| d.present && d.dtype == 0xc)
            .map(|(i, d)| (i, d.pos))
            .expect("the ally (dtype 0xC, Zoe) is placed");
        hit
    };
    eprintln!("[08 noise] zoe #{zi} at {zpos:?} after the walk (tick {}) ch4a {} ch1 {}", w.total_ticks,
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) as u8, w.scene.sim.as_ref().unwrap().drones.channels.on(1) as u8);
    // Board the carriage (the real floor plate), then ride while collecting.
    {
        let mut last = [0.0f32, 0.0];
        let mut still = 0u32;
        for t in 0..1200u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(4) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let target = [112.0f32, 28.0, -127.0];
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[08 noise board t{t}] bond {:?} ch4 {}", sim.bond.pos, sim.drones.channels.on(4) as u8);
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(4), "the floor plate fired ch 4 [D #236 mask 8]");
    }
    let mut ch83 = false;
    for t in 0..3600u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { ch83 = true; break; }
        w.tick(Input::default(), false);
        if t % 600 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let z = &sim.drones.list[zi];
            eprintln!("[08 noise ride t{t}] zoe state {:#x} pos {:?} opp {:?} shots {} | bond {:?} ch1 {} ch4 {} ch4a {}", z.state, z.pos, z.opp_drone, z.shots.len(), sim.bond.pos, sim.drones.channels.on(1) as u8, sim.drones.channels.on(4) as u8, sim.drones.channels.on(0x4a) as u8);
        }
    }
    assert!(ch83, "the ride ran to its ch-83 writer (the window covers the repro's fight) [D stream 46]");

    // ── the assertions ──────────────────────────────────────────────────────────────────────────
    let sim = w.scene.sim.as_ref().unwrap();
    let zoe = &sim.drones.list[zi];
    // Sister (can fail): a team-1 guard on the same route entered the combat family — the ambush
    // is alive and guard combat is not what this task broke.
    let guard_fought = sim.drones.list.iter()
        .any(|d| d.present && d.team == 1 && d.log.iter().any(|(_, s)| matches!(s, 0x56..=0x72 | 0x7f | 0x8c)));
    assert!(guard_fought, "sister: a team-1 guard fought on the same route [M the 0x228&4 combat arms]");
    // The ally walked the noise chain (the repro's 0xa2 → 0xa5 → 0x56 flow): the heard-noise
    // states or the Attack/Combat/NoOpponent family appear in her state log.
    let chain = zoe.log.iter().any(|(_, s)| matches!(s, 0xa2..=0xa5 | 0x56 | 0x58 | 0x60));
    assert!(chain, "the ally's noise chain ran (0xa2..0xa5 or 0x56/0x58/0x60 in her log), log {:?}",
        &zoe.log[..zoe.log.len().min(24)]);
    // No per-tick 0x59 ↔ 0x5c thrash: no (x, y, x) subsequence with x/y ∈ {0x59, 0x5c} spanning
    // ≤ 4 ticks.
    for win in zoe.log.windows(3) {
        let (fa, sa) = (win[0].0, win[0].1);
        let (_, sb) = (win[1].0, win[1].1);
        let (fc, sc) = (win[2].0, win[2].1);
        let thrash = matches!(sa, 0x59 | 0x5c) && matches!(sb, 0x59 | 0x5c) && sa == sc && sb != sa
            && fc.saturating_sub(fa) <= 4;
        assert!(!thrash, "the 0x59 ↔ 0x5c per-tick thrash is gone [M 0x1643e8 has no 0x5b/0x5c exit], window {win:?}");
    }
    // Every shot the ally fired aimed at her drone opponent — the aim ray misses Bond's own hit
    // cylinder (the port's guard-bullet volume, r 0.25) with the spread margin; a shot that
    // never intersects his cylinder cannot damage him [M 0x179d40:130 re-aims every shot].
    let bond_feet = [sim.bond.pos[0], sim.bond.pos[1] - 0.4, sim.bond.pos[2]];
    let vol = nf_game::bullet::PlayerVolume { feet: bond_feet, height: 1.8, radius: 0.25 };
    let mut bond_shots = 0usize;
    for (f, m, aim) in &zoe.shot_aims {
        let slack = [aim[0] - bond_feet[0], aim[1] - bond_feet[1], aim[2] - bond_feet[2]];
        let miss = (slack[0] * slack[0] + slack[1] * slack[1] + slack[2] * slack[2]).sqrt() > 1.0
            && nf_game::bullet::segment_cylinder(*m, *aim, &vol).is_none();
        if !miss { bond_shots += 1; }
        let _ = f;
    }
    assert_eq!(bond_shots, 0,
        "the ally never targets or fires at Bond on the noise path [M 0x1424e0:101 + 0x148338:20-31], aims {:?}",
        &zoe.shot_aims[..zoe.shot_aims.len().min(8)]);
    eprintln!("[08 noise] zoe states {:?} shots {} aims {}", &zoe.log[..zoe.log.len().min(20)], zoe.shots.len(), zoe.shot_aims.len());
    let _ = zpos;
}

/// Real damage: the whole-mission walk with the health top-up off for every walker it makes and
/// the walker's return fire on — Bond's death panics with the death spot and the last hits
/// (common/damage_log.rs), so a pass pins that the walk completes on the port's own damage.
/// No health/armour/ammo writes anywhere: the carried latches are the mission's ENTRY state
/// (set once at the cold spawn, header §The carried latches — in play they ride the RamSave
/// [M Player_RamSave 0x195670]), the fusebox wires, the safe bolts and the briefcase take are
/// the level's real interaction paths, and the gondola ride is the game's own script.
///
/// The fights pick weapon and range the way a player would (level-08-real-health-rockets): the
/// boarding platform's burst-rifle pair is answered with the GUN before the car
/// (`fight_close_guards`, real damage only), and the Hind fight fires the AT-420 only beyond
/// the blast's 10 u reach on a CLEAR corridor (`launch_line_clear`) — the old death at tick
/// 26696 was Bond's own rockets exploding on the bottom-station geometry 0.65–5.5 u from him
/// (−52.5/−7.5/−22.5/−3.7, frames 26477/26580/26639/26696) plus the pair's interval-4 bursts
/// (−9.0); docs/reports/level-08-long-range-hits.md.
#[test]
fn walk_the_whole_mission_on_real_input_real_damage() {
    walker::with_real_damage(walk_the_whole_mission_on_real_input);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-08-09-0a-0b): the
/// same walked station → fusebox → safe → briefcase → gondola → Hind route at the menu's Agent
/// pick — GameState+0x28 = 2 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group
/// (05-08) the pick scales Bond's pain by Plr_DMod_Normal 0.7 instead of Easy 0.6 and the
/// guards' hit chance by DroneFiring_Accuracy_Normal 1.1 instead of Easy 0.8 [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 05-08 group ← ReadTuningVars
/// 0x1d3a80]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input
/// only — the load path is the only difference.
#[test]
fn walk_the_whole_mission_on_real_input_real_damage_agent() {
    // HEAD-HUNT (the 05/07 00 Agent sisters' tactic on this level group): at Agent the guards'
    // hit chance is DroneFiring_Accuracy_Normal 1.1 (Operative 0.8) and Bond's pain is
    // Plr_DMod_Normal 0.7 (0.6) [M tuning.rs 05-08 group] — every tick a fight stands is a
    // burst tick, so the fights take the clear head box first (one head hit drops a 12-health
    // guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head]) instead of grinding bodies —
    // the plain pick's nearest-body order let the walkway shotgun guard #20 land 10 × −4.2.
    walker::with_real_damage_head_hunting(|| walk_the_whole_mission_on_real_input_at(2))
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-08-09-0a-0b): the same
/// walked station → fusebox → safe → briefcase → gondola → Hind route at the menu's 00 Agent
/// pick — GameState+0x28 = 3 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 2 carries 3 [D]; Txt 0x14f = "00 Agent"]. On this level
/// group (05-08) the pick scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.7) and the
/// guards' hit chance by DroneFiring_Accuracy_Hard 1.3 (Easy 0.8, Agent 1.1); the guards' base
/// bullet damage rides DroneDamage_Hard 0.8 (Agent 1.0) [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 cases 0x7000005-8;
/// tuning.rs 05-08 group]; the min-difficulty placements pass Drone_Create's gate at d2 AND d3
/// [M 0x136be0:49]. Real input only — the load path is the only difference (plus the d3-gated
/// tactics the deaths force, inside this file). The Hind fight's shoot-out fires on the
/// CONVERGED aim (the ray-gate fired nothing: the bounds-centre ray reads OPEN before a round
/// — 10/10 end panes die to one round each) and the launch gates read the CROSSHAIR the shot
/// itself flies through [M the crosshair aim law]: the tracker's convergence saturates the
/// free-aim cursor at its clamp and every launch sat up to 0.23 rad off the view — the chase
/// then clipped the window frames (the measured −75/−62.5 self-blasts [L the diag runs]).
#[test]
fn walk_the_whole_mission_on_real_input_real_damage_00agent() {
    walker::with_real_damage_head_hunting(|| walk_the_whole_mission_on_real_input_at(3))
}
