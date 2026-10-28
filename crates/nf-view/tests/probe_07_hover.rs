//! drone-bump-collision-9 diagnostic: the post-E hover wedge of probe_07_route. The -8 round's
//! dump pinned Bond at (93.40, 44.26, −46.32) — inside the 0x48 plate box — hanging with
//! grounded=false while the floor sits 0.23 below. Assertion-free, --nocapture: settle Bond at
//! the spot and dump EVERY face the port's scoped feet ray can read there vs the flat one
//! [M Player_FeetOnPoint 0x194188 = the cel-scoped ray; Player_Collision 0x194df0 = the band].

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump(walker: &mut walker::Walker, tag: &str) {
    let sim = walker.scene.sim.as_ref().unwrap();
    let p = sim.bond.pos;
    let ray_len = sim.bond.cc + 0.6;
    let to = [p[0], p[1] - ray_len, p[2]];
    println!("== {tag} pos {:?} grounded {} floor_ny {:.3} cc {:.3} cel {:?}",
        p, sim.bond.grounded, sim.bond.floor_ny, sim.bond.cc, sim.bond.cel);
    let scoped = sim.world.segment_hit_scoped(sim.bond.cel, p, to);
    let flat = sim.world.segment_hit(p, to);
    println!("   ray down {ray_len:.2}: scoped {:?} flat {:?}", scoped, flat);
    for (name, h) in [("scoped", &scoped), ("flat", &flat)] {
        if let Some((t, n)) = h {
            let hit = [p[0], p[1] + t * (to[1] - p[1]), p[2]];
            println!("   {name} hit at y {:.3} (feet y {:.3}, dy {:.3}) normal {:?}", hit[1], p[1] - sim.bond.cc, p[1] - sim.bond.cc - hit[1], n);
        }
    }
    if let Some(cel) = sim.bond.cel {
        let owners = sim.world.scoped_run_owners(cel, p, to);
        println!("   scoped run owners: {:?}", owners.map(|v| v.len()));
    }
    let dtris = sim.world.dynamic_tris_near([p[0] - 2.0, to[1] - 0.5, p[2] - 2.0], [p[0] + 2.0, p[1] + 0.5, p[2] + 2.0]);
    println!("   dynamic tris near: {} — first centres:", dtris.len());
    for t in dtris.iter().take(8) {
        let c = [(t[0][0] + t[1][0] + t[2][0]) / 3.0, (t[0][1] + t[1][1] + t[2][1]) / 3.0, (t[0][2] + t[1][2] + t[2][2]) / 3.0];
        println!("     ({:.2},{:.2},{:.2})", c[0], c[1], c[2]);
    }
    let floor = sim.world.floor_below(p, 6.0);
    println!("   floor_below(6): {:?}", floor);
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.pos[0].hypot(d.pos[2]) > 0.0 && walker::dxz(d.pos, p) < 3.5 {
            println!("   bot{i} pos {:?} state {:#04x} kind_464 {:#x} corpse {}", d.pos, d.state, d.kind_464, d.corpse);
        }
    }
    let d244 = sim.drones.doors.list.iter().find(|d| d.placement == 244);
    if let Some(d) = d244 {
        let live = d.live_sph();
        println!("   #244 state {:#04x} pos01 {:.2} live sph {:?} r {:.3}", d.state, d.pos01, live.0, live.1);
    }
}

#[test]
fn probe_the_post_e_hover() {
    let iso = iso();
    let mut w = walker::Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    // Settle A: the NORMAL root height at the -8 hover xz (the hovered root 44.26 was 0.23 low).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [93.40, 44.49, -46.32];
        sim.bond.vel = [0.0; 3];
    }
    for t in 0..240u32 {
        w.tick(Input::default(), false);
        if t % 80 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            println!("[settle A t{t}] pos {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        }
    }
    dump(&mut w, "settle A (normal root at the hover xz)");
    // Settle B: the -8 hovered root height itself.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [93.40, 44.26, -46.32];
        sim.bond.vel = [0.0; 3];
    }
    for t in 0..240u32 {
        w.tick(Input::default(), false);
        if t % 80 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            println!("[settle B t{t}] pos {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        }
    }
    dump(&mut w, "settle B (the -8 hovered root)");
    // Settle C/D: map the hover's extent OUTSIDE the 0x48 plate box (a teleport there fires the
    // story scene — see settle A), so the same 0.22-over-floor test away from the doorway.
    for (tag, at) in [("C west of the box", [92.5f32, 44.49, -46.3]), ("D north of the box", [94.0, 44.49, -44.5])] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = at;
            sim.bond.vel = [0.0; 3];
        }
        for _ in 0..160 { w.tick(Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        println!("== {tag} settled {:?} grounded {} floor_below {:?}", p, sim.bond.grounded, sim.world.floor_below(p, 6.0));
    }
    // The walkable-face hunt: every dynamic tri whose centroid sits within 1.2 of the capsule
    // segment at the settled B spot, with an up-ish normal — the face the capsule stands on.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = [93.410416f32, 44.25355, -46.316563];
        let a = [p[0], p[1] + 0.275, p[2]];
        let b = [p[0], p[1] + 0.55 - sim.bond.cc, p[2]];
        let dtris = sim.world.dynamic_tris_near([p[0] - 3.0, 42.5, p[2] - 3.0], [p[0] + 3.0, 45.5, p[2] + 3.0]);
        println!("== face hunt: {} dynamic tris in the 6x3x6 box", dtris.len());
        for t in dtris.iter() {
            let c = [(t[0][0] + t[1][0] + t[2][0]) / 3.0, (t[0][1] + t[1][1] + t[2][1]) / 3.0, (t[0][2] + t[1][2] + t[2][2]) / 3.0];
            let dx = c[0] - p[0]; let dy = c[1] - b[1]; let dz = c[2] - p[2];
            if dx * dx + dz * dz < 1.44 && c[1] > 42.9 && c[1] < 44.4 {
                let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
                let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
                println!("   tri cent ({:.2},{:.2},{:.2}) ny {:.2} verts {:?}", c[0], c[1], c[2], n[1] / nl, t);
            }
        }
    }
    // The walkable-face mapper: capsule_push_scoped's walkable/push over synthetic capsules
    // around the settled B spot — where does walkable flip, and what does the push say.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let cel = sim.bond.cel;
        let cc = sim.bond.cc;
        let ref_v = [0.0f32; 3];
        println!("== walkable map (x 93.41, r 0.55): z scan at y 44.25 / y scan at z -46.32");
        for z in [-44.5f32, -45.0, -45.4, -45.8, -46.0, -46.32, -46.6, -47.0] {
            let a = [93.41, 44.25 + 0.275, z];
            let b = [93.41, 44.25 + 0.55 - cc, z];
            let (push, walkable) = sim.world.capsule_push_scoped(cel, a, b, 0.55, false, ref_v);
            println!("   z {z:.2} walkable {} push {:?}", walkable, push);
        }
        for y in [44.6f32, 44.49, 44.25, 44.0, 43.8, 43.6] {
            let a = [93.41, y + 0.275, -46.32];
            let b = [93.41, y + 0.55 - cc, -46.32];
            let (push, walkable) = sim.world.capsule_push_scoped(cel, a, b, 0.55, false, ref_v);
            println!("   y {y:.2} walkable {} push {:?}", walkable, push);
        }
    }
}
