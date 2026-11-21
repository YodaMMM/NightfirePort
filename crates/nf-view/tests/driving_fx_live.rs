//! The driving view's tyre smoke: the land step's per-wheel slip feeds the game's vehicle emitters
//! (nf_game::driving::effects, FUN_001ee4a0: smoke at slip >= 0.2 on a no-spray surface), the pool
//! steps, and the view billboards the quads from the track sheet's `MAIN` atlas. Before, the driving
//! view drew no particles. Opt-in: NF_ISO.

#[test]
fn a_handbrake_turn_on_mis4_smokes_from_the_main_atlas() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let tex = d.fx.as_ref().and_then(|f| f.main_tex).expect("the sav_partic01 MAIN atlas");
    assert_eq!((scene.textures[tex].width, scene.textures[tex].height), (256, 256));
    let quiet = d.particle_groups([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // (the car's specular shine shares the particle groups; only the MAIN-atlas groups are smoke)
    assert!(quiet.iter().filter(|g| g.0 == Some(tex)).all(|g| g.2.is_empty()), "nothing smokes at rest");
    for t in 0..240 { d.step(1.0, 0.0, if t > 100 { 1.0 } else { 0.0 }, t > 120); }
    let g = d.particle_groups([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let smoke: Vec<_> = g.iter().filter(|x| x.0 == Some(tex)).collect();
    let quads: usize = smoke.iter().map(|x| x.2.len() / 6).sum();
    assert!(quads > 0, "the sliding wheels smoke from the MAIN atlas (slip {:?})", d.car.slip);
}
