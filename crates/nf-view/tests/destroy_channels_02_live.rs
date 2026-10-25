//! destroy-channels-02: the six 0xE1 destroyables of 07000002's main map and the out channel each
//! smash writes, loaded through the real level path.
//!
//! The disc [D `nf-extract placements-all 07000002.bin`, param 0 of every kind-0xE1 placement]:
//! #14 `Computer1` → 0x48 (objective 1's channel), #299/#301 kitchen extinguishers → 0,
//! #390 `cable light1` → **0**, #392 `cable light1` → **0x82** (the electric-arc chain), #394 → 0.
//!
//! The rule [M `Destroy_Create` 0x12c318:35 — obj+0xF6 = placement param 0; the smash writes
//! `switch_channels[obj+0xF6]` (props.rs / Destroy_Smash 0x12bb60)]. The old doc line in
//! `07000002.md` credited BOTH cable lights with 0x82; the disc gives #390 out 0 — only the middle
//! light (#392) feeds the emitter #9 → Hurt #13 chain. This pins the corrected census
//! (docs/levels/07000002-audit.md GAP-2).

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_destroyables_out_channels_match_the_disc() {
    let scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_ref().unwrap();
    let ds = &sim.drones.props.destroy;
    assert_eq!(ds.len(), 6, "the census: six 0xE1 placements [D #14/#299/#301/#390/#392/#394]");
    let ch = |i: u32| {
        ds.iter().find(|d| d.placement == i).unwrap_or_else(|| panic!("destroyable #{i} missing"))
    };
    // [D param 0] per placement; [M 0x12c318:35] the port carries param 0 into obj+0xF6.
    assert_eq!(ch(14).channel, 0x48, "Computer1 → ch 0x48 = objective 1's completion channel");
    assert_eq!(ch(299).channel, 0, "kitchen-extinquish1 writes nothing");
    assert_eq!(ch(301).channel, 0, "kitchen-extinquish2 writes nothing");
    assert_eq!(ch(390).channel, 0, "cable light1 (#390): out 0 on the disc — NOT an 0x82 writer (GAP-2)");
    assert_eq!(ch(392).channel, 0x82, "cable light1 (#392): the one 0x82 writer that feeds emitter #9");
    assert_eq!(ch(394).channel, 0, "cable light #394 writes nothing");
}
