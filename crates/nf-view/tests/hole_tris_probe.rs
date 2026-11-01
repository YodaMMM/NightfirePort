//! level-03-fall-through-floor — the hole's tris: raw winding cross vs stored plane normal.
//!
//! The flat feet-ray map found a through-world void at x 8.0..8.9, z −19.5..−16.0 (west of the
//! courtyard). The game's Intersect_RayGeom culls on the STORED PLANE normal
//! [M 0x10a8c8: `dot(plane_normal, ray_dir) >= 0` → skip]; the port's queries cull on the winding
//! cross (world.rs:1555) — and docs/formats/collision.md counts 146 winding-opposed + 477
//! disagreeing tris on disc. A floor tri whose winding opposes its plane is invisible to the
//! port's ray but visible to the game's. This probe resolves the leaf tris RAW (no swap) for the
//! hole box and prints both normals per tri.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn tri_box(w: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let mut lo = w[0]; let mut hi = w[0];
    for p in w.iter().skip(1) { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
    (lo, hi)
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn dump_the_hole_tris_both_normals() {
    let iso = iso();
    let data = read_archive(&iso, "07000003.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    let (blo, bhi) = ([7.6f32, 3.0, -20.2], [9.4f32, 8.0, -15.4]);
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        for bx in c.boxes.iter().filter(|b| b.children.is_none()) {
            let base = bx.vblock as usize * 0x20;
            for (ti, t) in c.tris[bx.tris.start as usize..bx.tris.end as usize].iter().enumerate() {
                let m = c.materials.get(bx.tris.start as usize + ti).copied().unwrap_or(0);
                let v = t.v.map(|i| {
                    let w = |k: usize| c.words.get(base + i as usize + k).copied().unwrap_or(0) as f32;
                    [w(0) * bx.scale + bx.origin[0], w(1) * bx.scale + bx.origin[1], w(2) * bx.scale + bx.origin[2]]
                });
                let w: [[f32; 3]; 3] = v.map(|p| nf_formats::placement::transform(&inst.matrix, p));
                let (tlo, thi) = tri_box(&w);
                if tlo[0] > bhi[0] || thi[0] < blo[0] || tlo[1] > bhi[1] || thi[1] < blo[1]
                    || tlo[2] > bhi[2] || thi[2] < blo[2] { continue; }
                // The stored plane normal (leaf words at plane·2: 3 s16 · 2⁻¹⁴).
                let p = t.plane as usize;
                let pw = |k: usize| c.words.get(base + p + k).copied().unwrap_or(0) as f32 * (1.0 / 16384.0);
                let pn = [pw(0), pw(1), pw(2)];
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let cr = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
                if l < 1e-9 { continue; }
                let wn = [cr[0] / l, cr[1] / l, cr[2] / l];
                let agree = pn[0] * wn[0] + pn[1] * wn[1] + pn[2] * wn[2] >= 0.0;
                println!("i{pi} t{ti} x {:+.2}..{:+.2} y {:+.2}..{:+.2} z {:+.2}..{:+.2} winding ({:+.2},{:+.2},{:+.2}) plane ({:+.2},{:+.2},{:+.2}) {} mat {m:#04x}",
                    tlo[0], thi[0], tlo[1], thi[1], tlo[2], thi[2], wn[0], wn[1], wn[2], pn[0], pn[1], pn[2],
                    if agree { "AGREE" } else { "OPPOSED" });
            }
        }
    }
}
