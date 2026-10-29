//! The disc's `DroneModeSettings` table (0x29ad40, 35 rows × 12 B) read straight out of the
//! ISO's ACTION.ELF — the [D] pin for the dispatch census in docs/systems/ai-dmode-inits.md
//! (`NDrone2_DoModeSettingsOLD` 0x14cfa0 dispatches row `param_5 < 0x24`: byte 0 → `Drone+0xC5`,
//! byte 1 → `Drone+0x44`, word +4 → `+0x500`/`+0x50C` [M 0x14cfa0:0x14d010-0x14d07c]; word +8 is
//! the row's init fn, `jalr` [M 0x14cfa0:0x14d080-0x14d090]). Every row's mode / team / resting
//! alertness / init function below is disc data, not a port choice. The companion per-init body
//! summaries ([M] decompile reads) live in the docs appendix.

use nf_formats::elf::Elf;
use nf_formats::iso9660::IsoImage;

/// (fn vaddr, name) — the names are the ELF symbol names of the init functions; the addresses
/// are what the disc table's word +8 must hold. `Defaults` 0x14d528 is NOT a row fn (no row
/// points at it) — it is only reached through Normal's tail-call, so it stays out of this list.
const FNS: [(u32, &str); 32] = [
    (0x14d958, "Normal"),
    (0x14d978, "Guard"),
    (0x14d998, "Retreater"),
    (0x14d9b8, "Sniper"),
    (0x14da78, "Stealth"),
    (0x14dab0, "Attacker"),
    (0x14dad0, "RunToPoint"),
    (0x14db18, "Assassin"),
    (0x14db38, "HostageKiller"),
    (0x14dc18, "Hostage"),
    (0x14dc88, "HostageTied"),
    (0x14dcf8, "JustStand4Demo"),
    (0x14dd20, "DeleteMe"),
    (0x14dd28, "Civilian"),
    (0x14de48, "CivilianScared"),
    (0x14df28, "MissionFailer"),
    (0x14df68, "Mayhew"),
    (0x14df90, "Ninja"),
    (0x14df98, "AlarmRaiser"),
    (0x14dfe0, "SearchLight"),
    (0x14e018, "Ambush"),
    (0x14e050, "Zoe"),
    (0x14e098, "PartyGirl"),
    (0x14e158, "CivilianGuard"),
    (0x14e238, "Interogator"),
    (0x14e258, "CivDoorGuard"),
    (0x14e348, "TruckDriver"),
    (0x14e370, "CastleChatGuard1"),
    (0x14e3a0, "CastleChatGuard2"),
    (0x14e3e0, "SniperAlert"),
    (0x14e490, "PartyGirlLooker"),
    (0x14e568, "Bot"),
];

/// (row, mode byte, team byte, alert word, fn) for all 35 rows — read off the disc table
/// ([D]; alert 0x3f800000 = the f32 1.0). Rows 31/32/34 carry fn 0 (the census's empty rows).
const ROWS: [(u32, u8, u8, u32, u32); 35] = [
    (0, 0x00, 1, 0x0000_0000, 0x14d958), // Normal
    (1, 0x01, 1, 0x0000_0000, 0x14d978), // Guard
    (2, 0x01, 1, 0x3f80_0000, 0x14d998), // Retreater
    (3, 0x02, 1, 0x0000_0000, 0x14d9b8), // Sniper
    (4, 0x01, 1, 0x0000_0000, 0x14da78), // Stealth
    (5, 0x06, 1, 0x0000_0000, 0x14dab0), // Attacker
    (6, 0x07, 1, 0x3f80_0000, 0x14dad0), // RunToPoint
    (7, 0x03, 1, 0x0000_0000, 0x14db18), // Assassin
    (8, 0x04, 1, 0x0000_0000, 0x14db38), // HostageKiller
    (9, 0x05, 3, 0x0000_0000, 0x14dc18), // Hostage
    (10, 0x05, 3, 0x0000_0000, 0x14dc88), // HostageTied
    (11, 0x08, 3, 0x0000_0000, 0x14dcf8), // JustStand4Demo
    (12, 0x15, 1, 0x0000_0000, 0x14dd20), // DeleteMe
    (13, 0x09, 3, 0x0000_0000, 0x14dd28), // Civilian
    (14, 0x0a, 3, 0x3f80_0000, 0x14de48), // CivilianScared
    (15, 0x0b, 1, 0x0000_0000, 0x14df28), // MissionFailer
    (16, 0x0c, 2, 0x0000_0000, 0x14df68), // Mayhew
    (17, 0x0d, 1, 0x0000_0000, 0x14df90), // Ninja
    (18, 0x0e, 1, 0x3f80_0000, 0x14df98), // AlarmRaiser
    (19, 0x0f, 1, 0x0000_0000, 0x14dfe0), // SearchLight
    (20, 0x10, 1, 0x0000_0000, 0x14e018), // Ambush
    (21, 0x11, 2, 0x0000_0000, 0x14e050), // Zoe
    (22, 0x12, 3, 0x0000_0000, 0x14e098), // PartyGirl
    (23, 0x13, 3, 0x0000_0000, 0x14e158), // CivilianGuard
    (24, 0x14, 1, 0x0000_0000, 0x14e238), // Interogator
    (25, 0x16, 3, 0x0000_0000, 0x14e258), // CivDoorGuard
    (26, 0x17, 3, 0x0000_0000, 0x14e348), // TruckDriver
    (27, 0x18, 1, 0x0000_0000, 0x14e370), // CastleChatGuard1
    (28, 0x18, 1, 0x0000_0000, 0x14e3a0), // CastleChatGuard2
    (29, 0x19, 1, 0x3f80_0000, 0x14e3e0), // SniperAlert
    (30, 0x12, 3, 0x0000_0000, 0x14e490), // PartyGirlLooker
    (31, 0x02, 1, 0x0000_0000, 0x0000_0000),
    (32, 0x1a, 3, 0x0000_0000, 0x0000_0000),
    (33, 0x1e, 2, 0x0000_0000, 0x14e568), // Bot
    (34, 0x08, 3, 0x0000_0000, 0x0000_0000),
];

#[test]
fn dmode_table_rows_match_the_disc() {
    if std::env::var_os("NF_ISO").is_none() {
        eprintln!("NF_ISO not set; skip");
        return;
    }
    let iso = IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open ISO");
    let elf_bytes = iso
        .read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF on the disc"))
        .expect("read ACTION.ELF");
    let e = Elf::parse(&elf_bytes).expect("parse ACTION.ELF");
    let segs = e.load_segments(&elf_bytes).expect("load segments");
    let at = |addr: u32| -> [u8; 12] {
        for (va, b, _) in &segs {
            if addr >= *va && ((addr - va) as usize + 12) <= b.len() {
                let o = (addr - va) as usize;
                let mut out = [0u8; 12];
                out.copy_from_slice(&b[o..o + 12]);
                return out;
            }
        }
        panic!("{addr:#x} outside the loaded data");
    };
    for (row, mode, team, alert, f) in ROWS {
        let b = at(0x29ad_40 + row * 12);
        let got_fn = u32::from_le_bytes([b[8], b[9], b[10], b[11]]);
        assert_eq!(b[0], mode, "row {row} mode byte");
        assert_eq!(b[1], team, "row {row} team byte");
        assert_eq!(
            u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            alert,
            "row {row} resting alertness"
        );
        assert_eq!(got_fn, f, "row {row} init fn");
    }
    // The 32 non-NULL pointers are exactly the init_DMODE_* row symbols (no stray fn, none
    // missing; Defaults 0x14d528 is the 33rd family member but never a row value).
    let mut table_fns: Vec<u32> = ROWS.iter().map(|r| r.4).filter(|f| *f != 0).collect();
    table_fns.sort_unstable();
    let mut syms: Vec<u32> = FNS.iter().map(|(a, _)| *a).collect();
    syms.sort_unstable();
    assert_eq!(table_fns, syms, "table fns == the init_DMODE_* symbol set");
}
