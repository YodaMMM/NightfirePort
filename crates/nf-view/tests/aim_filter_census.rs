//! aim-finish: the two aim-ray filters' per-level census — MEASURED counts so a filter that
//! drops nothing on a level cannot hide behind a void pass.
//!
//! Filter A (the aim ray's LEVEL stage, `CollisionWorld::build` → `segment_aim_scoped`): a stat
//! run is skipped for the aim ray unless its instance's PLACEMENT flags word carries 0x8000
//! (the runtime-cel gate [M parsemap_block_map_data_static 0x1d1998:92 — only a placement whose
//! flags carry 0x8000 got a runtime cel; Collide_Pick's walk sees cels]). The filter reads
//! `inst.placement.flags` (world.rs, the aim-snap-placement fix fc6e21313 — the object flags
//! never carry 0x8000 and the first cut read them, dropping every static run). Counted here per
//! level: the collision-bearing instances the filter KEEPS (0x8000 set) and DROPS (clear).
//!
//! Filter B (the aim ray's OBJECT stage, `Drones::aim_snap`): every script-entity/door
//! candidate is gated on the object's `obj+0xF0` word = glist+0x44 [M Control_SetGList
//! 0x134960:11; parsemap_block_entity_params 0x1d1508] — skip on `& 0x60` (the pick mask) or
//! `& 0x3000` (the per-cel pre-filter 0x1eb914). That word is the OBJECT record's flags (the
//! game's own gate reads the runtime object's word, not the placement's), so the census counts
//! the level's objects the gate would skip and the placements that resolve to them.
//!
//! Opt-in via NF_ISO; prints only (assertion-free).

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
#[ignore = "assertion-free aim-filter census; run with --ignored"]
fn census_the_two_aim_filters_per_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    for lvl in ["07000008", "0700000c", "07000014", "07000016", "0700001b"] {
        let data = read_archive(&iso, &format!("{lvl}.bin"));
        let lv = Level::parse(&data).expect("parse");
        let main = lv.main_map().unwrap();
        let mut insts = 0usize;      // collision-bearing instances (the stat fold's body)
        let mut kept = 0usize;       // filter A keeps (placement 0x8000 set)
        let mut dropped = 0usize;    // filter A drops (placement 0x8000 clear)
        let mut gate_objs = 0usize;  // filter B: objects whose word & 0x60 | & 0x3000
        let mut gate_insts = 0usize; // filter B: placements resolving to a gated object
        for p in &lv.maps[main].placements {
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[main].objects.len() { Some((main, p.index as usize)) } else { None };
            let Some((m, o)) = target else { continue };
            let obj = &lv.maps[m].objects[o];
            if obj.collision.is_none() { continue; }
            insts += 1;
            if p.flags & 0x8000 != 0 { kept += 1; } else { dropped += 1; }
            if obj.flags & 0x60 != 0 || obj.flags & 0x3000 != 0 { gate_insts += 1; }
        }
        for o in &lv.maps[main].objects {
            if o.collision.is_none() { continue; }
            if o.flags & 0x60 != 0 || o.flags & 0x3000 != 0 { gate_objs += 1; }
        }
        println!("{lvl}: filter A (aim_pick, placement 0x8000): kept {kept} dropped {dropped} of {insts} collision-bearing instances | filter B (obj+0xF0 gate 0x60/0x3000): {gate_objs} gated objects, {gate_insts} placements on them");
    }
}
