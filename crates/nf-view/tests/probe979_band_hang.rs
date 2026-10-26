//! 979 diagnostic: reproduce the band-edge hang deterministically — toggle door #4 CLOSED,
//! press into the slab, and dump vel[1]/fall state per tick to find what blocks the fall.
//! Diagnostic only — run with `--ignored --nocapture`.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "diagnostic dump"]
fn probe_band_hang() {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load warm"));
    // FORCED-STATE repro of the 941 hang: the exact band-fail state (dy = 1.1834, the floor
    // seen 1.1834 below, the band edge cc + 0.1 = 1.1347) — does the port's fall recovery sink
    // him the 4.9 cm back into the band?
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-17.87, 1.1834, 4.87];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = -std::f32::consts::FRAC_PI_2;
        sim.bond.grounded = false;
        sim.bond.jump = 0;
        sim.bond.land_delay = 0;
        sim.bond.fall_timer = 0;
    }
    w.wait(4);
    let mut ungt = 0u32;
    for t in 0..240u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if !sim.bond.grounded { ungt += 1; } else { ungt = 0; }
            if t < 30 || ungt > 0 || t % 20 == 0 {
                let flat = sim.world.floor_below(p, 3.0);
                let dy = flat.map(|f| p[1] - f);
                eprintln!("t{:3} pos ({:.6},{:.6},{:.6}) g={} ungt{} vel1 {:+.6} jump {} landd {} fall_t {} dy {:?} cel {:?} eye {:.6} cdrop {:.6} local {:?}",
                    t, p[0], p[1], p[2], sim.bond.grounded, ungt, sim.bond.vel[1], sim.bond.jump, sim.bond.land_delay, sim.bond.fall_timer, dy, sim.bond.cel, sim.bond.eye, sim.bond.crouch_drop, sim.bond.local);
            }
        }
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
        if ungt > 150 { eprintln!("[hang] still ungrounded at t{t} — the recovery never re-grounds"); break; }
    }
    // Per-tick push dump: 40 ticks, the push recomputed after each tick.
    for t in 0..40u32 {
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let bond = &sim.bond;
        let (a, b) = bond.capsule_seg();
        let (push, walkable) = sim.world.capsule_push_scoped(bond.cel, a, b, nf_game::player::RADIUS, false, [1.0e9, 1.0e9, 1.0e9]);
        let (push2, _) = sim.world.capsule_push(a, b, nf_game::player::RADIUS, false, [1.0e9, 1.0e9, 1.0e9]);
        eprintln!("p{:2} y {:.5} g={} push_scoped ({:+.5},{:+.5},{:+.5}) w{} push_flat ({:+.5},{:+.5},{:+.5})",
            t, bond.pos[1], bond.grounded, push[0], push[1], push[2], walkable, push2[0], push2[1], push2[2]);
    }
}
#[test]
#[ignore = "diagnostic dump"]
fn probe_mouth_crossing() {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000a.bin", false, Some(&save)).expect("load warm"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-11.01, 1.12, -18.60];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::FRAC_PI_2; // face +x (east)
        sim.bond.grounded = true;
    }
    w.wait(5);
    let to = [-9.88f32, -18.62];
    for t in 0..240u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if t % 5 == 0 || !sim.bond.grounded {
                let flat = sim.world.floor_below(p, 3.0);
                eprintln!("t{:3} pos ({:.4},{:.4},{:.4}) g={} vel1 {:+.5} dy {:?} yaw {:.2}",
                    t, p[0], p[1], p[2], sim.bond.grounded, sim.bond.vel[1], flat.map(|f| p[1] - f), sim.bond.yaw);
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let dx = to[0] - p[0];
        let dz = to[1] - p[2];
        if dx * dx + dz * dz < 0.5 { eprintln!("[arrived] t{t}"); return; }
        let desired = dx.atan2(dz);
        let mut err = desired - sim.bond.yaw;
        while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
        while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    eprintln!("[never arrived]");
}
