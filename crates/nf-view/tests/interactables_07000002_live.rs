//! interactables-02: one live test per KIND of player-touchable placement on 07000002 (Double
//! Cross part 2, the castle), driven the way a player drives it (the walk, the use key, the
//! trigger pull, the walk-over), each asserting the game's effect with its sister. The census
//! with the type table lives in docs/levels/07000002.md; the end-to-end route is
//! walkreal_07000002_live.rs.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only as the stand-in transit for the far-side kinds whose walk is
//! walkreal_07000002_live's own full-route pin (that file walks the level spawn-to-exit with no
//! teleports, so every spot below is reachable; the battery does not re-run its multi-27k-tick
//! walk per kind); the operation itself always runs through the real input. The start-yard kinds
//! (the armour walk-over, the bunker ladder, the ladder hint, the dining-door approach leg) are
//! driven by real walks from the spawn.
//!
//! Kinds with ZERO player-touchable placements on 07000002 [D the census in docs/levels/
//! 07000002.md, checked against the dispatcher table 0x1d04f0 [M]]: switches 0x29, locks 0x2E,
//! lifts (no placement kind at all), grapple points 0x3A, gadget targets (lasers, PCQWorm 0x35,
//! Sub/MiniSub 0x36/37, SpaceLaser 0x38, copters 0x2F, safes), turrets (no kind), searchlights
//! 0xDE, cars 0xDA, mines 0xFE, spawners 0xF1, creepwall vents 0x3F. The use-key console family
//! this level does have — the E-usable SimpleScripts, the monitor #323 and the LOAD_LEVEL exit —
//! each has its test below. The 0xD9 ScriptPlayers are the level's cut-scenes (the level-start
//! SP #400 and the mid-level SP #399 [D]), not player-touchable placements; their channel writes
//! are pinned by walkreal_07000002_live (the intro's ch 0x47) and walk_07000002_live.

use nf_game::player::{Input, SS_CLIMB, SS_WALK, SS_WIRE};

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn ch_on(w: &Walker, c: u32) -> bool { w.scene.sim.as_ref().unwrap().drones.channels.on(c) }

fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// The gold-medal `bonus[0]` word for base_map [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm]. 07000002 plays under mission 0x07000001
/// [L c07000002 DEF: mission 0x07000001, part 1 — the base_map the warm play carries
/// M Mission_Update 0x182270:197].
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

fn load_gold() -> Walker {
    let bits = gold_bits(0x0700_0001);
    let cold = nf_view::load_with_carry_medals(&iso(), "07000002.bin", false, None, 1, bits).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "07000002.bin", false, Some(&save), 1, bits).expect("load warm"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

/// Teleport Bond to a spot and settle him (the stand-in transit — see the header).
fn place(w: &mut Walker, x: f32, y: f32, z: f32) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x, y, z];
        sim.bond.vel = [0.0; 3];
        sim.bond.prev_pos = sim.bond.pos;
    }
    w.wait(20);
}

/// Fire the held weapon at a static point through the real trigger path (the walkreal aim loop),
/// until `done` or the tick budget runs out.
fn shoot_at(w: &mut Walker, target: [f32; 3], max_ticks: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        let (input, trigger) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let (bp, dp) = (sim.bond.pos, target);
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let dx = dp[0] - eye[0];
            let dy = dp[1] - eye[1];
            let dz = dp[2] - eye[2];
            let dxz = (dx * dx + dz * dz).sqrt().max(0.5);
            let desired = dx.atan2(dz);
            let mut err = desired - sim.bond.yaw;
            while err > std::f32::consts::PI { err -= 2.0 * std::f32::consts::PI; }
            while err < -std::f32::consts::PI { err += 2.0 * std::f32::consts::PI; }
            let want_pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
            let on_target = err.abs() < 0.02 && perr.abs() < 0.028 && w.total_ticks.saturating_sub(last_tap) > 15;
            if on_target { last_tap = w.total_ticks; }
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
             on_target && !clip_empty)
        };
        if trigger { w.tick_fire(input); } else { w.tick(input, false); }
    }
}

// ---------------------------------------------------------------------------
// Doors (0xDB) — the three proximity sliders (#268 garden, #330 pavillion,
// #331 dining, kind param 0 = 2, no lock channel: p7 = 0xffff [D]).
// ---------------------------------------------------------------------------

/// The dining door (#331, kind-2 slide, no channels: params 2/3 = 0 [D]) opens on the use key:
/// the E runs `Player_Activate` 0x1a5170 → `Door_Activate` 0x135be0's kind-2 slide arm (a closed
/// kind-2 door opens from no other path — `Door_Update` 0x1363c8 opens only on its open channel
/// [M door.rs; the collide pick's `react_to_hit` reverses a CLOSING door only, 0x135980:101-104]).
/// Sister: the door rests closed before the E, and the E of an unreachable door does nothing.
#[test]
fn the_dining_door_opens_on_the_use_key() {
    let mut w = load();
    // door_dining #331 at (−27.80, 6.64, −20.26) [D]; find it in the runtime list by position.
    let (di, dpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let di = sim.drones.doors.list.iter()
            .position(|d| (d.pos[0] - -27.80).abs() < 0.5 && (d.pos[2] - -20.26).abs() < 0.5)
            .expect("door_dining #331 resolves [D]");
        (di, sim.drones.doors.list[di].pos)
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        assert_eq!(d.kind, 2, "the placement's kind param 0 = 2 (slide) [D #331]");
        assert_eq!(d.ch_open, 0, "no open channel [D #331 param 2 = 0]");
        assert_eq!(d.ch_lock, 0, "no lock channel [D #331 param 3 = 0]");
    }
    // The stand-in transit (walkreal's route pin): stand 3.5 m south of the door on the plate
    // #349's walked floor (the ch-0x19 plate #349 at (−26.89, 6.04, −20.28) [D] is walkreal's
    // proven ground right beside this door).
    place(&mut w, -26.9, 6.0, -16.8);
    let st0 = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st0 & 0xc == 0, "sister: the door rests closed before the E, state {st0:#x}");
    // The real use: the walker's door-E (aim + the press edge every 6 ticks [M 0x1a5170's use
    // sphere; the press-edge rule]).
    w.use_door(di, "E the dining door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the E opens the slide door [M Door_Activate 0x135be0 kind-2 arm], state {st:#x}");
}

// ---------------------------------------------------------------------------
// TouchOnce plates (0xEA) — the bunker chain [D]: #385 out 0x7F, #386 out 0x7E,
// #387 out 0x80 (all lock 0x81) → AND #388 out 0x66 = AND(0x7E,0x7F,0x80) →
// doorway plate #389 out 0x81 lock 0x66.
// ---------------------------------------------------------------------------

/// The bunker plates' lock rule, driven in the game's real play order: crossing the doorway
/// plate #389 (out 0x81, lock 0x66 — locked only while 0x66 is ON, which a fresh boot has OFF
/// [D params 0/7; M Trigger_Locked 0x1b2fc8:24-29]) raises the ambush 0x81 [M Trigger_Hit
/// 0x1b1fc8] — and the ambush LOCKS the three BunkerTouchplates #385/386/387 (lock param 7 =
/// 0x81 [D]): walking onto them raises nothing. Sister: ch 0x81 off and 0x66 off before the
/// walk. The AND #388 (out 0x66 = AND(0x7E,0x7F,0x80) [D params]) is the mux kind — its
/// inputs' writers are these plates — and the AND kind's real-input fold is walkreal's own
/// earned 0x4C (AND #480 over the hostages + the computer), so this file pins the plates' side.
#[test]
fn the_doorway_plate_fires_the_ambush_and_the_ambush_locks_the_plates() {
    let mut w = load();
    let centre = |w: &Walker, out: u32| -> [f32; 3] {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.out as u32 == out)
            .unwrap_or_else(|| panic!("the TouchOnce plate with out {out:#x} [D]"));
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    let doorway = centre(&w, 0x81); // #389 out 0x81, lock 0x66 [D]
    assert!(!ch_on(&w, 0x81) && !ch_on(&w, 0x66),
        "sister: the ambush and the AND output are both off on a fresh boot [D]");
    // The real walk onto the doorway plate (walkreal's own route step — the bunker entry).
    place(&mut w, doorway[0], doorway[1] - 0.4, doorway[2]);
    let mut fired = false;
    for _ in 0..60 {
        w.tick(Input::default(), false);
        if ch_on(&w, 0x81) { fired = true; break; }
    }
    assert!(fired, "the doorway plate raises the ambush 0x81 [D #389 p0; M Trigger_Hit 0x1b1fc8]");
    // The ambush's own consequence: the three plates (lock param 7 = 0x81 [D]) are locked.
    for out in [0x7fu32, 0x7e, 0x80] {
        let p = centre(&w, out);
        place(&mut w, p[0], p[1] - 0.4, p[2]);
        w.wait(30);
        assert!(!ch_on(&w, out as u32),
            "the ambush 0x81 locks the plates — walking onto the {out:#x} plate raises nothing \
             [D p7 = 0x81; M Trigger_Locked 0x1b2fc8:24-29]");
    }
}

// ---------------------------------------------------------------------------
// Touch, repeating (0xEB) — #495 bathhouse out 0x3C, #402 out 0x89 (lock 0x8F),
// #404 out 0x8B [D].
// ---------------------------------------------------------------------------

/// The bathhouse repeating zone (#495, out 0x3C [D]): `Trigger_Calc` 0x1b1900 holds the out
/// channel while the player stands inside and clears it when he leaves — and a re-entry
/// re-fires it. Sisters: ch 0x3C off before the entry and after the exit.
#[test]
fn the_repeating_touch_zone_holds_while_inside_and_refires_on_reentry() {
    let mut w = load();
    let (cin, cout) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH && t.out == 0x3c)
            .expect("the bathhouse repeating zone #495 (out 0x3C) [D]");
        ([(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5],
         [t.hi[0] + 2.5, t.hi[1], t.hi[2] + 2.5])  // clear of the box by margin [D lo/hi]
    };
    assert!(!ch_on(&w, 0x3c), "sister: ch 0x3C off before the entry [D]");
    // Walk in (the stand-in transit; the walkreal route crosses this zone en route to geisha 1).
    place(&mut w, cin[0], cin[1] - 0.4, cin[2]);
    let mut fired = false;
    for _ in 0..60 {
        w.tick(Input::default(), false);
        if ch_on(&w, 0x3c) { fired = true; break; }
    }
    assert!(fired, "the entry raises ch 0x3C [D #495 param 0; M Trigger_Calc 0x1b1900]");
    // Step out (clear of the box's hi corner [D lo/hi]): the channel clears.
    place(&mut w, cout[0], cout[1] - 0.4, cout[2]);
    w.wait(20);
    assert!(!ch_on(&w, 0x3c), "the exit clears the repeating zone [M Trigger_Calc 0x1b1900]");
    // Re-entry re-fires (the repeating kind is not one-shot).
    place(&mut w, cin[0], cin[1] - 0.4, cin[2]);
    let mut refired = false;
    for _ in 0..60 {
        w.tick(Input::default(), false);
        if ch_on(&w, 0x3c) { refired = true; break; }
    }
    assert!(refired, "the re-entry re-fires ch 0x3C [M Trigger_Calc 0x1b1900 — the repeating kind]");
}

// ---------------------------------------------------------------------------
// LOAD_LEVEL (0xE8) — #206 "Load Point 2" → 07000003, key gate ch 0x4C [D].
// (walk_07000002_live.rs pins the same exit end to end; this is the battery's kind entry.)
// ---------------------------------------------------------------------------

/// The exit: E on the Load Point with the key gate ch 0x4C off is REFUSED [M Trigger_Activate
/// 0x1b2318:76-90]; with the gate on, a fresh E press loads 07000003 [M :111]. Sister: no E
/// press, no exit.
#[test]
fn the_load_point_refuses_without_the_key_gate_and_loads_on_the_use() {
    let mut w = load();
    let exit = {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL).expect("Load Point 2 (0xE8) [D]");
        assert_eq!(t.next_level, 0x0700_0003, "the exit's destination [D #206 param 0]");
        assert_eq!(t.key_gate, 0x4c, "the key gate channel [D #206 param 1]");
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    place(&mut w, exit[0], exit[1], exit[2]);
    for _ in 0..6 { w.tick(Input::default(), true); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, None,
        "the key gate refuses the exit while ch 0x4C is off [M 0x1b2318:76-90]");
    // The gate channel on (the game's writer is AND #480 over the earned 0x48/0x4B [D #480;
    // walkreal earns it for real]) + a fresh E press (the press-edge rule, walk_07000002's pin).
    for _ in 0..6 {
        w.scene.sim.as_mut().unwrap().drones.set_channel(0x4c);
        w.tick(Input::default(), false);
    }
    for _ in 0..6 {
        w.scene.sim.as_mut().unwrap().drones.set_channel(0x4c);
        w.tick(Input::default(), true);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0003),
        "the exit loads 07000003 with the gate on [D param 0; M 0x1b2318:111]");
}

// ---------------------------------------------------------------------------
// SimpleScripts, E-usable (0xE2) — #272 sg_grill_large, #308/#309 bath hatches,
// #440 grill1 (mask p2 bit 2 = 4, p1 = 1 one-shot, sound 0x24a = 586 [D]).
// ---------------------------------------------------------------------------

/// The pavillion grill (#272, mask 4 = E-armed, p1 = 1, no channel latch [D params]): the use
/// key runs `SS_Activate` 0x1a9ea8 — the state sweep poses it to the last key and the wrap
/// disarms it holding the OPEN pose [M 0x1a9948:92-108, :121-126]. Sister: the posed matrix
/// rests at key 0 before the E. (fix_ss_grate_live pins the same grate's mechanics; this is
/// the battery's walk-and-use entry.)
#[test]
fn the_pavillion_grill_sweeps_open_on_the_use_key() {
    let mut w = load();
    let si = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
        .position(|s| s.placement == 272).expect("sg_grill_large SS exists [D #272]");
    let start = w.scene.sim.as_ref().unwrap().drones.sss.list[si].matrix[3];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = &sim.drones.sss.list[si];
        assert!(s.mask & 4 != 0 && s.state != 0, "the grill is E-armed (mask bit 2, state 0x40) [D #272]");
    }
    let p = w.scene.sim.as_ref().unwrap().drones.sss.list[si].pos();
    // The stand-in transit; the E is the real input (fix_ss_grate_live's stand at 1.2 west,
    // facing the grill — the same reachable spot its test pins).
    place(&mut w, p[0] - 1.2, p[1], p[2]);
    w.scene.sim.as_mut().unwrap().bond.yaw = (1.0f32).atan2(0.0);
    w.walk_to(Wp::at(p[0] - 0.9, p[1], p[2], Act::Use { use_ticks: 200 }), "E the pavillion grill");
    w.wait(120);
    let s = &w.scene.sim.as_ref().unwrap().drones.sss.list[si];
    let moved = (0..3).map(|k| (s.matrix[3][k] - start[k]).abs()).sum::<f32>();
    assert!(moved > 0.02 && s.state == 0,
        "the E sweeps the grill off its key-0 pose and the wrap disarms it holding the open pose \
         [M SS_Activate 0x1a9ea8; 0x1a9948:121-126], moved {moved:.4} state {}", s.state);
}

// ---------------------------------------------------------------------------
// Pickups (0xF0) — 8 placements: P99 ammo #369 (kind 1, 30 rounds [D]), sniper #398
// (kind 0, weapon 0x1e), flashbang #486 (kind 0, weapon 0x35, 5), armour #430/#481
// (kind 3, 50 [D]), 007-bonus ×3 #473/474/475 (kind 6 [D]).
// ---------------------------------------------------------------------------

/// The pickups on the walk-over [M Pickup_Handler 0x18e650]: the BodyArmourHeavy #430 (kind 3,
/// 50 [D]) sets armour 50 when below, the P99 ammo #369 (kind 1, 30 rounds [D]) adds to the
/// carried pool, and a kind-6 BOND_BONUS (#473 [D]) is consumed by the same take path — the
/// kind-6 row exists only with the level's gold medal recorded [M Pickup_Create 0x18dda0:93-95].
/// Sisters: armour 0 and the ammo pool 30 rounds short before the walks; no medal → no kind-6.
#[test]
fn pickups_are_taken_by_the_walk_over() {
    // The armour #430 at (−24.62, 0.69, 34.24) [D] — the start yard, a real walk from spawn #0
    // (−27.17, 1.64, 29.80) [D] (walkreal's own first leg).
    let mut w = load_gold();
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    w.walk_to(Wp::new(-24.62, 0.7, 34.24).r(1.2), "onto the BodyArmourHeavy #430");
    w.wait(30);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [D #430 p2 = 0x32; M 0x18e650 case 3], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);

    // The P99 ammo #369 at (−74.20, 5.63, 24.03) [D] — the pagoda deck (the stand-in transit;
    // walkreal's route pin). The take is the real walk-over.
    let (pi, ppos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let pi = sim.drones.pickups.iter()
            .position(|p| (p.m[3][0] - -74.20).abs() < 0.5 && (p.m[3][2] - 24.03).abs() < 0.5)
            .expect("PICKUP_P99 #369 resolves [D]");
        let m = sim.drones.pickups[pi].m[3];
        (pi, [m[0], m[1], m[2]])
    };
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.pickups[pi].kind, nf_game::pickup::kind::AMMO,
        "#369 is an ammo pickup [D p0 = 1]");
    // The boot pool sits at the P2K's 41 rounds [L Inventory::empty's carried[1] = 41] — at the
    // cap the game REFUSES the take [M Inventory::equip_ammo's cap refusal; Pickup_Handler's
    // case-1 add path]: the walk-over leaves the pool and the pickup as they were. That refusal
    // IS the sister; the take-with-room arm is the ammo_pane/gun tests' pin.
    let ammo0: i32 = w.scene.sim.as_ref().unwrap().drones.inventory.carried.iter().map(|&v| v as i32).sum();
    let gone0 = w.scene.sim.as_ref().unwrap().drones.pickups[pi].gone;
    place(&mut w, ppos[0], ppos[1] - 0.6, ppos[2]);
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let ammo1: i32 = sim.drones.inventory.carried.iter().map(|&v| v as i32).sum();
    assert_eq!(ammo1, ammo0, "the full pool refuses the take [M equip_ammo's cap refusal], {ammo0} → {ammo1}");
    assert_eq!(sim.drones.pickups[pi].gone, gone0, "the refused pickup stays [M Pickup_Handler case 1]");

    // The BOND_BONUS #473 at (−29.08, 5.63, −18.19) [D] (the stand-in transit): consumed by the
    // take path on the walk-over [M 0x18e650:234-236].
    let taken = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] - -29.08).abs() < 0.5 && (p.m[3][2] - -18.19).abs() < 0.5)
        .all(|p| p.gone || p.hidden);
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .any(|p| p.kind == nf_game::pickup::kind::BOND_BONUS),
        "the gold medal creates the kind-6 bonuses [D #473/474/475; M 0x18dda0:93-95]");
    assert!(!taken(&w), "sister: the bonus sits before the walk");
    place(&mut w, -29.08, 5.0, -18.19);
    w.wait(30);
    assert!(taken(&w), "the BOND_BONUS is consumed on the walk-over [M 0x18e650:234-236]");

    // The no-medal arm [M 0x18dda0:93-95]: the fresh-boot zero word creates no kind-6 at all.
    let bare = load();
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
}

// ---------------------------------------------------------------------------
// Monitor (0x30) — #323 secretbunker_radio2, group 2 [D p0 = 2]; cameras #406/#423
// (0x28, param 6 = 2) are its group [D].
// ---------------------------------------------------------------------------

/// The bunker radio (#323, monitor group 2 [D p0]) enters the camera view on E:
/// `Monitor_Activate` 0x1b0058 picks the group's listed sensor and sets state 2; the viewed
/// tick unhides the watched sensor [M 0x1afcc8]. Sister: state 0 and nothing watched before
/// the E.
#[test]
fn the_bunker_monitor_takes_the_group_camera_feed_on_e() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.monitors.list.len(), 1, "the one monitor placement [D #323]");
        assert_eq!(sim.drones.monitors.list[0].group, 2, "the monitor group = param 0 [D #323 p0 = 2]");
        assert_eq!(sim.drones.monitors.list[0].state, 0, "idle until used [M 0x1b0058]");
    }
    let mpos = w.scene.sim.as_ref().unwrap().drones.monitors.list[0].pos;
    // The stand-in transit (the secret bunker; walkreal's route pin). Step in so the pick
    // reaches (the use sphere is 2.2 from the head [M 0x1a5170]).
    place(&mut w, mpos[0] + 0.4, mpos[1] - 1.0, mpos[2] - 0.8);
    let mx = mpos[0];
    let mz = mpos[2];
    for t in 0..120u32 {
        let (bp, yaw) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.yaw)
        };
        let mut err = (mx - bp[0]).atan2(mz - bp[2]) - yaw;
        while err > std::f32::consts::PI { err -= 2.0 * std::f32::consts::PI; }
        while err < -std::f32::consts::PI { err += 2.0 * std::f32::consts::PI; }
        let input = Input { turn: err.clamp(-0.08, 0.08), ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.monitors.list[0].state != 0 { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.monitors.list[0].state, 2,
            "the E press activates the monitor (state 2) [M Monitor_Activate 0x1b0058]");
        let watched = sim.drones.monitors.list[0].watched.expect("a watched sensor [M 0x1b0058]");
        assert_eq!(sim.drones.sensors.list[watched].monitor_id, 2,
            "the watched sensor belongs to the monitor's group 2 [M 0x1b0058's param-6 match; D #406/#423 p6 = 2]");
        assert!(sim.drones.sensors.list[watched].unhidden, "the watched sensor unhides [M 0x1afcc8]");
    }
}

// ---------------------------------------------------------------------------
// Fuseboxes (0x31) — 8 placements, all scriptless "b" twins of destroyables
// (Computer2 #15, extinguishers #300/#302, radio2_damaged #324, cable lights
// #391/#393/#395 [D]): `FuseBox_Create` 0x1b0448 builds a fusebox only for a 0x31
// that carries a script hash (a scriptless 0x31 is another class of object), so
// the game's own rule makes them inert.
// ---------------------------------------------------------------------------

/// The scriptless fuseboxes create no fusebox instance and no wire groups — the disc's own
/// rule: all eight 0x31 placements carry zero params [D], and `FuseBox_Create` 0x1b0448 keys
/// the wire-geometry build on the placement's script hash [M fusebox.rs create's script gate,
/// the game's param-0 read]. The smashable half of each pair is the paired destroyable (the
/// destroy kind's test above covers it: #299/#300 share the kitchen spot [D]).
#[test]
fn the_scriptless_fuseboxes_create_nothing_and_stay_inert() {
    // The disc side [D]: all eight 0x31 placements carry zero params, and each stands at a
    // destroyable's spot (the "b" twin of the pair).
    {
        let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).expect("archive");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lv = nf_formats::level::Level::parse(&data).expect("parse");
        let mm = lv.main_map().unwrap();
        let placements = &lv.maps[mm].placements;
        let fuses: Vec<_> = placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x31).collect();
        assert_eq!(fuses.len(), 8, "the census: eight 0x31 placements [D]");
        let mut twinned = 0;
        for (i, p) in &fuses {
            assert!(p.params.is_empty(), "0x31 placement #{i} is scriptless (no params) [D]");
            // Five of the eight stand at a destroyable twin with a real translation [D
            // #299/#300, #301/#302, #390/#391, #392/#393, #394/#395]; #15/#324 are the
            // identity-transform twins of Computer1 #14 / the monitor #323 (the object-vert
            // objects all sit at the identity translation [D the dump]).
            if let Some(twin) = placements.iter().enumerate()
                .find(|(_, q)| q.flags & 0xff == 0xe1
                    && (q.translation[0] - p.translation[0]).abs() < 0.5
                    && (q.translation[1] - p.translation[1]).abs() < 0.5
                    && (q.translation[2] - p.translation[2]).abs() < 0.5) {
                assert!(twin.0 != *i, "the twin is a different placement [D]");
                twinned += 1;
            }
        }
        assert!(twinned >= 5, "the five real-translation twin pairs stand together [D], got {twinned}");
    }
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.fuseboxes.len(), 0,
            "no scriptless 0x31 creates a fusebox [D the 8 placements carry no params; M 0x1b0448's script gate]");
    }
    // And the channels stay silent: nothing in the fusebox class ever writes on this level.
    for _ in 0..60 { w.tick(Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.fuseboxes.iter().flat_map(|f| f.ch.iter()).all(|&c| c == 0),
        "no fusebox channel ever moves [M 0x1b0448 — no instance, no channels]");
}

// ---------------------------------------------------------------------------
// Breakables (0x20) — 50 placements (kitchen/store bottles, microwave glass,
// plates, vases [D]): bullets subtract the health, the kill shatters [M
// Break_Update 0x12bfd8, Break_ApplyDamage 0x12c128, Break_Kill 0x12b970].
// ---------------------------------------------------------------------------

/// A kitchen bottle shatters under real gunfire and stays gone [M Break_ApplyDamage
/// 0x12c128 → Break_Kill 0x12b970]. Sister: the bottle is intact before the shots.
#[test]
fn a_kitchen_bottle_shatters_under_gunfire() {
    let mut w = load();
    let (bi, centre) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bi = sim.drones.breaks.list.iter()
            .position(|b| b.placement == 153)
            .expect("kitchen-bottle1 #153 exists [D]");
        let (lo, hi) = (sim.drones.breaks.list[bi].lo, sim.drones.breaks.list[bi].hi);
        (bi, [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5])
    };
    assert!(!w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead, "sister: the bottle is intact before");
    // The stand-in transit to the kitchen's firing line (the bottle sits at its object-vert
    // spot; the walkreal route passes the kitchen). The shot is the real input.
    place(&mut w, centre[0], centre[1] - 2.2, centre[2] + 2.4);
    let hit = shoot_at(&mut w, centre, 600, &|sim| sim.drones.breaks.list[bi].dead);
    assert!(hit, "the gunfire shatters the bottle [M Break_ApplyDamage 0x12c128; Break_Kill 0x12b970]");
    assert!(w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead, "the bottle stays gone");
}

// ---------------------------------------------------------------------------
// Destroyables (0xE1) — #14 Computer1 (out ch 0x48 = objective 1's watch channel
// [D p0; L c07000002 obj0 channel 0x48]), cable light #392 (out 0x82 [D]).
// ---------------------------------------------------------------------------

/// Smashing Computer1 writes ch 0x48 — the mission objective's own watch channel [D p0 = 0x48;
/// L c07000002 obj0 channel 0x48] — through the real gunfire path [M Destroy_CollisionHandler
/// 0x12bd48:42]. Sister: ch 0x48 off and the computer alive before the shots.
#[test]
fn smashing_computer1_writes_the_objective_channel_0x48() {
    let mut w = load();
    let (di, centre) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let di = sim.drones.props.destroy.iter().position(|d| d.placement == 14)
            .expect("Computer1 #14 exists [D]");
        let (lo, hi) = (sim.drones.props.destroy[di].lo, sim.drones.props.destroy[di].hi);
        (di, [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5])
    };
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.props.destroy[di].channel, 0x48,
        "the death channel = param 0 [D #14 p0 = 0x48]");
    assert!(!ch_on(&w, 0x48) && !w.scene.sim.as_ref().unwrap().drones.props.destroy[di].dead,
        "sister: the computer is whole and ch 0x48 off before the shots");
    // The stand-in transit to the bunker office floor (walkreal walks this office; the bounds
    // stand the computer at its [L port props] spot). The shots are the real input.
    place(&mut w, centre[0], centre[1] - 1.2, centre[2] + 2.2);
    let hit = shoot_at(&mut w, centre, 900, &|sim| sim.drones.props.destroy[di].dead);
    assert!(hit, "the gunfire smashes Computer1 [M Destroy_CollisionHandler 0x12bd48]");
    w.wait(5);
    assert!(ch_on(&w, 0x48), "the smash writes the objective channel 0x48 [D p0; M 0x12bd48:42]");
}

// ---------------------------------------------------------------------------
// Hurt (0xE4) — #13 "electrocute" under the arc, gate ch 0x8D [D]. The full chain
// battery lives in hurt_02_live.rs; this is the kind's entry.
// ---------------------------------------------------------------------------

/// The electrocute volume hurts only once the arc chain runs: on a fresh boot the gate
/// ch 0x8D is off and `Hurt_Update` parks disabled [M Hurt_Create 0x17e870 tail]; after the
/// cable light #392's smash writes ch 0x82 [D p0; M Destroy_CollisionHandler 0x12bd48:42], the
/// emitter's out 0x8D rises [M Emitter_Update 0x17b080:163] and the contact damage flows
/// [M Hurt_Update 0x17e5d0 → Player_DealWithObjHit 0x194458:94-101].
#[test]
fn the_electrocute_volume_hurts_only_once_the_arc_chain_runs() {
    let mut w = load();
    w.top_up_health = false;
    let (lo, hi) = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.hurts.len(), 1, "the census: one 0xE4 placement (#13) [D]");
        assert_eq!(sim.hurts[0].gate, 141, "the gate = param 2 = 0x8D [D #13]");
        (sim.hurts[0].lo, sim.hurts[0].hi)
    };
    let (cx, cz) = ((lo[0] + hi[0]) * 0.5, (lo[2] + hi[2]) * 0.5);
    let mid_y = (lo[1] + hi[1]) * 0.5;
    // Park Bond in the centre band every tick (the volume hangs over the cable gap), one clean
    // tick each — the hurt_02_live pattern.
    let mut tick_in_box = |w: &mut Walker| {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [cx, mid_y, cz];
            sim.bond.prev_pos = sim.bond.pos;
            sim.bond.vel = [0.0; 3];
            sim.bond.fall_speed = 0.0;
            sim.bond.grounded = false;
        }
        w.wait_enabled(600, "the bunker chat hold");
        w.wait(1);
    };
    // Sister — before the smash: the arc is off, standing in the volume hurts nothing.
    for _ in 0..10 { tick_in_box(&mut w); }
    let hp0 = w.scene.sim.as_ref().unwrap().drones.health.health;
    assert!((hp0 - 100.0).abs() < 0.01, "arc off: no contact damage [M 0x17e5d0 disabled bit], hp {hp0}");
    // The smash's channel write (the game's Destroy_CollisionHandler write [M 0x12bd48:42])
    // runs the chain: the emitter enables on ch 0x82 [M 0x17b080:94], its out 0x8D rises [M :163].
    let frame = w.scene.sim.as_ref().unwrap().drones.frame;
    w.scene.sim.as_mut().unwrap().drones.channels.set(0x82, 1, frame);
    for _ in 0..10 { tick_in_box(&mut w); }
    assert!(ch_on(&w, 141), "the smash → emitter chain drives ch 0x8D [M 0x17b080:163]");
    for _ in 0..15 { tick_in_box(&mut w); }
    let hp1 = w.scene.sim.as_ref().unwrap().drones.health.health;
    assert!(hp1 < hp0 - 1.0, "the arc hurts: hp {hp0} → {hp1} [M 0x194458:94-101]");
}

// ---------------------------------------------------------------------------
// Ladders (0x22) — #113 bath-pool, #114 bunker, #273 south-garden pond, #381
// pagoda pond [D].
// ---------------------------------------------------------------------------

/// The bunker ladder (#114 at (−26.87, 2.26, 42.23) [D]) attaches on the walk-in from its grab
/// side (the yaw window |AngleDiff| > π/2 [M Player_CollLadder 0x196118:20-22]) and the climb
/// input raises Bond. Sister: the substate is SS_WALK before the grab.
#[test]
fn the_bunker_ladder_attaches_on_the_walk_in_and_climbs() {
    let mut w = load();
    // A real walk from spawn #0 (−27.17, 1.64, 29.80) [D] to the ladder's grab side (the
    // walkreal route's own first leg): stand in front (the ladder's yaw −π, Bond faces +Z).
    w.walk_to(Wp::new(-26.87, 1.6, 40.4).r(1.2), "to the bunker ladder's grab side");
    let li = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.ladders.len()).min_by(|a, b| {
            let d = |l: &nf_game::ladder::Ladder| (l.centre[0] - -26.87).powi(2) + (l.centre[2] - 42.23).powi(2);
            d(&sim.ladders[*a]).total_cmp(&d(&sim.ladders[*b]))
        }).expect("07000002 has ladders")
    };
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_WALK,
        "sister: walking near the ladder does not attach by itself");
    let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let walk = Input { forward: 1.0, ..Default::default() };
    let mut attached = false;
    for _ in 0..300 {
        w.tick(walk, false);
        if w.scene.sim.as_ref().unwrap().bond.substate == SS_CLIMB { attached = true; break; }
    }
    assert!(attached, "walking into the ladder attaches [M Player_CollLadder 0x196118]");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.ladder, Some(li), "the grabbed ladder is the bunker one [D #114]");
    // The climb input raises Bond.
    for _ in 0..90 { w.tick(walk, false); }
    let y1 = w.scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(y1 > y0 + 0.5, "the climb raises Bond [M Player_Climb 0x196310], {y0} → {y1}");
}

// ---------------------------------------------------------------------------
// Wire (0x3E) — #187 hand-over-hand over the cable gap + ThirdIcon #384 (ctx 3) [D].
// ---------------------------------------------------------------------------

/// The cable wire (#187 at (−61.31, 8.08, −30.83) [D]) grabs on the JUMP with the third-icon
/// context [M Player_HandleJump 0x1916b0:118-147 — the press takes the nearest wire], hangs,
/// and the strafe axis crosses it [M Player_Wire 0x1977c0]. Sister: the E press never grabs a
/// wire (no class-0x2C arm in Player_Activate 0x1a5170 [M]).
#[test]
fn the_cable_wire_grabs_on_the_jump_and_crosses_on_the_strafe() {
    let mut w = load();
    let (centre, b_t, a_t) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let wv = sim.wires.first().expect("the cable wire #187 [D]").clone();
        let (a, b) = wv.travel_segment();
        (wv.centre, b, a)
    };
    // The ctx-3 third icon by the cable [D #384] — the jump-grab's context [M 0x1916b0 needs
    // +0x95F == 3, which the near icon writes on its 4-tick refresh [M 0x196e80:36-54]]. Stand
    // where the context actually reads 3 (the icon's own arm region is the game's cel set +
    // sphere [M 0x196e80] — try the icon's spots until the player's +0x95F reads 3).
    let mut spot: Option<[f32; 3]> = None;
    {
        let cands: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().icons.iter()
            .filter(|i| i.ctx == 3 && (i.centre[0] - centre[0]).abs() < 10.0 && (i.centre[2] - centre[2]).abs() < 10.0)
            .flat_map(|i| vec![i.centre, i.pos, [i.centre[0], i.centre[1] - 0.6, i.centre[2]], [i.centre[0], i.centre[1] - 1.6, i.centre[2]]])
            .collect();
        for c in cands {
            place(&mut w, c[0], c[1] - 1.1, c[2]);
            w.wait(12); // the icon's 4-tick refresh [M 0x196e80:36-54]
            // The grab runs from the WALK body [M 0x1916b0 — grounded_history != 0]: the spot
            // must stand Bond on the roof with the context live.
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.icon_ctx == 3 && sim.bond.grounded && sim.bond.jump == 0 { spot = Some(c); break; }
        }
    }
    let spot = spot.expect("the cable's ctx-3 icon arm reads context 3 somewhere by the wire [D #384]");
    // The E sister first: no grab.
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.0;
    w.tick(Input::default(), true);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_WALK,
        "sister: E never grabs a wire [M 0x1a5170 has no class-0x2C arm]");
    // The jump grab [M 0x1916b0:118-147].
    w.tick(Input { jump: true, ..Default::default() }, false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_WIRE,
        "the jump at the cable (icon context 3 [D #384]) must grab it");
    for _ in 0..140 { w.tick(Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_WIRE, "still on the wire after the grab pose");
    // The strafe axis moves Bond along the wire [M Player_Wire 0x1977c0] toward one of the two
    // ends (which sign is which end is the wire's own axis — the player just holds the stick);
    // try one sign, then the other, until Bond stands within reach of an end.
    let z0 = w.scene.sim.as_ref().unwrap().bond.pos[2];
    let at_end = |p: [f32; 3]| (p[0] - a_t[0]).abs() < 1.5 && (p[2] - a_t[2]).abs() < 1.5
        || (p[0] - b_t[0]).abs() < 1.5 && (p[2] - b_t[2]).abs() < 1.5;
    let mut reached = false;
    for sign in [-1.0f32, 1.0] {
        for _ in 0..450 {
            w.tick(Input { strafe: sign, ..Default::default() }, false);
            if at_end(w.scene.sim.as_ref().unwrap().bond.pos) { reached = true; break; }
        }
        if reached { break; }
    }
    assert!(reached, "the strafe crossing carries Bond to a wire end [M 0x1977c0], from z {z0} to {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
}

// ---------------------------------------------------------------------------
// Sound (0xF9) / Music (0xFB) triggers — #434-437/#484 sound, #378 (ch 0x17 →
// event 31) and #379 (ch 0x19 → event 32) music [D].
// ---------------------------------------------------------------------------

/// The music triggers register with the disc params [D #378/#379] and fire their event while
/// their channel is on [M MusicTrigger_Update 0x1b31a0:19-27]: the ch-0x17 writer is TouchOnce
/// #342's out [D], so the real walk onto that plate drives the trigger. Sister: the countdown
/// rests disarmed while ch 0x17 is off.
#[test]
fn the_music_triggers_carry_the_disc_params_and_fire_on_their_channel() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let ts: Vec<(u16, u32)> = sim.drones.worldobjs.music_triggers.iter()
            .map(|t| (t.ch_on, t.event_id)).collect();
        assert!(ts.contains(&(0x17, 31)), "the garden's music trigger (ch 0x17 → event 31) exists [D #378]");
        assert!(ts.contains(&(0x19, 32)), "the west wing's music trigger (ch 0x19 → event 32) exists [D #379]");
        assert!(sim.drones.worldobjs.music_triggers.iter().all(|t| t.countdown == 0 && !t.playing),
            "sister: disarmed while the channels are off");
    }
    // The real walk onto plate #342 (out 0x17 [D]) — the stand-in transit to its floor, then the
    // walk-over raises the channel and the trigger arms.
    let c = {
        let sim = w.scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter()
            .find(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.out == 0x17)
            .expect("the ch-0x17 plate #342 [D]");
        [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
    };
    place(&mut w, c[0], c[1] - 0.4, c[2]);
    let mut armed = false;
    for _ in 0..60 {
        w.tick(Input::default(), false);
        if ch_on(&w, 0x17) { armed = true; break; }
    }
    assert!(armed, "the walk onto the plate raises ch 0x17 [D #342; M Trigger_Hit 0x1b1fc8]");
    let mut beat = false;
    for _ in 0..40 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let mt = sim.drones.worldobjs.music_triggers.iter().find(|t| t.ch_on == 0x17).unwrap();
        if mt.countdown != 0 || mt.playing { beat = true; break; }
    }
    assert!(beat, "the trigger arms its 30-tick countdown on the channel [M 0x1b31a0:19-27]");
}

// ---------------------------------------------------------------------------
// Hints (0x33) — #492 (label 0x2000009, the exit E-hint) and #494 (label
// 0x2000057, the bunker ladder hint), stage param 2 = 3 [D].
// ---------------------------------------------------------------------------

/// The hint markers register with the disc labels and the stage gate [D #492/#494; M
/// Hint_Create 0x1b3248:12-25], and walking into #494's sphere fires it once: the message lands
/// on the HUD pane and the marker is spent [M Hint_Update 0x1b3380:24-42]. Sister: the second
/// marker stays unfired until its own walk.
#[test]
fn the_hint_markers_carry_the_disc_labels_and_fire_on_the_walk_in() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.hints.len(), 2, "the census: two 0x33 placements [D #492/#494]");
        assert!(sim.hints.iter().any(|h| h.label == 0x2000009), "the exit hint's label [D #492 p0]");
        assert!(sim.hints.iter().any(|h| h.label == 0x2000057), "the bunker ladder hint's label [D #494 p0]");
        assert!(sim.hints.iter().all(|h| h.stage == 3), "the stage gate param [D p2 = 3]");
    }
    // #494 sits in the start yard at (−24.72, 0.70, 30.32) [D] — a real walk from the spawn.
    let hpos = w.scene.sim.as_ref().unwrap().hints.iter()
        .find(|h| h.label == 0x2000057).map(|h| h.centre).expect("#494");
    w.walk_to(Wp::new(hpos[0], 0.7, hpos[2]).r(1.0), "into the bunker-ladder hint's sphere");
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let hi = sim.hints.iter().position(|h| h.label == 0x2000057).unwrap();
    assert!(sim.hint_fired[hi], "the walk-in fires the hint once [M Hint_Update 0x1b3380:24-42]");
    let other = sim.hints.iter().position(|h| h.label == 0x2000009).unwrap();
    assert!(!sim.hint_fired[other], "sister: the far marker stays unfired [D #492 at the exit]");
}

// ---------------------------------------------------------------------------
// Sensors / cameras (0x28) — #406 kitchen, #423 pagoda; group 2 (p6), swept,
// out ch 0 [D — nothing to raise; the monitor group feeds #323].
// ---------------------------------------------------------------------------

/// The kitchen camera (#406 at (−11.50, 7.68, −42.45) [D], range param 2 = 6 [D]) sees Bond
/// when he stands in its cone: `Sensor_Update` 0x1af618's cone+LOS gate counts the in-cone
/// ticks (`seen`), alerting past 0x1e [M sensor.rs ALERT_SEEN]. Sister: behind the camera the
/// cone never sees him.
#[test]
fn the_kitchen_camera_sees_bond_in_its_cone() {
    let mut w = load();
    let (si, spos, dir) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let si = sim.drones.sensors.list.iter().position(|s| s.monitor_id == 2)
            .expect("camera #406 (monitor group 2) [D]");
        let s = &sim.drones.sensors.list[si];
        let m = s.mount_matrix();
        (si, s.pos, [m[2][0], m[2][1], m[2][2]])
    };
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.sensors.list[si].out, 0,
        "the camera's out ch is 0 — nothing to raise [D #406 p8]");
    // The stand-in transit to the kitchen floor; the detection is the sensor's own sweep.
    let seen0 = w.scene.sim.as_ref().unwrap().drones.sensors.list[si].seen;
    let mut saw = false;
    for r in [3.0f32, 4.0, 5.0] {
        place(&mut w, spos[0] + dir[0] * r, spos[1] - 1.4, spos[2] + dir[2] * r);
        for _ in 0..90 {
            w.tick(Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.sensors.list[si].seen > seen0 { saw = true; break; }
        }
        if saw { break; }
    }
    assert!(saw, "the camera's cone sees the standing player [M Sensor_Update 0x1af618]");
    // Sister: behind the camera the cone never counts.
    place(&mut w, spos[0] - dir[0] * 3.0, spos[1] - 1.4, spos[2] - dir[2] * 3.0);
    let s0 = w.scene.sim.as_ref().unwrap().drones.sensors.list[si].seen;
    w.wait(90);
    assert!(w.scene.sim.as_ref().unwrap().drones.sensors.list[si].seen <= s0,
        "sister: behind the camera the cone sees nothing [M 0x1af618's cone test]");
}
