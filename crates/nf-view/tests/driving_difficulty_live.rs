//! The driving missions run at the difficulty the player picked: the GameState+0x28 pick (1..4)
//! rides the PTP word 0x100984 [M Boot_GetPTPData 0x1c8198:13-21], DRIVING main remaps it to
//! 0 / 1 / 2 [M 0x174bf0:70-81] and the mission object keeps that at +0x3b8 [M 0x204de8:67-69].
//! Checked here on MIS4 against the track's own MSet words [D]: the hostile health factor
//! (`FUN_002084a8`, +0x2c4 / +0x2c8 / +0x2cc), the player's start health (`FUN_00206ec0`, the
//! +0x294.. percents) and the snow2a_mis4 `paris_hench` → `junglesnow` swap, which runs only at
//! the remapped word 2 [M 0x12b8b0:96-125]. Opt-in: NF_ISO.

fn mis4_crp(iso_path: &str) -> Vec<u8> {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).unwrap();
    let e = iso.find("/DRIVING/MIS4.VIV").unwrap();
    let g = nf_formats::disc::read_bigf(&iso, e).unwrap();
    let c = g.entries.iter().find(|x| { let n = x.name.to_ascii_lowercase(); n.contains("data\\track\\") && n.ends_with(".crp") }).unwrap();
    let raw = iso.read_bytes(e.lba as u64 * 2048 + c.offset as u64, c.size as u64).unwrap();
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb { nf_formats::refpack::decompress(&raw).unwrap() } else { raw }
}

#[test]
fn driving_difficulty_follows_the_pick_live() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let bytes = mis4_crp(&iso);
    let crp = nf_formats::crp::Crp::parse(&bytes).unwrap();
    // The MSet sits in the Map article's window [M 0x22faf8:108-129].
    let script = nf_formats::trackstart::map_script(&crp).unwrap();
    let mset = script.mset.as_deref().expect("MIS4 MSet");
    let f32_at = |o: usize| f32::from_le_bytes(mset[o..o + 4].try_into().unwrap());
    let base = nf_formats::driveai::MapAi::from_crp(&crp).unwrap();
    let set = nf_game::driving::damage::HealthSet::parse(mset);
    // The track's paris_hench vehicles (6 on MIS4 [D docs/systems/driving-ai.md §13]).
    let hench: Vec<usize> = base.elements.iter().enumerate().filter(|(_, e)| e.kind == 1 && e.name == "paris_hench").map(|(i, _)| i).collect();
    assert!(!hench.is_empty(), "MIS4 has paris_hench vehicles");
    let mut player_health = Vec::new();
    for (pick, mapped) in [(1u32, 0u32), (2, 1), (3, 2), (4, 2)] {
        let (_, d) = nf_view::driving::load_driving_scene_at(&iso, "MIS4", pick).expect("load MIS4");
        assert_eq!(d.difficulty, mapped, "pick {pick}: mission+0x3b8");
        let h = d.hostiles.as_ref().expect("MIS4 hostiles");
        let factor = f32_at(0x2c4 + 4 * mapped as usize);
        // Every vehicle's health is the record's × the MSet factor for this band.
        for (i, e) in h.world.db.elements.iter().enumerate() {
            if e.rec.kind != 1 { continue; }
            let want = base.elements[i].health * factor;
            assert!((e.rec.health - want).abs() < 1e-3, "pick {pick}: element {i} health {} != {want}", e.rec.health);
        }
        assert_eq!(h.world.db.difficulty, mapped, "pick {pick}: the AIEl filter difficulty");
        // The player's start health (`FUN_00206ec0` percents by +0x3b8).
        let want = nf_game::driving::damage::PlayerHealth::start(&set, &nf_game::driving::damage::StartInputs { difficulty: mapped as i32, ..Default::default() });
        let hud = d.hud.as_ref().expect("MIS4 HUD");
        assert_eq!(hud.health.health, want.health, "pick {pick}: start health");
        assert_eq!(hud.health.armour, want.armour, "pick {pick}: start armour");
        player_health.push(hud.health.health);
        // The swap: only at the remapped word 2.
        let swapped: Vec<usize> = h.world.db.elements.iter().enumerate().filter(|(_, e)| e.rec.name == "junglesnow").map(|(i, _)| i).collect();
        if mapped == 2 {
            assert_eq!(swapped, hench, "pick {pick}: every paris_hench record swapped");
            for &i in &hench { assert!(h.world.db.elements[i].rec.riders.is_empty(), "pick {pick}: +0x7c zeroed"); }
            assert!(h.world.attrs.iter().any(|a| a.name == "junglesnow"), "pick {pick}: the junglesnow attribute (MISC.VIV) is in the set");
            assert!(h.pools.iter().any(|p| p.model.eq_ignore_ascii_case("junglesnow") && !p.slots.is_empty()), "pick {pick}: the junglesnow model is loaded");
            assert!(!h.pools.iter().any(|p| p.model.eq_ignore_ascii_case("paris_hench")), "pick {pick}: no paris_hench model left");
        } else {
            assert!(swapped.is_empty(), "pick {pick}: no swap");
            for &i in &hench { assert_eq!(h.world.db.elements[i].rec.name, "paris_hench"); assert_eq!(h.world.db.elements[i].rec.riders.len(), base.elements[i].riders.len()); }
        }
        eprintln!("MIS4 pick {pick}: +0x3b8 {mapped}, factor {factor}, player health {}, swapped {}", hud.health.health, swapped.len());
    }
    // The bands differ on this track [D: MSet health factors 1.0 / 1.2 / 1.4].
    assert!(f32_at(0x2c4) != f32_at(0x2c8) && f32_at(0x2c8) != f32_at(0x2cc), "MIS4's factors differ by band");
    let _ = player_health;
}

#[test]
fn race_has_no_swap_at_the_top_band_live() {
    // snow2a_race carries paris_hench too, but the swap's strcmp wants snow2a_mis4 [M 0x12b8b0:96].
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let (_, d) = nf_view::driving::load_driving_scene_at(&iso, "RACE", 4).expect("load RACE");
    assert_eq!(d.difficulty, 2);
    let h = d.hostiles.as_ref().expect("RACE hostiles");
    assert!(h.world.db.elements.iter().any(|e| e.rec.name == "paris_hench"));
    assert!(!h.world.db.elements.iter().any(|e| e.rec.name == "junglesnow"));
}

#[test]
fn default_loader_keeps_the_old_band_live() {
    // `load_driving_scene` = the boot default pick 2 → +0x3b8 = 1, what the view ran before.
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    let (_, d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    assert_eq!(d.difficulty, 1);
}
