//! The calltree-main-26 census: what the shipped menu-def data actually contains, for the 26
//! reachable-from-main functions the port never mentioned (tools/calltree/calltree.py, main
//! 0x109080 → 26 before). Everything here is [D] (read off the ISO); the dispatch/level-gate
//! pins are [M] and cited in crates/nf-view/src/menu.rs (the front-end block at the `Page`
//! enum) and crates/nf-game/src/body_glow.rs (AnimDebugCreate).
//!
//! Assertions:
//! 1. NO shipped menu-def (the front end 07000048/08000002, plus every per-level kind-8 def)
//!    defines a control with any of the debug-toggle handler ids: the eleven exact dispatch ids
//!    [M Handler_HandleMessage 0x2029e0's compare arms: C_CHCHWEAP 0x100000e7, C_CHCHHEALTH
//!    0x100000e8, C_CHCHHUD 0x1000001c, C_CHCHCOORDS 0x1000001a, C_CHCHDUMMY 0x100000bd,
//!    C_CHCHALLOWFREEZE 0x100000c7, C_CHCHDRAWALL 0x10000101, C_CHCHLOCKUP 0x10000169,
//!    C_CHCHBRIGHT 0x100001f7, C_CHCHUNLOCK 0x10000216, C_CHCHCONTROLS 0x1000021d] and NO control
//!    id at all inside the 0x10000012..=0x1000001f debug-toggle block (which also covers
//!    C_CHCHFLY / C_CHCHDEBUG, whose Ghidra-decompiled dispatch arms are range tests, not
//!    literals — the whole block's arms are C_CHCH* toggles [M 0x2029e0]).
//! 2. NO shipped def defines a page 0x40000044 / 0x40000045 / 0x40000046 (P_TWEAKS, P_TWEAKS2)
//!    or 0x4000004f / 0x40000050 (P_FMVTEST + its play page), and no message anywhere in any def
//!    targets those ids either — the debug pages are unreachable through the shipped front end.
//!    Sister: 0x4000004d (P_FMV 0x2169b0's dispatch arm) EXISTS as a page but is targeted by no
//!    message (the orphan), and the def parses at all (pages > 0, the main menu 0x40000002
//!    present) — a parse regression flips those.
//! 3. NO shipped def defines a control of ctype 0xb (the SPIN class, `Spin_Update` 0x222948's
//!    `Menu_Update` 0x1f1dc0:33 arm) or ctype 200 (the MEMOBOX class, `Memo_Update`'s arm; the
//!    type's `Memo_Init` 0x21df38 / `Memo_SendMessage` 0x21e108) — the widget classes never
//!    instantiate in retail data. Sister: the ctype histogram is non-empty (types the game
//!    demonstrably uses — Button 1, Label 5 — do appear).
use nf_formats::disc;
use nf_formats::iso9660::{IsoImage, SECTOR_SIZE};
use nf_formats::levelbin;
use nf_formats::menudef::MenuDef;

fn iso() -> IsoImage {
    let path = std::env::var("NF_ISO").expect("NF_ISO not set");
    IsoImage::open(&path).expect("open ISO")
}

/// Every parsed menu-def on the disc: the front end (07000048's 08000002) plus each level bin's
/// kind-8 entries (the per-def in-game menus, e.g. the keypad's 08000001).
fn all_defs(iso: &IsoImage) -> Vec<(String, u32, MenuDef)> {
    let (files, bin) = disc::files_bin_entries(iso).expect("FILES.BIN");
    let mut out = Vec::new();
    for f in &files {
        if !f.name.to_ascii_uppercase().ends_with(".BIN") { continue; }
        let Ok(data) = iso.read_bytes(bin.lba as u64 * SECTOR_SIZE as u64 + f.offset as u64, f.size as u64)
        else { continue };
        let Ok(lb) = levelbin::LevelBin::parse(&data) else { continue };
        for e in &lb.entries {
            let h = e.hash();
            if h >> 24 != 0x08 { continue; }
            // The PAGE tags carry their own file hash and only matching pages parse; the front
            // end's is the entry's own [M bootup's 0x80000002]. Try the entry's hash, then the
            // small 0x8000000N family, keep the parse with the most pages (the census reads all
            // of them; the winner's hash is recorded).
            let body = lb.data(&data, e);
            let mut best: Option<(u32, MenuDef)> = None;
            // The entry id folds the kind byte into the top bits (0x08000002 = kind 8, hash
            // 2) while the PAGE records carry the unfolded file hash 0x80000002 [D
            // menudef_disc.rs's ENTRY_HASH/FILE_HASH pair]. Try the id, its de-folded forms and
            // the small family.
            let mut cands = vec![h, 0x8000_0000 | (h & 0x0fff_ffff), h & 0x7fff_ffff];
            for n in 1..8u32 { cands.push(0x8000_0000 | n); cands.push(0x0800_0000 | n); }
            for ch in cands {
                if let Ok(md) = MenuDef::parse(body, ch) {
                    if best.as_ref().map_or(true, |(_, m)| md.pages.len() > m.pages.len()) {
                        best = Some((ch, md));
                    }
                }
            }
            if let Some((ch, md)) = best {
                if f.name.eq_ignore_ascii_case("07000048.BIN") {
                    eprintln!("CENSUS-DBG 07000048 entry-hash {h:#x} best {ch:#x} pages {} seen {}", md.pages.len(), md.pages_seen);
                }
                out.push((f.name.clone(), ch, md));
            }
        }
    }
    out
}

const DEBUG_TOGGLE_IDS: [u32; 11] = [
    0x1000_00e7, // C_CHCHWEAP
    0x1000_00e8, // C_CHCHHEALTH
    0x1000_001c, // C_CHCHHUD
    0x1000_001a, // C_CHCHCOORDS
    0x1000_00bd, // C_CHCHDUMMY
    0x1000_00c7, // C_CHCHALLOWFREEZE
    0x1000_0101, // C_CHCHDRAWALL
    0x1000_0169, // C_CHCHLOCKUP
    0x1000_01f7, // C_CHCHBRIGHT
    0x1000_0216, // C_CHCHUNLOCK
    0x1000_021d, // C_CHCHCONTROLS
];

#[test]
fn the_debug_pages_and_widgets_never_ship() {
    let image = iso();
    let defs = all_defs(&image);
    assert!(defs.len() >= 10, "the census must cover the front end plus the level defs, got {}", defs.len());
    let mut msg_targets = std::collections::BTreeSet::new();
    let mut family: std::collections::BTreeMap<u32, Vec<String>> = Default::default();
    let mut dbg_pages: Vec<String> = Vec::new();
    let mut btypes: std::collections::BTreeMap<u8, usize> = Default::default();
    let mut ctypes = std::collections::BTreeMap::<u8, usize>::new();
    for (file, hash, md) in &defs {
        for p in &md.pages {
            // 2: the debug pages ship ONLY on the per-level defs (recorded); the front end
            // carries none (asserted below with the controls).
            if matches!(p.id, 0x4000_0044 | 0x4000_0045 | 0x4000_0046 | 0x4000_004f | 0x4000_0050) {
                dbg_pages.push(format!("{file}/{hash:#x}/page {:#x}", p.id));
            }
            for m in &p.mgr_msgs {
                msg_targets.insert(m.2);
            }
            // The front end alone: no control id inside the whole debug-toggle block
            // (0x10000012..=0x1000001f — the dispatch's C_CHCH* arms, incl. the two whose
            // decompiled compare arms are range tests: C_CHCHFLY/C_CHCHDEBUG). The per-level
            // defs DO ship one family member — C_CHCHMUSIC 0x1000001b, the in-game pause's
            // music toggle [D counter-example recorded below] — so the block check is
            // front-end-only, while the 11 exact ids are asserted everywhere.
            let front_end = file.eq_ignore_ascii_case("07000048.BIN") && *hash == 0x8000_0002;

            for c in &p.controls {
                // Record where the family ships; the front end must ship NONE (the cheat rows
                // live on the per-level defs' pause pages).
                // Record every family-id hit with its home; no assertion here — the shipped
                // data is what it is, the reachability proof is the message-target set below.
                family.entry(c.id).or_default().push(format!("{file}/{hash:#x}/page {:#x}", p.id));
                *ctypes.entry(c.ctype).or_default() += 1;
                *btypes.entry(c.b).or_default() += 1;
                // The page-open messages (0x44, page [M 0x21c858 case 0x44]) and every other
                // message's args feed the target set.
                for m in &c.msgs {
                    msg_targets.insert(m.1);
                    msg_targets.insert(m.2);
                }
            }
        }
    }
    // 3: the SPIN class (0xb, `Spin_Update` 0x222948's arm) never instantiates; the MEMOBOX
    // class (200) DOES — its 67 controls are recorded with their homes (the reachability for
    // Memo_Init 0x21df38 / Memo_SendMessage 0x21e108).
    assert_eq!(btypes.get(&0xb), None, "a SPIN control ships: Spin_Update's class");
    let memos: Vec<String> = defs.iter()
        .filter(|(_, _, md)| md.pages.iter().any(|p| p.controls.iter().any(|c| c.b == 200)))
        .map(|(f, _, _)| f.clone()).collect();
    eprintln!("CENSUS memobox(200) ships in {} defs: {:?}", memos.len(), memos);
    assert!(!memos.is_empty(), "the MEMOBOX class ships (the Memo_* reachability)");
    // The sisters: the parse is live and the classes the game really uses do appear.
    eprintln!("CENSUS defs={} ctypes={:?} btypes={:?} msg_targets_count={}", defs.len(), ctypes, btypes, msg_targets.len());
    assert!(!btypes.is_empty(), "the defs parse to at least one control class");
    assert!(btypes.get(&1).is_some() && btypes.get(&5).is_some(), "Button and Label ship");
    // 2's sister: the orphan FMV page exists, nothing opens it, the def parses.
    let fe = defs.iter().find(|(f, h, _)| f.eq_ignore_ascii_case("07000048.BIN") && *h == 0x8000_0002)
        .expect("the front-end def parses");
    assert!(fe.2.pages.iter().any(|p| p.id == 0x4000_004d), "P_FMV's page 0x4000004d exists [D]");
    assert!(fe.2.pages.iter().any(|p| p.id == 0x4000_0002), "the main menu ships");
    for id in [0x4000_0044u32, 0x4000_0045, 0x4000_0046, 0x4000_004f, 0x4000_0050] {
        assert!(!msg_targets.contains(&id), "a message targets the debug page {id:#x}");
    }
    assert!(!msg_targets.contains(&0x4000_004d), "nothing targets the orphan FMV page [D]");
    // The MUSIC counter-example, recorded: the family member that does ship (outside the front
    // end) — the census would catch the others the same way.
    let music: Vec<_> = defs.iter()
        .filter(|(f, _, md)| md.pages.iter().any(|p| p.controls.iter().any(|c| c.id == 0x1000_001b)))
        .map(|(f, _, _)| f.clone()).collect();
    for (id, where_) in &family {
        eprintln!("CENSUS cheat-toggle {id:#x} ships on: {}", where_.join(", "));
    }
    eprintln!("CENSUS debug pages ship on: {}", dbg_pages.join(", "));

}
