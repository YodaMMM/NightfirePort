//! level-05-every-objective-4: dump the 07000005 script players (kind 0x31), the stream table
//! of hash 06000952 (the cut-scene that disables Bond at frame 8757 and never frees), placement
//! #284 and every placement whose params carry 0x42/0x62. Scratch probe.

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let mm = level.main_map().expect("main map");

    println!("== the script players (kind 0x31) ==");
    for (i, p) in level.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff == 0x31 {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { None };
            let name = target.map(|(m, o)| level.maps[m].objects[o].name.trim().to_string()).unwrap_or_default();
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("sp #{i} \"{}\" @({:.2},{:.2},{:.2}){ps}", name, p.translation[0], p.translation[1], p.translation[2]);
        }
    }

    println!("== placement 284 and the 0x42/0x62 param holders ==");
    for (i, p) in level.maps[mm].placements.iter().enumerate() {
        let has = p.params.iter().any(|(_, v)| *v == 0x42 || *v == 0x62);
        if i == 284 || has {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { None };
            let name = target.map(|(m, o)| level.maps[m].objects[o].name.trim().to_string()).unwrap_or_default();
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("pl #{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}){ps}", p.flags & 0xff, name, p.translation[0], p.translation[1], p.translation[2]);
        }
    }

    for h in [0x0600_0952u32] {
        let Some(n) = level.nis.get(&h) else { println!("nis {h:08x} NOT in the level set"); continue };
        println!("== nis {h:08x}: camera {} len {} streams {} ==", n.camera as u8, n.length, n.streams.len());
        for (si, st) in n.streams.iter().enumerate() {
            // Walk the port's op lengths: 5:4, 6:4, 4:4, 7:var, 9:2, 0xa:14, 0xc:2, 0x12:3+4n+1, 0x16:9, 0x17:?
            // Print every op 0x12's (n, kind, params) plus any unknown op byte.
            let c = &st.commands;
            let mut pc = 0usize;
            let mut events = String::new();
            while pc < c.len() {
                match c[pc] {
                    0x12 if pc + 2 < c.len() => {
                        let cnt = c[pc + 1] as usize;
                        let kind = c[pc + 2];
                        let mut ps = Vec::new();
                        for k in 0..cnt {
                            let o = pc + 3 + 4 * k;
                            if o + 4 <= c.len() {
                                ps.push(u32::from_le_bytes([c[o], c[o + 1], c[o + 2], c[o + 3]]));
                            }
                        }
                        events.push_str(&format!(" ev{kind:#02x}{ps:?}"));
                        pc += 3 + 4 * cnt + 1;
                    }
                    0x05 | 0x06 | 0x04 => pc += 4,
                    0x09 | 0x0c => pc += 2,
                    0x0a => pc += 14,
                    0x16 => pc += 9,
                    0x17 => pc += 7,
                    0x07 => {
                        if pc + 9 < c.len() {
                            let n = c[pc + 8] as usize;
                            pc += 1 + 9 + 4 * n + 1;
                        } else { pc = c.len(); }
                    }
                    _ => { events.push_str(&format!(" op{:02x}?", c[pc])); pc += 1; }
                }
            }
            println!("  stream {si}: len {} keys {}..{} events:{events}", c.len(), st.first_key, st.last_key);
        }
    }
}
