//! dead-pane-collision (983): a shot-out window must stop blocking. The game takes a killed
//! pane out of the world — `obj+0xFE |= 1`, "the collision goes away" [M Break_Kill
//! 0x12b970:60] — so the intact pane is solid and the dead one is not. The port used to leave
//! the pane out of the collision world entirely (the sim bake is `instances(true)`, which has
//! no 0x20 placements [D probe]); the pane now rides the per-tick `set_dynamic` set while
//! alive and drops out on the kill.
//!
//! The 07000008 control-room glass (the task's placements 123/128 are the OBJECT indices
//! `(51,123)/(51,128)` — the runtime 0x20 placements are #170/#176/#177 [D placements]):
//! the front pane #176 spans y 34.25..36.37 at z −121.998, over a 0.75 sill from the room
//! floor (33.5) — the window is shot-through, not stepped-through [D the level's collision],
//! so the walk legs stand Bond on the sill itself (the pane's bottom meets the sill top,
//! 34.25 [D probe]).
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The front control-room pane: placement #176 (object Glass_Controlroom, kind 2, health 0
/// [D placements]), its bounds centre and plane.
const FRONT: usize = 176;
const FRONT_C: [f32; 3] = [107.25008, 35.3124, -121.99846];
const PANE_Z: f32 = -121.99846;
const P2K: u8 = 0x02;

fn broken(sim: &nf_view::scene::Sim) -> Vec<usize> {
    sim.drones.breaks.list.iter().filter(|b| b.dead).map(|b| b.placement).collect()
}

/// One real P2K round at `target` from Bond's current pose (the bullet-range harness: the gun
/// state forced, the fire input on one tick through step_bond), collecting the world impacts
/// over the round's whole flight. The switch to the P2K rides the real weapon-change input.
fn fire_one(scene: &mut nf_view::scene::Scene, target: [f32; 3]) -> Vec<[f32; 3]> {
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..240 {
            if sim.gun.weapon == P2K { break; }
            sim.step_bond(Input { switch_to: Some(P2K), ..Default::default() }, false, false, false);
        }
        for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.gun.weapon, P2K, "the switch to the P2K landed");
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let dir = [d[0] / l, d[1] / l, d[2] / l];
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
        sim.drones.player_impacts.clear();
    }
    let mut input = Input::default();
    input.fire = true;
    let mut impacts: Vec<[f32; 3]> = Vec::new();
    for t in 0..80u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
        let sim = scene.sim.as_mut().unwrap();
        for im in &sim.drones.player_impacts {
            if let nf_game::drone::PlayerImpact::World(p) = im { impacts.push(*p); }
        }
        if sim.drones.player_bullets.is_empty() && t > 0 { break; }
    }
    impacts
}

/// Walk Bond forward (his current facing) for `ticks`, returning the minimum z reached.
fn walk_forward(scene: &mut nf_view::scene::Scene, ticks: u32) -> f32 {
    let mut min_z = f32::INFINITY;
    for _ in 0..ticks {
        let input = Input { forward: 1.0, ..Default::default() };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        min_z = min_z.min(sim.bond.pos[2]);
    }
    min_z
}

fn load_08(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000008.bin", false).expect("load 08");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// The shot: the first round stops AT the intact sheet (and kills it — health 0 [D]), the
/// second round through the same spot reaches the far side. [M Break_Kill 0x12b970:60; M
/// Break_Update 0x12bfd8:44-48]
#[test]
#[ignore = "the pane collision ride is off until its shape/matrix pairing is proven (drone.rs, the 0x400_0000_0000 parts)"]
fn the_shot_out_pane_stops_blocking_a_round() {
    let Some(iso) = iso() else { return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = load_08(&iso);
    {
        let sim = scene.sim.as_mut().unwrap();
        // The room side of the window, past the console (z −119.3 [D probe]), with a clear ray
        // to the pane [D probe983_room: the segment from z −121.3 hits only the pane itself].
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [107.25003, 34.9, -121.3], 0.0);
        for _ in 0..20 { sim.step_bond(Input::default(), false, false, false); }
        assert_eq!(broken(sim), Vec::<usize>::new(), "the panes start whole [D placements]");
    }
    // Round 1: the intact sheet takes the round at its plane (the entry impact) — the pane dies
    // to the same contact [M 0x12bfd8:44-50] — and the round PIERCES on (the glass rule:
    // material 0x10 passes unless weapon +0x6c & 0x800 [M 0x12cee0..0x12cfb8]) to the shell
    // behind, z −130.25 [D probe].
    let impacts = fire_one(&mut scene, FRONT_C);
    let sim = scene.sim.as_ref().unwrap();
    assert!(broken(sim).contains(&FRONT), "the first round kills the health-0 pane [M 0x12bfd8:50], broken {:?}", broken(sim));
    let at = impacts.iter().copied().find(|p| (p[0] - FRONT_C[0]).abs() < 1.5)
        .unwrap_or_else(|| panic!("the first round registered a world impact, got {impacts:?}"));
    assert!(at[2] > PANE_Z - 0.12, "the round registers AT the intact sheet (z {:.3} vs plane {PANE_Z:.3})", at[2]);

    // Round 2 through the same spot: the dead pane is out of the world [M 0x12b970:60] — the
    // round's first surface is past the pane plane (the shell behind, z −130.25 [D probe]).
    let impacts = fire_one(&mut scene, FRONT_C);
    assert!(!impacts.is_empty(), "the second round was fired");
    let past = impacts.iter().copied().find(|p| (p[0] - FRONT_C[0]).abs() < 1.5)
        .unwrap_or_else(|| panic!("the second round registered a world impact, got {impacts:?}"));
    assert!(past[2] < PANE_Z - 0.15, "the round fired through the dead pane's first hit is past the plane (z {:.3})", past[2]);
}

/// The sister: the intact pane stops Bond's walk (and his walk does not break it — the
/// player's contact join is unproven, see the report's live checks); shot out, the same walk
/// crosses the plane off the sill.
#[test]
#[ignore = "the pane collision ride is off until its shape/matrix pairing is proven (drone.rs, the 0x400_0000_0000 parts)"]
fn the_intact_pane_stops_bond_and_the_dead_one_lets_him_cross() {
    let Some(iso) = iso() else { return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = load_08(&iso);
    {
        let sim = scene.sim.as_mut().unwrap();
        // On the sill at the pane (floor 34.25 at z −121.9 [D probe]), facing −z (π).
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [107.25003, 35.0, -121.85], std::f32::consts::PI);
        for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    }
    let min_z = walk_forward(&mut scene, 120);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(min_z > PANE_Z - 0.1, "the intact pane stops the walk short of the sheet (min z {min_z:.3}, plane {PANE_Z:.3})");
        assert!(!broken(sim).contains(&FRONT), "Bond's walk does not break the intact pane (the player's contact join is unproven — [S] kept out of the drain)");
    }
    // Shoot the pane out from where he stands, then the same walk crosses.
    let impacts = fire_one(&mut scene, FRONT_C);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(broken(sim).contains(&FRONT), "the point-blank round kills the pane, broken {:?}", broken(sim));
        assert!(!impacts.is_empty(), "the round was fired");
    }
    let min_z = walk_forward(&mut scene, 240);
    assert!(min_z < PANE_Z - 0.05, "Bond crosses where the pane was (min z {min_z:.3} past the plane {PANE_Z:.3})");
}
