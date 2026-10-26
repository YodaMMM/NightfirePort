//! level-07 route probe: find the walkable real-input route to door #251's threshold that avoids
//! the story plate #412, and print the plate's bounds. Assertion-free, --nocapture.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_the_route() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for t in sim.drones.triggers.iter().filter(|t| t.out == 0x48 || t.out == 0x47) {
            println!("[plate] out {:#x} lo {:?} hi {:?} centre {:?}", t.out, t.lo, t.hi, t.centre());
        }
    }
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
    println!("[route] before E: ch48 {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8);
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    println!("[route] after E: bond {:?} ch48 {}", w.scene.sim.as_ref().unwrap().bond.pos,
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8);
    // Manual straight legs (no nav): hug x ≤ 92.8 west of the plate box (x 93.31..95.23).
    for (name, at) in [
        ("the hall's north-west corner", [92.6f32, 43.6, -47.5]),
        ("north past the plate's west edge", [92.5, 43.6, -45.3]),
        ("the middle band entry", [91.3, 43.6, -43.6]),
        ("the middle band west", [88.0, 43.6, -40.0]),
        ("#251's south side", [85.5, 43.6, -37.2]),
    ] {
        w.walk_to(Wp::new(at[0], at[1], at[2]), name);
        println!("[route] {name} ok bond {:?} ch48 {} ch130 {}", w.scene.sim.as_ref().unwrap().bond.pos,
            w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8,
            w.scene.sim.as_ref().unwrap().drones.channels.on(130) as u8);
    }
    w.walk_to(Wp::new(85.5, 43.6, -33.4), "through #251");
    println!("[route] through ch48 {} ch130 {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) as u8,
        w.scene.sim.as_ref().unwrap().drones.channels.on(130) as u8);
    println!("[route] through bond {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        println!("[route] ch0x48 {} ch130 {} door251 state {:#04x}",
            sim.drones.channels.on(0x48) as u8, sim.drones.channels.on(130) as u8,
            sim.drones.doors.list.iter().find(|d| d.placement == 251).map(|d| d.state).unwrap_or(0));
    }
    let _ = Input::default();
}
