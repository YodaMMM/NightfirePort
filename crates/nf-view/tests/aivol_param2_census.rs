//! aivol_param2_census: census for task `ai-volume-di28`.
//! (1) The AI-volume placements (0xf5/0xf8) and their parameter 2 / 3 / 4 values per level — the
//!     words `Drone_AIVolume_Create` 0x1376c0:85-87 stores at DI+0x28/+0x2c/+0x26.
//! (2) Drone placements (0x0f) with parameter 22 ≠ 0: does the game's `behaviour_util_get`
//!     [M 0x1c7a98] stats-tail walk actually READ the param-22 word of that placement?
//!     The walk is reproduced exactly (gate order, memcpy ranges, count words, tag, bit-pack).
//! Assertion-free probe [D level data].
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The `behaviour_util_get` 0x1c7a98 read walk over a placement's params (param k => `param(k)`).
/// Returns (ok, last_read_index) — the highest param index the walk reads, or the index it
/// stopped at when a gate fails (per the zeroing tail, nothing beyond is read).
fn behaviour_walk(param: impl Fn(u32) -> u32) -> (bool, u32) {
    let ty = param(15);
    if ty >= 0x11 { return (false, 15); }
    let c1 = param(17);
    if param(16) >= 6 { return (false, 16); }
    if !((1..=3).contains(&(c1 & 0xffff))) { return (false, 17); }
    if (c1 >> 16).wrapping_sub(1) > 0x5a { return (false, 17); }
    // memcpy 1 reads params 18 .. 18+c1low-1
    let a = 18 + (c1 & 0xffff);
    let mut last = 17 + (c1 & 0xffff);
    let am = param(a);
    last = last.max(a);
    let c2 = param(a + 1);
    last = last.max(a + 1);
    if am >= 6 { return (false, a); }
    if !((1..=3).contains(&(c2 & 0xffff))) { return (false, a + 1); }
    if (c2 >> 16).wrapping_sub(1) >= 0x5b { return (false, a + 1); }
    // memcpy 2 reads a+2 .. a+2+c2low-1
    let t = a + 2 + (c2 & 0xffff);
    last = last.max(a + 1 + (c2 & 0xffff));
    let tag = param(t);
    last = last.max(t);
    if (tag as i32) >= 0 { return (true, last); }
    if tag & 0xffff_0000 == 0x8001_0000 {
        let per = (tag & 0xffff) / 3;
        last += 3 * per;
    }
    (true, last)
}

#[test]
#[ignore = "assertion-free census dump; run with --ignored --nocapture"]
fn aivol_param2_census() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    println!("== AI volumes (0xf5/0xf8): placement params 2/3/4 per level ==");
    let mut vol_records = 0u32;
    let mut p22_records = 0u32;
    let mut p22_consumed = 0u32;
    for &lv in nf_view::dev::LEVELS {
        let name = format!("{lv:08x}.bin");
        let Ok(img) = IsoImage::open(&iso) else { continue };
        let Ok((entries, bin)) = nf_formats::disc::files_bin_entries(&img) else { continue };
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&name)) else {
            println!("MISSING ARCHIVE {name}"); continue;
        };
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let level = Level::parse(&data).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        let tag = format!("{lv:08x}");
        // (1) AI volumes
        let mut line = String::new();
        for map in &level.maps {
            for (i, p) in map.placements.iter().enumerate().filter(|(_, p)| matches!(p.flags & 0xff, 0xf5 | 0xf8)) {
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
                let v2 = param(2); let v3 = param(3); let v4 = param(4);
                let kind = param(1);
                vol_records += 1;
                if v2 != 0 || v3 != 0 || v4 != 0 {
                    line.push_str(&format!(" #{i:x} k{kind} p2={v2:08x} p3={v3:02x} p4={v4:04x}"));
                }
            }
        }
        if !line.is_empty() { println!("{tag}:{line}"); }
        // (2) drone param 22 consumption
        let mut cons_line = String::new();
        for map in &level.maps {
            for (i, p) in map.placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f) {
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
                let v22 = param(22);
                if v22 == 0 { continue; }
                p22_records += 1;
                let (ok, last) = behaviour_walk(param);
                if ok && last >= 22 { p22_consumed += 1; cons_line.push_str(&format!(" #{i:x}(read,last={last})")); }
                else { cons_line.push_str(&format!(" #{i:x}(UNREAD,ok={ok},last={last},v={v22:08x})")); }
            }
        }
        if !cons_line.is_empty() { println!("{tag} p22:{cons_line}"); }
    }
    println!("== totals: ai-volume records {vol_records}; 0x0f records with p22≠0: {p22_records}, of which the stats-tail walk reads the word: {p22_consumed} ==");
}
