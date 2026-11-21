//! The driving view runs the mission's enemy vehicles (nf_game::driving::hostiles::HostileWorld):
//! built on load from the track's AIEl set, stepped every tick against the player car, drawn from
//! their own car files on palette slots of their model, removed from the draw when they despawn.
//! Opt-in: NF_ISO.

#[test]
fn driving_hostiles_view_live() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set, skipping"); return };
    for viv in ["MIS01", "MIS4"] {
        let (scene, mut d) = nf_view::driving::load_driving_scene(&iso, viv).expect("load");
        let h = d.hostiles.as_ref().expect("the track has AI vehicles");
        eprintln!("{viv}: {} elements, pools {:?}", h.world.db.elements.len(), h.pools.iter().map(|p| (p.model.as_str(), p.slots.len())).collect::<Vec<_>>());
        let mut seen = std::collections::BTreeMap::<usize, ([f32; 3], [f32; 3], String)>::new();
        let mut removed = 0usize;
        let mut prev: Vec<usize> = Vec::new();
        for t in 0..3600 {
            let throttle = if t % 600 < 450 { 0.7 } else { 0.0 };
            d.step(throttle, 0.0, 0.0, false);
            let draws = d.hostile_draws();
            // Every live element with a slot is in the palette at its own matrix.
            let mut pal = vec![[[1.0f32; 4]; 4]; scene.attachments.len()];
            d.apply_to_palette(0, &mut pal);
            for (dr, slot) in &draws {
                let pool = d.hostiles.as_ref().unwrap().pools.iter().find(|p| p.slots.contains(slot)).expect("slot in a pool");
                assert!(pool.model.eq_ignore_ascii_case(&dr.model), "{viv}: element {} draws on a {} slot", dr.element, pool.model);
                assert!(scene.char_verts.iter().any(|v| v.attachment == Some(*slot)), "{viv}: slot {slot} has the model's vertices");
                let m = pal[*slot as usize];
                assert!((m[3][0] - dr.matrix[3][0]).abs() < 1e-3 && (m[3][2] - dr.matrix[3][2]).abs() < 1e-3, "{viv}: slot {slot} at the element's position");
                let p = [dr.matrix[3][0], dr.matrix[3][1], dr.matrix[3][2]];
                seen.entry(dr.element).and_modify(|e| e.1 = p).or_insert((p, p, dr.model.clone()));
            }
            let now: Vec<usize> = draws.iter().map(|(dr, _)| dr.element).collect();
            for e in &prev {
                if !now.contains(e) {
                    removed += 1;
                    // A removed element's slot folds away (no live element holds it).
                    let h = d.hostiles.as_ref().unwrap();
                    assert!(!h.world.live.iter().any(|v| v.element == *e), "{viv}: element {e} still live but not drawn");
                }
            }
            for p in d.hostiles.as_ref().unwrap().pools.iter() {
                for s in &p.slots {
                    if !draws.iter().any(|(_, t)| t == s) {
                        assert_eq!(pal[*s as usize], [[0.0; 4]; 4], "{viv}: free slot {s} folds away");
                    }
                }
            }
            prev = now;
            if t % 600 == 599 {
                let h = d.hostiles.as_ref().unwrap();
                eprintln!(
                    "{viv} t{t}: car {:?}, live {} drawn {} undrawn {}, script AI {:?}, hits so far {:.0} hp, health {:?}, state {:?}",
                    d.car.pos.map(|x| x.round()),
                    h.world.live.len(),
                    draws.len(),
                    h.undrawn,
                    h.applied,
                    h.damage_dealt,
                    d.hud.as_ref().map(|x| x.health.health),
                    d.mission_state
                );
            }
        }
        let moved = seen.values().filter(|(a, b, _)| ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() > 5.0).count();
        eprintln!("{viv}: {} drawn elements ({:?}), {moved} moved, {removed} removed", seen.len(), seen.iter().map(|(e, v)| (*e, v.2.as_str())).collect::<Vec<_>>());
        let h = d.hostiles.as_ref().unwrap();
        // MIS01: the four elements that start unheld (persistent) spawn at once [D AIEl flags];
        // two of them drive. MIS4: the script's EActivateAIElement releases the two snowmobiles,
        // they chase and shoot (supersnow.atr declares guns [D]), the hits take health off the
        // player car, and they are removed once the player is past their range.
        assert!(!seen.is_empty(), "{viv}: hostiles spawned and drew");
        assert!(moved > 0, "{viv}: a hostile moved");
        assert!(seen.values().all(|v| h.pools.iter().any(|p| p.model.eq_ignore_ascii_case(&v.2))), "{viv}: each drew with its own model");
        if viv == "MIS4" {
            assert!(h.applied[0] > 0, "{viv}: the script activated elements");
            assert!(removed > 0, "{viv}: a removed hostile stopped drawing");
            let hp = d.hud.as_ref().unwrap().health;
            assert!(h.damage_dealt > 0.0 && hp.health < hp.health_max, "{viv}: hostile hits took health: {} of {}", hp.health, hp.health_max);
        }
    }
}
