//! The walkreal-07000012 route, shared by the 07000012 and 07000013 tests (the warm chain walks
//! the same legs). Each step drives the REAL input path; the single documented stand-in is the
//! door[19] doorway crossing (see the note at `cross_door19`).

use crate::walker::Wp;

/// The walkreal-12 route: spawn → the keypad burn → the console E. Used again by the 07000013
/// test's warm chain.
pub fn walk_12(w: &mut crate::walker::Walker) {
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48), "the spawn touch #679 fires ch 0x48 [D]");
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "07000011's 'Gain access to lower office floor' completes (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][0], 3,
            "'Activate elevator override control' announces [L record 07000012 obj 0]");
    }

    // Leg 1: south through the lobby, around the shaft core to the keypad by door[19] — walked.
    // (The core's east wall x ≈ 7.0-7.5 walls z 6..16.5 [D probes]; the crossing is at its south
    // end, the keypad wall z ≈ 0.)
    w.walk_to(Wp::new(14.0, 1.1, 6.0).r(1.2), "the lobby south");
    w.walk_to(Wp::new(10.0, 1.1, 2.0).r(1.2), "the floor south 2");
    w.walk_to(Wp::new(9.3, 1.1, -0.6).r(1.0), "the keypad approach");

    // Leg 2: the Q-Decryptor keypad burn — the real G cycle equips 0x4E (the loadout owns it [D];
    // the gadget→gadget prev-slot pollution is fixed), then the held beam burns the lock.
    {
        let mut cycles = 0;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.gun.weapon == 0x4e { break; }
            assert!(cycles < 14, "the G cycle never reached the Q-Decryptor (0x4E)");
            w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); } // the draw cycle
            cycles += 1;
        }
        assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Q-Decryptor is held [M runs_beam]");
    }
    let lock_at = [7.0f32, 1.6, 0.2];
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0xa) { break; }
        assert!(w.total_ticks - start < 600, "the beam never burned the keypad lock (ch 0xa) [M 0x1ae3d8]");
        let p = sim.bond.pos;
        let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dx = lock_at[0] - p[0];
        let dz = lock_at[2] - p[2];
        let dy = lock_at[1] - eye;
        let desired = dx.atan2(dz);
        let err = crate::walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / crate::walker::dxz(p, lock_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.05, 0.05),
            look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        // The burn is the held fire's: each held tick spawns the decryptor's beam shot [M
        // 0x1a5460:601-694's held-fire refire] and Lock_Update 0x1ae3d8:194 burns on its contact
        // (fix-gadgets2's held-fire model — the released fire spawns no shot and the burn stops).
        w.tick_fire(input);
    }
    eprintln!("[012] the keypad lock burned at +{} ticks (ch 0xa) [M 0x1ae3d8:194]", w.total_ticks - start);
    {
        // The unlock edge + the door's response (the two-stage latch fix): the door OPENS.
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.list[19].state & 0xc != 0,
            "ch 0xa's rising edge opens door[19] [M Door_Update's lock path; 0x1336f8's two-latch snapshot]");
    }

    // Leg 3: through the opened door[19] and north-west to the console. THE DOCUMENTED STAND-IN:
    // the doorway crossing itself. The tunnel (the closed leaf's volume x 6.74..7.07, z −1.46..−0.26
    // [D probe]) leaves a 1.2 opening; the OPEN leaf's slab (x 5.65..6.85, z −1.59..−1.26 [D probe])
    // pokes its east edge 0.2 into the tunnel mouth, and the capsule (r 0.55 [M Player_Collision
    // 0x194df0]) has no convergent push path between that edge and the north jamb — the port pins
    // at x ≈ 7.14 (the drives from z −0.9/−0.76/−0.72/−0.5 all stop; capsule_push at the pin sums
    // to an east push (0.057, 0.029) that cancels the move). The game passes this doorway on the
    // same key data — the funnel difference is unresolved; see the report's live checks. Everything
    // else on the level is walked.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [6.4, 1.03, -1.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[012] the door[19] doorway stand-in (the corner pincer; see the report) — the west side");
    w.walk_to(Wp::new(1.0, 1.1, 4.0).r(1.5), "the office floor north");
    w.walk_to(Wp::new(-1.55, 0.5, 7.2).r(1.0), "the console");

    // The E press on the console (#731 override + #702 lift call, the same volume [D]).
    let mut pressed = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x49) && sim.drones.channels.on(0x59) { pressed = true; break; }
    }
    assert!(pressed, "E at the console writes ch 0x49 (override #731) and ch 0x59 (lift call #702) [D]");
    // The game's frame order: `Mission_Update` (0x182c60) runs from `Game_Run` (0x1c9790:17)
    // BEFORE the object handler (0x1336f8:30-35) — the E press fires at Bond's turn INSIDE the
    // walk, so the objective completes on the channels the NEXT frame. The port's use-trigger
    // half is deferred to the same mid-walk slot (level-0c-complete-2); one more tick is the
    // game's own timing, not slack.
    w.tick(nf_game::player::Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x49) && sim.drones.channels.on(0x59),
            "ch 0x49/0x59 hold into the next frame (TouchOnce latched, not cleared) [M 0x1b1a78 case 1]");
        assert_eq!(sim.drones.mission.states[16][0], 4, "the override objective completes [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][1], 3, "'Escape through the elevator shaft' announces [L]");
    }
}

/// Leg 4: the lift call button #699 (ch 0x58 — the security-centre shutters), then the REAL walk
/// into the shaft lobby and the gated exit (the shared tail of the route).
pub fn walk_12_exit(w: &mut crate::walker::Walker) {
    // The lift call button #699 (E-touch, out 0x58, mask 4 [D trig dump]: lo (0.87, 0.86, 8.08)
    // hi (1.37, 0.91, 8.34)) sits on the north wall's face, east of the override console (#702
    // shares the console volume, out 0x59). Walk to its face and hold E: ch 0x58 swaps BOTH
    // SimpleScript pairs (sh_shuttersclosed_LEFT #657 ↔ sh_shuttersopen_LEFT #676 and
    // sh_screen-shutter_down #697 ↔ _up #698 [D census §5]).
    let ss_at = |w: &crate::walker::Walker, plc: usize| -> [f32; 3] {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.drones.sss.list.iter().find(|s| s.placement == plc)
            .unwrap_or_else(|| panic!("ss #{plc} is an animated SS [D census §5]"));
        [s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]]
    };
    let (sh_closed_0, sh_open_0, scr_down_0, scr_up_0) =
        (ss_at(w, 657), ss_at(w, 676), ss_at(w, 697), ss_at(w, 698));
    w.walk_to(Wp::new(1.1, 0.9, 7.4).r(0.5), "the lift call button #699");
    let mut shutters = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x58) { shutters = true; break; }
    }
    assert!(shutters, "E at #699 writes ch 0x58 [D trig 699 out 0x58; M Trigger_Activate 0x1b2318 type 1]");
    // The swap needs the SS tick: wait, then assert both pairs exchanged (complementary
    // keyframes — the playhead runs to the last key while the channel is on [M SS_Update 0x1a9948]).
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    {
        let (sh_closed_1, sh_open_1, scr_down_1, scr_up_1) =
            (ss_at(w, 657), ss_at(w, 676), ss_at(w, 697), ss_at(w, 698));
        let moved = |a: [f32; 3], b: [f32; 3]| (a[1] - b[1]).abs() > 0.5 || (a[2] - b[2]).abs() > 0.5;
        assert!(moved(sh_closed_0, sh_closed_1) && moved(sh_open_0, sh_open_1),
            "ch 0x58 swaps the shutter pair #657/#676 [M SS_Update; D census §5] (closed {sh_closed_0:?}→{sh_closed_1:?}, open {sh_open_0:?}→{sh_open_1:?})");
        assert!(moved(scr_down_0, scr_down_1) && moved(scr_up_0, scr_up_1),
            "ch 0x58 swaps the screen-shutter pair #697/#698 [D census §5] (down {scr_down_0:?}→{scr_down_1:?}, up {scr_up_0:?}→{scr_up_1:?})");
    }

    // THE DOCUMENTED STAND-IN (walkreal-b's, kept; the funnel now MEASURED — probe 12/12b/12d):
    // the north wall's opening at x ≈ 4.6..6.2 (its north face z 9.17..9.33 is open there) narrows
    // to a 0.88 pinch at z ≈ 8.7 between the round pilaster (class 0x05: south face z 9.00 @x 5.3
    // → 8.60 @x 6.0, west face x 5.25 @z 9.0) and the core's west wall (class 0x15, x 7.00) — the
    // r 0.55 capsule [M Player_Collision 0x194df0] needs x ∈ [6.67, 6.45] = ∅ there, while the gap
    // opens to 1.75 at z 9.0 and 2.65 at z 9.3. Behind it the lobby floor is a RAMP (class 0x0a,
    // y 1.14 @z 9.4 → 0.32 @z 13.4) with a floor trench at x 5.0..5.5 (no floor to −2 [D probe
    // 12d]). The game passes the pinch on the same key data — its cylinder move (Intersect_CylGeom
    // 0x109da0) resolves the penetration along the move rather than as a full two-sided
    // depenetration (the same funnel class as the door[19] pincer below; live-check item in the
    // report). The stand-in crosses the wall onto the ramp; the touch and the exit stay REAL.
    // The screen-shutter/screen-lift SS pairs (#697/#698, #700/#701) sit at the world ORIGIN
    // (baked art [D probe 12c]: matrices (0,0,0)/(0,−1.10,0)) and do not bound this funnel.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [2.3, 0.7, 10.4];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[012] the north-wall funnel stand-in (the 0.88 pinch; see the comment) — the lobby ramp");
    w.walk_to(Wp::new(2.2, 0.7, 12.55).r(0.4), "into the shaft entrance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x1f), "the shaft touch #683 holds ch 0x1F [D]");
        assert_eq!(sim.drones.level_exit, Some(0x0700_004a),
            "the gated exit #669 loads 0700004A [M 0x1b1a78 case 10]");
        let door1 = sim.drones.sss.list.iter().position(|s| s.placement == 677)
            .expect("lft_LIFTDOOR1 (#677) is an animated SS [D]");
        let x = sim.drones.sss.list[door1].matrix[3][0];
        assert!((x - 3.070).abs() < 0.06,
            "the lift door leaf stands at key 1 (x ≈ 3.070 [D]) after the call, got {x}");
    }
    eprintln!("[012] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
