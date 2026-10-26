//! level-1b-audit: the gated dynamic object (0x3b) rides a posed slot and collapses when the
//! simulation deletes it. `DynamicObject_Update` 0x18d3b8:26 — mode 0 with armed channel B sets
//! obj+0xFE bit 0 and the movement pass's delete (`control_delete_object` 0x133178:16-17) removes
//! the object that tick — so 0700001b's GLASS_ControlRoom (the level's one 0x3b, p1 = ch 10 =
//! Drake's wait gate [D PL#0 census]) must VANISH when ch 10 fires. The old tree drew it as a
//! build-time static instance that never collapses.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere. Fails on the pre-fix tree (the slot does not exist —
//! the palette index reads the collapsed default — and the static glass stays up).
use nf_game::gt::gt_state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    for _ in 0..cap {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
}

fn opts() -> nf_view::scene::ViewOpts { nf_view::scene::ViewOpts::default() }

fn img_read(off: &u64, sz: &u64) -> Option<Vec<u8>> {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).ok()?;
    img.read_bytes(*off, *sz).ok()
}

fn disc_0x3b_count() -> usize {
    let img = match nf_formats::iso9660::IsoImage::open(&iso()) { Ok(i) => i, Err(_) => return 999 };
    let (entries, bin) = match nf_formats::disc::files_bin_entries(&img) { Ok(x) => x, Err(_) => return 999 };
    let fe = match entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700001b.bin")) { Some(f) => f, None => return 999 };
    let data = match img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64) { Ok(d) => d, Err(_) => return 999 };
    match nf_formats::level::Level::parse(&data) {
        Ok(lv) => lv.maps[lv.main_map().unwrap()].placements.iter().filter(|p| p.flags & 0xff == 0x3b).count(),
        Err(_) => 999,
    }
}

#[test]
fn the_1b_control_room_glass_vanishes_when_drakes_gate_fires() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "0700001b.bin", false).expect("load 0700001b");
    settle(&mut scene, 300);
    {
        let s = scene.sim.as_ref().unwrap();
        let d = &s.drones.worldobjs.dynobjs;
        assert_eq!(d.len(), 1, "0700001b's one 0x3b [D the level-1b-audit census]");
        assert_eq!((d[0].ch_a, d[0].ch_b, d[0].mode), (0, 10, 0),
            "GLASS_ControlRoom: no create gate, B = ch 10 (Drake's gate), mode 0 = delete-on-armed-B [D PL#0]");
        assert!(!d[0].dead, "alive before the gate");
    }
    // The posed slot: doors + SSs + breakables + 2/lock + GUNIMP_SLOTS/gunimp + 3/sensor, then
    // one per 0x3b (`lock_slot`'s chain) — the same arithmetic the build used.
    eprintln!("[probe] dynobj_matrices len {} misc0x3b {:?} placements-0x3b {}",
        scene.dynobj_matrices.len(), scene.misc_drawn.get(&0x3b), disc_0x3b_count());
    let sim = scene.sim.as_ref().unwrap();
    let slot = sim.drones.doors.list.len() + sim.drones.sss.list.len() + sim.drones.breaks.list.len()
        + 2 * sim.drones.locks.len()
        + (1 + nf_game::gunimp::TURRET_ARTS.len()) * scene.gunimp_placements
        + 3 * sim.drones.sensors.list.len();
    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
    let m0 = pal[layout.door_base as usize + slot];
    assert!(m0[3][1] > -9000.0, "the glass slot is posed before the gate (got y {})", m0[3][1]);
    let placed = scene.dynobj_matrices[0];
    for r in 0..3 {
        let d: f32 = (0..4).map(|c| (m0[r][c] - placed[r][c]).abs()).fold(0.0, f32::max);
        assert!(d < 1e-3, "the slot rides the placement matrix before the gate [M 0x18dbf8 stores it]");
    }
    let verts: Vec<[f32; 3]> = scene.char_verts.iter().filter(|v| v.door == Some(slot as u16)).map(|v| v.pos).collect();
    assert!(!verts.is_empty(), "the glass slot carries vertices");

    // Fire ch 10 — Drake's wait gate, the same write the game's chain makes (SP 0x060008f0's
    // t104 event [D]; the test drives the channel the way fix_finale_live's chain lands it).
    scene.sim.as_mut().unwrap().drones.set_channel(10);
    settle(&mut scene, 30);
    {
        let s = scene.sim.as_ref().unwrap();
        assert!(s.drones.worldobjs.dynobjs.is_empty(),
            "the sim removed the dead object the tick ch 10 armed B [M 0x18d3b8:26; \
             the port's delete pass retains !dead, the game's control_delete_object 0x133178:16-17 frees it]");
    }
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
    let m1 = pal[layout.door_base as usize + slot];
    assert!(m1[3][1] < -9000.0,
        "the glass slot COLLAPSED after the gate [M 0x18d3b8:26 + control_delete_object 0x133178:16-17; \
         the old tree's static instance stayed at the placement matrix]");
}
