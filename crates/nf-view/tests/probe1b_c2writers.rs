//! level-1b-drake-fight probe: scan every archive of 0700001b (the level bin + every loadable
//! pack) for event-0x12 commands whose channel word is 194 (0xc2) — the missing AND #202 input
//! (ch 19 = AND(193,194) → the launch order). Also scans for 10 (0x0a), 15 (0xf), 30 (0x1e),
//! 46 (0x2e), 75 (0x4b) for the map of every writer of the fight chain. Raw-byte sweep: the
//! event op 0x12 (count, kind 0x0D/0x11, params LE u32s). Assertion-free.

use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Option<Vec<u8>> {
    let img = IsoImage::open(iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).ok()?;
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive))?;
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()
}

fn scan(buf: &[u8], what: &str, chans: &[u32]) {
    let n = buf.len();
    for i in 0..n.saturating_sub(16) {
        if buf[i] != 0x12 { continue; }
        let cnt = buf[i + 1] as usize;
        if cnt == 0 || cnt > 8 || i + 4 + cnt * 4 + 1 > n { continue; }
        let kind = buf[i + 2];
        if kind != 0x0d && kind != 0x11 { continue; }
        let p = |k: usize| u32::from_le_bytes(buf[i + 3 + k * 4..i + 7 + k * 4].try_into().unwrap());
        for &c in chans {
            let hit = match kind {
                0x0d => cnt >= 1 && p(0) == c,
                0x11 => cnt >= 3 && (p(1) == c || p(2) == c),
                _ => false,
            };
            if hit {
                let ps: Vec<String> = (0..cnt).map(|k| format!("{:x}", p(k))).collect();
                println!("{what} @ {i:#x}: ev{kind:#04x}[{}]", ps.join(","));
            }
        }
    }
}

#[test]
#[ignore = "assertion-free sweep; run with --ignored --nocapture"]
fn sweep_0700001b_channel_writers() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let chans = [10u32, 15, 19, 30, 46, 75, 193, 194];
    let Some(data) = read_archive(&iso, "0700001b.bin") else { return };
    // The level bin's own embedded scripts (kind-7 entries of every pack the type-0xC list names).
    let lb = match LevelBin::parse(&data) { Ok(l) => l, Err(e) => { eprintln!("parse: {e}"); return } };
    for e in lb.entries.iter().filter(|e| e.kind == 0xC) {
        let r = lb.data(&data, e);
        if r.len() < 5 { continue; }
        let (hash, flag) = (u32::from_le_bytes([r[0], r[1], r[2], r[3]]), r[4]);
        if flag == 0 || flag > 3 { continue; }
        let pack_name = format!("{:08x}.bin", (0x0700_001b & 0xff0f_ffff) | ((flag as u32 + 7) << 20));
        let Some(pack) = read_archive(&iso, &pack_name) else { eprintln!("{pack_name} missing"); continue };
        let Ok(plb) = LevelBin::parse(&pack) else { continue };
        for pe in plb.entries.iter().filter(|p| p.kind == 7) {
            let bytes = plb.data(&pack, pe);
            scan(bytes, &format!("pack {pack_name} script {:08x}", pe.hash()), &chans);
        }
    }
    // The map blocks' script entities (map kind 7 entries inline in the level bin).
    for (mi, m) in lb.entries.iter().enumerate() {
        if m.kind != 7 { continue; }
        scan(lb.data(&data, m), &format!("levelbin entry {mi} hash {:08x}", m.hash()), &chans);
    }
    // The briefing/movie wrappers around the station (the entry flow's packs) — a wrapper's
    // own script can pre-set channels the level's chain waits on.
    for name in ["07100015.bin", "07100016.bin", "07200014.bin", "07200015.bin", "07300017.bin"] {
        let Some(pack) = read_archive(&iso, name) else { eprintln!("{name}: not on the disc"); continue };
        let Ok(plb) = LevelBin::parse(&pack) else { eprintln!("{name}: parse failed"); continue };
        for pe in plb.entries.iter().filter(|p| p.kind == 7) {
            scan(plb.data(&pack, pe), &format!("{name} script {:08x}", pe.hash()), &chans);
        }
        println!("{name}: scanned");
    }
}
