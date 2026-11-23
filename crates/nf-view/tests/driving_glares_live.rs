//! The driving view draws the track's placed glares (lamp glows, traffic lights, beacons:
//! nf_game::driving::trackfx, FUN_001f2058) as additive halo + spike sprites on the `flar` sheet.
//! Opt-in: NF_ISO.

#[test]
fn paris_lamps_glow_from_the_flar_sheet() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let tex = d.glares.as_ref().expect("the track's glares").tex;
    assert!(scene.textures[tex].width >= 32);
    for _ in 0..60 { d.step(0.0, 0.0, 0.0, false); }
    let g = d.particle_groups([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let glow = g.iter().find(|x| x.0 == Some(tex)).expect("a glare group");
    assert!(glow.1, "additive");
    assert!(glow.2.len() >= 6 * 10, "{} glare vertices", glow.2.len());
}
