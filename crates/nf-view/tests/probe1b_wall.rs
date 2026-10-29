//! Probe: the shaft wall at x ≈ 4.43 (z −8..0) and the shaft mouths at
//! z ±8.82. Assertion-free diagnostic, run with --ignored --nocapture.
//!
//! A. [D] every placement whose collision tris meet the wall box — name, kind, glist flags,
//!    0x8000, tri count, and each near-vertical triangle's plane x / normal / y / z extents /
//!    surface byte.
//! B. [D] the same for the two mouth boxes (z ±8.3..9.4) and the mid-shaft face (x 5.75..6.05).
//! C. [D] SP#141/#142: the placements, the live players, and script 0x06000818's FULL stream
//!    dump (every stream, key range, decoded commands).
//! D. The port world: segment / segment_bullet / segment_surface_flags (both directions)
//!    across the wall and the mouths, the capsule push at the float's stop, the cel graph
//!    around the shaft, and the bucket owners of the wall/mouth boxes.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::nis::Cmd;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn tri_box(t: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
    let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
    (lo, hi)
}

fn meets(lo: &[f32; 3], hi: &[f32; 3], blo: &[f32; 3], bhi: &[f32; 3]) -> bool {
    (0..3).all(|k| hi[k] >= blo[k] && lo[k] <= bhi[k])
}

/// The wall / mouth / mid-shaft boxes the dump walks.
const BOXES: &[(&str, [f32; 3], [f32; 3])] = &[
    ("wall x4.43", [4.28, -3.5, -8.3], [4.62, 17.0, 0.3]),
    ("mouth z+8.82", [3.8, -3.5, 8.3], [6.6, 17.0, 9.4]),
    ("mouth z-8.82", [3.8, -3.5, -9.4], [6.6, 17.0, -8.3]),
    ("midshaft x5.91", [5.72, 5.5, -9.4], [6.1, 13.0, 9.4]),
];

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe1b_wall_and_mouths() {
    let iso = iso();
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);

    for (label, blo, bhi) in BOXES {
        println!("\n=== A/B. box {label}: owners (x {:+.2}..{:+.2}, y {:+.2}..{:+.2}, z {:+.2}..{:+.2}) [D] ===",
            blo[0], bhi[0], blo[1], bhi[1], blo[2], bhi[2]);
        for (pi, inst) in instances.iter().enumerate() {
            let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
            let o = &lv.maps[inst.map].objects[inst.object];
            let mut n_meet = 0usize;
            let mut near_vert: Vec<String> = Vec::new();
            for (t, m) in c.triangles_with_material() {
                let w: [[f32; 3]; 3] = t.map(|p| nf_formats::placement::transform(&inst.matrix, p));
                let (tlo, thi) = tri_box(&w);
                if !meets(&tlo, &thi, blo, bhi) { continue; }
                n_meet += 1;
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if l < 1e-9 { continue; }
                let n = [n[0] / l, n[1] / l, n[2] / l];
                // near-vertical faces only (|ny| < 0.35) — the walls of interest
                if n[1].abs() < 0.35 {
                    near_vert.push(format!(
                        "    vert tri x {:+.3}..{:+.3} y {:+.2}..{:+.2} z {:+.2}..{:+.2} n ({:+.2},{:+.2},{:+.2}) mat {m:#04x}",
                        tlo[0], thi[0], tlo[1], thi[1], tlo[2], thi[2], n[0], n[1], n[2]));
                }
            }
            if n_meet == 0 { continue; }
            println!("  plc #{pi} obj {}.{} kind {:#04x} glist {:#x} 0x8000 {} name {:?} tris_meet {n_meet}",
                inst.map, inst.object, inst.placement.flags & 0xffff, o.flags,
                inst.placement.flags & 0x8000 != 0, o.name);
            for l in near_vert.iter().take(24) { println!("{l}"); }
            if near_vert.len() > 24 { println!("    … {} more", near_vert.len() - 24); }
        }
    }

    println!("\n=== C. SP#141/#142 + script 06000818 [D] ===");
    let mm = lv.main_map().expect("main map");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if i == 141 || i == 142 {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("SP #{i} pos=({:.2},{:.2},{:.2}) extra={:?}{ps}", p.translation[0], p.translation[1], p.translation[2], p.extra);
        }
    }
    let scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 1b");
    let sim = scene.sim.as_ref().unwrap();
    for p in &sim.drones.scripts.players {
        if p.placement == 141 || p.placement == 142 {
            println!("player #{} scripts [{:08x}, {:08x}] modes [{}, {}] gate {:#x} out {:#x} mask {:#x} stay {}",
                p.placement, p.scripts[0], p.scripts[1], p.modes[0], p.modes[1], p.gate, p.out, p.mask, p.stay);
        }
    }
    if let Some(nis) = sim.drones.scripts.lib.get(&0x0600_0818) {
        println!("script 06000818: length {} camera {} loop_start {}", nis.length, nis.camera, nis.loop_start);
        for (si, st) in nis.streams.iter().enumerate() {
            let cmds = st.decode();
            let list = cmds.iter().map(|c| match c {
                Cmd::End(t) => format!("End({t})"), Cmd::Wait(t) => format!("Wait({t})"),
                Cmd::Marker(t) => format!("Mark({t})"),
                Cmd::Entity { hash, parent, spline, obj_type, params } => format!(
                    "Ent({hash:08x},par{parent:?},spl{spline},t{obj_type},{:?})", params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>()),
                Cmd::EntityEnd => "EntEnd".into(),
                Cmd::Anim { script, skin, start } => format!("Anim({script:08x},{skin:08x},{start})"),
                Cmd::AnimEnd => "AnimEnd".into(),
                Cmd::Event { kind, params } => format!("ev{kind:#04x}[{}]", params.iter().map(|v| format!("{v:#x}")).collect::<Vec<_>>().join(",")),
                Cmd::Sound { id, flat, has_obj, stream } => format!("Snd({id:08x},flat{flat},obj{has_obj},s{stream})"),
                Cmd::SoundEnd => "SndEnd".into(),
                Cmd::Camera { spline, first, last } => format!("Cam(spl{spline},{first}..{last})"),
                Cmd::CameraEnd => "CamEnd".into(),
                Cmd::SubScript { hash, stream, mode } => format!("SUB({hash:08x},s{stream},m{mode})"),
                Cmd::Fade(f) => format!("Fade({f})"),
                Cmd::Sprite { rect, flags, hash } => format!("Sprite({rect:?},f{flags:#x},{hash:08x})"),
                Cmd::Light { parent, kind } => format!("Light({parent:?},{kind})"),
                Cmd::Text { label, time } => format!("Text({label:08x},{time})"),
                Cmd::Skip { op } => format!("Skip({op:#04x})"),
                Cmd::Unread(op) => format!("UNREAD({op:#04x})"),
                Cmd::Other(op) => format!("Other({op:#04x})"),
            }).collect::<Vec<_>>().join(" ");
            println!("  stream {si} keys {}..{} bytes {} : {}", st.first_key, st.last_key, st.commands.len(), if list.is_empty() { "(empty)" } else { &list });
        }
    } else {
        println!("script 06000818 NOT in the loaded lib");
    }

    println!("\n=== D. port-world reads ===");
    println!("--- across the wall x 5.5 -> 3.5 (and back), body / bullet / ray+flags ---");
    for y in [-2.4f32, 0.0, 2.5, 6.0, 9.45, 12.0, 15.0] {
        for z in [-4.0f32, 0.0, 8.82, -8.82] {
            let hit = sim.world.segment([5.5, y, z], [3.5, y, z]);
            let hitb = sim.world.segment_bullet([5.5, y, z], [3.5, y, z]);
            let hitr = sim.world.segment([3.5, y, z], [5.5, y, z]);
            let (tf, n, m, pass) = match sim.world.segment_surface_flags([5.5, y, z], [3.5, y, z], true) {
                Some((t, n, m, p)) => (format!("{t:.3}"), format!("({:+.2},{:+.2},{:+.2})", n[0], n[1], n[2]), format!("{m:#04x}"), p),
                None => ("-".into(), "-".into(), "-".into(), false),
            };
            println!("y {y:+.2} z {z:+.2}: body(e->w) {hit:?} body(w->e) {hitr:?} bullet {hitb:?} | ray(e->w) t {tf} n {n} mat {m} node_pass {pass}");
        }
    }
    println!("--- the capsule push at the float's stop (a=(5.3,0.64,-3.44) b below, r 0.55) ---");
    for x in [5.4f32, 5.3, 5.1, 4.98] {
        let (push, hit) = sim.world.capsule_push([x, 0.64, -3.44], [x, 0.09, -3.44], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        println!("x {x:+.2}: push ({:+.3},{:+.3},{:+.3}) hit {hit}", push[0], push[1], push[2]);
    }
    println!("--- cel graph: find_cel at the shaft's key points ---");
    for (lbl, p) in [
        ("deck", [8.0f32, -1.9, 0.0]), ("slit", [8.6, -1.9, -0.6]),
        ("float stop", [5.0, 0.64, -3.44]), ("shaft S", [5.23, 9.45, -8.82]),
        ("shaft N", [5.23, 9.45, 8.82]), ("shaft mid", [5.23, 9.45, 0.0]),
        ("arena", [-6.0, -1.9, 0.0]), ("above deck", [8.0, 16.0, 0.0]),
    ] {
        println!("  {lbl} {:?}: cel {:?}", p, sim.world.find_cel(p));
    }
    println!("--- bucket owners (cell -> runs meeting the box) ---");
    for (label, blo, bhi) in BOXES {
        println!("  {label}: {:?}", sim.world.debug_bucket_owners(*blo, *bhi));
    }
}
