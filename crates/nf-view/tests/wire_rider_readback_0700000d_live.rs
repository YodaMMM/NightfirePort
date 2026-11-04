//! wire-rider-readback (0700000d): what the guards' motion read reads off a WIRE rider.
//!
//! The game zeroes the player's move quad (BLData+0x10 vec3 and the BLData+0x30 rotation delta)
//! on EVERY tick the player state (+0xF4) is 1 — alive & in control — BEFORE the substate switch
//! (which reads the DIFFERENT field +0xF6) [M Player_Update 0x193600:128-134; raw MIPS: the gate
//! load `lh $v1, 0xf4($s3)` @0x1937e0 vs the switch load `lh $v1, 0xf6($s3)` @0x193938; the
//! state-0→1 promotion arm :312-315; docs/fields/obj-tag-player.md:59]. `Player_Wire` 0x197658
//! writes NO +0x10 lane (it zeroes BL+0x50 :105 and moves Bond by direct obj+0x30/34/38 writes)
//! and the mount runs `Player_ClearInertia` 0x195e08 (zeroes BL+0x10/0x20/0x30/0x40) — so a wire
//! rider's quad is ZERO on every ride tick, and the guards' motion read
//! `|(BL+0x10, 0, BL+0x18)| != 0 → moving` [M DroneWeap_DoOpponentTargetting 0x179a48:34-41,
//! the Y lane forced 0] reads the rider STILL, feeding DroneWeap_DoBulletAccuracy's
//! TargetMoving/FirstMoved/FirstStopped factors [M 0x179740:52-99]. The walk ticks overwrite the
//! quad by absolute store [M Player_Move 0x191dd0:166-170] and the climb's own pushes are ADDs on
//! the zeroed quad [M 0x196310:56,90] — both unchanged by the zero-every-tick law.
//!
//! Pinned on the real input path from the level start (no teleports): the walkreal-0d route legs
//! to the ladder #435 top, then (a) stationary Bond reads STILL (quad [0,0], acquired guards'
//! `target_motion.moving == false`), (b) a walking Bond reads MOVING, (c) the wire rider's quad
//! stays [0,0] on EVERY SS_WIRE tick (the decompile law — on the old climb-only zeroing this
//! retained the pre-grab walk lanes and read MOVING), and the ride lands on the rotate crane's
//! deck. Opt-in via `NF_ISO`.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

/// The enemies that hold Bond as their opponent (the class-3 fold [M 0x179a48:31-41]): their
/// `target_motion` is the DoOpponentTargetting readout.
fn acquired(w: &Walker) -> Vec<usize> {
    w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.opp_bond && d.enabled && d.present && !d.corpse && d.health > 0.0)
        .map(|(i, _)| i).collect()
}

#[test]
fn the_wire_rider_reads_still_and_a_still_bond_reads_still() {
    let iso = iso();
    let cold = nf_view::load_with_carry(&iso, "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x23), "the start trigger #436 fires ch 0x23 [D]");

    // The walkreal-0d route legs to the ladder top, real input (the crossfire pool is the
    // walker's own top-up here — this pin is about the move quad, not survival).
    w.walk_to(Wp::new(180.6, 11.2, -208.8).r(0.9), "the BodyArmourHeavy #525");
    assert!(w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the platform drop"), "the drop must reach the ground");
    for (at, r, label) in [
        ([188.0f32, 0.5, -228.0], 2.0, "the blind strip east 1"),
        ([196.0, 0.5, -228.5], 2.0, "the blind strip east 2"),
        ([204.0, 0.5, -229.0], 2.0, "the blind strip east 3"),
        ([212.0, 0.5, -230.0], 1.5, "the blind strip east 4"),
        ([215.0, 0.8, -233.2], 1.2, "east of the crane leg"),
        ([213.2, 1.5, -234.8], 0.8, "at the ladder base"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    assert!(w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb ladder #435"), "the ladder climb must top out");
    assert!(bond_pos(&w)[1] > 16.0, "the ladder tops at y ~17.37 [D]");

    // (a) STATIONARY: 140 idle ticks — the eased walk lanes decay to the dead-zone snap and the
    // zeroed quad reads STILL.
    for _ in 0..140 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.move_local, [0.0, 0.0], "a stationary Bond's quad is the zeroed pair [M 0x193600:133]");
        let read = acquired(&w).into_iter().filter(|&i| sim.drones.list[i].target_motion.moving).count();
        eprintln!("[still] acquired {} (of {}), reading moving: {}", acquired(&w).len(), sim.drones.list.len(), read);
        assert_eq!(read, 0, "a stationary Bond reads STILL on every acquired guard [M 0x179a48:41]");
    }

    // (b) WALKING: hold forward — the eased walk lanes store non-zero [M 0x191dd0:166-170] and
    // the acquired guards read MOVING. Walk away from the wire edge and back (no net travel).
    for k in 0..60 {
        let f = if k < 30 { 0.4 } else { -0.4 };
        w.tick(nf_game::player::Input { forward: f, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.move_local[0] != 0.0 || sim.bond.move_local[1] != 0.0,
            "a walking Bond's quad carries the eased walk lanes [M 0x191dd0:166-170]");
        let read = acquired(&w).into_iter().filter(|&i| sim.drones.list[i].target_motion.moving).count();
        eprintln!("[walking] acquired {}, reading moving: {}", acquired(&w).len(), read);
        assert!(read > 0, "a walking Bond reads MOVING [M 0x179a48:41]");
    }

    // (c) THE RIDER: face the wire's A end, settle, jump — the grab replaces the jump with the
    // icon #297 in context [M Player_HandleJump 0x1916b0:118-147] — then the quad must be ZERO
    // on every SS_WIRE tick (the decompile law), and the ride lands on the rotate crane's deck.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
        for _ in 0..30 {
            let sim = w.scene.sim.as_mut().unwrap();
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 0.3, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    }
    let mut grabbed = false;
    for k in 0..80u32 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { jump: (60..=63).contains(&k), forward: 0.3, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if sim.bond.substate == nf_game::player::SS_WIRE { grabbed = true; break; }
    }
    assert!(grabbed, "the wire #271 jump-grab must fire from the ladder top (icon #297 in context) [D]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[grab] quad {:?}", sim.bond.move_local);
    }
    // The ride: every SS_WIRE tick the quad is the zeroed pair [M 0x193600:133 + 0x197658 writes
    // no lane], the readback follows, and strafe +1 (mode 7) carries Bond to the deck.
    let mut ride_ticks = 0u32;
    let mut quad_violations = 0u32;
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 2600 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.bond.substate != nf_game::player::SS_WIRE { break; }
                if sim.bond.move_local != [0.0, 0.0] { quad_violations += 1; }
                if ride_ticks == 40 {
                    let read = acquired(&w).into_iter().filter(|&i| sim.drones.list[i].target_motion.moving).count();
                    eprintln!("[ride] acquired {}, reading moving at tick 40: {}", acquired(&w).len(), read);
                    assert_eq!(read, 0, "the wire rider reads STILL on every acquired guard [M 0x179a48:34-41 + 0x193600:133]");
                }
            }
            w.tick(nf_game::player::Input { strafe: 1.0, ..Default::default() }, false);
            ride_ticks += 1;
        }
    }
    assert!(ride_ticks > 100, "the ride must carry Bond ({ride_ticks} SS_WIRE ticks)");
    assert_eq!(quad_violations, 0, "the rider's quad stays zeroed on EVERY SS_WIRE tick [M 0x193600:128-134 (state gate) + 0x197658]; got {quad_violations} non-zero ticks — the climb-only zeroing retained the pre-grab walk lanes");
    // The auto-drop's fall settles onto the deck (polled).
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 500 {
            let sim = w.scene.sim.as_ref().unwrap();
            if w.total_ticks - start > 30 && sim.bond.grounded { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
    }
    {
        let p = bond_pos(&w);
        assert!((p[0] - 199.25).abs() < 2.5 && (p[2] + 260.16).abs() < 2.5 && (p[1] - 14.16).abs() < 1.5,
            "the wire ride must land on the rotate crane's deck (199.25, 14.16, −260.16), got {p:?}");
    }
}
