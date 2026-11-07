//! ssys-wire census: every story level's placed Switch (type 0x29) whose placement param 6
//! (`level_tag`+0x44, the SSys item index `Switch_Create` 0x1adcc0:29-31 reads) is non-zero —
//! the switch whose channel id arms a Phoenix security subsystem. Prints only. [D]
use std::env;

fn main() {
    let iso_path = env::var("NF_ISO").expect("NF_ISO");
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let levels = ["07000001","07000002","07000003","07000004","07000005","07000006","07000007",
                  "07000008","07000009","0700000a","0700000b","0700000c","0700000d","07000011",
                  "07000012","07000013","07000014","07000015","07000016","0700001b"];
    for lv in levels {
        let read = |name: &str| -> Option<Vec<u8>> {
            entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
                .and_then(|fe| iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok())
        };
        let Some(data) = read(&format!("{lv}.bin")) else { println!("{lv}: NO ARCHIVE"); continue };
        let Ok(level) = nf_formats::level::Level::parse(&data) else { println!("{lv}: PARSE FAILED"); continue };
        let Some(mm) = level.main_map() else { println!("{lv}: NO MAIN MAP"); continue };
        let mut hits = 0;
        for (pi, p) in level.maps[mm].placements.iter().enumerate() {
            if p.flags & 0xff != 0x29 { continue; }
            let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v);
            if par(6) != 0 {
                hits += 1;
                println!("{lv}: switch #{pi} pos ({:.0},{:.0},{:.0}) ch(p0)={:#x} auto(p2)={} rest(p3)={} busy(p4)={} gate(p5)={:#x} SSYS_ITEM(p6)={}",
                    p.translation[0], p.translation[1], p.translation[2], par(0), par(2), par(3), par(4), par(5), par(6));
            }
        }
        if hits == 0 { println!("{lv}: no 0x29 switch with non-zero param 6"); }
    }
}
