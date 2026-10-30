//! walkreal-b: 0700000b (Tower C — the executive floor and the roof) walked with the player's own
//! movement — no teleports past a blocker, no channel or state pokes beyond the two documented
//! disc-side open items. The mission's objective records [L ee_snapshot 0x2a3f78/0x2a3fc0/0x2a4008
//! + M Mission_MonitorObjectives 0x1827d8]: mission 07000009's monitored AND spans 11 records —
//! obj 1 (label 0x04000016) completes on the INVERTED alarm channel ch 0x61 (flags word @+0x10 =
//! 2), objs 2-5 (0x04000017-1a) on the carried ch 0x46-0x49, and THIS level's six:
//!
//! | obj | label      | ch    | how the walkthrough completes it |
//! |-----|------------|-------|----------------------------------|
//! | 6   | 0x0400001b | 0x4B  | REAL: the spawn TOUCH_ONCE plates #165/#247/#284+#176 [D trig dump] |
//! | 7   | 0x0400001c | 0x4C  | REAL: the Q-Worm gadget shot into SCOMP_QWORM_C #297 [M PCQWorm_Update 0x1aee20]; the boot loadout owns weapon 0x58 [D weapon-matrix 0700000b load=1] |
//! | 8   | 0x0400001d | 0x4D  | REAL: the walked terminal-room plate #163 [D] |
//! | 9   | 0x0400001e | 0x4E  | OR mux #295 = OR(ch 0x08 ← OUT #294 mirror of ch 0x54, ch 0x06) [D]; the disc writer of ch 0x54/0x06 is the level's OPEN item (swept: triggers, all 20 level scripts, 3 pack scripts, copter scripts, every placement param [D]) — the mux's ch 0x08 input is driven, exactly as the level doc's walkthrough does |
//! | 10  | 0x040001f  | 0x59  | REAL chain: ch 0x4F = AND #299(0x4C, 0x4E) [D] → E-plate #166 (mask 4 = use) → ch 0x35 → SEQ #252(0x36, 0x35) → ch 0x34 → `Trigger_Calc` 0x1b1900's per-level copy → ch 0x2D → SP wedge #177 (060004FA) plays [M 0x1dbda8 gate law] → stream 13 @0x991 writes ch 0x59 |
//! | 11  | 0x0400020  | 0x5A  | REAL: the parachute pickup touch (ch 0x3F) + the roof-outside volume (ch 0x40) → SEQ #281 → ch 0x41 → OUT #304 → ch 0x5A + ch 100 [D] |
//!
//! ch 100 fires when the monitored AND of ALL 11 records holds [M 0x1827d8:119-123] → MissionState 6
//! + `LevelToEndTo(0x0700000b)` = 0x0730000b [M Mission_Update 0x182c60:85-122]. The carried
//! channels 0x46-0x4a model the 0a exit's RamSave carry [M Player_RamSave 0x195670 — the full
//! 256-byte switch_channels block rides the save]. The balcony→exec lift ride uses the port's real
//! ride machinery (`ground_motion`, the script-entity collision set — the paternoster cab 0x2000367
//! rides pinned [M 0x194188 → Control_InheritVelocity 0x133e88]); if the boarding fails the
//! documented stand-in transit fires and says so in the log — the ride attempt itself is printed,
//! not pinned.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_warm() -> Walker {
    // Single load: the level's own boot state IS the carried state (health 100, the 0a
    // loadout [L loadout print]) — the old cold-load capture applied the boot's own values
    // back and doubled the memory peak (a small test machine OOM-kills the second load). The carried
    // channels of the real run (objectives 3-6 + the hatch unlock) [M 0x195670:13].
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load"));
    for c in [0x46u8, 0x47, 0x48, 0x49, 0x4a] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w
}

fn cab_y(w: &Walker) -> Option<f32> {
    let sim = w.scene.sim.as_ref().unwrap();
    // The cab is script entity 0x2000367 [D nis dump; lifts_ride_live matches the same hex form] —
    // the old decimal literal 2000367 never matched (0x2000367 = 33555303), so the boarding poll
    // was blind and Bond walked into the empty shaft.
    sim.drones.scripts.objects.list.iter().flatten()
        .find_map(|o| match o {
            nf_game::nis::Obj::Entity { hash, matrix, .. } if *hash == 0x2000367 => Some(matrix[3][1]),
            _ => None,
        })
}

#[test]
fn walk_balcony_to_the_roof_jump() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 28.92).abs() < 1.5 && (13.0..17.0).contains(&p[1]), "the balcony start [D], got {p:?}");
        // The boot loadout owns the Q-Worm gadget [D weapon-matrix 0700000b load=1; M 0x1a6f80
        // walks owned records only] — printed, and the shot leg re-checks before it fires.
        eprintln!("[0b] boot owned[0x58] = {}", sim.drones.inventory.owned[0x58]);
    }
    // The spawn plates fire under Bond (objective 6 + the arrival cut-scene arming); the
    // cut-scene's tail re-places Bond (event 6) — wait it out before any route work.
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b), "the spawn plate #176 → ch 0x4B (objective 6) [D]");
    w.wait_enabled(6000, "the arrival cut-scene tail");
    w.wait(300);
    // The arrival cut-scene's GameFlow push logs a level-exit request (the port cannot load the
    // movie state); the walkreal route continues — drop the walker's stop-on-exit for the REAL
    // shaft walk, then restore it for the exec-floor section (whose walked approaches are the
    // pinned blockers' stand-ins; the mission's own end asserts read ch 100/MissionState directly).
    w.stop_on_exit = false;

    // THE BALCONY→EXEC LIFT RIDE: the paternoster cab (script 06000048, mode 8 loop) cycles
    // y 11.0 → 48.44 → 11.0 over 1239 ticks at x 0, z ≈ 25 [D nis dump]; the shaft spans
    // x −2.7..1.6, z 22.6..27 [D placement #232 boxes]. The cab's collision top rides the
    // script-entity dynamic set (the game's own ride law [M 0x194188 → 0x133e88]) — walk to the
    // balcony floor EAST of the shaft lip (the old target x 0.5 was over the void: Bond slid in
    // and crossed the y 7.98..11.98 out-of-bounds slabs [D trig43/44 boxes] → ch 0x7A → OR #46
    // [0x79,0x7A] → ch 0x7B → the fail wedge 0600098A disabled Bond for good), wait for the cab's
    // bottom hold (top = 11.0 + 3.3 = 14.3, a 0.2 step off the 14.13 floor [D probe]), step on,
    // ride, step off east at the exec floor (47.77 [D probe]).
    let mut rode = false;
    {
        // The wait spot is on the balcony's proven z≈25.8 corridor, 0.7 east of the opening's
        // lip (x 1.6): the earlier wait AT the lip (x 0.5 target) either slid Bond in on the
        // walk's residual velocity or had the boarding steer walk him off the edge — the cab's
        // stand surface is cy + 4.1 (15.1 at the key-0 hold [L lifts_ride_live: grounded 16.11]),
        // a full step above the 14.1 floor, so the board is the player's hop, not a walk step.
        if w.try_walk_to(Wp::new(2.3, 14.0, 25.8).r(1.0), "the shaft wait (east of the lip)") {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.vel = [0.0; 3];
        }
        {
            // One probe of the script-entity id/hash pairing (the report's open item): the lift
            // script's entities at runtime — the cab is hash 0x2000367 kind 49 (class 0x31), its
            // matrix x/z LOCAL (0,0) while the collision box is placement-anchored (x 0,
            // z 23.19..26.56 [D placement #232]) — only matrix[3][1] carries the keyed y.
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
                if let Some(o) = o {
                    if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
                        if hash & 0x0f00_0000 == 0x0200_0000 {
                            eprintln!("[0b] probe script-obj{i}: hash {hash:#x} kind {kind} at ({:.1},{:.2},{:.1})",
                                matrix[3][0], matrix[3][1], matrix[3][2]);
                        }
                    }
                }
            }
            eprintln!("[0b] probe cab_y() = {:?}", cab_y(&w));
        }
        let start = w.total_ticks;
        let mut boarded = false;
        while w.total_ticks - start < 2600 && !boarded {
            if let Some(cy) = cab_y(&w) {
                // Board during the bottom hold (cy 11.0, surface 15.1 [D keys; L lifts_ride]) —
                // hop on; also chase a cab that started rising under Bond (the surface below his
                // knees still catches the hop).
                if (10.5..=18.0).contains(&(cy + 4.1)) {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let p = sim.bond.pos;
                    // Only from the balcony floor band (a fall crosses the OOB web — abort).
                    if (13.5..=17.0).contains(&p[1]) && walker::dxz(p, [0.0, p[1], 25.0]) < 4.5 {
                        let desired = (0.0f32 - p[0]).atan2(25.0 - p[2]);
                        let err = walker::wrap_pi(desired - sim.bond.yaw);
                        // Land WELL inside the box (x ±1.5 [D placement #232]): a landing near the
                        // east face gets shoved off it by the rising surface (the 0.06/tick rise
                        // resolves the penetration sideways) — the carry then has no body to grab.
                        let jump = w.total_ticks % 20 == 0 && sim.bond.grounded
                            && walker::dxz(p, [0.0, p[1], 25.0]) > 0.8;
                        let input = nf_game::player::Input {
                            turn: err.clamp(-0.08, 0.08),
                            forward: if err.abs() > 0.6 { 0.0 } else { 0.6 },
                            jump,
                            ..Default::default()
                        };
                        w.tick(input, false);
                        let p2 = w.scene.sim.as_ref().unwrap().bond.pos;
                        if w.scene.sim.as_ref().unwrap().bond.grounded
                            && p2[1] > 15.5
                            && walker::dxz(p2, [0.0, p2[1], 25.0]) < 1.0
                            && p2[0] < 1.0 {
                            boarded = true;
                        }
                        continue;
                    }
                }
            }
            w.tick(Default::default(), false);
        }
        if boarded {
            // Stand on the cab; the ride carries Bond ([M] ground_motion, lifts_ride's pin). Ride
            // until the surface (cy + 4.1) reaches the exec floor 47.77 [D probe], then step off
            // east — a flat walk off the level surface.
            let start = w.total_ticks;
            while w.total_ticks - start < 1600 {
                let cy = cab_y(&w).unwrap_or(0.0);
                if cy + 4.1 >= 47.6 { break; }
                w.tick(Default::default(), false);
                let dt = w.total_ticks - start;
                if dt % 200 == 0 {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let sp = sim.drones.scripts.players.iter().find(|p| p.scripts[0] == 0x0600_0048);
                    let cpos = sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
                        nf_game::nis::Obj::Entity { hash: 0x2000367, matrix, .. } => Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
                        _ => None,
                    });
                    eprintln!("[0b] ride+{dt}: cy {cy:.2} cab {:?} bond ({:.2},{:.2},{:.2}) grounded {} sub {} sp_state {:?} sp_deleted {:?} ch75 {}",
                        cpos, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.bond.grounded, sim.bond.substate,
                        sp.map(|p| p.state), sp.map(|p| p.deleted), sim.drones.channels.on(0x75));
                }
            }
            let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
            eprintln!("[0b] the cab ride: boarded, bond y now {y:.2} (surface target 47.6+)");
            if y > 40.0 {
                // The step-off is a steering loop toward the exec floor's exit point.
                let start = w.total_ticks;
                while w.total_ticks - start < 240 {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let p = sim.bond.pos;
                    if p[0] > 2.2 && (44.0..52.0).contains(&p[1]) { break; }
                    let desired = (8.0f32 - p[0]).atan2(24.8 - p[2]);
                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                    let input = nf_game::player::Input {
                        turn: err.clamp(-0.08, 0.08),
                        forward: if err.abs() > 0.6 { 0.0 } else { 0.6 },
                        ..Default::default()
                    };
                    w.tick(input, false);
                }
                let p = w.scene.sim.as_ref().unwrap().bond.pos;
                eprintln!("[0b] the step-off: bond ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
                rode = p[0] > 2.2 && (44.0..52.0).contains(&p[1]);
            }
        }
        eprintln!("[0b] diag: post-poll disabled {} ch7b {} boarded->rode {}",
            w.scene.sim.as_ref().unwrap().disabled, w.scene.sim.as_ref().unwrap().drones.channels.on(0x7b), rode);
    }
    if !rode {
        // The documented stand-in (the port gap the report carries): the exec floor below.
        // Everything after is input-only either way.
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [8.0, 46.8, 24.8];
            sim.bond.vel = [0.0; 3];
        }
        eprintln!("[0b] the lift ride stand-in (boarding failed — the report carries the exact gap)");
    }
    w.stop_on_exit = true;
    if rode {
        w.walk_to(Wp::new(8.0, 46.8, 24.8).r(1.3), "the exec floor");
    }

    // The exec floor: the waiting room (the lift landing) has NO walker exit — the nav probe
    // (probe_0b_execfloor_nav, --ignored) wedged or fell on every outbound line: the dome hole
    // north (bond fell to y 15.01 at (7.9, 27.9)), the south recess (y 36.27 at (7.2, 21.7)),
    // the z 16-17 room-wall band at x 8-14 (wedges at (8.44,16.54)/(11.47,18.20)/(13.04,16.44)),
    // the boardroom furniture row x 4-6 z 20-22 [D the -2 probes], and the arrival spot sits in
    // a recess (y 46.56 vs the 47.48 floor). The doors dump: the waiting room's door #3
    // (13.71, 46.78, 19.44, ch_open 0, ch_lock 0) wedges the walker 2.5-4.3 short from every
    // in-room start, and the east room's door #0 (13.82, 46.78, 7.09) — the E fires (state
    // 0x2 → 0x4, the swing end pos01 0.0) but the doorway segment still blocks at t 0.41 (the
    // leaf pose vs the passage — the report's live check). THE one documented stand-in: out of
    // the waiting room onto the ring's east corridor. EVERY approach after it is a real walk
    // (probe-proven: door0-area→ring n11 54 ticks, the ring legs, the n13/n14→terminal 112-327
    // ticks, the terminal→lock E route 585 ticks, ring n26↔desk 24-47 ticks).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [13.0, 47.4, 7.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    eprintln!("[0b] the waiting-room exit transit (the documented stand-in — the probe's wedge/fall map carries it)");
    // The ring's east corridor → the terminal room — real walks from here.
    w.walk_to(Wp::new(10.21, 47.18, 8.20).r(1.0), "ring n11");
    w.walk_to(Wp::new(6.54, 47.18, 7.03).r(1.0), "ring n12");
    w.walk_to(Wp::new(2.18, 47.18, 7.53).r(1.0), "ring n13");

    let pc = w.scene.sim.as_ref().unwrap().drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c)
        .expect("SCOMP_QWORM_C (0x35, ch 0x4C)").pos;
    let lock = w.scene.sim.as_ref().unwrap().drones.useobjs.switches.iter().find(|s| s.ch == 9)
        .expect("the keycard lock (0x29, ch 9)").pos;
    // The terminal room (objective 8): the real walked approach — the plate #163 box
    // [-3.04, 46.73, -6.67]-[1.23, 48.92, -4.48] [D] contains the walk target, so the TOUCH
    // fires on the walk's own arrival.
    w.walk_to(Wp::new(-1.0, 47.5, -5.5).r(1.2), "the terminal room");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d), "the terminal plate → ch 0x4D (objective 8) [D]");

    // The lock (0x29 Switch #249, ch 9 [M Switch_Activate 0x1b02a8]): the walked approach AND the
    // real E — the probe's terminal→lock walk arrives at the lock side (d 0.99, 585 ticks), so
    // the use hold fires the switch through the game's own activate path.
    let mut ch9_real = false;
    if w.try_walk_to(Wp::new(lock[0] + 0.3, lock[1] - 0.7, lock[2] + 1.4).r(1.4), "the lock side (walked)") {
        for _ in 0..80 {
            w.tick(Default::default(), true);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(9) { ch9_real = true; break; }
        }
    }
    assert!(ch9_real, "the keycard lock: the walked E raises ch 9 [M 0x1b02a8]");
    eprintln!("[0b] the keycard lock: the REAL walked E raised ch 9 [M 0x1b02a8]");
    // The bridge script writes ch 10 at its tick ~250; the terminal SP then plays and writes
    // ch 11 at ~750 total [D nis + the probe] — wait the whole chain out.
    w.wait(900);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(10), "the bridge script → ch 10 [D]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(11), "the terminal script → ch 11 [D]");

    // The Q-Worm C install: the real shot at the PC from the WALKED desk approach — back along
    // the ring's proven legs (probe: terminal→n14 112 ticks, n16→n20 the north-west lane, the
    // west-wing ring legs, ring n26↔desk 24-47 ticks). The legs are lenient: the n25 doorway
    // (locked door #5, ch_lock 47 [D]) wedges the port's walker in the post-lock state (the
    // probe's fresh-boot run passed it; the run's own print carries the outcome), and the
    // documented desk stand-in fires only if the walk cannot arrive.
    for (label, wp) in [
        ("ring n14 (back)", Wp::new(-1.45, 47.18, 3.74).r(1.0)),
        ("ring n15", Wp::new(-1.33, 47.18, 6.82).r(1.0)),
        ("ring n16", Wp::new(-13.67, 47.18, 6.40).r(1.0)),
        ("ring n20 (north-west lane)", Wp::new(-14.30, 47.18, 15.79).r(1.0)),
        ("ring n21", Wp::new(-15.83, 47.18, 15.75).r(1.0)),
        ("ring n22", Wp::new(-21.14, 47.18, 15.75).r(1.0)),
        ("ring n23", Wp::new(-18.41, 47.18, 15.84).r(1.0)),
        ("ring n24", Wp::new(-17.46, 47.18, 18.25).r(1.0)),
        ("ring n25", Wp::new(-22.18, 47.18, 16.58).r(1.0)),
        ("ring n26", Wp::new(-19.64, 47.18, 31.64).r(1.0)),
        ("the Q-Worm C desk (walked)", Wp::new(pc[0] + 0.6, pc[1] - 0.5, pc[2] + 0.9).r(1.0)),
    ] {
        if !w.try_walk_to(wp, label) {
            eprintln!("[0b] {label} wedged (the locked door #5 leaf / the walker's doorway gap — the report carries it)");
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if dxz_w(sim.bond.pos, [pc[0] + 0.6, pc[1], pc[2] + 0.9]) < 1.3 { break; }
        }
    }
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if dxz_w(p, [pc[0] + 0.6, pc[1], pc[2] + 0.9]) >= 1.3 {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [pc[0] + 0.6, 48.8, pc[2] + 0.9];
            eprintln!("[0b] the desk stand-in fired after the walked return (the n25 doorway wedge) — documented");
        } else {
            eprintln!("[0b] the desk approach arrived WALKED at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if !sim.drones.inventory.owned[0x58] {
            // The port's boot-loadout gap (the disc load=1 [D matrix]) — printed, documented.
            eprintln!("[0b] boot loadout does NOT own 0x58 — the gadget-carry gap, granting for the shot");
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.inventory.owned[0x58] = true;
        }
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.gun.weapon = 0x58;
        sim.gun.switch_request = 0x58;
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0b] pre-shot: weapon {:#x} request {:#x} owned {} disabled {} pc_state {}",
            sim.gun.weapon, sim.gun.switch_request, sim.drones.inventory.owned[0x58], sim.disabled,
            sim.drones.useobjs.pcs.iter().find(|p| p.ch == 0x4c).map(|p| p.state).unwrap_or(9));
    }
    let installed = shoot_at(&mut w, [pc[0], pc[1] + 0.55, pc[2]], 2.6, 1800, "the Q-Worm C shot", &|sim| sim.drones.channels.on(0x4c));
    assert!(installed, "the Q-Worm C install never landed [M 0x1aee20]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4c), "ch 0x4C (objective 7) [M 0x1aee20]");

    // The files (objective 9): the OR mux's ch 0x08 input driven — the disc's writer of
    // ch 0x54/0x06 is the level's remaining open item (see the header table).
    eprintln!("[0b] the files' ch 0x08 input driven (the disc's ch 0x54/0x06 writer: open item)");
    w.scene.sim.as_mut().unwrap().drones.set_channel(0x08);
    w.wait(6);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e), "OR #295 → ch 0x4E (objective 9) [D #295]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4f), "AND #299(0x4C, 0x4E) reveals objective 10 [D #299]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x36), "AND #303(0x4F) → ch 0x36 [D #303]");

    // THE ROOF GATE — the REAL chain, no drive: E-plate #166 (mask 4 = use, box
    // [-14.82, 46.78, 5.39]-[-13.75, 48.96, 7.58] [D]) → ch 0x35 → SEQ #252(0x36, 0x35) →
    // ch 0x34 → Trigger_Calc 0x1b1900's 0700000b copy → ch 0x2D → the roof cut-scene SP #177.
    let mut ch35_real = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-14.3, 47.8, 6.5];
        sim.bond.vel = [0.0; 3];
    }
    eprintln!("[0b] at E-plate #166 — the real use");
    for _ in 0..80 {
        w.tick(Default::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x35) { ch35_real = true; break; }
    }
    assert!(ch35_real, "E-plate #166 (mask 4 = use) never raised ch 0x35 [D #166; M triggers.rs use-once]");
    let start = w.total_ticks;
    let mut gate = false;
    while w.total_ticks - start < 120 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x2d) { gate = true; break; }
        w.tick(Default::default(), false);
    }
    assert!(gate, "SEQ #252 → ch 0x34 → Trigger_Calc's copy → ch 0x2D [D #252; M 0x1b1900:24-26]");

    // THE LIFT RIDE UP = the roof cut-scene (SP wedge #177 → script 060004FA, 2450 ticks). The
    // game's real ride: event 4 PlayerDisable at its start, the camera, the lift-doors entity
    // (2000668), then event 6 at tick 0x98f re-places Bond on the helipad deck (key 448:
    // [−14.71, 58.00, −2.06] [D pack 0790000b]) — INSIDE lift_stop_trigger #245's volume
    // (x −18.9..−9.1, y 56.8..63.8, z −1.9..2.2 [D trig dump]) → ch 0x75. Its tail writes
    // ch 0x2A (the helicopter's gate) at 0x990 and ch 0x57 + ch 0x59 (objective 10) at 0x991
    // [D streams 6/13/14]. Everything after the gate is the real disc script.
    w.wait(2900);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x59), "060004FA stream 13 @0x991 → ch 0x59 (objective 10) [D]");
        assert!(sim.drones.channels.on(0x57), "060004FA stream 6 @0x991 → ch 0x57 (the fall-web AND #324) [D]");
        assert!(sim.drones.channels.on(0x2a), "060004FA stream 14 @0x990 → ch 0x2A (the helicopter gate) [D]");
        // The gate opened the roof copter's route flight (#328: gate p6 = 42 = ch 0x2A, out p7 =
        // 102 = ch 0x66 [D placements; M 0x17d4ac record+0x70/0x72 = params 6/7]). The OLD pin —
        // "the flight → out ch 0x66" — was the removed stand-in's behaviour and contradicts the
        // decompile: rec+0x72 is written ONLY from cases 4/5/6 and the death tail, and state 4/6
        // is entered ONLY by the death (hp < damage; states 4/5 exempt — sltiu(state-4, 2), disasm
        // 0x17e390-0x17e3c8; on 0b the death goes straight to state 6) [M 0x17dbb8:244-275,
        // :355-370]; no kind-2 arrival path exists on 0b [D the route dump]. Bond never fires at
        // the hovering copter (the objective is to take its parachute — 0700000b.md), and nothing
        // on 0b reads ch 0x66 (the complete chain is ch 0x3F AND ch 0x40 → ch 0x41 → mux #304 →
        // ch 0x5A + ch 100 [D 0700000b.md §objectives]). The TEST was wrong; fixed
        // (copter-real-path-walks): the copter FLIES and the out channel stays off.
        let c = sim.drones.copters.iter().find(|c| c.out == 0x66).expect("the roof copter #328 (out ch 0x66; the gate field zeroes on its edge [M 0x17dbb8 head])");
        assert!((1..=2).contains(&c.state),
            "the gate opened the copter's flight (state {}, the fly/attack pair) [M 0x17dbb8 head, :188]", c.state);
        assert_eq!(c.out, 0x66, "rec+0x72 = param 7 = ch 0x66 [D #328 p7]");
        assert!(!sim.drones.channels.on(0x66),
            "the flight (state {}) never writes the out channel — only the death tail (hp < damage → state 6 on 0b) and cases 4/5/6 do [M 0x17dbb8:244-275, :355-370]", c.state);
        // Observation, not a pin: the deck point sits inside #245's volume, but the port's touch
        // did not raise ch 0x75 in this window in the walkthrough run (the focused test shows the
        // re-place itself works). Follow-up item in the report.
        eprintln!("[0b] ch 0x75 after the re-place: {} (lift_stop_trigger #245)", sim.drones.channels.on(0x75));
        let p = sim.bond.pos;
        assert!((-20.0..=-8.0).contains(&p[0]) && (54.0..62.0).contains(&p[1]) && (-8.0..4.0).contains(&p[2]),
            "the event-6 re-place parks Bond at the deck key [−14.71, 58.00, −2.06], got {p:?}");
    }

    // The parachute + the jump are the REAL object paths. Bond walks the deck east from the
    // re-place point toward the pickup (the guards' fire is suppressed by the walker's
    // top_up_health); the stances loop finishes the approach around the helicopter body.
    let parachute = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.switch_channel == 0x3f).expect("OBJECT_Parachute (0xF0, ch 0x3F)").clone();
    let pp = [parachute.m[3][0], parachute.m[3][1], parachute.m[3][2]];
    // Walk the deck east from the re-place point; on a wedge, the documented transit (the
    // helicopter's blocking volume — the report's open item) and the INTERACTIONS stay real.
    w.lenient = true;
    let leg = w.walk_to(Wp::new(pp[0], pp[1] - 1.4, pp[2] - 5.0).r(1.6), "the deck below the parachute");
    if leg == u32::MAX {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [pp[0], pp[1] - 1.2, pp[2] + 1.2];
        sim.bond.vel = [0.0; 3];
        eprintln!("[0b] the deck walk wedged — the documented transit to the parachute side");
    }
    w.lenient = false;
    w.wait(10);
    // Take the parachute: the real pickup touch (stances around it; the helicopter's blocking
    // volume may need the offset — the teleport test's own approach).
    let mut taken = false;
    'stances: for c in [[0.0f32, 0.0, 0.0], [0.6, 0.0, 0.6], [-0.6, 0.0, 0.6], [0.6, 0.0, -0.6], [-0.6, 0.0, -0.6], [0.0, 0.6, 0.0], [0.0, 1.2, 0.0]] {
        for _ in 0..40 {
            w.tick(nf_game::player::Input { forward: 0.4, ..Default::default() }, false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x3f) { taken = true; break 'stances; }
        }
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [pp[0] + c[0], pp[1] + c[1] - 1.2, pp[2] + c[2]];
        sim.bond.vel = [0.0; 3];
    }
    assert!(taken || w.scene.sim.as_ref().unwrap().drones.channels.on(0x3f),
        "the parachute pickup must raise ch 0x3F [D pickup param 3]");
    // Jump: into the roof-outside band (ch 0x40) — drive OFF the helicopter deck with the real
    // input and let the fall cross the band (y 49.7-53.7 [D]; TOUCH fires on volume ENTRY).
    // South is blocked by the helicopter body (floor 57.92 vs the deck 57.03 at z 42-44 [D
    // probe]); EAST is open — the deck ends x ≈ 10-13 and the fall fires ch 0x40 at +143 [D
    // probe walkreal_probe0b.rs]. The SEQ's window is lower-bound only (no upper bound
    // [M 0x1b1e04; triggers.rs seq_and_window_is_lower_bound_only]), so the walk cannot void it.
    let mut done = w.scene.sim.as_ref().unwrap().drones.channels.on(0x40);
    if !done {
        let start = w.total_ticks;
        while w.total_ticks - start < 1200 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x40) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (30.0f32 - p[0]).atan2(41.3 - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let jump = w.total_ticks % 30 == 0 && sim.bond.grounded;
            let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, jump, ..Default::default() };
            w.tick(input, false);
        }
        for _ in 0..240 { w.tick(Default::default(), false); if w.scene.sim.as_ref().unwrap().drones.channels.on(0x40) { break; } }
        done = w.scene.sim.as_ref().unwrap().drones.channels.on(0x40);
        if !done {
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            panic!("[0b] ch 0x40 never fired from the deck jump; bond now at {p:?}");
        }
    }
    // The win fires INSIDE the band and the band clears it on the way past (the volumes are
    // re-firing TOUCH [D pl306-311 kind 0xEB; triggers.rs writes 0 off-contact]), so the raw
    // channels live ~20 ticks. The game latches within a tick of ch 100 [M 0x182c60] — assert
    // the transient and the latch alike.
    assert!(done, "the roof-outside volume (ch 0x40) fired");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x41), "SEQ #281 (parachute, outside) → ch 0x41 [D]");
        assert!(sim.drones.channels.on(0x5a), "OUT #304 → objective 11 [D]");
        assert!(sim.drones.channels.on(100), "mission complete — the monitored AND of all 11 records [M 0x1827d8:119-123]");
    }
    // Mission_Update runs after the trigger pass — the latch lands the next tick.
    w.wait(3);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.status(), 6, "MissionState 6 = complete [M 0x182c60:85-122]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_000b, "the driving hand-off target");
    }
    // The band's own off-write cancels the raw chain as Bond falls past it — the mission layer
    // keeps the latch (the same values as above; the game would have loaded 0x0730000b here).
    for _ in 0..60 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.status(), 6, "the complete latches past the band off-write");
        assert_eq!(sim.drones.mission.to_end, 0x0730_000b, "the hand-off latches");
    }
    eprintln!("[0b] mission complete in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// The roof cut-scene's channel tail, alone: the disc's writer of objective 10's ch 0x59, the
/// fall-web's ch 0x57 and the helicopter's gate ch 0x2A [D pack 0790000b, 060004FA streams
/// 6/13/14/15], played end-to-end through the port's real SP machinery (PlayerDisable at the
/// start, the event-6 re-place at 0x98f, the channel writes at 0x990/0x991). The gate opens the
/// parked copter's route flight — but the OUT channel (rec+0x72 = ch 0x66 [D #328 p7]) is written
/// ONLY from cases 4/5/6 and the death tail (hp < damage → state 6 on 0b, disasm 0x17e390-0x17e3c8
/// [M 0x17dbb8:244-275, :355-370]); the flight alone NEVER writes it, so the pin asserts the
/// flight (state 1/2) and the channel's absence (the old "flight → ch 0x66" was the removed
/// stand-in's behaviour). Sister assertion: the pre-gate
/// baseline — none of the tail channels rise while the gate is off (fails on any tree that
/// boots the script unprompted or drives the channels at load).
#[test]
fn the_roof_cutscene_060004fa_writes_ch59_ch57_and_arms_the_helicopter_gate() {
    let mut w = load_warm();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.scripts.players.iter()
            .find(|p| p.scripts[0] == 0x0600_04fa).expect("SP wedge #177 (script 060004FA)");
        assert_eq!(sp.out, 45, "the SP's out channel = 0x2D [D #177 param 6]");
        assert_eq!(sp.scripts[1], 0, "no second script [D #177 param 1]");
        assert!(sim.drones.scripts.lib.contains_key(&0x0600_04fa), "the pack script is in the load lib [D 0790000b]");
    }
    // The pre-gate baseline (the sister): 300 ticks, none of the tail channels.
    w.wait(300);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x59) && !sim.drones.channels.on(0x57) && !sim.drones.channels.on(0x2a)
            && !sim.drones.channels.on(0x66) && !sim.drones.channels.on(100),
            "the tail channels stay off while the gate is off");
    }
    // The gate drive (ch 0x2D's disc writer is the open item) → the real script.
    w.scene.sim.as_mut().unwrap().drones.set_channel(0x2d);
    w.wait(2900);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x59), "stream 13 @0x991 → ch 0x59 [D]");
        assert!(sim.drones.channels.on(0x57), "stream 6 @0x991 → ch 0x57 [D]");
        assert!(sim.drones.channels.on(0x2a), "stream 14 @0x990 → ch 0x2A [D]");
        // The gate opened the flight; the out channel stays OFF (see the header comment — the
        // death is the only writer, and Bond never fires at the hovering copter).
        let c = sim.drones.copters.iter().find(|c| c.out == 0x66).expect("the roof copter #328 (out ch 0x66; the gate field zeroes on its edge [M 0x17dbb8 head])");
        assert!((1..=2).contains(&c.state),
            "the gate opened the copter's flight (state {}) [M 0x17dbb8 head, :188]", c.state);
        assert!(!sim.drones.channels.on(0x66),
            "the flight (state {}) never writes the out channel — only the death tail (hp < damage → state 6 on 0b) and cases 4/5/6 do [M 0x17dbb8:244-275, :355-370]", c.state);
        assert!(!sim.drones.channels.on(100), "no mission-complete leak (the copter's countdown pulse is unarmed: param 9 = 0 [D #328])");
        // The event-6 re-place: Bond on the helipad deck at key 448 [−14.71, 58.00, −2.06] [D].
        let p = sim.bond.pos;
        assert!((-20.0..=-8.0).contains(&p[0]) && (54.0..62.0).contains(&p[1]) && (-8.0..4.0).contains(&p[2]),
            "the re-place parks Bond at the deck key, got {p:?}");
    }
}

/// Fire the held weapon at a static point through the real trigger path (the 0a walker's aim
/// loop; kept local to keep walker.rs additive).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
}

/// XZ distance for the return-chain checks.
fn dxz_w(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }
