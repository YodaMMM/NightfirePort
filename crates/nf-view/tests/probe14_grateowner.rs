//! level-14 probe: who owns the grate? Every level instance whose transformed collision
//! intersects the grate box (x 40.0..41.7, y 2.3..3.9, z 58.6..59.1) — the placement, object
//! name, hash and its collision tri count. Mirrors CollisionWorld::build_scoped's loop. Prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_grate_owner() {
    let img = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();

    // Rebuild the instance list the way the port does (main map + pushed maps): use the port's
    // own instance builder via nf_view if exported; else walk main-map placements directly.
    let mm = level.main_map().unwrap_or(level.maps.len() - 1);
    for (pi, p) in level.maps[mm].placements.iter().enumerate() {
        let (m, o) = if p.by_hash() {
            match level.by_hash.get(&p.hashcode) { Some(x) => *x, None => continue }
        } else {
            if p.index as usize >= level.maps[mm].objects.len() { continue }
            (mm, p.index as usize)
        };
        let Some(c) = &level.maps[m].objects[o].collision else { continue };
        // Transform the collision's AABB by the placement translation (rotations are axis-aligned
        // in this level's vents; approximate with translation + bounds in object space).
        let t = p.translation;
        let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
        for tri in c.triangles() {
            for v in tri {
                let w = [v[0] + t[0], v[1] + t[1], v[2] + t[2]];
                for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
            }
        }
        let hit = lo[0] <= 41.7 && hi[0] >= 40.0 && lo[1] <= 3.9 && hi[1] >= 2.3 && lo[2] <= 59.1 && hi[2] >= 58.6;
        if hit || (lo[2] <= 63.8 && hi[2] >= 63.5 && lo[1] <= 3.9 && hi[1] >= 2.3 && lo[0] <= 41.7 && hi[0] >= 40.0) {
            eprintln!("plc {pi} \"{}\" kind {:#04x} hash {:08x} @({:.2},{:.2},{:.2}) coll_tris {} aabb x[{:.2}..{:.2}] y[{:.2}..{:.2}] z[{:.2}..{:.2}]{}",
                level.maps[m].objects[o].name.trim(), p.flags & 0xff, p.hashcode, t[0], t[1], t[2], c.tris.len(),
                lo[0], hi[0], lo[1], hi[1], lo[2], hi[2],
                if hit { " <== GRATE BOX" } else { " (mouth z=63.66)" });
        }
    }
}

#[test]
fn probe_grate_flags() {
    let img = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mm = level.main_map().unwrap_or(level.maps.len() - 1);
    for pi in [16usize, 100, 258, 261] {
        let p = &level.maps[mm].placements[pi];
        let (m, o) = if p.by_hash() { level.by_hash[&p.hashcode] } else { (mm, p.index as usize) };
        let obj = &level.maps[m].objects[o];
        eprintln!("plc {pi} \"{}\" obj_flags {:#010x} bits0x10={} bit0x40={} bit0x1000={} bit0x20={} bit0x2000={} bit0x800={} coll_tris {}",
            obj.name.trim(), obj.flags,
            obj.flags & 0x10 != 0, obj.flags & 0x40 != 0, obj.flags & 0x1000 != 0,
            obj.flags & 0x20 != 0, obj.flags & 0x2000 != 0, obj.flags & 0x800 != 0,
            obj.collision.as_ref().map_or(0, |c| c.tris.len()));
    }
}

#[test]
fn probe_bit1_census() {
    let img = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let mut n = 0; let mut with_coll = 0;
    for (mi, map) in level.maps.iter().enumerate() {
        for (oi, o) in map.objects.iter().enumerate() {
            if o.flags & 1 != 0 && o.flags & 0x3fe == 0 {
                n += 1;
                let c = o.collision.as_ref().map_or(0, |c| c.tris.len());
                if c > 0 { with_coll += 1; }
                if n <= 40 { eprintln!("bit1 m{mi} obj{oi} \"{}\" flags {:#x} tris {}", o.name.trim(), o.flags, c); }
            }
        }
    }
    eprintln!("bit-0x1 objects: {n}, with collision: {with_coll}");
}
