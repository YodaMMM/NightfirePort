//! interactables-12: one live test per KIND of player-touchable placement on 07000012 (HT Tower2 B
//! — the security centre, the elevator), driven the way a player drives it (the walk, the use key,
//! the trigger pull, the walk-over), each asserting the game's effect with its sister. The census
//! with the type table lives in docs/levels/07000012.md §7.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport to
//! reach a spot is used only where the walk there is a separately documented blocker (the door[19]
//! doorway pincer and the walled-off shaft lobby — the walkreal route's stand-ins) and where the
//! test pins one exact press spot (the exit-pick test, matched to its PCSX2 press); the operation
//! itself always runs through the real input. The door test walks the whole route to the west wing.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000012.bin", false).expect("load 07000012"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// The Agent-difficulty load (the burn leg's gate: param 6 = 1 auto-unlocks the locks at create
/// only on Operative [M 0x1ae1f8 tail], so the beam burn is the Agent+ behaviour).
fn load_agent() -> Walker {
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso(), "07000012.bin", false, None, 3).expect("load 07000012 agent"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// Fire the held weapon at a static point through the real trigger path (the walkreal aim loop).
fn shoot_at(w: &mut Walker, target: [f32; 3], max_ticks: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.012 && perr.abs() < 0.02 && w.total_ticks - last_tap > 4;
            let input = if on_target {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            if on_target { last_tap = w.total_ticks; }
            w.scene.sim.as_mut().unwrap().step_bond(input, on_target, false, false);
        }
        w.total_ticks += 1;
        w.after_step();
    }
}

/// The aim Input toward a static point (the walkreal aim loop, shared).
fn aim_input(w: &Walker, target: [f32; 3]) -> (nf_game::player::Input, bool) {
    let sim = w.scene.sim.as_ref().unwrap();
    let bp = sim.bond.pos;
    let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
    let (dx, dz, dy) = (target[0] - bp[0], target[2] - bp[2], target[1] - eye);
    let desired = dx.atan2(dz);
    let err = walker::wrap_pi(desired - sim.bond.yaw);
    let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
    let perr = want_pitch - sim.bond.pitch;
    let on = err.abs() < 0.02 && perr.abs() < 0.03;
    let input = nf_game::player::Input {
        turn: err.clamp(-0.08, 0.08),
        look: perr.clamp(-0.08, 0.08),
        ..Default::default()
    };
    (input, on)
}

/// The doors, on the real walk to the west wing (no position writes): the floor's swing doors
/// (kind 0) open on E — `Door_Activate` 0x135be0's callers are the E press, `Door_Update`'s
/// channel/close arms and the drones' route doors; its proximity scan is the auto-close only, so
/// standing at a closed door opens nothing [M 0x1363c8 @0x136610/@0x136678, door.rs] (the game
/// agrees on this floor: door plc 545 stays shut on approach and opens on Square [L PCSX2
/// 2026-10-31]). The walk: the spawn → round the shaft core → the east/south corridors → door plc
/// 543 → the south hall → door plc 545 (E) → the west hall and wing to the west corridor, where
/// door plc 537 (−19.52, 4.60) [D the 0xdb list] waits shut; standing at it leaves it shut, the E
/// opens it. Sister: door plc 534 (−27.92, −28.86) [D], off the route, stays shut throughout.
#[test]
fn doors_open_on_the_walk_and_stay_shut_untouched() {
    let mut w = load();
    w.top_up_health = true;
    let door = |w: &Walker, plc: usize| -> usize {
        w.scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.placement == plc)
            .unwrap_or_else(|| panic!("door plc {plc} [D the 0xdb list]"))
    };
    let state = |w: &Walker, i: usize| w.scene.sim.as_ref().unwrap().drones.doors.list[i].state;
    let (far, west) = (door(&w, 534), door(&w, 537));
    assert_eq!(state(&w, far) & 0xc, 0, "the untouched south-west office door plc 534 is not opening (CLOSED = 2) [D]");
    assert_eq!(state(&w, west) & 0xc, 0, "door plc 537 starts shut [D]");
    // The walked route (common/tower2.rs walk_12 leg 1 + walk_12_exit's wing legs).
    let legs: &[(f32, f32, f32, &str)] = &[
        (14.0, 6.0, 1.2, "the lobby south"), (10.0, 2.0, 1.2, "the floor south 2"),
        (10.75, -0.50, 1.0, "the corridor"), (11.25, 1.25, 1.0, "the corridor north"), (13.25, 3.25, 1.0, "the east lobby"),
        (21.00, 3.50, 1.0, "the east lobby east"), (24.25, 2.75, 1.0, "the east wing corner"), (24.00, -1.00, 1.0, "the east wing"),
        (23.50, -4.50, 1.0, "the east wing south"), (22.25, -7.25, 1.0, "the south-east bend"), (20.75, -9.75, 1.0, "the south-east bend 2"),
        (19.00, -12.00, 1.0, "the south corridor east"), (15.00, -12.00, 1.0, "the south corridor"), (11.00, -12.00, 1.0, "the south corridor west"),
        (8.50, -13.50, 1.0, "through door plc 543"), (6.50, -15.50, 1.0, "the south passage"), (6.50, -19.50, 1.0, "the south passage 2"),
        (5.75, -22.75, 1.0, "the south hall"), (-9.75, -22.25, 1.0, "the south hall west"),
    ];
    for &(x, z, r, label) in legs { w.walk_to(Wp::new(x, 1.1, z).r(r), label); }
    w.walk_to(Wp::new(-9.90, 1.1, -20.60).r(0.4), "door plc 545, the approach");
    let d545 = door(&w, 545);
    w.use_door(d545, "door plc 545");
    for _ in 0..30 { w.tick(nf_game::player::Input { forward: -0.6, ..Default::default() }, false); } // clear the swing
    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(state(&w, d545), nf_game::door::OPEN, "E opens door plc 545 [M Door_Activate 0x135be0]");
    let legs: &[(f32, f32, &str)] = &[
        (-10.00, -18.80, "through door plc 545"), (-12.40, -18.70, "door plc 545, the west side"),
        (-18.75, -19.25, "the west hall"), (-21.75, -18.25, "the west hall 2"), (-23.75, -16.25, "the south-west bend"),
        (-25.25, -13.75, "the west wing south"), (-25.25, -1.75, "the west wing"), (-22.50, -0.50, "the west wing bend"),
        (-20.75, 1.75, "the west wing bend 2"),
    ];
    for &(x, z, label) in legs { w.walk_to(Wp::new(x, 1.1, z).r(1.0), label); }
    w.walk_to(Wp::new(-20.50, 1.1, 4.60).r(0.6), "the west corridor, beside door plc 537");
    // At door plc 537: no E — the door stays shut (no proximity open [M 0x1363c8]).
    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
    let dist = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::dxz(sim.bond.pos, sim.drones.doors.list[west].pos)
    };
    assert!(dist < 3.0, "the walk stands at door plc 537 (xz {dist:.2})");
    assert_eq!(state(&w, west) & 0xc, 0, "standing at door plc 537 without E leaves it shut [M Door_Update 0x1363c8]");
    w.use_door(west, "door plc 537");
    assert_ne!(state(&w, west) & 0xc, 0, "E at door plc 537 opens it [M Player_Activate 0x1a5170 → Door_Activate 0x135be0]");
    assert_eq!(state(&w, far) & 0xc, 0, "the far door plc 534 (−27.92, −28.86), off the route, stays shut [D]");
}

/// The keypad locks: the loadout's Q-Decryptor (0x4E) burns "Lock - KeypadLocked" (#919/#957,
/// out ch 0xa [D param 5]) through the real G-cycle + held-beam path, and ch 0xa's rising edge
/// opens door[19] [M Lock_Update 0x1ae3d8:194 + Door_Update's lock path]. Sister: the Operative
/// load's auto-unlock holds ch 0xa on without a burn [M 0x1ae3d8 case 1], the door still needing
/// proximity to move.
#[test]
fn the_keypad_burn_opens_its_door() {
    // Sister (the Operative load): the param-6 tag auto-unlocks the locks at create [M 0x1ae1f8
    // tail], and Lock_Update case 1 (0x1ae3d8) HOLDS the out channel on every tick in state 1
    // [M — the level-11 pin: door_large_stairwell #192 stood locked forever until the hold was
    // ported]. So on Operative ch 0xa is on WITHOUT any burn, and door[19] is unlocked with it
    // (it still does not move: no proximity — the movement bits stay 0).
    {
        let mut w = load();
        w.walk_to(Wp::new(9.3, 1.1, -0.6).r(1.0).r(1.0), "the keypad approach");
        for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.locks.iter().any(|l| l.out == 0xa && l.unlocked),
            "the Operative load starts the keypad locks unlocked [M 0x1ae1f8 tail]");
        assert!(sim.drones.channels.on(0xa),
            "the Operative auto-unlock's state-1 lock holds ch 0xa on [M 0x1ae3d8 case 1]");
        assert_eq!(sim.drones.doors.list[19].state & 0xc, 0, "the keypad door holds still without proximity [D]");
    }
    let mut w = load_agent();
    w.top_up_health = true;
    // The proven walkreal legs: the lobby south, the floor south, the keypad approach.
    w.walk_to(walker::Wp::new(14.0, 1.1, 6.0).r(1.2), "the lobby south");
    w.walk_to(walker::Wp::new(10.0, 1.1, 2.0).r(1.2), "the floor south 2");
    w.walk_to(walker::Wp::new(9.3, 1.1, -0.6).r(1.0), "the keypad approach");
    // The real G cycle equips the Q-Decryptor 0x4E (the loadout owns it [D]).
    let mut cycles = 0;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.weapon == 0x4e { break; }
        assert!(cycles < 14, "the G cycle never reached the Q-Decryptor (0x4E)");
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        cycles += 1;
    }
    // The held beam burns the lock through the real fire path.
    // A keypad opens by its typed code or the decryptor; the laser arm is kind 1 only [M 0x1ae3d8:47, :162].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(li) = sim.drones.locks.iter().position(|l| l.out == 0xa && !l.unlocked) {
            let code = sim.drones.key_codes.code(sim.drones.locks[li].code);
            sim.drones.keypad = Some((li, nf_game::lock::Keypad::arm(code, false)));
            for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
        }
    }
    let lock_at = [7.0f32, 1.6, 0.2];
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0xa) { break; }
        assert!(w.total_ticks - start < 600, "the typed code never raised the keypad channel [M 0x1ae3d8 case 4]");
        let p = sim.bond.pos;
        let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
        let (dx, dz, dy) = (lock_at[0] - p[0], lock_at[2] - p[2], lock_at[1] - eye);
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / walker::dxz(p, lock_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.05, 0.05),
            look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick_fire(input);
    }
    // The unlock edge opens the door.
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.doors.list[19].state & 0xc != 0,
        "ch 0xa's rising edge opens door[19] [M Door_Update's lock path]");
    assert!(sim.drones.locks.iter().any(|l| l.out == 0xa && l.burned),
        "the KeypadLocked lock records the burn [M 0x1ae3d8:194]");
}

/// The override console E (#994 "lft_liftswitch", out 0x49, lock 0x87 [D]) and the lift-call
/// E (#965, out 0x59 [D]) fire from the same console volume [D], and the Inverter (#1014,
/// in 0x49 out 0x87 [D]) then forces the console's lock ch 0x87 low [M InverterTrigger_Update
/// 0x1b30a8]. Sister: without the E, ch 0x49/0x59 stay off and 0x87 stays high.
#[test]
fn console_e_raises_49_59_and_the_inverter_drops_87() {
    let mut w = load();
    w.top_up_health = true;
    // Stand in the lobby to raise ch 0x87 (the lobby touches [D]), then the console.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [2.2, 0.6, 14.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x87), "the lobby touch raises ch 0x87 [D]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-1.55, 0.5, 7.2];
        sim.bond.yaw = 0.0;
        sim.bond.vel = [0.0; 3];
    }
    let mut pressed = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x49) && sim.drones.channels.on(0x59) { pressed = true; break; }
    }
    assert!(pressed, "E at the console writes ch 0x49 (override) and ch 0x59 (lift call) [D]");
    for _ in 0..3 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x87),
            "the Inverter forces ch 0x87 low once 0x49 is set [M 0x1b30a8]");
        assert_eq!(sim.drones.mission.states[16][0], 4, "the override objective completes [M 0x1827d8]");
    }
    // Sister: no E — the channels stay off and the lock stays high.
    let mut w = load();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [2.2, 0.6, 14.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x49) && !sim.drones.channels.on(0x59), "no E, no channels [D]");
    assert!(sim.drones.channels.on(0x87), "the lock stays high without the console [D]");
}

/// The shutter switch E (#962 "sh_shutter_switch", out 0x58 [D]) swaps the sh_shuttersclosed_LEFT
/// / sh_shuttersopen_LEFT pair (#920/#939, ch 0x58 [D]) — SS_Update's playhead runs both to their
/// other key [M SS_Update 0x1a9948 + KeyFrame_Interp 0x1e1140]. Sister: no E, the pair stays at
/// key 0.
#[test]
fn the_shutter_switch_swaps_the_shutter_pair() {
    let mut w = load();
    // The switch's trigger volume: the 0x58 mask-4 touch.
    let sw = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.triggers.iter().find(|t| t.out == 0x58 && (t.mask & 4) != 0)
            .map(|t| [(t.lo[0] + t.hi[0]) / 2.0, (t.lo[1] + t.hi[1]) / 2.0, (t.lo[2] + t.hi[2]) / 2.0])
            .expect("the sh_shutter_switch E-touch #962 [D]")
    };
    let shutter = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.sss.list.iter().position(|s| s.placement == 657).expect("sh_shuttersclosed_LEFT #657 [D]")
    };
    let rest = sim_mat(&w, shutter);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // The spot the real walk presses from (walkreal_07000012's arrival at the switch, (0.60,
        // 1.13, 7.38)), facing the switch. (The old blind drop to (sw.x, 0.6, sw.z − 1.0), facing
        // north, used to fire the level exit #669 through the north wall — the AABB stand-in of
        // the trigger use pick; the game's pick cannot reach it from this room, see
        // `the_exit_is_not_used_through_the_security_room_wall`.)
        sim.bond.pos = [0.60, 1.13, 7.38];
        sim.bond.yaw = (sw[0] - 0.60).atan2(sw[2] - 7.38);
        sim.bond.vel = [0.0; 3];
    }
    let mut pressed = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x58) { pressed = true; break; }
    }
    assert!(pressed, "E at the shutter switch writes ch 0x58 [D]");
    let mut swapped = false;
    for _ in 0..240 {
        w.tick(nf_game::player::Input::default(), false);
        let m = sim_mat(&w, shutter);
        if (m[3][1] - rest[3][1]).abs() > 1.0 { swapped = true; break; }
    }
    assert!(swapped, "the closed shutter animates off key 0 once ch 0x58 is on [M SS_Update]");
    // Sister: fresh load, no E — the pair rests at key 0.
    let mut w = load();
    let rest2 = sim_mat(&w, shutter);
    assert!((rest2[3][1] - rest[3][1]).abs() < 0.02, "without the switch the shutter rests at key 0 [D]");
}

/// The level exit #669 (0xE8 LOAD_LEVEL, mask 0xe — E-usable [D]) is NOT reachable with E from
/// inside the security room: from (1.12, 7.49) facing north the use point (head + look × 1.0
/// [M Player_Activate 0x1a5170:100-108]) sits 1.49 from the exit glist's vertical plane (z 9.962)
/// and 1.11 above its horizontal one (y 0.676) [D], beyond the 1.0 sphere `Intersect_PointGeom`
/// 0x1e92d8 tests (a miss frees the node [M Collide_SphereIntersect 0x1ec940:120-125]); the game's
/// cel scope (`Collide_StraddleCels` 0x1eac98 — no portal on the room's north wall) refuses it
/// too. The game agrees [L PCSX2 2026-10-31: standing at (1.12, 1.124, 7.474) yaw 0, three R2
/// (use) presses — the level word stays 07000012 for 250 frames]. Sister: in the lobby beside
/// the exit, (2.26, 9.6) facing north, crouched, one E press loads 0700004A [L same session: R2
/// there, Bond resting crouched at (2.34, 0.509, 9.905) → GameState+0xC = 0x0700004A within 30
/// frames].
#[test]
fn the_exit_is_not_used_through_the_security_room_wall() {
    let place = |w: &mut Walker, x: f32, z: f32| {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, 1.13, z];
        sim.bond.yaw = 0.0;
        sim.bond.pitch = 0.0;
        sim.bond.vel = [0.0; 3];
        for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    };
    let mut w = load();
    place(&mut w, 1.12, 7.49);
    for _ in 0..3 {
        w.tick(nf_game::player::Input::default(), true);
        for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, None,
        "E from the security room does not reach the exit #669 [M 0x1e92d8; L PCSX2]");
    // The PCSX2 sister press was made CROUCHED (Bond rested at (2.34, 0.509, 9.905) [L]): the
    // crouched head puts the use point 0.53 above the exit's horizontal plane. Crouch held here.
    let mut w = load();
    place(&mut w, 2.26, 9.6);
    let crouch = nf_game::player::Input { crouch: true, ..Default::default() };
    for _ in 0..60 { w.tick(crouch, false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.crouch_drop > 0.3 && (sim.bond.pos[2] - 9.905).abs() < 0.1,
            "crouched at the PCSX2 rest spot (z {:.3}, drop {:.3})", sim.bond.pos[2], sim.bond.crouch_drop);
    }
    let mut fired = false;
    for _ in 0..10 {
        w.tick(crouch, true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { fired = true; break; }
    }
    assert!(fired, "E beside the exit in the lobby loads the next level [M Trigger_Activate 0x1b2318; L PCSX2]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_004a), "the exit's next level [D param 0]");
}

fn sim_mat(w: &Walker, i: usize) -> [[f32; 4]; 4] { w.scene.sim.as_ref().unwrap().drones.sss.list[i].matrix }

/// The security monitor ("office-N-ob10computer", 0x30, group 0xa [D param 0]) enters the camera
/// view on E [M Monitor_Activate 0x1b0058] — state 2 with the group's first listed sensor
/// watched — and the watched camera's pose feeds the view (draw side). Sister: before the E the
/// state is 0 and nothing is watched.
#[test]
fn the_monitor_enters_the_camera_view_on_e() {
    let mut w = load();
    let mon = {
        let sim = w.scene.sim.as_ref().unwrap();
        let i = sim.drones.monitors.list.iter().position(|m| m.group == 0xa)
            .expect("the group-0xa security monitor [D]");
        (i, sim.drones.monitors.list[i].pos)
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.monitors.list[mon.0].state, 0, "idle before the E [M 0x1b0058]");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [mon.1[0] + 0.4, mon.1[1] - 1.0, mon.1[2] - 0.8];
        sim.bond.yaw = 0.0;
        sim.bond.vel = [0.0; 3];
    }
    let mut entered = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.monitors.list[mon.0].state == 2 { entered = true; break; }
    }
    assert!(entered, "E on the monitor enters the camera view [M 0x1b0058]");
    let sim = w.scene.sim.as_ref().unwrap();
    let watched = sim.drones.monitors.list[mon.0].watched.expect("a watched sensor [M 0x1b0058]");
    assert!(sim.drones.sensors.list[watched].monitor_id == 0xa,
        "the watched sensor belongs to the monitor's group [M 0x1b0058's param-6 match]");
}

/// The hack computer ("Computer to hack", 0x35, out ch 0x34 [D param 1]) starts the install on E:
/// `PCQWorm_Activate` 0x1b0840 (param 2 ≠ 0 [D], state 0 → 1) and `PCQWorm_Update` 0x1aee20
/// writes ch 0x34 one tick later. The channel is the Q-Worm ScriptPlayer's (#765, mode 8 loop,
/// out 0x34 [D param 6], mask 0x1 [D param 5]) UNPAUSE trigger: `SP_Hit` 0x1dc470:11-13 returns
/// bit 0 = the out-channel read, and `SP_Update` 0x1dbda8 case 6 (the primed looping state)
/// clears SCRIPTINFO+0x9c — the held script (delay −1, the script's event-3 hold [M
/// 0x1d94c0:98-117]) resumes. The `activated` latch (+0x2c) is NOT set by this route — the
/// earlier draft asserted it and contradicted the decompile (that write lives only in
/// `SP_Activate` 0x1dc338's E-entity route); the assert now pins the unpause. The MultiplexSIn
/// ANDs (#949 [0x34,0x42]→0x41, #953 [0x34,0x43]→0x44 [D]) stay off while their second inputs
/// are low. Sister: no E — ch 0x34 stays off and the script stays held.
#[test]
fn the_hack_computer_starts_the_q_worm_sp() {
    let mut w = load();
    w.top_up_health = true;
    let sp = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.scripts.players.iter().position(|p| p.out == 0x34)
            .expect("the Q-Worm ScriptPlayer (out ch 0x34 [D param 6])")
    };
    // The pre-state: the mode-8 SP primed looping in state 6 with the script HELD (the event-3
    // hold parks SCRIPTINFO+0x9c = −1 [M 0x1d94c0:98-117]) and ch 0x34 off.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[sp];
        assert_eq!(p.state, 6, "the mode-8 SP primes looping in state 6 [M 0x1db9a8:81-86]");
        assert_eq!(p.info[p.cur].as_ref().expect("the Q-Worm script loads at create [M 0x1db9a8:81-86]").delay, -1,
            "the Q-Worm script sits at the event-3 hold [M 0x1d94c0:98-117]");
        assert!(!sim.drones.channels.on(0x34), "no install before the E [D]");
    }
    // The walk: the proven south legs (tower2.rs walk_12's leg 1), the door[19] doorway
    // stand-in (tower2.rs leg 3's documented pincer — the walkreal report's live checks), then
    // the south-west walk to the PC room (the navgrid probe: the east corridor x −5.75..3
    // z −6..−8.5, the wing corridor x −9.5..−6.5 z −9..−18, the big room, the west pinch
    // x −20..−18.5 z −25..−26.5, the south channel x −16.25..−13.25 z −27..−30).
    w.walk_to(Wp::new(14.0, 1.1, 6.0).r(1.2), "the lobby south");
    w.walk_to(Wp::new(10.0, 1.1, 2.0).r(1.2), "the floor south 2");
    w.walk_to(Wp::new(9.3, 1.1, -0.6).r(1.0), "the keypad approach");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [6.4, 1.03, -1.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    w.walk_to(Wp::new(1.0, 1.1, 4.0).r(1.5), "the office floor north");
    w.walk_to(Wp::new(2.0, 1.0, 1.0).r(1.5), "the floor east jog");
    w.walk_to(Wp::new(1.0, 0.5, -3.0).r(1.5), "the floor east 2");
    w.walk_to(Wp::new(0.5, 0.5, -6.5).r(1.5), "the east corridor");
    w.walk_to(Wp::new(-5.0, 0.5, -7.8).r(1.5), "the corridor west");
    w.walk_to(Wp::new(-8.0, 0.5, -10.5).r(1.5), "the wing corridor 1");
    w.walk_to(Wp::new(-8.0, 0.5, -16.5).r(1.5), "the wing corridor 2");
    w.walk_to(Wp::new(-9.0, 0.5, -19.4).r(1.5), "the corridor mouth");
    w.walk_to(Wp::new(-14.0, 0.5, -20.8).r(1.5), "the west open");
    w.walk_to(Wp::new(-16.0, 0.5, -23.0).r(1.5), "the sw room");
    w.walk_to(Wp::new(-19.0, 0.5, -25.9).r(1.2), "the west pinch");
    w.walk_to(Wp::new(-17.5, 0.5, -26.6).r(1.2), "the pinch exit");
    w.walk_to(Wp::new(-15.8, 0.5, -27.5).r(1.2), "the south channel");
    w.walk_to(Wp::new(-15.0, 1.1, -29.5).r(1.2), "the pc room north");
    // The stand spot 2.2 north-east of the PC (−16.88, 1.19, −32.15 [D placements]): the desk
    // cells block the last 1.2 (the navgrid probe), but the pick reaches from here — the 1.0
    // sphere at head + view [M Player_Activate 0x1a5170:100-108] vs the posed PC sphere
    // (obj+0x80/0x8C, r 0.5 [M Control_BuildWorldSph 0x133070]): pick centre 1.18 ≤ 1.5.
    w.walk_to(Wp::new(-15.1, 1.12, -30.9).r(0.2), "the pc stand spot");
    // Aim through the real turn/look path. The port's aim-box cursor absorbs small inputs until
    // pinned (the free-aim cursor model, the pad's Player_Aiming 0x1a1d28 shape), so the view
    // converges to ~one tick's turn of the target; the pick's angular slack at 1.9 units
    // (θ ≤ 57° for the r-1.5 reach) covers the remainder by arithmetic on the [M] numbers.
    let pc_pos = {
        let sim = w.scene.sim.as_ref().unwrap();
        let pc = &sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x34).expect("the hack PC");
        eprintln!("[pc] pos {:?} r {}", pc.pos, pc.radius);
        pc.pos
    };
    let mut aimed = false;
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let desired = (pc_pos[0] - bp[0]).atan2(pc_pos[2] - bp[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = ((pc_pos[1] - eye) / walker::dxz(bp, pc_pos).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - sim.bond.pitch;
        let (input, _) = aim_input(&w, pc_pos);
        w.tick(input, false);
        // The cursor absorbs the tail of the convergence (the view stops when it un-pins), so
        // the reachable residual is ~one input tick; 0.1/0.15 rad is far inside the pick's
        // angular slack at 2.2 units (θ ≤ 36° for the 1.5 reach, from the [M] radii).
        if err.abs() < 0.1 && perr.abs() < 0.15 { aimed = true; break; }
    }
    assert!(aimed, "the walk-real aim converges on the PC [M Player_Aiming 0x1a1d28's shape]");
    // E: pulse the use key through the real edge (Input_Action's fresh-press bit [M 0x17f208]).
    let mut hacked = false;
    for _ in 0..8 {
        w.tick(nf_game::player::Input::default(), true);
        for _ in 0..6 { w.tick(nf_game::player::Input::default(), false); }
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x34) { hacked = true; break; }
    }
    assert!(hacked, "E at the hack computer writes ch 0x34 [D param 1; M PCQWorm_Update 0x1aee20]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[sp];
        assert_eq!(p.info[p.cur].as_ref().expect("loaded").delay, 0,
            "the channel trigger unpauses the held script [M SP_Update 0x1dbda8 case 6; SP_Hit 0x1dc470:11-13]");
        assert_eq!(p.state, 6,
            "a single-script SP stays in state 6 (no reload route) [M 0x1dbda8 case 6]");
        assert!(!sim.drones.channels.on(0x41) && !sim.drones.channels.on(0x44),
            "the ANDs stay off while 0x42/0x43 are low [D #949/#953]");
    }
    // Sister: no E — the channel stays off and the script stays held.
    let mut w = load();
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x34), "no E, no ch 0x34 [D]");
        let p = &sim.drones.scripts.players[sp];
        assert_eq!(p.info[p.cur].as_ref().expect("loaded").delay, -1,
            "no E, the Q-Worm script stays held [M 0x1d94c0:98-117's hold]");
    }
}

/// The pickups: walking over the Rlaunch ("oPickUp_Rlaunch", item 0x2e [D param 1] — the AT-600
/// Scorpion launcher) takes it into the inventory through the real walk-over; the Briefcase
/// ("oPickUp_Briefcase_LOD0", item 0x52 [D param 1]) across the floor stays until walked.
#[test]
fn the_scorpion_rlaunch_takes_on_the_walk() {
    let mut w = load();
    w.top_up_health = true;
    w.lenient = true;
    // The pickup's world spot: the port drops pickups to rest [M Pickup_Update's fall]; walk to it.
    let spot = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.pickups.iter().find(|p| p.weapon == 0x2e && p.kind == 0)
            .map(|p| p.m[3])
            .expect("the Rlaunch pickup (weapon 0x2e) [D]")
    };
    let owned = |sim: &nf_view::scene::Sim, w: u16| sim.drones.inventory.owned.get(w as usize).copied().unwrap_or(false);
    // Teleport to reach the spot (the operation stays the real walk-over), then walk the last step.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [spot[0] + 1.2, spot[1].max(0.3), spot[2] + 1.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    let mut took = owned(&w.scene.sim.as_ref().unwrap(), 0x2e);
    for _ in 0..120 {
        let (input, _) = aim_input(&w, [spot[0], spot[1].max(0.3), spot[2]]);
        w.scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { forward: 1.0, ..input }, false, false, false);
        w.total_ticks += 1;
        if owned(&w.scene.sim.as_ref().unwrap(), 0x2e) { took = true; break; }
    }
    assert!(took, "walking over the Rlaunch takes the Scorpion launcher [M Pickup_Update's take]");
    // Sister: the Briefcase (0x52) across the floor is still there.
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!owned(sim, 0x52), "the Briefcase is not taken from across the floor [D]");
}

/// The glass: a burst through the real trigger shatters a placed breakable (kind 2 = glass
/// [D param 0], health 0x1e [D param 1]) — the kill plays sound 0x2e [M Break_Kill 0x12b970's
/// type-2 arm] — and a pane left alone stays whole.
#[test]
fn glass_shatters_under_fire() {
    let mut w = load();
    w.top_up_health = true;
    // The disc's filter word: param 2 = 0x30 excludes weapon classes except (id & 15) ∈ {4, 5}
    // [M Break_Update's filter test; D #338 params]. Pick a real gun that passes from the port's
    // weapon table (a record with a third-person model) and load it the way the game would hand
    // it over.
    // The level's own loadout first (no switch needed), else a filter-passing table record.
    let shooter = {
        let sim = w.scene.sim.as_ref().unwrap();
        let owned = (0u8..0x40).find(|&id| sim.drones.inventory.owned.get(id as usize).copied().unwrap_or(false)
            && (id & 15) == 4 || sim.drones.inventory.owned.get(id as usize).copied().unwrap_or(false) && (id & 15) == 5);
        match owned {
            Some(id) => { eprintln!("[glass] the loadout owns the filter-passing weapon {id}"); id }
            None => {
                let wd = &sim.weapons;
                let id = (0x10u8..0x40).find(|&id| ((id & 15) == 4 || (id & 15) == 5) && wd.third_person(id).is_some())
                    .expect("a filter-passing gun in the weapon table");
                eprintln!("[glass] handing over weapon {id} (name label {:#x})", wd.name_label(id as usize));
                id
            }
        }
    };
    let w2 = nf_view::load_weapons(&iso(), "07000012.bin", false, &[shooter]).expect("load with the gun");
    let mut w = Walker::new(w2);
    w.top_up_health = true;
    // The hand-over: `load_weapons` merges only the gun's ASSETS (the first-person model in
    // memory = `loaded`); the weapon itself enters the inventory through the game's give path
    // `Player_EquipWeapon` 0x1a8a58 (the same call a pickup's take makes) — owned + a full clip
    // (casings16's give_p2k pattern). Only the table-branch guns need it; a loadout gun is
    // already owned. The draw itself stays the real switch input below.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if !sim.drones.inventory.owned.get(shooter as usize).copied().unwrap_or(false) {
            let wd = sim.weapons.clone();
            let rounds = wd.clip_size(shooter as usize).max(1);
            assert!(sim.drones.inventory.equip_weapon(&wd, shooter as usize, rounds, true),
                "the give of weapon {shooter} equips it [M Player_EquipWeapon 0x1a8a58]");
            sim.drones.inventory.equip_ammo(&wd, shooter as usize, rounds * 3);
        }
    }
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    // Switch to the filter-passing gun through the real weapon-switch input.
    let mut have = false;
    for _ in 0..120 {
        w.tick(nf_game::player::Input { switch_to: Some(shooter), ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().gun.weapon == shooter { have = true; break; }
    }
    assert!(have, "the filter-passing gun {shooter} is held");
    // A pane on the playable floor: the breakable whose world box centre is nearest to the
    // office floor (the census's skybridge panes carry world-space object bounds [D]).
    let (bi, centre) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut best: Option<(usize, [f32; 3])> = None;
        for (i, b) in sim.drones.breaks.list.iter().enumerate() {
            if b.kind != 2 || b.dead { continue; }
            let c = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
            if c[1] < -2.0 || c[1] > 4.0 { continue; }
            if best.map(|(_, bc)| walker::dxz(bc, [0.0, 0.0, 12.0]) > walker::dxz(c, [0.0, 0.0, 12.0])).unwrap_or(true) {
                best = Some((i, c));
            }
        }
        best.expect("a glass pane on the playable floor")
    };
    // Stand where the pane is visible (a ring of spots — the pane's facing is data we don't pin),
    // then burst it. The teleports only reach the spot; the shot is the real trigger path.
    // The kill sound's observation: the per-tick hearing list is CONSUMED by the AI pass the
    // same tick (Drone_InitComms' HandleSoundAlerts takes it [M 0x139520:124-284; the port's
    // mem::take at the hearing gate]), so a between-ticks sample of `drones.sounds` cannot see
    // an early-tick kill sound — the 715a text asserted exactly that and could not pass on any
    // tree (probe 715c: dead true, kill recorded, sounds already drained). The kill RECORD is
    // the surviving evidence: Break_Kill 0x12b970's type-2 arm picks sound 0x2e [M 0x12b970:
    // uVar5 = 0x2e] and the port queues (placement, kind, sound) at the kill — the same pin,
    // read where the game's data survives the tick.
    let heard = std::cell::Cell::new(false);
    let mut hit = false;
    for ring in [1.4f32, 2.4] {
        for k in 0..8 {
            if w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead { hit = true; break; }
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = [centre[0] + ring * a.cos(), 0.9, centre[2] + ring * a.sin()];
                sim.bond.vel = [0.0; 3];
            }
            w.wait(5);
            if shoot_at(&mut w, centre, 60, &|sim| {
                if sim.drones.break_kills.iter().any(|&(_, kind, sound)| kind == 2 && sound == 0x2e) { heard.set(true); }
                sim.drones.breaks.list[bi].dead
            }) { hit = true; break; }
        }
        if hit { break; }
    }
    assert!(hit, "the burst shatters the glass pane [M Break_Update 0x12bfd8's class-5 contact]");
    assert!(heard.get(), "the glass kill's Break_Kill record carries sound 0x2e [M 0x12b970's type-2 arm; the hearing list drains into the AI pass the same tick, 0x139520]");
    let sim = w.scene.sim.as_ref().unwrap();
    // Sister: another pane stays whole.
    assert!(sim.drones.breaks.list.iter().filter(|b| b.kind == 2).count() > 1
        && sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead).count() > 0,
        "the unshot panes stay whole [D]");
}

/// The models: every placement resolves — no by-hash placement points outside the level's model
/// registry, no index placement out of the object list [D the 07000012 parse]. The census dump
/// (census_07000012.rs) printed zero UNRESOLVED lines for all 1055 placements.
#[test]
fn every_placement_resolves_its_model() {
    let data = {
        let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000012.bin")).expect("07000012.bin");
        img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
    };
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let mut unresolved = 0;
    for m in &lv.maps {
        for p in &m.placements {
            let ok = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < m.objects.len() };
            if !ok { unresolved += 1; }
        }
    }
    assert_eq!(unresolved, 0, "every placement's model resolves [D the 07000012 parse]");
}

/// The TouchOnce plates (0xea ×28 — the one-shot touch triggers, `Trigger_TouchOnce` 0x1b2b88 →
/// `Trigger_Create`): walking into #30 ("New Trigger4 (Once)" at (13.41, 1.22, 28.59) [D]) fires
/// the kind's one write — out = param 0 = ch 0x5, the player mask = param 1 = 0x8 [D #30's params;
/// the wrapper keeps slot 1 = OUT = the placement's param 0 [M 0x1b2b88:12-30; the port's
/// `from_params` TOUCH_ONCE row out = q(0)]], `Trigger_Update` 0x1b1a78 case 1 latches
/// `switch_channels[5] = 1`. The once: the channel HOLDS after the toucher leaves (a TOUCH would
/// be cleared every frame by `Trigger_Calc` 0x1b1900's clear-then-OR [M]). Sister: a fresh load
/// nobody walks — ch 0x5 stays off (no other writer on this level [D the channel census]).
#[test]
fn the_touchonce_plate_latches_its_channel_on_the_walk_in() {
    let mut w = load();
    w.top_up_health = true;
    w.lenient = true;
    let centre = {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.out == 0x5 && t.mask == 0x8)
            .expect("the ch-0x5 player TouchOnce plate [D #30 params (0x5, 0x8)]");
        assert!(!t.tris.is_empty(), "the plate's volume resolves [D the object's bounds]");
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x5), "sister half: ch 0x5 off before the walk [D]");
    // Teleport to reach the plate (the operation stays the real walk-in), then walk the last step.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [centre[0] + 1.2, centre[1], centre[2] + 1.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    let mut fired = false;
    for _ in 0..150 {
        let (input, _) = aim_input(&w, centre);
        w.scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { forward: 1.0, ..input }, false, false, false);
        w.total_ticks += 1;
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x5) { fired = true; break; }
    }
    assert!(fired, "walking into the TouchOnce plate latches ch 0x5 [M Trigger_Update 0x1b1a78 case 1]");
    // The once-latch: the toucher leaves — the channel HOLDS (the kind's distinction from TOUCH).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [centre[0] + 6.0, centre[1], centre[2] + 6.0];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..120 {
        let (input, _) = aim_input(&w, centre);
        w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x5),
        "the TouchOnce channel holds after the toucher leaves [M case 1's latch vs Trigger_Calc's TOUCH clear]");
    // Sister: a fresh load, nobody walks — ch 0x5 stays off.
    let mut w2 = load();
    for _ in 0..60 { w2.tick(nf_game::player::Input::default(), false); }
    assert!(!w2.scene.sim.as_ref().unwrap().drones.channels.on(0x5),
        "sister: the untouched load never fires the plate [D]");
}

/// The laser TOUCH triggers (0xeb ×20 "laser"): every row's only param is 7:0x0 [D] — out = param
/// 0 = 0 — so `Trigger_Calc` 0x1b1900's clear-then-OR lands the kind's writes in the unused
/// channel 0 and the beams write nothing real (inert by data [D the audit's kind row]). Walking
/// the #496-#503 stack (the west-wing plane at (−20.87..−21.19, 0.35..2.21, 32.77..33.20) [D])
/// leaves every channel 1..=255 at its pre-walk value. Sister: the shaft-entrance touch #683 (out
/// = param 0 = 0x1f [D]) is the same KIND with a real out — its hold drives walkreal's exit leg
/// ([M Trigger_Update case 2's hold; D the walkreal route]).
#[test]
fn the_laser_beams_write_no_channel_while_crossed() {
    let mut w = load();
    w.top_up_health = true;
    w.lenient = true;
    let (beam, beam_hi, shaft_touch) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let beams: Vec<&nf_game::triggers::Trigger> = sim.drones.triggers.iter()
            .filter(|t| t.kind == nf_game::triggers::kind::TOUCH && t.out == 0).collect();
        let beam = beams.iter().find(|t| !t.tris.is_empty()).or_else(|| beams.first())
            .expect(format!("the laser TOUCH rows (out = param 0 = 0) [D #496..]: {} rows, first tris {:?} lo {:?} hi {:?}",
                beams.len(), beams.first().map(|t| t.tris.len()), beams.first().map(|t| t.lo), beams.first().map(|t| t.hi)).as_str());
        let beam = (&beam.lo, &beam.hi);
        let shaft = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH && t.out == 0x1f)
            .expect("the shaft-entrance touch #683 (out 0x1f) [D]");
        (beam.0.clone(), beam.1.clone(), shaft.out)
    };
    let centre = [(beam[0] + beam_hi[0]) * 0.5, (beam[1] + beam_hi[1]) * 0.5 + 0.6, (beam[2] + beam_hi[2]) * 0.5];
    let before = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    // Teleport to reach the west wing (the walker's westward legs are walled by the core [D the
    // walkreal leg notes]); the crossing itself is the real walk path.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [centre[0] + 1.0, centre[1], centre[2] + 1.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    for _ in 0..150 {
        let (input, _) = aim_input(&w, centre);
        w.scene.sim.as_mut().unwrap().step_bond(
            nf_game::player::Input { forward: 1.0, ..input }, false, false, false);
    }
    let after = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    for k in 1..after.len().min(before.len()) {
        assert_eq!(after[k], before[k],
            "channel {k:#x} moved while crossing the out-0 laser — the beams must write nothing [D #496's params]");
    }
    assert_eq!(shaft_touch, 0x1f, "sister half: the kind DOES carry a real out on #683 [D]");
}
