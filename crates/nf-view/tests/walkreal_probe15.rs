//! probe 07000015's door[32] corner (the walkreal stop at (29.5, 41.45, 69) — the west-wall
//! doorway at z 70.1): the floor grid, the collision triangles, the placements (furniture),
//! the door record and the capsule push at the stuck spot. Diagnostic only — run --nocapture.

#[path = "common/walker.rs"]
mod walker;

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_door32_corner() {
    let iso_path = iso();

    // 1. The placements (furniture) near the corner, straight from the disc [D].
    let data = read_archive(&iso_path, "07000015.bin");
    let lv = Level::parse(&data).expect("parse");
    eprintln!("=== placements in x 20..36 z 60..80 (any y) ===");
    for (mi, m) in lv.maps.iter().enumerate() {
        for p in &m.placements {
            let t = p.translation;
            if (20.0..36.0).contains(&t[0]) && (60.0..80.0).contains(&t[2]) {
                let params = p.params.iter().map(|(a, b)| format!("({a:#x},{b:#x})")).collect::<Vec<_>>().join(",");
                eprintln!("map{mi} plc{} hash={:#010x} flags={:#010x} at=({:.2},{:.2},{:.2}) rot=({:.2},{:.2},{:.2}) scale=({:.2},{:.2},{:.2}) params=[{}]",
                    p.index, p.hashcode, p.flags, t[0], t[1], t[2],
                    p.rotation[0], p.rotation[1], p.rotation[2],
                    p.scale()[0], p.scale()[1], p.scale()[2], params);
            }
        }
    }

    // 2. The live sim: floor grid, collision tris, the door record, the capsule push.
    let scene = nf_view::load(&iso_path, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("=== floor grid y 41.5 depth 8, rows z 78..60 step 1, cols x 20..36 step 1 ===");
    let mut z = 78.0f32;
    while z >= 60.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 20.0f32;
        while x <= 36.0 {
            let f = sim.world.floor_below([x, 41.5, z], 8.0);
            match f {
                Some(fy) => {
                    let d = 41.5 - fy;
                    if d < 0.0 { row.push('^'); }
                    else if d < 2.5 { row.push('#'); }
                    else { row.push('.'); }
                }
                None => row.push(' '),
            }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 1.0;
    }

    eprintln!("=== doors in x 18..38 z 60..80 ===");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (18.0..38.0).contains(&d.pos[0]) && (60.0..80.0).contains(&d.pos[2]) {
            let (sc, sr) = d.sph;
            eprintln!("door[{i}] plc={} at=({:.2},{:.2},{:.2}) kind={} straddle={} ch_open={:#x} ch_lock={:#x} lock_flag={} approach={} state={:#x} sph=({:.2},{:.2},{:.2}) r={:.2} flags={:#x}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.straddle, d.ch_open, d.ch_lock, d.locked_flag, d.approach, d.state, sc[0], sc[1], sc[2], sr, d.flags);
        }
    }

    eprintln!("=== collision tris in x 24..32 y 39..46 z 66..74 ===");
    for t in sim.world.tris_near([24.0, 39.0, 66.0], [32.0, 46.0, 74.0]) {
        eprintln!("tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }

    eprintln!("=== the stuck spot: push probes from (29.55, 41.45, 68.97) ===");
    for (label, a, b) in [
        ("to threshold (27.6,41.4,70.1) head-height", [29.55f32, 42.4, 68.97], [27.6, 42.4, 70.1]),
        ("to threshold feet", [29.55f32, 41.45, 68.97], [27.6, 41.45, 70.1]),
        ("to mid-door (26.0,41.4,70.1)", [29.55f32, 41.45, 68.97], [26.0, 41.45, 70.1]),
    ] {
        let seg = sim.world.segment(a, b);
        let (push, _w) = sim.world.capsule_push(a, b, 0.4, false, [1.0e9, 1.0e9, 1.0e9]);
        eprintln!("{label}: segment={seg:?} capsule_push a->b push=({:.3},{:.3},{:.3})", push[0], push[1], push[2]);
    }

    // the capsule exactly at the walker's stuck pose
    let (a, c) = capsule_at([29.55f32, 41.45, 68.97]);
    let (push, walkable) = sim.world.capsule_push(a, c, 0.4, false, [1.0e9, 1.0e9, 1.0e9]);
    eprintln!("stuck capsule push=({:.3},{:.3},{:.3}) walkable={}", push[0], push[1], push[2], walkable);
    let (a2, c2) = capsule_at([29.55f32, 41.45, 70.1]);
    let (push2, walkable2) = sim.world.capsule_push(a2, c2, 0.4, false, [1.0e9, 1.0e9, 1.0e9]);
    eprintln!("at-threshold capsule push=({:.3},{:.3},{:.3}) walkable={}", push2[0], push2[1], push2[2], walkable2);
}

fn capsule_at(p: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    // the walker's capsule: feet p, head p + 1.9 (Bond's capsule_seg — the probe only needs the
    // shape, not the exact game height)
    (p, [p[0], p[1] + 1.9, p[2]])
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_nav_route_to_exit() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    // The stuck spot east of door[32] → the west band → the exit trigger.
    for (label, from, to) in [
        ("stuck spot -> west band", [29.55f32, 41.45, 68.97], [21.0, 41.4, 70.0]),
        ("stuck spot -> the exit trigger", [29.55f32, 41.45, 68.97], [0.0, 41.3, -24.8]),
        ("spawn -> the exit trigger", [94.5f32, 41.38, 0.0], [0.0, 41.3, -24.8]),
    ] {
        eprintln!("=== nav {label} ===");
        match walker::nav_route(sim, from, to).or_else(|| walker::nav_route_global(sim, from, to)) {
            Some(pts) => {
                for (i, p) in pts.iter().enumerate() { eprintln!("  {i}: ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
            }
            None => eprintln!("  NO ROUTE"),
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_door29_lane() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("=== floor grid y 41.5 depth 8, rows z 80..58 step 1, cols x 0..20 step 1 ===");
    let mut z = 80.0f32;
    while z >= 58.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 0.0f32;
        while x <= 20.0 {
            let f = sim.world.floor_below([x, 41.5, z], 8.0);
            match f {
                Some(fy) => {
                    let d = 41.5 - fy;
                    if d < 0.0 { row.push('^'); }
                    else if d < 2.5 { row.push('#'); }
                    else { row.push('.'); }
                }
                None => row.push(' '),
            }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 1.0;
    }

    eprintln!("=== doors in x 0..20 z 60..80 ===");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (0.0..20.0).contains(&d.pos[0]) && (60.0..80.0).contains(&d.pos[2]) {
            eprintln!("door[{i}] plc={} at=({:.2},{:.2},{:.2}) kind={} straddle={} state={:#x} flags={:#x} sph=({:.2},{:.2},{:.2}) r={:.2}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.straddle, d.state, d.flags, d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
        }
    }

    eprintln!("=== placements in x 2..12 z 64..80 y 39..45 ===");
    let data = read_archive(&iso, "07000015.bin");
    let lv = Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        for p in &m.placements {
            let t = p.translation;
            if (2.0..12.0).contains(&t[0]) && (64.0..80.0).contains(&t[2]) && (39.0..45.0).contains(&t[1]) {
                eprintln!("map{mi} plc{} hash={:#010x} flags={:#010x} at=({:.2},{:.2},{:.2}) rot=({:.2},{:.2},{:.2})",
                    p.index, p.hashcode, p.flags, t[0], t[1], t[2], p.rotation[0], p.rotation[1], p.rotation[2]);
            }
        }
    }

    eprintln!("=== wedge analysis at (5.58, 41.46, 72.78) ===");
    for (label, to) in [
        ("to (4.0, 70.0)", [4.0f32, 41.4, 70.0]),
        ("to (5.0, 71.5)", [5.0f32, 41.4, 71.5]),
        ("to (4.0, 73.0)", [4.0f32, 41.4, 73.0]),
        ("to (8.0, 70.0)", [8.0f32, 41.4, 70.0]),
    ] {
        let seg = sim.world.segment([5.58f32, 42.4, 72.78], [to[0], 42.4, to[2]]);
        let (push, _w) = sim.world.capsule_push([5.58f32, 41.46, 72.78], [5.58f32, 43.36, 72.78], 0.4, false, [1.0e9, 1.0e9, 1.0e9]);
        let floor_to = sim.world.floor_below(to, 8.0);
        eprintln!("{label}: seg={seg:?} stuck_push=({:.3},{:.3},{:.3}) floor_at_dest={floor_to:?}",
            push[0], push[1], push[2]);
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_south_lane() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("=== floor grid y 41.5 depth 8, rows z 62..-30 step 2, cols x -10..8 step 1 ===");
    let mut z = 62.0f32;
    while z >= -30.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = -10.0f32;
        while x <= 8.0 {
            let f = sim.world.floor_below([x, 41.5, z], 8.0);
            match f {
                Some(fy) => {
                    let d = 41.5 - fy;
                    if d < 0.0 { row.push('^'); }
                    else if d < 2.5 { row.push('#'); }
                    else { row.push('.'); }
                }
                None => row.push(' '),
            }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 2.0;
    }

    eprintln!("=== doors in x -10..8 z 60..-30 ===");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (-10.0..8.0).contains(&d.pos[0]) && (-30.0..60.0).contains(&d.pos[2]) {
            eprintln!("door[{i}] plc={} at=({:.2},{:.2},{:.2}) kind={} state={:#x} flags={:#x} ch_open={:#x} ch_lock={:#x} sph=({:.2},{:.2},{:.2}) r={:.2}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.state, d.flags, d.ch_open, d.ch_lock,
                d.sph.0[0], d.sph.0[1], d.sph.0[2], d.sph.1);
        }
    }
    eprintln!("=== triggers in x -10..8 z 60..-30 ===");
    for t in &sim.drones.triggers {
        let c = t.centre();
        if (-10.0..8.0).contains(&c[0]) && (-30.0..60.0).contains(&c[2]) {
            eprintln!("trig kind={:2} out={:#04x} gate={} next={:#x} at=({:.1},{:.1},{:.1})", t.kind, t.out, t.gate, t.next_level, c[0], c[1], c[2]);
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_triggers_and_scripts() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("=== ALL triggers ===");
    for (i, t) in sim.drones.triggers.iter().enumerate() {
        let c = t.centre();
        eprintln!("trig[{i}] kind={:2} out={:#04x} gate={} key={} next={:#x} at=({:.1},{:.1},{:.1}) tris={}",
            t.kind, t.out, t.gate, t.key_gate, t.next_level, c[0], c[1], c[2], t.tris.len());
    }
    eprintln!("=== scripts (the cut-scene holders) ===");
    for (h, s) in &sim.scripts {
        eprintln!("script {h:#010x}: commands={} looping={}", s.commands.len(), s.looping);
    }
    eprintln!("=== mission states at load ===");
    for (r, st) in sim.drones.mission.states.iter().enumerate() {
        if st.iter().any(|&s| s != 0) {
            eprintln!("record {r}: {:?}", st);
        }
    }
}
