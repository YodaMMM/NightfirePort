//! walkreal probe (level-14-no-route-hold): the 07000014 guard placements' param 26 — the
//! `NDrone2_DefaultInit` 0x14b300:483-493 lock: `params[0x1a] != 0 && != 0x600021f` sets the
//! movement lock `+0x4f8 & 0x10` at init (the game's ChooseCombatMove head then returns 0 every
//! tick [M 0x150c18:55-57]). Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_014_guard_param26() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("07000014.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let _ = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");

    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let name = level.maps[om].objects[oi].name.trim().to_lowercase();
            if !name.contains("drone") && !name.contains("guard") { continue; }
            let t = p.translation;
            // The guards of interest stand near the west door (-51.5, 40).
            let near = (t[0] + 60.0).abs() < 20.0 && (t[2] - 40.0).abs() < 15.0;
            // The game's params array is the (key, value) pair list the parser keeps; the drone's
            // DefaultInit reads the u32 words — piVar19[0x1a] is the 27th word of the RAW param
            // payload. The parser's pairs are (key, value): find the keys' word order via the raw
            // dump (print all pairs) and pick the pair whose key index matches word 0x1a.
            let p26 = p.params.get(0x1a).map(|&(_, v)| v).unwrap_or(0);
            let p16 = p.params.get(0x10).map(|&(_, v)| v).unwrap_or(0);
            let p19 = p.params.get(0x13).map(|&(_, v)| v).unwrap_or(0);
            let dump: Vec<String> = p.params.iter().enumerate().map(|(i, (k, v))| format!("{i}:{k:#x}={v:#x}")).collect();
            eprintln!("map{mi} plc{pi} \"{name}\" t ({:.2},{:.2},{:.2}) p[0x10]={p16:#x} p[0x13]={p19:#x} p[0x1a]={p26:#x}{}",
                t[0], t[1], t[2],
                if near { "  <-- near the west door" } else { "" });
            if near { eprintln!("     pairs [{}]", dump.join(" ")); }
        }
    }
    eprintln!("ok");
}
