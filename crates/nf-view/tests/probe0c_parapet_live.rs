//! PROBE (level-0c-parapet-hop): the display-ground→walkway link under the walkway's north
//! part, walked with real input. The plates-reach-2 scans walled every crossing at the rib's
//! west parapet (x −4.84..−4.79, top 8.67, z −112.21..−116.70 [D scan_npar hits 37-42: top
//! faces y 8.67/8.62]) and this session's scans complete the picture [D coll-scan 0700000c.bin,
//! /tmp/scan_{rib,npar,spar,east,l4,nw,deck}]:
//!   • the walkway north lane (room_sk3, top 6.92, x −8.25..−7.34, z −110.36..−120.31) has a
//!     tall east wall at x −7.34 (6.92→12.66, z −110.36..−111.86) and a north step to a 9.24
//!     ledge (z −110.02..−110.36) walled at z −110.02 (9.24→12.26) — no entry from the north;
//!   • the wide walkway (top 6.92, x −7.34..−5.65, z −111.86..−153.52) has a north wall at
//!     z −111.86 (6.92→12.25) and its east face at x −5.65 (6.92→7.24) carries the room_sk3
//!     7.24 ledge (x −5.65..−5.06, z −112.24..−116.47) whose north wall (z −112.24, 7.24→11.13)
//!     and the storage01 west strip (7.24, x −5.05..−4.54, z −112.24..−116.47) are separated
//!     from the rib (7.24, x −4.54..−4.37) by the parapet blade for its whole z range — the
//!     blade's north tip (−112.21) pokes 0.03 north of the band's north edge (−112.24) and its
//!     south tip (−116.70) 0.23 past the band's south edge (−116.47): no 7.24 walk-around [D];
//!   • UNDER all of it lies the room_storage01 6.15 slab, an L: west arm x −8.25..−5.57
//!     (z −111.62..−116.35) + east arm x −5.57..−4.50 (z −112.58..−116.35) [D scan_rib hits
//!     26-31, 22-23], and the display ground (6.27) reaches x −4.50 — a 0.12 step down onto the
//!     east arm at z ≤ −112.58 (the old L1 lane z −112.5 missed the slab by 0.08 and stopped at
//!     x −5.0 anyway);
//!   • under the walkway top the standing spine (A = pos+0.275, B = pos+0.55−cc, |d| 0.7597
//!     [M Player_Collision 0x194df0 disasm 0x195444/0x195464]) crosses the 6.92 plane in its
//!     BOTTOM half (f > 0.5) → solid → pushed up onto it [M the horizontal-face rule,
//!     Collide_Intersect 0x1ebaa8:321-322 — a floor crossed in the spine's top half is jumped
//!     through from below, the same face in the bottom half is solid]; the jump's headroom LOS
//!     runs pos→pos+0.95 [M Player_HandleJump 0x1916b0:160-165] and the plane at 6.92 is BELOW
//!     pos (7.18 on the slab), so the jump initiates and its rising spine skips the plane.
//! #[ignore]d: a route record, not a pin.
use walker::{Walker, Wp};

#[path = "common/walker_b2.rs"]
mod walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn log_pos(w: &Walker, label: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[parapet] {label}: grounded {} pos ({:.2}, {:.2}, {:.2})", sim.bond.grounded,
        sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

/// Raw forward ticks toward a target xz with a periodic log.
fn raw_walk_log(w: &mut Walker, tx: f32, tz: f32, max_ticks: u32, stop: &dyn Fn(&nf_view::scene::Sim) -> bool, tag: &str) {
    for i in 0..max_ticks {
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
        if i % 60 == 0 { log_pos(w, &format!("{tag} t{i}")); }
        if stop(w.scene.sim.as_ref().unwrap()) { break; }
    }
    log_pos(w, &format!("{tag} done"));
}

/// The decisive walk: the display eye → west-south-west onto the 6.15 slab (the 0.12 step at
/// x −4.50, z ≤ −112.58) → west under the walkway → the pop/jump-pop onto the 6.92 walkway →
/// south along the walkway → the cross-wall gap lane (x −9) → the west drop onto the ramp →
/// the ramp's south walk-off onto the yard. Every move is real input; the only placement is
/// the leg-3 end state (the display eye), the same placement the whole-mission walk uses.
#[test]
#[ignore = "route record: the display-slab→walkway under-pass, walked and logged"]
fn probe_the_display_slab_underpass_to_the_yard() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.4, 7.35, -113.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    log_pos(&w, "0 the display eye (the leg-3 end state)");
    // 1. South-west along the display ground to the southern entry lane: the 6.22 display
    //    piece (x −4.41..−2.93, z −112.41..−117.07 [D scan_sh/spar]) steps down 0.07 onto the
    //    slab's east arm (6.15, x −5.57..−4.50, z −112.58..−116.35) at z −115 — south of the
    //    oDrone's hover spot (its collision sat at x −3.82, z −112.9..−113.3 this run and
    //    walled the z −112.9 lane [D scan_sh: the oDrone triangles; the sim wedge log]).
    w.try_walk_to(Wp::new(-2.4, 7.4, -115.2).r(0.9), "1a the display south-west");
    log_pos(&w, "1a the display south-west");
    w.try_walk_to(Wp::new(-3.5, 7.35, -115.2).r(0.9), "1b the display west (the 6.22 piece)");
    log_pos(&w, "1b the display west");
    // 2. West across the 0.07 step onto the slab (6.15): the east arm then the west arm —
    //    under the wide walkway (6.92 overhead from x −5.65).
    raw_walk_log(&mut w, -6.0, -115.2, 400, &|sim| sim.bond.pos[0] < -5.6, "2a the slab east arm (z −115.2)");
    raw_walk_log(&mut w, -7.2, -114.6, 400, &|sim| sim.bond.pos[0] < -7.0, "2b the slab under the wide walkway");
    let under = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.bond.pos[0] < -5.65 && sim.bond.pos[1] < 7.6
    };
    log_pos(&w, "2c under the walkway?");
    // 3. The pop: first 120 plain ticks (does the solid bottom-half push settle him on top?),
    //    then the jump-pop (the rising spine skips the plane, the falling spine is solid).
    if under {
        for i in 0..120 {
            w.tick(nf_game::player::Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.grounded && sim.bond.pos[1] > 7.7 { break; }
            let _ = i;
        }
        log_pos(&w, "3a after 120 plain ticks under the plane");
        if w.scene.sim.as_ref().unwrap().bond.pos[1] < 7.7 {
            for i in 0..90 {
                w.tick(nf_game::player::Input { jump: i == 6, ..Default::default() }, false);
                let sim = w.scene.sim.as_ref().unwrap();
                if i > 30 && sim.bond.grounded { break; }
            }
            log_pos(&w, "3b after the jump-pop");
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] > 7.7 && sim.bond.pos[0] < -5.6 {
            eprintln!("[parapet] 3c ON the walkway — the under-pass walk is COMPLETE");
        } else {
            // Documented outcome, not a pin: the display→slab entry was walled this run at
            // x −3.82 (z −112.9 AND z −115.15 lanes) by a DYNAMIC blocker invisible to the
            // disc's static collision [D scan_w: no triangle within 1.5 of (−3.82, 7.0,
            // −114.5); the oDrone (map 11 obj 5) patrols the display and its sim-time
            // collision sat at (−3.0..−3.4, 7.16..7.55, −111.5..−111.8) in scan_sh's disc
            // placement]. The slab under-pass itself is PROVEN by
            // probe_the_underpass_pop_mechanics (P0-P3: the plain settle pops the 6.15 slab
            // onto the 6.92 walkway). The open question for the next window: how the GAME
            // routes the player west at the display's west edge past the patrolling drone
            // (ride it, or the patrol has a gap) — live check filed in the report.
            eprintln!("[parapet] 3c NOT on the walkway — the entry wedge (dynamic blocker): \
                       grounded {:?} pos {:?}", sim.bond.grounded, sim.bond.pos);
            return;
        }
    }
    log_pos(&w, "3c ON the walkway");
    // 4. South along the walkway's north part to the cross-wall's gap lane (x −9) — the
    //    full-route probe's proven lane.
    raw_walk_log(&mut w, -8.5, -117.0, 400, &|sim| sim.bond.pos[2] < -116.0, "4a the walkway south 1");
    w.walk_to(Wp::new(-9.0, 8.1, -128.0).r(1.2), "4b the walkway south 2");
    w.walk_to(Wp::new(-9.0, 8.1, -143.0).r(1.2), "4c the walkway south 3 (the gap lane)");
    raw_walk_log(&mut w, -9.0, -152.0, 300, &|sim| sim.bond.pos[2] < -149.5, "4d through the gap lane");
    log_pos(&w, "4e through the cross-wall gap (x −9)");
    // 5. West off the walkway's west edge (x −10.03) onto the ramp decks (4.52..4.83) — the
    //    D4 drop.
    raw_walk_log(&mut w, -11.5, -152.0, 200, &|sim| sim.bond.pos[0] < -10.6, "5a the west drop");
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] < 6.5,
            "the west drop must land on the ramp decks (4.52..4.83 [D]), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    log_pos(&w, "5b dropped onto the ramp");
    // 6. South-west down the ramp and off its south edge onto the y≈0 yard (F2).
    w.walk_to(Wp::new(-14.5, 5.4, -155.0).r(1.2), "6a the ramp south-west");
    raw_walk_log(&mut w, -15.0, -158.5, 200, &|sim| sim.bond.grounded && sim.bond.pos[1] < 2.0, "6b the ramp walk-off");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] < 2.0,
            "the ramp's south walk-off must land the yard (y≈0 [D]), got {:?}", sim.bond.pos);
    }
    log_pos(&w, "6c ON the yard — the chain is complete");
}

/// The under-pass mechanics in isolation: placed on the slab under the wide walkway, do the
/// plain settle and the jump-pop each land Bond on the 6.92 top? (Real input after the
/// placement; the placement is between probe sections.)
#[test]
#[ignore = "route record: the under-pass pop mechanics, logged"]
fn probe_the_underpass_pop_mechanics() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-6.5, 7.18, -113.3];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "P0 placed on the slab under the wide walkway");
    // Variant A: the plain settle (the solid bottom-half push, no jump).
    for i in 0..150 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] > 7.7 { break; }
        let _ = i;
    }
    log_pos(&w, "P1 variant A: the plain settle");
    let a = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.grounded, sim.bond.pos)
    };
    // Variant B: the jump-pop from wherever variant A left him (re-place first).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-6.5, 7.18, -113.3];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    for i in 0..120 {
        w.tick(nf_game::player::Input { jump: i == 6, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if i > 30 && sim.bond.grounded && sim.bond.pos[1] > 7.7 { break; }
    }
    log_pos(&w, "P2 variant B: the jump-pop");
    eprintln!("[parapet] P3 summary: variant A settled {:?} {:?}, variant B pos {:?}",
        a.0, a.1, w.scene.sim.as_ref().unwrap().bond.pos);
}

/// The rib side for the record: walk the rib south past the blade's north end and log every
/// west attempt (the blade walls them all [D]); then the wide part's west strip south edge.
#[test]
#[ignore = "route record: the rib west attempts, logged"]
fn probe_the_rib_west_attempts() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700000c.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-4.45, 8.35, -110.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    log_pos(&w, "R0 placed on the rib's north end (7.24)");
    raw_walk_log(&mut w, -4.45, -111.9, 240, &|sim| sim.bond.pos[2] < -111.6, "R1 the rib south to the blade's north end");
    // R2: west at z −111.8 (north of the blade) — the gap band (no 7.24 floor west of −4.54
    // between the strip's south edge −111.14 and the slab's north −111.62/−112.58 [D]).
    raw_walk_log(&mut w, -6.0, -111.8, 200, &|sim| !sim.bond.grounded || sim.bond.pos[0] < -5.1, "R2 the west walk north of the blade");
    log_pos(&w, "R2 done (fell or crossed?)");
    // R3: back on the rib, south past the blade's north tip, west INTO the blade (expect a
    // wedge at x −4.84, top 8.67 [D]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-4.45, 8.35, -110.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    raw_walk_log(&mut w, -4.45, -113.5, 240, &|sim| sim.bond.pos[2] < -113.0, "R3 the rib south past the tip");
    raw_walk_log(&mut w, -6.0, -113.5, 160, &|sim| sim.bond.pos[0] < -4.6, "R4 the west walk into the blade");
    log_pos(&w, "R4 done (wedge expected at x −4.84)");
}
