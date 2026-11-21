//! The player car's specular pass (bondrm.o MatCar* pass 2, docs/formats/car-model.md §10): the
//! Specular-pass strips draw additively over the base pass with ST = 0.5 + SpecularSpot·(n·W·basis).xy
//! on `carS` (32x32, data\render\common.ssh); the map coordinates follow the view, so turning the
//! camera moves the highlight. Opt-in: NF_ISO.

#[test]
fn the_vanquish_shine_samples_cars_and_follows_the_view() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let sh = d.shine.as_ref().expect("the specular pass");
    let tex = sh.tex.expect("carS");
    assert_eq!((scene.textures[tex].width, scene.textures[tex].height), (32, 32));
    assert!(!sh.strips.is_empty());
    let a = d.particle_groups([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let g = a.iter().find(|g| g.0 == Some(tex)).expect("the shine group");
    assert!(g.1, "additive");
    assert!(g.2.len() > 300, "{} shine vertices", g.2.len());
    let b = d.particle_groups([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let g2 = b.iter().find(|g| g.0 == Some(tex)).unwrap();
    let moved = g.2.iter().zip(&g2.2).filter(|(p, q)| (p.uv[0] - q.uv[0]).abs() + (p.uv[1] - q.uv[1]).abs() > 1e-3).count();
    assert!(moved * 2 > g.2.len(), "turning the view moves the highlight ({moved} of {})", g.2.len());
}
