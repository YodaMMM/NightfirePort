//! level-1b-platform-touch repro probe: walk Bond (real inputs only) onto solar-panel
//! placement #41 (script 0x06000832, hp 20) and watch the panel SPs' hold/damage/fragments;
//! then fire at the panel (the game's break action: bullet contacts, threshold 20
//! [M SP_Update 0x1dbda8:236-267]). Opt-in NF_ISO; prints only.

use nf_game::player::Input;

#[path = "common/walker.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn panel_pcs(sim: &nf_view::scene::Sim) -> Vec<(usize, u32, i32, f32, usize, usize)> {
    // (placement, cur script, delay, damage, activated?, object count with src 0x06000832)
    let objs = sim.drones.scripts.objects.list.iter().flatten()
        .filter(|o| matches!(o, nf_game::nis::Obj::Entity { src: (h, _), .. } if *h == 0x0600_0832))
        .count();
    sim.drones.scripts.players.iter()
        .filter(|p| (41..=56).contains(&p.placement))
        .map(|p| (p.placement, p.scripts[p.cur], p.info[p.cur].as_ref().map_or(1, |s| s.delay),
            p.damage, p.activated as usize, objs))
        .collect()
}

fn frac(sim: &nf_view::scene::Sim, t: u32) {
    let pcs = panel_pcs(sim);
    let held = pcs.iter().filter(|(_, _, d, _, _, _)| *d != 0).count();
    let dmg: Vec<String> = pcs.iter().filter(|(_, _, _, dm, _, _)| *dm != 0.0)
        .map(|(p, _, _, dm, _, _)| format!("{p}:{dm}")).collect();
    println!("t{t} bond=({:.2},{:.2},{:.2}) panels-held {held} dmg {dmg:?} frags {}",
        sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], pcs[0].5);
}

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_1b_walk_on_panel() {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "0700001b.bin", false, None, 3).expect("load 1b");
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    println!("after arrival: bond=({:.2},{:.2},{:.2})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    frac(sim, 0);
    drop(sim);

    // Walk onto panel #50 @(16.71,-2.88,4.62) — the nearest to the spawn (22.98,-1.90,3.13):
    // face it (the walker's convention: desired = atan2(dx, dz)), walk forward, then step past
    // its centre so Bond stands ON the panel.
    let target = [16.71f32, -2.88, 4.62];
    let mut t = 0u32;
    while t < 1200 {
        let sim = scene.sim.as_mut().unwrap();
        let here = sim.bond.pos;
        let dist = ((target[0] - here[0]).powi(2) + (target[2] - here[2]).powi(2)).sqrt();
        if dist < 0.6 { break; }
        let (turn, look, fwd) = if let Some((ye, pe)) = walker::zero_g_errs(sim, target) {
            (ye.clamp(-0.08, 0.08), pe.clamp(-0.08, 0.08), if ye.abs() > 1.0 { 0.0 } else { 1.0 })
        } else {
            let want = (target[0] - here[0]).atan2(target[2] - here[2]);
            let mut err = want - sim.bond.yaw;
            while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
            while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
            (err.clamp(-0.08, 0.08), 0.0, if err.abs() > 1.0 { 0.0 } else { 1.0 })
        };
        let input = Input { forward: fwd, turn, look, ..Default::default() };
        sim.step_bond(input, false, false, false);
        t += 1;
        if t % 100 == 0 { frac(scene.sim.as_ref().unwrap(), t); }
        if scene.sim.as_ref().unwrap().disabled { println!("DISABLED at t{t}"); return; }
    }
    println!("--- reached panel (t={t}) ---");
    frac(scene.sim.as_ref().unwrap(), t);

    // Stand on it a while (idle + small steps).
    for i in 0..600 {
        let sim = scene.sim.as_mut().unwrap();
        let input = if i % 60 < 30 { Input { forward: 0.3, ..Default::default() } } else { Input { forward: -0.3, ..Default::default() } };
        sim.step_bond(input, false, false, false);
        if i % 100 == 0 { frac(scene.sim.as_ref().unwrap(), 2000 + i); }
    }
    println!("--- now fire DOWN at the panel Bond stands on (the game's break action) ---");
    // Switch to the start weapon 0x33 (51, damage 20 [D weapon_data]) first, then aim + fire
    // straight down: the muzzle is above the feet, the panel's own sphere is right below.
    for i in 0..900 {
        let sim = scene.sim.as_mut().unwrap();
        let input = if i < 30 {
            Input { switch_to: Some(51), ..Default::default() }
        } else {
            Input { aiming: true, ..Default::default() }
        };
        let fire = i >= 60 && i % 24 < 8;
        sim.step_bond(input, fire, false, false);
        if i % 60 == 0 {
            let sim = scene.sim.as_ref().unwrap();
            let nb = sim.drones.player_bullets.len();
            let b0 = sim.drones.player_bullets.first()
                .map(|b| format!("pos {:?} dir {:?}", [b.pos[0],b.pos[1],b.pos[2]], b.dir));
            println!("  fire i{i} bullets {nb} first {b0:?}");
            frac(sim, 3000 + i);
        }
    }
    frac(scene.sim.as_ref().unwrap(), 3600);
}
