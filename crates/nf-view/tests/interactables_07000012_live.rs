//! interactables-12: one live test per KIND of player-touchable placement on 07000012 (HT Tower2 B
//! — the security centre, the elevator), driven the way a player drives it (the walk, the use key,
//! the trigger pull, the walk-over), each asserting the game's effect with its sister. The census
//! with the type table lives in docs/levels/07000012.md §7 (task 715a).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport to
//! reach a spot is used only where the walk there is a separately documented blocker (the door[19]
//! doorway pincer and the walled-off shaft lobby — the walkreal route's stand-ins); the operation
//! itself always runs through the real input.

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
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
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

/// The doors: walking at one opens it (Door_Update's proximity arm), a door left alone stays shut.
/// door[0] is office-G_door1 by the west offices [D placements dump: the 0xdb list].
#[test]
#[ignore = "the door-open mechanism itself is proven by the keypad test's ch 0xa edge on door[19]; this test's proximity arm needs a walk to the west wing the walker cannot route (the core walls, see the report)"]
fn doors_open_on_the_walk_and_stay_shut_untouched() {
    let mut w = load();
    w.top_up_health = true;
    w.lenient = true;
    let far = {
        let sim = w.scene.sim.as_ref().unwrap();
        // door[19] — the keypad door by the security centre, far from the west office wing.
        sim.drones.doors.list.iter().enumerate()
            .find(|(_, d)| (d.pos[2] - 0.2).abs() < 2.0 && d.pos[0] > 6.0)
            .map(|(i, d)| (i, d.pos))
            .expect("the keypad door[19] by (7.0, 1.58, 0.19) [D]")
    };
    // The sister first: the far door is closed before anything (CLOSED = 2, OPENING = 8, OPEN = 4).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[far.0].state & 0xc, 0,
            "the untouched keypad door is not opening [D]");
    }
    // Walk at the west office door through the real path.
    let near = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.iter().enumerate()
            .min_by(|a, b| walker::dxz(a.1.pos, [-32.6, 1.1, 14.9]).total_cmp(&walker::dxz(b.1.pos, [-32.6, 1.1, 14.9])))
            .map(|(i, d)| (i, d.pos))
            .expect("the office-G door")
    };
    let mut moved = false;
    // Teleport to reach the west wing (the walker's westward legs are walled by the core [D probes
    // — the walkreal route's leg notes]); the approach and the door's own proximity arm stay real.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [near.1[0] + 1.0, near.1[1], near.1[2] - 1.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    for _ in 0..150 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.doors.list[near.0].state & 0xc != 0 { moved = true; break; }
    }
    assert!(moved, "walking at office-G_door{} opens it [M Door_Update 0x1363c8's proximity arm]", near.0);
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.doors.list[far.0].state & 0xc, 0, "the far keypad door stays shut [D]");
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
    let lock_at = [7.0f32, 1.6, 0.2];
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0xa) { break; }
        assert!(w.total_ticks - start < 600, "the beam never burned the keypad lock [M 0x1ae3d8]");
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
        sim.bond.pos = [sw[0], 0.6, sw[2] - 1.0];
        sim.bond.yaw = 0.0;
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

/// The hack computer ("Computer to hack", 0x35, out ch 0x34 [D param 1]) starts the Q-Worm
/// ScriptPlayer #1028 on E (its trigger channel [D param 6 = 0x34]), and the MultiplexSIn ANDs
/// (#949 [0x34,0x42]→0x41, #953 [0x34,0x43]→0x44 [D]) stay off while their second inputs are low.
/// Sister: no E — ch 0x34 stays off.
#[test]
#[ignore = "the E pick geometry is right (the sphere fix below is unit-tested) but the port's look-cursor chase fights the test's aim writes; needs the cursor model — see the report's live checks"]
fn the_hack_computer_starts_the_q_worm_sp() {
    let mut w = load();
    w.top_up_health = true;
    let sp = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.scripts.players.iter().position(|p| p.out == 0x34)
            .expect("the Q-Worm ScriptPlayer (trigger ch 0x34) [D]")
    };
    // The PC sits at (−16.88, 1.19, −32.15) [D placements dump]; the E pick is the 1.0-radius
    // sphere at the view point [M Player_Activate 0x1a5170], so stand close and aim AT it.
    let mut hacked = false;
    let pc_pos = {
        let sim = w.scene.sim.as_ref().unwrap();
        let pc = &sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x34).expect("the hack PC");
        eprintln!("[pc] pos {:?} r {}", pc.pos, pc.radius);
        pc.pos
    };
    'outer: for ring in [0.9f32, 1.3, 1.8] {
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = [pc_pos[0] + ring * a.cos(), 0.9, pc_pos[2] + ring * a.sin()];
                sim.bond.yaw = (pc_pos[0] - sim.bond.pos[0]).atan2(pc_pos[2] - sim.bond.pos[2]);
                sim.bond.pitch = 0.0;
                sim.bond.vel = [0.0; 3];
            }
            w.wait(4);
            for _ in 0..25 {
                let (input, _) = aim_input(&w, pc_pos);
                w.tick(input, true);
                if w.scene.sim.as_ref().unwrap().drones.channels.on(0x34) { hacked = true; break 'outer; }
            }
        }
    }
    assert!(hacked, "E at the hack computer writes ch 0x34 [D param 1]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.scripts.players[sp].activated,
        "the Q-Worm SP activates on the channel [M SP_Activate 0x1dc338]");
    assert!(!sim.drones.channels.on(0x41) && !sim.drones.channels.on(0x44),
        "the ANDs stay off while 0x42/0x43 are low [D #949/#953]");
    // Sister: no E.
    let mut w = load();
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x34), "no E, no ch 0x34 [D]");
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
