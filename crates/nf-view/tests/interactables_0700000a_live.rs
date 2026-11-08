//! interactables-0a: one live test per KIND of player-touchable placement on 0700000a (Tower B,
//! the office floor), driven the way a player drives it (the walk, the use key, the trigger pull,
//! the fall), each asserting the game's effect with its sister. The full census with the type
//! table lives in docs/levels/0700000a.md; the end-to-end route is
//! walkreal_0700000a_live.rs.
//!
//! The kind set (the census [D every placement kind] + the 0x1d04f0 dispatch [M]): the SS lift
//! doors (0xE2), the pickups (0xF0), the breakables (0x20), the laser Touch plates (0xEB), the
//! alarm switches (0x29), the ladder (0x22), the Hurt shaft volumes (0xE4), the TouchOnce plates
//! (0xEA), the doors (0xDB), the keypad locks (0x2E), the Q-Worm PC kind (0x35), the keyboard
//! ScriptPlayer (0xD9) and the hatch LOAD_LEVEL exit (0xE8). Not player-touchable on this level's
//! data: the 0x31 fuseboxes (script-hash-0 shells, inert art [M 0x1dc7e0:69-71]), the 0x30
//! monitor (script-keyed at the origin — no live use target [D]), the 0x71 rotor and 0xFA icon
//! (presentation).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only where walking there is a separate documented blocker; the
//! operation itself always runs through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a"))
}

/// The warm load the real run hands this level (the keyboard test's carry: the RamSave round-trip
/// [M 0x195670:13]; the mode-64 SP preload and the hatch chain run on the game's own warm path),
/// with the game's own boot KeyCodes over the port's local-stream default [L ee_start_0700000a.bin
/// @0x2beff8 — KeyCodes, 0x33·5 bytes; the port's Locks_Init stand-in generates on a local stream,
/// so the keypad tests type the snapshot's digits — walkreal_0700000a_live's load_carry override].
fn load_warm_snap() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load 0700000a cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load 0700000a warm"));
    if let Ok(snap) = std::fs::read("../../work/ee_start_0700000a.bin") {
        let mut rows = [[0u8; 5]; 0x33];
        for (r, row) in rows.iter_mut().enumerate() {
            let off = 0x2beff8 + r * 5;
            if off + 5 <= snap.len() { row.copy_from_slice(&snap[off..off + 5]); }
        }
        w.scene.sim.as_mut().unwrap().drones.key_codes = nf_game::lock::KeyCodes(rows);
    }
    w
}

/// Teleport Bond to a spot and settle him standing (the documented stand-in transit — every
/// operation under test runs through the real input after it).
fn teleport(w: &mut Walker, pos: [f32; 3]) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = pos;
        sim.bond.vel = [0.0; 3];
    }
    w.wait(12);
}

/// Aim the view at a world point and pulse the use key until `ready` (the keyboard test's
/// activation loop, compacted): each cycle turns onto the point and presses E once it is aimed.
fn use_toward(w: &mut Walker, target: [f32; 3], ready: &dyn Fn(&nf_view::scene::Sim) -> bool, cycles: u32) -> bool {
    for _ in 0..cycles {
        for t in 0..40u32 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if ready(sim) { return true; }
                let bp = sim.bond.pos;
                let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                let dx = target[0] - bp[0];
                let dz = target[2] - bp[2];
                let dy = target[1] - eye;
                let desired = dx.atan2(dz);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let perr = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
                let input = nf_game::player::Input {
                    turn: err.clamp(-0.05, 0.05),
                    look: perr.clamp(-0.05, 0.05),
                    ..Default::default()
                };
                let hold = t == 15 && err.abs() < 0.2;
                w.scene.sim.as_mut().unwrap().step_bond(input, false, false, hold);
            }
            w.total_ticks += 1;
        }
    }
    false
}

/// Fire the held weapon at a static point through the real trigger path (walkreal-b2's shoot_at,
/// aimed at a world point): turn+look onto the point, pulse the trigger while in range.
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
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
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            if dist > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
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

/// The cab-mouth SS lift doors (placements 524/525) sweep OPEN at create and hold the open pose:
/// their state param is 1 (armed) with a 5-key list and wrap flag 1 [D placements dump] —
/// `SS_Update` 0x1a9948's state-1 sweep runs from load, and the wrap (`(int)last < (int)t`,
/// wrap_flag != 0 → state 0) holds the last posed matrix [M 0x1a9948:119-128; grates-02's state
/// model]. The old ignore's premise ("a writer the port lacks") predates grates-02: the sweep IS
/// the writer, and the [L] start snapshot caught t = 0 (key 0, closed) before the first update.
#[test]
fn ss_lift_doors_sweep_open_at_create() {
    let mut w = load();
    // The doors' key-0 (closed) matrices [L ee_start_0700000a.bin: leaves at (−2.973, 1.303, 13.031)
    // and (−4.893, 1.275, 13.031) — the placement transforms [D]].
    let leaf = |w: &Walker, placement: usize| {
        w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .find(|s| s.placement == placement)
            .map(|s| s.matrix[3][0])
            .expect("the cab-mouth SS leaf")
    };
    let l2_start = leaf(&w, 524);
    let l1_start = leaf(&w, 525);
    assert!((l2_start + 2.9727).abs() < 0.05 && (l1_start + 4.8933).abs() < 0.05,
        "the leaves rest at their key-0 closed pose at load [D], got {l2_start} / {l1_start}");
    // The sweep is 75 ticks (rate word 0 → SS_Create's default 75 [M 0x1a9c58:51]); step past it.
    w.wait(100);
    let l2_open = leaf(&w, 524);
    let l1_open = leaf(&w, 525);
    assert!((l2_open - l2_start).abs() > 0.5 && (l1_open - l1_start).abs() > 0.5,
        "both leaves swept aside within the 75-tick create sweep and HOLD the open pose \
         [M SS_Update 0x1a9948 states + the wrap], got {l2_start}→{l2_open} / {l1_start}→{l1_open}");
    // Sister: without the state-1 create arm the leaves would still stand at key 0 — pinned by
    // the pre-wait read above (the pose at tick 0 IS key 0, closed [L]).
}

/// The BodyArmourHeavy pickup (#730, param 0 = 3, param 2 = 50 [D]) grants armour 50 on the
/// walk-over: `Pickup_Handler` 0x18e650's case 3 hard-sets 50.0 when the armour is below it
/// [M pickup.rs take()]. Sister: the armour reads 0 before the touch.
#[test]
fn armour_pickup_taken_by_walking_over_it() {
    let mut w = load();
    assert!(w.scene.sim.as_ref().unwrap().drones.health.armour == 0.0,
        "Bond spawns without armour");
    // #730 sits on the office floor at (−13.46, 0.90, 7.61) [D]; out of the cab, then the AI
    // ring's west-office legs — the anchors walkreal-0a's ring walks green on this branch (the
    // west_doorway_repro legs). The graph's own route to the armour room turns north off the
    // z≈5.5 corridor at its node 8, and the steering wedges against the passage wall there:
    // under the game-speed rain's stream the wedge parks Bond inside the west office's ch-0x15
    // floor plates (#11/#22, x −24.27..−19.96, z 8.89..14.45 [D trigger dump]) for ~1000 ticks —
    // long enough for the mission's LevelEnd to fire and the sim to request the level reload
    // mid-route [M Mission case 4 → ResetMap_LevelToLoad(LevelToEndTo) 0x182c60:73; the plates'
    // brief crossings are harmless — the ring repro walks straight over them]. The player's line
    // is the ring's own: down the west office block, east along the z≈5.5 corridor, then
    // north-east onto the armour — every leg wall-clear at body height [probe leg rays].
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 16.0).r(1.2), "the west hall");
    for (to, label) in [
        ([-20.5, 1.3, 16.5], "the west offices' north corridor"),
        ([-20.5, 1.3, 5.5], "down the west office block"),
        ([-17.5, 1.3, 5.5], "east of the west door"),
        ([-16.2, 1.3, 5.8], "the ai node 27 east"),
        ([-15.0, 1.3, 5.7], "the ai node 26 east"),
    ] {
        w.walk_to(Wp::new(to[0], to[1], to[2]).r(1.2), label);
    }
    w.walk_to(Wp::new(-13.46, 0.4, 7.61).r(0.9), "onto the BodyArmourHeavy");
    w.wait(20);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over take grants armour 50 [M 0x18e650 case 3; D param 2 = 50], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);
    // Sister: the second armour copy (#731, 17.65, 0.50, −7.83) still sits untouched — the take
    // consumed #730, not everything.
    let taken = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] + 13.46).abs() < 0.3 && (p.m[3][2] - 7.61).abs() < 0.3)
        .all(|p| p.gone || p.hidden);
    assert!(taken, "the walked-over pickup is consumed (gone, or hidden for the respawn) [M 0x18e650's respawn tail]");
}

/// The vending-machine glass (#379, 0x20 Break_Create at (6.12, 1.22, 16.35) [D]) shatters under
/// the held fire — and the count is the decompile's arithmetic, pinned end to end:
/// - health = `Break_Create`'s (float)(int)param 1 = 9.0 [M 0x12c2b4-0x12c2bc `cvt.s.w`; D param 1 = 9];
/// - the contact damage is the round's per-tick HITTEST+0x90 = weapon_data+0x0C [M Bullet_update
///   0x12e7b0 disasm 0x12f05c-0x12f060; the copy to the victim's obj+0xD0 node+8 at Collide_Update
///   0x1ea398 disasm 0x1ea690-98] = 0.1f for the held K5 dart 0x43 [L ee_snapshot weapon_data
///   0x2bf150 + 0x43·0x10C + 0x0C];
/// - ONE contact per dart: the collide list is sorted nearest-first and stops at the first solid
///   node, and the round dies on it [M Collide_Update 0x1ea398:216-229 + Bullet_CollisionHandler
///   0x12ce80 disasm 0x12cff0-4].
/// The K5 dart phase pins the 0.1f contact (one dart = one contact, health 9.0 → 8.9f); the kill
/// count rides the P2K (0x02, damage 3.5f [L weapon_data w2 +0x0C], the level's 7+41 rounds
/// [D inventory] — the K5's 5+10 darts cannot reach 9.0f): 9.0f − 3.5f is exact in f32, so the
/// kill lands on contact 3 — the op-for-op mirror of Break_Update's subtraction [M 0x12bfd8:37]
/// and its `< 0` kill [M :49].
/// Sister: the far glass #378 keeps health 9.0 and stays alive.
#[test]
fn vending_glass_breaks_under_fire() {
    let mut w = load();
    let br_of = |w: &Walker, placement: usize| {
        w.scene.sim.as_ref().unwrap().drones.breaks.list.iter()
            .find(|b| b.placement == placement).map(|b| (b.health, b.dead)).expect("the vending glass break")
    };
    // Walk to the lobby vending machine (right of the spawn, through the real path; the AI graph
    // carries the loop, the direct line pins on the lobby's north wall).
    let glass = [6.12, 1.22, 16.35];
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    assert!(w.follow_nav_route([4.6, 0.4, 15.2], "the lobby by the vending machine"), "no nav route");
    w.walk_to(Wp::new(4.2, 0.4, 16.35).r(0.9), "face the vending machine");
    let (h0, dead0) = br_of(&w, 379);
    assert!((h0 - 9.0).abs() < 1.0e-6 && !dead0,
        "Break_Create's health is (float)(int)param 1 = 9.0 [M 0x12c2b4-0x12c2bc; D param 1 = 9], got {h0}");
    // One dart, one 0.1f contact: fire until a dart actually leaves (shot_tick), then wait out
    // its flight and pin the drop. (The K5 carries 5+10 darts [D inventory] — enough for this
    // pin, not for a 91-dart kill; the kill count rides the P2K phase below.)
    let contact = 9.0f32 - 0.1f32;
    let mut fired = false;
    for _ in 0..600 {
        shoot_at(&mut w, glass, 4.5, 1, "the vending glass", &|_| false);
        if w.scene.sim.as_ref().unwrap().shot_tick { fired = true; break; }
    }
    assert!(fired, "the K5 never fired at the glass");
    w.wait(120);
    let (h1, dead1) = br_of(&w, 379);
    assert!(h1 == contact && !dead1,
        "one dart scores ONE contact of weapon+0x0C = 0.1f [M 0x12f05c-60 + 0x1ea690-98 + the \
         one-solid-hit stop 0x1ea398:216-229], got {h1} (want {contact}) dead {dead1}");
    // The kill count: cycle to the P2K (0x02, damage 3.5 [L weapon_data w2 +0x0C], 7+41 rounds
    // [D inventory]) through the real weapon wheel. 3.5f is exact in f32, so the count from the
    // dart-dented 8.9f mirrors Break_Update's arithmetic [M 0x12bfd8:37 (the f32 sub), :49 (< 0)]:
    // 5.4f, 1.9f, kill on the 3rd.
    let mut expect_health = h1;
    let mut expect_contacts = 0u32;
    while expect_health >= 0.0 {
        expect_health -= 3.5f32;
        expect_contacts += 1;
    }
    assert_eq!(expect_contacts, 3, "8.9f − 3.5f per contact kills on the 3rd [M]");
    let mut switched = false;
    for _ in 0..12 {
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x02 { switched = true; break; }
    }
    assert!(switched, "the P2K never came up on the wheel (real switch path)");
    let mut samples: Vec<f32> = vec![h1];
    let samples_cell = std::cell::RefCell::new(&mut samples);
    let landed = shoot_at(&mut w, glass, 4.5, 3000, "the vending glass", &|sim| {
        let br = sim.drones.breaks.list.iter().find(|b| b.placement == 379).unwrap();
        let mut s = samples_cell.borrow_mut();
        if s.last() != Some(&br.health) { s.push(br.health); }
        br.dead
    });
    assert!(landed, "the glass never broke under the real fire (gun {:#x})",
        w.scene.sim.as_ref().unwrap().gun.weapon);
    let (hk, deadk) = br_of(&w, 379);
    assert!(deadk && hk < 0.0, "Break_Kill's dead bit set [M 0x12b970]");
    // The observed contacts = the f32 arithmetic's count (each landed round scores exactly one
    // 3.5f contact; missed rounds change nothing; samples[0] is the dart-dented seed).
    assert_eq!(samples.len() as u32, expect_contacts + 1,
        "the contact sequence is the decompile's f32 accumulation [M 0x12bfd8:37,49], got {} samples: {:?}",
        samples.len(), samples);
    assert!((samples[expect_contacts as usize - 1] - (expect_health + 3.5f32)).abs() < 1.0e-6,
        "the last alive contact leaves 1.9f, got {}", samples[expect_contacts as usize - 1]);
    // Sister: the other vending machine (−33.0, 1.22, −18.08) is untouched.
    let (hs, deads) = br_of(&w, 378);
    assert!(!deads && (hs - 9.0).abs() < 1.0e-6, "the far glass stands at health 9.0");
}

/// Walking through the laser web fires the beam's out channel: #607 (out ch 0x1F, mask 8, lock
/// 0x37 [D triggers dump]) spans x ≈ −8.45, y 1.49-1.76, z 15-20.25 — Bond's capsule crosses it
/// at a walk. The lock DISABLES while its channel is on [M triggers.rs Trigger_Update's
/// `lock != 0 && ch.on(lock)` gate], so on fresh boot (ch 0x37 off [L ee_start_0700000a.bin
/// @0x26fc90]) the beams are live. Sister: ch 0x1F is off before the crossing.
#[test]
fn laser_web_crossing_fires_the_alert_channel() {
    let mut w = load();
    w.wait(30);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f), "the beam is silent before the walk");
    // Through the beam: the lobby south band then west across x −8.45 between z 15 and 20.
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 17.5).r(1.2), "through the laser wall");
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x1f),
        "crossing beam #607 fired its out channel ch 0x1F [D triggers dump]");
}

/// E on an alarm switch does nothing: `Switch_Activate` 0x1b02a8 refuses a class-3 (player)
/// activator on a ch-0x61 switch (`cVar1 != 3 || ch != 0x61` [M]) — the 8 alarmswitchplayer
/// placements (#617-631, p0 = 0x61 [D]) cannot be player-fired. The control arm proves the use
/// path itself is alive: the same E press opens the unlocked office door on the route.
#[test]
fn alarm_switch_refuses_the_player_use() {
    let mut w = load();
    // #619 stands at (−15.48, 1.33, −13.13) [D]; the AI graph's proven leg to the south-west
    // office (walkreal's route) carries the loop — the direct nav to the switch itself crosses
    // the node-15 wall stub [D probe, the walkreal's note].
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    assert!(w.follow_nav_route([-15.8, 0.4, -15.0], "the south-west corridor by alarm switch #619"),
        "no nav route to the alarm switch");
    w.wait(5);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x61), "the alarm is off before the use");
    // E the switch from its wedge front (the use sphere is 1.0 at head+look [M 0x1a5170]; the
    // switch stands 0.6 off the wedge — in range where the walk settles).
    w.walk_to(Wp::at(-15.65, 0.4, -13.6, Act::Use { use_ticks: 240 }), "E the alarm switch");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x61),
            "the player's E must not toggle ch 0x61 [M 0x1b02a8's class-3 exclusion]");
        // The switch's stored value is untouched (the toggle never ran).
        let sw = sim.drones.useobjs.switches.iter().find(|s| s.ch == 0x61).expect("the alarm switch");
        assert_eq!(sw.value, 0, "the stored value still reads the rest value [M 0x1b02a8's value gate]");
    }
    // Control arm: the use path IS alive — E the nearest unlocked door and watch it open.
    let (dpos, di) = walker::door_near(&w.scene.sim.as_ref().unwrap(), [-15.48, 1.1, -13.0])
        .expect("a door near the switch");
    w.use_door(di, "E the office door (the use path's control arm)");
    let open = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(open != 0, "the same use key opens a door (state {} at {dpos:?}) — the use path is alive", open);
}

/// The vent ladder (#686, 0x22 at (−13.16, −1.30, −9.55) [D]) grabs and climbs on the walk-in:
/// `Player_CollLadder` 0x196118 attaches, `Player_Climb` 0x196310 ascends [M ladder.rs]. The
/// teleport only reaches the StoreRoom floor below the office level (the operation — the climb —
/// is the real input). Sister: the pre-contact substate is not the climb.
#[test]
fn vent_ladder_grabs_and_climbs() {
    let mut sim = load().scene.sim.take().unwrap();
    let lad = sim.ladders.iter()
        .min_by(|a, b| {
            let d = |l: &nf_game::ladder::Ladder| (l.centre[0] + 13.16).powi(2) + (l.centre[2] + 9.55).powi(2);
            d(a).total_cmp(&d(b))
        })
        .map(|l| l.clone())
        .expect("the Vent_StoreRoom_Ladder");
    assert!((lad.centre[0] + 13.16).abs() < 2.0 && (lad.centre[2] + 9.55).abs() < 2.0,
        "the nearest ladder to the vent spot is #686 [D], at {:?}", lad.centre);
    // Stand facing the ladder from its front (the yaw convention: forward = (sin, ·, cos) [M ladder.rs]).
    let (s, c) = (lad.yaw.sin(), lad.yaw.cos());
    let stand = [lad.centre[0] + s * 1.2, lad.bottom() + 0.1, lad.centre[2] + c * 1.2];
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, stand, lad.yaw + std::f32::consts::PI);
    assert!(sim.bond.substate != nf_game::player::SS_CLIMB, "not climbing before the walk-in");
    let y0 = sim.bond.pos[1];
    let fwd = nf_game::player::Input { forward: 1.0, ..Default::default() };
    let mut grabbed = None;
    for t in 0..600 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(fwd, false, false, false);
        if sim.bond.substate == nf_game::player::SS_CLIMB { grabbed = Some(t); break; }
    }
    let t = grabbed.expect("walking into the vent ladder must attach the climb [M 0x196118]");
    // The climb ascends [M 0x196310].
    let y_grab = sim.bond.pos[1];
    for _ in 0..120 { sim.step_bond(fwd, false, false, false); }
    assert!(sim.bond.pos[1] > y_grab + 0.5,
        "the climb ascends the vent shaft [M 0x196310], {} → {}",
        y_grab, sim.bond.pos[1]);
    eprintln!("[0a] the vent ladder grabbed at t{t}, climbed {:.2} → {:.2}", y_grab, sim.bond.pos[1]);
    let _ = y0;
}

/// The lift-shaft HurtPoly (#745-749, p1 = 1000 damage, at y = −29.2 [D]) kills on the fall-in:
/// `Hurt_Create`/`Hurt::update_touch` deliver the volume's damage [M hurt.rs]. The teleport only
/// drops Bond into the shaft mouth above the volume; the damage runs through the real touch.
/// Sister: standing on the office floor the touch never fires.
#[test]
fn lift_shaft_hurt_volume_deals_the_fall_damage() {
    let mut sim = load().scene.sim.take().unwrap();
    let hurt = sim.hurts.iter().enumerate()
        .min_by(|a, b| sim.hurts[a.0].lo[1].total_cmp(&sim.hurts[b.0].lo[1]))
        .map(|(_, h)| h.clone())
        .expect("the shaft HurtPolys");
    assert!(hurt.lo[1] < -20.0, "the shaft volumes sit at y = −29.2 [D], got {}", hurt.lo[1]);
    assert!(hurt.damage >= 1000.0, "p1 = 1000 damage [D], got {}", hurt.damage);
    let hurt_xz = [(hurt.lo[0] + hurt.hi[0]) / 2.0, (hurt.lo[2] + hurt.hi[2]) / 2.0];
    // Sister: on the office floor, above the volume, no touch.
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [0.0, 0.1, 0.0], 0.0);
    let h0 = sim.drones.health.health;
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!((sim.drones.health.health - h0).abs() < 0.5, "no hurt touch on the office floor");
    // Drop Bond into the shaft at the volume's x/z, just above it — the real fall reaches it.
    sim.bond = nf_game::player::Bond::stand_at(&sim.world,
        [hurt_xz[0], hurt.lo[1] + 2.0, hurt_xz[1]], 0.0);
    for _ in 0..120 {
        if sim.drones.health.health <= 0.0 { break; }
        sim.step_bond(Default::default(), false, false, false);
    }
    assert!(sim.drones.health.health <= 0.0,
        "the shaft volume's 1000 damage kills [M hurt.rs], health {}",
        sim.drones.health.health);
}

/// The TouchOnce plates (0xEA ×19 [D]) fire their out channel once on the player's contact and
/// disarm: `Trigger_Update`'s type-1 arm writes the out channel and latches the glist-0x10
/// self-delete [M 0x1b1a78's type-1 arm + 0x1b1d78-90]. Two arms pinned here:
/// - #58, the arrival plate under the spawn (out ch 0x47 = objective 3 [D]): off at the fresh
///   boot's tick 0 (the port's pre-update state; the [L] snapshot's ch 0x47-on is the game's own
///   boot AFTER its first updates), on within the first ticks, then `gone` — the once.
/// - #757, the hatch voice plate (out ch 0xB7, lock ch 0x3A [D]) on the EARLY approach: its lock
///   channel is off on a fresh boot [L ee_start_0700000a.bin @0x26fc90], so the plate is live and
///   the touch fires ch 0xB7 (the May-Hatch line's channel; the dead-after-keyboard arm is
///   walkreal_0700000a's end-of-route pin). The teleport stands Bond at the hatch mouth's
///   anteroom floor — the walkreal walks this transit; this test pins the plate.
#[test]
fn touchonce_plates_fire_once_and_the_hatch_plate_fires_on_the_early_approach() {
    let mut w = load();
    // Sister: the arrival channel is off before any tick ran.
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x47),
        "sister: ch 0x47 is off at the fresh boot's tick 0");
    let ti = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .position(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.out == 0x47)
        .expect("the arrival plate #58 (TouchOnce, out ch 0x47) [D]");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x47),
            "the arrival plate fires under the spawn [D #58 out ch 0x47; M Trigger_TouchOnce's contact write]");
        assert!(sim.drones.triggers[ti].gone,
            "the TouchOnce latched its self-delete after the fire [M 0x1b1d78-90's glist-0x10 latch] — the once");
    }
    // The hatch voice plate on the EARLY approach (the keyboard's ch 0x3A still off [L @0x26fc90]).
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x3a),
        "sister: the keyboard's ch 0x3A is off on the fresh boot — #757's lock gate is open");
    teleport(&mut w, [21.5, 1.2, 19.0]); // the hatch mouth's anteroom floor (the load point's y 1.12 [D])
    w.wait(20);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xb7),
        "the hatch voice plate #757 fires ch 0xB7 on the early approach (lock ch 0x3A off) [D #757; M triggers.rs's lock gate]");
}

/// Doors (0xDB ×19 [D]): the unlocked swing opens on the E (`Door_Activate`'s use path [M
/// door.rs]); the keypad-locked one (#724 room-L_door2, lock ch 48 [D p3]) refuses the same E
/// while its lock channel is off — the activation only plays the hit sound [M door.rs's +0x68
/// map]. #540 office_b_door (−14.2, 1.1, −13.0) carries no lock channel [D].
#[test]
fn doors_open_on_use_and_the_keypad_locked_door_refuses() {
    let mut w = load();
    // The alarm test's proven legs to the west offices.
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 16.0).r(1.2), "the west hall");
    // (a) office_b_door #540 [D p3 = 0]: the AI graph's route to its front, then E it.
    let (dpos, di) = walker::door_near(&w.scene.sim.as_ref().unwrap(), [-14.2, 1.1, -13.0])
        .expect("office_b_door #540 [D]");
    assert!((dpos[0] + 14.2).abs() < 0.4 && (dpos[2] + 13.0).abs() < 0.4,
        "the nearest door to the spot is #540, got {dpos:?}");
    assert!(w.follow_nav_route([dpos[0], 0.4, dpos[2] + 1.6], "the office door's front"),
        "no nav route to office_b_door's front");
    w.use_door(di, "E office_b_door #540");
    assert!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0xc != 0,
        "the unlocked door opens on the E [M Door_Activate]");
    // (b) the keypad-locked #724 (−10.6, 1.1, −19.3, lock ch 48 [D]) refuses while ch 48 is off:
    // the AI graph's route back to the south-west corridor, then the pocket south of the pillar
    // to its north face (the ring route's tail [D probe rays]).
    assert!(w.follow_nav_route([-15.8, 0.4, -15.0], "the south-west corridor"), "no nav route");
    w.walk_to(Wp::new(-14.5, 0.4, -15.0).r(1.2), "the pocket west");
    w.walk_to(Wp::new(-12.0, 0.4, -16.8).r(1.0), "the pocket south-east");
    let ldi = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == 724).expect("room-L_door2 #724 [D]");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[ldi].ch_lock, 48,
        "#724's lock channel [D p3]");
    let lpos = w.scene.sim.as_ref().unwrap().drones.doors.list[ldi].pos;
    w.walk_to(Wp::new(lpos[0] - 1.0, 0.4, lpos[2] + 1.5).r(0.9), "the locked door's north-west front");
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[ldi].state;
    w.use_door(ldi, "E the keypad-locked #724 (sister)");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[ldi].state, before,
        "the door locked on ch 48 must not move while the channel is off [M door.rs's ch_lock map], state {before:#x}");
}

/// The keypad lock's E mini-game (0x2E Lock ×4 [D]): E on keypad #648 (−10.7, 1.5, −17.5 [D],
/// kind 0, code index 1 [D p1]) arms the keypad menu (state 3 [M Lock_Activate 0x1aed00's kind-0
/// no-key branch]); the digit presses fill the typed buffer (`C_KEYPAD_Handler` 0x216810's msg
/// 0x4b [M]); a full WRONG buffer never matches (`Lock_Update` case 4's compare [M
/// 0x1ae3d8:262-296]); the game's own code (the snapshot boot table [L ee_start_0700000a.bin
/// @0x2beff8, code index 1]) solves: `Lock_Unlock` sets the lock's state 1 and the next tick's
/// hold writes switch_channels[48] [M case 4 → case 1] — door #724's rising-edge unlock +
/// auto-activate [M 0x1363c8:101-107] sweeps it open (state 4).
#[test]
fn keypad_lock_menu_takes_the_code_and_unlocks_its_door() {
    let mut w = load_warm_snap();
    // The walkreal ring's tail legs to the keypad approach.
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    w.walk_to(Wp::new(-10.0, 0.4, 16.0).r(1.2), "the west hall");
    assert!(w.follow_nav_route([-15.8, 0.4, -15.0], "the south-west corridor"), "no nav route");
    w.walk_to(Wp::new(-14.5, 0.4, -15.0).r(1.2), "the pocket west");
    w.walk_to(Wp::new(-12.0, 0.4, -16.8).r(1.0), "the pocket south-east");
    w.walk_to(Wp::new(-11.9, 0.4, -17.5).r(0.8), "the keypad's stand");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.keypad.is_none(), "sister: no keypad page before the E");
        assert!(!sim.drones.channels.on(48), "sister: the lock's out channel is off [L fresh boot]");
        let d = sim.drones.doors.list.iter().find(|d| d.placement == 724).expect("door #724 [D]");
        assert!(d.state & 0xc == 0, "the code door stands closed [state 2 = the closed marker], got {:#x}", d.state);
    }
    // E the keypad: aim at its world sphere and pulse (the walkreal's mouth pattern
    // [M 0x1a5170:158-166's facing gate + 0x1aed00's kind-0 arm]).
    let kp: [f32; 3] = w.scene.sim.as_ref().unwrap().drones.locks.iter()
        .find(|l| l.placement == 648).expect("keypad #648 [D]").pos;
    let armed = use_toward(&mut w, kp, &|sim| sim.drones.keypad.is_some(), 8);
    assert!(armed, "the E arms the keypad menu (state 3) [M 0x1aed00's kind-0 no-key branch]");
    // The wrong-code sister: four wrong digits fill the buffer; the case-4 compare fails every
    // tick and the page stays open, unlocked nothing [M 0x1ae3d8:262-296].
    for d in [9u32, 9, 9, 9] {
        w.scene.sim.as_mut().unwrap().drones.keypad_digit(d);
        w.wait(1);
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.keypad.is_some(), "the page stays open while the buffer is wrong [M case 4's compare]");
        assert!(!sim.drones.channels.on(48)
            && sim.drones.locks.iter().filter(|l| l.placement == 648).all(|l| !l.unlocked),
            "the wrong code never unlocks [M case 4]");
    }
    // The game's own code (the snapshot boot table [L 0x2beff8], the lock's index [D p1]).
    let code = {
        let sim = w.scene.sim.as_ref().unwrap();
        let idx = sim.drones.locks.iter().find(|l| l.placement == 648).expect("keypad #648").code;
        sim.drones.key_codes.code(idx)
    };
    for d in code {
        assert!(d >= b'0' && d <= b'9', "the snapshot code digits are ASCII [L 0x2beff8]");
        w.scene.sim.as_mut().unwrap().drones.keypad_digit((d - b'0') as u32);
        w.wait(1);
    }
    // The solve → the channel hold → the door's auto-activate (state 4).
    let mut ok = false;
    for _ in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = sim.drones.doors.list.iter().find(|d| d.placement == 724).expect("door #724 [D]");
        if sim.drones.channels.on(48) && d.state == 4 { ok = true; break; }
        w.wait(1);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(ok, "the code match unlocks ch 48 [M 0x1ae3d8 case 4 → case 1] and door #724 auto-opens \
        [M 0x1363c8:101-107], ch48 {} door state {:#x}",
        sim.drones.channels.on(48), sim.drones.doors.list.iter().find(|d| d.placement == 724).unwrap().state);
    assert!(sim.drones.locks.iter().filter(|l| l.placement == 648).all(|l| l.unlocked),
        "the used lock's state 1 [M 0x1b0650]");
}

/// The Q-Worm PC kind (0x35 ×3 [D]): the install writer is the weapon-0x58 BULLET entering the
/// PC's contact list — `PCQWorm_Update` 0x1aee20's class-5 toucher with ext+0x44 == 0x58, gated
/// on param 3 (0x1aee20:49-51) — so a non-Q-Worm round never installs. #688 Q-WORM_PC_1
/// (−16.88, 1.20, −32.15, p1 = 0x48, p3 = 1 [D]) writes ch 0x48 (objective 4) and runs to its
/// done state [M 0x1aee20's state 2]; the decoy office PC #684 (45.45, 0.03, −17.84, p2 = 1, no
/// channel [D]) is E-usable instead (`PCQWorm_Activate` 0x1b0840's p2 gate): the E starts the
/// same machine and its completion reveals the KeyCodes entry p0 (the flag byte + the INFO
/// message) [M 0x1aee20:24-31] while ch 0x48 stays the armed PC's alone.
#[test]
fn qworm_pc_install_needs_the_gadget_and_the_office_pc_reveals_its_code_on_the_e() {
    let mut w = load();
    // The walked south stand (the walkreal's bullet-clear band [D probe: segment_bullet CLEAR
    // from every stand x −16..−17.8, z −33.5..−34.4]) — the teleport is the stand-in transit;
    // the equip, the trigger pull and the install are the real path.
    teleport(&mut w, [-16.5, 1.2, -34.3]);
    let pc = w.scene.sim.as_ref().unwrap().drones.useobjs.pcs.iter().find(|p| p.ch == 0x48)
        .expect("Q-WORM_PC_1 (0x35, p1 = 0x48) [D]").pos;
    // Sister: the spawn K5 (0x43) round is not the Q-Worm — no install.
    let _ = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 3.0, 240, "the K5 at the PC", &|_| false);
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x48).unwrap();
        assert!(p.state == 0 && !sim.drones.channels.on(0x48),
            "sister: a weapon-0x43 round never installs [M 0x1aee20's 0x58 test], state {}", p.state);
    }
    // The real equip through the gadget cycle (mode-1 bounds 0x4A..0x5E, owned entries only
    // [M player.rs Input::gadget_cycle]).
    let mut cycles = 0;
    loop {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x58 { break; }
        assert!(cycles < 20, "the gadget cycle never reached the Q-Worm 0x58 (weapon {:#x})",
            w.scene.sim.as_ref().unwrap().gun.weapon);
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        cycles += 1;
    }
    let shots = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 3.0, 400, "the Q-Worm install",
        &|sim| sim.drones.channels.on(0x48));
    w.wait(6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48),
            "ch 0x48 (objective 4) from the real install: {shots} shots [M PCQWorm_Update 0x1aee20]");
        assert!(sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x48).unwrap().state == 2,
            "the install machine ran to its done state [M 0x1aee20's state 2]");
    }
    // The decoy office PC #684: the E arm (p2 = 1 [D]) reveals its keycode entry. The stand:
    // the first candidate east-office spot that grounds (the placement's floor level is y ≈ 0
    // [D pos], off the walked route — the probe finds the standing spot the E needs).
    let mut stand: Option<[f32; 3]> = None;
    for cand in [[44.4, 0.6, -17.84], [44.9, 0.6, -17.0], [44.9, 0.6, -18.7], [43.0, 0.6, -17.84], [45.45, 0.7, -17.84]] {
        teleport(&mut w, cand);
        let ok = w.scene.sim.as_ref().unwrap().bond.grounded;
        eprintln!("[0a] decoy stand candidate {cand:?} grounded {ok} pos {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
        if ok { stand = Some(cand); break; }
    }
    let (dk, dpos, dr) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.useobjs.pcs.iter()
            .find(|p| (p.pos[0] - 45.45).abs() < 0.4 && (p.pos[2] + 17.84).abs() < 0.4)
            .expect("the office PC #684 [D]");
        (p.keys, p.pos, p.radius)
    };
    eprintln!("[0a] decoy #684: pos {dpos:?} r {dr} keys {dk}");
    assert!(dk != 0, "the decoy carries a KeyCodes index [D p0], got {dk}");
    let Some(stand) = stand else {
        eprintln!("[0a] no grounded stand found near #684 — the E arm stays unit-pinned (useobj.rs's p2 tests), the live drive skipped");
        return;
    };
    let _ = stand;
    let target = [dpos[0], dpos[1] + 0.55, dpos[2]];
    let started = use_toward(&mut w, target,
        &|sim| sim.drones.useobjs.pcs.iter()
            .find(|p| (p.pos[0] - 45.45).abs() < 0.4 && (p.pos[2] + 17.84).abs() < 0.4)
            .map_or(false, |p| p.state != 0), 8);
    assert!(started, "the E starts the decoy PC's install [M PCQWorm_Activate 0x1b0840's p2 arm]");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.useobjs.pcs.iter()
            .find(|p| (p.pos[0] - 45.45).abs() < 0.4 && (p.pos[2] + 17.84).abs() < 0.4).unwrap();
        assert!(p.state == 2, "the E-install ran to its done state [M 0x1aee20's state 2], got {}", p.state);
        assert_eq!(sim.drones.key_codes.0[dk as usize][4], 1,
            "the revealed flag byte set [M 0x1aee20:24-31]");
        assert!(sim.drones.channels.on(0x48),
            "ch 0x48 is unaffected by the decoy's E (its p1 = none [D]) — the armed PC's channel alone");
    }
}

/// The security keyboard SP (#60 SCRIPTPLAYER_KeyboardMH, 0xD9 mask 4 [D]) is the
/// player-triggered SimpleScript kind: the mode-64 preload runs script 0x06000597's t=0 Entity
/// op so the keyboard prop (glist hash 0x020005E1) exists from load [M SP_CreateScriptPlayer
/// 0x1db9a8's load beat]; the E pick reaches its baked sphere (−1.54, 1.33, 8.48) r 0.66 from the
/// corridor wedge [M 0x1a5170's class-0 +0xE4 dispatch → SP_Activate 0x1dc338], the state-1
/// activation plays the script — its stream's tick-15 op spawns the second indicator 0x020005E2
/// [D nis dump] — and toggles the out ch 0x3A [M SP_Update 0x1dbda8], whose AND mux #741 raises
/// ch 0x49 (objective 5) on the mission graph's own update [D #741]. Sisters: 0x020005E2 absent
/// and ch 0x3A/0x49 off before the E.
#[test]
fn keyboard_sp_use_plays_the_script_and_fires_the_objective_mux() {
    let mut w = load_warm_snap();
    w.wait(150); // the SS lift doors' create sweep (the keyboard test's wait)
    // The keyboard test's proven legs to the corridor wedge.
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    w.walk_to(Wp::new(-3.2, 0.0, 9.4).r(1.0), "hug the cab east wall south end");
    let _ = w.try_walk_to(Wp::new(-2.55, 0.0, 8.1).r(1.0), "round the wall end");
    let ents = |w: &Walker, hash: u32| w.scene.sim.as_ref().unwrap().drones.scripts.entities().iter()
        .filter(|e| e.2 == hash).count();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(ents(&w, 0x0200_05e1) >= 1,
            "the preload's keyboard prop exists from load [M 0x1db9a8's load beat]");
        assert_eq!(ents(&w, 0x0200_05e2), 0,
            "sister: the tick-15 indicator is absent before the activation");
        assert!(!sim.drones.channels.on(0x3a) && !sim.drones.channels.on(0x49),
            "sister: the out chain is off before the E");
    }
    let target: [f32; 3] = [-1.54, 1.33, 8.48];
    let fired = use_toward(&mut w, target, &|sim| sim.drones.channels.on(0x3a), 12);
    assert!(fired,
        "the keyboard SP's E activation writes ch 0x3A [M SP_Activate 0x1dc338 + SP_Update 0x1dbda8 state 1]");
    // The script's tick-15 op + the mux follow on their own updates.
    let mut got = false;
    for _ in 0..120 { if ents(&w, 0x0200_05e2) >= 1 { got = true; break; } w.wait(1); }
    assert!(got,
        "the activation's stream spawns the second indicator 0x020005E2 [D nis 0x06000597's tick-15 op]");
    let mut mux = false;
    for _ in 0..180 { if w.scene.sim.as_ref().unwrap().drones.channels.on(0x49) { mux = true; break; } w.wait(1); }
    assert!(mux, "AND mux #741: ch 0x3A → ch 0x49 (objective 5) [D #741]");
}

/// The hatch exit (0xE8 LOAD_LEVEL #59, next 0x0700000b, key gate ch 0x4A [D]) refuses the E
/// while the key gate is off — no load, no channel writes, the locked message 0x2000021 posted
/// [M 0x1b2318:76-90] — and loads through the real objective chain: the keyboard's ch 0x3A → AND
/// #741 → ch 0x49, the Q-Worm install's ch 0x48, AND #742 → ch 0x4A [D muxes], then the same E
/// requests the level [M 0x1b2318:111] (and the auto path's gate-off arm writes the fallback
/// channel [M 0x1b2318:96-108's else arm]). The teleports between stations are the stand-in
/// transits (the walkreal walks them); every operation runs through the real input.
#[test]
fn hatch_load_point_refuses_then_loads_through_the_real_chain() {
    let mut w = load_warm_snap();
    // The exit's volume centre (the walk_0700000a hatch test's proven E stand — the plate's own
    // AABB centre, y +1.0 onto the floor).
    let centre = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL && t.next_level == 0x0700_000b)
        .map(|t| [(t.lo[0] + t.hi[0]) * 0.5, t.lo[1] + 1.0, (t.lo[2] + t.hi[2]) * 0.5])
        .expect("the hatch exit #59 [D]");
    // Sister: the blocked E on the fresh carry.
    teleport(&mut w, centre);
    for t in w.scene.sim.as_ref().unwrap().drones.triggers.iter() {
        if t.kind == nf_game::triggers::kind::LOAD_LEVEL {
            eprintln!("[0a] LOAD_LEVEL: lo {:?} hi {:?} next {:#x} key {:#x} gate {:#x} fb {:#x}",
                t.lo, t.hi, t.next_level, t.key_gate, t.gate, t.fallback);
        }
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a),
        "sister: the key gate ch 0x4A is off on the fresh carry");
    assert!(w.scene.sim.as_ref().unwrap().drones.level_exit.is_none(), "no exit before the E");
    // The E press, the walk_0700000a hatch test's proven shape: from the volume centre, the use
    // key held for the press edge (the game's Input_Action mask-4 edge [M 0x1a5460:395-403]).
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.0;
    for k in 0..12 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, k % 6 == 0);
        w.total_ticks += 1;
        w.after_step();
        let sim = w.scene.sim.as_ref().unwrap();
        if !sim.drones.info_labels.is_empty() || !sim.hud.info_labels.is_empty() {
            eprintln!("[0a] tick {k}: drones.info_labels {:?} hud.info_labels {:?}",
                sim.drones.info_labels, sim.hud.info_labels);
        }
    }
    eprintln!("[0a] after the direct E: pos {:?} exit {:?}", w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.level_exit);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.level_exit.is_none(),
            "the blocked E never loads [M 0x1b2318:76-90's key-gate arm]");
        // The locked message: `Trigger_Activate` posts label 0x2000021 (type-1 TXTMSG, life 0xb4
        // [M 0x1b2318:81-86]); the port's pipeline takes the trigger flag into `drones.info_labels`,
        // drains it into `hud.info_labels` and binds it into the INFO pane's `info_msgs` the same
        // tick — the pane's message list is the end observable.
        assert!(sim.hud.info_msgs.iter().any(|m| m.label == 0x200_0021),
            "the locked message posted [M 0x1b2318:81-86], msgs {:?} labels {:?}",
            sim.hud.info_msgs.iter().map(|m| m.label).collect::<Vec<_>>(), sim.hud.info_labels);
    }
    // The chain, station by station: the keyboard first (silent), then the install.
    // In the security room, in front of the keyboard's screen (its collision faces face −z [D the
    // entity glist 0x20005e1]); the use pick keeps it only from in front [M Intersect_PointGeom
    // 0x1e92d8 via Collide_Intersect 0x1ebaa8; nf_game::usepick] — not through the corridor
    // wall the old sphere-only pick reached it from.
    teleport(&mut w, [-1.41, 1.09, 7.21]);
    let kb: [f32; 3] = [-1.55, 1.5, 8.7];
    let fired = use_toward(&mut w, kb, &|sim| sim.drones.channels.on(0x3a), 12);
    assert!(fired, "the keyboard's E writes ch 0x3A [M SP_Activate 0x1dc338]");
    let mut mux49 = false;
    for _ in 0..180 { if w.scene.sim.as_ref().unwrap().drones.channels.on(0x49) { mux49 = true; break; } w.wait(1); }
    assert!(mux49, "AND mux #741: ch 0x3A → ch 0x49 [D #741]");
    // The install from the walked south stand — the T4 shape (the K5 primer burst first, then the
    // real gadget-cycle equip; the same stand the T4 install lands from).
    teleport(&mut w, [-16.5, 1.2, -34.3]);
    let pc = w.scene.sim.as_ref().unwrap().drones.useobjs.pcs.iter().find(|p| p.ch == 0x48)
        .expect("Q-WORM_PC_1 [D]").pos;
    let _ = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 3.0, 120, "the K5 primer at the PC", &|_| false);
    let mut cycles = 0;
    loop {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x58 { break; }
        assert!(cycles < 20, "the gadget cycle never reached the Q-Worm 0x58");
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        cycles += 1;
    }
    let shots = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 3.0, 400, "the Q-Worm install",
        &|sim| sim.drones.channels.on(0x48));
    eprintln!("[0a] chain install: shots landed {} bond {:?} weapon {:#x} ch48 {}",
        shots, w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().gun.weapon,
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x48));
    w.wait(6);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48),
        "the install writes ch 0x48 [M PCQWorm_Update 0x1aee20]");
    let mut mux4a = false;
    for _ in 0..30 { if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) { mux4a = true; break; } w.wait(1); }
    assert!(mux4a, "AND mux #742: 0x48+0x49 → the exit's key ch 0x4A [D #742]");
    // The same E now loads.
    teleport(&mut w, centre);
    let fb = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL && t.next_level == 0x0700_000b)
        .map(|t| t.fallback).expect("the hatch exit #59 [D]");
    w.walk_to(Wp::at(centre[0], centre[1], centre[2], Act::Use { use_ticks: 600 }), "E the hatch (the key gate on)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_000b),
            "the hatch loads 0700000b [D #59 p0; M 0x1b2318:111]");
        // The auto path's gate-off arm (the gate is param 2 = 0 [D]) writes the fallback channel
        // and still loads [M 0x1b2318:96-108's else arm].
        if fb != 0 {
            assert!(sim.drones.channels.on(fb as u32),
                "the fallback channel {fb} wrote on the successful E [M 0x1b2318:96-108's else arm]");
        }
    }
}
