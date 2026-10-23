//! Probe: the pit's hurt volumes and the shuttle-bay lift.
//! Dumps the 07000016 main-map placements of kinds 0xE4 (hurt), 0x3B (blocks), 0xD9 (script
//! players), the pit-region collision bboxes, and — the point — each hurt placement's framelist:
//! the game's level-tag +8 = m_paths[placement+0x44] [M parsemap_block_map_data_dynamic
//! 0x1d1b68: uStack_128 = m_paths[puVar8[0x22]]], key count = placement+0x42−1 [M Hurt_Create
//! 0x17e870:26], rate divisor = placement+0x40 [M 0x17e870:50] — the volume ANIMATES through
//! that key list [M Hurt_Update 0x17e5d0 Spline_Interp into obj+0x90], so the disc keys say
//! whether the deadly poly stays on the pit floor during the ch-0xf3 pulse. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, levelbin::LevelBin, level::Level};

fn rd_u16(w: u32, hi: bool) -> u16 { if hi { (w >> 16) as u16 } else { w as u16 } }

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lb = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let m = level.main_map().expect("main map");
    let map = &level.maps[m];
    println!("main map {} placements {} objects {} path_data blocks {}", m, map.placements.len(), map.objects.len(), map.path_data.len());
    for (i, p) in map.placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        if !matches!(kind, 0xe4 | 0x3b | 0xd9) { continue; }
        let pos = p.translation;
        println!("place[{i}] kind {kind:#04x} pos ({:.2},{:.2},{:.2}) hash {:08x} index {} params {:?} rot {:?}",
            pos[0], pos[1], pos[2], p.hashcode, p.index, p.params, p.rotation);
        // +0x40 = extra[7] low u16 (rate divisor), +0x42 = extra[7] high u16 (key count+1),
        // +0x44 = extra[8] (the m_paths index).
        println!("         +0x40 divisor {} | +0x42 count {} | +0x44 path {}",
            rd_u16(p.extra[7], false), rd_u16(p.extra[7], true), p.extra[8]);
        let pi = p.extra[8] as usize;
        if pi < map.path_data.len() {
            let pd = &map.path_data[pi];
            println!("         path[{pi}] {} bytes: {:02x?}", pd.len(), &pd[..pd.len().min(96)]);
            // The framelist_tag: try the NIS key layout (0x30-byte keys after a u32 count).
            if pd.len() >= 4 {
                let n = u32::from_le_bytes([pd[0], pd[1], pd[2], pd[3]]) as usize;
                println!("         path[{pi}] first u32 (count?) {n} (0x{:08x})", n);
                if n > 0 && n < 4096 && pd.len() >= 4 + n * 0x30 {
                    for k in 0..n {
                        let o = 4 + k * 0x30;
                        let f = |q: usize| f32::from_le_bytes(pd[o + q..o + q + 4].try_into().unwrap());
                        println!("           key{k} pos({:.2},{:.2},{:.2}) time {:.2}", f(0), f(4), f(8), f(0x2c));
                    }
                }
            }
        }
    }
    // The hurt objects themselves: name, bbox, and the tri y/xz layout (is the slab solid or sparse?).
    for idx in [182usize, 187] {
        let o = &map.objects[idx];
        println!("\n== hurt object[{idx}] hash {:08x} name {:?}", o.hashcode, o.name);
        if let Some(c) = o.collision.as_ref() {
            let tris = c.triangles();
            let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3];
            for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            println!("   tris {} lo({:.2},{:.2},{:.2}) hi({:.2},{:.2},{:.2})", tris.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            // Bucket the tris by y-band and print per-band xz spans.
            let mut bands: Vec<(f32, f32, usize, [f32; 2], [f32; 2])> = Vec::new();
            for t in &tris {
                let ylo = t.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
                let yhi = t.iter().map(|v| v[1]).fold(-f32::MAX, f32::max);
                let xlo = t.iter().map(|v| v[0]).fold(f32::MAX, f32::min);
                let xhi = t.iter().map(|v| v[0]).fold(-f32::MAX, f32::max);
                let zlo = t.iter().map(|v| v[2]).fold(f32::MAX, f32::min);
                let zhi = t.iter().map(|v| v[2]).fold(-f32::MAX, f32::max);
                match bands.iter_mut().find(|(a, b, _, _, _)| (ylo >= *a - 0.01 && ylo <= *b + 0.01)) {
                    Some((_, _, n, xl, zl)) => { *n += 1; xl[0] = xl[0].min(xlo); xl[1] = xl[1].max(xhi); zl[0] = zl[0].min(zlo); zl[1] = zl[1].max(zhi); }
                    None => bands.push((ylo, yhi, 1, [xlo, xhi], [zlo, zhi])),
                }
            }
            bands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (ylo, yhi, n, x, z) in &bands {
                println!("   band y {ylo:.2}..{yhi:.2}: {n} tris x {:.2?} z {:.2?}", x, z);
            }
        } else { println!("   no collision"); }
    }

    println!("\n-- pit-region collision objects (bbox lo.y < -14) --");
    for (oi, o) in map.objects.iter().enumerate() {
        let Some(c) = o.collision.as_ref() else { continue };
        let tris = c.triangles();
        let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3]; let mut n = 0usize;
        for t in &tris { for v in t { n += 1;
            for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); }
        } }
        if n > 0 && lo[1] < -14.0 && lo[2] > -205.0 && hi[2] < -125.0 {
            println!("  obj[{oi}] hash {:08x} name {:?} tris {} lo({:.2},{:.2},{:.2}) hi({:.2},{:.2},{:.2})",
                o.hashcode, o.name, tris.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        }
    }

// The lift platform map object 02000a79: authored transform + collision.
for (oi, o) in map.objects.iter().enumerate() {
    if o.hashcode == 0x0200_0a79 {
        println!("== lift platform obj[{oi}] hash {:08x} name {:?}", o.hashcode, o.name);
        if let Some(c) = o.collision.as_ref() {
            let tris = c.triangles();
            let mut lo = [f32::MAX; 3]; let mut hi = [-f32::MAX; 3];
            for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            println!("   collision tris {} lo({:.2},{:.2},{:.2}) hi({:.2},{:.2},{:.2})", tris.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
        } else { println!("   no collision"); }
    }
}
}
