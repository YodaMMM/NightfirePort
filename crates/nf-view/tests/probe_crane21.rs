//! probe21: the boarding question (level-0d-boarding). The yard catwalk (z −283..−292,
//! x 137..193 — probe20 fine map A read the beam 14.32 under it at rest) over the sweeping
//! gantry: its floor heights and holes, the access from the bay, the beam's live collision
//! under it, and an env-parameterised boarding attempt. Print-only.
//!   P21_X / P21_Z / P21_YAW / P21_TICKS: the boarding attempt's stand point + hold.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

fn envf(name: &str, d: f32) -> f32 { std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(d) }

fn load_real() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    w
}

/// Multi-hit downward column, top → bot, up to n hits.
fn col(sim: &nf_view::scene::Sim, x: f32, z: f32, top: f32, bot: f32, n: usize) -> Vec<f32> {
    let mut hits = Vec::new();
    let mut t = top;
    for _ in 0..n {
        match sim.world.segment_hit([x, t, z], [x, bot, z]) {
            Some((f, _)) => { let y = t - (t - bot) * f; hits.push(y); t = y - 0.05; }
            None => break,
        }
    }
    hits
}

fn beam_ox(sim: &nf_view::scene::Sim) -> f32 {
    sim.drones.scripts.entity_matrix((0x0600_09ef, 1)).map(|m| m[3][0]).unwrap_or(f32::NAN)
}

/// A: the yard catwalk band (z −282..−293) and the connector region, multi-hit columns.
#[test]
#[ignore = "probe21: the catwalk map; run with --ignored --nocapture"]
fn probe21_catwalk_map() {
    let mut w = load_real();
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    // A1. The catwalk band: x 133..194 step 1, z −282..−293 step 1, from y 18.5 down, 5 hits.
    let mut z = -282.0f32;
    while z >= -293.0 {
        let mut row = String::new();
        let mut x = 133.0f32;
        while x <= 194.0 {
            let c = col(&sim, x, z, 18.5, 0.0, 5);
            let s: Vec<String> = c.iter().map(|y| format!("{y:.2}")).collect();
            row.push_str(&format!("x{x:.0}[{}] ", s.join(" ")));
            x += 1.0;
        }
        eprintln!("[p21] cat z{z:.0}: {row}");
        z -= 1.0;
    }
    // A2. The connector region between the mezzanine (z −272) and the catwalk (z −283):
    //     x 160..190 step 1, z −273..−282 step 1.
    let mut z = -273.0f32;
    while z >= -282.5 {
        let mut row = String::new();
        let mut x = 160.0f32;
        while x <= 190.0 {
            let c = col(&sim, x, z, 18.5, 0.0, 5);
            let s: Vec<String> = c.iter().map(|y| format!("{y:.2}")).collect();
            row.push_str(&format!("x{x:.0}[{}] ", s.join(" ")));
            x += 1.0;
        }
        eprintln!("[p21] con z{z:.0}: {row}");
        z -= 1.0;
    }
    // A3. The bay interior columns 9.0 → 18.5 (the steps up): x 184..198 step 1, z −266..−292
    //     step 2, multi-hit from 9.2 (above the bay floor) up as well as down from 18.5.
    let mut z = -266.0f32;
    while z >= -292.0 {
        let mut row = String::new();
        let mut x = 184.0f32;
        while x <= 198.0 {
            let up = col(&sim, x, z, 9.2, 18.5, 6);
            let s: Vec<String> = up.iter().map(|y| format!("{y:.2}")).collect();
            row.push_str(&format!("x{x:.0}[{}] ", s.join(" ")));
            x += 1.0;
        }
        eprintln!("[p21] bay z{z:.0}: {row}");
        z -= 2.0;
    }
    // A4. Horizontal body-band rays west from the bay: can a body at y 13..17 cross the
    //     bay's west wall (x 184) at z −283..−292?
    for y in [13.0f32, 14.0, 15.0, 16.0, 16.5] {
        for z in [-283.0f32, -286.0, -289.0, -292.0] {
            let e = sim.world.segment([184.5, y, z], [176.0, y, z]);
            let wst = sim.world.segment([176.0, y, z], [184.5, y, z]);
            eprintln!("[p21] wall y{y} z{z}: bay→yard {e:?} yard→bay {wst:?}");
        }
    }
}

/// B: the beam's live collision under the catwalk band — E #460 from the bay, sample.
#[test]
#[ignore = "probe21: the sweep under the catwalk; run with --ignored --nocapture"]
fn probe21_sweep() {
    let mut w = load_real();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [189.0, 10.6, -287.6];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    let used = w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E #460");
    eprintln!("[p21] E460 {used} ch0x2b {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b));
    // Sample: the beam origin + the top readings under the band at fixed x columns
    // (137/139/141 over the track, 150/165/175 further east) at z −287 (the band middle).
    // Ray from y 16.4 (under the catwalk floor if it is ~16.8) down to 8 — only the beam can
    // be in that window (the yard is 4.0).
    let probes = [137.0f32, 139.0, 141.0, 150.0, 165.0, 175.0];
    for k in 0..450u32 {
        w.wait(8);
        let sim = w.scene.sim.as_ref().unwrap();
        let ox = beam_ox(&sim);
        let mut row = String::new();
        for x in probes {
            let top = sim.world.segment_hit([x, 16.4, -287.0], [x, 8.0, -287.0]).map(|(f, _)| 16.4 - 8.4 * f);
            row.push_str(&format!("x{x:.0}:{top:>5.2?} "));
        }
        eprintln!("[p21] t{:>5} ox {ox:6.2} {}", (k + 1) * 8, row);
    }
}

/// D: the roof over the beam track — columns from y 22 down (multi-hit), the canopy's extent
/// and holes, plus the main-roof connection band (z −266..−282).
#[test]
#[ignore = "probe21: the roof map; run with --ignored --nocapture"]
fn probe21_roof_map() {
    let mut w = load_real();
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    // D1. Over the track: x 130..190 step 1, z −270..−305 step 2.5, from 22 down, 4 hits.
    let mut z = -270.0f32;
    while z >= -305.0 {
        let mut row = String::new();
        let mut x = 130.0f32;
        while x <= 190.0 {
            let c = col(&sim, x, z, 22.0, 0.0, 4);
            let s: Vec<String> = c.iter().map(|y| format!("{y:.1}")).collect();
            row.push_str(&format!("x{x:.0}[{}] ", s.join(" ")));
            x += 1.0;
        }
        eprintln!("[p21] roof z{z:.0}: {row}");
        z -= 2.5;
    }
    // D2. The main-roof connection: x 150..190 step 1, z −264..−282 step 1, from 22 down.
    let mut z = -264.0f32;
    while z >= -282.0 {
        let mut row = String::new();
        let mut x = 150.0f32;
        while x <= 190.0 {
            let c = col(&sim, x, z, 22.0, 0.0, 4);
            let s: Vec<String> = c.iter().map(|y| format!("{y:.1}")).collect();
            row.push_str(&format!("x{x:.0}[{}] ", s.join(" ")));
            x += 1.0;
        }
        eprintln!("[p21] link z{z:.0}: {row}");
        z -= 1.0;
    }
    // D3. The main roof's west/south rim at walk height: horizontal rays at y 19.6 (the walkreal
    //     walks the roof at pos ~19.65) — where is the roof edge open southward?
    for z in [-269.0f32, -272.0, -276.0, -280.0, -284.0, -288.0] {
        let mut row = String::new();
        for x in [158.0f32, 165.0, 172.0, 179.0, 186.0] {
            let hit = sim.world.segment([x, 19.65, -269.5], [x, 19.65, z]);
            let back = sim.world.segment([x, 19.65, z], [x, 19.65, -269.5]);
            row.push_str(&format!("x{x:.0} s{hit:?}/n{back:?} "));
        }
        eprintln!("[p21] rim z{z:.0}: {row}");
    }
}

/// C: the boarding attempt — stand on the catwalk at (P21_X, P21_Z), face P21_YAW, hold
/// forward P21_TICKS while the beam passes; print every 4th tick and the landing.
#[test]
#[ignore = "probe21: the boarding attempt; run with --ignored --nocapture"]
fn probe21_board() {
    let x0 = envf("P21_X", 139.0);
    let z0 = envf("P21_Z", -287.2);
    let yaw = envf("P21_YAW", std::f32::consts::FRAC_PI_2);
    let hold = envf("P21_TICKS", 240.0) as u32;
    let mut w = load_real();
    // The catwalk stand height: read the floor at (x0, z0) first.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let c = col(&sim, x0, z0, 18.5, 0.0, 4);
        eprintln!("[p21] stand col at ({x0},{z0}): {c:?}");
    }
    let floor = {
        let sim = w.scene.sim.as_ref().unwrap();
        col(&sim, x0, z0, 18.5, 0.0, 4).first().copied().unwrap_or(16.8)
    };
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [x0, floor + 1.05, z0];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = yaw;
    }
    w.wait(10);
    // Hold forward, printing every 4th tick; stop on a settled landing below the stand.
    let mut landed = false;
    for k in 0..hold {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
        }
        w.total_ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let ox = beam_ox(&sim);
        if k % 4 == 0 {
            let top = sim.world.segment_hit([p[0], 16.4, p[2]], [p[0], 8.0, p[2]]).map(|(f, _)| 16.4 - 8.4 * f);
            eprintln!("[p21] b{:>4} pos ({:.2},{:.2},{:.2}) g{} ox {ox:6.2} top {top:5.2?}", k, p[0], p[1], p[2], sim.bond.grounded as u8);
        }
        if sim.bond.grounded && k > 30 && p[1] < floor - 0.5 { landed = true; break; }
    }
    eprintln!("[p21] board ended at {:?} grounded {} landed {landed} ox {:.2}",
        pos(&w), w.scene.sim.as_ref().unwrap().bond.grounded, beam_ox(&w.scene.sim.as_ref().unwrap()));
    // If landed on the beam: keep ticking (no input) and watch the carry for 400 ticks.
    if landed {
        for k in 0..100u32 {
            w.wait(4);
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let ox = beam_ox(&sim);
            if k % 5 == 0 { eprintln!("[p21] ride {:>4} pos ({:.2},{:.2},{:.2}) g{} ox {ox:6.2}", (k + 1) * 4, p[0], p[1], p[2], sim.bond.grounded as u8); }
        }
    }
}
