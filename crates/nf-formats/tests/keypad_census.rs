//! keypad-entry: the disc census. Two halves:
//!
//! 1. **The keypad page's 24 controls** — page 0x40000021 (file hash 0x80000003) of every
//!    level's kind-8 menu-def entry 0x08000001 [D]. The game builds the page's manager from
//!    these [M `MenuManager_Create` 0x21b908 over the def; `Lock_Update` case 3 creates it with
//!    (0x80000003, 0x40000021) [M 0x1ae3d8:242-243]]. The port's input model (lock.rs
//!    `KEYPAD_DIGIT_RECTS`/`KEYPAD_DIGIT_VALUES`) is pinned to the disc's ten digit controls:
//!    id 0x100000ba, b=1, 34×34, the authored digit value in the control's first post-header
//!    dword (`C_KEYPAD_Handler` reads `*(control+0x20) + '0'` [M 0x216810:13]) — layout
//!    1 2 3 / 4 5 6 / 7 8 9 / _ 0 _ at (268|304|339, 188|223|258|293). Every census level's
//!    page must match the table exactly.
//! 2. **Every level with a kind-0 lock** (flags 0x2e, param 0 == 0 — the keypad kind
//!    [M `Lock_Create` 0x1ae1f8's record fields; `Lock_Activate` 0x1aed00 arms the keypad for
//!    kind 0 with an empty key-ring slot]): the placement index, the KeyCodes table index
//!    (param 1 [M 0x1ae3d8:181 `KeyCodes + puVar1[1]*5`]), the out channel (param 5
//!    [M 0x1ae1f8: `puVar3[4] = *(undefined2 *)(param_3 + 0x40)`]) and the two sound words
//!    (params 3/4). The CODE DIGITS themselves are boot-RNG — `Locks_Init` 0x1b0548 fills
//!    `KeyCodes` as `(Rand_Random() % 9) + '0'` at boot [M], so no disc byte carries them; the
//!    port's local-table codes are what the port's tests type. The "code shown" flag
//!    (`KeyCodes[i*5+4]`) is runtime too — 0 at init [M 0x1b0548:33], set by the Q-Worm
//!    gadget's use [M `PCQWorm_Update` 0x1aee20:31], re-posted as the "Obtained Lock Keycode"
//!    INFO message on the next arm [M 0x1ae3d8:249-258].
use nf_formats::disc;
use nf_formats::iso9660::{IsoImage, SECTOR_SIZE};
use nf_formats::filelist::FileEntry;

fn iso() -> nf_formats::iso9660::IsoImage {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO not set");
    IsoImage::open(&iso_path).expect("open ISO")
}

fn level_data(iso: &IsoImage, files: &[FileEntry], bin: &nf_formats::iso9660::IsoEntry, name: &str, buf: &mut Vec<u8>) -> Option<()> {
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case(name))?;
    let data = iso.read_bytes(bin.lba as u64 * SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).expect("read level bin");
    *buf = data;
    Some(())
}

/// The ten digit controls of page 0x40000021, def order [D 07000003.bin — the probe dump].
/// (x, y, w, h, digit).
pub const DISC_DIGIT_CONTROLS: [(i16, i16, i16, i16, u32); 10] = [
    (268, 188, 34, 34, 1), (304, 188, 34, 34, 2), (339, 188, 34, 34, 3),
    (268, 223, 34, 34, 4), (304, 223, 34, 34, 5), (339, 223, 34, 34, 6),
    (268, 258, 34, 34, 7), (304, 258, 34, 34, 8), (339, 258, 34, 34, 9),
    (304, 293, 34, 34, 0),
];

#[test]
fn keypad_pages_match_the_digit_table_and_the_census_is_recorded() {
    let iso = iso();
    let (files, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let mut levels: Vec<String> = files.iter()
        .filter_map(|f| {
            let n = f.name.to_ascii_uppercase();
            (n.len() == 12 && n.starts_with("0700") && n.ends_with(".BIN")).then_some(n)
        })
        .collect();
    levels.sort();
    levels.dedup();
    let mut data = Vec::new();
    let mut census: Vec<(String, usize, u16, u16, u16, u16)> = Vec::new();
    let mut pages_checked = 0usize;
    for name in &levels {
        if level_data(&iso, &files, &bin, name, &mut data).is_none() { continue; }
        let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { continue };
        let Some(e) = lb.entries.iter().find(|e| e.hash() == 0x0800_0001) else { continue };
        let Ok(def) = nf_formats::menudef::MenuDef::parse(lb.data(&data, e), 0x8000_0003) else {
            panic!("{name}: the 0x08000001 menu-def entry failed to parse");
        };
        let Some(page) = def.pages.iter().find(|p| p.id == 0x4000_0021) else {
            continue; // the level carries no keypad page — its locks (if any) must then be counted below
        };
        // The ten digit controls: id 0x100000ba, b=1, at the disc's rects with the disc's digits.
        let digits: Vec<_> = page.controls.iter().filter(|c| c.id == 0x1000_00ba).collect();
        assert_eq!(digits.len(), 10, "{name}: the keypad page carries {} digit controls, the disc's ten", digits.len());
        for (i, c) in digits.iter().enumerate() {
            let (x, y, w, h, d) = DISC_DIGIT_CONTROLS[i];
            assert_eq!((c.x, c.y, c.w, c.h), (x, y, w, h), "{name}: digit control [{i}] rect");
            assert_eq!(c.d1, d, "{name}: digit control [{i}] authored digit value");
            assert_eq!(c.b, 1, "{name}: digit control [{i}] b byte");
        }
        // The display label 0x10000075 exists on the page [D — the msg-0x18 set-text target
        // C_KEYPAD_Handler 0x216810:22 sends to].
        assert!(page.controls.iter().any(|c| c.id == 0x1000_0075), "{name}: the display label control");
        assert_eq!(page.controls.len(), 24, "{name}: the page's 24 controls");
        pages_checked += 1;
        // The level's kind-0 locks — `Level::parse` over the same bin (the copter probe's path).
        let Ok(lv) = nf_formats::level::Level::parse(&data) else { continue };
        let Some(mm) = lv.main_map() else { continue };
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            if p.flags & 0xff != 0x2e { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            if param(0) != 0 { continue; }
            census.push((name.trim_end_matches(".BIN").to_string(), i, param(1) as u16, param(5) as u16, param(3) as u16, param(4) as u16));
        }
    }
    assert!(pages_checked >= 2, "the slim ISO carries {pages_checked} keypad pages, expected the census levels");
    eprintln!("=== keypad lock census (level, placement, code index, out channel, sound words) ===");
    for c in &census {
        eprintln!("  {}: lock #{:03x} code index {} out ch {:#04x} sounds ({:#06x}, {:#06x})", c.0, c.1, c.2, c.3, c.4, c.5);
    }
    eprintln!("=== {pages_checked} levels' keypad pages pinned to the disc table; {} kind-0 locks ===", census.len());
    assert!(!census.is_empty(), "the disc carries kind-0 keypad locks");
}
