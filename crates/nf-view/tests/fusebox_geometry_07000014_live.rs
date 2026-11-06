//! 07000014 fusebox: the laser must cut the VISIBLE cable run, which sits 1.5 m above the wire
//! entity's translation.
//!
//! [D] the box's wires are `OBJECT_GreenWire1` (hash 0x02000aa1) and `OBJECT_BlueWire1` (0x02000aa4);
//! their script entity matrix translation is (-89.3055, -2.2213, 30.787) while the wire geometry's local
//! y 1.49..1.72 puts the visible cables at world y -0.73..-0.50 — 1.5 m ABOVE the entity origin.
//! [M] the game collides the round against the wire OBJECT's geometry (`Collide_PickObj` 0x1eae80 →
//! obj+0xD0 → `FuseBox_Update` 0x1adfe8:88-123); the port used a 0.5-radius sphere at the entity
//! translation, so a beam on the cables never touched anything.
//!
//! The approach is the player's: the room's doorway leaf (placement 384, `OBJECT_Door`) stands
//! between the shooter's stance and the wire, and a shot at the CLOSED leaf dies on it (the
//! game's own built sphere r 0.4937 catches the beam — sqDist 0.0147 < r² 0.2437 [M
//! `Collide_PickObj` 0x1eae80's sphere arm; the disc word 0x0 carries no pass bits [D
//! glist_flags_probe]]). The player opens the leaf with the E press through the real
//! `Player_Activate` → `Door_Activate` path [M 0x1a5170 → 0x135be0] and the open leaf's posed
//! triangles stand out of the beam — the way the player's cut works. The leaf's auto-close
//! holds 360 quiet ticks [M 0x13660c-0x136700]; the cut's rounds leave inside that window.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn wrap(mut a: f32) -> f32 {
    let t = std::f32::consts::PI * 2.0;
    while a > std::f32::consts::PI { a -= t; }
    while a < -std::f32::consts::PI { a += t; }
    a
}

/// Steer Bond toward `to` on the ground plane (the walker's yaw convention) holding `fwd`;
/// returns the horizontal distance remaining.
fn walk_step(sim: &mut nf_view::scene::Sim, to: [f32; 3], fwd: f32) -> f32 {
    let b = &sim.bond;
    let d = [to[0] - b.pos[0], to[2] - b.pos[2]];
    let dist = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let want = d[0].atan2(d[1]);
    let ye = wrap(want - b.yaw).clamp(-0.08, 0.08);
    sim.step_bond(nf_game::player::Input { turn: ye, forward: fwd, ..Default::default() }, false, false, false);
    dist
}

/// Aim the view at a point (the walker's convention).
fn aim_at(sim: &mut nf_view::scene::Sim, to: [f32; 3]) {
    let head = sim.bond.head();
    let d = [to[0] - head[0], to[1] - head[1], to[2] - head[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).atan() / std::f32::consts::FRAC_PI_2;
}

#[test]
fn the_laser_cuts_the_visible_cables() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    // the reported spot; the aim target is the wire slab's POSED centre (the wire entity's gfx
    // bounds through the live matrix — the visible cable run), computed below like
    // interactables_07000014's wire_aim_point. The beam converges on the aim ray's snap point
    // (the muzzle-to-snap line), so the crosshair on the slab centre puts the beam on the slab.
    let spot = [-89.71f32, -1.10, 32.49];
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); }
    let target = {
        let sim = scene.sim.as_ref().unwrap();
        let obj = sim.drones.fuseboxes.iter()
            .find(|f| (f.pos[0] + 89.31).abs() < 2.0)
            .and_then(|f| f.alive_wires().next().map(|(_, _, o)| o))
            .expect("the box's wire object");
        match sim.drones.scripts.objects.list.get(obj) {
            Some(Some(nf_game::nis::Obj::Entity { hash, matrix, .. })) => {
                let (lo, hi) = sim.drones.wire_geo.get(hash).expect("the wire's gfx bounds [D]");
                let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
                for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                    let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                    let w = nf_formats::placement::transform(&matrix, p);
                    for c in 0..3 { wlo[c] = wlo[c].min(w[c]); whi[c] = whi[c].max(w[c]); }
                } } }
                [(wlo[0] + whi[0]) * 0.5, (wlo[1] + whi[1]) * 0.5, (wlo[2] + whi[2]) * 0.5]
            }
            _ => panic!("the wire entity exists"),
        }
    };
    eprintln!("[014-fusebox] the wire slab's posed centre {target:?}");
    // The near leaf: placement 384 (`OBJECT_Door`), the doorway between the stance and the wire.
    let di = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.doors.list.iter().position(|d| d.placement == 384)
            .expect("the near leaf (placement 384) exists [D placement]")
    };
    let wires0 = {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..120 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        // The door stance is the model test's (the same 'OBJECT_Door' record): dpos + (0, 0.6, 1.6),
        // yaw π, the aim at dpos + (0, 1.0, 0) — the E opens it there (interactables_07000014's
        // a_plain_door_opens_on_use green). The transit to the room is the documented stand-in;
        // the approach to the stance and the walk out to the reported spot are the player's.
        {
            let dpos = sim.drones.doors.list[di].pos;
            sim.bond.pos = [dpos[0], dpos[1] + 0.6, dpos[2] + 1.6];
            sim.bond.vel = [0.0; 3];
            sim.bond.yaw = std::f32::consts::PI;
        }
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        let before = sim.drones.doors.list[di].state;
        {
            let dpos = sim.drones.doors.list[di].pos;
            aim_at(sim, [dpos[0], dpos[1] + 1.0, dpos[2]]);
        }
        let mut opened = false;
        for k in 0..240 {
            let st = sim.drones.doors.list[di].state;
            if st & 0xc != 0 || st != before { opened = true; break; }
            sim.step_bond(nf_game::player::Input::default(), false, false, k % 6 == 0);
        }
        eprintln!("[014-fusebox] leaf opened {opened} state {:#x}", sim.drones.doors.list[di].state);
        assert!(opened, "the E press at the leaf opens it [M Player_Activate 0x1a5170 -> Door_Activate 0x135be0]");
        // Walk out to the reported stance (the x/z the pin gives; the room floor holds Bond) and
        // bring the laser up inside the leaf's 360-tick open window [M 0x13660c-0x136700].
        for _ in 0..300 {
            let d = walk_step(sim, spot, 1.0);
            if d < 0.3 { break; }
        }
        sim.bond.pos[0] = spot[0];
        sim.bond.pos[2] = spot[2];
        sim.bond.vel = [0.0; 3];
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false, false, false);
        for _ in 0..200 { sim.step_bond(nf_game::player::Input::default(), false, false, false); if sim.gun.weapon == 0x4e { break; } }
        eprintln!("[014-fusebox] at the stance {:?} leaf {:#x}", sim.bond.pos, sim.drones.doors.list[di].state);
        let fb = sim.drones.fuseboxes.iter().find(|f| (f.pos[0] + 89.31).abs() < 2.0).expect("the box");
        fb.alive_wires().count()
    };
    assert!(wires0 > 0, "the box's script registered wires at load");
    // The cut: steer the crosshair at the visible cable and hold fire — the round's miss
    // against the wire's world slab shifts the aim (the interactables_07000014 pattern).
    let mut delta = [0.0f32; 3];
    let start_frame = scene.sim.as_ref().unwrap().drones.frame;
    loop {
        let wires_now = scene.sim.as_ref().unwrap().drones.fuseboxes.iter()
            .find(|f| (f.pos[0] + 89.31).abs() < 2.0).map(|f| f.alive_wires().count())
            .unwrap_or(0);
        if wires_now < wires0 { break; }
        assert!(scene.sim.as_ref().unwrap().drones.frame - start_frame < 900, "the laser cut never dropped a wire");
        // Aim at the slab centre + the accumulated miss delta, hold fire one tick.
        let (aim_t, n0) = {
            let sim = scene.sim.as_ref().unwrap();
            let mut t = target;
            for k in 0..3 { t[k] += delta[k]; }
            (t, sim.drones.player_bullets.len())
        };
        {
            let sim = scene.sim.as_mut().unwrap();
            let b = &sim.bond;
            let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
            let d = [aim_t[0] - eye[0], aim_t[1] - eye[1], aim_t[2] - eye[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
            let yaw = d[0].atan2(d[2]);
            let pitch = (d[1] / len).asin() / std::f32::consts::FRAC_PI_2;
            let ye = wrap(yaw - b.yaw).clamp(-0.08, 0.08);
            let pe = (pitch - b.pitch).clamp(-0.08, 0.08);
            sim.step_bond(nf_game::player::Input { turn: ye, look: pe, aiming: true, ..Default::default() }, true, false, false);
        }
        // The newest round's miss against the slab centre shifts the aim.
        let miss = {
            let sim = scene.sim.as_ref().unwrap();
            let round = sim.drones.player_bullets.get(n0).map(|b| (b.pos, b.dir, b.speed));
            let obj = sim.drones.fuseboxes.iter()
                .find(|f| (f.pos[0] + 89.31).abs() < 2.0)
                .and_then(|f| f.alive_wires().next().map(|(_, _, o)| o));
            let ent = obj.and_then(|o| sim.drones.scripts.objects.list.get(o).and_then(|x| x.clone()));
            match (round, ent) {
                (Some((pos, dir, speed)), Some(nf_game::nis::Obj::Entity { hash, matrix, .. })) => {
                    match sim.drones.wire_geo.get(&hash) {
                        Some((lo, hi)) => {
                            let (mut wlo, mut whi) = ([f32::MAX; 3], [f32::MIN; 3]);
                            for cx in 0..2 { for cy in 0..2 { for cz in 0..2 {
                                let p = [if cx == 0 { lo[0] } else { hi[0] }, if cy == 0 { lo[1] } else { hi[1] }, if cz == 0 { lo[2] } else { hi[2] }];
                                let w = nf_formats::placement::transform(&matrix, p);
                                for c in 0..3 { wlo[c] = wlo[c].min(w[c]); whi[c] = whi[c].max(w[c]); }
                            } } }
                            let c = [(wlo[0] + whi[0]) * 0.5, (wlo[1] + whi[1]) * 0.5, (wlo[2] + whi[2]) * 0.5];
                            let e = [pos[0] + dir[0] * speed, pos[1] + dir[1] * speed, pos[2] + dir[2] * speed];
                            let ab = [e[0] - pos[0], e[1] - pos[1], e[2] - pos[2]];
                            let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                            let tt = if ab2 > 1e-9 { (((c[0] - pos[0]) * ab[0] + (c[1] - pos[1]) * ab[1] + (c[2] - pos[2]) * ab[2]) / ab2).clamp(0.0, 1.0) } else { 0.0 };
                            let closest = [pos[0] + ab[0] * tt, pos[1] + ab[1] * tt, pos[2] + ab[2] * tt];
                            let mut d3 = [0.0f32; 3];
                            for k in 0..3 { d3[k] = (c[k] - closest[k]).clamp(-0.8, 0.8); }
                            Some(d3)
                        }
                        None => None,
                    }
                }
                _ => None,
            }
        };
        if let Some(d3) = miss { for k in 0..3 { delta[k] = (delta[k] + d3[k]).clamp(-0.8, 0.8); } }
    }
    let sim = scene.sim.as_ref().unwrap();
    {
        let d = &sim.drones.doors.list[di];
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        if let Some(tris) = sim.data().world.shape(d.glist) {
            for t in tris { for p in t {
                let w = nf_formats::placement::transform(&d.matrix, *p);
                for c in 0..3 { lo[c] = lo[c].min(w[c]); hi[c] = hi[c].max(w[c]); }
            } }
        }
        eprintln!("[014-fusebox] leaf posed bounds at fire end lo {lo:?} hi {hi:?}");
        for (k, (a, b)) in sim.drones.bullet_paths().0.iter().take(4).enumerate() {
            eprintln!("[014-fusebox] round {k} seg {a:?} -> {b:?}");
        }
    }
    let fb = sim.drones.fuseboxes.iter().find(|f| (f.pos[0] + 89.31).abs() < 2.0).unwrap();
    let wires1 = fb.alive_wires().count();
    assert!(wires1 < wires0,
        "the held laser aimed at the visible cable (world y -0.73..-0.50, 1.5 m above the wire entity's \
         translation) must cut: the contact is the wire object's geometry [M Collide_PickObj 0x1eae80], \
         not the 0.5 sphere at the entity origin; alive wires {wires0} -> {wires1}");
}
