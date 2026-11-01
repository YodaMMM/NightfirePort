//! The 0xFE mine's bob: the disc census across every level, and the drawn bob.
//!
//! The bob: `Mine_Update` 0x17c930 writes `obj+0x34 = inst+4 +
//! sin(GameState+0x52 · inst+0x14) · inst+0x18` [M disasm 0x17c9a8-0x17c9f4], where `Mine_Create`
//! 0x17ce28 fills inst+0x14 = param3·0.0002 and inst+0x18 = param4·0.1 [M 0x17ce28:45-47]. The
//! drawn pose is obj+0x90, rebuilt every tick by `Control_BuildWorldSph` 0x133070:17 →
//! `Control_GetObjMatrix` 0x134210:10-12 from obj+0x50 (rot) + obj+0x30 (pos), and
//! `View_RotTransMatrix` 0x1e5168:46-48 writes obj+0x30..0x38 into the drawn matrix's translation;
//! `View_DrawObjectGlist` 0x1e6780:44 draws the glist at obj+0x90. So the drawn mine rides the
//! bobbed y. (The game also accumulates `obj+0x54 += param2·0.0002·FRAME_RATE_MUL` [M 0x17c978-
//! 0x17c988], the spin angle — not modelled by the port, see the report.)
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_formats::{disc, iso9660::IsoImage, level::Level};

/// Every parsed level archive: (name, maps). A `.bin` entry that fails to parse is reported and
/// skipped (the census is over level archives).
fn levels<'a>(iso: &'a IsoImage) -> Vec<(String, Level)> {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let mut out = Vec::new();
    for e in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = iso
            .read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64)
            .unwrap_or_else(|e| panic!("{}: read: {e}", e));
        match Level::parse(&data) {
            Ok(level) => out.push((e.name.clone(), level)),
            Err(err) => eprintln!("[census] {} does not parse as a level ({err}) — skipped", e.name),
        }
    }
    out
}

/// The disc census: every level archive, every map, every 0xFE placement, with its params 0..5.
/// Pins the whole census [D]: which levels carry mines, and the bob parameters of each.
#[test]
fn the_disc_census_of_0xfe_mines() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let mut mines: Vec<(String, usize, [u32; 6], [f32; 3])> = Vec::new();
    for (name, level) in levels(&iso) {
        for (mi, m) in level.maps.iter().enumerate() {
            for p in m.placements.iter().filter(|p| p.flags & 0xff == 0xfe) {
                let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0u32, |&(_, v)| v);
                let params = [param(0), param(1), param(2), param(3), param(4), param(5)];
                eprintln!("[census] {} map {mi} placement #{} at {:?} params {:?}", name, p.index, p.translation, params);
                mines.push((name.clone(), mi, params, p.translation));
            }
        }
    }
    // The census [D]: 07000015's nine tripbombs (docs/levels/07000015.md) and 07000041's single
    // Laser_Mine #582 (docs/levels/07000041-audit.md) are the disc's only 0xFE placements.
    assert_eq!(mines.len(), 10, "the disc's 0xFE placements: 9 on 07000015 + 1 on 07000041 [D]");
    let on_15 = mines.iter().filter(|(n, ..)| n == "07000015.bin").count();
    let on_41 = mines.iter().filter(|(n, ..)| n == "07000041.bin").count();
    assert_eq!(on_15, 9, "07000015 carries the nine tripbombs [D docs/levels/07000015.md]");
    assert_eq!(on_41, 1, "07000041 carries the one Laser_Mine [D docs/levels/07000041-audit.md]");
    // 07000015's nine: bob off (p3 = the sin rate, p4 = the amplitude — both 0 [D]).
    for (name, _, params, _) in mines.iter().filter(|(n, ..)| n == "07000015.bin") {
        assert_eq!((params[3], params[4]), (0, 0), "{name}: the bob is off [D]");
    }
    // 07000041's Laser_Mine #582: p0 = 0x32 (damage), p1 = 0xa (radius/shake/scale), p2 = 0x32
    // (the unmodelled spin rate), p3 = 0x32 (the bob rate), p4 = 0xa (the bob amplitude), p5 =
    // 0x6000052 (the blast script) [D docs/levels/07000041-audit.md line 455].
    let (_, _, params, at) = mines.iter().find(|(n, ..)| n == "07000041.bin").unwrap();
    assert_eq!(params, &[0x32, 0xa, 0x32, 0x32, 0xa, 0x0600_0052], "07000041 #582's params [D]");
    assert!((at[0] - 52.40).abs() < 0.01 && (at[1] - -8.52).abs() < 0.01 && (at[2] - -41.43).abs() < 0.01,
        "07000041 #582 sits at 52.40,-8.52,-41.43 [D]: {:?}", at);
}

/// The 07000041 Laser_Mine's drawn slot follows the bob: `posed_y = pos.y +
/// sin(tick·param3·0.0002)·param4·0.1` [M 0x17c930 disasm 0x17c9a8-0x17c9f4; params 0x32/0xa
/// [D]], the slot's matrix translation carrying it the way `View_RotTransMatrix` 0x1e5168:46-48
/// writes obj+0x30..0x38 into the drawn pose. The static slot matrix stays at the placement.
#[test]
fn the_07000041_laser_mine_bobs_in_its_draw_slot() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let mut scene = nf_view::load(&iso, "07000041.bin", false).unwrap();
    let (mx_static, base) = {
        let s = scene.sim.as_ref().unwrap();
        let mn = s.drones.mines.first().expect("07000041 carries one mine [D]");
        // The [D] parameters through the port's fields: rate = 0x32·0.0002, amp = 0xa·0.1.
        assert!((mn.bob_rate - 0x32 as f32 * 0.0002).abs() < 1e-6, "the bob rate from p3 [D]: {}", mn.bob_rate);
        assert!((mn.bob_amp - 0xa as f32 * 0.1).abs() < 1e-6, "the bob amplitude from p4 [D]: {}", mn.bob_amp);
        let mx_static = *scene.mine_placements.iter().find(|(pl, _)| *pl == mn.placement)
            .expect("the mine has a draw slot").1;
        (mx_static, mn.pos[1])
    };
    // The slot: the door-region slot drawn at the mine's x/z (and, before any tick, its y —
    // `posed_y` starts at pos.y, sin(0) = 0). The static slot matrix must stay at the placement.
    let (slot, _) = {
        let s = scene.sim.as_ref().unwrap();
        let mut found = None;
        for k in 0..8192usize {
            if let Some(m) = scene.door_slot_matrix(s, k) {
                if (m[3][0] - mx_static[3][0]).abs() < 1e-3 && (m[3][2] - mx_static[3][2]).abs() < 1e-3
                    && (m[3][1] - base).abs() < 1e-3 {
                    found = Some((k, m));
                    break;
                }
            }
        }
        found.expect("the 07000041 mine has a posed draw slot at its placement")
    };
    assert_eq!(slot[3][1], mx_static[3][1], "before any tick the slot sits at the placement y");
    // Tick 1..260: the slot's y tracks the [M] formula over the sin's rise to its peak (the rate
    // 0.01 rad/tick puts the first quarter period at tick 157, so the amplitude 1.0 shows).
    let mut max_y = f32::MIN;
    for k in 1..=260usize {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let mn = &s.drones.mines[0];
        let want = base + (k as f32 * mn.bob_rate).sin() * mn.bob_amp;
        let m = scene.door_slot_matrix(s, slot).unwrap();
        assert!((m[3][1] - want).abs() < 1e-4, "tick {k}: slot y {} != posed_y {want} [M 0x17c930]", m[3][1]);
        assert_eq!(mn.posed_y, want, "tick {k}: the sim's posed_y is the [M] formula");
        max_y = max_y.max(m[3][1]);
    }
    // The amplitude shows: the sin(rate·k) rise (rate 0.01 rad/tick) peaks at tick 157, so the
    // window rides ≈1.0 (param4·0.1 [D]) above the base at the peak.
    assert!(max_y > base + 0.99, "the amplitude shows: the slot rides ≈1.0 above the base, max {max_y}");
    assert_eq!(mx_static[3][1], scene.mine_placements[0].1[3][1], "the static slot matrix keeps the placement y");
}

/// Sister: 07000015's nine tripbombs carry p3 = p4 = 0 [D the census], so `posed_y` stays at
/// pos.y — the drawn slots never move.
#[test]
fn the_07000015_mines_stay_put_in_their_draw_slots() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let mut scene = nf_view::load(&iso, "07000015.bin", false).unwrap();
    let s = scene.sim.as_ref().unwrap();
    assert_eq!(s.drones.mines.len(), 9, "07000015 carries the nine [D]");
    let mut slots: Vec<(usize, [f32; 3])> = Vec::new();
    for mn in &s.drones.mines {
        let mx_static = scene.mine_placements.iter().find(|(pl, _)| *pl == mn.placement).expect("draw slot").1;
        let mut found = None;
        for k in 0..8192usize {
            if let Some(m) = scene.door_slot_matrix(s, k) {
                if (m[3][0] - mx_static[3][0]).abs() < 1e-3 && (m[3][1] - mx_static[3][1]).abs() < 1e-3
                    && (m[3][2] - mx_static[3][2]).abs() < 1e-3 {
                    found = Some(k);
                    break;
                }
            }
        }
        let Some(slot) = found else { panic!("mine placement {} has no posed draw slot", mn.placement) };
        slots.push((slot, mx_static[3]));
    }
    for k in 1..=120usize {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        for (slot, at) in &slots {
            let m = scene.door_slot_matrix(s, *slot).expect("the mines stay unexploded at the start");
            assert!((m[3][1] - at[1]).abs() < 1e-4, "tick {k}: mine slot y {} moved off the placement y {} [D bob off]", m[3][1], at[1]);
        }
    }
}
