//! The disc's own HUD pane-def table (`PaneList` 0x2be5b8 and the 18 `.data` pane defs) and
//! the front-end menu-def census for type-0xc (M_TEXTBOX) controls. Everything here is [D]
//! (read off the ISO's ACTION.ELF / 07000048.bin); the decompile pins are [M] and cited.
//! Docs: docs/systems/hud-panes.md. The three BSS Msg pane defs (MsgObjectiveStatusPane
//! 0x319710, MsgInfoStatusPane 0x319810, MsgPickupStatusPane 0x319910) are built by the C++
//! static initialiser, so the disc ELF holds no bytes for them — their create/update words
//! are [L ee_snapshot.bin] evidence in the docs, not asserted here.

use nf_formats::disc;
use nf_formats::elf::Elf;
use nf_formats::levelbin;
use nf_formats::menudef::MenuDef;
use nf_formats::hud::HudTables;

fn action_elf() -> Vec<u8> {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO not set");
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF on the disc")).expect("read ACTION.ELF")
}

fn menu_def() -> MenuDef {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO not set");
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    let (files, bin) = disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case("07000048.BIN"))
        .expect("07000048.bin in FILES.BIN");
    let data = iso
        .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64)
        .expect("read 07000048.bin");
    let lb = levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu-def entry");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

/// (pane-def vaddr, create fn, update fn, sprite count) for the `.data` defs the task's 13
/// functions live in. Values read straight from the disc ELF's `.data` [D]; the symbol names
/// and the +0x08/+0x0c/+0x14 layout per docs/systems/hud-panes.md [M + L ee_snapshot.bin].
const PANES: [(u32, &str, u32, u32, u32); 15] = [
    (0x2bcf_b8, "AmmoPane", 0x1a0e_d8, 0x19a7_f0, 8),
    (0x2bd4_70, "HealthPane", 0x1a0f_40, 0x19b9_c8, 24),
    (0x2bd5_38, "MsgMissionStatusPane", 0x1a10_28, 0x19c3_10, 3),
    (0x2bd5_18, "AirPane", 0x199b_70, 0x1a11_c8, 3),
    (0x2bd5_e0, "SightPane", 0x1a0b_b8, 0x1a12_d0, 3),
    (0x2bd8_b8, "NightSightPane", 0x1a0b_b8, 0x19d2_18, 6),
    (0x2bd7_90, "CameraPane", 0x199b_70, 0x1a12_d8, 9),
    (0x2bd0_30, "BloodPane", 0x1a0b_b8, 0x19a6_a8, 2),
    (0x2bde_90, "XrayPane", 0x199b_70, 0x1a14_10, 5),
    (0x2bdf_e8, "SecCamPane", 0x199b_70, 0x1a15_38, 7),
    (0x2be1_40, "OICWPane", 0x19cc_90, 0x19ce_f8, 7),
    (0x2be2_68, "RoninPane", 0x1a0b_b8, 0x1a12_d0, 6),
    (0x2be3_90, "LaserPane", 0x1a0b_b8, 0x1a12_d0, 6),
    (0x2be5_98, "SpacePane", 0x199b_70, 0x19d3_b8, 11),
    (0x2be6_98, "MPAmmoPane", 0x1a0e_d8, 0x19a7_f0, 3),
];

/// [D] every task pane def on the disc carries the task function as its create or update
/// pointer — the dispatch table `HUD_Init` 0x199e88:44-58 walks and `HUD_Update` 0x19a088:20-69
/// ticks. Sister: each def's sprite count comes out non-zero (a swapped pointer pair would
/// zero it).
#[test]
fn pane_defs_wire_the_task_functions() {
    let elf = action_elf();
    let e = Elf::parse(&elf).expect("parse ACTION.ELF");
    let segs = e.load_segments(&elf).expect("load segments");
    let at = |addr: u32| -> u32 {
        for (va, b, _) in &segs {
            if addr >= *va && ((addr - va) as usize) < b.len() {
                let o = (addr - va) as usize;
                return u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
            }
        }
        panic!("{addr:#x} outside the loaded data");
    };
    for (def, name, create, update, count) in PANES {
        assert_eq!(at(def + 8), create, "{name} create");
        assert_eq!(at(def + 0xc), update, "{name} update");
        assert_eq!(at(def + 0x14) & 0xffff, count, "{name} sprite count");
    }
    // PaneList walks 22 slots [M 0x199e88:44-46]; the .data list's own symbol is 136 bytes [D symtab].
    let list = 0x2be5_b8;
    let mut slots = 0;
    for k in 0..22u32 {
        if at(list + 4 * k) != 0 {
            slots += 1;
        }
    }
    assert_eq!(slots, 20, "PaneList non-null slots (13 + 7 lists, 13/14 null)");
}

/// [D] the HudTables parse reads every task pane's SprInfo table through its ELF symbol.
/// Sister: the counts the tables produce are the symbol sizes / 0x2c (tables carry trailing
/// format data, so only `>=` the pane-def count is pinned — [D symtab sizes]).
#[test]
fn hud_tables_contain_the_new_pane_sprite_tables() {
    let elf = action_elf();
    let t = HudTables::from_action_elf(&elf).expect("HudTables");
    assert!(t.oicw.len() >= 7, "OICWSprInfo");
    assert!(t.nightsight.len() >= 6, "NightSightSprInfo");
    assert!(t.xray.len() >= 5, "XraySprInfo");
    assert!(t.secam.len() >= 7, "SecCamSprInfo");
    assert!(t.sight.len() >= 3, "SightSprInfo");
    assert!(t.rccar.len() >= 6, "RCCarSprInfo");
    assert!(t.blood.len() >= 2, "BloodSprInfo");
    assert!(t.ronin.len() >= 6, "RoninSprInfo");
    assert!(t.laser.len() >= 6, "LaserSprInfo");
    assert!(t.camera.len() >= 9, "CameraSprInfo");
}

/// [D] the M_TEXTBOX reachability census: the control factory `Manager_SendMessage`
/// 0x21c858:206-208 builds a textbox (type 0xc) for any page-def control whose type byte is
/// 0xc. This parses the shipped front-end menu def (kind-8 entry 08000002 in 07000048.bin)
/// and counts them; the trio's
/// reachability rides on what ships here.
#[test]
fn menu_def_textbox_census() {
    let md = menu_def();
    let mut hits: Vec<(u32, u32)> = Vec::new();
    for p in &md.pages {
        for c in &p.controls {
            if c.ctype == 0xc {
                hits.push((p.id, c.id));
            }
        }
    }
    // Print for the report (cargo shows captured output on failure or with --nocapture).
    println!("type-0xc (M_TEXTBOX) controls on the shipped pages: {:?}", hits);
    assert!(
        !md.pages.is_empty(),
        "the menu def parsed but is empty — the census would be vacuous"
    );
}
