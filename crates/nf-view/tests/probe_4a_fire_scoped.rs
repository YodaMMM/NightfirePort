//! SCRATCH PROBE (level-4a-every-objective-2) — measures, from the ride line, the eye→drone
//! ray under the FLAT candidate set (`segment_bullet`, every AABB-passing triangle) versus the
//! GAME's candidate set (`Collide_Pick` 0x1eb320:125 → `Collide_StraddleCels` 0x1eac98: the BFS
//! from the query's cel through the portals the ray crosses, cap 0x7e — the port's
//! `segment_bullet_scoped(find_cel(eye), …)`), and then fires through the REAL fire path
//! (`tick_fire` → PlayerBullet → the scoped flight trace [M Bullet_update 0x12e7b0 disasm
//! 0x12f0cc-0x12f0d8 re-feeds the bullet's cel every tick]) to see whether the garrison takes
//! fire end-to-end. Probe only — prints, no pass/fail on the game's behaviour.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[path = "common/walker.rs"] mod walker;
use walker::Walker;

fn key_y(y0: f32, t: u32) -> f32 {
    (y0 - ((t as f32 - 90.0).max(0.0)) * (150.0 / 4950.0)).max(y0 - 150.0)
}

struct Ride { w: Walker, y0: f32, t: u32, z: f32 }

impl Ride {
    fn new(iso: &str) -> Ride {
        let scene = nf_view::load(iso, "0700004a.bin", false).expect("load 0700004a");
        let mut w = Walker::new(scene);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
        }
        let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
        Ride { w, y0, t: 0, z: -3.4 }
    }
    fn step(&mut self, input: Input) {
        if self.t % 30 == 0 {
            let sim = self.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(self.y0, self.t) + 1.2, self.z];
            sim.drones.health.health = 100.0;
        }
        self.w.tick(input, false);
        self.t += 1;
    }
}

#[test]
fn probe_one_aimed_shot_trace() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    // REAL LOADOUT (the watch 0x10, 7 rounds [M fresh_11]) — aim at the first scoped-clear
    // chest each tick, press on converged ticks (release between: semi-auto), trace the hp.
    let hp0: Vec<f32> = r.w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.health).collect();
    let mut presses = 0u32; let mut clear_ticks = 0u32; let mut released = true;
    for tick in 0..2400u32 {
        if tick % 30 == 0 {
            let sim = r.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(r.y0, r.t) + 1.2, -3.4];
            sim.drones.health.health = 100.0;
        }
        let hit = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().enumerate().any(|(i, d)| d.corpse || d.health < hp0[i] - 0.001)
        };
        if hit { eprintln!("PROBE5 HIT at t={tick} presses={presses}"); break; }
        let aim = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye, sim.bond.pos[2]];
            let cell = sim.world.find_cel(e);
            sim.drones.list.iter().enumerate()
                .filter(|(_, dr)| dr.present && !dr.corpse && dr.enabled)
                .find_map(|(i, dr)| {
                    let c = [dr.pos[0], dr.pos[1] + 1.4, dr.pos[2]];
                    (sim.world.segment_bullet_scoped(cell, e, c).is_none()).then_some((i, c))
                })
        };
        let Some((di, chest)) = aim else { released = true; r.w.tick(Input::default(), false); r.t += 1; continue; };
        clear_ticks += 1;
        let (input, converged) = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let e = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
            let d = [chest[0] - e[0], chest[1] - e[1], chest[2] - e[2]];
            let want_yaw = d[0].atan2(d[2]);
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
            let want_pitch = (d[1] / len).asin() / std::f32::consts::FRAC_PI_2;
            let yaw_err = (want_yaw - b.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
            let perr = want_pitch - b.pitch;
            (Input { turn: yaw_err.clamp(-0.06, 0.06), look: perr.clamp(-0.06, 0.06), ..Default::default() },
             yaw_err.abs() < 0.02 && perr.abs() < 0.02)
        };
        if converged && released && sim_clip(&r) > 0 {
            presses += 1; released = false;
            r.w.tick_fire(input);
        } else { released = true; r.w.tick(input, false); }
        r.t += 1;
        if tick % 300 == 0 {
            let sim = r.w.scene.sim.as_ref().unwrap();
            eprintln!("PROBE5 t={tick} presses={presses} clear={clear_ticks} weapon={} clip={} target={di}",
                sim.gun.weapon, sim.gun.clip);
        }
    }
    let sim = r.w.scene.sim.as_ref().unwrap();
    let hurt: Vec<String> = sim.drones.list.iter().enumerate()
        .filter(|(i, d)| d.corpse || d.health < hp0[*i] - 0.001)
        .map(|(i, d)| format!("#{i} {:.1}→{:.1}", hp0[i], d.health)).collect();
    eprintln!("PROBE5 END: presses={presses} clear={clear_ticks} clip={} hurt={hurt:?}", sim.gun.clip);
}

fn sim_clip(r: &Ride) -> u32 { r.w.scene.sim.as_ref().unwrap().gun.clip }

#[test]
fn probe_fire_scoped_vs_flat_from_the_ride_line() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    let (di, hp0) = loop {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
            if let Some(&d0) = sp.spawned.first() { break (d0, sim.drones.list[d0].health); }
        }
        r.step(Input::default());
        assert!(r.t < 3000, "spawner never released");
    };
    eprintln!("PROBE: released at t={} drone={di} hp0={hp0}", r.t);

    let mut flat_clear = 0u32; let mut scoped_clear = 0u32; let mut fired = 0u32;
    let mut hit_tick: Option<u32> = None;
    let mut first_scoped: Option<(u32, f32)> = None; // (tick, bond y)
    let mut diag_printed = 0u32;
    let mut any_flat_clear: Option<(u32, usize, [f32; 3])> = None; // (tick, drone, drone pos)
    let mut per_drone_scoped = std::collections::BTreeMap::<usize, u32>::new();
    for tick in 0..2400u32 {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            if sim.drones.list[di].corpse || sim.drones.list[di].health < hp0 { hit_tick = Some(tick); }
        }
        if hit_tick.is_some() { break; }
        if tick % 30 == 0 {
            let sim = r.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(r.y0, r.t) + 1.2, -3.4];
            sim.drones.health.health = 100.0;
        }
        let (bp, eye_h, crouch, dyaw, dpitch, at_now, flat, scoped) = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let at = sim.drones.list[di].pos;
            let flat = sim.world.segment_bullet(e, at).is_none();
            let cell = sim.world.find_cel(e);
            let scoped = sim.world.segment_bullet_scoped(cell, e, at).is_none();
            (sim.bond.pos, sim.bond.eye, sim.bond.crouch_drop, sim.bond.yaw, sim.bond.pitch, at, flat, scoped)
        };
        if flat { flat_clear += 1; }
        if scoped {
            scoped_clear += 1;
            if first_scoped.is_none() { first_scoped = Some((tick, bp[1])); }
        }
        // EVERY alive drone this tick: any level/across ray that truly clears?
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let cell = sim.world.find_cel(e);
            for (j, dr) in sim.drones.list.iter().enumerate() {
                if !dr.present || dr.corpse { continue; }
                if sim.world.segment_bullet_scoped(cell, e, dr.pos).is_none() {
                    *per_drone_scoped.entry(j).or_insert(0) += 1;
                    if sim.world.segment_bullet(e, dr.pos).is_none() && any_flat_clear.is_none() {
                        any_flat_clear = Some((tick, j, dr.pos));
                    }
                }
            }
        }
        if !scoped && diag_printed < 8 && tick % 60 == 0 {
            // which half blocks, at this pose
            let sim = r.w.scene.sim.as_ref().unwrap();
            let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let cell = sim.world.find_cel(e);
            let at = sim.drones.list[di].pos;
            let (flood, always_n, stat_sc, stat_fl, miss, dyna) =
                sim.world.probe_segment_sets(cell, e, at);
            let pt = |t: Option<f32>| t.map(|tt| [e[0] + (at[0] - e[0]) * tt, e[1] + (at[1] - e[1]) * tt, e[2] + (at[2] - e[2]) * tt]);
            eprintln!("PROBE t={tick} y={:.1} cell={cell:?} flood={flood} always={always_n} stat_scoped={stat_sc:?}@{:?} stat_flat={stat_fl:?} missile={miss:?} dynamic={dyna:?}@{:?} drone={:?}",
                bp[1], pt(stat_sc), pt(dyna), at);
            // pose-grid: does ANY in-car eye pose clear the ray to the drone — and is that
            // clear a REAL opening (the flat query — every triangle — clears too) on the
            // platform (the floor below the pose is the platform surface)?
            let (bond_eye, cd) = (sim.bond.eye, sim.bond.crouch_drop);
            let base_y = bp[1] + bond_eye - cd;
            let mut clear_scoped = 0u32; let mut clear_both = 0u32;
            let mut first_both: Option<[f32; 3]> = None;
            for gx in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0] {
                for gz in [-5.5, -5.0, -4.5, -4.0, -3.4, -3.0, -2.5, -2.0, -1.5, -1.0, -0.5] {
                    let e2 = [gx, base_y, gz];
                    let cell2 = sim.world.find_cel(e2);
                    let sc = sim.world.segment_bullet_scoped(cell2, e2, at).is_none();
                    let fl = sim.world.segment_bullet(e2, at).is_none();
                    if sc { clear_scoped += 1; }
                    if sc && fl {
                        clear_both += 1;
                        if first_both.is_none() { first_both = Some(e2); }
                    }
                }
            }
            let floor_note = first_both.map(|p| {
                let fy = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 6.0, p[2]]);
                (fy.map(|(_t, pt)| pt[1]), fy.map(|(_t, pt)| key_y(r.y0, r.t) - pt[1]))
            });
            eprintln!("PROBE grid@t={tick}: scoped-clear {clear_scoped}/110, flat+scoped {clear_both}, first_both={first_both:?} floor_below={floor_note:?} (car floor at {:.1})", key_y(r.y0, r.t));
            diag_printed += 1;
        }
        // aim at the live drone
        let dx = at_now[0] - bp[0];
        let dz = at_now[2] - bp[2];
        let dy = at_now[1] - (bp[1] + eye_h - crouch);
        let desired = dx.atan2(dz);
        let err = ((desired - dyaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI) as f32;
        let dxz = (dx * dx + dz * dz).sqrt().max(0.5);
        let want_pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - dpitch;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        // hold the trigger on scoped-clear ticks (the game-path predicate) — fire through the
        // real path so the spawned bullet's own scoped flight decides.
        if scoped { fired += 1; r.w.tick_fire(input); } else { r.w.tick(input, false); }
        r.t += 1;
        if r.t > 4400 { break; }
    }
    let sim = r.w.scene.sim.as_ref().unwrap();
    eprintln!("PROBE RESULT: hit_tick={hit_tick:?} drone_hp={}->{} corpse={} | flat_clear={flat_clear} scoped_clear={scoped_clear} fired_ticks={fired} | first_scoped={first_scoped:?} bond_y_end={} drone_pos={:?}",
        hp0, sim.drones.list[di].health, sim.drones.list[di].corpse, sim.bond.pos[1], sim.drones.list[di].pos);
    eprintln!("PROBE any_flat_clear(any drone)={any_flat_clear:?}");
    eprintln!("PROBE per-drone scoped-clear ticks: {:?} (drone y spans the group's spots)",
        per_drone_scoped.iter().map(|(j, n)| (*j, *n)).collect::<Vec<_>>());
    if let Some(sp) = sim.drones.spawners.iter().find(|s| s.placement == 233) {
        eprintln!("PROBE: spawner #233 spawned={} alive={}", sp.spawned.len(),
            sp.spawned.iter().filter(|&&d| sim.drones.list[d].present && !sim.drones.list[d].corpse).count());
    }
}
