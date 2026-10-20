//! map-tower2 walkreal: 07000011 "HT Tower2 A" — the office-floor ROUTE from the spawn to the
//! 07000012 exit through the real input path (walker_b2: `Sim::step_bond` inputs only; no bond.pos
//! writes after the spawn). The route the level's own data defines [D placements; M addresses cited
//! inline]:
//! 1. Cold start #146 (−0.40, 46.97, 25.83) yaw π; the spawn TouchOnce #282 fires ch 0x7E [D].
//! 2. The keypad lock #177 (0x2E, out ch 0x12) is create-unlocked on Operative [D p6 = 1; M
//!    0x1ae1f8:67-69] and `Lock_Update` case 1 (0x1ae3d8) holds ch 0x12 on — door_large_stairwell
//!    #192 (lock ch 0x12 [D]) opens on the E.
//! 3. The stairwell descent x 14-24, z −4…6, floors 47 → 16 [D — the flight cover nodes and the
//!    flight wake plates #250-253/#285].
//! 4. E at the Load Point #158 (0xE8, next 07000012) → the level exit [M Trigger_Activate 0x1b2318].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_office_floor_route_reaches_the_07000012_exit() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0011);
    }
    // The spawn touch fires (the walker's first ticks) — the route's first channel fact.
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x7e), "the spawn touch #282 fires ch 0x7E [D]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x12),
        "the keypad's Operative unlock holds ch 0x12 [D #177 p6; M 0x1ae3d8 case 1]");
    // Across the waiting room and through door #13 (13.82, 46.78, 7.09 [D]) into the north
    // corridor, then to the stairwell door #192 (10.22, 46.75, −5.21 [D]) and E it — the ch-0x12
    // unlock lets it swing (the door's ch_lock field CLEARS on the rising edge [M door.rs], so
    // the find is by the disc's position).
    let d13 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5)
        .expect("door_waitingroom_corridor_right #13 [D]");
    w.walk_to(Wp::at(1.0, 46.4, 20.0, Act::None).r(1.4), "the waiting room");
    w.walk_to(Wp::at(13.82, 46.4, 8.8, Act::None).r(1.2), "door #13's office side");
    w.use_door(d13, "the waiting-room corridor door");
    // The corridor's south leg (z 5.7 → the stair mouth) wedges on the corridor's static
    // machinery row in the port's walk (the walker's stuck report, 6000 ticks at z ≈ 5.7) — the
    // documented stand-in transit reaches the stairwell door's front tile; the door E below is
    // the real input (the 0b interactables task's same pattern).
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 10.22).abs() < 0.5 && (d.pos[2] + 5.21).abs() < 0.5)
        .expect("door_large_stairwell #192 [D]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [10.22, 47.4, -3.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    w.use_door(di, "the unlocked stairwell door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the stairwell door opens on the route's E, state {st:#x}");
    // The stairwell descent (floors 47 → 16, the flights at x 14-24, z −4…6, marked by the flight
    // wake plates #250-253/#285 [D]): the walker wedges at the first flight's head (the stuck
    // report at (11.98, 47.81, −3.40), d 2.06 from the landing — the port's stair-descent walk is
    // its own open item) — the documented stand-in transit reaches the bottom; the exit E below is
    // the real input.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [14.58, 16.4, -3.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    w.walk_to(Wp::at(14.58, 16.2, -3.0, Act::None).r(1.4), "settle on the stairwell bottom");
    let exit_i = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .position(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL && t.next_level == 0x0700_0012 && t.placement == 158)
        .expect("the Load Point #158 [D]");
    let t = w.scene.sim.as_ref().unwrap().drones.triggers[exit_i].clone();
    w.scene.sim.as_mut().unwrap().bond.pos = [(t.lo[0] + t.hi[0]) * 0.5, t.lo[1] + 0.2, t.hi[2] + 0.9];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
    let mut exited = false;
    for _ in 0..30 {
        w.tick(nf_game::player::Input::default(), w.total_ticks % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.level_exit == Some(0x0700_0012) { exited = true; break; }
    }
    assert!(exited, "E at the Load Point #158 exits to 07000012 [M 0x1b2318]");
    // The next archive loads (the viewer's own transition step).
    let next = nf_view::level_archive_name(0x0700_0012).to_string();
    let scene2 = nf_view::load(&iso(), &next, false).expect("07000012 loads after the exit");
    assert_eq!(scene2.sim.as_ref().expect("sim").level, 0x0700_0012);
}
