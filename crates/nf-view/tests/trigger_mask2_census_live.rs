//! trigger-mask2-bullets: the `mask & 2` (class-5 BULLET contact) census over every story level's
//! trigger placements — `Trigger_Create` 0x1b1828 defaults the mask to 2 when the param is 0
//! [M docs/fields/trigger.md +0x36], and `Trigger_Hit` 0x1b1fc8:124-163 fires on class-5 contacts
//! (the port's live bullet objects — rounds, grenades, rockets, satchels; `Bullet_init` 0x12c5a0:87
//! writes obj+0xff = 5) when mask bit 2 is set: types 1/2 need every nonzero filter entry to equal a
//! touching bullet's weapon-def id, an empty filter list fires on any contact. This census says
//! where the case is reachable on the disc and what ids each plate's filter wants. Opt-in NF_ISO
//! dump; prints only (the pass/fail pins live in the triggers.rs unit tests).
//!
//! The trigger records come from the port's own `triggers::build` — the verified param→field
//! mapping (docs/fields/trigger.md's [L-read] rows), so mask and filter list are exactly what the
//! port's `Trigger::hit` reads. Redo of standins-gadget-937's census (its arm was fed the explosion
//! list; this task drives the same placements from the port's bullet/grenade state).

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
fn dump_mask2_triggers_on_every_story_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let levels = ["07000001", "07000002", "07000003", "07000004", "07000005", "07000006",
        "07000007", "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d",
        "0700000e", "0700000f", "07000010", "07000011", "07000012", "07000013", "07000014",
        "07000015", "07000016", "0700001b", "0700004a"];
    let mut grand_total = 0usize;
    let mut grand_with2 = 0usize;
    for lv_name in levels {
        let Ok(data) = std::panic::catch_unwind(|| read_archive(&iso, &format!("{lv_name}.bin"))) else {
            println!("{lv_name}: archive unreadable"); continue;
        };
        let Ok(Ok(lv)) = std::panic::catch_unwind(|| Level::parse(&data)) else {
            println!("{lv_name}: parse failed"); continue;
        };
        let triggers = nf_game::triggers::build(&lv);
        let mut with2: Vec<String> = Vec::new();
        for t in &triggers {
            grand_total += 1;
            if t.mask & 2 == 0 { continue; }
            grand_with2 += 1;
            let filters: Vec<String> = t.list.iter().take_while(|&&f| f != 0).map(|f| format!("{f:#x}")).collect();
            with2.push(format!("  #{} kind {} mask {:#x} out {:#x} filter [{}] tris {}",
                t.placement, t.kind, t.mask, t.out, filters.join(","), t.tris.len()));
        }
        println!("{lv_name}: {} built triggers, {} with mask&2", triggers.len(), with2.len());
        for line in &with2 { println!("{line}"); }
    }
    println!("TOTAL: {grand_total} built triggers, {grand_with2} with mask&2");
}
