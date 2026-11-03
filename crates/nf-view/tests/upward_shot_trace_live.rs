//! player-upward-shot: the DIAGNOSTIC TRACE (a measurement-spot teleport, documented per the
//! ladder495 convention — the pinned real-input test is separate). Subject: the aimed weapon-0x1e
//! round fired UP from the 0700000c yard floor at the deck sentry ~35 u away — three clips dry
//! on the real-health branch while the eye-ray predictor says clear [L the yard_sniper_stands
//! leg]. Aims exactly like `DamageLog::stand_and_fire` (the exposed box centre through the view),
//! then walks the flight tick by tick: position, `find_cel` scope seed, the scoped and the flat
//! segment picks, every impact — so the stop is named and compared with the game's pick law
//! [M Bullet_update 0x12e7b0 + Collide_Pick 0x1eb320 + control_handle_cel_change 0x133288].
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test upward_shot_trace_live -- --nocapture`

use nf_game::player::Input;
use walker::DamageLog;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[path = "common/walker.rs"]
mod walker;

#[test]
fn trace_the_upward_shot_flight() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700000c.bin", false).expect("load 0700000c");
    nf_view::settle_start_draw(scene.sim.as_mut().unwrap());
    {
        let sim = scene.sim.as_mut().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.weapon == 0x1e && d.pos[1] > 6.0 {
                eprintln!("[census] drone {i} skin {:#x} hp {:.1} pos {:?}", d.skin, d.health, d.pos);
            }
        }
    }
    let target_idx = {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.list.iter().enumerate()
            .find(|(_, d)| d.weapon == 0x1e && (d.pos[0] - 36.4).abs() < 2.5 && (d.pos[2] + 208.8).abs() < 3.5)
            .map(|(i, _)| i)
            .expect("the deck sentry (weapon 0x1e at ~(36.4, 12.1, -208.8) [L the real-health run's yard census])")
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        // The pre-climb stand spot from the real-health run [L the yard_sniper_stands leg]: the
        // ladder base, the yard floor 0.69-1.2, the deck snipers 10.7-11 u above.
        sim.bond.pos = [11.6, 1.2, -186.6];
        sim.bond.vel = [0.0; 3];
        sim.gun.weapon = 0x1e;
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 8;
        sim.gun.pending = 0;
        let aim = DamageLog::exposed_part(sim, target_idx);
        let aim_gun = DamageLog::exposed_part_gun_rays(sim, target_idx);
        eprintln!("[predictor] exposed_part {aim:?} gun_rays {aim_gun:?}");
        let at = aim.expect("the stand's own predictor must expose the sentry (the run's premise)");
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
        let h = (dx * dx + dz * dz).sqrt();
        let l = (dx * dx + dy * dy + dz * dz).sqrt();
        let dir = [dx / l, dy / l, dz / l];
        sim.bond.yaw = dx.atan2(dz);
        sim.bond.pitch = (dy / h.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        eprintln!("[aim] eye {eye:?} at {at:?} d {l:.2} yaw {:.3} pitch {:.3}", sim.bond.yaw, sim.bond.pitch);
        // The scoped pick over the whole aim lane, vs the flat world.
        let end = [eye[0] + dir[0] * 60.0, eye[1] + dir[1] * 60.0, eye[2] + dir[2] * 60.0];
        let cel = sim.world.find_cel(eye);
        eprintln!("[lane] eye cel {cel:?} scoped t {:?} flat t {:?}",
            sim.world.segment_bullet_scoped(cel, eye, end), sim.world.segment_bullet(eye, end));
    }
    let mut fired = false;
    for t in 0..40u32 {
        let mut input = Input::default();
        if !fired { input.fire = true; fired = true; }
        scene.sim.as_mut().unwrap().step_bond(input, t == 0, false, false);
        let sim = scene.sim.as_mut().unwrap();
        for im in &sim.drones.player_impacts {
            eprintln!("[t{t}] impact {im:?}");
        }
        sim.drones.player_impacts.clear();
        for (bi, b) in sim.drones.player_bullets.iter().enumerate() {
            let cel = sim.world.find_cel(b.pos);
            let step = b.speed;
            let end = [b.pos[0] + b.dir[0] * step, b.pos[1] + b.dir[1] * step, b.pos[2] + b.dir[2] * step];
            let sc = sim.world.segment_bullet_scoped(cel, b.pos, end);
            let fl = sim.world.segment_bullet(b.pos, end);
            eprintln!("[t{t} b{bi}] pos {:?} trav {:.1} cel {cel:?} scoped {sc:?} flat {fl:?}",
                b.pos, b.travelled);
        }
        if sim.drones.player_bullets.is_empty() && t > 0 { eprintln!("[t{t}] bullets gone"); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[outcome] drone {target_idx} hp {:.1} (started 10.0)", sim.drones.list[target_idx].health);
}
