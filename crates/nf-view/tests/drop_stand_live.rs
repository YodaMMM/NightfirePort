//! drop-stand-height: Bond standing over a resting guard drop and taking it, through the real
//! path (`nf_view::load` + `Sim::step_bond`). Opt-in via `NF_ISO`.
//!
//! The game's ground ray skips both the drop and the corpse: `build_PointOnFloor` 0x1c95a8:52 runs
//! flags 0x70c (bit 0x8 → `Collide_PickObj` 0x1eae80:72-78 adds the skip bit 0x40 = pickups) and the
//! base filter skips bit 0x20 (the corpse bit `NDrone2_SetAsDead` 0x14a2f0 sets) for EVERY query —
//! so Bond stands at floor + 1.1 over any drop, never lifted onto it [M]. The take ray
//! (`Pickup_Update` 0x18e148:210-214, flags 0x721, the pickup's own obj excluded) ends on the
//! resting floor contact without refusing [L 01-raptor.trace 6.0 s — the game takes floor-resting
//! drops]. Sister: the LOS gate against a real wall is pinned ISO-free by `pickup.rs`'s
//! `the_take_ray_origin_decides_the_take` (a 0.5-high wall refuses from below the top, clears above).

use nf_view::dev::Dev;
use nf_view::scene::Scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str, level: &str) -> Scene {
    let mut scene = nf_view::load(iso, level, false).expect("load");
    let intro: &[u32] = match level { "07000001.bin" => &[0x0600_0087], _ => &[] };
    if !intro.is_empty() {
        for t in 0..600u32 {
            let sim = scene.sim.as_mut().unwrap();
            let pending = sim.drones.scripts.players.iter().any(|p| intro.contains(&p.scripts[0]) && (matches!(p.state, 1 | 2 | 7) || p.info[p.cur].as_ref().is_some_and(|s| s.play & 1 != 0)));
            if t >= 2 && !pending && !sim.disabled { break; }
            sim.drones.scripts.skip_request = true;
            sim.step_bond(Default::default(), false, false, false);
        }
    }
    scene
}

fn cheats() -> Dev { let mut d = Dev::default(); d.god_mode = true; d.infinite_ammo = true; d }

fn step(scene: &mut Scene, trigger: bool, dev: Option<&mut Dev>) {
    { let sim = scene.sim.as_mut().unwrap(); sim.step_bond(Default::default(), trigger, false, false); }
    if let Some(d) = dev { d.after_tick(scene); }
}

fn tick_idle(scene: &mut Scene, n: u32, mut dev: Option<&mut Dev>) {
    for _ in 0..n { step(scene, false, dev.as_deref_mut()); }
}

fn aim_at(scene: &mut Scene, to: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let e = sim.bond.head();
    let d = [to[0] - e[0], to[1] - e[1], to[2] - e[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).clamp(-1.0, 1.0).asin() / (std::f32::consts::FRAC_PI_2);
}

fn kill_guard(scene: &mut Scene, gi: usize, mut dev: Option<&mut Dev>) -> bool {
    let spots = |gp: [f32; 3], floor: f32| -> Vec<[f32; 3]> {
        let mut v = Vec::new();
        for r in [3.0f32, 4.5] {
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                v.push([gp[0] + r * a.sin(), floor + 1.1, gp[2] + r * a.cos()]);
            }
        }
        v
    };
    let mut spot = 0usize;
    for t in 0..2000 {
        let (gp, gchest) = { let dr = &scene.sim.as_ref().unwrap().drones.list[gi]; (dr.pos, [dr.pos[0], dr.pos[1] + 0.3, dr.pos[2]]) };
        let los = |from: [f32; 3]| scene.sim.as_ref().unwrap().world.line_of_sight([from[0], from[1] + 1.0, from[2]], gchest, nf_game::world::LOS_PROBE_8);
        let floor = scene.sim.as_ref().unwrap().world.floor_below(gp, 6.0).unwrap_or(gp[1]);
        let cands = spots(gp, floor);
        if t == 0 || !los(cands[spot % cands.len()]) {
            spot += 1;
            let Some(next) = (spot..spot + cands.len()).map(|i| i % cands.len()).find(|&i| los(cands[i])) else { continue };
            spot = next;
            let sim = scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, cands[spot], 0.0);
        }
        aim_at(scene, gchest);
        step(scene, t % 9 == 0, dev.as_deref_mut());
        if scene.sim.as_ref().unwrap().drones.list[gi].health <= 0.0 { break; }
    }
    let dead = scene.sim.as_ref().unwrap().drones.list[gi].health <= 0.0;
    tick_idle(scene, 90, dev.as_deref_mut()); // the drop falls and lands (Pickup_Update's fall)
    dead
}

/// Bond placed exactly on a resting guard drop stands at floor + 1.1 (never lifted onto the drop or
/// the corpse — the game's ground ray skips both [M 0x1c95a8:52 + 0x1eae80:72-78]) and the take
/// happens through the real tick path (the ray ends on the resting floor contact; the end band
/// absorbs the rest noise, pickup.rs `touches`).
#[test]
fn bond_on_a_resting_drop_stands_at_floor_plus_1_1_and_takes_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000001.bin");
    tick_idle(&mut scene, 200, None);
    // The first Raptor (weapon 0xE [D 07000001.bin placement param 9]) has no gate channel —
    // alive from load, the deterministic kill.
    let gi = scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .find(|(_, dr)| dr.present && !dr.corpse && dr.health > 0.0 && dr.weapon == 0xe)
        .map(|(i, _)| i).expect("07000001 has a Raptor guard");
    assert!(kill_guard(&mut scene, gi, Some(&mut cheats())), "guard {gi} died");

    let (dc, floor_hi) = { let sim = scene.sim.as_ref().unwrap();
        let di = sim.drones.pickups.iter().enumerate()
            .filter(|(_, p)| p.kind == nf_game::pickup::kind::WEAPON && !p.gone)
            .map(|(i, _)| i).next_back().expect("the guard's weapon drop rests");
        let c = sim.drones.pickups[di].centre();
        (c, sim.world.floor_below([c[0], c[1] + 3.0, c[2]], 10.0).expect("the drop rests on floor")) };

    // Stand exactly on the drop (the take_drop convention: 0.4 above its centre).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [dc[0], dc[1] + 0.4, dc[2]], 0.0);
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let root = sim.bond.pos[1];
        assert!((root - (floor_hi + 1.1)).abs() < 1e-3,
            "Bond stands at floor + 1.1 over the drop [M 0x1c95a8:52's ground ray skips pickups and corpses] — root {root:.4} vs floor {floor_hi:.4} + 1.1 (the batch-1022l symptom lifted him 0.139)");
    }
    // The take: within the distance gate (radius + 1.1 [M 0x18e148:210-214]) and the ray clear.
    let mut taken = false;
    for _ in 0..40 {
        let before = scene.sim.as_ref().unwrap().drones.taken.len();
        step(&mut scene, false, None);
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.taken.len() > before { taken = true; break; }
    }
    assert!(taken, "the drop under Bond's feet is taken through the real tick path");
}
