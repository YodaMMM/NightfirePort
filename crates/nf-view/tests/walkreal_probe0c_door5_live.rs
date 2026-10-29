//! walkreal probe #5 (level-0c-door2turbine-3): the band crossing, part 3 — the SS (scripted
//! sequence) objects. A bridge across the pit can only be a mover (the static collision and the
//! drone roster have nothing there [D probes 3/4]); SS objects carry keyframe paths, so a mover
//! shows as key0 != keyN. Dump every level SS whose path touches the crossing box, every wire,
//! every trigger regardless of height, and every whole-level mover (key0 != keyN) as fallback.
//! Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn in_box(p: [f32; 3]) -> bool {
    p[0] > 40.0 && p[0] < 80.0 && p[2] > -216.0 && p[2] < -194.0
}

#[test]
fn probe_ss_movers() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    eprintln!("--- SS objects near the crossing box (any key or rest position in it) ---");
    for (i, s) in sim.drones.sss.list.iter().enumerate() {
        let at = [s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]];
        let first = s.keys.first().map(|k| k.pos).unwrap_or([0.0; 3]);
        let last = s.keys.last().map(|k| k.pos).unwrap_or([0.0; 3]);
        let key_hit = s.keys.iter().any(|k| in_box(k.pos));
        if in_box(at) || key_hit {
            eprintln!("ss[{i}] placement={} ch={:#04x} state={:#x} mask={:#x} keys={} rate={} wrap={} rest={:.3} spline={} hash={:#x} at=({:.2},{:.2},{:.2}) key0=({:.2},{:.2},{:.2}) keyN=({:.2},{:.2},{:.2})",
                s.placement, s.channel, s.state, s.mask, s.keys.len(), s.rate, s.wrap_flag, s.rest, s.spline, s.hash,
                at[0], at[1], at[2], first[0], first[1], first[2], last[0], last[1], last[2]);
            for (ki, k) in s.keys.iter().enumerate() {
                eprintln!("   key{ki} ({:.2},{:.2},{:.2})", k.pos[0], k.pos[1], k.pos[2]);
            }
        }
    }

    eprintln!("--- whole-level MOVERS (key0 != keyN, any distance) ---");
    for (i, s) in sim.drones.sss.list.iter().enumerate() {
        let (Some(f), Some(l)) = (s.keys.first().map(|k| k.pos), s.keys.last().map(|k| k.pos)) else { continue };
        let d = ((f[0] - l[0]).powi(2) + (f[1] - l[1]).powi(2) + (f[2] - l[2]).powi(2)).sqrt();
        if d > 1.0 {
            eprintln!("ss[{i}] placement={} ch={:#04x} state={:#x} mask={:#x} keys={} rate={} wrap={} travel {:.1} at=({:.2},{:.2},{:.2}) key0=({:.2},{:.2},{:.2}) keyN=({:.2},{:.2},{:.2})",
                s.placement, s.channel, s.state, s.mask, s.keys.len(), s.rate, s.wrap_flag, d,
                s.matrix[3][0], s.matrix[3][1], s.matrix[3][2], f[0], f[1], f[2], l[0], l[1], l[2]);
        }
    }

    eprintln!("--- wires (all) ---");
    for w in &sim.wires {
        let (sy, cy) = w.yaw.sin_cos();
        let a = [w.centre[0] - sy * w.radius, w.centre[1], w.centre[2] - cy * w.radius];
        let b = [w.centre[0] + sy * w.radius, w.centre[1], w.centre[2] + cy * w.radius];
        eprintln!("wire at=({:.2},{:.2},{:.2}) r={:.1} yaw={:.2} type={} ch={} A=({:.2},{:.2},{:.2}) B=({:.2},{:.2},{:.2})",
            w.centre[0], w.centre[1], w.centre[2], w.radius, w.yaw, w.wire_type, w.channel, a[0], a[1], a[2], b[0], b[1], b[2]);
    }

    eprintln!("--- triggers x 40..80 z -216..-194 (any y) ---");
    for t in &sim.drones.triggers {
        if t.tris.is_empty() { continue; }
        let c = t.centre();
        if c[0] > 40.0 && c[0] < 80.0 && c[2] > -216.0 && c[2] < -194.0 {
            let list = t.list.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
            eprintln!("trig kind={:2} out={:#04x} gate={} key={} mask={:#04x} at=({:.2},{:.2},{:.2}) list=[{}]",
                t.kind, t.out, t.gate, t.key_gate, t.mask, c[0], c[1], c[2], list);
        }
    }

    eprintln!("ok");
}
