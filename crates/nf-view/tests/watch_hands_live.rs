//! watch-hands (draw-handoffs row 11): `Player_SetWatch` 0x1a69d0 [M :79-137] puts the hour /
//! minute / second hand meshes 0x2000876/877/878 on the watch arm's (fp skin 0x0500004B) datum 2.
//! The port builds one attachment per hand on that view model. Sister: no other view model
//! carries a hand. Opt-in via NF_ISO.

#[test]
fn the_watch_arm_carries_three_hands_on_datum_2() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    // One view model per watch gadget record (0x38/0x4E/0x4F/0x5A/0x5C share the arm [D]); each
    // carries the three hands, and only the held one draws.
    let watch_vms: Vec<usize> = (0..scene.characters.len()).filter(|&c| scene.characters[c].skin.hashcode == 0x0500_004b).collect();
    assert!(!watch_vms.is_empty(), "the level has the watch arm");
    for &c in &watch_vms {
        let mut kinds: Vec<u8> = scene.attachments.iter().filter(|a| a.character as usize == c).filter_map(|a| a.watch_hand).collect();
        kinds.sort();
        assert_eq!(kinds, vec![0, 1, 2], "character {c}: hours, minutes, seconds [M 0x1a69d0:91-102]");
    }
    // Sister: no hand rides any other character.
    assert!(scene.attachments.iter().filter(|a| a.watch_hand.is_some()).all(|a| watch_vms.contains(&(a.character as usize))));
}
