//! Census (print-only): every level's trigger-volume glist flags, hunting for glists whose flags
//! carry bit 0x40 but are not 0x645 — the game routes every bit-0x40 glist through the raw
//! one-sided refine for the cylinder hit test [M Collide_Intersect 0x1ebaa8:89-96 sets HITDATA+0x51
//! bit 1 from obj+0xf0 & 0x40], while the port's standing-segment raw law keys on the EXACT word
//! (`seg_beam = flags == 0x645`, self-declared [S] scope in c56bbe871). A bit-0x40 non-0x645
//! trigger is a spot where the game's standing segment takes the raw law and the port's does not.
//! Also compares the record-plane side vs the winding side per tri (the use pick reads the record
//! plane [M Intersect_PointGeom 0x1e92d8]; `sphere_meets_tris` reads the winding).
//! Opt-in NF_ISO; prints only (a census, like census_07000015).
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn winding_unit_normal(t: &[[f32; 3]; 3]) -> [f32; 3] {
    let a = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
    let b = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
    let n = [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l <= 0.0 { [0.0, 0.0, 0.0] } else { [n[0] / l, n[1] / l, n[2] / l] }
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn census_trigger_glist_flags_all_levels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, _bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut levels: Vec<String> = entries.iter()
        .filter(|e| e.name.len() == 12 && e.name.starts_with("0700") && e.name[4..].to_ascii_lowercase().ends_with(".bin"))
        .map(|e| e.name.to_ascii_lowercase()).collect();
    levels.sort();
    levels.dedup();
    for level_name in &levels {
        let data = read_archive(&iso, level_name);
        let Ok(lv) = Level::parse(&data) else { println!("== {level_name}: PARSE FAILED"); continue };
        let Some(mm) = lv.main_map() else { println!("== {level_name}: no main map"); continue };
        let map = &lv.maps[mm];
        let mut rows = 0usize;
        for (pi, p) in map.placements.iter().enumerate() {
            if !matches!(p.flags, 0xea | 0xeb | 0xec | 0xed | 0xee | 0xef | 0xe8 | 0xf4 | 0xdc) { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < map.objects.len() { Some((mm, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(c) = lv.maps[om].objects[oi].collision.as_ref() else { continue };
            let flags = lv.maps[om].objects[oi].flags;
            let tris = c.triangles_with_planes();
            if tris.is_empty() { continue; }
            rows += 1;
            let bit40 = flags & 0x40 != 0;
            let exact = flags == 0x645;
            if bit40 && !exact {
                let mut flips = 0usize; let mut same = 0usize;
                for (tri, _m, n_rec, _d) in &tris {
                    let n_win = winding_unit_normal(&tri.map(|v| v));
                    let dot = n_rec[0] * n_win[0] + n_rec[1] * n_win[1] + n_rec[2] * n_win[2];
                    if n_rec.iter().all(|&x| x == 0.0) { continue; }
                    if dot < -0.25 { flips += 1; } else if dot > 0.25 { same += 1; }
                }
                println!("{} placement #{pi} flags {flags:#04x} (bit40, NOT 0x645) tris {} rec-flips {flips} rec-same {same} t {:?} kind {:#x}",
                    level_name, tris.len(), p.translation, p.flags & 0xff);
            }
        }
        println!("== {level_name}: {rows} trigger volumes with collision");
    }
}
