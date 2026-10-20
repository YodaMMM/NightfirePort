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

/// The straight fire path from the gate-0 start: the rocket's line crosses `DOOR_Shutter_2`'s
/// second glass pane (mat 0x10, x 139.63..140.62 at the closed pose) — the glass rule passes it
/// (no 0x800 in the rocket's +0x6c, no RNG draw) and the round explodes on the missile-cage back
/// wall at z −100.0 [D cage triangles; L the live pass shot]. The same law rocket_glass08_live
/// pins, re-pinned here through this task's own harness.
#[test]
fn the_straight_shot_pierces_the_glass_pane_to_the_back_wall() {
    let Some(iso) = iso() else { return };
    let mut scene = setup(&iso);
    let (_spawn, _last, impacts) = fire(&mut scene);
    let blast = world_impact(&impacts);
    assert!(blast[2] < -99.5 && blast[2] > -100.6, "the pierce ends on the −100 back wall, got z {:.3}", blast[2]);
}

/// A sidestep left puts the rocket's line on the closed shutters' WOOD — `DOOR_Shutter_1`'s right
/// stile abuts `DOOR_Shutter_3`'s left stile into one wood band x 139.44..139.62 at the closed
/// pose [D coll-obj: leaf 1 mat-0x03 stiles x_obj −0.001..0.095 and 1.093..1.186 at t.x 138.346
/// (identity closed quat); leaf 3 mirrored at t.x 140.719 (180° quat)]. The round STOPS —
/// material 3 has no glass, pass or roll bits, so `Bullet_CollisionHandler`'s walk ends at the
/// first hit — and explodes at the hit point, z −96.706 with normal +z (the live −96.71 stop;
/// the flight light rides the rocket's own position one tick behind — the live −96.2 read).
/// The stop branch, pinned at the surface + the material law: the same harness whose straight
/// line pierces (shot 1 below is that measurement), sidestepped onto the closed shutters' WOOD —
/// the world probe names the surface the game's round stops on: mat 0x03 (no glass/pass/roll
/// bits), facing +z, front face z −96.706 [D coll-obj; the closed key-1 pose]. The material law
/// [M Bullet_CollisionHandler 0x12ce80] then ends the walk at the first hit with NO RNG draw —
/// pinned unit-side below — so the round explodes at the hit point (the live −96.71 stop; the
/// flight light rides the rocket's own position one tick behind — the live −96.2 read). The
/// end-to-end steered shot needs a held aim offset this harness's strafe-then-fire does not
/// reproduce deterministically (the aim inertia) — the main agent's live render owns that check.
#[test]
fn the_steer_line_lands_on_the_wood_stile_at_the_games_z() {
    let Some(iso) = iso() else { return };
    let mut scene = setup(&iso);
    // Shot 1 measures THIS harness's flight line at the shutters' plane (the spawn→last segment;
    // the pane it crosses may shatter — the wood stop below is pane-independent).
    let sim = scene.sim.as_mut().unwrap();
    let bx0 = sim.bond.pos[0];
    let (s1, last1, imp1) = fire(&mut scene);
    let blast1 = world_impact(&imp1);
    assert!(blast1[2] < -99.5, "shot 1 rides the straight (glass) line to −100, got z {:.3}", blast1[2]);
    let dir = [last1[0] - s1[0], last1[1] - s1[1], last1[2] - s1[2]];
    let t = (-96.75 - s1[2]) / dir[2];
    let cross1 = s1[0] + dir[0] * t;
    let off = cross1 - bx0; // the muzzle+drift offset of this harness's line
    assert!(off > 0.15 && off < 0.35, "the harness's line offset is the muzzle's, got {off:.3}");
    // Sidestep left until the line sits inside the wood band (139.44..139.62): one strafe tick
    // moves ≤ 0.107, so exiting at line ≤ 139.56 keeps line ≥ 139.453 — always wood.
    let sim = scene.sim.as_mut().unwrap();
    let mut ticks = 0;
    while sim.bond.pos[0] + off > 139.56 && ticks < 40 {
        sim.step_bond(Input { strafe: 1.0, ..Default::default() }, false, false, false);
        ticks += 1;
    }
    assert!(ticks < 40, "Bond sidestepped");
    let line = sim.bond.pos[0] + off;
    assert!(line > 139.44 && line < 139.62, "the line rides the wood band, got x {line:.3}");
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    let a: nf_game::world::V3 = [line, 50.77, -90.0];
    let b: nf_game::world::V3 = [line, 50.77, -110.0];
    let (tt, n, m) = sim.world.segment_surface_bullet(a, b).expect("the closed shutter is solid on the steered line");
    assert_eq!(m, 3, "the steered line lands on the wooden stile, got mat {m}");
    assert!(n[2] > 0.9, "the stile faces +z (down the corridor)");
    let z = -90.0 + (b[2] - a[2]) * tt;
    assert!(z < -96.6 && z > -96.8, "the shutters' closed face at z ≈ −96.706, got {z:.3}");
    // The same probe straight down the corridor line names the GLASS pane (the pierce surface).
    let a: nf_game::world::V3 = [cross1, 50.77, -90.0];
    let (_t2, _n2, m2) = sim.world.segment_surface_bullet(a, b).expect("the pane is solid");
    assert_eq!(m2, 0x10, "the straight line lands on the glass pane, got mat {m2:#x}");
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
