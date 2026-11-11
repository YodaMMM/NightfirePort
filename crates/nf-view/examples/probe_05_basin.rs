//! Scratch (level-05-basin-floor): why does the port's room-14 collision build drop the east-basin
//! floor? Measured on s/level-05-valley-hurt: past the lip (132.0,14.17,125.37) every point
//! resolves find_cel = room 14, and room 14's collision build covers the east basin but has NO
//! floor quads there; the west basin floor is room 15 and works. Basin = hurt #3's band
//! (x 80.78..168.38, z 96.66..144.68, floor y ~5.1). Dumps:
//!   1. room cels whose bbox meets the basin box (index -> cell, bbox, class);
//!   2. debug_bucket_owners over the basin box (which scope buckets hold runs there);
//!   3. per probe point: the eye cell (find_cel), the scoped down ray, the flat down ray,
//!      and the near-horizontal tris found under the point;
//!   4. every placement whose transformed collision bbox meets the basin box: kind, flags word,
//!      creates_cel, own graph cell, room class, chain owner (final bucket);
//!   5. for each such placement, whether its own room bucket or final bucket actually holds runs.
//! Assertion-free. Run: NF_ISO=<iso> cargo run -p nf-view --release --example probe_05_basin

use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

const BASIN: ([f32; 3], [f32; 3]) = ([78.0, -12.0, 94.0], [172.0, 16.0, 148.0]);
const POINTS: &[(&str, [f32; 3])] = &[
    ("lip-top", [132.0, 14.17, 125.37]),
    ("lip-below", [132.0, 11.0, 125.37]),
    ("basin-ne", [150.0, 10.0, 130.0]),
    ("basin-e", [165.0, 10.0, 138.0]),
    ("basin-n", [120.0, 10.0, 115.0]),
    ("basin-nw", [95.0, 10.0, 105.0]),
    ("basin-se", [140.0, 10.0, 142.0]),
];

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000005.bin");
    let lv = Level::parse(&data).expect("parse");
    let sc = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let g = &sc.cel_graph;
    let world = &sc.sim.as_ref().expect("sim").world;

    println!("== rooms meeting the basin box (rooms {} ==)", sc.cels.len());
    for (ri, r) in sc.cels.iter().enumerate() {
        let meets = !(0..3).any(|k| r.max[k] < BASIN.0[k] || r.min[k] > BASIN.1[k]);
        if meets {
            let cell = g.room_to_cell.get(ri).copied().flatten();
            println!("  room {ri}: bbox {:?}", (r.min, r.max));
            println!("           cell {cell:?}  ambient {:?} water {}", r.ambient, r.water);
        }
    }

    println!("\n== scope buckets holding runs that meet the basin box ==");
    for (c, n) in world.debug_bucket_owners(BASIN.0, BASIN.1) {
        let label = if c < 0 { "always".to_string() } else {
            let f16 = g.cells.get(c as usize).map(|n| n.flags & 0xffff).unwrap_or(0);
            let room = g.room_to_cell.iter().position(|m| *m == Some(c as usize));
            format!("cell {c} f16 {f16:#x} room {room:?}")
        };
        println!("  bucket {label}: {n} runs");
    }

    println!("\n== probe points ==");
    for (tag, p) in POINTS {
        let mut state = nf_view::scene::ViewerCel::default();
        let eye = nf_view::scene::eye_cell(g, &mut state, *p, Some(world));
        let eye_room = eye.and_then(|c| g.room_to_cell.iter().position(|m| *m == Some(c)));
        let down = [p[0], p[1] - 12.0, p[2]];
        let scoped = eye.and_then(|c| world.segment_hit_scoped(Some(c), *p, down));
        let flat = world.segment_hit(*p, down);
        println!("  {tag} {p:?}: eye cell {eye:?} room {eye_room:?}");
        println!("      scoped down ray: {scoped:?}   flat down ray: {flat:?}");
        let lo = [p[0] - 2.0, p[1] - 12.0, p[2] - 2.0];
        let hi = [p[0] + 2.0, p[1] + 2.0, p[2] + 2.0];
        let mut floors: Vec<f32> = Vec::new();
        for t in world.tris_near(lo, hi) {
            let ys = [t[0][1], t[1][1], t[2][1]];
            let ny = ((t[1][0] - t[0][0]) * (t[2][1] - t[0][1]) - (t[1][1] - t[0][1]) * (t[2][0] - t[0][0])).abs();
            if ys.iter().all(|y| (*y - p[1]).abs() < 14.0) && ny > 1e-6 {
                // near-horizontal: normal mostly up or down
                let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
                let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if l > 0.0 && n[1].abs() / l > 0.9 {
                    let y = ys[0];
                    if !floors.contains(&(y * 64.0).round()) { floors.push(y); }
                }
            }
        }
        floors.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("      flat tri set: near-horizontal tri ys {:?}", floors);
    }

    println!("\n== placements whose transformed collision bbox meets the basin box ==");
    // The scene's own replay (scene.rs b9): which cell each pushed instance's placement owns.
    let main = lv.main_map().unwrap_or(0);
    let inst = lv.instances(true).0;
    let mut node_owner: Vec<Option<u32>> = vec![None; g.cells.len()];
    for (o, ch) in g.chains.iter().enumerate() {
        for &n in ch { if (n as usize) < node_owner.len() { node_owner[n as usize] = Some(o as u32); } }
    }
    for (ii, i) in inst.iter().enumerate() {
        let o = &lv.maps[i.map].objects[i.object];
        let Some(c) = o.collision.as_ref() else { continue };
        let m = &i.matrix;
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for &cx in &[o.min[0], o.max[0]] { for &cy in &[o.min[1], o.max[1]] { for &cz in &[o.min[2], o.max[2]] {
            let w = nf_formats::placement::transform(m, [cx, cy, cz]);
            for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
        } } }
        if (0..3).any(|k| hi[k] < BASIN.0[k] || lo[k] > BASIN.1[k]) { continue; }
        let p = &i.placement;
        let mut cell = None;
        // Replay the scene's inst_cells (parse-wide cell counter, sub-maps first, main last).
        let mut counter = 0usize;
        for (mi, mp) in lv.maps.iter().enumerate() {
            for pp in &mp.placements {
                if !pp.creates_cel() { continue; }
                if mi == i.map && pp.index == p.index && pp.hashcode == p.hashcode && pp.translation == p.translation {
                    cell = Some(counter);
                }
                counter += 1;
            }
        }
        let f16 = cell.and_then(|c| g.cells.get(c)).map(|n| n.flags & 0xffff).unwrap_or(0);
        let room = cell.and_then(|c| g.room_to_cell.iter().position(|m| *m == Some(c)));
        let bucket: Option<usize> = if matches!(f16, 0xc023 | 0xc047) { cell } else { cell.and_then(|c| node_owner.get(c).copied().flatten().map(|v| v as usize)) };
        let kind = p.flags & 0xff;
        println!("  inst {ii} map {} plc#{} kind {kind:#04x} flags {:#010x} cel {} tris {} @ {:?}", i.map, p.index, p.flags, p.creates_cel(), c.triangles().len(), p.translation);
        println!("      wbox {:?}", (lo, hi));
        println!("      own cell {cell:?} f16 {f16:#x} room {room:?} final bucket {bucket:?}");
    }
}
