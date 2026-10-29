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

/// The whole mission on one fresh load: the intro, the fusebox, the safe, the briefcase
/// (objective 1 → 4), the descent back down, the boarding, the ride, the cinematic, the Hind
/// shoot-down on real aim and trigger (objective 2 → 4), and the mission end into 0x0730008.
#[test]
fn walk_the_whole_mission_on_real_input() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
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
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        ("the station doors", [139.5, 49.5, -97.5]),
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
        ("by the fusebox", [153.0, 43.4, -118.5]),
    ] {
        w.walk_nav_to(at, name, 1.5);
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
    let fusebox = [154.35f32, 43.41, -119.67];
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
    }
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
        eprintln!("[08] ride done: bond {:?} (started y {start_y:.1})", sim.bond.pos);
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
    }
    assert!(flying, "the gate ch 5 opened the route flight (state 1/2) [M 0x17dbb8 head]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b), "no arrival while the route flies");
    let ci = w.scene.sim.as_ref().unwrap().drones.copters.iter().position(|c| c.out == 0x4b).unwrap();
    let hp = w.scene.sim.as_ref().unwrap().drones.copters[ci].hp;
    // A real player fights the machine with a GUN, not the targeting laser (the walk arrives with
    // the 0x4E laser equipped from the safe burn; its beam carries no bullet contact at the
    // route's 24-37 u — the old test's point-blank re-stand hid that). Switch through the real
    // weapon-cycle input to the first owned gun the carry has (the laser 0x4E and the fists stay
    // out) — the cycle is unlocked by the carried ch 0x48 [M Player_Weapon 0x1a24c8:242-275].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        let own = |wid: u8| inv.owned.get(wid as usize).copied().unwrap_or(false);
        let ammo = |wid: u8| inv.clip.get(wid as usize).copied().unwrap_or(0) > 0
            || inv.carried.get(wid as usize).copied().unwrap_or(0) > 0;
        let owned: Vec<u8> = (0x02u8..0x4e).filter(|&wid| own(wid)).collect();
        eprintln!("[08] owned weapons at the fight: {owned:02x?} (current {:#x})", sim.gun.weapon);
        let gun = owned.into_iter()
            .find(|&wid| wid != 0x4e && wid != 0x01 && ammo(wid))
            .expect("the carry owns at least one loaded gun for the Hind fight");
        for _ in 0..240 {
            if w.scene.sim.as_ref().unwrap().gun.weapon == gun { break; }
            w.tick(Input { switch_to: Some(gun), ..Default::default() }, false);
        }
        assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, gun, "the switch to {gun:#x} never landed");
        eprintln!("[08] fighting the Hind with the gun {gun:#x}");
    }
    let mut shots = 0u32;
    let mut ch4b = false;
    for t in 0..40000u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b) { ch4b = true; break; }
        let sim = w.scene.sim.as_ref().unwrap();
        let cp = sim.drones.copters[ci].pos();
        let head = sim.bond.head();
        let to = [cp[0] - head[0], cp[1] - head[1], cp[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let pitch_err = desired_pitch - sim.bond.pitch;
        let input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            aiming: true,
            ..Default::default()
        };
        // Press/release the trigger at the flying Hind (the old fight's cadence, the aim now the
        // real tracked one).
        let press = w.total_ticks % 2 == 0;
        if press { w.tick_fire(input); shots += 1; } else { w.tick(input, false); }
        if t % 600 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 hind t{t}] dist {l:.1} damage {:.1} hp {hp} weapon {:#x} bond {:?} copter {:?} st {}",
                sim.drones.copters[ci].damage, sim.gun.weapon, sim.bond.pos, cp, sim.drones.copters[ci].state);
        }
    }
    let damage = w.scene.sim.as_ref().unwrap().drones.copters[ci].damage;
    eprintln!("[08] the shoot-down: {shots} trigger pulls; damage {damage} vs hp {hp}");
    assert!(ch4b, "the shots pushed damage {damage} past hp {hp} → state 4 → rec+0x72 [M :358-370]");
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
        ("the station doors", [139.5, 49.5, -97.5]),
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
