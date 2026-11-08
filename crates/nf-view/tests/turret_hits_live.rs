//! Rounds against the wall guns' real collision models (docs/reports/agent-turret-hits.md).
//!
//! The game's round-vs-object test is two-stage [M Collide_Update 0x1ea398]: `Collide_PickObj`
//! 0x1eae80 admits an object whose world sphere the round's line passes within, then
//! `Collide_Intersect` 0x1ebaa8 runs `Intersect_RayGeom` 0x10a8c8 over the object's glist
//! triangles at obj+0x90; a contact with no triangle hit is freed. `GT_Update` 0x180498 sums the
//! parts' contact damage into ext+0xCC — script A always, script B in state 4 [M :128-176] — for
//! every round, whoever fired it [M Collide_GetDamageNObjects 0x1ee220]. Opt-in: `NF_ISO`.

use nf_game::drone::PlayerBullet;
use nf_game::player::Input;
use nf_view::scene::Sim;

type V3 = [f32; 3];

fn load(level: &str) -> Option<nf_view::scene::Scene> {
    let iso = std::env::var("NF_ISO").ok()?;
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso, level, false).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 {
        if !sim.disabled { break; }
        sim.drones.scripts.skip_request = true;
        sim.step_bond(Input::default(), false, false, false);
    }
    // Two ticks so the dynamic set holds the turrets' posed parts.
    for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
    Some(scene)
}

fn round(pos: V3, dir: V3, frame: u32) -> PlayerBullet {
    PlayerBullet {
        pos, dir, speed: 2.0, travelled: 0.0, range: 200.0, damage: 10.0, weapon: 2, first: true, dead: false, bounces: 0,
        homing: false, target: None, target_copter: false, gravity: false, guided_cam: false, snd: 0, swoosh: None, born: frame,
        from_turret: None,
    }
}

fn add(a: V3, b: V3, k: f32) -> V3 { [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn norm(a: V3) -> V3 { let l = dot(a, a).sqrt(); [a[0] / l, a[1] / l, a[2] / l] }

/// Every script part of turret `ti`: (script slot, dynamic part id, world sphere centre, radius).
fn parts(sim: &Sim, ti: usize) -> Vec<(usize, u64, V3, f32)> {
    let t = &sim.drones.gt_turrets[ti];
    let mut out = Vec::new();
    for (slot, pi) in [(0usize, t.script_player_a), (1, t.script_player_b)] {
        for (o, h, m) in sim.drones.scripts.stream_glists(pi) {
            let Some(md) = sim.models.get(&h) else { continue };
            let c = md.centre;
            let w = [
                m[0][0] * c[0] + m[1][0] * c[1] + m[2][0] * c[2] + m[3][0],
                m[0][1] * c[0] + m[1][1] * c[1] + m[2][1] * c[2] + m[3][1],
                m[0][2] * c[0] + m[1][2] * c[1] + m[2][2] * c[2] + m[3][2]];
            out.push((slot, (o as u64) << 6, w, md.radius));
        }
    }
    out
}

fn line_dist2(a: V3, b: V3, p: V3) -> f32 {
    let ab = sub(b, a);
    let t = (dot(sub(p, a), ab) / dot(ab, ab)).clamp(0.0, 1.0);
    let q = sub(add(a, ab, t), p);
    dot(q, q)
}

/// The nearest part triangle hit of `a → b` over every turret part, as a fraction.
fn any_part_hit(sim: &Sim, ps: &[(usize, u64, V3, f32)], a: V3, b: V3) -> Option<(usize, f32)> {
    ps.iter().filter_map(|&(slot, id, _, _)| sim.world.segment_dynamic_part(id, a, b).map(|t| (slot, t)))
        .min_by(|x, y| x.1.total_cmp(&y.1))
}

/// A line of 6 units, ending past the turret, that the world stops first on a script-A part
/// (`hit`), and a parallel one that runs inside a part's world sphere yet crosses no part
/// triangle and no world surface before the far side of the gun (`miss`).
fn lines(sim: &Sim, ti: usize) -> ((V3, V3), (V3, V3)) {
    let ps = parts(sim, ti);
    let mut hit = None;
    let mut miss = None;
    // Approach directions: the gun's own facing, then the eight horizontal headings.
    let fd = sim.drones.gt_turrets[ti].fire_dir;
    let mut dirs = vec![[-fd[0], -fd[1], -fd[2]]];
    for k in 0..8 { let a = k as f32 * std::f32::consts::FRAC_PI_4; dirs.push([a.sin(), 0.0, a.cos()]); }
    for &(slot, id, c, r) in &ps {
        for &dir in &dirs {
            let dir = norm(dir);
            // A side axis perpendicular to the flight.
            let side = norm(if dir[1].abs() < 0.9 { [dir[2], 0.0, -dir[0]] } else { [1.0, 0.0, 0.0] });
            let up = [dir[1] * side[2] - dir[2] * side[1], dir[2] * side[0] - dir[0] * side[2], dir[0] * side[1] - dir[1] * side[0]];
            for i in -20..=20 {
                for j in -20..=20 {
                    let off = add(add([0.0; 3], side, i as f32 * r / 20.0), up, j as f32 * r / 20.0);
                    let through = add(c, off, 1.0);
                    let a = add(through, dir, -3.0);
                    let b = add(through, dir, 3.0);
                    let wt = sim.world.segment_flight(a, b, true);
                    let pt = any_part_hit(sim, &ps, a, b);
                    if hit.is_none() && slot == 0 {
                        // The world's first stop is this script-A part's own triangle.
                        if let (Some(w), Some(t)) = (wt, sim.world.segment_dynamic_part(id, a, b)) {
                            if t <= w && pt.is_some_and(|(s, _)| s == 0) { hit = Some((a, dir)); }
                        }
                    }
                    if miss.is_none() && line_dist2(a, b, c) < r * r * 0.81 && pt.is_none() {
                        // No world surface before the line leaves the gun's spheres.
                        let far = ps.iter().map(|&(_, _, pc, pr)| (dot(sub(pc, a), dir) + pr) / 6.0).fold(0.0f32, f32::max);
                        if wt.is_none_or(|w| w > far) { miss = Some((a, dir)); }
                    }
                }
            }
        }
    }
    (hit.expect("a line onto a script-A part"), miss.expect("a line through a part sphere that misses the model"))
}

fn shoot(sim: &mut Sim, from: V3, dir: V3) {
    let frame = sim.drones.frame;
    sim.drones.player_bullets.push(round(from, dir, frame));
    for _ in 0..8 {
        sim.drones.health.health = 100.0;
        sim.step_bond(Input::default(), false, false, false);
    }
}

/// [M Collide_PickObj 0x1eae80 + Collide_Intersect 0x1ebaa8 + Intersect_RayGeom 0x10a8c8] On
/// 0700000a's first wall gun: a round through a part's world sphere that crosses none of the
/// parts' triangles takes no health off the gun; the same round onto a script-A part does.
/// Red on the sphere test: the sphere entry alone counted the first round.
#[test]
fn a_round_beside_the_model_misses_and_one_on_it_hits_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = sim.drones.gt_turrets.iter().position(|t| t.facing == 1).expect("a wall gun");
    let ((ha, hd), (ma, md)) = lines(sim, ti);
    // Bond far away so the gun stays dormant (script A counts in every state).
    sim.bond.pos = add(sim.drones.gt_turrets[ti].pos, [0.0, 0.0, 1.0], 80.0);
    let h0 = sim.drones.gt_turrets[ti].health;
    for _ in 0..5 { shoot(sim, ma, md); }
    assert_eq!(sim.drones.gt_turrets[ti].health, h0, "rounds through the sphere beside the model do not hurt the gun");
    shoot(sim, ha, hd);
    let h1 = sim.drones.gt_turrets[ti].health;
    let contact = sim.weapons.bullet_contact_damage(2);
    // Each landing takes the round's contact damage; a metal ricochet off one part can land on
    // the next part (the reflected flight re-picks every tick), so count whole contacts.
    let n = (h0 - h1) / contact;
    assert!(n >= 1.0 && (n - n.round()).abs() < 1e-3, "the round on the model takes whole contacts off: {h0} → {h1} (contact {contact})");
}

/// [M Collide_Update 0x1ea398:160-215, Collide_GetDamageNObjects 0x1ee220] A guard's round on a
/// script-A part damages the gun like Bond's. Red before: the guard bullet pass fed no turret.
#[test]
fn a_guard_round_on_the_model_hurts_the_gun_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = sim.drones.gt_turrets.iter().position(|t| t.facing == 1).expect("a wall gun");
    let ((ha, hd), (ma, md)) = lines(sim, ti);
    sim.bond.pos = add(sim.drones.gt_turrets[ti].pos, [0.0, 0.0, 1.0], 80.0);
    let h0 = sim.drones.gt_turrets[ti].health;
    let guard_round = |pos: V3, dir: V3| nf_game::bullet::Bullet {
        pos, dir, speed: 2.0, travelled: 0.0, range: 200.0, damage: 10.0, owner: 0, state: 0, weapon: 2,
        flyby: false, bounces: 0, glass_pass: true, ..nf_game::bullet::Bullet::TAIL
    };
    // Sister: the miss line stays a miss for a guard's round too.
    sim.drones.bullets.push(guard_round(ma, md));
    for _ in 0..8 { sim.drones.health.health = 100.0; sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.gt_turrets[ti].health, h0, "a guard round beside the model does not hurt the gun");
    sim.drones.bullets.push(guard_round(ha, hd));
    for _ in 0..8 { sim.drones.health.health = 100.0; sim.step_bond(Input::default(), false, false, false); }
    let h1 = sim.drones.gt_turrets[ti].health;
    let contact = sim.weapons.bullet_contact_damage(2);
    assert!((h0 - h1 - contact).abs() < 1e-3, "a guard round on the model takes its contact damage off: {h0} → {h1} (contact {contact})");
}

/// The turret's own rounds never land on its own parts (the port's own-round skip, see
/// `PlayerBullet::from_turret`), but they are ordinary rounds for every other turret's parts.
#[test]
fn a_turret_round_skips_only_its_own_gun_0a() {
    let Some(mut scene) = load("0700000a.bin") else { return };
    let sim = scene.sim.as_mut().unwrap();
    let ti = sim.drones.gt_turrets.iter().position(|t| t.facing == 1).expect("a wall gun");
    let ((ha, hd), _) = lines(sim, ti);
    sim.bond.pos = add(sim.drones.gt_turrets[ti].pos, [0.0, 0.0, 1.0], 80.0);
    let h0 = sim.drones.gt_turrets[ti].health;
    let own = sim.drones.gt_turrets[ti].sp_key;
    let other = sim.drones.gt_turrets.iter().find(|t| t.sp_key != own).map(|t| t.sp_key).expect("a second gun");
    let frame = sim.drones.frame;
    sim.drones.player_bullets.push(PlayerBullet { from_turret: Some(own), ..round(ha, hd, frame) });
    for _ in 0..8 { sim.drones.health.health = 100.0; sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.drones.gt_turrets[ti].health, h0, "the gun's own round does not hurt it");
    let frame = sim.drones.frame;
    sim.drones.player_bullets.push(PlayerBullet { from_turret: Some(other), ..round(ha, hd, frame) });
    for _ in 0..8 { sim.drones.health.health = 100.0; sim.step_bond(Input::default(), false, false, false); }
    assert!(sim.drones.gt_turrets[ti].health < h0, "another gun's round does");
}
