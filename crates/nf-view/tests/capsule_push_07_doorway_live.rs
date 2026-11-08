//! 07000007's hall doorway: the two guards standing in the doorway (x 92.86..95.65, z -48.1..-46.4)
//! must never shove Bond through its west jamb. A real walk (no teleports): the probe_07_route
//! legs to the hall door, the E press, then the walk toward the doorway's west side. On the old
//! code the first airborne tick in the doorway (the floor band drops out on the threshold slope)
//! delivered the whole guard overlap at once (-0.70 x), Bond's spine crossed the jamb plane at
//! x 92.855, the wall pass resolved him out of the far side and he fell to y -115.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn doorway_guards_never_push_bond_through_the_jamb() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    for (name, at) in [
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall west mouth", [76.6, 43.6, -50.7]),
        ("the hall floor", [83.1, 43.6, -50.5]),
        ("the hall east", [93.3, 43.6, -50.4]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    let bots_near = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        sim.drones.list.iter().filter(|d| d.enabled && !d.corpse
            && ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < 2.0).count()
    };
    assert!(bots_near(&w) >= 2, "the two doorway guards stand beside Bond after the E press");
    // The press toward the west jamb: at the 07000007 party pace (0.9 [M Player_Move 0x191dd0:139-141])
    // the guard nearer the jamb can stand between Bond and the west side, so the walk may end
    // wedged against him; the asserts below are what the push must never break either way.
    w.try_walk_to(Wp::new(92.6, 43.6, -47.5), "the doorway's west side");
    w.wait(240);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    println!("[doorway] bond {p:?}");
    // The doorway floor sits under y 44.1; a Bond pushed into the jamb hangs with no floor and falls.
    assert!(p[1] > 43.5 && p[1] < 44.6, "Bond stays on the doorway floor (y {})", p[1]);
    let floor = w.scene.sim.as_ref().unwrap().world.segment_hit(p, [p[0], p[1] - 1.5, p[2]]);
    assert!(floor.is_some(), "Bond has floor under him, not the void inside the jamb ({p:?})");
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "Bond stands grounded in the doorway");
}
