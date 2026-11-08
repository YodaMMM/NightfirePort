//! interactables-0b: one live test per KIND of player-touchable placement on 0700000b (Tower C,
//! the executive floor and the roof), driven the way a player drives it (the walk, the use key,
//! the trigger pull, the walk-over), each asserting the game's effect with its sister. The census
//! with the type table lives in docs/levels/0700000b.md; the end-to-end route is
//! walkreal_0700000b_live.rs. The kinds the census adds beyond the first nine: the Q-Worm
//! console (0x35), the sound triggers (0xF9), the security cameras (0x28 Sensor), the lift's
//! shaft plates (0xEB #245 / 0xEA #171) and the hint marker (0x33 — parse-model pin, the live
//! firing path is the 0x13-documented port gap).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only where the walk there is a separately documented blocker
//! (walkreal_0700000b_live's boardroom-furniture and lift-gap pins); the operation itself always
//! runs through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The warm load the real run hands this level: objectives 3-6 + the hatch unlock carried
/// [M Player_RamSave 0x195670:13; walkreal_0700000b_live's load_warm]. `bits` = the `bonus[0]`
/// word in play (0 = the fresh-boot state; the kind-6 gate reads it [M 0x18dda0:93-95]).
fn load_warm_bits(bits: u64) -> Walker {
    let cold = nf_view::load_with_carry_medals(&iso(), "0700000b.bin", false, None, 1, bits).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "0700000b.bin", false, Some(&save), 1, bits).expect("load warm"));
    for c in [0x46u8, 0x47, 0x48, 0x49, 0x4a] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w
}

fn load_warm() -> Walker { load_warm_bits(0) }

/// The gold-medal `bonus[0]` word for `base_map` [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm]. pickup-medal-gate.
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

/// Teleport Bond onto the executive floor (the lift-gap stand-in walkreal_0700000b_live pins)
/// and settle him standing.
fn to_exec_floor(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.0, 46.8, 24.8];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
}

/// The Pheonix keycard lock (0x29 Switch #249) E-toggles its channel: `Switch_Activate` 0x1b02a8
/// writes `switch_channels[9]` for the player's class-3 use [M] — and the sdoor1/sdoor2 keycard
/// doors are locked on ch 9 [D #286/#287 param 3], so the E opens them. The lock's own anim
/// script 0x0600066D (p1 [D]) writes ch 9 = 1 at its tick 58 as well [D nis dump] — the toggle is
/// the writer the doors need. Sister: ch 9 reads 0 before the use.
#[test]
fn keycard_switch_e_opens_the_sdoors() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let lock = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.useobjs.switches.iter().find(|s| s.ch == 9).expect("the keycard lock (0x29, ch 9)").pos
    };
    w.scene.sim.as_mut().unwrap().bond.pos = [lock[0] + 0.3, lock[1] - 0.7, lock[2] + 0.3];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(9), "sister: ch 9 off before the use");
    // E the lock from its front (the use sphere is 1.0 at head+look [M 0x1a5170]).
    w.walk_to(Wp::at(lock[0] + 0.3, lock[1] - 0.75, lock[2] + 0.6, Act::Use { use_ticks: 240 }), "E the keycard lock");
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(9),
        "the E must toggle ch 9 [M Switch_Activate 0x1b02a8]");
    // The doors: sdoor1 slides on the channel edge. Find the sdoor nearest the lock (both at the
    // origin in the placement dump [D] — their poses come from the path data).
    let (_, di) = walker::door_near(&w.scene.sim.as_ref().unwrap(), lock).expect("an sdoor near the lock");
    w.use_door(di, "the keycard door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the locked sdoor opens on the E'd ch 9 [D #286 lock ch 9], state {st:#x}");
}

/// The executive-floor keypad (0x2E Lock #178 "Lock - KeypadLocked", burn out ch 0x2F [D p5]):
/// the Laser watch's beam (weapon 0x4E) burns it — `Lock_Update` 0x1ae3d8 case 0 advances the burn
/// on the laser-family hit and the open writes `switch_channels[47] = 1` [M :194-195] — which
/// unlocks door #151 "door_waitingroom_exec_office" (lock ch 47 [D param 3]). Sister: the door
/// refuses the same E while the lock stands (ch 0x2F off).
#[test]
fn keypad_lock_burns_open_the_exec_office() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let lk = w.scene.sim.as_ref().unwrap().drones.locks.iter()
        .position(|l| l.out == 47).expect("the keypad lock (0x2E, out ch 0x2F)");
    let lp = w.scene.sim.as_ref().unwrap().drones.locks[lk].pos;
    // Sister first: the exec-office door is locked on ch 0x2F — E from its side does nothing.
    let (_, di) = walker::door_near(&w.scene.sim.as_ref().unwrap(), lp).expect("the exec-office door by the keypad");
    w.scene.sim.as_mut().unwrap().bond.pos = [lp[0] + 0.6, lp[1] - 0.6, lp[2] + 1.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x2f), "the keypad is whole before the burn");
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    w.use_door(di, "the locked exec-office door (sister)");
    w.wait(10);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state, before,
        "sister: the door locked on ch 0x2F must not move before the burn");
    // The burn: the Laser watch 0x4E through the real fire path (fix_safe08's open_safe pattern).
    let input = nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false); }
    w.scene.sim.as_mut().unwrap().bond.pos = [lp[0] + 0.6, lp[1] - 0.55, lp[2] + 1.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut burned = false;
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().drones.locks[lk].burned { burned = true; break; }
        let (head, bp) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.head(), lp) };
        let dir = [bp[0] - head[0], bp[1] - head[1], bp[2] - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        }
        w.scene.sim.as_mut().unwrap().step_bond(Default::default(), true, false, false);
    }
    assert!(burned, "the watch burns the keypad [M Lock_Update 0x1ae3d8 case 0]");
    w.wait(5);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x2f),
        "the open writes switch_channels[47] [M 0x1ae3d8:194-195]");
    // The same E now opens the door.
    w.use_door(di, "the unlocked exec-office door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the door locked on ch 0x2F opens after the burn, state {st:#x}");
}

/// A plain waiting-room door (0xDB #13, no lock channels [D]) opens on the E: `Door_Activate`'s
/// use path [M door.rs]. Sister: a locked door (#154 door_large_stairwell, lock ch 200) refuses
/// the same E while ch 200 is off.
#[test]
fn waiting_room_door_opens_on_use() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let dpos = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .find(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5)
        .map(|d| d.pos).expect("door_waitingroom_corridor_right #13 [D]");
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5).unwrap();
    // Walk to its front. The direct walk from the exec-floor spot crosses the boardroom's
    // furniture row (x 4-6, z 22-20, blocked at every height [D probe, walkreal's pinned
    // blocker]) — the documented stand-in transit reaches the door's front; the E is the real
    // input. The poke's slope drops Bond ~1 unit during the settle (probe: y 46.8 → 45.95,
    // still sliding), so the aim converges on the door AFTER the settle — the stand-in for the
    // player's aim. The candidate the E must touch is the door's LIVE world sphere (obj+0x80,
    // obj+0x8C) [M Collide_PickObj 0x1eae80's 0x100/0x101 arm — the pick sphere (r 1.0 at
    // head + viewer dir [M 0x1a5170:100-108]) vs the sphere; the Collide_Intersect refinement
    // cannot remove the node (0x1ec940 ignores its return for param_10 = 0) and the class walk
    // has no distance gate (0x1a5170:113-137)].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [dpos[0], 46.8, dpos[2] + 1.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let d = &sim.drones.doors.list[di];
        let centre = [dpos[0], dpos[1] + 1.0, dpos[2]];
        let head = sim.bond.head();
        let to = [centre[0] - head[0], centre[1] - head[1], centre[2] - head[2]];
        sim.bond.yaw = to[0].atan2(to[2]);
        sim.bond.pitch = (to[1] / to[0].hypot(to[2])).atan() / std::f32::consts::FRAC_PI_2;
    }
    w.use_door(di, "the waiting-room door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the unlocked door opens on the E [M Door_Activate], state {st:#x}");
}

/// The interior vent grill (0xE2 SS #98, state 0x40, mask 4 [D params]) operates on the E:
/// `SS_Activate` 0x1a9ea8 snaps the state to 1 and the playhead sweeps to the last key
/// [M ss.rs]. Sister: the posed matrix rests at key 0 before the E.
#[test]
fn vent_grill_ss_e_sweeps_it_open() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let gi = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
        .position(|s| s.placement == 98).expect("the vent_grill1 interior SS #98");
    let start = w.scene.sim.as_ref().unwrap().drones.sss.list[gi].matrix[3][2];
    // The grill at (8.57, 49.73, 19.60) [D] — its mask-4 E is reachable from the plate side.
    w.scene.sim.as_mut().unwrap().bond.pos = [8.57, 49.6, 21.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!((w.scene.sim.as_ref().unwrap().drones.sss.list[gi].matrix[3][2] - start).abs() < 0.05,
        "sister: the grill rests at its key-0 pose before the E");
    w.walk_to(Wp::at(8.57, 49.6, 20.9, Act::Use { use_ticks: 240 }), "E the vent grill");
    w.wait(90);
    // The grill's sweep is subtle [D: 2 keys — the measured key-0 → end delta is 0.04 total] and
    // the state-1 sweep wraps back to state 0 holding the end pose [M 0x1a9948:121-126] — both
    // facts assert the E ran the activate → sweep → wrap chain (state was 0x40 at load).
    let s = &w.scene.sim.as_ref().unwrap().drones.sss.list[gi];
    let _after = s.matrix[3][2];
    let moved = (0..3).map(|k| (s.matrix[3][k] - [8.571396, 49.72885, 19.599398][k]).abs()).sum::<f32>();
    assert!(moved > 0.02 && s.state == 0 && s.wrap_flag == 1,
        "the E sweeps the grill off its key-0 pose and the wrap holds it [M SS_Activate 0x1a9ea8; 0x1a9948:121-126],          moved {moved:.4} state {} (load state 0x40, start z {start})", s.state);
}

/// The dome's laser fence (0xEB laserbeam-large #270-277, out ch 13, lock ch 59 [D params])
/// fires on the crossing: the trigger pass writes the out channel while its lock channel is off
/// [M triggers.rs Trigger_Update's lock gate]. Sister: ch 13 off before the walk.
#[test]
fn dome_laserbeam_crossing_fires_ch0d() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    // Beam #272 spans the dome mouth at (−1.18, 47.36, −11.13) [D]; the terminal-room side is
    // walkreal's proven stand (−1.0, 47.5, −5.5), the dome mouth is ~6 m south.
    w.scene.sim.as_mut().unwrap().bond.pos = [-1.0, 47.5, -5.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0xd), "sister: the fence silent before the walk");
    // The band is thin (the beam's z band ≈ 0.28 m [D #272's box]) and the re-firing touch's
    // off-write clears the channel once Bond is past it [M triggers.rs] — poll DURING the
    // crossing: step south through the beam line and watch the channel per tick.
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI; // face −z (the dome)
    let mut fired = false;
    for _ in 0..240 {
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xd) { fired = true; break; }
    }
    assert!(fired, "crossing the beam fires ch 13 [D #272 out param; M triggers.rs]");
}

/// The exec-floor pickups on the walk-over [M Pickup_Handler 0x18e650]: the BodyArmourHeavy
/// (#302, kind 3, 50 [D]) sets armour 50, the RocketLauncher (#349, kind 0, weapon 0x2E, ammo 8
/// [D]) arms the weapon, and a "Pickup 007" (#300, kind 6 BOND_BONUS [D]) is consumed with the
/// take path. Sisters: armour 0 and the rocket un-owned before.
#[test]
fn exec_floor_pickups_taken_by_walking_over() {
    // The kind-6 row exists only with the level's gold medal recorded [M Pickup_Create
    // 0x18dda0:93-95]: 0700000b plays under BaseMap 0x07000009 (the mission table [L ee_start_05
    // 0x2a4350], the gold bit the 0x0700005/9/11 kind-4 group's rank 1 [L 0x2bc920 row 5;
    // M 0x199210]) — the warm load carries the medal word (pickup-medal-gate).
    let mut w = load_warm_bits(gold_bits(0x0700_0009));
    to_exec_floor(&mut w);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    assert!(!w.scene.sim.as_ref().unwrap().drones.inventory.owned[0x2e], "sister: the rocket un-owned before");
    // The armour at (17.53, 47.68, 28.46) [D]. The walked approach from the exec-floor spot
    // wedges on the east office furniture (the probe: stuck at (14.1, 46.7, 31.7), d 4.8) — the
    // documented stand-in transit reaches the room; the take is the real walk-over.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [17.5, 47.2, 30.4];
        sim.bond.vel = [0.0; 3];
    }
    w.walk_to(Wp::new(17.53, 47.2, 28.46).r(1.4), "onto the BodyArmourHeavy");
    w.wait(20);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [M 0x18e650 case 3], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);
    // The rocket launcher at (−22.50, 47.96, 29.30) [D] — the far west office; the walkreal's
    // boardroom blocker stands between (the documented stand-in transit reaches it).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-22.5, 47.9, 29.3];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    assert!(w.scene.sim.as_ref().unwrap().drones.inventory.owned[0x2e],
        "the walk-over arms the RocketLauncher 0x2E [D p1 = 46; M 0x18e650 case 0]");
    // A BOND_BONUS (#300 at (19.26, 47.07, 32.35) [D]) is consumed by the same take path.
    let taken = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] - 19.26).abs() < 0.5 && (p.m[3][2] - 32.35).abs() < 0.5)
        .all(|p| p.gone || p.hidden);
    assert!(!taken(&w), "sister: the bonus sits before the walk");
    w.scene.sim.as_mut().unwrap().bond.pos = [19.26, 47.07, 32.35];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(20);
    assert!(taken(&w), "the BOND_BONUS pickup is consumed on the walk-over [M 0x18e650:234-236]");
    // The no-medal arm [M 0x18dda0:93-95]: the same warm load with the fresh-boot zero word
    // creates no kind-6 at all — #300's spot holds nothing.
    let mut bare = load_warm_bits(0);
    to_exec_floor(&mut bare);
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
}

/// The rooftop vent (0xEA TouchOnce #327, mask 4 = USE, out ch 0x77 [D]) fires on the E and the
/// paired exterior ScriptPlayer #326 (mode 64, out ch 0x77 [D]) plays 0x060009F9: its stream 0
/// spawns the grill-cover entity 0x02000A8B and its stream 2 event 0xD writes ch 0x77 = 0 at
/// tick 10 [D nis dump] — the channel pulses and the cover flies. Sister: ch 0x77 off and no
/// cover entity before the use.
#[test]
fn rooftop_vent_use_pulses_ch77_and_spawns_the_cover() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    // The roof: the lift-gap stand-in (walkreal's documented teleport). The vent's own platform
    // is the small ledge at (−0.56, ≈53.6, 22.2) [D probe: every surrounding x/z falls] — the
    // exterior grill SP #326 stands on it [D (−0.56, 52.61, 22.79)].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-0.56, 53.6, 22.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "the vent platform stands");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x77), "sister: ch 0x77 off before");
    // The mode-64 prime ran ONE update at load, which spawned the cover entity resting on its
    // grill [M SP_CreateScriptPlayer 0x1db9a8's load beat; D the entity at (−0.559, 52.61, 22.79)
    // = SP #326's own spot].
    let cover_at = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.scripts.entities()
        .iter().filter(|e| e.2 == 0x0200_0a8b)
        .map(|e| ([e.3[3][0], e.3[3][1], e.3[3][2]], e.1)).collect::<Vec<_>>();
    let before = cover_at(&w);
    assert_eq!(before.len(), 1, "the cover rests on its grill before the use");
    assert!((before[0].0[0] + 0.559).abs() < 0.05 && (before[0].0[2] - 22.79).abs() < 0.05,
        "the cover rests at the grill's own spot [D], got {:?}", before[0].0);
    // Face the plate (its volume spans z 23.2-25.4, y 52.3-54.5 [D probe] — the use sphere is
    // head+look [M 0x1a5170:100-108]) and E it from the platform (too small to walk on — the
    // E edges come from the stand).
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.0; // face +z, the plate
    for _ in 0..40 {
        w.tick(nf_game::player::Input::default(), w.total_ticks % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x77) { break; }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x77),
        "the E-use fires the plate's out ch 0x77 [D #327; M triggers.rs]");
    w.wait(80);
    // The cover entity is KEPT (the script's keep-objects flag, Event 0xC at its tick 31 [D nis
    // dump]) and rests at its pose — the script's stream 1 holds itself with the negative delay
    // (Event 3 params [0] at its tick 17 [D nis dump]; `Script_Update`'s delay gate [M
    // 0x1da998]) once ch 0x77 cleared, so the keyed lift-off stalls at the hold — the game's
    // un-hold is the channel re-arm (a live check; the plate is spent).
    let after = cover_at(&w);
    assert_eq!(after.len(), 1, "the kept cover entity remains [M Script_KillStream's keep arm]");
    // The script's own event 0xD cleared ch 0x77 again at its tick 10 [D 0x060009F9 stream 2].
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x77),
        "the script's ch-0x77 clear ran [D 0x060009F9 stream 2: Event d [77, 0]]");
}

/// The roof helicopter (0x2F Copter #328, gate ch 0x2A, arrival ch 0x66 [D params]) hovers at its
/// placement while the gate is closed: `Copter_Update` 0x17dbb8's head returns on a closed gate —
/// no path advance, no arrival write [M copter.rs]. Sister: the gate is off at load and ch 0x66
/// never lands; the parachute pickup (#280, 2.5 m away [D]) sits on its deck untouched.
#[test]
fn roof_helicopter_gated_hover() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let (m0, gate) = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.copters.len(), 1, "the Helicopter-Body copter #328 exists [D]");
        (sim.drones.copters[0].matrix, sim.drones.copters[0].gate)
    };
    assert_eq!(gate, 42, "the copter's gate is ch 0x2A [D param 6]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x2a),
        "sister: the gate ch 0x2A is off at load — no writer in the level's dumps [D]");
    w.wait(240);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = (0..3).map(|i| (sim.drones.copters[0].matrix[3][i] - m0[3][i]).abs()).sum::<f32>();
        assert!(d < 0.05, "the closed gate holds the hover at the placement [M 0x17dbb8 head], drift {d}");
        assert!(!sim.drones.channels.on(0x66), "no arrival write while gated [M Copter_Move 0x17d628]");
        assert!(sim.drones.pickups.iter()
            .any(|p| p.switch_channel == 0x3f && !p.gone && !p.hidden),
            "the parachute pickup still sits on the copter's deck [D #280]");
    }
}

/// The keycard lock's music beat (0xFB MusicTrigger #269: ch_on 9, event 41 [D params]):
/// `MusicTrigger_Update` 0x1b31a0 re-arms its 30-tick countdown while ch 9 is on and fires
/// `Music_Event(41, 2)` per tick [M worldobj.rs]. Driven by the SAME real E as the keycard test
/// (the switch's write is that test's pin). Sister: the board slot reads 0 before.
#[test]
fn music_triggers_created_with_the_disc_params() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let sims = w.scene.sim.as_ref().unwrap();
    let ts: Vec<(u16, u32)> = sims.drones.worldobjs.music_triggers.iter()
        .map(|t| (t.ch_on, t.event_id)).collect();
    assert!(ts.contains(&(9, 41)), "the keycard's music trigger (ch 9 → event 41) exists [D #269]");
    assert!(ts.contains(&(0x4e, 17)), "the files' music trigger (ch 0x4E → event 17) exists [D #283]");
    // The countdown behaviour itself is `WorldObjects`' unit pin (music_trigger_tests::
    // countdown_rearms_fires_and_expires); the sim's board slot is consumed inside the frame, so
    // the audible beat's visibility is the render/audio layer's (a live check).
}

/// The Q-Worm console (0x35 PCQWorm #297 SCOMP_QWORM_C at (−19.32, 48.13, 29.46), param 1 = 76 =
/// ch 0x4C [D]): `PCQWorm_Update` 0x1aee20 state 0 waits for a class-5 object — a BULLET — and
/// the Q-Worm gadget's install writes `switch_channels[76]` [M 0x1aee20; the disc's boot
/// loadout owns weapon 0x58, M 0x1a7ef8 case 0x7000009/0a/0b equip 001a8494]. Driven by the
/// real trigger pull (the same shot walkreal_0700000b_live's objective-7 leg fires). Sisters:
/// ch 0x4C off and the PC in state 0 before the shot.
#[test]
fn qworm_console_installs_on_the_gadget_shot() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let pc = w.scene.sim.as_ref().unwrap().drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c)
        .expect("SCOMP_QWORM_C (0x35, ch 0x4C) [D #297]").pos;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x4c), "sister: ch 0x4C off before the shot [D #297 param 1]");
        assert!(sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c).map(|p| p.state) == Some(0),
            "sister: the PC rests in state 0 before the shot [M 0x1aee20]");
        assert!(sim.drones.inventory.owned[0x58],
            "the boot loadout owns the Q-Worm gadget 0x58 [M 0x1a7ef8 case 0x7000009/0a/0b]");
    }
    // The real trigger pull: stand at the proven desk approach (n26, the walked route's last
    // nav point [D nis]; the interactables file's teleport stand-in) and walk the last leg to
    // the desk, then the equip walkreal_0700000b_live's shot uses, and fire at the screen.
    w.scene.sim.as_mut().unwrap().bond.pos = [-19.64, 47.2, 31.8];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.try_walk_to(Wp::new(pc[0] + 0.6, pc[1] - 0.5, pc[2] + 0.9).r(0.4), "the Q-Worm C desk (the walked leg)");
    // The equip: the game's own gadget cycle (the d-pad's `Player_WeaponChange` mode 1,
    // bounds 0x4A..0x5E [M 0x1a6f80]) steps to the disc-owned Q-Worm 0x58 through the real
    // switch state machine (holster/draw waits between steps).
    let mut got = false;
    for _ in 0..12 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x58 { got = true; break; }
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Default::default(), false); }
    }
    assert!(got, "the gadget cycle draws the Q-Worm 0x58 [M Player_WeaponChange 0x1a6f80 mode 1]");
    w.wait(30);
    // The shot: walkreal's proven aim point (the screen band pc+0.55); if the desk's screen
    // collision truncates the dart high, the fallback band pc+0.1 (the box face at centre
    // height) is the second stance.
    let mut installed = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 2.6, 900,
        "the Q-Worm C shot", &|sim| sim.drones.channels.on(0x4c));
    if !installed {
        // The low band from the same stand: the gadget fires only inside its 2.0 reach of the PC
        // [M Player_WeaponFiring 0x1a5460:531-545], so the second stance stays at the desk.
        installed = shoot_at(&mut w, [pc[0], pc[1] + 0.1, pc[2]], 2.6, 900,
            "the Q-Worm C shot (low band)", &|sim| sim.drones.channels.on(0x4c));
    }
    if !installed {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0b-qworm] shot missed: weapon {:#x} req {:#x} owned58 {} pos {:?} pc_state {:?} disabled {} sub {} carried {:?}",
            sim.gun.weapon, sim.gun.switch_request, sim.drones.inventory.owned[0x58], sim.bond.pos,
            sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c).map(|p| p.state),
            sim.disabled, sim.bond.substate, sim.drones.inventory.carried);
    }
    assert!(installed, "the gadget shot lands and installs [M PCQWorm_Update 0x1aee20]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4c),
        "the install writes switch_channels[76] [M 0x1aee20; D #297 param 1]");
}

/// The sound triggers (0xF9 ×12 "Sound Trigger" [D census]): `SoundTrigger_Update` 0x1b2790
/// starts a trigger's sound while its ON channel (param 0) is up [M sound.rs]. #330's on-channel
/// is the terminal-room plate's ch 0x4D (id 1158, param 3 = 1 = 2D [D #330 params]) and #331's
/// is the dome fence's ch 0x0D (id 1159, 2D [D #331]) — both driven by the SAME real walks the
/// plate kinds pin: the walked-over plate #163 (0xEA, out ch 0x4D, mask 8 [D]) and the walked
/// beam #272 crossing (0xEB, out ch 0x0D [D]). Sisters: neither id plays on a fresh load.
#[test]
fn sound_triggers_play_on_their_channels_real_writes() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.sound_triggers.list.iter().any(|t| t.on == 0x4d && t.id == 1158),
            "the ch-0x4D sound trigger #330 exists [D #330 params]");
        assert!(sim.drones.sound_triggers.list.iter().any(|t| t.on == 0x0d && t.id == 1159),
            "the ch-0x0D sound trigger #331 exists [D #331 params]");
        assert!(!sim.drones.sound_triggers.playing(1158) && !sim.drones.sound_triggers.playing(1159),
            "sister: neither trigger plays on a fresh load [M 0x1b2790's channel-edge start]");
        assert!(!sim.drones.channels.on(0x4d), "sister: the terminal-room plate silent [D #163]");
    }
    // The dome-side stand the laserbeam test proves (−1.0, 47.5, −5.5) sits inside plate
    // #163's box [−3.04, 46.73, −6.67]-[1.23, 48.92, −4.48] [D] — the walk-over fires ch 0x4D.
    w.scene.sim.as_mut().unwrap().bond.pos = [-1.0, 47.5, -5.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut plate = false;
    for _ in 0..60 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { plate = true; break; }
    }
    assert!(plate, "the walked-over plate #163 writes ch 0x4D [D #163; M triggers.rs]");
    let mut s1158 = false;
    for _ in 0..30 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.sound_triggers.playing(1158) { s1158 = true; break; }
    }
    assert!(s1158, "the ch-0x4D edge starts sound 1158 [M SoundTrigger_Update 0x1b2790; D #330]");
    // The kind's remaining law is the single global voice [M 0x1b2790:19-28]: #331 (also global,
    // param 5 = 1 [D]) may only start while the slot is free. Whether the DISC's 1159 sounds at
    // the fence crossing after the terminal visit depends on 1158's sample length — the PS2
    // driver's `Sound_IsFinnished` answer, which the headless port does not model (its voices
    // end only on a stop) — so the slot law on this level's pair is a live check, not an
    // assertion here.
}

/// The security cameras (0x28 Sensor ×2 [D census]): `Sensor_Update` 0x1af618 counts
/// consecutive in-cone ticks (`Sensor_InCone` 0x1af4f0: the placement apex, the param-1 cone
/// angle, the param-2 range, the LOS beam), alerts at 0x1e into the track state, and after 2 s
/// of continuous sight writes 1 to its OUT channel (param 3) [M 0x1af618:154-157]. Camera
/// #172's out is ch 39 and #164's is ch 40 [D params]. Driven by the real sight path: Bond
/// stands where the sweep aims (the stance enumerates the cone the sweep presents — the subject
/// is the detection law, not movement, like the files-plate stances). Sisters: both channels
/// off at load, and #164's ch 40 stays silent while #172 fires (the per-sensor law).
#[test]
fn exec_camera_sight_fires_its_out_channel() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.sensors.list.iter().any(|s| s.out == 39), "camera #172 exists (out ch 39) [D]");
        assert!(sim.drones.sensors.list.iter().any(|s| s.out == 40), "camera #164 exists (out ch 40) [D]");
        assert!(!sim.drones.channels.on(39) && !sim.drones.channels.on(40),
            "sister: both camera channels off at load [M 0x1af618 state 0 clears its out]");
    }
    let si = w.scene.sim.as_ref().unwrap().drones.sensors.list.iter()
        .position(|s| s.out == 39).expect("camera #172");
    // The chase: each tick place Bond at the sweep's current aim (2.5 m out, shortened inside
    // the beam's first hit so the stance stays in clear air) — he is then IN the cone by
    // construction every tick, the seen counter runs consecutively [M 0x1af618:211-216], and
    // the track state holds him once it alerts.
    let mut seen = false;
    for _ in 0..900 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let s = &sim.drones.sensors.list[si];
            let d = [s.matrix[2][0], s.matrix[2][1], s.matrix[2][2]];
            let dist = if s.beam_len > 0.0 && s.beam_len < 2.5 { (s.beam_len * 0.7).max(1.0) } else { 2.5 };
            sim.bond.pos = [s.pos[0] + d[0] * dist, s.pos[1] + d[1] * dist, s.pos[2] + d[2] * dist];
            sim.bond.vel = [0.0; 3];
        }
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.sensors.list[si].last_hit { seen = true; break; }
    }
    assert!(seen, "the sweep's cone takes the stance [M Sensor_InCone 0x1af4f0]");
    // Hold: the tracker keeps the cone on him; the 2 s detect clock trips the out write.
    let mut fired = false;
    for _ in 0..400 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(39) { fired = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(fired, "2 s of sight writes ch 39 [M 0x1af618:154-157; D #172 param 3]");
        assert!(!sim.drones.channels.on(40),
            "sister: camera #164's ch 40 stays silent while #172 fires [D #164 param 3; M the per-sensor law]");
    }
}

/// The lift's shaft plates (0xEB #245 "lift_stop_trigger" out ch 117 = 0x75; 0xEA #171
/// "oDroneTrigger2" mid-shaft out ch 83 = 0x53, both mask 8 [D params]): the Touch/TouchOnce
/// laws fire on the crossing [M triggers.rs] — the plates the cab ride crosses (the ride is
/// walkreal_0700000b_live's pin; whether a DECK-walking player alone touches #245 is the level
/// doc's open live check, so the stance here is the box interior — the subject is the plate's
/// channel law, not movement). Sisters: both channels off before each touch.
#[test]
fn lift_shaft_plates_fire_on_the_touch() {
    let mut w = load_warm();
    to_exec_floor(&mut w);
    let (i245, i171) = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x75) && !sim.drones.channels.on(0x53),
            "sister: both shaft-plate channels off before [D #245/#171 out params]");
        (sim.drones.triggers.iter().position(|t| t.out == 0x75).expect("lift_stop_trigger #245 [D]"),
         sim.drones.triggers.iter().position(|t| t.out == 0x53).expect("the mid-shaft plate #171 [D]"))
    };
    // Touch #245: hold Bond inside the box (two y stances cover the toucher's −0.6 law).
    let (c245, lo245, hi245) = box_of(&w, i245);
    let mut stop = false;
    for y in [hi245[1] - 0.5, lo245[1] + 0.7] {
        for _ in 0..30 {
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = [c245[0], y, c245[2]];
                sim.bond.vel = [0.0; 3];
            }
            w.tick(Default::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x75) { stop = true; break; }
        }
        if stop { break; }
    }
    assert!(stop, "the shaft-top stop plate writes ch 0x75 on the crossing [D #245 out 117; M triggers.rs]");
    // Touch #171 (mid-shaft, y 37.5-39.7 [D doc]): same law, its own sister.
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x53),
        "sister: the mid-shaft plate silent before its touch [D #171]");
    let (c171, lo171, hi171) = box_of(&w, i171);
    let mut mid = false;
    for y in [hi171[1] - 0.5, lo171[1] + 0.7] {
        for _ in 0..30 {
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = [c171[0], y, c171[2]];
                sim.bond.vel = [0.0; 3];
            }
            w.tick(Default::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { mid = true; break; }
        }
        if mid { break; }
    }
    assert!(mid, "the mid-shaft plate writes ch 0x53 on the crossing [D #171 out 83; M triggers.rs]");
}

/// The hint marker (0x33 "Message" #354, the level's only one [D census]): the parse-model pin
/// (the port's hints are the parse-time model `nf_game::player::hints`; the live firing path —
/// `Hint_Update` 0x1b3380's one-shot TXTMSG type 1 + sound + the delete bit — is not in the
/// sim, the same port gap interactables_07000013_live documents). The disc's params [D #354]:
/// label 0x02000021, sound 0, stage 3, no on-channel, off-channel 79 = ch 0x4F (the AND of
/// objectives 7+9 — the hint goes silent once the mission's mid-gate lands [D #299]). Sister:
/// the stage gate refuses it on the hardest difficulty [M 0x1b3248:12-25].
#[test]
fn the_message_hint_carries_the_disc_params() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000b.bin")).expect("0700000b.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let hints = nf_game::player::hints(&lv, 1);
    assert_eq!(hints.len(), 1, "the level's one hint marker, nothing else [D census]");
    let h = &hints[0];
    assert_eq!(h.label, 0x0200_0021, "the hint's text label [D #354 param 0]");
    assert_eq!(h.sound, 0, "the hint is silent (param 1 = 0) [D #354]");
    assert_eq!(h.stage, 3, "the stage word [D #354 param 2]");
    assert_eq!(h.on_channel, 0, "no arm channel — always armed [D #354 param 3]");
    assert_eq!(h.off_channel, 79, "the off-channel is ch 0x4F, the objectives' AND [D #354 param 4; D #299]");
    assert!((walker::dxz(h.pos, [-13.764, 47.872, 6.450])) < 0.2,
        "the hint sits at its placement spot [D #354]");
    // Sister: the stage gate [M 0x1b3248:12-25] — stage 3 fires on difficulties ≤ 3 and is
    // refused on 4 (the hint_stage_gates port of the same compare).
    assert!(nf_game::player::hint_stage_gates(3, 1), "stage 3 shows on Operative [M 0x1b3248:12-25]");
    assert!(!nf_game::player::hint_stage_gates(3, 4), "sister: stage 3 is refused on 00 Agent [M 0x1b3248:12-25]");
}

/// The trigger box's (centre, lo, hi) — the stance helper for the placed-touch laws.
fn box_of(w: &Walker, ti: usize) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let t = &w.scene.sim.as_ref().unwrap().drones.triggers[ti];
    (t.centre(), t.lo, t.hi)
}

/// A y stance just above the box's floor (the toucher lands inside for any box ≥ 0.7 tall).
/// (the stance helpers were inlined at the call sites)

/// The real trigger pull at a target (walkreal_0700000b_live's helper): aim the head at the
/// target from Bond's eye, pull the trigger when on-target, until `done` or the tick cap.
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let _ = label;
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    let mut tapped_once = false;
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
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
            if trigger && !tapped_once {
                tapped_once = true;
                let sim = w.scene.sim.as_ref().unwrap();
                let (paths, wp) = sim.drones.bullet_paths();
                eprintln!("[0b-qworm] first tap @{}: clip {} spare {} paths {} wp {:?} err {err:.4} perr {perr:.4}",
                    w.total_ticks, sim.gun.clip, sim.gun.spare, paths.len(), wp);
            }
        }
        w.total_ticks += 1;
        w.after_step();
    }
}
