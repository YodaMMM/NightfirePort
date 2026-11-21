//! The driving HUD: hudNTSC.gal's sheets load as 256x256 atlases (each sheet also holds 1x1
//! entries; the element UVs index the big one), and at a mission's start the ported passes
//! (nf_game::driving_hud_draw, FUN_0023da30) show the health iris for 3 s with the mission's own
//! health (nf_game::driving::damage). The view draws its sprites as overlay quads. Opt-in: NF_ISO.

#[test]
fn the_health_iris_shows_at_the_start_of_mis4() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let h = d.hud.as_ref().expect("the HUD gallery");
    assert!(!h.sheets.is_empty());
    for (_, hash) in &h.sheets {
        let t = &scene.hud_textures.iter().find(|(k, _)| k == hash).unwrap().1;
        assert_eq!((t.width, t.height), (256, 256), "the sheet's atlas, not its 1x1 entries");
    }
    let (hp, hm, _, _) = h.health.hud();
    assert!(hp > 0.0 && hp == hm, "the mission's start health {hp}/{hm}");
    for _ in 0..60 { d.step(0.0, 0.0, 0.0, false); }
    let f = d.hud_frame(&scene, 16.0 / 9.0);
    assert!(f.runs.len() >= 7, "the iris segments draw at the start: {} runs", f.runs.len());
    for _ in 0..400 { d.step(0.0, 0.0, 0.0, false); }
    let _ = d.hud_frame(&scene, 16.0 / 9.0);
    let later = d.hud_frame(&scene, 16.0 / 9.0);
    assert!(later.runs.len() < f.runs.len(), "the meter hides after its 3 s show");
}
