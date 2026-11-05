//! level-1b-platform-touch proof tests: the Drake-fight solar panels' break rule.
//!
//! The game's rule (all cited in docs/reports/level-1b-platform-touch.md):
//!   - damage source: class-5 bullet contacts land in the panel objects' obj+0xD0 HITDATA
//!     chains [M Collide_Update 0x1ea398], summed with mask 0 (no filter [M
//!     Collide_FilterBullets 0x1ece30:24-31]) over the current script's stream objects by
//!     SP_GetHitDamage 0x1dc5f8 + Collide_GetDamageNObjects 0x1ee220;
//!   - threshold: SP+0x14 = placement param 7 = 20 for the panels [D 0700001b placements
//!     #41-56]; SP_Update 0x1dc1c0:236-267 accumulates into SP+0x18 while the current script
//!     is HELD (SCRIPTINFO+0x9c != 0) and unholds at hp <= damage (SP_UnPause + the
//!     remainder kept);
//!   - what counts as a hit: Collide_PickObj 0x1eae80's gates + the glist bounding-sphere
//!     test — fast bullets (step >= radius*0.5) take the 0x201 closest-point-on-segment arm
//!     with the object's own radius [M Bullet_update 0x12e7b0:432-458 arms +0x94 = 0x101/0x201].
//!   - there is NO touch break: the player's body contacts carry no break path [M
//!     Destroy_CollisionHandler 0x12bd48 subtracts only class-0x05 records], and one
//!     20-damage round breaks a panel BY DESIGN [D weapon 0x33 damage 20].
//!
//! Proven here through the real input path from the level start (no teleport): walking onto
//! the panel walkway breaks nothing; firing the start weapon (the game's break action)
//! breaks a panel and plays its debris sequence.

use nf_game::player::Input;

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// (held count, panels with damage > 0, objects spawned by the panel script 0x06000832).
fn panel_state(sim: &nf_view::scene::Sim) -> (usize, usize, usize) {
    let objs = sim.drones.scripts.objects.list.iter().flatten()
        .filter(|o| matches!(o, nf_game::nis::Obj::Entity { src: (h, _), .. } if *h == 0x0600_0832))
        .count();
    let panels = sim.drones.scripts.players.iter().filter(|p| (41..=56).contains(&p.placement));
    let held = panels.clone().filter(|p| p.info[p.cur].as_ref().is_some_and(|s| s.delay != 0)).count();
    let dmg = panels.filter(|p| p.damage != 0.0).count();
    (held, dmg, objs)
}

/// Walks Bond (real inputs, the walker's steering incl. zero-g) onto the panel walkway —
/// panel placement #50 @(16.71,-2.88,4.62), the nearest to the post-arrival spawn — and
/// stands/steps there for `ticks`. Returns the last position.
fn walk_onto_walkway(scene: &mut nf_view::scene::Scene, ticks: u32) -> [f32; 3] {
    let target: [f32; 3] = [16.71, -2.88, 4.62];
    let mut t = 0u32;
    while t < ticks {
        let sim = scene.sim.as_mut().unwrap();
        let here = sim.bond.pos;
        let dist = ((target[0] - here[0]).powi(2) + (target[2] - here[2]).powi(2)).sqrt();
        if dist < 0.8 { break; }
        let (turn, look, fwd) = if let Some((ye, pe)) = walker::zero_g_errs(sim, target) {
            (ye.clamp(-0.08, 0.08), pe.clamp(-0.08, 0.08), if ye.abs() > 1.0 { 0.0 } else { 1.0 })
        } else {
            let want = (target[0] - here[0]).atan2(target[2] - here[2]);
            let mut err = want - sim.bond.yaw;
            while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
            while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
            (err.clamp(-0.08, 0.08), 0.0, if err.abs() > 1.0 { 0.0 } else { 1.0 })
        };
        sim.step_bond(Input { forward: fwd, turn, look, ..Default::default() }, false, false, false);
        t += 1;
        assert!(!scene.sim.as_ref().unwrap().disabled, "Bond went down at walk tick {t}");
    }
    scene.sim.as_ref().unwrap().bond.pos
}

/// Touching a platform by walking on it breaks nothing: 900 ticks of stepping/standing on
/// the walkway leave every panel held, with zero accumulated damage and no debris.
#[test]
fn walking_on_a_platform_breaks_nothing() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "0700001b.bin", false, None, 3).expect("load 1b");
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let (held, dmg, objs) = panel_state(scene.sim.as_ref().unwrap());
    assert_eq!((held, dmg, objs), (16, 0, 16), "16 whole panels held at the arena, no damage yet [D placements #41-56]");
    let pos = walk_onto_walkway(&mut scene, 1200);
    // Stand on it, step back and forth across it — the walkway's own surface, real inputs only.
    for i in 0..900 {
        let input = if i % 60 < 30 { Input { forward: 0.3, ..Default::default() } } else { Input { forward: -0.3, ..Default::default() } };
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let (held, dmg, objs) = panel_state(scene.sim.as_ref().unwrap());
    assert_eq!(held, 16, "every panel still held after walking on the walkway — the game has no touch break [M 0x12bd48: bullets only]");
    assert_eq!(dmg, 0, "no panel accumulated any damage from the walk [M SP_Update 0x1dc1c0:236-267 sums bullet records only]");
    assert_eq!(objs, 16, "no debris fragments spawned (16 = the whole panels only)");
    let _ = pos;
}

/// The game's break action — a bullet contact — breaks a panel: one 20-damage round from the
/// start weapon reaches the threshold [D weapon 0x33 damage 20; M SP_Update 0x1dc1c0:265-267],
/// the SP unholds and the debris sequence plays (fragments spawn over the following ticks).
#[test]
fn a_bullet_contact_breaks_a_panel() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "0700001b.bin", false, None, 3).expect("load 1b");
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let pos = walk_onto_walkway(&mut scene, 1200);
    let _ = pos;
    // Switch to the start weapon 0x33 (51), then fire: the bullet's flight through the
    // walkway panels' pick spheres registers the contact [M Collide_PickObj 0x1eae80].
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(51), ..Default::default() }, false, false, false); }
    let mut broke_at = None;
    for i in 0..600i32 {
        let fire = i >= 30 && i % 24 < 8;
        scene.sim.as_mut().unwrap().step_bond(Input { aiming: true, ..Default::default() }, fire, false, false);
        let (held, _, objs) = panel_state(scene.sim.as_ref().unwrap());
        if held < 16 && objs > 16 { broke_at = Some(i); break; }
    }
    let Some(at) = broke_at else { panic!("no panel broke under sustained fire — the game's break action is a bullet contact [M SP_Update 0x1dc1c0:265-267]") };
    // The break plays the script: fragments spawn over the following seconds [D 0x06000832
    // streams 1-10], the SP stays unheld (the debris is the resumed script's streams).
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let (held, _, objs) = panel_state(scene.sim.as_ref().unwrap());
    assert!(held < 16, "the broken panel stays unheld (its script resumed)");
    assert!(objs > 16, "the debris fragments spawned after the break (got {objs}, broke at fire tick {at})");
}
