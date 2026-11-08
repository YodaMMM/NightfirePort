//! The door contact refine on 07000003's west double doors (leaves #2/#3, hinges at x 11.86
//! and 9.99, z -20.26; the doorway runs x 10.0..11.9).
//!
//! `Collide_Update` 0x1ea398 mirrors a mover onto the door's own contact list only when the
//! node survives `Collide_Intersect` 0x1ebaa8: the 0x800 arm presets the distance to 1e8 and
//! only `Intersect_CylGeom` 0x109da0 (the mover's capsule against the door's coll data)
//! overwrites it [M 0x1ebe00-0x1ebe68]; every 1e8 node is freed before the mirror [M 0x1ea398].
//! `Door_ReactToHit` 0x135980 reverses a swinging leaf from that list alone. The port's old
//! contact was the `Collide_PickObj` 0x1eae80 sphere reach only (1.5 x 1.41 + 0.55 = 2.67 from
//! the leaf's centre), so a Bond 1.7 west of the hinge, nowhere near the leaf, reversed every
//! swing his own E press started: the doors opened a few degrees and shut again.

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn west_doors(sim: &nf_view::scene::Sim) -> Vec<usize> {
    sim.drones.doors.list.iter().enumerate()
        .filter(|(_, d)| ((d.pos[0] - 10.9f32).powi(2) + (d.pos[2] + 20.2f32).powi(2)).sqrt() < 1.5)
        .map(|(i, _)| i).collect()
}

/// Load, let the level settle, hold the guards out, stand Bond at `at` (floor point) facing the
/// doors, press E once and step `ticks`. Returns the doors' (state, pos01) per tick.
fn press_from(at: [f32; 3], yaw: f32, ticks: u32) -> Vec<Vec<(u16, f32)>> {
    let mut scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    for d in sim.drones.list.iter_mut() { if d.team == 1 { d.enabled = false; } }
    sim.place_bond(at, yaw);
    let doors = west_doors(sim);
    assert_eq!(doors.len(), 2, "the west double door is two leaves");
    let mut out = Vec::new();
    for k in 0..ticks {
        sim.step_bond(Input::default(), false, false, k == 5);
        out.push(doors.iter().map(|&i| (sim.drones.doors.list[i].state, sim.drones.doors.list[i].pos01)).collect());
    }
    out
}

#[test]
fn a_press_from_beside_the_jamb_opens_the_doors_all_the_way() {
    // South of the wall, 1.7 west of the west hinge: the use pick reaches the leaf, the leaf's
    // swing never comes within 0.8 of Bond.
    let run = press_from([8.25, 5.35, -20.9], 1.15, 40);
    assert!(run[6].iter().all(|&(s, _)| s == nf_game::door::OPENING), "the press opens both leaves: {:?}", run[6]);
    let last = run.last().unwrap();
    assert!(last.iter().all(|&(s, p)| s == nf_game::door::OPEN && (p - 0.5).abs() > 0.49),
        "the swing runs to the end and stays open (no reversal): {last:?}");
    assert!(!run.iter().any(|t| t.iter().any(|&(s, _)| s == nf_game::door::CLOSING)), "never reversed to CLOSING");
}

#[test]
fn a_press_from_the_wall_recess_opens_the_doors_all_the_way() {
    // The recess west of the doorway (x 7.27..8.8, back wall z -19.56): the walk's stuck spot.
    let run = press_from([8.25, 5.35, -20.15], 1.15, 40);
    let last = run.last().unwrap();
    assert!(last.iter().all(|&(s, p)| s == nf_game::door::OPEN && (p - 0.5).abs() > 0.49),
        "the swing runs to the end and stays open: {last:?}");
}

#[test]
fn a_leaf_closing_into_bond_still_reverses() {
    // Sister: a body the leaf really meets still turns it. Open the doors from the south, then
    // stand Bond north of the doorway inside the west leaf's closing sweep and press
    // E again from the south: the closing leaf meets his capsule and swings open again (the anti-crush).
    let mut scene = nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    for d in sim.drones.list.iter_mut() { if d.team == 1 { d.enabled = false; } }
    sim.place_bond([8.25, 5.35, -20.9], 1.15);
    let doors = west_doors(sim);
    // The west leaf: the hinge nearer x 10.
    let west = *doors.iter().min_by(|&&a, &&b| sim.drones.doors.list[a].pos[0].total_cmp(&sim.drones.doors.list[b].pos[0])).unwrap();
    let shut = sim.drones.doors.list[west].live_sph().0;
    for k in 0..40 { sim.step_bond(Input::default(), false, false, k == 5); }
    assert!(doors.iter().all(|&i| sim.drones.doors.list[i].state == nf_game::door::OPEN), "opened");
    let open01 = sim.drones.doors.list[west].pos01;
    // Stand on the bisector of the west leaf's swing, 0.7 out from its hinge: inside the sweep.
    let (hinge, open) = { let d = &sim.drones.doors.list[west]; ([d.matrix[3][0], d.matrix[3][2]], d.live_sph().0) };
    let u = |c: [f32; 3]| { let v = [c[0] - hinge[0], c[2] - hinge[1]]; let l = v[0].hypot(v[1]); [v[0] / l, v[1] / l] };
    let (a, b) = (u(shut), u(open));
    let m = [a[0] + b[0], a[1] + b[1]];
    let ml = m[0].hypot(m[1]);
    let at = [hinge[0] + m[0] / ml * 0.7, 5.35, hinge[1] + m[1] / ml * 0.7];
    sim.place_bond(at, 3.14);
    // Close it the game's way from the south (a second E on an open door is the close; the
    // activator stands outside the sweep): the use pick from a head 0.6 south of the closed
    // leaf's centre, looking north at it — as the walk's own presses land.
    {
        let eye = [shut[0], shut[1], shut[2] - 0.6];
        let look = [0.0, 0.0, 1.0];
        let _ = sim.drones.doors.player_use(eye, look, &mut sim.drones.channels, sim.drones.frame, &sim.world);
    }
    let mut closing = false;
    let mut reversed = false;
    for _ in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        let st = sim.drones.doors.list[west].state;
        if st == nf_game::door::CLOSING { closing = true; }
        if closing && st == nf_game::door::OPENING { reversed = true; break; }
    }
    assert!(closing, "the close started (open pos01 {open01:.2})");
    assert!(reversed, "the closing leaf met Bond at {at:?} and reversed");
}
