//! standins-gadget-937: the `mask & 2` (class-5 contact) reach census over every story level's
//! trigger placements — `Trigger_Create` 0x1b1828 defaults the mask (param 9) to 2 when 0
//! [M docs/atlas/trigger.md +0x36], and `Trigger_Hit` 0x1b1fc8:124-159 fires a Touch on a
//! class-5 (explosion) contact when mask bit 2 is set. The port models no class-5 contacts
//! (triggers.rs hit(): "contacts bit 1 never sets"). This census says whether the case is
//! reachable on the disc: any Touch/TouchOnce (0xEA/0xEB, and 0xDC typed 1/2) whose mask
//! carries bit 2 (pure or mixed with 0x18). Opt-in NF_ISO dump; prints only.

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
fn dump_trigger_masks_on_every_story_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let levels = ["07000001", "07000002", "07000003", "07000004", "07000005", "07000006",
        "07000007", "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d",
        "0700000e", "0700000f", "07000010", "07000011", "07000012", "07000013", "07000014",
        "07000015", "07000016", "0700001b", "0700004a"];
    for lv_name in levels {
        let Ok(data) = std::panic::catch_unwind(|| read_archive(&iso, &format!("{lv_name}.bin"))) else {
            println!("{lv_name}: archive unreadable"); continue;
        };
        let Ok(Ok(lv)) = std::panic::catch_unwind(|| Level::parse(&data)) else {
            println!("{lv_name}: parse failed"); continue;
        };
        let Ok(Some(mm)) = std::panic::catch_unwind(|| lv.main_map()).map(|r| r) else { println!("{lv_name}: no main map"); continue };
        let mut total = 0usize;
        let mut with2 = Vec::new();
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let flags = p.flags & 0xff;
            if !matches!(flags, 0xea | 0xeb | 0xdc) { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0u32, |&(_, v)| v);
            let kind = if flags == 0xea { 1u32 } else if flags == 0xeb { 2 } else { param(0) };
            if kind != 1 && kind != 2 { continue; }
            total += 1;
            let mask = if param(9) == 0 { 2u32 } else { param(9) }; // the Trigger_Create default [M 0x1b1828]
            if mask & 2 != 0 {
                with2.push(format!("  #{i} flags {flags:#04x} kind {kind} mask {mask:#x} (raw p9 {:#x}) out {:#x}",
                    param(9), param(1)));
            }
        }
        println!("{lv_name}: {total} touch-family triggers, {} with mask&2", with2.len());
        for line in &with2 { println!("{line}"); }
    }
}
