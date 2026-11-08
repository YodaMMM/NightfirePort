//! A guard's sight ray is blocked by another drone's body: `Collide_LineOfSight` 0x1edf50 is
//! `Collide_RayIntersect` 0x1ec690, whose `Collide_Pick` offers every object but the looker and
//! the target [M Collide_PickObj 0x1eae80:57-59]; the guard flags 0xa27 carry no 0x100, so the
//! class-2 drones stay in [M 0x1eae80:70-85], and a character's glist without collision data goes
//! to `Collide_Jointy` [M Collide_Intersect 0x1ebaa8:28-31]. The port's body test is the posed
//! bone boxes (`drone_ray_part`) [S for the joint test]. On 07000001 the start guard #0 and the
//! contact #1 [D placements] give the shape. NF_ISO-gated.

use nf_game::player::Input;

/// The guard's head ray to Bond's chest, with the contact `t` of the way along it (or beside it).
fn body_on_ray(contact_on_line: bool) -> bool {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut s = nf_view::load(&iso, "07000001.bin", false).expect("load");
    // Past the intro to its hand-back (Bond enabled, ~tick 2200), and no further: by tick 2400 the
    // start guards are fighting and the contact has entered LeadHide 0x1f with its cower pose
    // [M 0x15ec08 case 1: anim 0x19], his boxes 5.4..6.1 under the head ray (the old 2400-tick
    // warm-up met that on the joined tree). The law under test needs him standing.
    for _ in 0..4000 {
        let sim = s.sim.as_mut().unwrap();
        if !sim.disabled { break; }
        sim.step_bond(Input::default(), false, false, false);
    }
    let guard = [-5.29f32, 6.34, -0.50];
    let bond = [-8.49f32, 6.44, -9.78];
    let on = [guard[0] + (bond[0] - guard[0]) * 0.77, 6.34, guard[2] + (bond[2] - guard[2]) * 0.77];
    let contact = if contact_on_line { on } else { [on[0] + 3.0, on[1], on[2] - 1.0] };
    for _ in 0..10 {
        let sim = s.sim.as_mut().unwrap();
        sim.bond.pos = bond;
        sim.drones.list[0].pos = guard;
        sim.drones.list[1].pos = contact;
        sim.drones.list[1].health = 10.0;
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
    }
    let sim = s.sim.as_ref().unwrap();
    assert_ne!(sim.drones.list[1].state, 0x1f, "the contact stands (not in LeadHide's cower)");
    let d = sim.data();
    nf_game::drone::drone_ray_part(&sim.drones.list[1], &d, sim.drones.list[0].head(), sim.bond.head()).is_some()
}

#[test]
fn the_contact_on_the_line_stands_in_the_guards_sight_ray() {
    if std::env::var("NF_ISO").is_err() { eprintln!("NF_ISO not set"); return; }
    assert!(body_on_ray(true), "the contact's body boxes take the head ray");
    assert!(!body_on_ray(false), "sister: beside the line the ray is clear of him");
}
