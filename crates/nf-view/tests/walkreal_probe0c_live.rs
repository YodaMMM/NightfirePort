//! walkreal probe for 0700000c (level-0c-every-objective): the route-relevant geometry —
//! floors along the south edge of the start walkway, the bay/duct/vent data, the skylight
//! plates, the wires/ziplines with their ends, the winch SS keys, the circle ring floors,
//! the band, the warehouse ground, the secret-door district and the exit gate. Diagnostic
//! only — run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_0700000c() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== 0700000c probe ===");
    let b = &sim.bond;
    eprintln!("start: bond=({:.2},{:.2},{:.2}) yaw={:.3} disabled={}", b.pos[0], b.pos[1], b.pos[2], b.yaw, sim.disabled);

    eprintln!("--- triggers (all non-empty + AND/OR/OUT/LOAD) ---");
    for t in &sim.drones.triggers {
        let c = t.centre();
        let list = t.list.iter().map(|x| format!("{x:#x}")).collect::<Vec<_>>().join(",");
        eprintln!("trig kind={:2} out={:#04x} gate={} key={} mask={:#04x} at=({:.2},{:.2},{:.2}) list=[{}]",
            t.kind, t.out, t.gate, t.key_gate, t.mask, c[0], c[1], c[2], list);
    }
    eprintln!("--- simple scripts (SS) ---");
    for (i, s) in sim.drones.sss.list.iter().enumerate() {
        let first = s.keys.first().map(|k| k.pos).unwrap_or([0.0; 3]);
        let last = s.keys.last().map(|k| k.pos).unwrap_or([0.0; 3]);
        eprintln!("ss[{i}] placement={} ch={:#04x} state={:#x} mask={:#x} keys={} rate={} wrap={} rest={:.3} spline={} hash={:#x} at=({:.2},{:.2},{:.2}) key0=({:.2},{:.2},{:.2}) keyN=({:.2},{:.2},{:.2})",
            s.placement, s.channel, s.state, s.mask, s.keys.len(), s.rate, s.wrap_flag, s.rest, s.spline, s.hash,
            s.matrix[3][0], s.matrix[3][1], s.matrix[3][2], first[0], first[1], first[2], last[0], last[1], last[2]);
    }
    eprintln!("--- wires ---");
    for w in &sim.wires {
        let (sy, cy) = w.yaw.sin_cos();
        let a = [w.centre[0] - sy * w.radius, w.centre[1], w.centre[2] - cy * w.radius];
        let b_end = [w.centre[0] + sy * w.radius, w.centre[1], w.centre[2] + cy * w.radius];
        eprintln!("wire at=({:.2},{:.2},{:.2}) r={:.1} yaw={:.2} type={} ch={} A=({:.2},{:.2},{:.2}) B=({:.2},{:.2},{:.2})",
            w.centre[0], w.centre[1], w.centre[2], w.radius, w.yaw, w.wire_type, w.channel, a[0], a[1], a[2], b_end[0], b_end[1], b_end[2]);
    }
    eprintln!("--- doors ---");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        eprintln!("door[{i}] at=({:.1},{:.1},{:.1}) kind={} ch_open={:#x} ch_lock={:#x} lock_flag={} approach={} state={:#x}",
            d.pos[0], d.pos[1], d.pos[2], d.kind, d.ch_open, d.ch_lock, d.locked_flag, d.approach, d.state);
    }
    eprintln!("--- locks ---");
    for l in &sim.drones.locks {
        eprintln!("lock placement={} at=({:.2},{:.2},{:.2}) r={:.1} out={:#04x} keys={:?}",
            l.placement, l.pos[0], l.pos[1], l.pos[2], l.radius, l.out, l.key_channels);
    }
    eprintln!("--- ladders ---");
    for (c, top, bot) in walker::ladders(sim) {
        eprintln!("ladder at=({:.2},{:.2},{:.2}) top={top:.2} bottom={bot:.2}", c[0], c[1], c[2]);
    }

    // Floor probes: straight down at named spots (segment from y+3 to y-6 below the spot's y).
    let spots: &[(&str, [f32; 3])] = &[
        ("start walkway", [8.98, 15.07, -149.65]),
        ("walkway east of corner x16", [16.0, 15.6, -152.6]),
        ("walkway south edge x18", [18.0, 15.6, -153.0]),
        ("walkway south edge x20", [20.0, 15.6, -153.0]),
        ("walkway south edge x25", [25.0, 15.6, -153.0]),
        ("walkway z-156 x18", [18.0, 15.6, -156.0]),
        ("lower walkway south end", [5.84, 12.93, -154.9]),
        ("lower walkway mid z-130", [5.8, 12.93, -130.0]),
        ("lower walkway north z-115", [5.8, 12.93, -115.0]),
        ("plate274 spot", [5.79, 14.09, -111.10]),
        ("balcony west", [0.0, 14.61, -115.0]),
        ("balcony east", [13.0, 14.61, -115.0]),
        ("east deck 12.14", [11.0, 12.14, -120.0]),
        ("staircase #359", [10.0, 10.0, -125.0]),
        ("display ground", [5.4, 7.35, -113.0]),
        ("case floor east", [9.5, 7.35, -113.0]),
        ("case floor east2", [10.5, 7.35, -112.0]),
        ("warehouse ground near plate559", [52.76, 2.05, -173.59]),
        ("warehouse north floor", [50.0, 2.05, -160.0]),
        ("warehouse floor z-165", [53.8, 6.9, -165.36]),
        ("warehouse floor z-168", [53.8, 6.9, -168.0]),
        ("yard floor", [56.5, 1.8, -176.5]),
        ("yard mid", [40.0, 1.4, -184.5]),
        ("circle NW", [10.1, 11.7, -205.4]),
        ("circle west bend", [3.4, 11.2, -205.4]),
        ("circle SW bend", [3.4, 11.2, -209.0]),
        ("circle south", [20.0, 11.2, -208.0]),
        ("circle south mid", [38.0, 11.2, -208.0]),
        ("circle south east", [55.0, 11.2, -209.0]),
        ("band east", [90.0, 11.6, -208.8]),
        ("gate", [96.0, 11.6, -209.3]),
    ];
    eprintln!("--- floor probes (down from y+2, 12 deep) ---");
    for (name, at) in spots {
        let hit = sim.world.segment_hit([at[0], at[1] + 2.0, at[2]], [at[0], at[1] - 10.0, at[2]]);
        match hit {
            Some((t, _n)) => {
                let y = at[1] + 2.0 - 12.0 * t;
                eprintln!("floor {name} @({:.1},{:.1},{:.1}): y={:.3}", at[0], at[1], at[2], y);
            }
            None => eprintln!("floor {name} @({:.1},{:.1},{:.1}): NONE in y {:.1}..{:.1}", at[0], at[1], at[2], at[1] + 2.0, at[1] - 10.0),
        }
    }
    // The start walkway's south edge: scan the wall/drop profile for x 5..26 at z from -150 to -157,
    // y from 16 down to 8 — where does a segment heading south first pass?
    eprintln!("--- south edge scan (z -151..-158 step 1, x 5..26 step 3; segment x,15.6,z -> x,8,z) ---");
    for z in [-151.0, -152.0, -153.0, -154.0, -155.0, -156.0, -157.0, -158.0] {
        for x in [5.0f32, 8.0, 11.0, 14.0, 17.0, 20.0, 23.0, 26.0] {
            let hit = sim.world.segment_hit([x, 15.6, z], [x, 8.0, z]);
            match hit {
                Some((t, _)) => eprintln!("  x={x} z={z}: first hit y={:.2}", 15.6 - 7.6 * t),
                None => eprintln!("  x={x} z={z}: clear to y 8"),
            }
        }
    }
    // The circle ring: floors along the ring every 15 degrees (centre ~(46,-209)? use the
    // nav path 1 ring — probe a grid instead).
    eprintln!("--- circle grid floors (x 0..96 step 8, z -212..-200 step 2) ---");
    for x in (0..=96).step_by(8) {
        for z in [-212.0f32, -210.0, -208.0, -206.0, -204.0, -202.0, -200.0] {
            let hit = sim.world.segment_hit([x as f32, 13.0, z], [x as f32, 8.0, z]);
            if let Some((t, _)) = hit {
                let y = 13.0 - 5.0 * t;
                eprintln!("  ring x={x} z={z}: floor y={:.2}", y);
            }
        }
    }
}
