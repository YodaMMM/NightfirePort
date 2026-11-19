use nf_view::scene::Layer;
#[test]
#[ignore]
fn probe14_glass() {
    let iso = std::env::var("NF_ISO").unwrap();
    let scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    for (i, m) in scene.materials.iter().enumerate() {
        let Some(t) = m.key.and_then(|k| scene.textures.iter().find(|t| t.key == k)) else { continue };
        let n = t.rgba.len() / 4;
        let mid = (0..n).filter(|&p| { let a = t.rgba[p * 4 + 3]; a > 8 && a < 240 }).count();
        if mid * 3 > n {
            eprintln!("[glass] #{i} layer {:?} pass {} blend {} zwrite {} atest {} key {:?} centre {:?} r {:.1} late {}",
                m.layer, m.draw.pass, m.draw.blend, m.draw.zwrite, m.draw.atest, m.key, m.centre, m.radius, nf_view::gpu::alpha_shaped(&t.rgba));
        }
    }
}

#[test]
#[ignore]
fn probe14_glass_guards() {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load");
    for _ in 0..240 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    let glass: Vec<[f32; 3]> = scene.materials.iter().filter(|m| m.key == Some((54, 94))).map(|m| m.centre).collect();
    let sim = scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        for g in &glass {
            let dd = ((d.pos[0]-g[0]).powi(2) + (d.pos[2]-g[2]).powi(2)).sqrt();
            if dd < 12.0 { eprintln!("[gg] drone {i} pos {:?} near glass {:?} d {:.1}", d.pos, g, dd); }
        }
    }
}
