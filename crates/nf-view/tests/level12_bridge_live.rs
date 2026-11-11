//! The 07000012 sky bridge under the copter, on the real level.
//!
//! The scene [D script 0x06000530, SP #660 p5 = 2, p7 = 120; copter #707 gate 0x25]: the touches
//! #670/#781 at the bridge's two ends release the copter, it spots Bond through the bridge's glass,
//! its rounds strike the bridge's script objects, and that starts the break script, which holds at
//! t1, t74, t99 and t134 until 120 points of damage land on each hold [M SP_Update 0x1dbda8 tail].
//!
//! Opt-in: set `NF_ISO`.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const MISSILE: u16 = 0x6d;
const DECK: [f32; 3] = [0.0, 1.1, 35.0];

fn step(sim: &mut Sim) { sim.step_bond(Input::default(), false, false, false); }

/// Bond on the deck with the copter's gate open, the guards out of the picture.
fn deck_scene(iso: &str, release: bool) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000012.bin", false).expect("load 07000012");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..200 { step(sim); }
    sim.bond.pos = DECK;
    for d in sim.drones.list.iter_mut().skip(1) { d.present = false; }
    if release { sim.drones.channels.set(0x25, 1, sim.drones.frame); }
    scene
}

fn bridge(sim: &Sim) -> &nf_game::nis::Player {
    sim.drones.scripts.players.iter().find(|p| p.placement == 660).expect("the bridge SP #660 [D]")
}

fn round(pos: [f32; 3], dir: [f32; 3], speed: f32, range: f32, damage: f32) -> nf_game::drone::PlayerBullet {
    nf_game::drone::PlayerBullet {
        pos, dir, speed, travelled: 0.0, range, damage, weapon: 2,
        first: false, dead: false, bounces: 0, homing: false, target: None, target_copter: false, from_turret: None,
        gravity: false, guided_cam: false, snd: 0, swoosh: None, born: 0,
    }
}

/// `Collide_LineOfSight`'s flags 0x2e drop the class-0x10 glass [M 0x17dbb8:171-172, 0x1ebaa8:315-317]:
/// the copter's spot check sees Bond through the panes and it goes to the attack state.
#[test]
fn the_copter_spots_bond_through_the_bridge_glass() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = deck_scene(&iso, true);
    let sim = scene.sim.as_mut().unwrap();
    let mut attacked = None;
    for t in 0..1200 {
        step(sim);
        sim.drones.health.health = 100.0;
        if sim.drones.copters[0].state == 2 { attacked = Some(t); break; }
    }
    assert!(attacked.is_some(), "the copter never spotted Bond on the deck");
}

/// Its fire reaches him through the glass and strikes the bridge, which starts the break script.
#[test]
fn the_copter_fire_starts_the_bridge_break_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = deck_scene(&iso, true);
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(bridge(sim).state, 1, "primed [M 0x1dbc08]");
    // With the route step at count/scale (agent-copter-rate, [M 0x17d2b8:9-16]) the first
    // attack pass's rounds strike the bridge (state 2 at t≈526) before the second pass reaches
    // Bond through the glass (t≈657), so the walk waits for both rather than stopping on the first.
    let mut hurt = false;
    for _ in 0..1500 {
        step(sim);
        hurt |= sim.drones.health.health < 100.0;
        sim.drones.health.health = 100.0;
        if hurt && bridge(sim).state == 2 { break; }
    }
    assert!(hurt, "no copter round reached Bond behind the glass");
    assert_eq!(bridge(sim).state, 2, "the copter's rounds never started the break script");
}

/// `Bullet_init` 0x12c5a0:187-191 quarters a `+0x6C & 0x200` round, `Bullet_DoTrails` ramps it
/// back up, and the `+0x70 & 4` round's stop is an explosion [M 0x12ce80:193-199].
#[test]
fn the_copter_missile_ramps_up_and_explodes_where_it_stops() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = deck_scene(&iso, true);
    let sim = scene.sim.as_mut().unwrap();
    let (mut slowest, mut fastest, mut blast) = (f32::MAX, 0.0f32, None);
    for _ in 0..4000 {
        step(sim);
        sim.drones.health.health = 100.0;
        for b in sim.drones.bullets.iter().filter(|b| b.weapon == MISSILE) {
            slowest = slowest.min(b.speed);
            fastest = fastest.max(b.speed);
        }
        if let Some(e) = sim.drones.explosions.iter().find(|e| e.4 == MISSILE) { blast = Some((e.1, e.2)); break; }
    }
    assert!(slowest < 0.06, "the missile left at full speed: {slowest}");
    assert!(fastest >= 0.2, "the missile never reached the record's 0.2: {fastest}");
    assert_eq!(blast, Some((4.0, 25.0)), "the missile's stop is a blast of its record's radius and damage [L 0x6d +0x08/+0x0C]");
}

/// `Collide_PickObj`'s sphere only nominates an object; `Collide_Intersect` runs its triangles [M
/// 0x1ebaa8:300-321]. A round in the open air over the deck sits inside the arch's 8.5 sphere and
/// touches nothing.
#[test]
fn a_round_in_the_open_air_over_the_deck_leaves_the_bridge_alone() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = deck_scene(&iso, false);
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.player_bullets.push(round([0.0, 1.5, 34.5], [0.0, 0.0, 1.0], 0.4, 0.4, 3.5));
    for _ in 0..8 { step(sim); }
    assert_eq!(bridge(sim).state, 1, "a round that struck nothing started the break script");
}

/// One round that strikes the bridge is one contact, not one per overlapping sphere.
#[test]
fn a_round_on_a_held_bridge_counts_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = deck_scene(&iso, false);
    let sim = scene.sim.as_mut().unwrap();
    // The first contact starts the script; it runs to its t1 hold.
    sim.drones.player_bullets.push(round([0.0, 1.5, 34.0], [0.0, 0.0, 1.0], 13.2, 50.0, 3.5));
    for _ in 0..40 { step(sim); }
    assert_eq!(bridge(sim).state, 2, "the shot at the bridge starts it");
    let held = |sim: &Sim| { let p = bridge(sim); p.info[p.cur].as_ref().is_some_and(|s| s.delay != 0) };
    assert!(held(sim), "the script holds at t1 [D stream 8]");
    let before = bridge(sim).damage;
    // The next pane along the outer wall [D stream 22, x -5.1..-2.4]: the first round broke its own.
    sim.drones.player_bullets.push(round([-3.7, 1.5, 35.0], [0.0, 0.0, 1.0], 13.2, 50.0, 3.5));
    for _ in 0..6 { step(sim); }
    assert_eq!(bridge(sim).damage - before, 3.5, "one round, one contact");
}

/// The guards' walk has no world-collision gate: root motion moves them and the only horizontal
/// constraint is the AI-network bounds push [M DroneAnim_PreTransCallback 0x13b490,
/// AINetwork_GetBoundsPushVectorForSphere 0x157bd0]. On a level with bounds a guard running its
/// route never stands rejected by scenery; only a shut door (the moving set) holds him.
#[test]
fn woken_guards_do_not_stall_against_scenery() {
    use nf_game::drone::state as st;
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mover = |s: u8| matches!(s, st::ATTACK | st::RUN_FOR_COVER | st::ALERT_TO_POSITION | st::SEARCH_AREA
        | st::GO_TO_GOAL | st::PATROL | st::COMBAT_OUT_OF_RANGE | st::UNDER_COVER_RETURN);
    let mut stalls = Vec::new();
    for spot in [[-15.0f32, 1.0, 28.6], [-30.0, 1.0, -10.0]] {
        let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).expect("load 07000012");
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..200 { step(sim); }
        sim.bond.pos = spot;
        for c in [1u32, 2, 3, 5, 7, 8, 9, 0x21, 0x34, 0x41, 0x44] { let f = sim.drones.frame; sim.drones.channels.set(c, 1, f); }
        let mut anchor = vec![([0.0f32; 3], 0u32, 0u32); sim.drones.list.len() + 32];
        for t in 1..3000u32 {
            step(sim);
            sim.drones.health.health = 100.0;
            sim.bond.pos[0] = spot[0];
            sim.bond.pos[2] = spot[2];
            for (i, d) in sim.drones.list.iter().enumerate().take(anchor.len()) {
                if !d.present || !d.enabled || d.corpse || d.health <= 0.0 || !mover(d.state) || d.blocked_reach == 0.0 {
                    anchor[i].1 = 0;
                    continue;
                }
                let moved = ((d.pos[0] - anchor[i].0[0]).powi(2) + (d.pos[2] - anchor[i].0[2]).powi(2)).sqrt();
                if anchor[i].1 == 0 || moved >= 0.1 { anchor[i] = (d.pos, t, 0); continue; }
                if t - anchor[i].1 >= 120 {
                    let door = sim.world.dynamic_boxes().iter().any(|(_, lo, hi)|
                        (lo[0] - 1.5..hi[0] + 1.5).contains(&d.pos[0]) && (lo[2] - 1.5..hi[2] + 1.5).contains(&d.pos[2]));
                    if !door { stalls.push((i, d.state, d.pos)); }
                    anchor[i].1 = t;
                }
            }
        }
    }
    assert!(stalls.is_empty(), "guards stood rejected by scenery: {stalls:?}");
}
