//! point-blank-pile-hits — 48 rounds at ~1 unit into four guards standing on one spot landed
//! almost no hits on origin/s/enemy-stats-3's 07000003 full-route run. A player firing
//! point-blank into a group should drop them: the game has NO minimum-range gate in the bullet
//! path — the round spawns at the eye (`Player_WeaponInitBullet` 0x1a4bf0:152 `Player_GetHeadPos`;
//! at most a 50/50 blend with the muzzle datum [M 0x1a4bf0:233-241]), the first flight tick sweeps
//! [eye, eye + (0.5+FRand(0.5))·speed] [M Bullet_update 0x12e7b0:382-384,448-464], `Collide_PickObj`
//! 0x1eae80 admits every candidate whose bounding sphere (obj+0x80 centre, obj+0x8C ≈ 1.05 radius)
//! the segment touches [M 0x1eae80:93-107], and `Collide_Jointy` 0x1ec180 returns t = 0 (hit,
//! sorted first at HITDATA+0x0C = −1.0) when the segment STARTS INSIDE a box [M 0x1e9008:18-25;
//! 0x1ec180:124-137]. The sorted contact list is walked to the first solid node [M Collide_Update
//! 0x1ea398:216-229] — one victim per round, corpses (`obj+0xF0 & 0x20`) skipped, so a pile soaks
//! rounds front to back. Repro: four held team-1 guards on one spot on 07000003, rounds at 1 u
//! (sister 5 u) through the real input path — both the direct `step_bond(fire)` path and the
//! walker's own aim/tap discipline (`exposed_aim` + the crosshair/autoaim fire path the route run
//! used).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn load_03() -> Option<nf_view::scene::Scene> {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return None };
    match nf_view::load(&iso, "07000003.bin", false) {
        Ok(s) => Some(s),
        Err(e) => { eprintln!("load 07000003: {e}"); None }
    }
}

fn aim_at(sim: &mut nf_view::scene::Sim, to: [f32; 3]) {
    let e = sim.bond.head();
    let d = [to[0] - e[0], to[1] - e[1], to[2] - e[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = (d[1] / l).clamp(-1.0, 1.0).asin() / (std::f32::consts::FRAC_PI_2);
}

/// Four live TEAM-1 guards moved onto ONE spot (the first guard's own floor position) and held
/// there (`hold_npcs` — the guards freeze but stay shootable, hold_npcs_live), with Bond armed
/// (P2K) standing `dist` in front of the pile on the same floor. Killing a team-3 servant fails
/// the mission and disables firing (walkreal_07000003's `killing_a_servant_fails_the_mission`),
/// so the pile is team-1 only.
fn setup_pile(scene: &mut nf_view::scene::Scene, dist: f32) -> ([f32; 3], Vec<usize>) {
    let sim = scene.sim.as_mut().unwrap();
    // Most of 07000003's guards spawn later via channels — at boot only a couple of team-1
    // guards are live, so force-present spawner-held team-1 placements to fill the pile up.
    let mut idxs: Vec<usize> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1)
        .map(|(i, _)| i).take(4).collect();
    if idxs.len() < 4 {
        for i in 0..sim.drones.list.len() {
            if idxs.len() >= 4 { break; }
            let d = &sim.drones.list[i];
            if !idxs.contains(&i) && !d.corpse && d.health > 0.0 && d.team == 1 {
                sim.drones.list[i].present = true;
                sim.drones.list[i].enabled = true;
                idxs.push(i);
            }
        }
    }
    assert!(idxs.len() == 4, "07000003 needs four team-1 guards for the pile, got {}", idxs.len());
    let pile = sim.drones.list[idxs[0]].pos;
    sim.drones.hold_npcs = true;
    for &i in &idxs {
        sim.drones.list[i].pos = pile;
        let dr = &sim.drones.list[i];
        eprintln!("piled guard {i}: skin {:06x} hp {:.1} boxes {}", dr.skin, dr.health,
            dr.hit_boxes(&sim.data()).len());
    }
    // Bond `dist` in front of the pile, on the pile's floor, facing it.
    let yaw0 = sim.bond.yaw;
    let d = [pile[0] - sim.bond.pos[0], 0.0, pile[2] - sim.bond.pos[2]];
    let l = (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-6);
    let spot = [pile[0] - d[0] / l * dist, pile[1], pile[2] - d[2] / l * dist];
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, spot, yaw0);
    sim.gun.weapon = 2;
    sim.gun.switch_request = 2;
    sim.gun.switch_state = 0;
    for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
    (pile, idxs)
}

/// The pin: N = 5 P2K rounds at 1 u into the four-guard pile land 5/5 guard hits — the game's
/// first-solid-node walk ends on a guard for every round while the pile stands [M Collide_Update
/// 0x1ea398:216-229; the round spawns at the eye and the first segment already covers the guard
/// (Bullet_update 0x12e7b0:382-384,448-464; Collide_Jointy's t=0 inside hit sorts first,
/// 0x1ec180:124-137); corpses pass so the soak runs front to back]. Fired through the real fire
/// path with the torso box re-aimed every shot.
#[test]
fn five_rounds_at_1u_into_the_pile_all_land() {
    let Some(mut scene) = load_03() else { return };
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let (pile, idxs) = setup_pile(&mut scene, 1.0);
    let stats0 = |s: &nf_view::scene::Sim| (s.drones.progress.stats.iter().map(|s| s.shots_fired).sum::<u32>(),
        s.drones.progress.stats.iter().map(|s| s.shots_hit_enemy).sum::<u32>());
    let (f0, h0) = stats0(scene.sim.as_ref().unwrap());
    for _ in 0..5 {
        for _ in 0..14 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        {
            let sim = scene.sim.as_mut().unwrap();
            let Some(ti) = idxs.iter().copied().find(|&i| !sim.drones.list[i].corpse && sim.drones.list[i].health > 0.0) else { break };
            let boxes = sim.drones.list[ti].hit_boxes(&sim.data());
            let c = match boxes.iter().find(|b| b.part == 4).map(|b| b.m[3]) {
                Some(c) => [c[0], c[1], c[2]],
                None => { eprintln!("guard {ti} has NO torso box (boxes {})", boxes.len()); pile }
            };
            aim_at(sim, c);
            sim.step_bond(Input::default(), true, false, false); // the fire press
        }
    }
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); } // the fold
    let (f1, h1) = stats0(scene.sim.as_ref().unwrap());
    let sim = scene.sim.as_ref().unwrap();
    let hp: Vec<(usize, f32)> = idxs.iter().map(|&i| (i, sim.drones.list[i].health)).collect();
    eprintln!("1 u: fired {} hits {}, pile {pile:?}, hp {hp:?}", f1 - f0, h1 - h0);
    assert_eq!(f1 - f0, 5, "five rounds fired");
    assert_eq!(h1 - h0, 5, "every point-blank round lands on the pile (the game's no-minimum-range pick, the accuracy row), hp {hp:?}");
    assert!(hp.iter().any(|&(_, h)| h < 24.0), "the pile took damage, hp {hp:?}");
}

/// Sister at 5 u: the same rounds land 5/5 (the pile is not a point-blank-only artefact).
#[test]
fn five_rounds_at_5u_into_the_pile_all_land() {
    let Some(mut scene) = load_03() else { return };
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let (pile, idxs) = setup_pile(&mut scene, 5.0);
    let stats0 = |s: &nf_view::scene::Sim| (s.drones.progress.stats.iter().map(|s| s.shots_fired).sum::<u32>(),
        s.drones.progress.stats.iter().map(|s| s.shots_hit_enemy).sum::<u32>());
    let (f0, h0) = stats0(scene.sim.as_ref().unwrap());
    for _ in 0..5 {
        for _ in 0..14 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        {
            let sim = scene.sim.as_mut().unwrap();
            let Some(ti) = idxs.iter().copied().find(|&i| !sim.drones.list[i].corpse && sim.drones.list[i].health > 0.0) else { break };
            let boxes = sim.drones.list[ti].hit_boxes(&sim.data());
            let c = boxes.iter().find(|b| b.part == 4).map(|b| [b.m[3][0], b.m[3][1], b.m[3][2]]).unwrap_or(pile);
            aim_at(sim, c);
            sim.step_bond(Input::default(), true, false, false);
        }
    }
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); } // the fold
    let (f1, h1) = stats0(scene.sim.as_ref().unwrap());
    let sim = scene.sim.as_ref().unwrap();
    let hp: Vec<(usize, f32)> = idxs.iter().map(|&i| (i, sim.drones.list[i].health)).collect();
    eprintln!("5 u: fired {} hits {}, pile {pile:?}, hp {hp:?}", f1 - f0, h1 - h0);
    assert_eq!(f1 - f0, 5, "five rounds fired");
    assert_eq!(h1 - h0, 5, "every 5 u round lands on the pile, hp {hp:?}");
}

/// The route run's own discipline: the walker's `shoot` (exposed_aim + the input-driven turn +
/// the crosshair/autoaim fire path + the tap thresholds) against the pile's nearest guard, with
/// `prefer_head` as the enemy-stats-3 walker set. Every shot the walker fires must land on the
/// pile — at 1 u the game drops the group.
#[test]
fn the_walker_shoots_the_pile_at_1u() {
    let Some(scene) = load_03() else { return };
    let mut w = Walker::new(scene);
    for _ in 0..30 { w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let (pile, idxs) = setup_pile(&mut w.scene, 1.0);
    let mut fired0 = w.scene.sim.as_ref().unwrap().drones.progress.stats.iter().map(|s| s.shots_fired).sum::<u32>();
    let mut hit0 = w.scene.sim.as_ref().unwrap().drones.progress.stats.iter().map(|s| s.shots_hit_enemy).sum::<u32>();
    let mut shots = 0u32;
    let mut hits = 0u32;
    for k in 0..4 {
        let Some(di) = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
            .filter(|(i, d)| idxs.contains(i) && d.team == 1 && d.enabled && d.present && !d.corpse && d.health > 0.0)
            .map(|(i, _)| i).min_by(|&a, &b| {
                let b0 = w.scene.sim.as_ref().unwrap().bond.pos;
                let (pa, pb) = (w.scene.sim.as_ref().unwrap().drones.list[a].pos, w.scene.sim.as_ref().unwrap().drones.list[b].pos);
                let da = (pa[0] - b0[0]).powi(2) + (pa[2] - b0[2]).powi(2);
                let db = (pb[0] - b0[0]).powi(2) + (pb[2] - b0[2]).powi(2);
                da.total_cmp(&db)
            }) else { break };
        let ok = w.shoot(di, &format!("pile guard {k}"));
        let sim = w.scene.sim.as_ref().unwrap();
        let fired1 = sim.drones.progress.stats.iter().map(|s| s.shots_fired).sum::<u32>();
        let hit1 = sim.drones.progress.stats.iter().map(|s| s.shots_hit_enemy).sum::<u32>();
        eprintln!("walker shoot {k}: drone {di} ok {ok} fired {} hits {} (pile {pile:?})",
            fired1 - fired0, hit1 - hit0);
        shots += fired1 - fired0;
        hits += hit1 - hit0;
        fired0 = fired1;
        hit0 = hit1;
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let hp: Vec<(usize, f32)> = idxs.iter().map(|&i| (i, sim.drones.list[i].health)).collect();
    eprintln!("walker 1 u: fired {shots} hits {hits}, hp {hp:?}");
    assert!(shots > 0, "the walker fired at the pile");
    assert_eq!(hits, shots, "every round the walker fired at 1 u lands on the pile (the accuracy row), hp {hp:?}");
}
