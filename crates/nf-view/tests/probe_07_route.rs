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
        // drone-bump-collision-3 re-derivation: the old anchors crossed the hall's south band
    // (z ≈ −50.4) — the party crowd's roam band (the kind-8 patrol nodes' z −48.6..−57.7
    // [D the nav dump]; the hall bots boot at z −48.6/−50.4/−50.5 [L probe_07_who]). The
    // crowd is ARMED: enabled bots with running control (the party states write no +0x4F8
    // collision bits [M PartyGirlInit 0x1728c0 / PartyGirl 0x160140]) fill +0x154 on their
    // own control tick [M PreDroneControl 0x148db0:35-48], so DoCollision's near arm fires
    // inside 2.0 [M 0x149ec8:30-33, disasm] and NDrone2_Collision writes kind +0x464 = 0x800
    // [M 0x140e90:95-101] → the Jointy push runs at capsule contact [M 0x1ec180:229-281;
    // drone_bump_live 2/2]. The game shoves Bond off the crowd line (the walker's own
    // 6000-tick wedge at (79.19, −51.21) vs the bot at (79.57, −50.34) — 0.99 apart, just
    // outside the 0.95 contact band the push resolves to). The anchors move to the hall's
    // north lane (z ≈ −46.6, clear of the patrol nodes and of the plate box x 93.31..95.23
    // z −46.43..−45.41 [D trigger bounds]), the line the probe's post-plate skirt already
    // walks. The main agent's live check (does the game's crowd stop Bond mid-floor?) is
    // listed in docs/reports/drone-bump-collision-3.md.
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall north lane", [80.0, 43.6, -46.6]),
        ("the hall north lane east", [90.5, 43.6, -46.6]),
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
