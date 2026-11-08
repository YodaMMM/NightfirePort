//! The 0700001b solar panels take a round on their triangles, not their bounding sphere: the
//! sphere only queues the object [M Collide_PickObj 0x1eae80] and `Collide_Update` 0x1ea398 drops
//! it unless `Collide_Intersect` 0x1ebaa8 (the flight's 0x201 pick into `Intersect_RayGeom`
//! 0x10a8c8) meets the glist's triangles. The panel's sphere is 4.77 [D glist 0x02000a42], so
//! the walkway above it sits inside the sphere of the panel under Bond's feet and the next one.
//! NF_ISO-gated.

use nf_game::player::Input;

fn broken(sim: &nf_view::scene::Sim) -> Vec<u32> {
    sim.drones.scripts.players.iter().filter(|p| (41..=56).contains(&p.placement))
        .filter(|p| !p.info[p.cur].as_ref().is_some_and(|s| s.delay != 0)).map(|p| p.placement as u32).collect()
}

/// Bond at the walkway spot; the load's float faces -x, `about_face` turns the zero-g body round.
fn at_the_walkway(iso: &str, about_face: bool) -> nf_view::scene::Scene {
    let mut scene = nf_view::load_with_carry_difficulty(iso, "0700001b.bin", false, None, 1).unwrap();
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [16.74, -1.84, 4.62];
    if about_face {
        let m = &mut sim.bond.zero_g_rot;
        for k in [0, 2] { for c in 0..3 { m[k][c] = -m[k][c]; } }
    }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    assert!(broken(sim).is_empty());
    scene
}

/// Facing down the slope the row falls away from the round for the first panels, so a tap
/// leaves #50 under Bond's feet whole; #51-56 stay below the rising line too.
#[test]
fn firing_down_the_row_breaks_no_panel() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = at_the_walkway(&iso, true);
    let sim = scene.sim.as_mut().unwrap();
    for i in 0..120 { sim.step_bond(Input::default(), i < 2, false, false); }
    assert!(broken(sim).is_empty(), "{:?}", broken(sim));
}

/// Facing up the slope the panel under Bond's feet (#50) and the one ahead keep their place while
/// the round flies 1.6 above them inside their spheres; the row's rising surface only meets the
/// round at the far end of #49, x 7.4 [D glist 0x02000a42 at (10.14, -1.72, 4.62)], on the tick
/// the flight reaches it.
#[test]
fn the_round_breaks_only_the_panel_it_crosses() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = at_the_walkway(&iso, false);
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input::default(), true, false, false);
    assert!(broken(sim).is_empty(), "the first tick spans x 16.2 to 13.5 over #50: {:?}", broken(sim));
    sim.step_bond(Input::default(), true, false, false);
    assert!(broken(sim).is_empty(), "the second tick ends at x 9.2 over #49: {:?}", broken(sim));
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(broken(sim), vec![49], "the third tick crosses #49's surface and stops there");
}

/// The astronauts' rounds go through the same Collide pass, so they chip the panels too: the
/// round's 5.0 [D weapon 0x6a] sums to the panel's 20 hit points on the fourth, and only on
/// the panel it crosses [M SP_GetHitDamage 0x1dc5f8 reads every class-5 contact].
#[test]
fn enemy_rounds_wear_a_panel_down() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = at_the_walkway(&iso, false);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [30.0, 6.0, 0.0];
    let round = |sim: &mut nf_view::scene::Sim| {
        sim.drones.bullets.push(nf_game::bullet::Bullet {
            pos: [16.7, 0.0, 4.6], dir: [0.0, -1.0, 0.0], speed: 4.25, travelled: 0.0, range: 200.0,
            damage: 5.0, owner: 0, state: 0, weapon: 0x6a, flyby: false, bounces: 0, ..nf_game::bullet::Bullet::TAIL
        });
        for _ in 0..8 { sim.step_bond(Input::default(), false, false, false); }
    };
    for _ in 0..3 { round(sim); }
    assert!(broken(sim).is_empty(), "15 of 20: {:?}", broken(sim));
    round(sim);
    assert_eq!(broken(sim), vec![50], "the fourth round reaches the hit points");
}

/// The missile state machine clocks its scripts from tables: every channel write in them is
/// the disc's, and only the disc's. The disarm scripts' later write is the Apocalypse latch
/// 0x83..0x8a; ch 89 and 90 belong to the AND triggers #150 and #149 alone [D].
#[test]
fn missile_tables_match_the_disc() {
    use nf_game::space_missile::{P1_WRITES, P2_WRITES};
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 1).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let writes = |h: u32| {
        let mut v: Vec<(u32, u16)> = nf_game::nis::channel_writes(&sim.drones.scripts.lib[&h]).iter().map(|&(t, c, _)| (t, c as u16)).collect();
        v.sort();
        v
    };
    for (i, h) in [0x0600091cu32, 0x06000920, 0x06000921, 0x06000922, 0x0600091b, 0x06000923, 0x06000924, 0x06000925].into_iter().enumerate() {
        assert_eq!(writes(h), P2_WRITES[i].to_vec(), "missile {} disarm script {h:#x}", i + 1);
    }
    for h in [0x060007c6u32, 0x060007e1] { assert_eq!(writes(h), P1_WRITES.to_vec(), "launch script {h:#x}"); }
}

/// `Pickup_Update` falls by `WldGravity`, and the station zeroes it [M 0x18e148:47-55, Player_Init
/// 0x1910f8:152-172]: an astronaut's dropped gun (`Pickup_CreateSimple`, weapon 0x6a taken as 0x33
/// [M 0x18f348]) floats where it was dropped instead of sinking out of the station.
#[test]
fn a_dropped_gun_floats_in_zero_g() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, 1).unwrap();
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.world.gravity_y(), 0.0, "the station's start has param 3 == 2 [D]");
    let at = [30.0f32, 6.0, 0.0];
    let m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [at[0], at[1], at[2], 1.0]];
    let p = nf_game::pickup::Pickup::create_simple(&m, 0x6a, 12, 0, &sim.weapons, &sim.models).expect("the drop");
    let y0 = p.m[3][1];
    sim.drones.pickups.push(p);
    for _ in 0..240 { sim.step_bond(Input::default(), false, false, false); }
    let p = sim.drones.pickups.last().unwrap();
    assert!((p.m[3][1] - y0).abs() < 1e-3, "the drop stays put: y {y0} -> {}", p.m[3][1]);
}
