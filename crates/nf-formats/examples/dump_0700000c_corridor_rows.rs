//! level-0c-secret-door-reach-3 scratch: dump every 0700000c MAIN-MAP placement inside the
//! corridor box (x 40..70, z -195..-135) by kind with params — the [D] rows behind the barrel-row
//! stop at z -150.23 [L 2026-10-31]. Prints only.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("0700000c.bin"))
        .expect("archive");
    let data = img
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    println!(
        "maps {} main {mm} placements {}",
        lv.maps.len(),
        lv.maps[mm].placements.len()
    );
    for (mi, map) in lv.maps.iter().enumerate() {
        if mi == mm { continue; }
        for (i, p) in map.placements.iter().enumerate() {
            let t = p.translation;
            if !(-200.0..=-130.0).contains(&t[2]) { continue; }
            let k = p.flags & 0xff;
            let name = if p.by_hash() {
                lv.by_hash.get(&p.hashcode)
                    .and_then(|&(m, o)| lv.maps.get(m).and_then(|mp| mp.objects.get(o)).map(|ob| ob.name.trim().to_string()))
                    .unwrap_or_else(|| format!("hash {:08x}", p.hashcode))
            } else {
                map.objects.get(p.index as usize).map(|ob| ob.name.trim().to_string()).unwrap_or_else(|| format!("idx {}", p.index))
            };
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            println!("map{mi} #{i} kind {k:#04x} \"{name}\" @({:.2},{:.2},{:.2}) params [{}]", t[0], t[1], t[2], params.join(" "));
        }
    }
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        let t = p.translation;
        let want = [556usize, 558, 559, 560];
        if !want.contains(&i) { continue; }
        let k = p.flags & 0xff;
        let target = if p.by_hash() {
            lv.by_hash.get(&p.hashcode).copied()
        } else if (p.index as usize) < lv.maps[mm].objects.len() {
            Some((mm, p.index as usize))
        } else {
            None
        };
        let name = target
            .map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
            .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode));
        let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
        let cel = if p.creates_cel() { " CEL" } else { "" };
        println!(
            "#{i} kind {k:#04x}{cel} \"{name}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
            t[0], t[1], t[2], params.join(" "), p.hashcode, p.flags
        );
    }
}
