//! lock census: dump every 0x2E (lock) placement across every level archive, with kind (param 0),
//! code (param 1), out channel (param 5) and the Operative tag (param 6). Opt-in NF_ISO; prints
//! only. The kind-0 + non-zero-out rows are the `Lock_Update` 0x1ae3d8 case-2 re-lock arm's
//! reachable set.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored"]
fn dump_all_lock_placements() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, _bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut archives: Vec<String> = entries.iter()
        .map(|e| e.name.to_ascii_uppercase())
        .filter(|n| n.len() == 12 && n.starts_with("0700") && n.ends_with(".BIN"))
        .collect();
    archives.sort();
    archives.dedup();
    for a in &archives {
        let data = match std::panic::catch_unwind(|| read_archive(&iso, a)) {
            Ok(d) => d,
            Err(_) => { println!("== {a}: READ FAIL"); continue; }
        };
        let lv = match Level::parse(&data) {
            Ok(lv) => lv,
            Err(e) => { println!("== {a}: parse fail {e}"); continue; }
        };
        let mm = match lv.main_map() { Some(m) => m, None => { println!("== {a}: no main map"); continue; } };
        let mut rows = 0;
        for (mi, map) in lv.maps.iter().enumerate() {
            for (i, p) in map.placements.iter().enumerate() {
                if p.flags & 0xff != 0x2e { continue; }
                rows += 1;
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
                let (kind, code, out, tag) = (param(0), param(1), param(5), param(6));
                let main = if mi == mm { " MAIN" } else { "" };
                let mark = if kind == 0 && out != 0 { "  <<<< KIND0 OUT!=0" } else { "" };
                println!("{a} map{mi}{main} #{i} kind {kind} code {code:#x} out {out:#x} tag {tag} p3 {:#x} p4 {:#x} params [{}]{mark}",
                    param(3), param(4), p.params.iter().map(|(x, y)| format!("{x}:{y:#x}")).collect::<Vec<_>>().join(" "));
            }
        }
        println!("== {a}: {rows} lock placements ==");
    }
}
