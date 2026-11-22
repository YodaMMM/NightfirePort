//! rocket-hit-size: the guided rocket's ride probe. While Bond rides the AT-420's 'Guided' round
//! (record 0x2C, `+0x6C & 4` [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C: +0x6C = 0x12204]),
//! `Bullet_update` case 1 runs, right after the look-input steer, a SPHERE pick of radius 0.1 at
//! the round (`Collide_SphereIntersect(0.1, obj+0xC0, cel, bullet, owner-or-none, &list, 0, 0,
//! 0xa0a, 4)`) and any hit calls `Bullet_handle_object_destruction` — the round explodes where it
//! is [M Bullet_update 0x12e7b0:218-234; disasm 0x12ec1c-0x12ec50; Bullet_handle_object_destruction
//! 0x12cd98: +0x68 & 0x4000 and +0x70 & 0x80204 hold for 0x2C (0x4000 / 0x4) → Explode_Create].
//! So a ridden rocket that grazes a solid face within 0.1 dies on it even when its 1-unit flight
//! segment (the plain ray every round tests [M 0x12e7b0 tail, mode 0x201]) runs clear past.
//!
//! The rest of the round's world contact is the game's: the flight test is a zero-radius segment
//! from the round's own position along its direction for max(step, 1) [M 0x12e7b0 tail], and the
//! 'Guided' round starts at the head (its +0x68 = 0x4000 lacks the 0x10000 midpoint bit
//! [M Player_WeaponInitBullet 0x1a4bf0:216-240; L the record]).
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const GUIDED: u16 = 0x2c;

/// 07000008's gate-0 start with the guided launcher drawn, one round fired and riding (its spawn
/// tick done, so the next update is case 1).
fn launched(iso: &str) -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load_weapons(iso, "07000008.bin", false, &[GUIDED as u8]).expect("load 08");
    let sim = scene.sim.as_mut().unwrap();
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(GUIDED as usize).max(1);
    assert!(sim.drones.inventory.equip_weapon(&wd, GUIDED as usize, rounds, true), "equip the guided launcher");
    sim.drones.inventory.equip_ammo(&wd, GUIDED as usize, rounds * 3);
    sim.gun.switch_request = GUIDED as u8;
    for _ in 0..150 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.gun.weapon, GUIDED as u8, "the launcher is drawn");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    let mut fired = false;
    for _ in 0..30 {
        sim.step_bond(Input::default(), true, false, false);
        if sim.drones.player_bullets.iter().any(|b| b.weapon == GUIDED && !b.dead) { fired = true; break; }
    }
    assert!(fired, "a guided round was fired");
    sim.step_bond(Input::default(), false, false, false);
    let b = sim.drones.player_bullets.iter().find(|b| b.weapon == GUIDED && !b.dead).expect("the round flies");
    assert!(b.guided_cam && !b.first, "the round rides the camera past its spawn tick");
    scene
}

fn live(sim: &nf_view::scene::Sim) -> Option<usize> {
    sim.drones.player_bullets.iter().position(|b| b.weapon == GUIDED && !b.dead)
}

/// A solid, upright face near the round: (the face point, its normal, a flight direction along
/// it) such that 0.05 in front of the face the 1.5-unit flight segment both ways is clear air, the
/// 0.1 probe touches the face, and 0.3 in front nothing is within the probe's reach.
fn graze_face(sim: &nf_view::scene::Sim, from: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let w = &sim.world;
    let at = |hit: [f32; 3], n: [f32; 3], g: f32| [hit[0] + n[0] * g, hit[1] + n[1] * g, hit[2] + n[2] * g];
    for k in 0..32 {
        let a = k as f32 * std::f32::consts::TAU / 32.0;
        let to = [from[0] + a.sin() * 30.0, from[1], from[2] + a.cos() * 30.0];
        let Some((t, n, mat)) = w.segment_surface(from, to, true) else { continue };
        if mat & 0x3f == 0x10 || mat & 0xc0 != 0 || n[1].abs() > 0.2 { continue; }
        let hit = [from[0] + (to[0] - from[0]) * t, from[1], from[2] + (to[2] - from[2]) * t];
        let l = (n[0] * n[0] + n[2] * n[2]).sqrt();
        if l < 1e-3 { continue; }
        let d = [n[2] / l, 0.0, -n[0] / l];
        let near = at(hit, n, 0.05);
        let clear = |p: [f32; 3], s: f32| w.segment_flight_scoped(w.find_cel(p), p, [p[0] + d[0] * s, p[1], p[2] + d[2] * s], true).is_none();
        if !clear(near, 1.5) || !clear(near, -1.5) { continue; }
        if !w.sphere_hits(near, 0.1, 0xa0a) { continue; }
        let far = at(hit, n, 0.3);
        if w.sphere_hits(far, 0.1, 0xa0a) || !clear(far, 1.5) { continue; }
        return (hit, n, d);
    }
    panic!("no solid upright face with a clear grazing lane near {from:?}");
}

/// Put the live round at `gap` in front of the face, flying along it, and run one tick.
fn ride_one_tick(gap: f32) -> (bool, [f32; 3]) {
    let iso = iso().unwrap();
    let mut scene = launched(&iso);
    let sim = scene.sim.as_mut().unwrap();
    let i = live(sim).unwrap();
    let (hit, n, d) = graze_face(sim, sim.drones.player_bullets[i].pos);
    let p = [hit[0] + n[0] * gap, hit[1] + n[1] * gap, hit[2] + n[2] * gap];
    { let b = &mut sim.drones.player_bullets[i]; b.pos = p; b.dir = d; }
    sim.step_bond(Input::default(), false, false, false);
    (live(sim).is_some(), p)
}

/// The round grazing a solid wall 0.05 in front of its face, flying ALONG it: the flight segment
/// is clear, the 0.1 probe touches the face, and the round explodes on that tick.
#[test]
fn a_ridden_rocket_grazing_a_wall_explodes_on_the_ride_probe() {
    if iso().is_none() { return }
    let (alive, p) = ride_one_tick(0.05);
    assert!(!alive, "the grazing round explodes on the 0.1 ride probe at {p:?}");
}

/// The same lane 0.3 out from the face: outside the probe's 0.1 reach and off the segment's
/// line, the round flies on.
#[test]
fn a_ridden_rocket_clear_of_the_wall_flies_on() {
    if iso().is_none() { return }
    let (alive, p) = ride_one_tick(0.3);
    assert!(alive, "the round 0.3 from the face flies on at {p:?}");
}
