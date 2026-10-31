//! walkreal probe (level-0c-secret-door-reach-2): the north-end route REHEARSAL. Six facts the
//! real legs need, each printed, diagnostic only --nocapture:
//! F. the camera photograph (ch 0x1c) from the STAIR APPROACH (12.4) — the display basin is a
//!    sealed dead end [D probe rays], so the photograph must fire through the staircase opening
//!    from the hall level if the walk is to continue east;
//! A. the lane south extension (12.3 floor to (17, −158) [D upperlane columns]) and the deck lip
//!    (the deck's west face x 17.16, top 13.3, 1.0 above the lane [D upperlane]) — jump rehearsal;
//! B. the slow walk-off the deck's east edge onto the 9.24 patch (4.06 m [D upperlane columns]);
//! C. the walk-off the patch's north edge onto the east ramp (9.24 → ~4.65 [D the same scan]);
//! D. the ramp east + the pit jump (the pit x 30..32 floorless [D the same scan], the strip
//!    0.1 x 33..39 [D]);
//! E. the strip → the x-40 bridge → the warehouse ground south-east → the corridor's north end
//!    (step-continuous 0.69 → 0.90 at z −165 [D secretreach]) → south to the plates #558/#559
//!    (the body-touch ch 2 under Bond [D #559]).

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{wrap_pi, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_north_end_route() {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);

    // ---- F. the photograph from the stair approach (teleport = the diagnostic spot; the real
    // walk arrives here on the lane legs).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [13.0, 12.4, -128.2];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    w.tick(Input { switch_to: Some(0x54), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x54 { break; }
        w.tick(Input::default(), false);
    }
    eprintln!("[ne] F: camera held = {}", w.scene.sim.as_ref().unwrap().gun.weapon == 0x54);
    let sc = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| [(s.lo[0] + s.hi[0]) / 2.0, (s.lo[1] + s.hi[1]) / 2.0, (s.lo[2] + s.hi[2]) / 2.0])
            .expect("the jetpack camera subject [D #414/#454]")
    };
    eprintln!("[ne] F: subject centre {sc:?}");
    let mut photographed = false;
    {
        let start = w.total_ticks;
        let mut last_tap = 0u32;
        loop {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x1c) { photographed = true; break; }
            if w.total_ticks - start >= 900 { break; }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
                let dx = sc[0] - bp[0];
                let dz = sc[2] - bp[2];
                let dy = sc[1] - eye;
                if (dx * dx + dy * dy + dz * dz).sqrt() > 30.0 { eprintln!("[ne] F: out of range"); break; }
                let desired = dx.atan2(dz);
                let err = wrap_pi(desired - sim.bond.yaw);
                let want_pitch = (dy / walker::dxz(bp, sc).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let perr = want_pitch - sim.bond.pitch;
                let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
                let trigger = on_target;
                if trigger { last_tap = w.total_ticks; }
                let input = if trigger {
                    Input { aiming: true, ..Default::default() }
                } else {
                    Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), aiming: true, ..Default::default() }
                };
                w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
            }
            w.total_ticks += 1;
        }
    }
    eprintln!("[ne] F: photograph from the stair approach = {photographed}");

    // ---- A2. the walkway SOUTH branch: the ST-1 walkway south end (13.5, 15.7, −152 [D]) and
    // the step-down chain 14.6 → 13.5 → the deck 13.3 (x 18..21, z −162..−156.5 [D upperlane
    // columns]) — the deck hangs OFF the walkway, not off the lane (the lane's own south end
    // wedged at (14.57, −152.57) on the 11.27 shelf [this probe's first run]).
    let mut on_deck = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [13.5, 15.7, -152.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    w.walk_to(Wp::new(13.8, 15.0, -153.6).r(1.0), "the walkway south end");
    // Pop south over the walkway's south lip (the first run hung grounded=false on the edge at
    // (13.24, −153.65) — the walk's hold state; a jump clears the lip the player's own way).
    for t in 0..90u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.grounded)
        };
        if grounded && p[1] < 15.0 && p[2] < -153.9 {
            eprintln!("[ne] A2: off the lip at t{t} ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
            break;
        }
        let jump = grounded && t % 24 == 0;
        w.tick(Input { forward: 1.0, jump, ..Default::default() }, false);
    }
    w.walk_to(Wp::new(15.5, 13.9, -155.2).r(1.2), "the step-down 1");
    w.walk_to(Wp::new(17.0, 13.5, -156.8).r(1.0), "the step-down 2");
    w.walk_to(Wp::new(19.5, 13.3, -158.8).r(1.2), "onto the deck");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        on_deck = sim.bond.grounded && p[0] > 18.2 && p[0] < 21.6 && p[2] > -161.6 && p[2] < -156.8;
        eprintln!("[ne] A2: deck = {on_deck} at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    }

    // ---- B. the slow walk-off the deck's east edge onto the patch (forward 0.35; a full-speed
    // walk-off flies ~4.7 m east and overshoots the 1-m patch width [D upperlane columns]).
    if on_deck {
        let mut landed = false;
        for _ in 0..240u32 {
            let (p, grounded) = {
                let sim = w.scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.bond.grounded)
            };
            if grounded && p[1] < 12.0 {
                landed = true;
                let under = w.scene.sim.as_ref().unwrap().world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
                    .map(|(tt, _)| p[1] - 3.0 * tt);
                eprintln!("[ne] B: landed at ({:.2},{:.2},{:.2}) under {under:?} (want the patch 9.24, x 22..23, z −162..−157 [D])", p[0], p[1], p[2]);
                break;
            }
            let desired = (24.0f32 - p[0]).atan2(-158.5f32 - p[2]);
            let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
            w.tick(Input { turn: err.clamp(-0.08, 0.08), forward: 0.35, ..Default::default() }, false);
        }
        if !landed { eprintln!("[ne] B: never left the deck / never landed"); }
    }

    // ---- C. walk north off the patch onto the ramp (the patch's north edge z ~−157; the ramp
    // ~4.65 at x 22.5 [D upperlane columns]).
    let mut on_ramp = false;
    for _ in 0..240u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.grounded)
        };
        if grounded && p[1] < 8.5 && p[1] > 3.5 {
            on_ramp = true;
            let under = w.scene.sim.as_ref().unwrap().world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
                .map(|(tt, _)| p[1] - 3.0 * tt);
            eprintln!("[ne] C: on the ramp at ({:.2},{:.2},{:.2}) under {under:?} (want ~4.65 [D])", p[0], p[1], p[2]);
            break;
        }
        let desired = (22.5f32 - p[0]).atan2(-153.0f32 - p[2]);
        let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        w.tick(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    if !on_ramp { eprintln!("[ne] C: no ramp landing, pos {:?}", w.scene.sim.as_ref().unwrap().bond.pos); }

    // ---- D. the ramp east + the pit jump (run east, jump at the ramp's end x ≥ 29.2, land on
    // the strip 0.1 x 33..39 [D upperlane columns]; the carry 4.42 m flat [M player.rs]).
    let mut on_strip = false;
    if on_ramp {
        w.walk_to(Wp::new(27.5, 4.2, -149.0).r(1.0), "the ramp east");
        let mut jumped = false;
        for t in 0..300u32 {
            let (p, grounded) = {
                let sim = w.scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.bond.grounded)
            };
            if jumped && grounded && t > 20 {
                let under = w.scene.sim.as_ref().unwrap().world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
                    .map(|(tt, _)| p[1] - 3.0 * tt);
                eprintln!("[ne] D: landed at ({:.2},{:.2},{:.2}) under {under:?} (want the strip 0.1, x 33..40.5, z −152..−143 [D])", p[0], p[1], p[2]);
                if p[0] > 32.5 && p[0] < 41.0 && (p[1] - 0.1).abs() < 0.8 { on_strip = true; }
                break;
            }
            if jumped && !grounded && p[1] < -2.0 {
                eprintln!("[ne] D: fell into the pit at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
                break;
            }
            let desired = (36.0f32 - p[0]).atan2(-148.0f32 - p[2]);
            let err = wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
            let jump = !jumped && grounded && p[0] >= 29.2;
            if jump { jumped = true; eprintln!("[ne] D: pit jump at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
            w.tick(Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, jump, ..Default::default() }, false);
        }
    }

    // ---- E. the strip east, the x-40 bridge, the warehouse ground south-east, the corridor's
    // north end, south to the plates (ch 2 under Bond [D #559]).
    if on_strip {
        w.walk_to(Wp::new(39.0, 0.2, -150.0).r(1.0), "the strip east");
        w.walk_to(Wp::new(42.5, 0.5, -150.5).r(1.2), "the x-40 bridge");
        w.walk_to(Wp::new(48.0, 0.5, -156.0).r(1.5), "the warehouse ground south-east");
        w.walk_to(Wp::new(55.0, 0.7, -163.0).r(1.5), "the ground at the corridor's north end");
        w.walk_to(Wp::new(54.0, 1.0, -166.5).r(1.2), "into the corridor (0.90 [D secretreach])");
        w.walk_to(Wp::new(54.0, 1.0, -170.0).r(1.2), "the corridor south 1");
        w.walk_to(Wp::new(53.0, 1.0, -173.0).r(1.0), "the corridor south 2");
        w.wait(20);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("[ne] E: at the plates ({:.2},{:.2},{:.2}) ch2 {} ch3 {} grounded {}",
                p[0], p[1], p[2], sim.drones.channels.on(2), sim.drones.channels.on(3), sim.bond.grounded);
        }
        // The E pulse on the plates (the real arm path): ch 3 -> AND #560 -> ch 8.
        for _ in 0..120 {
            w.tick(Input::default(), true);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(8) { break; }
        }
        eprintln!("[ne] E: ch8 (the AND) = {}", w.scene.sim.as_ref().unwrap().drones.channels.on(8));
        for _ in 0..900 {
            w.tick(Input::default(), true);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(d) = sim.drones.sss.list.iter().find(|s| s.channel == 8) {
                if d.state == 0 { eprintln!("[ne] E: the door swept open"); break; }
            }
        }
    }
    eprintln!("[ne] done: photo {} deck {} ramp {} strip {}", photographed, on_deck, on_ramp, on_strip);
}
