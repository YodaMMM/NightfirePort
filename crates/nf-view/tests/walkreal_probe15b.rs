//! probe 07000015's lower deck ladder base (ladder #382 at (84.53, 31.73, −2.29); the deck at
//! y 23.8) and the north band east of door[47] (the 007 #813 at (53.40, 40.57, 71.14)): the
//! floor grids, the placements (furniture), the ladders' live records and the nav routes.
//! Diagnostic only — run with `--ignored --nocapture`.

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
fn probe_ladder_deck_and_north_band() {
    let iso_path = iso();

    // 1. The placements near the ladder and on the north band, straight from the disc [D].
    let data = read_archive(&iso_path, "07000015.bin");
    let lv = Level::parse(&data).expect("parse");
    eprintln!("=== placements (main map) in x 74..92 z −12..6 (the ladder deck) ===");
    if let Some(m) = lv.maps.get(54) {
        for p in &m.placements {
            let t = p.translation;
            if (74.0..92.0).contains(&t[0]) && (-12.0..6.0).contains(&t[2]) {
                let params = p.params.iter().map(|(a, b)| format!("({a:#x},{b:#x})")).collect::<Vec<_>>().join(",");
                eprintln!("plc{} hash={:#010x} flags={:#010x} at=({:.2},{:.2},{:.2}) rot=({:.2},{:.2},{:.2}) scale=({:.2},{:.2},{:.2}) params=[{}]",
                    p.index, p.hashcode, p.flags, t[0], t[1], t[2],
                    p.rotation[0], p.rotation[1], p.rotation[2],
                    p.scale()[0], p.scale()[1], p.scale()[2], params);
            }
        }
    }
    eprintln!("=== placements (main map) in x 34..58 z 60..80 (the north band) ===");
    if let Some(m) = lv.maps.get(54) {
        for p in &m.placements {
            let t = p.translation;
            if (34.0..58.0).contains(&t[0]) && (60.0..80.0).contains(&t[2]) {
                let params = p.params.iter().map(|(a, b)| format!("({a:#x},{b:#x})")).collect::<Vec<_>>().join(",");
                eprintln!("plc{} hash={:#010x} flags={:#010x} at=({:.2},{:.2},{:.2}) rot=({:.2},{:.2},{:.2}) scale=({:.2},{:.2},{:.2}) params=[{}]",
                    p.index, p.hashcode, p.flags, t[0], t[1], t[2],
                    p.rotation[0], p.rotation[1], p.rotation[2],
                    p.scale()[0], p.scale()[1], p.scale()[2], params);
            }
        }
    }

    // 2. The live sim: ladders, floor grids, nav routes.
    let scene = nf_view::load(&iso_path, "07000015.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();

    eprintln!("=== ladders (live) ===");
    for l in &sim.ladders {
        eprintln!("ladder centre=({:.2},{:.2},{:.2}) r={:.2} yaw={:.3} y_span=({:.2},{:.2}) half=({:.3},{:.3}) tris={}",
            l.centre[0], l.centre[1], l.centre[2], l.radius, l.yaw, l.y_span[0], l.y_span[1], l.half[0], l.half[1], l.tris.len());
    }

    eprintln!("=== floor grid at deck level (from y 25, depth 4), rows z 6..−12 step 1, cols x 74..92 step 1 ===");
    let mut z = 6.0f32;
    while z >= -12.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 74.0f32;
        while x <= 92.0 {
            let f = sim.world.floor_below([x, 25.0, z], 4.0);
            match f {
                Some(fy) => {
                    let d = 25.0 - fy;
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

    eprintln!("=== floor grid at deck level (from y 24.6, depth 3), rows z 6..−12 step 0.5, cols x 80..90 step 0.5 (ladder close-up) ===");
    let mut z = 6.0f32;
    while z >= -12.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 80.0f32;
        while x <= 90.0 {
            let f = sim.world.floor_below([x, 24.6, z], 3.0);
            match f {
                Some(fy) => {
                    let d = 24.6 - fy;
                    if d < 0.0 { row.push('^'); }
                    else if d < 2.0 { row.push('#'); }
                    else { row.push('.'); }
                }
                None => row.push(' '),
            }
            x += 0.5;
        }
        eprintln!("{row}");
        z -= 0.5;
    }

    eprintln!("=== floor grid at gantry level (from y 42, depth 20 — what is below the gantry x 74..92) ===");
    let mut z = 6.0f32;
    while z >= -12.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 74.0f32;
        while x <= 92.0 {
            let f = sim.world.floor_below([x, 42.0, z], 20.0);
            match f {
                Some(fy) => {
                    let d = 42.0 - fy;
                    if d < 2.5 { row.push('#'); }
                    else if (15.0..20.0).contains(&d) { row.push('d'); }
                    else { row.push('.'); }
                }
                None => row.push(' '),
            }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 1.0;
    }

    eprintln!("=== floor grid at gantry level (from y 42, depth 8), rows z 78..60 step 1, cols x 34..58 step 1 (the north band) ===");
    let mut z = 78.0f32;
    while z >= 60.0 {
        let mut row = format!("z {:5.1}: ", z);
        let mut x = 34.0f32;
        while x <= 58.0 {
            let f = sim.world.floor_below([x, 42.0, z], 8.0);
            match f {
                Some(fy) => {
                    let d = 42.0 - fy;
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

    eprintln!("=== nav route spawn → the ladder base (84.5, 23.8, −2.29) ===");
    let route = walker::nav_route_global(sim, [94.5, 41.38, 0.0], [84.5, 23.8, -2.29]);
    match route {
        Some(pts) => {
            eprintln!("route ({} nodes):", pts.len());
            for p in &pts { eprintln!("  ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
        }
        None => eprintln!("no global nav route to the ladder base"),
    }

    eprintln!("=== nav route door[47] north (34.6, 41.4, 65) → the 007 #813 (53.4, 40.57, 71.14) ===");
    let route2 = walker::nav_route_global(sim, [34.6, 41.4, 65.0], [53.4, 41.4, 71.14]);
    match route2 {
        Some(pts) => {
            eprintln!("route ({} nodes):", pts.len());
            for p in &pts { eprintln!("  ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
        }
        None => eprintln!("no global nav route door[47] → #813"),
    }

    eprintln!("=== the doors near the ladder deck x 70..94 z −12..6 ===");
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if (70.0..94.0).contains(&d.pos[0]) && (-12.0..6.0).contains(&d.pos[2]) {
            eprintln!("door[{i}] plc={} at=({:.2},{:.2},{:.2}) kind={} straddle={} ch_open={:#x} ch_lock={:#x} state={:#x}",
                d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.straddle, d.ch_open, d.ch_lock, d.state);
        }
    }

    eprintln!("=== collision tris x 80..89 y 22..27 z −6..2 (the wall the teleport hit) ===");
    for t in sim.world.tris_near([80.0, 22.0, -6.0], [89.0, 27.0, 2.0]) {
        eprintln!("tri ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2}) ({:.2},{:.2},{:.2})",
            t[0][0], t[0][1], t[0][2], t[1][0], t[1][1], t[1][2], t[2][0], t[2][1], t[2][2]);
    }
}
