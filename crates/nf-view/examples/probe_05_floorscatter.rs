//! Scratch (level-05-basin-floor): scatter map of Room_Road_A1's / Room_Road_B's up-facing
//! near-horizontal tris with centre y in 3.5..7.0 — where the valley-floor quads live on disc.
//! Run: NF_ISO=<iso> cargo run -p nf-view --release --example probe_05_floorscatter

use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn scatter(lv: &Level, main: usize, pi: usize, name: &str) {
    let p = &lv.maps[main].placements[pi];
    let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
    let Some((om, oi)) = target else { println!("{name}: unresolved"); return };
    let Some(c) = lv.maps[om].objects[oi].collision.as_ref() else { println!("{name}: no collision"); return };
    println!("== plc {pi} \"{name}\" tris {} — up-facing horizontal tris, centre y 3.5..7.0 ==", c.triangles().len());
    let mut n = 0usize;
    for t in c.triangles() {
        let e1 = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
        let e2 = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
        let nn = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
        let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt();
        if l <= 0.0 || nn[1] / l < 0.85 { continue; }
        let cy = (t[0][1] + t[1][1] + t[2][1]) / 3.0;
        if !(3.5..=7.0).contains(&cy) { continue; }
        let cx = (t[0][0] + t[1][0] + t[2][0]) / 3.0;
        let cz = (t[0][2] + t[1][2] + t[2][2]) / 3.0;
        let ex = ((t[0][0] - t[1][0]).abs() + (t[1][0] - t[2][0]).abs() + (t[0][0] - t[2][0]).abs()).max(
            (t[0][2] - t[1][2]).abs() + (t[1][2] - t[2][2]).abs() + (t[0][2] - t[2][2]).abs());
        n += 1;
        println!("  centre ({cx:.1}, {cy:.3}, {cz:.1}) span {ex:.1}");
    }
    println!("  total {n}");
}

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000005.bin");
    let lv = Level::parse(&data).expect("parse");
    let main = lv.main_map().unwrap();
    scatter(&lv, main, 86, "Room_Road_A1");
    scatter(&lv, main, 87, "Room_Road_B");
}
