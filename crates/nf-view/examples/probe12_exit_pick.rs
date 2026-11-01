//! level-12-exit-pick probe: the use pick from inside the security room near (1.12, 7.49) facing
//! north — Bond's cel, the use point, exit #669's volume and its `build_FindCel` cel, and the
//! straddle set the use sphere (r 1.0 at head + look) reaches from Bond's cel through portals
//! [M Collide_SphereIntersect 0x1ec940 → Collide_Pick 0x1eb320 → Collide_StraddleCels 0x1eac98 /
//! Intersect_Portal 0x1ea7b8 v0x101]. Prints only.
use nf_game::player::Input;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000012.bin", false).expect("load 07000012");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
    doors(sim);
    let ti = sim.drones.triggers.iter().position(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL).expect("exit");
    {
        let t = &sim.drones.triggers[ti];
        println!("exit placement {} kind {} mask {:#x} gate {:#x} lo {:?} hi {:?} tris {}", t.placement, t.kind, t.mask, t.gate, t.lo, t.hi, t.tris.len());
        for tr in &t.tris { println!("  tri {:?}", tr); }
    }
    let x = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1.12f32);
    let z = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(7.49f32);
    sim.bond.pos = [x, sim.bond.pos[1], z];
    sim.bond.yaw = 0.0;
    sim.bond.pitch = 0.0;
    sim.bond.vel = [0.0; 3];
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    let t = &sim.drones.triggers[ti];
    let c = [(t.lo[0] + t.hi[0]) / 2.0, (t.lo[1] + t.hi[1]) / 2.0, (t.lo[2] + t.hi[2]) / 2.0];
    let head = sim.bond.head();
    let look = [sim.bond.yaw.sin() * sim.bond.pitch.cos(), 0.0, sim.bond.yaw.cos()];
    let up = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
    println!("bond pos {:?} yaw {} cel {:?} head {:?} use point ~{:?}", sim.bond.pos, sim.bond.yaw, sim.bond.cel, head, up);
    let tcel = nf_view::scene::find_cell_raycast(&sim.cel_graph, c, Some(&sim.world));
    println!("exit box centre {:?} -> build_FindCel cel {:?}", c, tcel);
    for (i, n) in sim.cel_graph.cells.iter().enumerate() {
        if (0..3).all(|k| n.min[k] <= c[k] && c[k] <= n.max[k]) || (0..3).all(|k| n.min[k] <= up[k] && up[k] <= n.max[k]) {
            println!("  cel {} flags {:#x} min {:?} max {:?}", i, n.flags, n.min, n.max);
        }
    }
    if let Some(bc) = sim.bond.cel {
        let set = nf_view::scene::icon_straddle_cels(&sim.cel_graph, bc, up, 1.0);
        println!("use-sphere straddle set from cel {}: {:?}", bc, set);
        for p in &sim.cel_graph.portals {
            if p.cel_a == bc || p.cel_b == bc {
                let mut mn = p.corners[0]; let mut mx = p.corners[0];
                for q in &p.corners[1..] { for k in 0..3 { mn[k] = mn[k].min(q[k]); mx[k] = mx[k].max(q[k]); } }
                println!("  portal {}<->{} aabb {:?}..{:?}", p.cel_a, p.cel_b, mn, mx);
            }
        }
    }
    // The real E press from here.
    let mut fired = false;
    for _ in 0..10 {
        sim.step_bond(Input::default(), false, false, true);
        if sim.drones.level_exit.is_some() { fired = true; break; }
    }
    println!("E press fires the exit: {fired} ({:?})", sim.drones.level_exit);
}

#[allow(dead_code)]
fn doors(sim: &nf_view::scene::Sim) {
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        println!("door[{i}] plc {} pos {:?} kind {} state {} glist {:#x}", d.placement, d.pos, d.kind, d.state, d.glist);
    }
}
