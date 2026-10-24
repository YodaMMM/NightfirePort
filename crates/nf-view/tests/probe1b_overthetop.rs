//! level-1b-drake-4 probe: the over-the-top crossing. The deck's one-sided room top (the slab at
//! y ≈ 16.74 [D probe1b_float's up-columns, x 5.5–24]) was never scanned WEST of x 5.5 — the pit
//! column between the boundary wall (x 4.433, y −16.56..+16.74) and the slab's scanned west edge
//! may be open, and the WALL itself ends at y 16.74. Plan: the float entry (the disc's own SP#211
//! gate drive [D 0x060007e2 s1 t180 ch 30; SP#211 out 0x2e]), rise in the pit column past the
//! wall's top, thrust west over it, descend into the arena. Real input only. Assertion-free.
#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, SS_ZEROG_9};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_over_the_top_crossing() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.scene.sim.as_mut().unwrap().bond.substate = nf_game::player::SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    let mut entered = false;
    for _ in 0..120u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().bond.substate == SS_ZEROG_9 { entered = true; break; }
    }
    eprintln!("[ot] float entered {entered}");
    // Phase 1: rise. Stay at x ≈ 6.5 (between the wall 4.433 and the slab's scanned west 5.5 —
    // and east of the island's 4.98 face). Zero the westward drift; pure lift.
    let mut peak = f32::MIN;
    for t in 0..3000u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        // Gentle west nudge only while east of the target column; hold x ≈ 6.5.
        let input = if p[0] > 6.8 {
            Input { lift: 1.0, forward: 1.0, ..Default::default() } // forward = toward the look; set yaw west below
        } else {
            Input { lift: 1.0, ..Default::default() }
        };
        drop(sim);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.yaw = -std::f32::consts::FRAC_PI_2; // look west (-x)
        }
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        peak = peak.max(sim.bond.pos[1]);
        if t % 100 == 0 { eprintln!("[ot rise t{t}] bond {:?} grounded {}", sim.bond.pos, sim.bond.grounded); }
        if sim.bond.pos[1] > 18.5 { eprintln!("[ot] ABOVE THE WALL TOP at t{t}"); break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[ot] peak y {:.2} at {:?}", peak, sim.bond.pos);
    }
    // Phase 2: thrust west over the wall.
    for t in 0..1200u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        }
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t % 60 == 0 { eprintln!("[ot west t{t}] bond {:?}", sim.bond.pos); }
        if sim.bond.pos[0] < 3.5 { eprintln!("[ot] CROSSED x 4.433 at t{t} — the arena side!"); break; }
    }
    // Phase 3: descend into the arena.
    for t in 0..1200u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t % 60 == 0 { eprintln!("[ot down t{t}] bond {:?}", sim.bond.pos); }
        if sim.bond.grounded && sim.bond.pos[1] < 3.0 { eprintln!("[ot] LANDED in the arena at t{t}"); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[ot final] bond {:?} min_x tracked separately", sim.bond.pos);
    // Drake exposure from wherever we ended.
    if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
        eprintln!("[ot] drake present idx {i} at {:?}", sim.drones.list[i].pos);
    } else {
        eprintln!("[ot] drake not spawned (gate not driven in this probe)");
    }
}
