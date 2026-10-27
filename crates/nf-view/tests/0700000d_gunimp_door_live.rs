//! level-0d-gun-door-objective: the mounted laser's beam hit must kill the C5 door.
//!
//! The game's fire tick for the 0x67 laser [M `GunImp_Update` 0x1320d8]:
//! - the round spawn :327-331 (`Bullet_init(weapon_data + 0x67·0x10c, muzzle, dir, 0xffff)`) —
//!   the port flies this (the class-5 contacts drain the Destroy leaf through
//!   `Destroy_CollisionHandler` 0x12bd48:31-35 — health −= HITDATA+8 per contact; 50 a contact
//!   [L ee_start_05 +0x67·0x10c+0x0c], 201 contacts for bigdoor02's 10000 [D p3]);
//! - AND the 1000-unit `Collide_RayIntersect` along the barrel (:372-384): the beam stops at the
//!   hit (:385-387 — the far point becomes the hit), and the 0x67 arm feeds the hit position to
//!   `Explode_Propagate(4.0, 5.0, hit, cel, the imp obj)` (:433-436).
//! `Explode_Propagate` 0x17c518's case 0x21 **instantly smashes** a destroyable the sweep reaches
//! with ≥ 1.0 damage (`Destroy_Smash` 0x12bb60:40 writes `switch_channels[+0x12] = 1`) — the C5
//! death raises ch 0x1f, the objective-6 channel, on the FIRST beam hit on the door. The port
//! treated that sweep as presentation only (`gunimp.rs` "Beam visuals ... are presentation"), so
//! the mount needed the 201-bullet drain where the PS2 kills on the first beam contact. The tests
//! drive the real path: the ST-3 plate (the objective's reveal channel 0x1d [D #378]), the mount,
//! the aim with the same `Input` a player produces, fire.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= 2.0 * std::f32::consts::PI; }
    while a < -std::f32::consts::PI { a += 2.0 * std::f32::consts::PI; }
    a
}

/// Load 0700000d, release Bond from the intro, walk the ST-3 plate #378 (ch 0x1d — objective 6's
/// reveal channel [D #378; M 0x1827d8:55-67]), then mount #225.
fn mounted_after_plate() -> Option<nf_view::scene::Scene> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 {
            sim.step_bond(Default::default(), false, false, false);
            if !sim.disabled { break; }
        }
        // The ST-3 plate #378 (118.48–157.56, 3.74–13.26, −311.35..−306.49 [D]) — ch 0x1d.
        // A walk-in: Bond starts south of it and walks north across it (the walkreal's route).
        sim.bond.pos = [136.5, 4.5, -300.0];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI; // the port's forward (sin, cos) — π faces −z
        for _ in 0..110 {
            sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
            if sim.drones.channels.on(0x1d) { break; }
        }
        assert!(sim.drones.channels.on(0x1d), "the ST-3 plate fires ch 0x1d [D #378]");
        // The mount: the deck gun's placement, the use key until the imp owns the tick.
        sim.bond.pos = [137.27, 9.33, -323.0];
        sim.bond.yaw = std::f32::consts::PI;
        sim.bond.pitch = 0.0;
        for _ in 0..6 {
            sim.step_bond(Default::default(), false, false, true);
            if sim.drones.props.gunimps.iter().any(|g| g.active) { break; }
        }
    }
    let ok = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().any(|g| g.active);
    if !ok { return None; }
    Some(scene)
}

/// The live mount's view of the world: (muzzle, barrel dir, yaw_base + yaw_s, pitch_s).
fn mount(scene: &nf_view::scene::Scene) -> ([f32; 3], [f32; 3], f32, f32) {
    let sim = scene.sim.as_ref().unwrap();
    let g = sim.drones.props.gunimps.iter().find(|g| g.active).unwrap();
    (g.muzzle(), g.barrel_dir(), g.yaw_base + g.yaw_s, g.pitch_s)
}

/// bigdoor02 #96's bounds centre — the C5 leaf the objective names [D placements; props.parse].
fn c5(scene: &nf_view::scene::Scene) -> [f32; 3] {
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.drones.props.destroy.iter().find(|d| d.placement == 96).expect("bigdoor02 #96");
    [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0]
}

/// Drive the mount's aim onto `target` with the player's own Input deltas, then hold fire.
/// Returns the ticks spent and whether the aim converged (the barrel within ~0.02 rad).
fn fire_at(scene: &mut nf_view::scene::Scene, target: [f32; 3], max_ticks: u32, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> (u32, bool) {
    let mut ticks = 0u32;
    let mut converged = false;
    while ticks < max_ticks {
        if done(scene.sim.as_ref().unwrap()) { return (ticks, true); }
        let (muzzle, _dir, yaw_t, pitch_s) = mount(scene);
        let dx = target[0] - muzzle[0];
        let dy = target[1] - muzzle[1];
        let dz = target[2] - muzzle[2];
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
        let want_yaw = dx.atan2(dz);
        // the barrel's forward row is (sy·cp, −sp, cy·cp) [M barrel_dir] — down-positive pitch.
        let want_pitch = (-dy / len).asin();
        let turn = wrap_pi(want_yaw - yaw_t).clamp(-0.08, 0.08);
        let look = (pitch_s - want_pitch).clamp(-0.08, 0.08);
        converged = wrap_pi(want_yaw - yaw_t).abs() < 0.02 && (pitch_s - want_pitch).abs() < 0.02;
        let input = nf_game::player::Input { turn, look, aiming: true, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, converged, false, false);
        ticks += 1;
    }
    (ticks, converged)
}

#[test]
fn the_beam_hit_smashes_the_c5_door_and_raises_ch_1f() {
    let Some(mut scene) = mounted_after_plate() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let target = c5(&scene);
    let d0 = scene.sim.as_ref().unwrap().drones.props.destroy.iter().find(|d| d.placement == 96).cloned().unwrap();
    assert!(!d0.dead && d0.health > 9000.0, "bigdoor02 stands at ~10000 [D p3], got {}", d0.health);
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x1d), "the reveal channel is latched [D #378]");
    assert!(!scene.sim.as_ref().unwrap().drones.channels.on(0x1f), "ch 0x1f off before the fire");

    // The budget is the discriminator: the beam smash kills on the FIRST beam contact (the aim
    // converges inside ~60 ticks, the beam touches the door the fired tick after), while the
    // bullet drain alone needs 201 contact ticks (50 a contact [L] against 10000 [D p3]) —
    // ~260 total. 160 covers the converge plus a handful of fired ticks, and no more.
    let (ticks, hit) = fire_at(&mut scene, target, 160, &|sim| {
        sim.drones.channels.on(0x1f)
            && sim.drones.props.destroy.iter().filter(|d| d.placement == 96).all(|d| d.dead)
    });
    eprintln!("fire ticks {ticks}, aim converged {hit}");
    // The objective's own arms: the enable announce (state 1 → 3 [M 0x1827d8:55-67]) and the
    // completion (state 3 → 4 on the channel [M :78-85]) run on Mission_Update ticks — the kill
    // tick's announce completes the NEXT tick (the existing C5 test steps 2 ticks before the
    // state read).
    for _ in 0..2 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let d1 = sim.drones.props.destroy.iter().find(|d| d.placement == 96).cloned().unwrap();
    eprintln!("bigdoor02 after: dead {} health {}", d1.dead, d1.health);
    assert!(hit, "the mount must reach the aim onto the C5 door within the budget");
    assert!(d1.dead, "the beam's Explode_Propagate smash [M 0x1320d8:433-436 → 0x17c518 case 0x21] \
        kills the door on the first beam hit — the 50-a-contact drain alone needs 201 fired ticks");
    assert!(sim.drones.channels.on(0x1f), "Destroy_Smash writes switch_channels[0x12] = 1 [M 0x12bb60:40]");
    assert_eq!(sim.drones.mission.states[12][2], 4, "objective 6 (ch 0x1f) completes [M 0x1827d8:78-85]");
}

#[test]
fn the_beam_stops_at_the_surface_it_hits() {
    // The game's beam far point is the ray hit [M 0x1320d8:385-387 — the hit replaces the
    // 1000-unit point], not the raw muzzle + dir·1000 the pre-fix port drew.
    let Some(mut scene) = mounted_after_plate() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let target = c5(&scene);
    // The beam lives on the fired ticks (the cooldown ticks clear it — 0700000d_gunimp_beam_
    // lifetime pins that): stop the loop on the first fired tick's beam and read it intact.
    let _ = fire_at(&mut scene, target, 400, &|sim| {
        sim.drones.props.gunimps.iter().any(|g| g.beam.is_some())
    });
    let (muzzle, dir, _, _) = mount(&scene);
    let beam = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().find(|g| g.active)
        .and_then(|g| g.beam);
    let Some((m0, far)) = beam else { panic!("a held trigger leaves the shot's beam up"); };
    assert!((m0[0] - muzzle[0]).abs() < 3.0 && (m0[2] - muzzle[2]).abs() < 3.0, "the beam starts at the muzzle");
    // The raw 1000-unit point, for contrast:
    let raw = [m0[0] + dir[0] * 1000.0, m0[1] + dir[1] * 1000.0, m0[2] + dir[2] * 1000.0];
    let door_z = target[2];
    let d_raw = (raw[2] - door_z).abs();
    let d_far = (far[2] - door_z).abs();
    eprintln!("beam far {far:?} vs raw {raw:?} (door z {door_z})");
    assert!(d_far < 6.0, "the beam must stop at the C5 surface (far z within 6 of the door plane), got {far:?}");
    assert!(d_far < d_raw || d_raw < 6.0, "the raw 1000-unit point overshoots the door by {}", d_raw);
}
