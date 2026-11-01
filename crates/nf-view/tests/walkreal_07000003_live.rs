//! walkreal: 07000003 "Alpine Escape" (Castle C) walked with the player's own movement — no
//! teleports after the spawn. The route (docs/levels/07000003.md + the disc dump):
//!   the 0x24 start (−8.85, 6.44, −23.19; the 0x4d objective-reveal touch fires under Bond) →
//!   the front gate (door[0] at (14.7, 6.6, −35.4) starts locked behind ch 0x31; the E-plate
//!   kind-1 mask-4 at (11.8, 6.0, −35.6) writes 0x31, which unlocks the door and plays the
//!   gate SS #502, ch 0x31, 2 keys — the "key-gated route") → the 0x21 guard-enable plates →
//!   the servants (skin 0x0500002b ×3, team 3, death channel 0x60 = protected) → the east lawn →
//!   the Load Point #451 (18.11, 6.65, 16.05; E, key gate ch 0x52).
//!
//! The objective channels: the disc's mux family writes the gate —
//!   AND(0x4f, 0x50) → 0x51 (obj1 "Rescue Mayhew's servants") and AND(0x4e, 0x51) → 0x52
//!   (obj2's reveal AND the exit's key gate) [D the kind-3 triggers, out 0x51 list 4f,50 and
//!   out 0x52 list 4e,51]. The LEAF writers are on the disc and ported:
//!   - 0x4e — the Dragon Safe key: E on the case (SP #110, script 0x06000563, mask 4 use-key
//!     [M SP_Activate 0x1dc338:82]) spawns the kind-2 pickup with +0x28 = 0x4e [D script stream 1;
//!     M Pickup_Handler 0x18e650:330-333 writes it on take];
//!   - 0x4f/0x50 — the servants' rescue channels, each placement's param 6 = the game's
//!     Drone+0x135 (#389 carries 0x4f, #388 0x50, #422 none [D probe; 07000002's servants carry
//!     0x49/0x4A the same way [D]]): the enable plate (p3) starts the HOSTAGE cower [D
//!     DroneTypeSettings row 5 start 0xa via dtype 0xe + prop 0x4e; M 0x14c938], the captor — the
//!     nearest armed alive guard at the entering tick [M 0x144030; the port's Drones tick] — dying
//!     moves the Hostage UPDATE to HostageSaved [M 0x15cb78:24-36], whose ENTER raises the channel
//!     [M 0x15ccf0 case 1 → DroneFunc_HostageSaved 0x1441c0:11-18].
//!   0x53 (obj2's watch) is written by 07000004's story script [D 07000003-audit] — in this level
//!   obj2 can only reach state 3 (announced), and the mission ENDS on the exit trigger, not on
//!   mission completion. The short route below still hands the leaves at the moments the level
//!   offers (documented stand-in, not the whole-mission walk); the full-route test drives every
//!   leaf through the real input.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission-table row and objective indices for this level [L ee_start_05 @ 0x2a3f18 via
/// mission::c07000003]: obj0 0x4e "Retrieve Dragon Safe contents", obj1 0x51 "Rescue Mayhew's
/// servants", obj2 0x53 "Find door leading to Mayhew" (enable 0x52).
fn mission_row() -> (usize, usize, usize, usize) {
    let li = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0003).expect("the 07000003 mission record");
    (li, 0, 1, 2)
}

#[test]
fn walk_the_alpine_route_to_the_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(60);
    {
        // The spawn's objective-reveal touch (out 0x4d at (−9.0,−23.2) [D dump]) — both first
        // objectives' enable channel, so the monitor announces them [M 0x1827d8:55-67].
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, o0, o1, _) = mission_row();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal touch fires under Bond [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced through the real monitor");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced through the real monitor");
    }

    // The route: north-east across the courtyard to the Load Point. The nav graph fills between
    // the anchors; the guard-enable plates (0x21/0x23/0x24/0x25) fire under Bond's feet.
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
        ("the east lawn", [22.0, 5.9, 5.0]),
        ("the exit lawn", [18.5, 5.9, 13.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }

    // 6. The exit: E on the Load Point #451 without ch 0x52 refuses [M Trigger_Activate 0x1b2318:76-90].
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E without the key");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x52), "neither objective fired, so the AND(0x4e,0x51) has not written 0x52 [D]");
        assert_eq!(sim.drones.level_exit, None, "the key gate refuses the exit without ch 0x52 [M 0x1b2318:76-90]");
    }
    // The short route's stand-in: hand the LEAF channels at the moments the level offers — the
    // real writers (the servants' p6 rescue channels and the safe key's take) are driven by the
    // full-route test below; 0x51 and 0x52 are the mux family's OUTPUTS (the AND triggers drive
    // them every tick, so a direct hand is overwritten [M 0x1b1a78's unconditional AND write; the
    // recon]). Everything downstream is the disc's own mux family: AND(0x4f,0x50) writes 0x51
    // [D kind-3 out 0x51 list 4f,50], then AND(0x4e,0x51) writes 0x52 [D out 0x52 list 4e,51],
    // which announces obj2 (enable 0x52) and unlocks the exit.
    let sim = w.scene.sim.as_mut().unwrap();
    for &c in &[0x4eu32, 0x4f, 0x50] { sim.drones.channels.set(c, 1, sim.drones.frame); }
    drop(sim);
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, _, o1, o2) = mission_row();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced when its enable channel 0x52 fires [M 0x1827d8:55-67]");
    }
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E with the key");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 through the real reload path");
    eprintln!("[03] walked to the exit in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// The full play: every objective, every leaf written by the disc's own writers under real input
/// (walking, turning, E, real fire) — the servants rescued through their enable plates + captor
/// kills, the Dragon Safe opened and its key taken, the exit fired. No channel or state pokes.
#[test]
fn walk_the_full_route_and_complete_every_objective() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    let (li, o0, o1, o2) = mission_row();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
    }
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced [M 0x1827d8:55-67]");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced [M 0x1827d8:55-67]");
    }

    // The route legs up to the servants' ground (the guard-enable plates 0x21/0x23/0x24/0x25 fire
    // under Bond's feet as he crosses them [D the TouchOnce census]).
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }

    // 1. "Rescue Mayhew's servants" — the leaves 0x4f (servant #389) and 0x50 (#388) are the
    //    placements' param 6 [D probe]. Each servant's enable plate (p3) enables the servant AND
    //    the guards sharing the channel [D], the servant starts the HOSTAGE cower (DroneTypeSettings
    //    row 5 start 0xa) and the captor binds. Killing the captor completes the save: Hostage
    //    UPDATE → HostageSaved ENTER → DroneFunc_HostageSaved writes switch_channels[+0x135 = p6]
    //    [M 0x15cb78:24-36, 0x15ccf0 case 1, 0x1441c0:11-18].
    //    The enable channel 0x21 hangs off TWO kind-1 plates [D two out-0x21 placements]. The east
    //    twin (centre 14.3, −33.3; footprint x 13.1..15.4, z −35.2..−31.5 [D the trigger lo/hi])
    //    is cut off from every standable lane by the garden moat ditch: the straight walk from the
    //    gate-plate lawn burns the 6000-tick cap and the nav route parks on the ledge above (y
    //    9.37 — the prior session's stuck dump). The WEST twin (centre 8.98, −34.35; footprint x
    //    7.27..10.69, z −35.05..−33.65 [D the trigger lo/hi]) IS crossed on real input: the west
    //    lane's approach to the gate-plate lawn stands inside it at (9.8, 6.37, −34.67) and ch 0x21
    //    fires (a plate probe run on the same legs as below). So the plate is fired
    //    BEFORE the servant walk — #389 is already HOSTAGE with its captor bound when Bond arrives
    //    [M 0x144030's bind at the HOSTAGE entering tick].
    let sim = w.scene.sim.as_ref().unwrap();
    let plate0x25 = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x25).map(|t| t.centre())
        .expect("servant #388's enable plate");
    let servants: Vec<(usize, [f32; 3], u8)> = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.skin & 0xffff == 0x2b && d.team == 3)
            .map(|(i, d)| (i, d.pos, d.second_channel)).collect()
    };
    assert_eq!(servants.len(), 3, "three protected servants [D the dump]");
    drop(sim);
    // The 0x21 enable: the west lane to the gate-plate lawn (each leg [D the plate lo/hi] — the
    // last one's nav node stands inside the west twin's footprint).
    for (name, at) in [
        ("west lawn", [0.0f32, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
        ("gate plate lawn", [11.0, 5.9, -35.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x21),
            "the west-lane approach stands inside the west 0x21 plate's footprint [D lo/hi x 7.3..10.7 z −35.0..−33.7]");
    }
    w.walk_nav_to([13.0, 5.9, -18.0], "back across the courtyard", 1.6);
    for (plate, want_ch) in [(None, 0x4fu8), (Some(plate0x25), 0x50u8)] {
        let (si, at, rescue_ch) = *servants.iter().find(|(_, _, ch)| *ch == want_ch)
            .unwrap_or_else(|| panic!("the servant carrying {want_ch:#x} [D p6]"));
        // #389's 0x21 plate already fired on the lane approach; #388's 0x25 plate is walked here.
        if let Some(plate) = plate {
            w.walk_nav_to(plate, &format!("onto the {want_ch:#x}-servant's enable plate"), 1.8);
        }
        w.wait(10);
        w.walk_nav_to([at[0], at[1] - 0.3, at[2]], &format!("servant {si}"), 1.7);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} alive when Bond arrives");
        assert_eq!(d.death_channel, 0x60, "servant {si} is protected [D death_ch]");
        let captor = d.captor;
        let ch_on = sim.drones.channels.on(rescue_ch as u32);
        drop(sim);
        if !ch_on {
            // The captor (nearest armed alive guard at the HOSTAGE entering tick [M 0x144030; the
            // port's Drones tick]) — kill it with real fire; its death is the save's trigger.
            if let Some(ci) = captor {
                w.shoot(ci, &format!("servant {si}'s captor"));
            }
            w.wait(30);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} survived the rescue");
        assert!(d.state != 0x01 && d.state != 0x0a,
            "servant {si} left WAIT_SWITCH/HOSTAGE [M the save path], got {:#x}", d.state);
        eprintln!("[03] servant {si} post-save state {:#x}", d.state);
        assert!(sim.drones.channels.on(rescue_ch as u32),
            "the save raised servant {si}'s rescue channel {rescue_ch:#x} [D p6; M 0x1441c0:11-18]");
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
    }

    // 2. The front gate. The plate (kind 1, mask 4 = E-usable [M Trigger_Activate 0x1b2318:38-49])
    //    writes 0x31; the door starts locked behind it and the gate SS #502 listens on 0x31. The
    //    approach is the west lane (z≈−33..−36): the straight courtyard line crosses the garden
    //    moat ditch ((11.8,−34.5) and (16,−31) drop into the water [D the dump's 5 water cels]).
    for (name, at) in [
        ("west lawn", [0.0f32, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_plate = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x31).map(|t| t.centre())
        .expect("the gate plate");
    let gate_door = sim.drones.doors.list.iter()
        .position(|d| d.ch_lock == 0x31).expect("the locked gate door");
    let gate_ss = sim.drones.sss.list.iter()
        .position(|s| s.channel == 0x31).expect("the gate SS");
    drop(sim);
    w.walk_to(Wp { at: gate_plate, act: Act::None, arrive: 1.4 }, "to the gate plate");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x31), "the gate is shut before the plate [sister]");
        assert_eq!(sim.drones.doors.list[gate_door].state, 2, "the gate door sits closed at create [M Door_Create 0x136100 +0xf4 = 2]");
    }
    // Sister first: E on the locked door refuses (the ch_lock refusal [M Door_Activate 0x135be0 —
    // the ch_lock gate + the hit feedback]).
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_at = sim.drones.doors.list[gate_door].pos;
    drop(sim);
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 90 }, arrive: 1.3 }, "E the locked gate door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[gate_door];
        assert!(d.pos01 < 0.5 && d.state != 4, "the locked door refuses E while ch 0x31 is off [M 0x135be0] — state {:#x} pos01 {}", d.state, d.pos01);
    }
    // The plate: E writes 0x31, the door's lock channel turns on, the gate SS plays its keys —
    // a 2-key one-shot (wrap flag non-zero → the wrap disarms to state 0 [M 0x1a9948:119-128]) whose
    // pose HOLDS the end key, so pin the posed matrix leaving its rest pose.
    let ss_rest = w.scene.sim.as_ref().unwrap().drones.sss.list[gate_ss].matrix;
    w.walk_to(Wp { at: gate_plate, act: Act::Use { use_ticks: 90 }, arrive: 1.4 }, "E on the gate plate");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x31), "the plate writes ch 0x31 [D out 0x31 mask 4]");
        let ss = &sim.drones.sss.list[gate_ss];
        assert!(ss.matrix != ss_rest || ss.t > 0.0 || ss.state == 1,
            "the gate SS plays its keys on 0x31 [D ss #502 ch 0x31], got state {:#x} t {}", ss.state, ss.t);
    }
    w.wait(90);
    // Now the door opens to E and Bond walks through to the gate's north side.
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 120 }, arrive: 1.3 }, "E the unlocked gate door");
    w.walk_nav_to([14.7, 5.9, -36.8], "through the gate", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] < -36.0, "Bond is behind the front gate [route], got {:?}", sim.bond.pos);
    }

    // 3. obj0 "Retrieve Dragon Safe contents" through the REAL chain: behind the gate, E on the
    //    Dragon Safe (SP #110, script 0x06000563 [D]) spawns the kind-2 key pickup with +0x28 =
    //    0x4e [M Pickup_Create 0x18dda0:122 + the script's Entity params], the script's stream 0/1
    //    lifts it above the counter [D the keys; M Script_Interp 0x1d9a08:94-110], and stepping
    //    back clears the take's line of sight — the take writes ch 0x4e [M 0x18e650:330-333; L
    //    PCSX2 2026-10-22: the first back-step took it].
    //    The items test's repro spot faces the display case; walking there through the real path.
    w.walk_nav_to([5.90, 5.9, -42.41], "to the dragon safe", 0.35);
    {
        // Aim at the case (SP #110's body at (6.7, 6.1, −42.6) [D the SP dump]) with the real view
        // (turn/look inputs). The use is EDGE-triggered — the world-use block fires on the action
        // bits' press edge, `Input_Action(p, 0xe, 4)`, and a held E activates the world ONCE [M
        // 0x1a5460:395-403; the action machine 0x17ee88 latches per press] — so the view converges
        // FIRST with no use (the earlier hold-E loop spent its one edge while still facing away:
        // no spawn), then ONE press tick at the converged view: the proven doublecross_items_live
        // shape [L PCSX2 2026-10-22: (5.90,5.97,−42.41) yaw 1.710 pitch −0.458, one E press → the
        // card].
        let case = [6.7f32, 6.1, -42.6];
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { break; }
            let bp = sim.bond.pos;
            let dx = case[0] - bp[0];
            let dz = case[2] - bp[2];
            let dy = case[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, case).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let settled = err.abs() < 0.02 && perr.abs() < 0.02;
            let input = nf_game::player::Input {
                turn: if settled { 0.0 } else { err.clamp(-0.08, 0.08) },
                look: if settled { 0.0 } else { perr.clamp(-0.08, 0.08) },
                ..Default::default()
            };
            drop(sim);
            w.tick(input, false); // converge the view only — the edge is NOT spent yet
            if settled { break; }
        }
        // The single press at the converged view (one tick = one edge, the repro's shape).
        w.tick(Default::default(), true);
        // The stream reaches the Entity command within a couple of seconds [M 0x1d8968's 0xF0 arm].
        let mut live = false;
        for _ in 0..120 {
            w.tick(Default::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { live = true; break; }
        }
        assert!(live, "the converged single E press spawned the key pickup [D script 0x06000563 s1]");
    }
    for _ in 0..130 { w.tick(Default::default(), false); } // the reveal plays; the card lifts
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = sim.drones.pickups.iter().find(|p| p.kind == 2 && !p.gone).expect("the keycard is live");
        assert!(k.m[3][1] > 6.04, "the stream lifted the keycard to its last key (y 6.05) [D 0x06000563 s0/s1], got y {:.3}", k.m[3][1]);
        assert_eq!(k.switch_channel, 0x4e, "the keycard's take channel is obj0's leaf [D params]");
    }
    // Standing at the counter the take's LOS is blocked (as in the game [L]); step back (−x, the
    // case faces +x) and the take writes ch 0x4e [M 0x18e650:330-333].
    {
        let back = nf_game::player::Input { forward: -1.0, ..Default::default() };
        let mut taken = false;
        for _ in 0..120 {
            w.tick(back, false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e) { taken = true; break; }
        }
        assert!(taken, "backing off the counter, Bond takes the keycard and ch 0x4e goes on [L + M]");
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[li][o0], 4, "obj0 COMPLETE through the real take [M 0x1827d8:78-85]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced by its enable channel [M 0x1827d8:55-67]");
        assert!(!sim.drones.channels.on(0x60) && !sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x63),
            "no fail hook fired on the route [M 0x1827d8:98-117]");
        // The safe's alarm responders: guards #456/#462's enable channel IS 0x4e [D p3].
        for di in sim.drones.list.iter().filter(|d| d.team == 1 && d.switch_channel == 0x4e) {
            assert!(di.enabled, "guard with enable 0x4e responded to the key [D p3]");
        }
    }

    // 4. The exit: E on the Load Point loads 07000004 through the real reload path (the exit
    //    lawn first — the direct line rides the slope without settling).
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_nav_to([18.5, 5.9, 13.0], "the exit lawn", 1.6);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E the exit");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 [D param 0]");
    eprintln!("[03] full walk: {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// Sister of the full walk's key gate: one objective's channel alone never opens the gate —
/// the AND needs both inputs [D the kind-3 out-0x52 list 4e,51].
#[test]
fn the_key_gate_stays_shut_without_both_objectives() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    // obj0's channel only: the AND(0x4e,0x51) holds 0x52 off [D].
    sim.drones.channels.set(0x4e, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x52), "0x4e alone never opens the gate [D AND list 4e,51]");
    assert!(!sim.drones.channels.on(0x51), "0x51's own AND(0x4f,0x50) is still missing an input [D]");
    // obj1's sub-channels alone: same.
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("reload");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.channels.set(0x4f, 1, sim.drones.frame);
    sim.drones.channels.set(0x50, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x51), "AND(0x4f,0x50) completes obj1's channel [D]");
    assert!(!sim.drones.channels.on(0x52), "0x51 alone never opens the gate [D AND list 4e,51]");
}

/// The servants are protected: killing one raises ch 0x60 and the standing fail hook fails the
/// mission with "Killed Civilians" [M Mission_MonitorObjectives 0x1827d8:98-104; the death channel
/// is the placement's own [D the dump]].
#[test]
fn killing_a_servant_fails_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let si = sim.drones.list.iter().position(|d| d.skin & 0xffff == 0x2b && d.team == 3).expect("a servant");
    let at = sim.drones.list[si].pos;
    drop(sim);
    w.walk_nav_to([at[0], at[1] - 0.3, at[2]], "to the servant", 2.0);
    w.shoot(si, "the servant");
    for _ in 0..120 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.list[si].corpse, "the servant died to real fire");
    assert!(sim.drones.channels.on(0x60), "the placement's death channel 0x60 rose [D]");
    // The mission layer's own records (the events vec is drained per step by the view path — the
    // persistent globals are the evidence): the standing hook's label, the fail state, ch 99.
    assert_eq!(sim.drones.mission.fail_label, 0x0400_0034, "the Killed Civilians fail label [M 0x1827d8:98-104]");
    assert_eq!(sim.drones.mission.state, 2, "MissionState 2 (the fail arm) [M 0x182c60:45-57]");
    assert!(sim.drones.channels.on(99), "the mission-fail channel rose [M mission.rs ch.set(99)]");
}
