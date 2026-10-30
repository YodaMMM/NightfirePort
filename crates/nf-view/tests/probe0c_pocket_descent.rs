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
    // L3: continue the walkway's north walk further (L2's 200 ticks ran out mid-walk).
    raw_walk(&mut w, -8.0, -100.0, 400, &|sim| !sim.bond.grounded);
    log_pos(&w, "L3 the walkway's north walk continued (z −100?)");
    // L4: from the display floor at z −111.5, jump WEST onto the walkway (the 0.65 step up).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-4.6, 7.4, -111.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "L4 placed on the display floor's west edge (x −4.6, z −111.5)");
    jump_off(&mut w, -7.5, -111.5, 300, "L4 the west jump onto the walkway (6.27→6.92)");
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

/// The FULL route, real input end to end: the walk's leg-3 end state (the display eye) → west
/// along the display ground → the west-edge hop onto the 6.92 walkway (L4) → south to the
/// cross-wall gap (K1) → the south part → the west-edge drop onto the room_sk3 ramp (K2/D4) →
/// down the ramp to the y≈0 yard (F2) → south-east across the yard, the bridge and the corridor
/// → the warehouse east arm → the pit → the plates' body touch (ch 2) → the E arm → ch 8 [D
/// #560]. No teleports after the leg-3 placement.
#[test]
#[ignore = "route record: the full display-ground→plates walk (the leg-4 candidate)"]
fn probe_the_full_display_ground_to_plates_walk() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    // The walk's leg-3 end state (the display eye, the staircase-routed spot).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.4, 7.35, -113.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    // 1-2. West-north along the display ground, then the jump up onto the 7.24 platform's wide
    //      part (room_storage01, x −5.05..−2.04, z −108.67..−109.80 [D scan_westentry]; 0.97
    //      above the 6.27 floor — the E1-class rise).
    for (x, z, l) in [(3.5f32, -112.5f32, "west 1"), (1.5, -111.5, "west 2"), (0.0, -110.5, "west 3"), (-1.2, -109.8, "west 4")] {
        if !w.try_walk_to(Wp::new(x, 7.4, z).r(0.9), &format!("the display ground {l}")) {
            eprintln!("[desc] the {l} hop wedged");
        }
        log_pos(&w, &format!("1 {l}"));
    }
    jump_off(&mut w, -4.0, -108.9, 300, "2 the jump onto the 7.24 platform");
    // 3. South-west along the platform to the rib's north end (the rib x −4.54..−4.37,
    //    z −109.80..−116.47 [D scan_westentry]) and the west jump onto the 6.92 walkway
    //    (drop 0.32, gap ~1.2 [D the walkway's east edge x −5.65]).
    w.walk_to(Wp::new(-4.45, 8.35, -110.4).r(0.5), "the rib's north end");
    // The walkway's east portion starts at z −111.86 [D scan_rampne] — walk the rib south past
    // it, then the west jump (the rib 7.24 → the walkway 6.92, drop 0.32).
    raw_walk(&mut w, -4.45, -113.0, 200, &|sim| sim.bond.pos[2] < -112.6);
    log_pos(&w, "3a mid-rib");
    jump_off(&mut w, -7.0, -112.8, 300, "3b the rib→walkway west jump");
    // 4. South along the walkway's north part to the cross-wall's gap lane (x −9).
    w.walk_to(Wp::new(-8.5, 8.1, -116.0).r(1.2), "the walkway south 1");
    w.walk_to(Wp::new(-9.0, 8.1, -128.0).r(1.2), "the walkway south 2");
    w.walk_to(Wp::new(-9.0, 8.1, -143.0).r(1.2), "the walkway south 3 (the gap lane)");
    raw_walk(&mut w, -9.0, -152.0, 260, &|sim| sim.bond.pos[2] < -149.5);
    log_pos(&w, "3 through the cross-wall gap (x −9)");
    w.walk_to(Wp::new(-9.3, 8.1, -152.0).r(1.0), "the walkway's south part");
    // 4. West off the walkway's west edge (x −10.03) onto the ramp decks (4.52..4.83).
    raw_walk(&mut w, -11.5, -152.0, 160, &|sim| sim.bond.pos[0] < -10.6);
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    log_pos(&w, "4 dropped onto the ramp");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] < 6.5,
            "the west drop must land on the ramp decks (4.52..4.83 [D]), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    // 5. South-west down the ramp and off its south edge onto the y≈0 yard (F2).
    w.walk_to(Wp::new(-14.5, 5.4, -155.0).r(1.2), "the ramp south-west");
    raw_walk(&mut w, -15.0, -158.5, 150, &|sim| sim.bond.grounded && sim.bond.pos[1] < 2.0);
    log_pos(&w, "5 on the yard (y≈0)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] < 2.0,
            "the ramp's south walk-off must land the yard (y≈0 [D]), got {:?}", sim.bond.pos);
    }
    // 6. The yard: south-east to the bridge, east over it, the corridor to the strip.
    w.walk_to(Wp::new(-10.0, 1.1, -163.0).r(1.5), "the yard south-east 1");
    w.walk_to(Wp::new(-4.0, 1.0, -167.0).r(1.5), "the yard south-east 2");
    w.walk_to(Wp::new(1.5, 0.9, -172.0).r(1.5), "the yard south 3");
    w.walk_to(Wp::new(4.0, 0.9, -179.5).r(1.2), "the yard south-east");
    w.walk_to(Wp::new(8.5, 0.9, -179.0).r(1.2), "the bridge west");
    w.walk_to(Wp::new(14.5, 0.9, -175.0).r(1.2), "the bridge east");
    for (x, y, z, label) in [
        (20.0f32, 0.9f32, -174.0f32, "the corridor 1"),
        (27.0, 0.9, -174.0, "the corridor 2"),
        (34.0, 0.9, -174.0, "the corridor 3"),
        (41.0, 1.2, -174.0, "the corridor 4 (the 0.38 step)"),
        (47.0, 1.2, -174.0, "the corridor 5"),
        (50.5, 1.5, -173.8, "the strip approach"),
        (51.6, 1.5, -173.6, "the strip at the door"),
    ] {
        w.walk_to(Wp::new(x, y, z).r(0.9), label);
    }
    log_pos(&w, "6 on the strip");
    // 7. East into the plates' box (the body touch, ch 2 [D #559]).
    w.walk_to(Wp::new(52.7, 1.7, -173.5).r(0.5), "into the plates' box");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[desc] 7 in the box: pos {:?} ch2 {} ch3 {} ch8 {}", sim.bond.pos,
            sim.drones.channels.on(2), sim.drones.channels.on(3), sim.drones.channels.on(8));
        assert!(sim.drones.channels.on(2), "the body-touch plate fires ch 2 under Bond [D #559]");
    }
    // 8. The E arm (the walk's leg-5 shape) → the AND → ch 8.
    let mut and_at = None;
    for _ in 0..120 {
        w.tick(nf_game::player::Input::default(), true);
        if and_at.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(8) { and_at = Some(w.total_ticks); break; }
    }
    assert!(and_at.is_some(), "E on the plate never raised ch 8 (the AND [D #560]) within 120 ticks");
    eprintln!("[desc] 8 ch 8 rose at +{:?} — the plates reached by real input end to end", and_at);
}
