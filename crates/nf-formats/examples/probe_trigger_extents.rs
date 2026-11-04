//! trigger-glist-extents probe (prints only):
//! 1. The disc side: for the divergent trigger cels — 0700000a #607/#614/#621, 07000011 #235,
//!    0700000c's out-0x1b (ch 27) Touch plates — dump the placement record, the target object's
//!    render bounds, the collision block's OWN root box (min/max, origin/scale of every leaf) and
//!    the port's parsed WORLD band (placement matrix × local verts), so the parsed-vs-real
//!    divergence has numbers on the page.
//! 2. The game side: walk work/ee_start_0700000a.bin (file offset = EE address) from the object
//!    list head 0x2705a0 (control_add_object_to_list 0x134698: head+0x14 = first), print every
//!    class-0x1d trigger object's obj+0x30 pos, obj+0x90 matrix translation row, obj+0xD8 glist
//!    pointer and ext+0x20 (the out channel at ext obj+0xE0 → +0x20 [M Trigger_Create 0x1b1720:68]),
//!    so the game's own placement can be matched against the placement records.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::{transform, view_rot_trans_matrix};

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

fn aabb(tris: &[[[f32; 3]; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for t in tris {
        for v in t {
            for k in 0..3 {
                lo[k] = lo[k].min(v[k]);
                hi[k] = hi[k].max(v[k]);
            }
        }
    }
    (lo, hi)
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");

    // ---- 1. The disc side ----
    let targets: &[(&str, &[usize])] = &[
        ("0700000a.bin", &[607usize, 614, 621]),
        ("07000011.bin", &[235]),
        ("0700000c.bin", &[]), // census: every trigger placement with out 0x1b and a volume
    ];
    for (file, picks) in targets {
        let lv = load(&img, file);
        let mm = lv.main_map().expect("main map");
        println!("== {file}");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            if !matches!(k, 0xea | 0xeb | 0xdc | 0xec | 0xed | 0xee | 0xef) { continue; }
            let out = p.params.iter().find(|(j, _)| *j == 0).map(|&(_, v)| v).unwrap_or(0);
            let want = if file.contains("0c") { out == 0x1b } else { !picks.is_empty() && picks.contains(&i) };
            if !want { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            println!("  #{i} flags 0x{:x} out 0x{out:x} index {} hash 0x{:x} by_hash {}",
                p.flags, p.index, p.hashcode, p.by_hash());
            println!("     translation {:?} rotation {:?}", p.translation, p.rotation);
            println!("     params {:?}", p.params);
            let Some((om, oi)) = target else { println!("     no target"); continue };
            let o = &lv.maps[om].objects[oi];
            println!("     -> obj {oi} \"{}\" flags 0x{:x}", o.name.trim(), o.flags);
            println!("        render centre {:?} r {} min {:?} max {:?}", o.centre, o.radius, o.min, o.max);
            let Some(col) = o.collision.as_ref() else { println!("        no collision block"); continue };
            let root = &col.boxes[0];
            println!("        coll root min {:?} max {:?}", root.min, root.max);
            let leaves: Vec<_> = col.boxes.iter().filter(|b| b.children.is_none()).collect();
            println!("        {} leaves, {} tris", leaves.len(), col.tris.len());
            for (li, b) in leaves.iter().enumerate().take(6) {
                println!("          leaf {li}: min {:?} max {:?} origin {:?} scale {} tris {}",
                    b.min, b.max, b.origin, b.scale, b.tris.end - b.tris.start);
            }
            let mx = view_rot_trans_matrix(p.rotation, p.translation);
            let tris: Vec<[[f32; 3]; 3]> = col.triangles().into_iter().map(|t| t.map(|v| transform(&mx, v))).collect();
            let (lo, hi) = aabb(&tris);
            println!("        PARSED world band lo {:?} hi {:?}", lo, hi);
        }
    }

    // ---- 2. The game side: the 0700000a boot snapshot ----
    let snap = std::fs::read("/home/z/NightfirePort/work/ee_start_0700000a.bin").expect("snapshot");
    let u32at = |a: usize| u32::from_le_bytes(snap[a..a + 4].try_into().unwrap());
    let f32at = |a: usize| f32::from_le_bytes(snap[a..a + 4].try_into().unwrap());
    println!("== ee_start_0700000a.bin object walk (head 0x2705a0)");
    let mut p = u32at(0x2705a0 + 0x14) as usize; // head+0x14 = first object [M 0x134698]
    let mut n = 0;
    while p != 0 && (p & 3) == 0 && n < 4000 {
        n += 1;
        let class = snap.get(p + 0xff).copied().unwrap_or(0);
        if class == 0x1d {
            let pos = [f32at(p + 0x30), f32at(p + 0x34), f32at(p + 0x38)];
            // obj+0x90 matrix: View_RotTransMatrix writes rows; translation in the 4th column of
            // the 3 rows (m[0][3], m[1][3], m[2][3]) or the 4th row — print both readings.
            let m = [f32at(p + 0x90), f32at(p + 0x94), f32at(p + 0x98), f32at(p + 0x9c),
                     f32at(p + 0xa0), f32at(p + 0xa4), f32at(p + 0xa8), f32at(p + 0xac),
                     f32at(p + 0xb0), f32at(p + 0xb4), f32at(p + 0xb8), f32at(p + 0xbc),
                     f32at(p + 0xc0), f32at(p + 0xc4), f32at(p + 0xc8), f32at(p + 0xcc)];
            let glist = u32at(p + 0xd8);
            let ext = u32at(p + 0xe0);
            let out = if ext != 0 && (ext as usize + 0x24) <= snap.len() { u32at(ext as usize + 0x20) } else { 0 };
            let kind = if ext != 0 && (ext as usize + 0x40) <= snap.len() { u32at(ext as usize + 0x38) & 0xffff } else { 0 };
            println!("  obj 0x{p:08x} class 0x1d out 0x{:02x} kind {} glist 0x{glist:08x} ext 0x{ext:08x}", out & 0xff, kind);
            println!("     obj+0x30 pos {:?}", pos);
            println!("     obj+0x90 rows: {:?} / {:?} / {:?} / {:?}", &m[0..4], &m[4..8], &m[8..12], &m[12..16]);
        }
        p = u32at(p + 0x14) as usize;
    }
    println!("  walked {n} objects");
}
