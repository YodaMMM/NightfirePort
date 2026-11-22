//! rocket-stop-plane-08 (GAPS G-049 follow-up): the 07000008 start-corridor rocket's stop
//! surface at z ≈ −96.7. The game's rocket usually explodes there (normal +z) and sometimes
//! passes to the wall at −100.0. The disc data: the corridor end wall `Room_Outside_1` (z
//! −96.75, mat 0x0a) has a window hole (x 138.35..140.72, y 49.5..51.5, arch to ~52.8) and the
//! hole is filled by the CLOSED SHUTTER DOORS — placements 49–52 (kind 0xdb): `DOOR_Shutter_1`
//! (obj 45, wood, mat 0x03) with `DOOR_Shutter_2` (obj 47, 18 glass tris all mat 0x10) covering
//! x 138.35..139.54, and `DOOR_Shutter_3` (obj 46) + `DOOR_Shutter_2` again covering x
//! 139.53..140.72 [D `nf-extract coll-obj`/`coll-scan`, the placements' 0x19 keys: main-map
//! path_data 4..7, closed = key 1]. The leaves' closed front face sits at z −96.706, the panes
//! at −96.753.
//!
//! The stop-vs-pierce variance is AIM variance across that assembly, not the surface byte's 0x80
//! roll: the pane bytes carry no 0x80 and the wood is mat 0x03 — neither byte draws Rand(0x3f)
//! [D + M `Bullet_CollisionHandler` 0x12ce80: the roll only fires on byte & 0x80]. The pierce is
//! the glass rule (the rockets' `weapon_data+0x6c` = 0x12204 / 0x1a200 carry no 0x800 [L
//! ee_start_05.bin @0x2bf150]) → through the pane to the missile-cage back wall at −100.08 (mat
//! 0x0a) — the live −100.0 pass. The wood stile stops the round: hit z −96.706, normal +z — the
//! live −96.71 stop (the explode position is HITDATA+0x40 [M 0x12ce80's 0x4 arm]; the flight
//! light rides the rocket's own position ≈ 0.55 = one tick's travel behind — the live −96.2).
//!
//! From the level start pose the round stops on the wood: the aim snap takes the closed leaves
//! [L muzzle-live/aim-ray-08-rocket: HIT (139.562, 50.792, -96.706) on the class-0x1c object] and
//! two live shots exploded at z -96.706, x 139.52..139.54 [L live-1108].
//!
//! The port carries the same collision: the doors' dynamic set (world.rs `set_dynamic`, closed
//! key-1 pose at create), shapes under the synthetic 0x8000_0000|map<<16|obj keys — mid-sim
//! segment probes hit the stile (mat 3, z −96.706, normal +z) and the pane (mat 0x10).
//!
//! Opt-in: set `NF_ISO` to the disc image.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

use nf_game::player::Input;

/// Equip the unguided launcher on 07000008's gate-0 start and settle (the real fire path's setup,
/// the same one rocket_glass08_live pins).
fn setup(iso: &str) -> nf_view::scene::Scene {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load_weapons(iso, "07000008.bin", false, &[0x2d]).expect("load 08");
    let sim = scene.sim.as_mut().unwrap();
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x2d).max(1);
    assert!(sim.drones.inventory.equip_weapon(&wd, 0x2d, rounds, true), "equip the unguided launcher");
    sim.drones.inventory.equip_ammo(&wd, 0x2d, rounds * 3);
    sim.gun.switch_request = 0x2d;
    for _ in 0..150 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.gun.weapon, 0x2d, "the launcher is drawn");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// Fire one rocket through the real trigger path; returns (spawn, last live position, impacts).
fn fire(scene: &mut nf_view::scene::Scene) -> ([f32; 3], [f32; 3], Vec<nf_game::drone::PlayerImpact>) {
    let sim = scene.sim.as_mut().unwrap();
    let mut spawn = None;
    for _ in 0..30 {
        let before = sim.drones.player_bullets.len();
        sim.step_bond(Input::default(), true, false, false);
        if sim.drones.player_bullets.len() > before {
            let b = sim.drones.player_bullets.last().unwrap().clone();
            spawn = Some([b.pos[0] - b.dir[0] * b.travelled, b.pos[1] - b.dir[1] * b.travelled, b.pos[2] - b.dir[2] * b.travelled]);
            break;
        }
    }
    let spawn = spawn.expect("a rocket was fired");
    let mut last = spawn;
    for _ in 0..140 {
        if sim.drones.player_bullets.iter().all(|b| b.weapon != 0x2d) { break; }
        sim.step_bond(Input::default(), false, false, false);
        if let Some(b) = sim.drones.player_bullets.iter().find(|b| b.weapon == 0x2d) { last = b.pos; }
    }
    assert!(sim.drones.player_bullets.iter().all(|b| b.weapon != 0x2d), "the rocket exploded within 140 ticks");
    let impacts = sim.drones.player_impacts.clone();
    (spawn, last, impacts)
}

fn world_impact(impacts: &[nf_game::drone::PlayerImpact]) -> [f32; 3] {
    impacts.iter().find_map(|i| match i {
        nf_game::drone::PlayerImpact::World(p) => Some(*p),
        _ => None,
    }).expect("the rocket's world impact was recorded")
}

/// The straight fire path from the gate-0 start: the round's line rides the wood band where the
/// closed leaves meet, material 0x03 stops it (no glass, pass or roll bit [M 0x12ce80]) and the
/// rocket explodes on the shutters' face, z -96.706 [L live-1108: from this pose two shots
/// exploded at x 139.538 / 139.523, z -96.706, normal (0, 0, 1)]. The old door pass (every door
/// passed rounds) carried this shot through to the -100 wall.
#[test]
fn the_straight_shot_stops_on_the_shutter_wood() {
    let Some(iso) = iso() else { return };
    let mut scene = setup(&iso);
    let (_spawn, _last, impacts) = fire(&mut scene);
    let blast = world_impact(&impacts);
    assert!((blast[2] + 96.706).abs() < 0.03, "the round stops on the shutters' face z -96.706, got z {:.3}", blast[2]);
    assert!(blast[0] > 139.44 && blast[0] < 139.62, "on the wood band x 139.44..139.62, got x {:.3}", blast[0]);
}

/// The aim snap from the start pose takes the closed shutters: the game's crosshair ray (head
/// (139.531, 50.792, -76.431) to the 256-out point (139.924, 50.792, -332.431)) first hits the
/// class-0x1c door object at (139.562, 50.792, -96.706) [L audits muzzle-live/aim-ray-08-rocket:
/// the Collide_RayIntersect HIT read at Player_WeaponInitBullet 0x1a4bf0]. The leaves' glist word
/// carries no 0x60, so `Collide_PickObj` 0x1eae80's mask (0x20 | 0x40 for the 0x20a flags word
/// [M disasm 0x1a4d68-0x1a4d88]) keeps them, and the leaf's own posed collision triangles give
/// the hit; the wood (0x03) is no glass, so the 0x20a pass-glass bit does not skip it. The old
/// placement-kind word (0xdb, bit 0x40) skipped every door here.
#[test]
fn the_aim_snap_lands_on_the_shutter_wood_at_the_games_point() {
    let Some(iso) = iso() else { return };
    let scene = setup(&iso);
    let sim = scene.sim.as_ref().unwrap();
    let d = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let head = [139.531f32, 50.792, -76.431];
    let raw = [139.924f32, 50.792, -332.431];
    // 0x2d's +0x6C = 0x1a200 [L ee_start_05.bin @0x2bf150]: no 0x800, the ray passes glass.
    let (pt, hits) = sim.drones.aim_snap_traced(&d, head, raw, true);
    assert_eq!(hits.last().map(|h| h.stage), Some("door"), "the closed shutters take the snap, stops {hits:?}");
    let live = [139.562f32, 50.792, -96.706];
    let miss = ((pt[0] - live[0]).powi(2) + (pt[1] - live[1]).powi(2) + (pt[2] - live[2]).powi(2)).sqrt();
    assert!(miss < 0.01, "the snap lands at the game's point {live:?}, got {pt:?} (off {miss:.4})");
    let a = [pt[0], pt[1], pt[2] + 1.0];
    let b = [pt[0], pt[1], pt[2] - 1.0];
    let (_, n, m) = sim.world.segment_surface_bullet(a, b).expect("the shutter is solid at the snap");
    assert_eq!(m, 3, "the snapped surface is the wood stile, got mat {m:#x}");
    assert!(n[2] > 0.9, "the stile faces +z (down the corridor)");
}

/// The material law on this window's two surfaces [M 0x12ce80:136-159]: the rocket's +0x6c
/// (0x1a200 [L ee_start_05.bin @0x2bf150]) carries no 0x800, so the pane byte 0x10 [D coll-obj]
/// passes by the glass rule with NO RNG draw, and the stile byte 0x03 stops with NO draw — the
/// surface byte's 0x80 roll NEVER fires on this window (the pane bytes carry no 0x80 [D]), so
/// the stop-vs-pierce variance is aim variance, not the seeded roll.
#[test]
fn the_window_surfaces_decide_without_the_roll() {
    let mut rng = nf_game::drone::Rng::new(0x1234);
    let d0 = rng.draws;
    assert!(nf_game::effect::penetration_roll_node(&mut rng, 0x10, 0x2d, false, 0x1_a200),
        "the pane passes by the glass rule");
    assert_eq!(rng.draws, d0, "the glass rule draws no RNG");
    assert!(!nf_game::effect::penetration_roll_node(&mut rng, 0x03, 0x2d, false, 0x1_a200),
        "the wooden stile stops the round");
    assert_eq!(rng.draws, d0, "the stile draws no RNG either");
}

/// Sister: Bond cannot walk through the closed shutters (they are doors — the game's actor
/// queries test the cel-linked objects' collision [M Collide_PickObj 0x1eae80]; the port's
/// capsule_push tests the dynamic set). Walking straight at the window from the start, Bond's z
/// stays outside the panes' plane.
#[test]
fn bond_cannot_walk_through_the_closed_shutters() {
    let Some(iso) = iso() else { return };
    let mut scene = setup(&iso);
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..400 {
        sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let z = sim.bond.pos[2];
    assert!(z > -96.3, "Bond is held outside the closed shutters (z {:.3} must stay > −96.3)", z);
}
