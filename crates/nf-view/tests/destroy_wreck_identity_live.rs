//! destroy_wreck_identity: an intact Destroy leaf (0xE1) draws until it is smashed, even when its
//! destroyed glist shares its bounding sphere exactly.
//!
//! The renderer tells a leaf's intact draws from its baked wreck draws by centre bits
//! (`destroy_dead` / `destroy_wreck_all`). Some wreck glists carry the intact glist's own sphere,
//! so both baked to the same centre and the intact leaf counted as its own wreck: hidden while
//! alive, dropped when dead. Cases [D]: 0700001b PL#1 `SCRIPT_ControlRoom` (the control-room
//! shell — the arrival cut-scene showed space through its back wall) and 07000003's servants'
//! props #266 tv set, #360 stereo, #364 tv, #376 computer screen. `Destroy_Create` 0x12c318
//! attaches the intact glist and only `Destroy_Smash` 0x12bb60 swaps the wreck in [M].

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn intact_destroy_leaves_never_share_a_wreck_identity() {
    let iso = iso();
    for (lv, pls) in [("0700001b.bin", &[1usize][..]), ("07000003.bin", &[266, 360, 364, 376][..])] {
        let scene = nf_view::load_with_carry_difficulty(&iso, lv, false, None, 3).expect("load");
        let wrecks: Vec<[u32; 3]> = scene.destroy_wrecks.iter().map(|(c, _)| c.map(f32::to_bits)).collect();
        for &pl in pls {
            let (ic, _) = scene.destroy_centres.iter().find(|(_, p)| *p == pl).expect("the destroy leaf is drawn");
            let ib = ic.map(f32::to_bits);
            assert!(!wrecks.contains(&ib), "{lv} PL#{pl}: the intact draw's identity is not a wreck's");
            assert!(scene.destroy_wrecks.iter().any(|(_, p)| *p == pl), "{lv} PL#{pl}: its wreck is still baked");
            let n = scene.materials.iter().filter(|m| m.centre.map(f32::to_bits) == ib).count();
            assert!(n > 0, "{lv} PL#{pl}: the intact leaf has draws");
        }
    }
}
