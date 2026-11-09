//! A debris piece draws with the white tweak, not character 0's ambient.
//!
//! Every debris piece (the impact `Debris_Create` 0x1aaa00, a killed breakable's pieces, the
//! blast's `Debris_CreateEx` 0x1ac808) is a `control_create_object` object: its tint bytes
//! obj+0x103..0x105 stay at `control_init_object`'s 0xFF [M 0x134630:18-21], and its obj+0xF0 is
//! the mesh's glist flags | 0x20000020 [M 0x1ac808, 0x1aaa00]. No debris mesh carries the 0x800
//! glist bit [D: checked over every level archive], so `View_SetupRenderModes_Tweak` passes the
//! 0xFF bytes [M 0x1e5888:46-49]: the piece shows its mesh's own vertex colour (the metal strips'
//! light grey). The port's colour pass fell through to character 0's multiplier — in the 07000011
//! storeroom the cel ambient (0x2a, 0x23, 0x22) — and drew the strips near black.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn debris_pieces_take_the_white_tweak_not_an_ambient_dim() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    // In front of the storeroom's steel shelf, where an MGL 'Grenade' throws its metal strips.
    let pos = [11.11f32, 49.9, 19.6];
    {
        let d = &mut scene.sim.as_mut().unwrap().drones;
        for &model in nf_game::explode::METAL_HASH_132.iter() {
            d.blast_debris.push(nf_game::shatter::Debris {
                model, pos, rot: [0.0; 3], vel: [0.0; 3], spin: [0.0; 3],
                life: 50, fade: 0xff, e8: 1.0, draw: Default::default(),
            });
        }
    }
    let s = scene.sim.as_ref().unwrap();
    let amb = s.ambient_at(pos).expect("the storeroom cel");
    assert!(amb.iter().all(|&b| b < 0x80), "the storeroom is a dim cel (the case that hid the bug), got {amb:?}");
    let (mults, _pk, attach) = scene.colour_mults_dense();
    assert!(mults.first().is_some_and(|m| m.iter().all(|&v| v < 0.5)), "character 0's multiplier is the dim one, got {:?}", mults.first());
    let mut seen = 0;
    for (k, a) in scene.attachments.iter().enumerate() {
        let Some((2, _, mesh)) = a.effect else { continue };
        seen += 1;
        assert_eq!(attach[k], [1.0; 3],
            "debris slot {k} (mesh {mesh:#010x}) draws with tweak 0xFF [M 0x134630:18-21, 0x1e5888:46-49]");
    }
    assert!(seen > 0, "07000011 builds debris slots");
    // The same arm through the GPU palette's colour sink (the path the window draws with).
    let (layout, _attrs) = scene.skin_layout();
    let (_pal, cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    for (k, a) in scene.attachments.iter().enumerate() {
        let Some((2, _, mesh)) = a.effect else { continue };
        let c = cols[layout.col_att as usize + k];
        assert_eq!([c[0], c[1], c[2]], [1.0; 3], "palette colour of debris slot {k} (mesh {mesh:#010x}) is the white tweak");
    }
}
