//! The driving view keeps vertex colours on the PS2 scale (0x80 = 1.0; the shader rescales, scene.rs
//! module doc). The car's base pass sends RGBAQ = min(255, 127·L) [bondrm.o, docs/formats/car-model.md
//! §10]: the Vanquish's paint facing up under paris's World light is L ≈ 1.17..1.20, so 149..153 (other CarRender classes reach ~170). The
//! view doubled it (and the track's per-part colours) on top of the shader, which washed the car out
//! to white and lit the night track twice as bright. Opt-in: NF_ISO.

#[test]
fn the_vanquish_paint_facing_up_is_127_times_its_light() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, d) = nf_view::driving::load_driving_scene(&iso, "MIS01").expect("load MIS01");
    // The Vanquish's own draws: its body slot and its posed parts (the traffic and the mission's
    // AI vehicles draw on other slots with their own CarRender classes).
    let mut mine: Vec<u32> = d.player_attach_slot.into_iter().collect();
    if let Some(r) = &d.car_rig { mine.extend(r.parts.iter().map(|p| p.1)); }
    let slot_of = |vi: usize| vi.checked_sub(scene.char_vertex_start).and_then(|k| scene.char_verts.get(k)).and_then(|c| c.attachment);
    let car: Vec<_> = scene.materials.iter()
        .filter(|m| m.layer == nf_view::scene::Layer::Character && m.key.is_some())
        .filter(|m| m.index_count > 0 && slot_of(scene.indices[m.first_index as usize] as usize).is_some_and(|s| mine.contains(&s)))
        .collect();
    assert!(!car.is_empty(), "the car's textured draws");
    let mut up = Vec::new();
    for m in car {
        for k in 0..m.index_count as usize {
            let v = scene.vertices[scene.indices[m.first_index as usize + k] as usize];
            if v.nrm[1] > 125 { up.push(v.rgba); }
        }
    }
    assert!(!up.is_empty(), "upward-facing car vertices");
    let in_band = up.iter().filter(|c| (120..=200).contains(&c[0])).count();
    assert!(in_band * 2 > up.len(), "most upward car vertices sit at 127·L (paint 149..153, other classes up to ~170), well under the doubled 255 (got {:?}…)", &up[..up.len().min(8)]);
}
