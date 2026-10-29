//! script-channel-events census [D]: every `Script_EventHandler` 0x1d94c0 channel-write event —
//! kind 0xD (set channel, params [ch, value]) and kind 0x11 (conditional set, params
//! [test_ch, dest_if_nonzero, dest_if_zero, value]) — in every on-foot level's script streams and
//! loadable packs. Row: archive, script hash, stream, fire tick (the stream's Wait timeline), kind,
//! params. Packs resolved the way the game does: the level archive's type-0xC records
//! (script hash, flag) name the pack `%08x.bin` = level_id with nibble (flag+7) at bits 20-23
//! [M LoadableLoad 0x1d69e8, SP_LoadScript 0x1dbc08; D 07000007's records → 07800007/07900007].
//! Opt-in: NF_ISO. Prints only (the committed test asserts the rows).
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin::LevelBin;
use nf_formats::nis::{Cmd, Nis};

/// The 23 on-foot levels (the walkable set; channel_persist_live's list).
const ONFOOT: [&str; 23] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d", "07000011",
    "07000012", "07000013", "07000014", "07000015", "07000016", "0700001b", "07000041",
    "0700004a", "0700004b",
];

fn read_archive(img: &IsoImage, bin_lba: u64, entries: &[nf_formats::filelist::FileEntry], name: &str) -> Option<Vec<u8>> {
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
    Some(img.read_bytes(bin_lba * 2048 + fe.offset as u64, fe.size as u64).expect("read"))
}

/// (fire_tick, cmd) in the stream's Wait timeline — op 5 arms the wait [M Script_Run 0x1d8548 case 5],
/// the stream's clock passes it, then following commands fire.
fn timed(stream: &nf_formats::nis::Stream) -> Vec<(u32, Cmd)> {
    let mut out = Vec::new();
    let mut wait = 0u32;
    for c in stream.decode() {
        if let Cmd::Wait(t) = c { wait = t as u32; }
        out.push((wait, c.clone()));
    }
    out
}

/// Every 0xD/0x11 row of one parsed archive. `tag` distinguishes the level's own bin from its packs.
fn rows_in(img: &IsoImage, bin_lba: u64, entries: &[nf_formats::filelist::FileEntry], archive: &str, tag: &str) -> Vec<String> {
    let mut rows = Vec::new();
    let Some(data) = read_archive(img, bin_lba, entries, archive) else { return rows };
    let Ok(lb) = LevelBin::parse(&data) else { rows.push(format!("{archive}: PARSE FAIL")); return rows };
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(nis) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        for (si, s) in nis.streams.iter().enumerate() {
            for (t, c) in timed(s) {
                if let Cmd::Event { kind: k @ (0xd | 0x11), params } = &c {
                    rows.push(format!(
                        "{:>10} {:>5} | script {:08x} | stream {:2} | t{:4} | kind 0x{:02x} | params {:x?}",
                        archive, tag, nis.hashcode, si, t, k, params
                    ));
                }
            }
        }
    }
    rows
}

fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let bin_lba = bin.lba as u64;

    // What 07*.bin archives exist at all (attribution context).
    let sevens: Vec<String> = entries.iter().filter(|e| {
        let n = e.name.to_ascii_uppercase();
        n.starts_with("07") && n.ends_with(".BIN") && n.len() == 12
    }).map(|e| e.name.to_ascii_uppercase()).collect();
    println!("==== 07*.bin archives on disc ({}): {} ====\n", sevens.len(), sevens.join(" "));

    for level in ONFOOT {
        let mut rows = Vec::new();
        // The level's own scripts.
        rows.extend(rows_in(&img, bin_lba, &entries, &format!("{level}.bin"), "own"));
        // The level's loadable packs: the type-0xC records name them; read every
        // (flag+7) nibble pack the level's records actually reference.
        if let Some(data) = read_archive(&img, bin_lba, &entries, &format!("{level}.bin")) {
            if let Ok(lb) = LevelBin::parse(&data) {
                let mut packs: Vec<(u32, u8)> = Vec::new();
                for e in lb.entries.iter().filter(|e| e.kind == 0xC) {
                    let r = lb.data(&data, e);
                    if r.len() < 5 { continue; }
                    let (hash, flag) = (u32::from_le_bytes([r[0], r[1], r[2], r[3]]), r[4]);
                    if flag == 0 || flag > 3 { continue; }
                    packs.push((hash, flag));
                }
                packs.sort();
                packs.dedup();
                for (hash, flag) in &packs {
                    let pack = format!("{:08x}.bin", (0x0700_0000 | (u32::from_str_radix(&level[2..], 16).unwrap_or(0) & 0xff0f_ffff)) | ((*flag as u32 + 7) << 20));
                    let n = rows_in(&img, bin_lba, &entries, &pack, "pack");
                    if n.is_empty() {
                        println!("[pack] {level}: {:08x} flag {flag} → {pack}: no 0xD/0x11 rows", hash);
                    } else {
                        rows.extend(n);
                    }
                }
                // The 07F packs: every level also ships a 0x07F000XX.bin (the level loader's own
                // pack, not the SP path's type-0xC records). The census sweeps it too — it is a
                // loadable pack of the level [D the archive list; lib.rs's "the level's 07F pack"].
                let id = u32::from_str_radix(&level[2..], 16).unwrap_or(0);
                let f = format!("{:08x}.bin", 0x07F0_0000 | (id & 0xff0f_ffff));
                let n = rows_in(&img, bin_lba, &entries, &f, "packF");
                if !n.is_empty() { rows.extend(n); }
            }
        }
        if rows.is_empty() {
            println!("==== {level}: no 0xD/0x11 events ====\n");
        } else {
            println!("==== {} ({} rows) ====", level, rows.len());
            for r in &rows { println!("{r}"); }
            println!();
        }
    }
}
