//! level-0b-every-objective-3 probe (assertion-free, run with --ignored): walk every
//! exec-floor nav route on 0700000b and dump where each one dies.
//!
//! The -2 report's prerequisite: the walked desk/terminal/lock approaches wedge when the walk
//! wanders into the dome hole's fail web (the y 7.98..11.98 out-of-bounds slabs trig40-45 →
//! ch 0x7A → OR #46 → ch 0x7B → the fail wedge 0600098A disables Bond [D trig boxes; M OR law]).
//! The probe enumerates the nav paths whose nodes sit on the exec floor (y 44..50), walks each
//! route node-to-node through the harness Walker (the proven movement machinery), and prints
//! every leg's outcome with the fail-web flips, so the real walks attach onto proven lanes.
//! Nothing here is pinned; Bond is re-placed at each leg start (a diagnostic reset, and the
//! fail-wedge disable is cleared the same way).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

#[test]
#[ignore]
fn probe_execfloor_nav_routes() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load"));
    w.stop_on_exit = false;
    for _ in 0..3000u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            if sim.disabled { sim.drones.scripts.skip_request = true; }
        }
        w.tick(Default::default(), false);
        if !w.scene.sim.as_ref().unwrap().disabled { break; }
    }
    // Place the level's bond on the exec floor (the arrival spot the ride step-off reaches).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.0, 46.8, 24.8];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[probe] bond at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    let sim = w.scene.sim.as_ref().unwrap();
    let exec_paths: Vec<usize> = (0..sim.nav.paths.len())
        .filter(|&pi| sim.nav.paths[pi].nodes.iter().filter(|n| (44.0..50.0).contains(&n.pos[1])).count() >= 2)
        .collect();
    eprintln!("[probe] {} of {} paths carry exec-floor nodes", exec_paths.len(), sim.nav.paths.len());
    for &pi in &exec_paths {
        let p = &sim.nav.paths[pi];
        eprintln!("[probe] path {pi} '{}' kind {:#x} nodes {} links {}", p.name, p.kind, p.nodes.len(), p.links.len());
        for (ni, n) in p.nodes.iter().enumerate() {
            eprintln!("[probe]   {pi}.{ni} ({:.2},{:.2},{:.2})", n.pos[0], n.pos[1], n.pos[2]);
        }
    }
    drop(sim);

    // Walk each exec-floor route node-to-node. Bond is re-placed (teleport reset) at each leg
    // start; a web flip or a wedge ends the leg and the next leg re-places cleanly.
    for &pi in &exec_paths {
        let nodes: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().nav.paths[pi].nodes.iter().map(|n| n.pos).collect();
        if nodes.len() < 2 { continue; }
        eprintln!("[probe] === walking path {pi} '{}' ===", w.scene.sim.as_ref().unwrap().nav.paths[pi].name);
        for i in 0..nodes.len() - 1 {
            let (a, b) = (nodes[i], nodes[i + 1]);
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.disabled = false;
                sim.drones.channels.set(0x7b, 0, sim.drones.frame);
                sim.bond.pos = [a[0], a[1] + 0.2, a[2]];
                sim.bond.vel = [0.0; 3];
            }
            w.wait(8);
            let ok = w.try_walk_to(Wp::new(b[0], b[1], b[2]).r(1.0), &format!("{pi}.{i}→{}", i + 1));
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let web = sim.disabled || sim.drones.channels.on(0x7b);
            eprintln!("[probe] leg {pi}.{i}→{} {} bond ({:.2},{:.2},{:.2}) d {:.2}{}", i + 1,
                if ok { "OK" } else if web { "WEB/DISABLED" } else { "DIED" }, p[0], p[1], p[2], dxz(p, b),
                if web { " [ch 0x7B web or disabled]" } else { "" });
        }
    }

    // The key walked legs the real test needs, at the REAL object positions (the Q-Worm C PC
    // ch 0x4C, the keycard lock Switch ch 9) — the same anchors the stand-ins teleport to.
    let (pc, lock) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c).map(|p| p.pos),
         sim.drones.useobjs.switches.iter().find(|s| s.ch == 9).map(|s| s.pos))
    };
    eprintln!("[probe] SCOMP_QWORM_C pos {pc:?}; keycard lock pos {lock:?}");
    let desk = pc.map(|p| [p[0] + 0.6, p[1] - 0.5, p[2] + 0.9]);
    let lockside = lock.map(|p| [p[0] + 0.3, p[1] - 0.7, p[2] + 1.4]);
    let _arrival = [8.0f32, 46.8, 24.8];
    let terminal = [-1.0f32, 47.5, -5.5];
    let n8 = [7.72f32, 47.18, 15.05];
    let n13 = [2.18f32, 47.18, 7.53];
    let n14 = [-1.45f32, 47.18, 3.74];
    let n26 = [-19.64f32, 47.18, 31.64];
    // The doors on the exec floor: the waiting room's exit is a door (the level doc's
    // "in through the waiting room"); dump every door near the exec-floor band with its
    // open/lock channels, then walk to the southern ones and press E through the walker.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.doors.list.iter().enumerate() {
            if (d.pos[1] - 47.5).abs() < 4.0 {
                eprintln!("[probe] door #{i} pos ({:.2},{:.2},{:.2}) kind {} ch_open {} ch_lock {} state {:#x}",
                    d.pos[0], d.pos[1], d.pos[2], d.kind, d.ch_open, d.ch_lock, d.state);
            }
        }
    }
    let mut key_legs: Vec<(&str, [f32; 3], [f32; 3])> = vec![
        ("stepoff→door3", [2.5, 47.6, 25.0], [12.9, 47.4, 19.6]),
        ("pit→south out", [8.0, 46.6, 24.5], [8.0, 47.5, 21.0]),
        ("door3→east room", [14.8, 47.4, 19.4], [14.0, 47.4, 8.5]),
        ("door0→ring n11", [13.0, 47.4, 7.0], [10.21, 47.18, 8.20]),
        ("ring n13→terminal", n13, terminal),
        ("terminal→lock E route", terminal, lockside.unwrap_or(terminal)),
        ("ring n26→desk", n26, desk.unwrap_or(n26)),
    ];
    for (label, a, b) in key_legs.drain(..) {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.disabled = false;
            sim.drones.channels.set(0x7b, 0, sim.drones.frame);
            sim.bond.pos = a;
            sim.bond.vel = [0.0; 3];
        }
        w.wait(8);
        let t0 = w.total_ticks;
        let ok = w.try_walk_to(Wp::new(b[0], b[1], b[2]).r(1.1), label);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let web = sim.disabled || sim.drones.channels.on(0x7b);
        eprintln!("[probe] KEY {label}: {} in {} ticks bond ({:.2},{:.2},{:.2}) d {:.2}{}", ok,
            w.total_ticks - t0, p[0], p[1], p[2], dxz(p, b), if web { " WEB" } else { "" });
    }

    // The floor ray map over the exec-floor band: floor y under each grid point (a ray from
    // y 50 down to y 5). 47.x = the exec floor; ~14 = the balcony/shaft levels; none = void
    // (the dome hole).
    eprintln!("[probe] === floor map (x -30..18 step 2, z -8..36 step 2; '.'=47+floor, 'b'=14-20, ' '=void) ===");
    for x in (-30..18).step_by(2) {
        let mut row = String::new();
        for z in (-8..36).step_by(2) {
            let hit = w.scene.sim.as_ref().unwrap().world.segment_hit([x as f32, 50.0, z as f32], [x as f32, 5.0, z as f32]);
            match hit {
                Some((t, _)) => {
                    let y = 50.0 - t * 45.0;
                    if y >= 44.0 { row.push('.'); } else if (14.0..20.0).contains(&y) { row.push('b'); } else { row.push('?'); }
                }
                None => row.push(' '),
            }
        }
        eprintln!("[probe] x{x:>4}: {row}");
    }
}
