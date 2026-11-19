//! 07000001: the door and interactable colour entries stay as built through kills, dropped guns and
//! collected pickups (a door once drew black after a pickup route; the palette overflow fixed by the
//! section-bounded writes). Opt-in via NF_ISO.
#[path = "common/walker_b2.rs"]
mod walker;
use walker::Walker;

#[test]
fn the_doors_keep_their_colour_after_kills_and_pickups() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000001.bin", false, None).unwrap());
    for _ in 0..4000 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if !sim.disabled { break; }
    }
    for _ in 0..60 { w.tick(Default::default(), false); }
    let (layout, _) = w.scene.skin_layout();
    let doors = |s: &nf_view::scene::Scene| {
        let (_, cols) = s.skin_palette(&layout, None, 0.0, nf_view::scene::ViewOpts::default(), None);
        cols[layout.col_door as usize..=layout.col_one as usize].to_vec()
    };
    let c0 = doors(&w.scene);
    eprintln!("layout pick {} sl {} door {} one {}; pickups {}", layout.col_pick, layout.col_sl, layout.col_door, layout.col_one, w.scene.sim.as_ref().unwrap().drones.pickups.len());
    for k in 0..3 {
        let di = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len())
                .filter(|&i| { let d = &sim.drones.list[i]; d.present && d.enabled && !d.corpse && d.health > 0.0 })
                .min_by(|&a, &b| {
                    let f = |p: [f32; 3]| (p[0] - bp[0]).powi(2) + (p[2] - bp[2]).powi(2);
                    f(sim.drones.list[a].pos).partial_cmp(&f(sim.drones.list[b].pos)).unwrap()
                })
        };
        let Some(di) = di else { break };
        { let sim = w.scene.sim.as_mut().unwrap(); let dp = sim.drones.list[di].pos; sim.bond.pos = [dp[0] - 3.0, dp[1], dp[2]]; }
        let killed = w.shoot(di, "guard");
        for _ in 0..90 { w.tick(Default::default(), false); }
        eprintln!("kill {k}: {killed}; pickups {}", w.scene.sim.as_ref().unwrap().drones.pickups.len());
    }
    // Collect every pickup by standing on it.
    let n = w.scene.sim.as_ref().unwrap().drones.pickups.len();
    for k in 0..n {
        let p = { let q = &w.scene.sim.as_ref().unwrap().drones.pickups[k]; if q.gone || q.hidden { continue } [q.m[3][0], q.m[3][1], q.m[3][2]] };
        { let sim = w.scene.sim.as_mut().unwrap(); sim.bond.pos = p; }
        for _ in 0..20 { w.tick(Default::default(), false); }
    }
    let gone = w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|q| q.gone).count();
    let c1 = doors(&w.scene);
    eprintln!("collected {gone}/{n}");
    for (i, (a, b)) in c0.iter().zip(c1.iter()).enumerate() { if a != b { eprintln!("slot {i}: {a:?} -> {b:?}"); } }
    assert_eq!(c0, c1, "door colours unchanged");
}
