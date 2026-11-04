//! trigger-plane-records probe: read the trigger glists' OWN plane records (the tri's 4th u16 →
//! the leaf s16 area at base + plane·2: 3×i16 normal, then the i32 at byte +6 [M asm 0x10a1d8-0x10a2f0,
//! the decomp fetch]) and pin, on real disc data, (1) the (n, d) scale pair that makes each record
//! consistent with its own tri's vertices, (2) how often the record normal's sign opposes the RAW
//! tri winding's cross (the one-sided entry's authoritative side), per cel. Prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn load(img: &IsoImage, name: &str) -> Level {
    let (entries, bin) = nf_formats::disc::files_bin_entries(img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
        .unwrap_or_else(|| panic!("no {name}"));
    let data = img
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    Level::parse(&data).expect("parse")
}

fn f32bits(x: f32) -> u32 { f32::to_bits(x) }

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let targets: &[(&str, &[usize])] = &[
        ("0700000a.bin", &[607usize, 614, 621]),
        ("07000011.bin", &[235]),
        ("07000005.bin", &[]), // the spawn-plate cel: census every touch trigger's target
    ];
    for (file, picks) in targets {
        let lv = load(&img, file);
        let mm = lv.main_map().expect("main map");
        println!("== {file}");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            if !matches!(k, 0xea | 0xeb | 0xdc | 0xec | 0xed | 0xee | 0xef) { continue; }
            let out = p.params.iter().find(|(j, _)| *j == 0).map(|&(_, v)| v).unwrap_or(0);
            if file.contains("05") && out != 0x45 { continue; }
            if !picks.is_empty() && !picks.contains(&i) { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { println!("  #{i}: no target"); continue };
            let Some(col) = lv.maps[om].objects[oi].collision.as_ref() else { println!("  #{i}: target {} {} \"{}\" no collision", om, oi, lv.maps[om].objects[oi].name.trim()); continue };
            println!("  #{i} -> obj {oi} \"{}\" tris {}", lv.maps[om].objects[oi].name.trim(), col.tris.len());
            // Per-leaf walk: verts (s16·scale+origin) and the plane records the CylGeom reader
            // fetches (leaf byte base = vblock·0x40; record at base + plane·2: n = 3×i16, d = i32 @+6).
            let (mut h1, mut h2, mut h3, mut bad_idx) = (0usize, 0usize, 0usize, 0usize);
            let (mut oppose, mut agree, mut degen) = (0usize, 0usize, 0usize);
            let mut worst1 = 0.0f32; let mut worst2 = 0.0f32; let mut worst3 = 0.0f32;
            for bx in col.boxes.iter().filter(|b| b.children.is_none()) {
                let base = bx.vblock as usize * 0x20; // halfwords
                for t in &col.tris[bx.tris.start as usize..bx.tris.end as usize] {
                    let w = |i: usize| col.words.get(base + i).copied().unwrap_or(0) as f32;
                    let v: Vec<[f32; 3]> = t.v.map(|vi| {
                        [w(vi as usize) * bx.scale + bx.origin[0],
                         w(vi as usize + 1) * bx.scale + bx.origin[1],
                         w(vi as usize + 2) * bx.scale + bx.origin[2]]
                    }).to_vec();
                    let pb = base + t.plane as usize; // halfwords
                    let g = |i: usize| col.words.get(pb + i).copied().unwrap_or(0) as f32;
                    let n16 = [g(0), g(1), g(2)];
                    let d16 = g(3); // the low halfword of the i32 @byte pb*2+6
                    let d32 = i32::from_le_bytes([
                        col.words.get(pb + 3).copied().unwrap_or(0) as u8 as i8 as u8,
                        (col.words.get(pb + 3).copied().unwrap_or(0) as u32 >> 8) as u8,
                        col.words.get(pb + 4).copied().unwrap_or(0) as u8,
                        (col.words.get(pb + 4).copied().unwrap_or(0) as u32 >> 8) as u8,
                    ]) as f32;
                    let nn = (n16[0] * n16[0] + n16[1] * n16[1] + n16[2] * n16[2]).sqrt();
                    if nn < 1.0 { degen += 1; continue; }
                    let nh = [n16[0] / nn, n16[1] / nn, n16[2] / nn];
                    // winding cross (RAW order, no alignment swap)
                    let e1 = [v[1][0] - v[0][0], v[1][1] - v[0][1], v[1][2] - v[0][2]];
                    let e2 = [v[2][0] - v[0][0], v[2][1] - v[0][1], v[2][2] - v[0][2]];
                    let cr = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                    let dot = nh[0] * cr[0] + nh[1] * cr[1] + nh[2] * cr[2];
                    if dot < 0.0 { oppose += 1 } else { agree += 1 }
                    // consistency: n̂·v_k + d ≈ 0 for k = 0..3 under three d-scale hypotheses
                    let r1 = (0..3).map(|kk| (nh[0] * v[kk][0] + nh[1] * v[kk][1] + nh[2] * v[kk][2] + d16).abs())
                        .fold(0.0f32, f32::max); // d = the i16 @+6 (4-i16 quad)
                    let r2 = (0..3).map(|kk| (nh[0] * v[kk][0] + nh[1] * v[kk][1] + nh[2] * v[kk][2] + d32 * (1.0 / 16384.0)).abs())
                        .fold(0.0f32, f32::max); // d = the i32 @+6 · 2⁻¹⁴ (same fixed-point scale as n)
                    let r3 = (0..3).map(|kk| (nh[0] * v[kk][0] + nh[1] * v[kk][1] + nh[2] * v[kk][2] + d32).abs())
                        .fold(0.0f32, f32::max); // d = the i32 @+6 raw (world units)
                    if r1 < 0.01 { h1 += 1 } else { worst1 = worst1.max(r1) }
                    if r2 < 0.01 { h2 += 1 } else { worst2 = worst2.max(r2) }
                    if r3 < 0.01 { h3 += 1 } else { worst3 = worst3.max(r3) }
                    if bad_idx < 3 {
                        println!("    tri {}: n16 ({:.0},{:.0},{:.0}) |n| {:.1} d16 {} d32 {} (bits {:08x}) v0 ({:.3},{:.3},{:.3})",
                            bad_idx, n16[0], n16[1], n16[2], nn, d16, f32bits(d32), f32bits(d32), v[0][0], v[0][1], v[0][2]);
                        println!("      n̂·cross {:.3}  |n̂·v0+d16| {:.4}  |n̂·v0+d32·2⁻¹⁴| {:.4}  |n̂·v0+d32| {:.4}", dot, r1, r2, r3);
                        bad_idx += 1;
                    }
                }
            }
            println!("    records: winding-opposes {} / agrees {} / degenerate {}", oppose, agree, degen);
            println!("    d-scale consistency: i16@+6 {} · i32@+6·2⁻¹⁴ {} · i32@+6 raw {}   (worst residual {:.4} / {:.4} / {:.4})",
                h1, h2, h3, worst1, worst2, worst3);
        }
    }
    probe2();
}

// Second probe: the 0a beams' records were inconsistent under the leaf-relative read — dump the
// beam glist's leaf layout (vblock, tris range) and try the GLOBAL words base for the plane index.
fn probe2() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let lv = load(&img, "0700000a.bin");
    let mm = lv.main_map().expect("main map");
    let p = &lv.maps[mm].placements[621];
    let (om, oi) = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied().expect("hash") } else { (mm, p.index as usize) };
    let col = lv.maps[om].objects[oi].collision.as_ref().expect("collision");
    println!("== beam #621 target obj {oi}: {} boxes, {} leaves", col.boxes.len(), col.boxes.iter().filter(|b| b.children.is_none()).count());
    for (bi, bx) in col.boxes.iter().filter(|b| b.children.is_none()).enumerate() {
        println!("  leaf {bi}: vblock {} vblocks {} tris {}..{} origin ({:.3},{:.3},{:.3}) scale {}",
            bx.vblock, bx.vblocks, bx.tris.start, bx.tris.end, bx.origin[0], bx.origin[1], bx.origin[2], f32bits(bx.scale));
        let base = bx.vblock as usize * 0x20;
        for t in col.tris[bx.tris.start as usize..bx.tris.end as usize].iter().take(3) {
            let w = |i: usize| col.words.get(base + i).copied().unwrap_or(0);
            let v: Vec<[f32; 3]> = t.v.map(|vi| {
                [w(vi as usize) as f32 * bx.scale + bx.origin[0],
                 w(vi as usize + 1) as f32 * bx.scale + bx.origin[1],
                 w(vi as usize + 2) as f32 * bx.scale + bx.origin[2]]
            }).to_vec();
            // leaf-relative record
            let g = |i: usize| col.words.get(base + t.plane as usize + i).copied().unwrap_or(0);
            // global record
            let q = |i: usize| col.words.get(t.plane as usize + i).copied().unwrap_or(0);
            println!("    tri {}: verts idx {:?} v0 ({:.3},{:.3},{:.3}) v1 ({:.3},{:.3},{:.3}) v2 ({:.3},{:.3},{:.3})",
                bx.tris.start, t.v, v[0][0], v[0][1], v[0][2], v[1][0], v[1][1], v[1][2], v[2][0], v[2][1], v[2][2]);
            println!("      leaf-rel @hw {} : n ({},{},{}) d {}", t.plane, g(0), g(1), g(2), g(3));
            println!("      global   @hw {} : n ({},{},{}) d {}", t.plane, q(0), q(1), q(2), q(3));
            // consistency under global
            let nn = ((q(0) as f32) * (q(0) as f32) + (q(1) as f32) * (q(1) as f32) + (q(2) as f32) * (q(2) as f32)).sqrt();
            if nn > 1.0 {
                let nh = [q(0) as f32 / nn, q(1) as f32 / nn, q(2) as f32 / nn];
                let r = (0..3).map(|kk| (nh[0] * v[kk][0] + nh[1] * v[kk][1] + nh[2] * v[kk][2] + q(3) as f32).abs()).fold(0.0f32, f32::max);
                let r2 = (0..3).map(|kk| (nh[0] * v[kk][0] + nh[1] * v[kk][1] + nh[2] * v[kk][2] + q(3) as f32 * (1.0 / 16384.0)).abs()).fold(0.0f32, f32::max);
                println!("      global consistency: |n̂·v+d| {:.4}  |n̂·v+d·2⁻¹⁴| {:.4}", r, r2);
            }
        }
    }
}
