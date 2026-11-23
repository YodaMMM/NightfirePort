//! probe 0700000d's north-armour POCKET (retune-nb-0d-20, v2): v1 measured the detour
//! walkable from the band to 3.6 u short (stall at (205.7, 1.13, -208.96) [L]) with the
//! armour pickup centre (205.41, 0.11, -205.31) ~1 u BELOW the band floor (the origin
//! 1.0347 band vs the pickup's own y 0.11 [L]), and the east touch test slipped to a lower
//! floor y 0.72 without taking it [L]. This probe:
//!   1. walks the last 10 u from FOUR band x-approaches (202/204/206/208);
//!   2. teleports INTO the pocket, takes the armour, and climbs back OUT (the exit is the
//!      lever's real cost: the detour must rejoin the route);
//!   3. maps the pocket rim with vertical columns (the walls/pillars around it).
//! Teleport probe (assertion-free diagnostic; never runs in CI) — run --ignored --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn snap(w: &mut Walker, ticks: u32) {
    for _ in 0..ticks { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_0d_north_armour2() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // 1. The rim map: vertical columns around the armour (x 200..210, z -212..-202).
    eprintln!("=== 1. the pocket rim columns (bullet geometry) ===");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for x in [200.0f32, 202.0, 204.0, 205.41, 207.0, 209.0] {
            for z in [-212.0f32, -209.0, -206.0, -204.0, -202.0] {
                let mut hits = Vec::new();
                let mut bot = -1.0f32;
                for _ in 0..10 {
                    match sim.world.segment_hit([x, bot, z], [x, 8.0, z]) {
                        Some((f, _)) => { let y = bot + (8.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                        None => break,
                    }
                }
                eprintln!("[col] x {x} z {z}: {:?}", hits.iter().map(|y| format!("{y:.2}")).collect::<Vec<_>>());
            }
        }
    }

    // 2. Four band approaches to the armour spot (the last 10 u each).
    eprintln!("=== 2. the band approaches ===");
    for (ax, az) in [(202.0f32, -210.0f32), (204.0, -210.0), (206.0, -210.0), (208.0, -210.0)] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [ax, 1.0347, az];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        w.lenient = true;
        let before = bond_pos(&w);
        let ok = w.try_walk_to(Wp::new(205.41, 1.18, -205.31).r(1.2), "the approach");
        snap(&mut w, 40);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[app] from ({ax},{az}): {before:?} -> {:?} ok {ok} grounded {} armour {:.1}",
            bond_pos(&w), sim.bond.grounded, sim.drones.health.armour);
    }

    // 3. Into the pocket, the touch, and the climb back OUT (from three exits).
    eprintln!("=== 3. the pocket touch + the exits ===");
    {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [205.41, 1.18, -206.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        let armour0 = w.scene.sim.as_ref().unwrap().drones.health.armour;
        eprintln!("[pocket] settled {:?}", bond_pos(&w));
        let ok = w.try_walk_to(Wp::new(205.41, 0.2, -205.31).r(1.0), "the pocket touch");
        snap(&mut w, 40);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[pocket] touch ok {ok}: armour {armour0:.1} -> {:.1} at {:?}", sim.drones.health.armour, bond_pos(&w));
        // The exits: south back down the band, north past the warehouse, east.
        for (label, at) in [
            ("south exit", [205.4f32, 1.03, -212.0f32]),
            ("north exit", [205.4, 1.03, -201.0]),
            ("east exit", [209.5, 1.03, -205.5]),
        ] {
            let before = bond_pos(&w);
            let ok = w.try_walk_to(Wp::new(at[0], at[1], at[2]).r(1.2), label);
            snap(&mut w, 30);
            eprintln!("[exit] {label}: {before:?} -> {:?} ok {ok} grounded {}", bond_pos(&w),
                w.scene.sim.as_ref().unwrap().bond.grounded);
        }
    }
}
