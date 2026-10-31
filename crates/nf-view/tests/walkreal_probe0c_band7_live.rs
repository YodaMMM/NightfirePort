//! walkreal probe #13 (level-0c-gate-band-crossing): (1) the definitive band-void floor census —
//! every disc tri (any object, any placement) with a vertex in x 54..68, z −209.5..−206.0,
//! y 10.0..10.7, with its stored normal; (2) the same region's port-world down-ray map at 0.25;
//! (3) the strip gap x 52..61 z −206.5..−204.8 the same two ways; (4) every 0xe2 SS placement in
//! the level with its world glist extent (parked movers slide). Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_band_void_census() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    // 1+4. Every placement/object pair; band-void tris + SS extents.
    eprintln!("=== band-void disc tris (any vertex in x 54..68 z -209.5..-206.0 y 10.0..10.7) ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            if p.flags & 0xff == 0xe2 {
                // SS extent (any map).
                let xs: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[0])).collect();
                let ys: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[1])).collect();
                let zs: Vec<f32> = col.triangles().iter().flat_map(|t| t.iter().map(|&v| plc::transform(&matrix, v)[2])).collect();
                eprintln!("SS map{mi} plc{pi} \"{}\" keys {:?} params {:?} extent x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}",
                    level.maps[om].objects[oi].name.trim(), p.extra.iter().map(|w| w & 0xffff).collect::<Vec<_>>(), p.params,
                    xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max),
                    ys.iter().cloned().fold(f32::MAX, f32::min), ys.iter().cloned().fold(f32::MIN, f32::max),
                    zs.iter().cloned().fold(f32::MAX, f32::min), zs.iter().cloned().fold(f32::MIN, f32::max));
            }
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                if !w.iter().any(|v| v[0] >= 54.0 && v[0] <= 68.0 && v[2] >= -209.5 && v[2] <= -206.0 && v[1] >= 10.0 && v[1] <= 10.7) { continue; }
                e1_check(&w.try_into().unwrap(), mi, pi, om, oi, &level, p);
            }
        }
    }

    // 2+3. The port world (as scene.rs builds it) down-ray maps.
    let instances = level.instances(true).0;
    let world = nf_game::world::CollisionWorld::build(&level, &instances);
    eprintln!("=== port world: strip gap fine map (0.25, x 52..62, z -206.5..-204.8, y 12->8) ===");
    for zi in 0..7 {
        let z = -206.5f32 + 0.25 * zi as f32;
        let mut row = format!("z {z:7.2}: ");
        for i in 0..41 {
            let x = 52.0f32 + 0.25 * i as f32;
            match world.segment_hit([x, 12.0, z], [x, 8.0, z]) {
                Some((tt, _)) => row.push_str(&format!("{:5.2} ", 12.0 - 4.0 * tt)),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== port world: band void map (0.5, x 54..68, z -209.5..-205.5, y 12->8) ===");
    for zi in 0..9 {
        let z = -209.5f32 + 0.5 * zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..29 {
            let x = 54.0f32 + 0.5 * i as f32;
            match world.segment_hit([x, 12.0, z], [x, 8.0, z]) {
                Some((tt, _)) => row.push_str(&format!("{:5.2} ", 12.0 - 4.0 * tt)),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}

fn e1_check(w: &[[f32; 3]; 3], mi: usize, pi: usize, om: usize, oi: usize, level: &Level, p: &nf_formats::placement::Placement) {
    let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
    let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
    let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
    eprintln!("map{mi} plc{pi} class {:#04x} \"{}\" tri ({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2}) n ({:.2},{:.2},{:.2})",
        p.flags & 0xff, level.maps[om].objects[oi].name.trim(),
        w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2], n[0] / l, n[1] / l, n[2] / l);
}
