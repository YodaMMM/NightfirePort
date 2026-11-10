//! A guard that leaves every cel is removed, and the boundary push walks only his own cel's walls.
//!
//! `control_handle_cel_change` 0x133288 runs for every moved dynamic object
//! (`control_movement_object_handler` 0x1336f8). A relink miss (no portal crossing out of the old
//! cel, outside its box, no `build_LinkToRoom` hit) on a type-2 object calls `Drone_FellOutMap`
//! 0x13a878 [M 0x133288:175-181], whose SP body is `NDrone2_Enable(0)` + `obj+0xFE |= 1`, the
//! control walk's delete [M 0x13a878:8-15].
//!
//! `NDrone2_Collision` 0x140e90:169-174 pushes obj+0x30 out of the AI boundary walls on the
//! guard's own cel's list only (`AINetwork_GetBoundsPushVectorForSphere` 0x157bd0:83-133 walks
//! `cel+0x34`; `AIBounds_Prepare` 0x1d3008 files each wall under its two posts' cels).

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_guard_outside_every_cel_is_removed() {
    let mut w = walker::Walker::new(nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001"));
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());
    for _ in 0..30 { w.tick(Default::default(), false); }
    let (i, home) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let i = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.obj_cel.is_some())
            .expect("a placed guard with a linked cel");
        (i, sim.drones.list[i].pos)
    };
    let out = [home[0], home[1] - 400.0, home[2]];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.world.find_cel(out), None, "the probe point {out:?} is in no cel");
        assert!(sim.drones.list.iter().filter(|d| d.present && !d.corpse).count() > 1);
    }
    w.scene.sim.as_mut().unwrap().drones.list[i].pos = out;
    w.tick(Default::default(), false);
    let d = &w.scene.sim.as_ref().unwrap().drones.list[i];
    assert!(!d.present && !d.enabled && d.delete_me,
        "drone {i} moved from {home:?} to {out:?} (no cel) must be removed by Drone_FellOutMap: present {} enabled {} delete_me {}",
        d.present, d.enabled, d.delete_me);
    // Sister: the guards that stayed inside their cels are untouched.
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.list.iter().enumerate().any(|(j, d)| j != i && d.present && !d.delete_me));
}

/// On this level's boundary network, a point 0.2 u from a wall whose posts both sit in other cels
/// than the point's own: the flat every-wall push moves it, the cel-scoped push leaves it.
#[test]
fn the_boundary_push_skips_walls_of_other_cels() {
    let sc = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    let sim = sc.sim.as_ref().unwrap();
    assert!(sim.nav.bounds, "07000001 carries AI boundaries");
    let mut found = 0;
    let mut posts_linked = 0;
    for w in sim.nav.walls.iter().filter(|w| w.flags & 0x1c00 == 0) {
        if w.cels[0].is_some() && w.cels[1].is_some() { posts_linked += 1; }
        let mid = [(w.a[0] + w.b[0]) * 0.5, (w.a[1] + w.b[1]) * 0.5 + 1.0, (w.a[2] + w.b[2]) * 0.5];
        let (dx, dz) = (w.b[0] - w.a[0], w.b[2] - w.a[2]);
        let l = (dx * dx + dz * dz).sqrt();
        if l < 0.5 { continue; }
        for s in [1.0f32, -1.0] {
            let p = [mid[0] - dz / l * 0.2 * s, mid[1], mid[2] + dx / l * 0.2 * s];
            let Some(c) = sim.world.find_cel(p) else { continue };
            if w.cels.contains(&Some(c)) { continue; }
            let flat = sim.nav.push_out(p, 0.4);
            if flat == p { continue; }
            let scoped = sim.nav.push_out_in_cel(p, 0.4, Some(c));
            let own_wall_near = sim.nav.walls.iter().any(|o| o.flags & 0x1c00 == 0 && o.cels.contains(&Some(c))
                && { let q = [o.a, o.b]; q.iter().any(|e| (e[0] - p[0]).powi(2) + (e[2] - p[2]).powi(2) < 4.0) });
            if own_wall_near { continue; }
            assert_eq!(scoped, p, "cel {c}: a wall filed under {:?} must not push {p:?} (flat push gave {flat:?})", w.cels);
            found += 1;
        }
    }
    eprintln!("[bounds] {found} off-cel push points; {posts_linked} walls with both posts linked of {}", sim.nav.walls.len());
    assert!(found > 0, "the level has walls whose flat push reaches into another cel");
}
