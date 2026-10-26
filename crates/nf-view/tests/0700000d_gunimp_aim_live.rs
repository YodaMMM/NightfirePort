//! gunimp-aim-signs: the mounted laser gun's aim follows the game's axes and signs, with the
//! two axes deliberately NOT symmetric.
//!
//! [M `GunImp_Update` 0x1320d8:124-161]: `yaw(ext+0x74) -= Input_Actionf(slot, 3, 1)·FR·0.015`,
//! `pitch(ext+0x70) -= Input_Actionf(slot, 4, 1)·FR·0.015`, pitch clamped to [−π/2, +π/4], both eased
//! toward their smoothed mirrors (ext+0x80/0x84) by 0.15 per tick.
//!
//! The port's inputs: `Input.turn` is a yaw delta with the port's yaw growing LEFT (`player.rs:860`
//! `yaw += turn`), opposite the game's action 3 (raw stick x, right+), so `yaw -= action3` becomes
//! `yaw += turn`. `Input.look` is a pitch delta that is UP-positive (`lib.rs:6067` `look -= dy`, and
//! the invert-look setting is applied upstream at `lib.rs:4823`, so after that point positive always
//! means "look up"), while the gun's `pitch` is the game's own down-positive angle (the mount's view
//! pitch is `−pitch_s·0.5`, `mounted_pose`) — so `pitch -= look` keeps the game's formula. An earlier
//! version flipped both axes; the pitch half of
//! that, and these tests pin both.
//!
//! The tests drive the real path: load 0700000d, press the use key at the deck gun (#225), then tick
//! with the same `Input` a player produces and read the mount's angles and the view pitch the mount
//! writes for the player (up-positive, π/2 units).

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// 0700000d's deck gun (#225 "PowerStationGunImpBase", placement translation (137.27, 9.33, −324.66)
/// [D]). The intro cut-scene runs first; `step_bond` releases Bond after it.
fn mounted() -> Option<nf_view::scene::Scene> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 {
            sim.step_bond(Default::default(), false, false, false);
            if !sim.disabled { break; }
        }
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

fn gun(scene: &nf_view::scene::Scene) -> (f32, f32, f32, f32) {
    let g = scene.sim.as_ref().unwrap().drones.props.gunimps.iter().find(|g| g.active).unwrap();
    (g.yaw, g.pitch, g.yaw_s, g.pitch_s)
}

fn view_pitch(scene: &nf_view::scene::Scene) -> f32 { scene.sim.as_ref().unwrap().bond.pitch }

#[test]
fn a_left_turn_turns_the_gun_left_and_a_look_up_lowers_the_game_pitch() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    let (y0, p0, _, _) = gun(&scene);
    let v0 = view_pitch(&scene);
    {
        let sim = scene.sim.as_mut().unwrap();
        // One tick of "turn left, look up" — the port's positive axes after the invert-look option.
        sim.step_bond(nf_game::player::Input { turn: 0.05, look: 0.05, ..Default::default() }, false, false, false);
    }
    let (y1, p1, ys1, ps1) = gun(&scene);
    let v1 = view_pitch(&scene);
    assert!(y1 > y0,
        "a left turn must turn the gun LEFT (`yaw -= action3` with the port's turn = −action3): yaw {y0} -> {y1}");
    assert!(p1 < p0,
        "looking up must RAISE the aim, and the gun's pitch is the game's down-positive angle: \
         `pitch -= look` [M 0x1320d8:130], so pitch must fall — got {p0} -> {p1}");
    assert!(v1 > v0,
        "the mounted view the player sees must go UP for a look-up (it is −pitch_s·0.5, up-positive \
         in the port): {v0} -> {v1}");
    assert!((ys1 - y0 - 0.15 * (y1 - y0)).abs() < 1e-4,
        "the yaw mirror eases 0.15 of the way per tick: {y0} -> {ys1} with target {y1}");
    assert!((ps1 - p0 - 0.15 * (p1 - p0)).abs() < 1e-4,
        "the pitch mirror eases 0.15 of the way per tick: {p0} -> {ps1} with target {p1}");
}

#[test]
fn the_pitch_clamps_and_the_yaw_does_not() {
    let Some(mut scene) = mounted() else { eprintln!("NF_ISO not set or the mount is unreachable — skipping"); return };
    for _ in 0..400 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { look: 1.0, ..Default::default() }, false, false, false); }
    let (_, p_up, _, _) = gun(&scene);
    assert!((p_up + std::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "a held look-up drives the down-positive pitch to its −π/2 clamp [M 0x1320d8:132-135], got {p_up}");
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { look: -1.0, ..Default::default() }, false, false, false); }
    let (_, p_dn, _, _) = gun(&scene);
    assert!((p_dn - std::f32::consts::FRAC_PI_4).abs() < 1e-4,
        "a held look-down drives it to the +π/4 clamp [M 0x1320d8:132-135], got {p_dn}");
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn: 0.5, ..Default::default() }, false, false, false); }
    let (y, _, _, _) = gun(&scene);
    assert!(y.abs() > std::f32::consts::PI, "the yaw accumulates without a clamp (7.5 turns), got {y}");
}

/// Invert-look both ways, through the value the gun actually receives: `lib.rs:4823` negates `look`
/// when the player's setting is on, so the gun must follow the LOOK (the player's intent), not the raw
/// stick. Feeding +1 then −1 is exactly what the two settings produce for the same stick push.
#[test]
fn the_pitch_follows_the_look_under_both_invert_settings() {
    for (setting, look) in [("invert off (stick up -> look +1)", 1.0f32), ("invert on (stick up -> look -1)", -1.0f32)] {
        let Some(mut scene) = mounted() else { return };
        let (_, p0, _, _) = gun(&scene);
        for _ in 0..30 {
            scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { look, ..Default::default() }, false, false, false);
        }
        let (_, p1, _, _) = gun(&scene);
        // The gun's pitch must move DOWN for a look-up (+look) and UP for a look-down (−look).
        let want_down = look > 0.0;
        assert!((p1 < p0) == want_down,
            "{setting}: the gun's down-positive pitch must {} for look {look}, got {p0} -> {p1}", if want_down { "fall" } else { "rise" });
    }
}
