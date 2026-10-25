//! Census [D]: every level bin's kind-0xD9 (script player) placements whose trigger mask
//! (param 5) carries bit 2 — the SP_Hit 0x1dc470 bullet-bit reactors (level-07-urn-break).
//! Prints the full param set (hash A/B, modes, stay, mask, out, hit points p7, bullet
//! filter p8 = SCRIPTPLAYER_tag+0x37, gate p9, scale p10) and the transform, so the port's
//! bullet arm can be checked against every disc instance. Also counts all kind-0xD9
//! placements per level for context.
use std::fmt::Write as _;

fn g(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let mut out = String::new();
    for fe in entries.iter().filter(|e| e.name.len() == 12 && e.name.ends_with(".bin")) {
        let data = match iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let Ok(level) = nf_formats::level::Level::parse(&data) else { continue };
        let Some(mi) = level.main_map() else { continue };
        let m = &level.maps[mi];
        let sps: Vec<(usize, &nf_formats::placement::Placement)> = m.placements.iter().enumerate()
            .filter(|(_, p)| p.flags & 0xff == 0xd9).collect();
        if sps.is_empty() { continue; }
        let hits: Vec<&(usize, &nf_formats::placement::Placement)> =
            sps.iter().filter(|(_, p)| g(p, 5) & 2 != 0).collect();
        let _ = writeln!(out, "== {} — {} script players, {} with mask bit 2", fe.name, sps.len(), hits.len());
        for (i, p) in &hits {
            let _ = writeln!(out,
                "  #{i} script {:08x}+{:08x} modes [{}, {}] stay {} mask {:04x} out {} hp(p7) {} filter(p8) {} gate {} scale {}",
                g(p, 0), g(p, 1), g(p, 2), g(p, 3), g(p, 4), g(p, 5), g(p, 6), g(p, 7), g(p, 8), g(p, 9), g(p, 10));
            let _ = writeln!(out, "      rot {:?} trans {:?}", p.rotation, p.translation);
            let mut rest = p.params.iter().filter(|(k, _)| *k > 10);
            for (k, v) in rest { let _ = writeln!(out, "      param {k} = {v}"); }
        }
    }
    print!("{out}");
}
