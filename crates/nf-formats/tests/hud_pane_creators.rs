//! The HUD pane create fns (`HUD_CreateAmmoPane` 0x1a0ed8, `HUD_CreateHealthPane` 0x1a0f40,
//! `HUD_CreateInfoStatusPane` 0x1a10b8, `HUD_CreatePickupStatusPane` 0x1a1140) vs what the port
//! does in their place. Docs: docs/fields/hud-panes.md ("The four create fns" section).
//!
//! Everything asserted here is [D] — bytes read out of the ISO's ACTION.ELF. The [M] decompile pins
//! live in the doc. The load-bearing fact: `HUD_CreateHealthPane`'s 4-sprite loop writes
//! (x 0, y 0, w 512, h 448) into runtime sprites 0x14..0x17, and their `HealthSprInfo` templates
//! ALREADY carry exactly those values — so the port's template-driven draw (the damage-glow
//! full-screen quads in nf-view/src/hud.rs) reproduces create + template without running the fn.

use nf_formats::elf::Elf;
use nf_formats::iso9660::IsoImage;
use nf_formats::hud::HudTables;

fn action_elf() -> Vec<u8> {
    let iso = IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO not set")).expect("open ISO");
    iso.read_file(iso.find("/ACTION.ELF").expect("ACTION.ELF on the disc"))
        .expect("read ACTION.ELF")
}

/// [D] HealthSprInfo 0x2bd050, sprites 0x14..0x17 (the damage-glow quads): x=0, y=0, w=512,
/// h=448 — byte-identical to what `HUD_CreateHealthPane` 0x1a0f40:15-26 writes at create, so the
/// loop is idempotent and the port reading the template is equal to the game's create.
/// Sister: textures 0x0300018d/0x0300018e and colour 0xff0000ff, the values the port's glow
/// block draws with.
#[test]
fn health_glow_templates_carry_the_create_loop_values() {
    if std::env::var_os("NF_ISO").is_none() {
        eprintln!("NF_ISO not set; skip");
        return;
    }
    let elf = action_elf();
    let t = HudTables::from_action_elf(&elf).expect("HudTables");
    assert!(t.health.len() >= 24, "HealthSprInfo has 24 entries");
    for (k, tex) in [(20usize, 0x0300_018d), (21, 0x0300_018d), (22, 0x0300_018e), (23, 0x0300_018e)] {
        let s = &t.health[k];
        assert_eq!((s.x, s.y), (0, 0), "HealthSprInfo[{k}] x/y");
        assert_eq!((s.w, s.h), (512, 448), "HealthSprInfo[{k}] w/h (the create-loop's 0x200/0x1c0)");
        assert_eq!(s.texture, tex, "HealthSprInfo[{k}] texture");
        assert_eq!(s.colour & 0xffffff00, 0xff0000_00, "HealthSprInfo[{k}] red glow colour");
    }
}

/// [D] the health pane's create fn pointer in its `HUDPANECREATE_tag` (0x2bd470 + 8) is
/// 0x1a0f40 — `HUD_CreateHealthPane` — and the ammo pane's is 0x1a0ed8. Sister to the
/// hud_pane_census pin, written from the same disc bytes so a table regression in either
/// file flips both.
#[test]
fn ammo_and_health_create_pointers() {
    if std::env::var_os("NF_ISO").is_none() {
        eprintln!("NF_ISO not set; skip");
        return;
    }
    let elf = action_elf();
    let e = Elf::parse(&elf).expect("parse ACTION.ELF");
    let segs = e.load_segments(&elf).expect("load segments");
    let at = |addr: u32| -> u32 {
        for (va, b, _) in &segs {
            if addr >= *va && ((addr - va) as usize + 4) <= b.len() {
                let o = (addr - va) as usize;
                return u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
            }
        }
        panic!("{addr:#x} outside the loaded data");
    };
    assert_eq!(at(0x2bcf_b8 + 8), 0x1a0e_d8, "AmmoPane create");
    assert_eq!(at(0x2bd4_70 + 8), 0x1a0f_40, "HealthPane create");
}
