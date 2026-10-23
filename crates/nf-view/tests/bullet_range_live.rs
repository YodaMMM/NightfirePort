//! bullet-range-210: the P2K round's flight death range on the real path.
//!
//! The game kills a flying round when `weapon_data+0x1C < BU+0xF4` [M `Bullet_update` case 1,
//! 0x12e95c-0x12e984 via the state jump table @0x2f4300], with the step clamped to the +0x1C
//! boundary [M 0x12f014-0x12f02c] and the death landing the round ON the boundary. The P2K's
//! record carries +0x1C = 50.0 [D weapon_data 0x2bf150 rec 2], so a P2K round reaches a wall
//! ~45 units out and dies in mid-air with the first geometry at ~80 — NOT at the 210.0 the
//! merged task-831 note claimed (`BU+0xFC` is the fuse clock: `Bullet_init` 0x12c5a0:103's
//! 210.0, counted down only for `+0x70 & 0x80000` records at 0x12ee08 and id 0x45 at 0x12efa4).
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test bullet_range_live`

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The flagged-tree cluster east of the spawn (the fix-foliage-bullets fixtures): the shot lane
/// from the east is clear terrain until its east shell (x = 93.52 [D inst 80 bbox]).
const TREE_HI_X: f32 = 93.52;

/// Fire one real P2K round from `x` at the tree crown and collect the world impacts over the
/// round's whole flight (the fire input on tick 0 only, mirroring fix_foliage_bullets_live).
fn fire_one_p2k(iso: &str, x: f32) -> Vec<[f32; 3]> {
    let mut scene = nf_view::load(iso, "07000005.bin", false).expect("load 07000005");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        assert!(matches!(sim.gun.weapon, 0x02 | 0x03), "the spawn gun is the P2K, got {:#x}", sim.gun.weapon);
        sim.bond.pos = [x, 24.30, 90.80];
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + 0.70, sim.bond.pos[2]];
        let target = [91.0f32, 25.5, 90.8];
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let dir = [d[0] / l, d[1] / l, d[2] / l];
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
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

/// A P2K round fired at the tree from ~45 units reaches it: the world impact lands on the
/// tree's shell span (inside its +0x1C = 50.0 flight death range [M 0x12e95c; D weapon_data
/// rec 2 = 50.0]). The task's "a wall 150 units away" scenario is the sister's job with the
/// 1200-unit sniper record — the P2K cannot fly 150 in the game at all.
#[test]
fn the_p2k_round_reaches_a_wall_45_units_out() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let x = TREE_HI_X + 45.0;
    let impacts = fire_one_p2k(&iso, x);
    assert!(!impacts.is_empty(), "the round records a world impact (the tree at ~45 < 50)");
    for p in &impacts {
        assert!((88.0..=94.5).contains(&p[0]), "the impact is the tree shell span, got {p:?}");
    }
    eprintln!("[reach] impacts from x={x}: {impacts:?}");
}

/// Sister: from ~80 units the SAME lane's first geometry is beyond the P2K's +0x1C = 50 — the
/// round dies in mid-air ON the clamped boundary [M 0x12f014-0x12f02c + the next tick's case-1
/// kill at 0x12e95c] and records NO world impact. On the 831 note's 210.0 model this round
/// would have reached the tree (80 < 210) — the refutation in flight.
#[test]
fn the_p2k_round_dies_midair_before_the_wall_80_units_out() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let x = TREE_HI_X + 80.0;
    let impacts = fire_one_p2k(&iso, x);
    assert!(impacts.is_empty(), "the round must die at its +0x1C = 50 before the tree at ~80, got impacts {impacts:?}");
}

/// The task's long-reach scenario with the record that actually carries it: the sniper-family
/// +0x1C = 1200.0 [D weapon_data rec 0x1e]. Fired from x=211.8 down the clear west lane the
/// round flies ~119 units to the tree shell (the probe's first west geometry) — four times the
/// P2K's 50 boundary in the same lane — proving a real-path round whose flight far exceeds the
/// P2K's range-out. The hit lands on the tree shell span.
#[test]
fn the_sniper_record_rounds_fly_far_past_the_p2k_boundary() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_ref().unwrap();
    let vol = nf_game::bullet::PlayerVolume { feet: [1e4, 0.0, 1e4], height: 0.1, radius: 0.1 };
    let x0 = 211.8f32;
    let mut b = nf_game::bullet::Bullet { pos: [x0, 25.0, 90.8], dir: [-1.0, 0.0, 0.0], speed: 13.2, travelled: 0.0,
        range: 1200.0, damage: 0.0, owner: 0, state: 0, weapon: 0x1e, flyby: false, bounces: 0 };
    let mut rng = nf_game::drone::Rng::new(5);
    let mut hit = None;
    for _ in 0..40 {
        if let Some(nf_game::bullet::BulletEvent::World(p)) = b.update(&sim.world, &vol, [1e4, 0.0, 1e4], &mut rng) {
            hit = Some(p);
            break;
        }
    }
    let p = hit.expect("the 1200 record's round reaches the lane's first geometry (the tree at ~119)");
    assert!((88.0..=94.5).contains(&p[0]), "the impact is the tree shell span, got {p:?}");
    assert!(x0 - p[0] > 100.0, "the round flew {} units — far past the P2K's 50 boundary", x0 - p[0]);
}
