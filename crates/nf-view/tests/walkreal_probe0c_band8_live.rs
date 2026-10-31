//! walkreal probe #22 (level-0c-gate-band-crossing): the full placement census over the gate
//! band box — every placement (any map) whose translation lands in x 40..85, z -220..-190, any
//! y: index, class byte, resolved object name + hash, translation, quaternion, params. Plus the
//! sim's grapple points and the class census of the main map's placements. Diagnostic only,
//! --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_band8_census() {
    let isopath = iso();
    {
        let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
        w.wait_enabled(3000, "the intro");
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== sim grapple points ===");
        for (i, g) in sim.drones.grapple_points.iter().enumerate() {
            eprintln!("gp{i}: pos ({:.3},{:.3},{:.3}) radius {:.2}", g.pos[0], g.pos[1], g.pos[2], g.radius);
        }
        eprintln!("=== sim wires ===");
        for (i, wire) in sim.drones.wire_geo.iter().enumerate() {
            eprintln!("wire{i}: {:?}", wire);
        }
    }

    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let _ = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    // Class census over every map's placements.
    eprintln!("=== placement class census per map ===");
    for (mi, m) in level.maps.iter().enumerate() {
        let mut classes: std::collections::BTreeMap<u32, usize> = Default::default();
        for p in m.placements.iter() { *classes.entry(p.flags & 0xff).or_default() += 1; }
        let kinds: Vec<String> = classes.iter().map(|(k, v)| format!("{k:#04x}:{v}")).collect();
        eprintln!("map{mi} kind {} placements {} [{}]", m.kind, m.placements.len(), kinds.join(" "));
    }

    // The band-box census over every map.
    eprintln!("=== placements in the band box x 40..85 z -220..-190 (any y, any map) ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let t = p.translation;
            if !(40.0..=85.0).contains(&t[0]) || !(-220.0..=-190.0).contains(&t[2]) { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string()).unwrap_or("<unresolved>".into());
            let hash = target.map(|(om, oi)| level.maps[om].objects[oi].hashcode).unwrap_or(0);
            let q = p.quaternion();
            eprintln!("map{mi} plc{pi} class {:#04x} \"{}\" hash {hash:08x} t ({:.2},{:.2},{:.2}) q ({:.2},{:.2},{:.2},{:.2}) params {:?}",
                p.flags & 0xff, name, t[0], t[1], t[2], q[0], q[1], q[2], q[3], p.params);
        }
    }
    eprintln!("ok");
}
