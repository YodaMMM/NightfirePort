//! level-0a-rain-indoors-3 probe (a dump, not a test): why does the armour walk's sim request
//! `level_exit` mid-route ("node 8: level_exit reached at +1017 ticks")? The level's own
//! LOAD_LEVEL #59 "Load Point" sits at (21.72, 1.12, 19.79) [D docs/levels/0700000a.md] — the
//! far east side, nowhere near the west-office route — so the walker's `level_exit.is_some()`
//! should be the death restart [M death.rs 0x216618: level_exit = Some(level_current)]. This
//! probe replays the armour walk's exact legs through the real walker and, the moment a leg
//! returns with `level_exit` set (the walker's `stop_on_exit` keeps the pre-drain sim), dumps
//! the cause candidates: health (a kill), bond y vs the floor (a fall into a HurtPoly), the
//! trigger/hurt volumes still containing Bond, and the armed guards within 15 with a live
//! segment to him.
//!
//! Run with:
//!   NF_ISO=<iso> cargo run -p nf-view --release --example probe_0a_armour_exit

#[path = "../tests/common/walker_b2.rs"]
mod walker;

use walker::{dxz, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump(sim: &nf_view::scene::Sim) {
    let bond = sim.bond.pos;
    println!("\n=== level_exit {:?} ===", sim.drones.level_exit);
    println!("bond ({:.2},{:.2},{:.2}) health {:.1} armour {:.1} disabled {} grounded {}",
        bond[0], bond[1], bond[2], sim.drones.health.health, sim.drones.health.armour, sim.disabled, sim.bond.grounded);
    // The volumes still containing Bond: triggers (any kind) and hurt polys.
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let inside = (0..3).all(|ax| bond[ax] >= t.lo[ax] && bond[ax] <= t.hi[ax]);
        if inside {
            println!("  trigger #{i} kind {:#04x} out {:#04x} gate {:?} box lo {:?} hi {:?}", t.kind, t.out, t.gate, t.lo, t.hi);
        }
    }
    for (i, h) in sim.hurts.iter().enumerate() {
        let inside = (0..3).all(|ax| bond[ax] >= h.lo[ax] && bond[ax] <= h.hi[ax]);
        if inside {
            println!("  hurt #{i} damage {:.1} lo {:?} hi {:?}", h.damage, h.lo, h.hi);
        }
    }
    // Armed, started guards within 15 — alive ones with a live segment to Bond's eye.
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.team != 1 || !d.started || d.corpse || !d.enabled { continue; }
        let dist = dxz(d.pos, bond);
        if dist > 15.0 { continue; }
        let eye = [bond[0], bond[1] + sim.bond.eye, bond[2]];
        let los = sim.world
            .segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none();
        println!("  guard #{i} state {:#x} health {:.1} dist {dist:.1} los {los} pos ({:.2},{:.2},{:.2})",
            d.state, d.health, d.pos[0], d.pos[1], d.pos[2]);
    }
}

fn main() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a"));
    // All LOAD_LEVEL triggers + the 0x15 plate: the reset pair's exact boxes (static — dump at load).
    for (i, t) in w.scene.sim.as_ref().unwrap().drones.triggers.iter().enumerate() {
        if t.kind == 10 || t.out == 0x15 {
            println!("TRIG #{i} kind {:#04x} out {:#04x} gate {:?} next {:?} lo {:?} hi {:?}",
                t.kind, t.out, t.gate, t.next_level, t.lo, t.hi);
        }
    }
    w.walk_to(Wp::new(-3.9, 0.0, 13.2), "out of the lift");
    if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { dump(w.scene.sim.as_ref().unwrap()); return; }
    w.walk_to(Wp::new(-3.9, 0.0, 15.5).r(1.2), "lobby south");
    if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { dump(w.scene.sim.as_ref().unwrap()); return; }
    assert!(w.follow_nav_route([-15.5, 0.4, 6.0], "to the mid room"), "no nav route to the mid room");
    // The player-true approach legs, wall-checked at two body heights (the plate spans
    // x −24.27..−19.96, z 8.89..11.68 — every candidate stays east/south of it):
    let legs: [(&str, [f32; 3], [f32; 3]); 4] = [
        ("corridor east", [-20.27, 0.0, 5.44], [-18.0, 0.0, 5.5]),
        ("to the mid room", [-18.0, 0.0, 5.5], [-15.5, 0.0, 6.0]),
        ("to the armour", [-15.5, 0.0, 6.0], [-13.46, 0.0, 7.61]),
        ("lobby direct", [-10.04, 0.0, 17.4], [-13.46, 0.0, 7.61]),
    ];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (name, a, b) in legs {
            for y in [0.5f32, 1.0, 1.5] {
                let hit = sim.world.segment([a[0], a[1] + y, a[2]], [b[0], b[1] + y, b[2]]);
                println!("LEG {name} y{y}: {:?}", hit.map(|d| format!("blocked at {d:.3}")).unwrap_or_else(|| "clear".into()));
            }
        }
    }
    if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { dump(w.scene.sim.as_ref().unwrap()); return; }
    w.walk_to(Wp::new(-13.46, 0.4, 7.61).r(0.9), "onto the BodyArmourHeavy");
    if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { dump(w.scene.sim.as_ref().unwrap()); return; }
    w.wait(20);

    let sim = w.scene.sim.as_ref().unwrap();
    let bond = sim.bond.pos;
    println!("\n=== WALK COMPLETE: level_exit {:?} armour {} health {:.1} bond ({:.2},{:.2},{:.2}) ===",
        sim.drones.level_exit, sim.drones.health.armour, sim.drones.health.health, bond[0], bond[1], bond[2]);
}
