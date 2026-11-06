//! 07000012's opening: the office worker sees armed Bond, shouts and runs, and the two guards in
//! the corridor take it up 60 ticks later. [L] PCSX2 2026-11-05, breakpoints on Drone_SM_SetState
//! and DoForcedAttack with Bond idle: the worker goes 0x12 -> 0x85 -> 0x64 at frame 168, both guards
//! get msg 0x1f at 228 (one in Alert, one in PlayScript) and enter combat; the worker ends at
//! about (33, 1, 34) in CivilianHiding.
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_scared_worker_wakes_both_corridor_guards() {
    let mut scene = nf_view::load(&iso(), "07000012.bin", false).expect("load 07000012");
    let (worker, a, b) = (0usize, 6usize, 7usize);
    let (mut ran_at, mut woke) = (None, [None, None]);
    for t in 0..1000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let list = &sim.drones.list;
        if ran_at.is_none() && list[worker].state == 0x64 { ran_at = Some(t); }
        for (k, i) in [a, b].into_iter().enumerate() {
            if woke[k].is_none() && !matches!(list[i].state, 3 | 4 | 5) { woke[k] = Some(t); }
        }
    }
    let ran = ran_at.expect("the worker never ran to a goal");
    for w in woke {
        let w = w.expect("a corridor guard never woke");
        assert!((55..=65).contains(&(w - ran)), "a guard woke {} ticks after the worker ran, the game takes 60", w - ran);
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = sim.drones.list[worker].pos;
    assert_eq!(sim.drones.list[worker].state, 0x13, "the worker hides at the end of the run");
    assert!((p[0] - 33.0).abs() < 1.5 && (p[2] - 34.0).abs() < 1.5, "the worker stopped at {p:?}");
}
