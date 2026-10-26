//! The gondola's windows must shoot out. The 07000008 cabin's 16
//! windows are script entities: the ride script 0x0600004C (a kind-7 entry of the LEVEL BIN)
//! creates them with op-7 Entity streams 7-18/23-26, obj_type 0x20, parented to stream 2, params
//! [2, 1, 0] [D decode — gondola_windows_script.rs] — `Script_CreateEntity` 0x1d8968:120-123's
//! `Break_Create` arm. They ride the carriage (parented to the stream that descends it [M
//! `Script_Interp` 0x1d9a08:94-112]) and the bullet's collision tests the pane's CURRENT local
//! space [M `Collide_Intersect` 0x1ebaa8:157-177]. The port registers them as `Breaks` script
//! panes (kind/health/filter from the params, bounds re-derived from the live matrix each tick).
//!
//! The REAL fire path (the equipped gun, `step_bond`'s trigger, the bullets, the `bullet_hit`
//! pass): on the ride Bond fires at a window — the pane drains [M `Break_Update` 0x12bfd8:36-37],
//! health 1.0 [D params] − damage < 0 → `Break_Kill`: dead bit + kind-2 sound 0x2E [M
//! 0x12b970:60-76], NO pieces (psiCreateBreakableData is placement-only [M 0x1d04f0:70-73]), and
//! the window's object is deleted — it leaves the collision set and the draw [M 0x12b970:60].
//! Sister: the same aim with the trigger never pressed breaks nothing.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The ride boarding (the walkreal-covered stretch stands in — walkreal_07000008_live walks the
/// whole route to the floor plate): the chain latches set cold [D placements; M Player_RamSave],
/// Bond ON the cable-car floor plate #236 (its box x 111.10..112.92, y 26.77..29.21,
/// z −128.10..−126.75 [D the trigger's tri bbox]) → ch 4; the briefcase ch 0x4A stands in for the
/// walked chain [M Pickup_Handler 0x18e650:330-333] → mux #235 → ch 1 starts SP #295 [D].
fn board_the_ride(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Bond on the plate: the settle drops him onto the car floor (the plate's Touch #236 is
        // mask 8 = the player walk-in [D #236]).
        sim.bond.pos = [112.0, 28.6, -127.4];
        for _ in 0..90 { sim.step_bond(Input::default(), false, false, false); }
        let f = sim.drones.frame;
        sim.drones.channels.set(0x4a, 1, f);
    }
    let mut ch4 = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(4) { ch4 = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(ch4, "the cable-car floor plate fired ch 4 [D #236 mask 8]");
    let mut ride = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(1) { ride = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(ride, "mux #235 (0x4a AND 4) fired ch 1 — the ride script starts [D]");
}

/// An armed GUN through the real weapon-cycle input (the 09 pattern — the cold loadout's draw
/// loop): a gun's weapon word 0x0008 passes the base 0x106 filter [L dump]; the laser (0x0002)
/// and the watch (0x0100) classes are dropped by Collide_FilterBullets [M 0x1ece30:24-33] —
/// they cannot break glass, so the test must hold a gun-class weapon.
fn arm_a_gun(w: &mut Walker) {
    let word = |sim: &nf_view::scene::Sim, id: u8| sim.weapons.records.get(id as usize)
        .map_or(0u16, |r| u16::from_le_bytes([r[0x10], r[0x11]]));
    for _ in 0..14 {
        let (c, s, wp, wd) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.gun.clip, sim.gun.spare, sim.gun.weapon, word(sim, sim.gun.weapon))
        };
        if c + s > 3 && wd & 0x106 == 0 {
            eprintln!("[gondola] armed weapon {wp:#x} word {wd:#x} clip {c} spare {s}");
            return;
        }
        w.tick(Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Input::default(), false); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    panic!("no gun-class weapon with ammo on 08 (weapon {:#x} word {:#x})",
        sim.gun.weapon, word(sim, sim.gun.weapon));
}

#[test]
fn firing_at_a_ride_window_breaks_it_dead_bit_glass_sound_entity_gone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    board_the_ride(&mut w);
    // The panes: their streams start with the script; wait for the Breaks sync to register them.
    // (The [D] exact 16 is pinned in nf-formats gondola_windows_script.rs; the live set churns —
    // a stream's later Entity command of another class deletes+recreates its window entity
    // [M 0x1d8968:92-102], and the pane rows follow.)
    let mut n_panes = 0usize;
    for _ in 0..1500 {
        n_panes = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter().filter(|b| b.script_obj.is_some()).count();
        if n_panes >= 14 { break; }
        w.tick(Input::default(), false);
    }
    assert!(n_panes >= 14, "the windows registered as script panes [D decode: 16 streams]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for b in sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some()) {
            assert_eq!(b.kind, 2, "kind 2 glass [D params; M 0x12c230:21]");
            assert_eq!(b.health, 1.0, "health (f32)(i32)1 = 1.0 [D params; M 0x12c230 disasm 0x12c2a8-0x12c2c4]");
            assert_eq!(b.filter, 0, "param 0 → filter 0 [M 0x12c230:29-35]");
        }
    }
    arm_a_gun(&mut w);
    // Lock ONE target pane (the nearest at arm time) and hold the aim on it — the nearest pane
    // alternates sides as the cabin descends, and a chasing aim never settles. The pane is
    // tracked by its object index; the aim rides the INPUT (turn/look, the walkreal-08 fire
    // pattern), the trigger in bursts (the port's gun fires on the edge).
    let target_obj = {
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some() && !b.dead)
            .min_by(|a, c| {
                let d = |b: &nf_game::breakable::Break| (b.lo[0] - head[0]).powi(2) + (b.lo[1] - head[1]).powi(2) + (b.lo[2] - head[2]).powi(2);
                d(a).partial_cmp(&d(c)).unwrap()
            }).and_then(|b| b.script_obj).expect("a target pane")
    };
    eprintln!("[gondola] target pane obj {target_obj}");
    // Converge the aim WITHOUT firing (the aim rides the input; the pane re-picked on churn),
    // then press the trigger in bursts and watch `break_kills` — the kill's row and entity are
    // both gone the same tick (the deletion), so the kill queue is the observable.
    let mut target_obj = target_obj;
    let mut paths_at_kill = 0usize;
    let (kills_before, panes_before) = loop {
        let sim = w.scene.sim.as_mut().unwrap();
        if !sim.drones.breaks.list.iter().any(|b| b.script_obj == Some(target_obj) && !b.dead) {
            let head = sim.bond.head();
            target_obj = sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some() && !b.dead)
                .min_by(|a, c| {
                    let d = |b: &nf_game::breakable::Break| (b.lo[0] - head[0]).powi(2) + (b.lo[1] - head[1]).powi(2) + (b.lo[2] - head[2]).powi(2);
                    d(a).partial_cmp(&d(c)).unwrap()
                }).and_then(|b| b.script_obj).expect("a live pane");
        }
        let head = sim.bond.head();
        let Some(br) = sim.drones.breaks.list.iter().find(|b| b.script_obj == Some(target_obj) && !b.dead)
            else { w.tick(Input::default(), false); continue };
        let target = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        let to = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let pitch_err = desired_pitch - sim.bond.pitch;
        let done = yaw_err.abs() < 0.04 && pitch_err.abs() < 0.02;
        let input = Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.05, 0.05), ..Default::default() };
        let kb = sim.drones.break_kills.len();
        let pb = sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some() && !b.dead).count();
        drop(sim);
        w.tick(input, false);
        if done { break (kb, pb); }
    };
    eprintln!("[gondola] aim converged on obj {target_obj} (kills {kills_before}, live panes {panes_before})");
    // Press: up to 8 edge presses, each a fresh shot at the converged aim.
    let mut new_kills: Vec<(usize, u8, u16)> = Vec::new();
    for _ in 0..80 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            new_kills = sim.drones.break_kills[kills_before..].to_vec();
            if !new_kills.is_empty() {
                // The kill tick's flight segments are still on the books (the per-tick clear runs
                // at the NEXT tick's bullet update): the kill came from the bullet pass.
                paths_at_kill = sim.drones.bullet_paths_len();
                break;
            }
            let head = sim.bond.head();
            let input = if let Some(br) = sim.drones.breaks.list.iter().find(|b| b.script_obj == Some(target_obj) && !b.dead) {
                let target = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
                let to = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
                let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
                Input { turn: walker::wrap_pi(to[0].atan2(to[2]) - sim.bond.yaw).clamp(-0.08, 0.08),
                        look: ((to[1] / l).asin() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch).clamp(-0.05, 0.05), ..Default::default() }
            } else { Input::default() };
            drop(sim);
            w.tick_fire(input);
        }
        for _ in 0..7 { w.tick(Input::default(), false); }
    }
    eprintln!("[gondola] new kills {new_kills:?}");
    assert!(!new_kills.is_empty(), "a window broke under the real fire [M Break_Kill 0x12b970]");
    for &(_, k, s) in &new_kills {
        assert_eq!((k, s), (2, 0x2E), "every kill is the glass one: kind 2, sound 0x2E [M 0x12b970:64-76]");
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let (obj, _, _) = new_kills[0];
    assert!(sim.drones.scripts.objects.list.get(obj).is_none_or(|o| o.is_none()),
        "the window's object is deleted — it left the collision set and the draw [M 0x12b970:60]");
    assert!(sim.drones.breaks.list.iter().all(|b| b.script_obj != Some(obj)),
        "the dead row dropped at the sync (the pane left the world with its object)");
    assert!(sim.drones.breaks.shatter.shards.is_empty() && sim.drones.breaks.shatter.debris.is_empty(),
        "a script pane spawns NO pieces (psiCreateBreakableData is placement-only [M 0x1d04f0:70-73])");
    // The kill came from the bullet pass: the flight segments were on the books on the kill tick.
    // (The old pin here — paths non-empty 7 ticks after the kill — held on the pre-stairwell
    // flight only through the accidental double-pierce of the pass-flagged glass static in front
    // of the pane: the game's sorted walk [M Collide_Update 0x1ea398:216-229] passes the glass
    // class-0x10 surface AND the kind-2 pane, then the round dies on the solid far wall behind —
    // the whole flight is one 2.5 m tick. The drain-without-stop is the game's glass mechanic:
    // the pass appends the contact to the pane's obj+0xD0 chain and `Break_Update` 0x12bfd8:36-37
    // drains it, while `Bullet_CollisionHandler` 0x12ce80's glass arm (+0x50 & 0x3f == 0x10 under
    // the +0x6c & 0x800 gate) keeps the round flying.)
    assert!(paths_at_kill > 0, "the kill came from the bullet pass (flight segments on the kill tick)");
}

/// Sister: the same ride, the same aim, the trigger never pressed — no pane dies.
#[test]
fn the_aim_without_fire_leaves_the_windows_intact() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    board_the_ride(&mut w);
    let mut n_panes = 0usize;
    for _ in 0..1500 {
        n_panes = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter().filter(|b| b.script_obj.is_some()).count();
        if n_panes == 16 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(n_panes, 16, "the 16 windows registered [D decode]");
    arm_a_gun(&mut w);
    // Aim at the nearest pane without ever pressing the trigger.
    for t in 0..240u32 {
        let sim = w.scene.sim.as_mut().unwrap();
        let head = sim.bond.head();
        if let Some(target) = sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some() && !b.dead)
            .map(|b| ([(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0],
                      (b.lo[0] - head[0]).powi(2) + (b.lo[1] - head[1]).powi(2) + (b.lo[2] - head[2]).powi(2)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(c, _)| c) {
            let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
            let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        }
        w.tick(Input::default(), false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some()).all(|b| !b.dead),
        "aiming does not break the windows");
    assert!(sim.drones.break_kills.is_empty(), "no kill effects without the fire");
}

/// The moving-bounds law: mid-ride the panes ride the carriage — their rows' bounds follow the
/// live matrix (a descent of metres, not the parked create spot) [M 0x1d9a08:94-112].
#[test]
fn the_window_bounds_follow_the_carriage_down_the_ride() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    board_the_ride(&mut w);
    // The first reading once the panes are up.
    let mut first: Option<([f32; 3], [f32; 3])> = None;
    for _ in 0..1500 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.breaks.list.iter().filter(|b| b.script_obj.is_some()).count() == 16 {
            first = sim.drones.breaks.list.iter().find(|b| b.script_obj.is_some())
                .map(|b| (b.lo, b.hi));
            break;
        }
        w.tick(Input::default(), false);
    }
    let (lo0, hi0) = first.expect("the panes registered");
    // Ride on; the carriage descends ~0.03/tick (53 z over 1420 ticks, y 32.4 → 18.0 [D stream 1]).
    for _ in 0..600 { w.tick(Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let (lo1, hi1) = sim.drones.breaks.list.iter().find(|b| b.script_obj.is_some()).map(|b| (b.lo, b.hi)).unwrap();
    let drop = hi0[1] - hi1[1];
    eprintln!("[gondola ride] pane hi.y {0:.2} → {1:.2} (drop {2:.2})", hi0[1], hi1[1], drop);
    assert!(drop > 1.0, "the pane's bounds rode the carriage down (dropped {drop:.2} y in 600 ticks)");
    assert!(lo1[1] < lo0[1] && hi1[1] < hi0[1], "the whole pane moved with the ride [M 0x1d9a08]");
    assert!(!sim.drones.channels.on(0x7c), "the plunge never fired on the ride [D §6]");
}
