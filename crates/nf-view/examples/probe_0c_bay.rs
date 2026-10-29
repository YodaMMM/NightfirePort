//! Scratch (level-0c-display-hatch-2): the port at three fresh [L 2026-10-26]
//! placements — the bay drop (the touchplate ch 27 pulse + the rest root), the roof walk south
//! (ch 27 must stay 0), and the mid floor's north-edge hold.
use nf_game::player::Input;
fn main() {
    let path = std::env::var("NF_ISO").unwrap();
    block_rays(&path);
    let load = || -> nf_view::scene::Scene {
        let mut scene = nf_view::load(&path, "0700000c.bin", false).expect("load 0700000c");
        {
            let sim = scene.sim.as_mut().unwrap();
            for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
            for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
        }
        scene
    };
    let yaw_s = std::f32::consts::PI;
    let yaw_n = 0.0;

    // [L 1] placed at (5.0, 16.4, -110.5) yaw 3.1416: the game settles (5.3328, 15.7852, -111.0017),
    // falls; frame 10 (5.3947, 14.2643, -111.3391) with ch 27 = 1; frame 20+ rests
    // (5.42, 12.8773, -111.38), ch 27 back to 0.
    let mut scene = load();
    {
        // DIRECT placement (stand_at snaps up to the bay roof — the game's probe placement sits
        // INSIDE the bay air and falls).
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [5.0, 16.4, -110.5], yaw_s);
        sim.bond.pos = [5.0, 16.4, -110.5];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = yaw_s;
        sim.bond.grounded = false;
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        for (i, t) in sim.drones.triggers.iter().enumerate() {
            if t.out == 0x1b || (t.lo[1] > 12.0 && t.hi[1] < 16.0 && (t.lo[2] - -112.0) < 0.0 && (t.hi[2] - -109.0) > 0.0 && t.lo[0] < 8.0 && t.hi[0] > 3.0) {
                println!("  trig {i}: kind {} out {:#x} lock {:#x} mask {:#x} lo {:?} hi {:?} tris {}",
                    t.kind, t.out, t.lock, t.mask, t.lo, t.hi, t.tris.len());
            }
        }
    }
    println!("== [L1] the bay drop from (5.0, 16.4, -110.5) [direct placement] ==");
    for t in 1..=60u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let p = sim.bond.pos;
        let ch27 = sim.drones.channels.on(0x1b);
        if t <= 25 || t % 10 == 0 {
            println!("  t{t}: ({:.4}, {:.4}, {:.4}) ch27={}", p[0], p[1], p[2], ch27 as u8);
        }
    }

    // [L 2] placed on the roof at (5.75, 17.6, -110.6) walking south (the same load, re-stood).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [5.75, 17.6, -110.6], yaw_s);
    }
    println!("== [L2] the roof walk south ==");
    let mut fired = false;
    for t in 1..=400u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = sim.bond.pos;
        if sim.drones.channels.on(0x1b) { fired = true; }
        if t % 40 == 0 { println!("  t{t}: ({:.4}, {:.4}, {:.4}) ch27={}", p[0], p[1], p[2], sim.drones.channels.on(0x1b) as u8); }
    }
    let p = scene.sim.as_ref().unwrap().bond.pos;
    println!("  roof walk end: ({:.4}, {:.4}, {:.4}) ch27_ever_fired={}", p[0], p[1], p[2], fired);

    // [L 4] placed on the mid floor at (3.0, 11.4, -114.0) walking north (the same load).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, 11.4, -114.0], yaw_n);
    }
    println!("== [L4] the mid floor's north-edge hold ==");
    for t in 1..=200u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = sim.bond.pos;
        if t % 20 == 0 { println!("  t{t}: ({:.4}, {:.4}, {:.4})", p[0], p[1], p[2]); }
    }

    // Route probe: the walkway walk south from the -1 test's own start — does the port carry
    // Bond off the walkway end into the bay (the game's route per the driver)?
    let mut scene = load();
    {
        let sim = scene.sim.as_mut().unwrap();
        for k in 0..sim.drones.locks.len() {
            if (14..=17).contains(&sim.drones.locks[k].out) {
                sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
            }
        }
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [5.8, 15.8, -108.5], yaw_s);
    }
    println!("== route: the walkway walk south (hinges burnt, hatch open) ==");
    for t in 1..=300u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        let p = sim.bond.pos;
        if t % 30 == 0 { println!("  t{t}: ({:.4}, {:.4}, {:.4}) ch27={}", p[0], p[1], p[2], sim.drones.channels.on(0x1b) as u8); }
    }
    let p = scene.sim.as_ref().unwrap().bond.pos;
    println!("  walkway walk end: ({:.4}, {:.4}, {:.4})", p[0], p[1], p[2]);
}

// Which geometry blocks the mouth? Rays at body height across z -110.0..-113.0, y 18.6 and
// 17.4 and 16.0, x 5.0/5.75 — plus the burned-hinge variant (does SS #179's rise unblock?).
#[allow(dead_code)]
fn block_rays(path: &str) {
    let mut scene = nf_view::load(path, "0700000c.bin", false).expect("load 0700000c");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        for _ in 0..100 { sim.step_bond(Input::default(), false, false, false); }
    }
    println!("== unburned rays ==");
    for &y in &[18.6f32, 18.4, 18.2, 18.0, 17.8, 17.4, 16.0, 14.5] {
        for &x in &[5.0f32, 5.75] {
            let from = [x, y, -109.5];
            let to = [x, y, -113.5];
            let hit = scene.sim.as_ref().unwrap().world.segment_hit(from, to);
            match hit {
                Some((t, n)) => {
                    let z = -109.5 + t * (-113.5 - -109.5);
                    println!("  y{y} x{x}: hit at z {:.3} normal {:?}", z, n);
                }
                None => println!("  y{y} x{x}: clear"),
            }
        }
    }
    // Burn + rays again.
    {
        let sim = scene.sim.as_mut().unwrap();
        for k in 0..sim.drones.locks.len() {
            if (14..=17).contains(&sim.drones.locks[k].out) {
                sim.drones.locks[k].burn_through(&mut sim.drones.channels, 1);
            }
        }
    }
    for _ in 0..400 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    println!("== burned rays (400 ticks after the burn) ==");
    for &y in &[18.6f32, 18.4, 18.2, 18.0, 17.8, 17.4, 16.0, 14.5] {
        for &x in &[5.0f32, 5.75] {
            let from = [x, y, -109.5];
            let to = [x, y, -113.5];
            let hit = scene.sim.as_ref().unwrap().world.segment_hit(from, to);
            match hit {
                Some((t, n)) => {
                    let z = -109.5 + t * (-113.5 - -109.5);
                    println!("  y{y} x{x}: hit at z {:.3} normal {:?}", z, n);
                }
                None => println!("  y{y} x{x}: clear"),
            }
        }
    }
    // The SS #179/#138 poses + the blocker's own keys.
    {
        let sim = scene.sim.as_ref().unwrap();
        for pi in &[179usize, 138] {
            if let Some(idx) = sim.drones.sss.of_placement(*pi) {
                let ss = &sim.drones.sss.list[idx];
                let m = &ss.matrix;
                println!("  SS #{pi}: state {:#x} t {} rate {} keys {} matrix trans {:.3} {:.3} {:.3}",
                    ss.state, ss.t, ss.rate, ss.keys.len(), m[3][0], m[3][1], m[3][2]);
                for (ki, k) in ss.keys.iter().enumerate() {
                    println!("    key{ki}: pos {:.3} {:.3} {:.3}", k.pos[0], k.pos[1], k.pos[2]);
                }
            }
        }
    }
    // The dynamic triangles inside the mouth box.
    let tris = scene.sim.as_ref().unwrap().world.dynamic_tris_near([3.5, 14.0, -113.5], [8.5, 21.0, -109.5]);
    println!("  dynamic tris in the mouth box: {}", tris.len());
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for t in &tris {
        for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } }
    }
    if !tris.is_empty() {
        println!("  their union box: ({:.2},{:.2},{:.2})..({:.2},{:.2},{:.2})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        for (i, t) in tris.iter().take(8).enumerate() {
            println!("    t{i}: ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
                t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
        }
    }
}
