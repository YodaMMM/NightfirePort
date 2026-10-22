//! level-14-vent-grate probe: the REAL [D] data behind the bit-0x1 skip question.
//! For 07000014's main map: (1) every placement whose resolved object name matches the vent
//! furniture — full placement flags word (does it carry 0x8000, the parsemap create-cel gate
//! [M 0x1d1998:92]) + the object's glist word; (2) the 2x2 census creates_cel × glist-bit-0x1
//! over placements with collision tris. Prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Level {
    let img = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

#[test]
fn probe_placement_flags_vent_furniture() {
    let level = load();
    let mm = level.main_map().unwrap_or(level.maps.len() - 1);
    for (pi, p) in level.maps[mm].placements.iter().enumerate() {
        let (m, o): (usize, usize) = if p.by_hash() {
            match level.by_hash.get(&p.hashcode) { Some(x) => *x, None => continue }
        } else {
            if p.index as usize >= level.maps[mm].objects.len() { continue }
            (mm, p.index as usize)
        };
        let obj = &level.maps[m].objects[o];
        let name = obj.name.trim();
        let interesting = ["AirCon", "Grill", "Glass", "Window", "Door_Slide", "Door_Win",
            "Vent_", "Hatch", "Slime", "Cargo"]
            .iter().any(|k| name.contains(k));
        if !interesting { continue; }
        let tris = obj.collision.as_ref().map_or(0, |c| c.tris.len());
        eprintln!(
            "plc {pi} \"{}\" plc_flags {:#010x} (0x8000={} kind {:#04x}) obj_glist {:#010x} (b1={} b20={} b40={} b1000={}) tris {} @({:.2},{:.2},{:.2})",
            name, p.flags, p.creates_cel(), p.flags & 0xff, obj.flags,
            obj.flags & 1 != 0, obj.flags & 0x20 != 0, obj.flags & 0x40 != 0, obj.flags & 0x1000 != 0,
            tris, p.translation[0], p.translation[1], p.translation[2]);
    }
}

#[test]
fn probe_census_createscel_x_bit1() {
    let level = load();
    let mm = level.main_map().unwrap_or(level.maps.len() - 1);
    // 2x2: creates_cel × glist-bit-0x1, over placements whose resolved object has collision tris.
    let mut cells: [[(usize, Vec<String>); 2]; 2] = std::array::from_fn(|_|
        std::array::from_fn(|_| (0usize, Vec::new())));
    for (pi, p) in level.maps[mm].placements.iter().enumerate() {
        let (m, o): (usize, usize) = if p.by_hash() {
            match level.by_hash.get(&p.hashcode) { Some(x) => *x, None => continue }
        } else {
            if p.index as usize >= level.maps[mm].objects.len() { continue }
            (mm, p.index as usize)
        };
        let obj = &level.maps[m].objects[o];
        if obj.collision.as_ref().map_or(0, |c| c.tris.len()) == 0 { continue; }
        let c = p.creates_cel() as usize;
        let b1 = (obj.flags & 1 != 0) as usize;
        let e = &mut cells[c][b1];
        e.0 += 1;
        if e.1.len() < 12 { e.1.push(format!("plc{pi} \"{}\" glist {:#x}", obj.name.trim(), obj.flags)); }
    }
    for (c, row) in cells.iter().enumerate() {
        for (b, cell) in row.iter().enumerate() {
            let (n, ex) = (&cell.0, &cell.1);
            eprintln!("creates_cel={} glist_bit1={} : {} placements", c, b, n);
            for s in ex { eprintln!("    {}", s); }
        }
    }
    // And: every glist-bit-0x1 object whose placement DOES create a cel (these are what the
    // bit-0x1 skip removes from a world the game WOULD give collision).
    eprintln!("---");
}

#[test]
fn probe_bit1_objects_with_cel_placements() {
    let level = load();
    let mm = level.main_map().unwrap_or(level.maps.len() - 1);
    let mut n = 0;
    for (pi, p) in level.maps[mm].placements.iter().enumerate() {
        if !p.creates_cel() { continue; }
        let (m, o): (usize, usize) = if p.by_hash() {
            match level.by_hash.get(&p.hashcode) { Some(x) => *x, None => continue }
        } else {
            if p.index as usize >= level.maps[mm].objects.len() { continue }
            (mm, p.index as usize)
        };
        let obj = &level.maps[m].objects[o];
        let tris = obj.collision.as_ref().map_or(0, |c| c.tris.len());
        if obj.flags & 1 != 0 && tris > 0 {
            n += 1;
            if n <= 50 {
                eprintln!("cel-plc {pi} \"{}\" glist {:#010x} tris {} @({:.2},{:.2},{:.2})",
                    obj.name.trim(), obj.flags, tris, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
    }
    eprintln!("glist-bit-0x1 WITH cel + collision: {n}");
}
