//! The start-script census (playscript-deaf-0x168): every 07* level's drone placements (object
//! kind 0x0F, main map — the map the game instantiates [M ResetMap_Load 0x1d6360's single flag-1
//! create over the last-parsed map's block 0x1a; docs/systems/cover-nodes-load.md]) through the
//! port's own resolver `drone::start_script` (placement parameter 1 custom, else the
//! `NDrone2_DefaultInit` 0x14b300 table row — the only +0x554 writer [M 0x14b300:611-627]), pinning
//! which placements resolve to the three deaf scripts — the scripts whose PlayScript ENTER sets
//! `+0x4F8 |= 0x400000` [M 0x15ba68:73 (the 0x6000168/0x6000169 shared tail, disasm
//! 0x15bbf4-0x15bc08), :84 (0x60007ac's own OR, disasm 0x15bc40-0x15bc54)].
//!
//! [D] result on the retail disc: **no placement anywhere resolves to 0x06000168** — the arm (and
//! its deafness) is dead on disc; the only deaf-script placements are 07000001 → 0x06000169 (the
//! drive guard, docs/systems/drone-first-shot.md §1) and 07000012 → 0x060007ac.
//! Opt-in like the goldens: set `NF_ISO`; without it every test prints "skip" and passes.

use nf_formats::iso9660::IsoImage;
use std::collections::BTreeMap;

#[test]
fn the_deaf_start_scripts_census() {
    let Some(path) = std::env::var_os("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&path).unwrap_or_else(|e| panic!("NF_ISO={path:?}: {e}"));
    let elf = iso.read_file(iso.find("/ACTION.ELF").expect("no /ACTION.ELF")).unwrap();
    let tables = nf_formats::drone::DroneAnimData::from_action_elf(&elf).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    // (level file name, kind-0x0F placements) → the resolved start scripts, keyed for the pin.
    let mut census: BTreeMap<(String, u32), Vec<u32>> = BTreeMap::new();
    for fe in entries.iter().filter(|e| e.name.len() == 12 && e.name.starts_with("070") && e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(level) = nf_formats::level::Level::parse(&data) else { continue };
        let Some(m) = level.main_map() else { continue };
        let level_id = u32::from_str_radix(&fe.name[..8], 16).unwrap_or(0);
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x0f) {
            let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
            if let Some(s) = nf_formats::drone::start_script(&tables, param, level_id).script() {
                census.entry((fe.name.to_ascii_uppercase(), param(0))).or_default().push(s);
            }
        }
    }
    // The pin [D]: across the whole disc, exactly two drone placements resolve to a deaf script,
    // and NONE resolves to 0x06000168 — its PlayScript ENTER arm is dead on retail data.
    let deaf: Vec<(String, u32)> = census.iter()
        .flat_map(|((lvl, _), v)| v.iter().filter(|s| matches!(**s, 0x0600_0168 | 0x0600_0169 | 0x0600_07ac)).map(move |s| (lvl.clone(), *s)))
        .collect();
    assert_eq!(deaf, vec![
        ("07000001.BIN".to_string(), 0x0600_0169), // the drive guard [D, docs/systems/drone-first-shot.md §1]
        ("07000012.BIN".to_string(), 0x0600_07ac), // the corridor's scripted guard
    ], "the deaf-script placements on disc: {deaf:?}");
    assert!(!census.values().flatten().any(|s| *s == 0x0600_0168),
        "0x06000168 placement found on disc — the census premise broke: {census:?}");
}
