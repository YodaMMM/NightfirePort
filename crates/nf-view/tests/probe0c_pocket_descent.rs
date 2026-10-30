//! PROBE (level-0c-plates-reach-2): the pocket→yard descent candidates, walked in the sim with
//! real input. The prior task's scans stopped at r 2.5 and misread the pocket's west edge as
//! x −1.8; the wider scans (this session, [D] coll-scan 0700000c.bin) show the 2.37 floor runs
//! west to x −8.25 (z −153.76..−143.11), the 6.27 display floor runs west to x −4.50, and a
//! `room_sk3` ramp rises from the y≈0 yard floor (x −20.63..−18.03) to 4.52..4.83 decks
//! (x −18.03..−9.98, z −129.36..−156.81), with a 6.92 walkway top (x −10.03..−5.65,
//! z −155.02..−110.36) above them. This probe walks every candidate transition and logs where
//! the body actually goes. #[ignore]d: a route record, not a pin.
use walker::{Walker, Wp};

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn log_pos(w: &Walker, label: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[desc] {label}: grounded {} pos ({:.2}, {:.2}, {:.2})", sim.bond.grounded,
        sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

/// Raw forward ticks toward a target xz (the walker will not cross a floor edge by itself).
fn raw_walk(w: &mut Walker, tx: f32, tz: f32, max_ticks: u32, stop: &dyn Fn(&nf_view::scene::Sim) -> bool) {
    for _ in 0..max_ticks {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let yaw = (tx - sim.bond.pos[0]).atan2(tz - sim.bond.pos[2]);
            let err = walker::wrap_pi(yaw - sim.bond.yaw);
            drop(sim);
            w.tick(nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.4 { 0.3 } else { 1.0 },
                ..Default::default()
            }, false);
        }
        if stop(w.scene.sim.as_ref().unwrap()) { break; }
    }
}

/// A raw jump-off: face the target (turn until aligned), run forward, jump at tick 6, keep
/// forward; log the landing.
fn jump_off(w: &mut Walker, tx: f32, tz: f32, max_ticks: u32, label: &str) {
    for _ in 0..60 {
        let sim = w.scene.sim.as_ref().unwrap();
        let yaw = (tx - sim.bond.pos[0]).atan2(tz - sim.bond.pos[2]);
        let err = walker::wrap_pi(yaw - sim.bond.yaw);
        drop(sim);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
        if err.abs() < 0.04 { break; }
    }
    let mut landed = None;
    for i in 0..max_ticks {
        w.tick(nf_game::player::Input {
            forward: 1.0,
            jump: i == 6,
            ..Default::default()
        }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if i > 30 && sim.bond.grounded {
            landed = Some((sim.bond.pos, sim.bond.fall_damage));
            break;
        }
    }
    match landed {
        Some((p, d)) => eprintln!("[desc] {label}: LANDED ({:.2}, {:.2}, {:.2}) fall_damage {d:?}",
            p[0], p[1], p[2]),
        None => log_pos(w, &format!("{label}: never re-grounded in {max_ticks} ticks")),
    }
}

#[test]
#[ignore = "route record: the pocket→yard descent candidates, walked and logged"]
fn probe_the_pocket_to_yard_descent_candidates() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    w.wait(30);
    // A. The pocket, walked west: the 2.37 floor's west extension (the prior scans' blind spot).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [1.2, 3.5, -153.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "A0 placed on the pocket (2.37)");
    w.walk_to(Wp::new(-2.5, 3.5, -153.0).r(1.0), "A1 the pocket west 1 (x −2.5)");
    log_pos(&w, "A1 on the pocket west");
    w.walk_to(Wp::new(-6.9, 3.5, -150.5).r(1.0), "A2 the pocket west 2 (x −6.9, the west strip)");
    log_pos(&w, "A2 on the 2.37 west strip");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[0] < -5.0,
            "the 2.37 floor's west extension must carry the walk past x −5, got {:?}", sim.bond.pos);
    }

    // B. The west edge (x −8.25): walk into it, then the jump-off west.
    raw_walk(&mut w, -8.8, -150.5, 120, &|sim| !sim.bond.grounded);
    log_pos(&w, "B0 pushed against the x −8.25 west wall");
    jump_off(&mut w, -9.5, -150.5, 400, "B1 the west-edge jump-off (x −8.25 → west)");

    // C. The south edges of the 2.37 west strips: the z −153.76 wall (x −8.25..−5.57) and the
    //    z −155.77 slab face (x −5.03..15.25). Wedge expected; then the jump-offs.
    w.walk_to(Wp::new(-6.9, 3.5, -153.2).r(0.8), "C0 back to the west strip's south edge");
    raw_walk(&mut w, -6.9, -154.4, 120, &|sim| !sim.bond.grounded);
    log_pos(&w, "C1 pushed against the z −153.76 wall");
    jump_off(&mut w, -6.9, -155.0, 400, "C2 the z −153.76 wall-top jump-off (south)");
    w.walk_to(Wp::new(-4.7, 3.5, -155.2).r(0.8), "C3 the pocket's south-west (x −4.7)");
    raw_walk(&mut w, -4.7, -156.2, 120, &|sim| !sim.bond.grounded);
    log_pos(&w, "C4 pushed against the z −155.77 slab face at x −4.7");
    jump_off(&mut w, -4.7, -156.6, 400, "C5 the slab-face jump-off (south) at x −4.7");

    // D. The 6.92 walkway's SOUTH part (room_sk3, x −10.03..−5.65, z −155.02..−148.77 — the
    //    cross-wall at z −148.77..−147.86 blocks the north part [D scan_rampne]): its south end
    //    and its WEST edge (x −10.03, over the ramp decks 4.52..4.83 [D]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-8.0, 8.1, -152.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "D0 placed on the 6.92 walkway's south part");
    if !w.try_walk_to(Wp::new(-8.0, 8.1, -154.0).r(0.9), "D1 the walkway south (z −154)") {
        eprintln!("[desc] D1 wedged (the walkway's south end?)");
    }
    log_pos(&w, "D1 at the walkway's south end");
    raw_walk(&mut w, -8.0, -155.8, 120, &|sim| !sim.bond.grounded);
    log_pos(&w, "D2 pushed off/into the walkway's south end (z −155.02 face?)");
    jump_off(&mut w, -8.0, -156.5, 400, "D3 the walkway south-end jump-off");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-9.5, 8.1, -152.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    jump_off(&mut w, -11.5, -152.0, 400, "D4 the walkway WEST jump-off (x −10.03 → the ramp 4.52..4.83 below)");

    // E. The display-ground side hops: 6.27 → the 7.24 strips (x −5.05..−2.93) → the tower's
    //    6.69 top (x −3.19..2.17, z −158.5..−159.74) → the yard; and 6.27 → the 6.92 walkway.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-3.2, 7.4, -153.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "E0 placed on the 6.27 floor's west part");
    jump_off(&mut w, -6.0, -153.2, 400, "E1 the 6.27→west jump-off (toward the 7.24/6.92 complex)");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-4.7, 8.4, -152.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "E2 placed on the 7.24 strip");
    if !w.try_walk_to(Wp::new(-4.7, 8.4, -156.2).r(0.9), "E3 the 7.24 strip south") {
        eprintln!("[desc] E3 wedged on the 7.24 strip");
    }
    log_pos(&w, "E3 on the 7.24 strip's south part");
    jump_off(&mut w, -4.7, -158.5, 400, "E4 the 7.24 strip south jump-off (onto the tower's 6.69 top?)");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-2.8, 7.8, -159.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "E5 placed on the tower's 6.69 top");
    jump_off(&mut w, -6.0, -159.0, 400, "E6 the tower-top WEST jump-off (onto the 6.92 walkway?)");
    jump_off(&mut w, -2.8, -162.5, 400, "E7 the tower-top SOUTH jump-off (onto the y≈0 ground south of the tower?)");

    // G. The notch entry: the tower's open shaft (x −2.40..−0.34, z −158.5..−161.11, floor y≈0
    //    [D scan_pocketvoid up faces]) hangs just south of the 6.27 display floor. A jump due
    //    south from the 6.27 floor at the notch's x range should clear the slab face (top 6.15
    //    at z −155.77 [D]) and drop into the shaft — the descent, no teleport.
    for (x0, label) in [(-1.2f32, "G1 the notch jump due south (x −1.2)"),
                        (-2.0, "G2 the notch jump due south (x −2.0)"),
                        (-0.7, "G4 the notch jump due south (x −0.7)")] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [x0, 7.4, -154.5];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        log_pos(&w, &format!("{label} placed on the 6.27 floor"));
        jump_off(&mut w, x0, -159.8, 400, label);
    }
    // G3: a plain walk-off (no jump) south off the 6.27 floor at the notch's x range.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-1.2, 7.4, -154.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    raw_walk(&mut w, -1.2, -160.0, 240, &|sim| sim.bond.grounded && sim.bond.pos[1] < 3.0);
    log_pos(&w, "G3 the plain walk-off south at x −1.2 (into the notch?)");
    // H. The tower's south edge: from the east strip walk south off z −159.74 (onto the y≈0
    //    ground south of the tower — the yard).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [0.5, 7.8, -159.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "H0 placed on the tower's east strip");
    raw_walk(&mut w, 0.5, -162.5, 200, &|sim| sim.bond.grounded && sim.bond.pos[1] < 3.0);
    log_pos(&w, "H1 walked south off the tower's east strip (the yard?)");

    // I. The 7.24 strip's south edge (z −156.72 [D scan_midgap]): place ON the strip's south
    //    part and walk south (no jump) — does the strip's south edge drop onto the tower's
    //    6.69 top (z −158.5 [D scan_topbox])?
    for (x0, label) in [(-4.4f32, "I1 the strip walk-off south (x −4.4)"),
                        (-2.6, "I2 the strip walk-off south (x −2.6)")] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [x0, 8.35, -156.3];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        log_pos(&w, &format!("{label} placed on the strip's south part"));
        raw_walk(&mut w, x0, -162.0, 240, &|sim| sim.bond.grounded && sim.bond.pos[1] < 7.6 && sim.bond.pos[1] > 5.0);
        log_pos(&w, &format!("{label} result"));
    }
    // I3: the standing jump from the strip's south edge, due south (onto the tower top?).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-4.4, 8.35, -156.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    jump_off(&mut w, -4.4, -159.5, 400, "I3 the strip south-edge standing jump (onto the tower top?)");

    // J. The 7.24 cover over the notch (x −2.40..−0.34 — the G jumps landed on it at z −156.17):
    //    walk it south to its end — the notch's open shaft or the tower's south face beyond.
    for (x0, label) in [(-1.2f32, "J1 the notch-cover walk south (x −1.2)"),
                        (-2.0, "J2 the notch-cover walk south (x −2.0)"),
                        (-0.6, "J3 the notch-cover walk south (x −0.6)")] {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [x0, 8.35, -156.4];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(20);
        log_pos(&w, &format!("{label} placed on the cover"));
        raw_walk(&mut w, x0, -165.0, 260, &|sim| sim.bond.grounded && sim.bond.pos[1] < 7.0);
        log_pos(&w, &format!("{label} result"));
    }

    // K. The remaining candidates: (K1) the 6.27 floor → the tower's east top strip (6.69,
    //    x −0.34..2.17, z −158.5..−159.74 [D scan_topbox]) — a standing jump due south off the
    //    walkway04 strip (6.27) clears the slab face top (6.15 [D]) and lands 0.42 higher-floor
    //    — the E1-class jump (which rose 0.97 over 2.6 z). (K2) the cross-wall gap on the 6.92
    //    walkway (the wall spans x −8.17..−5.65 [D scan_rampne]; x −10.03..−8.17 open).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [1.5, 7.4, -155.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "K4 placed on the walkway04 strip's south edge (x 1.5)");
    jump_off(&mut w, 1.2, -159.2, 400, "K4 the 6.27→tower-top jump due south (x 1.5)");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-9.0, 8.1, -146.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "K1 placed on the walkway north part (x −9, north of the cross-wall)");
    raw_walk(&mut w, -9.0, -152.0, 240, &|sim| sim.bond.pos[2] < -149.5);
    log_pos(&w, "K1 walked south through the cross-wall gap (x −9)?");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-9.5, 8.1, -152.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "K2 placed on the walkway south part's east side");
    raw_walk(&mut w, -11.0, -152.0, 160, &|sim| sim.bond.pos[0] < -10.5);
    log_pos(&w, "K2 walked west off the walkway's west edge (onto the ramp 4.52..4.83?)");

    // L. The walkway north part's ENTRY from the display ground: the walkway (6.92) runs north
    //    to z −110.36 [D scan_rampne]; the display ground is 6.27 [D probe rays]. L1: walk west
    //    from the display eye along z −113 — does the display floor step/jump onto the walkway?
    //    L2: from the walkway's north end walk north — where does it end?
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [2.0, 7.4, -112.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "L1 placed on the display ground's west part (x 2, z −112.5)");
    raw_walk(&mut w, -8.0, -112.5, 260, &|sim| sim.bond.pos[0] < -5.0 || !sim.bond.grounded);
    log_pos(&w, "L1 walked west along z −112.5 (onto the walkway 6.92?)");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-8.0, 8.1, -112.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "L2 placed at the walkway's north end (z −112)");
    raw_walk(&mut w, -8.0, -106.0, 200, &|sim| !sim.bond.grounded);
    log_pos(&w, "L2 walked north past the walkway's north end (z −110.36)");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-15.0, 5.6, -152.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "F0 placed mid-ramp (y≈4.2 deck at x −15, z −152.5)");
    w.walk_to(Wp::new(-15.0, 5.3, -156.0).r(1.2), "F1 the ramp south-west (toward the drop)");
    log_pos(&w, "F1 on the ramp's south-west");
    raw_walk(&mut w, -15.0, -158.0, 150, &|sim| sim.bond.grounded && sim.bond.pos[1] < 2.0);
    log_pos(&w, "F2 walked off the ramp's south edge (landing on y≈0?)");
    // The ramp's north end: walk north along the deck.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-15.0, 5.6, -152.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    w.walk_to(Wp::new(-15.0, 5.3, -146.0).r(1.2), "F3 the ramp north");
    log_pos(&w, "F3 on the ramp's north deck");
    w.walk_to(Wp::new(-15.0, 5.3, -132.0).r(1.2), "F4 the ramp far north (z −132)");
    log_pos(&w, "F4 the ramp's north end");
    w.walk_to(Wp::new(-15.0, 5.3, -126.0).r(1.2), "F5 past the ramp's north end (z −126)");
    log_pos(&w, "F5 past the ramp north end");
}
