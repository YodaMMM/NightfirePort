//! trigger-hittest-segment scratch: dump the glist flags (celglist+0x44, MapObject.flags) of the
//! trigger cels whose contact law is in question: 0700000a beam #621, 07000011 plate #235, the
//! 0700000c ch-0x27 bay volume, and every mask-8 touch trigger's flags census. Prints only.
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

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    for (file, picks, ch_filter) in [
        ("0700000a.bin", vec![607usize, 614, 621], None),
        ("07000011.bin", vec![235usize], None),
        ("0700000c.bin", vec![], Some(0x27u32)),
    ] {
        let lv = load(&img, file);
        let mm = lv.main_map().expect("main map");
        let map = &lv.maps[mm];
        println!("== {file}: placements {}", map.placements.len());
        for (i, p) in map.placements.iter().enumerate() {
            let k = p.flags & 0xff;
            if !matches!(k, 0xea | 0xeb | 0xdc | 0xec | 0xed | 0xee | 0xef) { continue; }
            let out = p.params.iter().find(|(j, _)| *j == 0).map(|&(_, v)| v).unwrap_or(0);
            let mask = p.params.iter().find(|(j, _)| *j == 9).map(|&(_, v)| v).unwrap_or(0);
            if let Some(cf) = ch_filter {
                if out != cf { continue; }
            }
            if !picks.is_empty() && !picks.contains(&i) { continue; }
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < map.objects.len() { Some((mm, p.index as usize)) } else { None };
            let (fl, name, ntris, bounds) = target
                .and_then(|(om, oi)| lv.maps.get(om).and_then(|mp| mp.objects.get(oi)))
                .map(|ob| (ob.flags, ob.name.trim().to_string(),
                    ob.collision.as_ref().map(|c| c.triangles().len()).unwrap_or(0),
                    ob.collision.as_ref().map(|c| {
                        let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
                        for t in c.triangles() { for v in t { for d in 0..3 { lo[d] = lo[d].min(v[d]); hi[d] = hi[d].max(v[d]); } } }
                        format!("lo {:.2} {:.2} {:.2} hi {:.2} {:.2} {:.2}", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2])
                    }).unwrap_or("no collision".into())))
                .unwrap_or((0, "<no target>".into(), 0, "-".into()));
            let t = p.translation;
            println!(
                "  #{i} kind {k:#04x} out {out:#04x} mask {mask:#x} @({:.2},{:.2},{:.2}) -> \"{name}\" flags {fl:#010x} (bits:{}) tris {ntris}\n    local bounds: {bounds}",
                t[0], t[1], t[2],
                [("0x1", 0x01u32), ("0x2", 0x02), ("0x4", 0x04), ("0x8", 0x08), ("0x10", 0x10), ("0x20", 0x20), ("0x40", 0x40), ("0x80", 0x80)]
                    .iter().filter(|(_, b)| fl & b != 0).map(|(n, _)| *n).collect::<Vec<_>>().join(","),
            );
        }
    }
}
