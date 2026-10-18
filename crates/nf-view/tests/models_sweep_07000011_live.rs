//! level-11 model sweep (task level-11-interactables): every material on 07000011 resolves a
//! texture — no white fallback — and every placement's world instance resolves its mesh (the
//! loader census prints "0 unresolved" for the level). The GPU's missing-texture path draws a 1×1
//! white; `NF_TEX_TRACE=1` prints the fallbacks at load; this test pins the same condition through
//! the scene's material table (the 0b/08 sweeps' pattern). Opt-in via NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn every_07000011_material_resolves_a_texture_no_white_fallback() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
    let w = &scene;
    let tex_keys: std::collections::HashSet<_> = w.textures.iter().map(|t| t.key).collect();
    let fallbacks: Vec<_> = w.materials.iter().enumerate()
        .filter(|(_, m)| m.key.map_or(true, |k| !tex_keys.contains(&k))).collect();
    eprintln!("[11] materials {} textures {} white-fallback {}", w.materials.len(), w.textures.len(), fallbacks.len());
    for (i, m) in fallbacks.iter().take(16) {
        eprintln!("[11]   fallback mat {i}: layer {:?} pass {:?} key {:?}", m.layer, m.draw.pass, m.key);
    }
    assert!(fallbacks.is_empty(), "every material resolves a texture on 07000011 (no white fallback)");
}

/// The level's 37 guards (0x0F oDrone placements [D census]: office skins 0x05000038/3A + the
/// 0x050000E/F civilians + the stairwell 0x05000004/06/08) exist as spawner templates with their
/// skins; the 7 AI_DroneSpawners hold them absent until their spawn conditions run [M
/// DroneSpawner_Control 0x137e68] — the sweep asserts the TEMPLATES' skins, not the AI timing.
#[test]
fn every_07000011_drone_has_its_skin() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000011.bin", false).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[11] drones {} spawners {} present {}",
        sim.drones.list.len(), sim.drones.spawners.len(),
        sim.drones.list.iter().filter(|d| d.present).count());
    // The census's 37 oDrone placements [D]: 17 of them are the 7 spawners' group templates, held
    // absent until their spawn conditions run [M DroneSpawner_Control 0x137e68] — the sim's list
    // carries the other 20 directly.
    assert_eq!(sim.drones.list.len(), 20, "the 20 direct oDrone templates [D census 37 − the spawners' 17]");
    assert_eq!(sim.drones.spawners.len(), 7, "the seven AI_DroneSpawners exist [D]");
    for (i, d) in sim.drones.list.iter().enumerate() {
        assert!(d.skin != 0, "drone {i} carries a skin hash (kind {})", d.kind);
    }
}
