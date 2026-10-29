//! The `Text_SendMessage` 0x223600 census: which shipped menu defs carry type-0xc (M_TEXTBOX)
//! controls, and which message ids anything sends to them. The game routes a message to a
//! control by `__Menu_SendMessage` 0x1f1c18's switch on `control+0x7b` (the type byte) —
//! case 0xc → `Text_SendMessage` [M], whose arms are 0x14 ExitLoop / 0x16 Script_Play /
//! 0x21 Script_Select+slot / 0x2b byte+0x7a / 0x39 identity / 0x3b / 0x3d / 0x47 free-all /
//! 0x53 reveal-tick, default → `Label_SendMessage(+0x94)` (the embedded label) [M 0x223600].
//! `TextBox_Update` 0x2237c8 additionally sends 0x53 to itself per frame [M 0x2237c8:114,309].
//!
//! This test pins the [D] result across every 07*.bin archive on the disc: NO shipped menu def
//! (front-end 08000002 or any in-game 08000001) contains a type-0xc control, so the case-0xc
//! constructor never fires and `Text_SendMessage` — and `TextBox_Update` 0x2237c8, which only
//! walks existing textboxes — are unreachable in the shipped game. The port needs no Text-only
//! arm; its generic `control_msg` (nf-formats/src/menudef.rs) covers every shipped widget type.
//! (Sister census: `hud_pane_census.rs::menu_def_textbox_census`, the front-end def alone.)

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::levelbin;
use nf_formats::menudef::MenuDef;

fn iso() -> IsoImage {
    IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO not set")).expect("open ISO")
}

/// (def hash label, files, menudefs) for every kind-8 menu-def entry on the disc.
struct Found {
    file: String,
    hash: u32,
    def: MenuDef,
}

fn all_menu_defs(iso: &IsoImage) -> Vec<Found> {
    let (files, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    let mut out = Vec::new();
    for f in files.iter().filter(|f| {
        f.name.len() >= 10
            && f.name[..2].eq_ignore_ascii_case("07")
            && f.name.to_ascii_lowercase().ends_with(".bin")
    }) {
        let data = iso
            .read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + f.offset as u64, f.size as u64)
            .unwrap_or_default();
        let Ok(lb) = levelbin::LevelBin::parse(&data) else { continue };
        for e in lb.entries.iter() {
            let h = e.hash();
            if h >> 24 != 0x08 {
                continue;
            }
            // The PAGE tags carry their own file hash; the entry id folds the kind into the
            // top bits (0x08000002 = kind 8, hash 2) while the pages carry 0x80000002 [D
            // menudef_disc.rs's ENTRY_HASH/FILE_HASH pair]. Try the id, its de-folded forms
            // and the small family; keep the parse with the most pages (sister:
            // calltree_census.rs's walk).
            let body = lb.data(&data, e);
            let mut cands = vec![h, 0x8000_0000 | (h & 0x0fff_ffff), h & 0x7fff_ffff];
            for n in 1..8u32 {
                cands.push(0x8000_0000 | n);
                cands.push(0x0800_0000 | n);
            }
            let mut best: Option<MenuDef> = None;
            for ch in cands {
                if let Ok(md) = MenuDef::parse(body, ch) {
                    if best.as_ref().map_or(true, |m: &MenuDef| md.pages.len() > m.pages.len()) {
                        best = Some(md);
                    }
                }
            }
            if let Some(def) = best {
                out.push(Found { file: f.name.to_ascii_uppercase(), hash: h, def });
            }
        }
    }
    out
}

#[test]
fn textbox_census_across_the_disc() {
    if std::env::var_os("NF_ISO").is_none() {
        eprintln!("NF_ISO not set; skip");
        return;
    }
    let iso = iso();
    let defs = all_menu_defs(&iso);
    assert!(defs.len() > 30, "expected the ~40 archives' defs, got {}", defs.len());

    let mut controls_per_def: Vec<(String, u32, Vec<u32>)> = Vec::new();
    let mut create_msgs: BTreeMap<u32, u32> = BTreeMap::new(); // msg id -> count, on 0xc controls
    let mut runtime_msgs: BTreeMap<u32, u32> = BTreeMap::new(); // msg id -> count, targeted at 0xc
    let mut runtime_targets: BTreeSet<(u32, u32, u32)> = BTreeSet::new(); // (hash, page, ctrl)
    let mut fe_textboxes: Vec<u32> = Vec::new();

    // Multiple entries can share the 0800000x name prefix; only the content-bearing parses
    // (pages > 0) are real menu defs — the others are fragments that parse empty.
    let with_pages: Vec<&Found> = defs.iter().filter(|d| !d.def.pages.is_empty()).collect();
    for d in &with_pages {
        let mut ids = Vec::new();
        for p in &d.def.pages {
            for c in &p.controls {
                if c.ctype != 0xc {
                    continue;
                }
                ids.push(c.id);
                if d.hash == 0x0800_0002 {
                    fe_textboxes.push(c.id);
                }
                for (m, _a, _b) in &c.msgs {
                    *create_msgs.entry(*m as u32).or_default() += 1;
                }
            }
            // keyframe msgs aimed at a 0xc control on this page
            let tb: BTreeSet<u32> = p.controls.iter().filter(|c| c.ctype == 0xc).map(|c| c.id).collect();
            for c in &p.controls {
                for s in &c.scripts {
                    for k in &s.keyframes {
                        for m in &k.msgs {
                            let aimed = m.target >= 0 && tb.contains(&(m.target as u32));
                            let self_shot = m.target == -2 && c.ctype == 0xc;
                            if aimed || self_shot {
                                *runtime_msgs.entry(m.id).or_default() += 1;
                                runtime_targets.insert((d.hash, p.id, m.target.max(0) as u32));
                            }
                        }
                    }
                }
            }
        }
        if !ids.is_empty() {
            controls_per_def.push((d.file.clone(), d.hash, ids));
        }
    }

    println!("defs with type-0xc controls ({} files):", controls_per_def.len());
    for (f, h, ids) in &controls_per_def {
        println!("  {f} (0x{h:08x}): {:?} ", ids);
    }
    println!("create-time msg ids on 0xc controls: {create_msgs:?}");
    println!("runtime msg ids aimed at 0xc controls: {runtime_msgs:?}");
    println!("(page, target) pairs: {runtime_targets:?}");
    println!("front-end (08000002) textbox ids: {fe_textboxes:?}");
    let pages: usize = with_pages.iter().map(|d| d.def.pages.len()).sum();
    println!("{} def parses, {} with pages, {} pages walked", defs.len(), with_pages.len(), pages);
    for d in &with_pages {
        println!("  {} 0x{:08x} pages {} ctrls {}", d.file, d.hash, d.def.pages.len(),
                 d.def.pages.iter().map(|p| p.controls.len()).sum::<usize>());
    }

    // [D] pin: NO shipped menu def builds a type-0xc control — the census that makes
    // Text_SendMessage 0x223600 unreachable (its only constructor, Manager_SendMessage
    // case 0xc -> Text_Init 0x222b80 [M, sole jal at 0x21cad4], never fires; and
    // TextBox_Update 0x2237c8 only walks textboxes that exist). A def regression that
    // introduces a 0xc control flips this, which is the point.
    assert!(
        controls_per_def.is_empty(),
        "type-0xc controls appeared: {controls_per_def:?}"
    );
    // Walk sanity: the disc's defs parsed with real content (the FE def among them).
    assert!(defs.len() > 30, "def parses: {}", defs.len());
    assert!(pages > 30, "pages walked: {pages}");
    assert!(with_pages.iter().any(|d| d.hash == 0x0800_0002), "the front-end def parsed with pages");
}
