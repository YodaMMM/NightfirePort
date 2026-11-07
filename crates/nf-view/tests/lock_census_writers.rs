//! lock census part 2: for every level with a kind-0 lock placement whose out channel is
//! non-zero, dump the level's own writers that can raise or DROP that channel — the mux/copy
//! triggers (`Trigger_Update` 0x1b1a78 cases 3/4/5/0xC), the SimpleScripts' operate latch
//! (`SS_Update` 0x1a9948) and the switches (`Switch_Create` 0x1adcc0 / `Switch_Update`
//! 0x1adea8). Opt-in NF_ISO; prints only.
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
fn dump_lock_channel_writers() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for a in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin", "07000014.bin", "07000015.bin", "07000046.bin"] {
        let data = read_archive(&iso, a);
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        let mut chans: Vec<u8> = Vec::new();
        for p in lv.maps[mm].placements.iter() {
            if p.flags & 0xff != 0x2e { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            if param(0) == 0 && param(5) != 0 && param(5) < 256 { chans.push(param(5) as u8); }
        }
        chans.sort(); chans.dedup();
        if chans.is_empty() { println!("== {a}: no kind-0 lock with an out channel"); continue; }
        println!("== {a}: lock out channels {:02x?} ==", chans);
        for t in nf_game::triggers::build(&lv) {
            let hit_out = chans.contains(&t.out);
            let hit_in = t.list.iter().any(|&c| c < 256 && chans.contains(&(c as u8)));
            if hit_out || hit_in {
                println!("  trigger #{pl} kind {k} out {o:#04x} lockch {lc:#04x} list {l:02x?}",
                    pl = t.placement, k = t.kind, o = t.out, lc = t.lock, l = t.list.iter().map(|&c| c & 0xff).collect::<Vec<_>>());
            }
        }
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            let param = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
            if k == 0x25 {
                // SS: param 3 is the driving channel word (ss.rs `channel`).
                if chans.contains(&(param(3) as u8)) {
                    println!("  SS #{i} ch {:#04x} mask {:#x} state {:#x} params [{}]", param(3), param(2), param(0),
                        p.params.iter().map(|(x, y)| format!("{x}:{y:#x}")).collect::<Vec<_>>().join(" "));
                }
            }
            if k == 0x29 && chans.contains(&(param(0) as u8)) {
                println!("  switch #{i} ch {:#04x} rest {:#x} gate {:#x} busy {:#x}", param(0), param(3), param(5), param(4));
            }
        }
        // Doors: `Door_SetState` 0x135618:26 writes the door's ch_open on every motion
        // transition — a closing door writes 0 there, a real dropper candidate.
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            if p.flags & 0xff != 0xdb || p.flags & 0xa000 != 0 { continue; }
            let param = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
            let ch_open = param(2) as u16;
            if ch_open != 0 && chans.contains(&(ch_open as u8)) {
                println!("  door #{i} ch_open {:#04x} ch_lock {:#04x} params [{}]", ch_open, param(3),
                    p.params.iter().map(|(x, y)| format!("{x}:{y:#x}")).collect::<Vec<_>>().join(" "));
            }
        }
        // Sensors (0x28): `Sensor_Update` 0x1af618 CLEARS its out channel every tick in states
        // 0/2 — the one writer family that can hold a lock channel at 0. Sound triggers (0xf9):
        // param 4 = the OFF channel — cleared/written on the trigger's off edge.
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            let param = |kk: u32| p.params.iter().find(|(j, _)| *j == kk).map_or(0, |&(_, v)| v);
            if k == 0x28 && p.flags & 0xa000 == 0 {
                let out = param(3) as u16;
                if out != 0 && chans.contains(&(out as u8)) {
                    println!("  sensor #{i} out {out:#04x} (clears it every tick until tripped)");
                }
            }
            if k == 0xf9 && p.flags & 0xa000 == 0 {
                let (on, off) = (param(0) as u16, param(4) as u16);
                if chans.contains(&(on as u8)) || off != 0 && chans.contains(&(off as u8)) {
                    println!("  soundtrigger #{i} on {on:#04x} off {off:#04x}");
                }
            }
        }
    }
}
