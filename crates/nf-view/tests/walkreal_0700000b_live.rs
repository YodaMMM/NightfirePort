//! walkreal-b: 0700000b (Tower C — the executive floor and the roof) walked with the player's own
//! movement — no teleports past a blocker; the only channel pokes left are the warm walk's
//! carried-channels model (the RamSave ch 0x46-0x4a block [M 0x195670]). The mission's objective records [L ee_snapshot 0x2a3f78/0x2a3fc0/0x2a4008
//! + M Mission_MonitorObjectives 0x1827d8]: mission 07000009's monitored AND spans 11 records —
//! obj 1 (label 0x04000016) completes on the INVERTED alarm channel ch 0x61 (flags word @+0x10 =
//! 2), objs 2-5 (0x04000017-1a) on the carried ch 0x46-0x49, and THIS level's six:
//!
//! | obj | label      | ch    | how the walkthrough completes it |
//! |-----|------------|-------|----------------------------------|
//! | 6   | 0x0400001b | 0x4B  | REAL: the spawn TOUCH_ONCE plates #165/#247/#284+#176 [D trig dump] |
//! | 7   | 0x0400001c | 0x4C  | REAL: the Q-Worm gadget shot into SCOMP_QWORM_C #297 [M PCQWorm_Update 0x1aee20]; the boot loadout owns weapon 0x58 [D weapon-matrix 0700000b load=1] |
//! | 8   | 0x0400001d | 0x4D  | REAL: the walked terminal-room plate #163 [D] |
//! | 9   | 0x0400001e | 0x4E  | REAL on BOTH walks: the terminal script 06000035 stream 8 spawns at its tick 0x1e5 the type-0xEA TouchOnce (hash 0x2000ab5, params [6, 4, ..] = out ch 6, mask 4 = use [D nis dump; M Script_CreateEntity 0x1d8968 case 0xEA → Trigger_TouchOnce]) — the level's ONLY ch-0x06 writer [D: the trigger census (60 rows) has no out 0x6, no level/pack script Event-0xD writes 6, no constant switch_channels[6] store in the decompile]; E-firing it → ch 0x06 → OR #295 → ch 0x4E. The OR's other arm (OUT #294 mirrors ch 0x54 into ch 0x08) is writer-less on 0b, both packs and 0a [D the same sweeps] — the old warm-walk ch 0x08 poke modelled a writer that does not exist and is deleted |
//! | 10  | 0x040001f  | 0x59  | REAL chain: ch 0x4F = AND #299(0x4C, 0x4E) [D] → E-plate #166 (mask 4 = use) → ch 0x35 → SEQ #252(0x36, 0x35) → ch 0x34 → `Trigger_Calc` 0x1b1900's per-level copy → ch 0x2D → SP wedge #177 (060004FA) plays [M 0x1dbda8 gate law] → stream 13 @0x991 writes ch 0x59 |
//! | 11  | 0x0400020  | 0x5A  | REAL: the parachute pickup touch (ch 0x3F) + the roof-outside volume (ch 0x40) → SEQ #281 → ch 0x41 → OUT #304 → ch 0x5A + ch 100 [D] |
//!
//! ch 100 fires when the monitored AND of ALL 11 records holds [M 0x1827d8:119-123] → MissionState 6
//! + `LevelToEndTo(0x0700000b)` = 0x0730000b [M Mission_Update 0x182c60:85-122]. The carried
//! channels 0x46-0x4a model the 0a exit's RamSave carry [M Player_RamSave 0x195670 — the full
//! 256-byte switch_channels block rides the save]. The balcony→exec lift ride is the game's own
//! route on real input (the boarding hop, the carry on the script-entity collision set [M
//! 0x194188 → Control_InheritVelocity 0x133e88], the step-off) and the exec office goes through
//! the burned keypad's door (the Laser watch 0x4E on Lock #178 → ch 0x2F → door #151 [M
//! 0x1ae3d8 case 0; D #178/#151]) — and the waiting-room exit transit (doors #3 and #0) is
//! WALKED on the E-opened leaves: the pick is the game's 0x800 arm `1.5·r_door + r_pick` [M
//! Collide_PickObj 0x1eae80:152-168], the anti-crush holds the leaf while Bond stands at the E spot
//! [M 0x135980:85-97], so the walk goes THROUGH the swinging doorway (probe_0b_doors: both
//! crossings pass, the leaves complete once Bond is through); the deck→parachute leg walks
//! for real and the take is WALKED: the game's reach is the deck slot south-west of the
//! pickup's housing (the take-ray threads the housing's open south face [D disc tris; M
//! Pickup_Update 0x18e148:202-220]), and the files-plate placements are interaction
//! measurement spots, not route transits.

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

/// THE FRESH LOAD: the level's own boot — NO channel pokes. The carried objectives' channels
/// (records 1-5: ch 0x61 inverted + 0x46-0x49 [L ee_snapshot 0x2a3f78..0x2a407f]) read OFF at
/// this boot: `Mission_Init` 0x182270 wipes only the CURRENT + FUTURE parts' condition channels
/// to their init values and leaves the previous parts' alone (the record loop's part<order arm
/// only reads the count [M 0x182270 disasm 0x182488-0x1824f8; the disc's init values are 0
/// [L ee_snapshot]]), and their writers live on 09/0a — out of a fresh 0b load's reach. The
/// mission AND [M 0x1827d8:119-123] therefore cannot fold on a fresh boot; the END rides OUT
/// #304's OWN ch 100 write (the mirror of ch 0x41 into ch 0x5A AND ch 0x64 [D 0700000b.md
/// §objectives]), which Mission_Update latches from ANY writer [M 0x182c60:23-28].
fn load_fresh() -> Walker {
    Walker::new(nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load"))
}

/// The shared walkthrough — both walks reuse the same legs: the spawn plates → the arrival
/// cut-scene → the cab ride → the exec floor → the waiting-room exit transit (the one kept
/// stand-in) → the ring → the terminal → the lock's walked E → the Q-Worm install → the files
/// (objective 9, the REAL writer on both walks) → the roof gate → the roof cut-scene → the
/// parachute → the jump → the mission latch. `fresh` selects only the mission-latch wording
/// (a fresh boot's AND cannot fold — the end rides OUT #304's own ch 100 write).
fn walk_to_the_mission_end(mut w: &mut Walker, fresh: bool) {
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
                // The step-off: the proven east steer — off the cab's east edge (x 1.6) onto the
                // dock band (51.8). (The south-edge variant wedges on the cab's raised south
                // section at z 23.45 once the cab docks; the dock band is an ISLAND — the pit
                // x 3-8 z 23-25.5, the x 2-3.5 hole and the dome hole x 7-9.5 z 26.5+ surround
                // it [D the floor grid in probe_0b_doors] — no walked descent exists.)
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
    // No stand-in: the cab ride IS the game's route (the boarding hop, the carry on the
    // script-entity collision set, the step-off south — all real input above). The fixed 60 Hz
    // + the bit-exact RNG make the boarding deterministic: once green, always green.
    assert!(rode, "the paternoster cab ride must board and step off on real input [M 0x194188 → Control_InheritVelocity 0x133e88; D the cab keys y 11.0 → 48.44]");
    w.stop_on_exit = true;
    w.walk_to(Wp::new(8.0, 46.8, 24.8).r(1.3), "the exec floor (the proven z≈25 leg)");

    // The exec floor: the waiting room's walker exit is the two-door transit — door #3
    // (13.71, 46.78, 19.44 [D]) into the east room, south down the room, and door #0
    // (13.82, 46.78, 7.09 [D]) onto the ring's east corridor. The lane east of the room's
    // barrier is the game's own nav path 0 'Navigation' [D n54 (12.21, 19.68) → n53 (14.13,
    // 18.35) → n52 (15.85, 16.77) → n50 (17.10, 14.00) → n49 (17.03, 10.83) → n19 (16.23,
    // 8.07) → n18 (14.50, 6.54) → n17 (13.18, 6.63) → n11 (10.21, 8.20)]. THE ONE KEPT
    // STAND-IN (the dock→east-approach hop, ~6 m): the waiting room's floor is split — the
    // dock band (51.8) is an island (the pit x 3-8 z 23-25.5, the x 2-3.5 hole, the dome hole
    // x 7-9.5 z 26.5+ [D the floor grid in probe_0b_doors]; the pit's SW rim line descends
    // into the bowl and its east rim walls the walker at body height — walked and wedged),
    // and the main floor's x 4.5-6.5 barrier (the N-S slot at x 5.0 z 17.5-23, the furniture
    // 48.2-48.8 at x 6-9.5 z 18-21.5, the wall band x 8-14 z 16-17 [D the -2 probes]) leaves
    // only the z 17.0 seam, which the wall band closes at x 9.5+. The nav lane's own n41→n55
    // hop rides furniture tops the AI ignores (n55 sits on the 48.7 band [D the grid]).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [12.21, 47.4, 19.68];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(10);
    // The doors: disc swing doors (kind 0, 3-key symmetric anims — the closed pose at key 1,
    // the open ends at keys 0/2 [D the 0x19 keys]); the E-open runs `dir = sign((inv_closed ·
    // activator).z)·(+0x5C)` [M Door_Activate 0x135be0:136-146], +0x5C from `Door_SetupSwing`
    // 0x135440:65-80. The pick is the game's 0x800 arm `1.5·r_door + r_pick` [M Collide_PickObj
    // 0x1eae80:152-168 — the prior branch's radius-alone change was the 0x200/0x201 arm,
    // :91-107, and is REVERTED], so Bond at the E spot IS a contact, and `Door_ReactToHit`'s
    // anti-crush [M 0x135980:85-97 — `f = local.z·(+0x5C)`, negated past half travel (+0x50 >
    // 0.5)] holds the leaf near the lz = 0 angle while he stands there (a 3-tick limit cycle,
    // the NF_DOORTRACE capture: lz falls +0.353 → −0.028 as pos01 0.48 → 0.34, reversal at the
    // crossing, the cycle 0.3448 ↔ 0.3966). The game's flow — and this test — never waits it
    // out: the walker presses E and WALKS THROUGH the swinging doorway (the leaf yields /
    // closes behind — probe_0b_doors: both crossings pass, +70/+73 ticks, both leaves complete
    // to their open ends 1.000/0.000 once Bond is through).
    let d3 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 13.71).abs() < 0.5 && (d.pos[2] - 19.44).abs() < 0.5)
        .expect("the waiting-room door #3 [D]");
    w.walk_to(Wp::new(12.21, 47.18, 19.68).r(0.6), "nav n54 (door #3's approach)");
    w.use_door(d3, "the waiting-room door");
    w.walk_to(Wp::new(13.4, 47.18, 18.3).r(0.6), "through door #3's opening");
    w.walk_to(Wp::new(14.2, 47.18, 17.8).r(0.8), "clearing the swung leaf south");
    w.walk_to(Wp::new(15.85, 47.18, 16.77).r(1.0), "nav n52 (the east room)");
    w.walk_to(Wp::new(17.10, 47.18, 14.00).r(1.0), "nav n50 (the east room south)");
    w.walk_to(Wp::new(17.03, 47.18, 10.83).r(1.0), "nav n49");
    w.walk_to(Wp::new(16.23, 47.18, 8.07).r(1.0), "nav n19");
    let d0 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5)
        .expect("the east room's ring door #0 [D]");
    w.walk_to(Wp::new(14.50, 47.18, 6.54).r(0.7), "nav n18 (door #0's east side)");
    w.use_door(d0, "the ring corridor door");
    w.walk_to(Wp::new(13.18, 47.18, 6.63).r(0.7), "nav n17 (through door #0)");
    eprintln!("[0b] the waiting-room exit transit WALKED (both doors E-opened on the route)");
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

    // The Q-Worm C install: the real shot at the PC from the WALKED desk approach. The game's
    // route into the exec office (the desk's room) is the keypad: the ring's west wing ends at
    // n22 (−21.14, 15.75), ~1.0 m from the keypad Lock #178 "Lock - KeypadLocked" (−22.17, 48.12,
    // 15.44 [D]) — the Laser watch's beam (weapon 0x4E) burns it: `Lock_Update` 0x1ae3d8 case 0
    // advances the burn on the laser-family hit (0x4E −3, 0x4F −10 per hit) and Lock_Unlock writes
    // `switch_channels[47] = 1` [M 0x1ae3d8 case 0 + :194-195] — which unlocks door #151
    // "door_waitingroom_exec_office" (ch_lock 47 [D #151]; door.rs `is_locked` = the channel OFF
    // [M 0x136810:10-16]). The E at the doorway is the walker's own stuck reaction (the game's
    // 1.0-sphere pick [M Player_Activate 0x1a5170] → Door_Activate).
    for (label, wp) in [
        ("ring n14 (back)", Wp::new(-1.45, 47.18, 3.74).r(1.0)),
        ("ring n15", Wp::new(-1.33, 47.18, 6.82).r(1.0)),
        ("ring n16", Wp::new(-13.67, 47.18, 6.40).r(1.0)),
        ("ring n20 (north-west lane)", Wp::new(-14.30, 47.18, 15.79).r(1.0)),
        ("ring n21", Wp::new(-15.83, 47.18, 15.75).r(1.0)),
        ("ring n22 (the keypad side)", Wp::new(-21.14, 47.18, 15.75).r(1.0)),
    ] {
        w.walk_to(wp, label);
    }
    // THE KEYPAD BURN — the Laser watch 0x4E through the real fire path (the interactables_0700000b
    // keypad test's pattern): switch via the game's own switch request, aim the head at the lock,
    // hold the trigger until `Lock_Unlock` lands ch 0x2F.
    let lk = w.scene.sim.as_ref().unwrap().drones.locks.iter().position(|l| l.out == 47)
        .expect("the keypad lock (0x2E, out ch 0x2F)");
    let lp = w.scene.sim.as_ref().unwrap().drones.locks[lk].pos;
    {
        let input = nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() };
        for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false); }
    }
    let mut burned = false;
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().drones.locks[lk].burned { burned = true; break; }
        let (head, target) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.head(), lp) };
        let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.yaw = dir[0].atan2(dir[2]);
            sim.bond.pitch = (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        }
        w.scene.sim.as_mut().unwrap().step_bond(Default::default(), true, false, false);
    }
    assert!(burned, "the Laser watch burns the keypad [M Lock_Update 0x1ae3d8 case 0]");
    w.wait(5);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x2f),
        "the open writes switch_channels[47] [M 0x1ae3d8:194-195]");
    eprintln!("[0b] the keypad burned — ch 0x2F up, door #151 unlocked [M 0x1ae3d8]");
    // Through the burned keypad's doorway to the desk: the E fires through the walker's own
    // stuck reaction, then the flat ring legs to the office.
    for (label, wp) in [
        ("ring n23", Wp::new(-18.41, 47.18, 15.84).r(1.0)),
        ("ring n24", Wp::new(-17.46, 47.18, 18.25).r(1.0)),
        ("ring n25 (the exec-office doorway)", Wp::new(-22.18, 47.18, 16.58).r(1.0)),
        ("ring n26", Wp::new(-19.64, 47.18, 31.64).r(1.0)),
        ("the Q-Worm C desk (walked)", Wp::new(pc[0] + 0.6, pc[1] - 0.5, pc[2] + 0.9).r(1.0)),
    ] {
        w.walk_to(wp, label);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if dxz_w(sim.bond.pos, [pc[0] + 0.6, pc[1], pc[2] + 0.9]) < 1.3 { break; }
        }
    }
    {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        assert!(dxz_w(p, [pc[0] + 0.6, pc[1], pc[2] + 0.9]) < 1.3,
            "the desk approach must be the walked route: the burned keypad's door #151 → n25 → n26 [M 0x1ae3d8; D #151/#178], got {p:?}");
        eprintln!("[0b] the desk approach arrived WALKED at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    }
    {
        // The disc's island arm equips the Q-Worm at level load (`EquipWeapon(BL, 0x58, 0)` at
        // 001a8494 — the 09 kit the 0a/0b arms fall into on a cold boot [M 0x1a7ef8 cases
        // 0x7000009/0a/0b; the row decoded from /ACTION.ELF by loadout_disc_rows_live]), so the
        // boot loadout OWNS it — hard-pinned, no poke: a loadout that drops the row fails here.
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.inventory.owned[0x58],
            "[0b] the boot loadout does not own the Q-Worm 0x58 — the disc's island kit equips it \
             at level load [M 0x1a7ef8 case 0x7000009/0a/0b, equip 001a8494]");
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

    // The files (objective 9) — the REAL writer on BOTH walks: the terminal script 06000035
    // stream 8 spawns at its tick 0x1e5 the type-0xEA TouchOnce (hash 0x2000ab5, params
    // [6, 4, ..] = out ch 6, mask 4 = use [D nis dump; M Script_CreateEntity 0x1d8968 case 0xEA
    // → Trigger_TouchOnce]; the port's per-tick script-trigger rebuild lives at drone.rs's
    // control-list head). The writer census [D]: the trigger dump (60 rows) has NO out 0x6, no
    // level or pack script Event-0xD writes ch 6 (0700000b.bin + 0780000b/0790000b), 0a carries
    // none, and the decompile has no constant switch_channels[6]/[0x54] store — the spawned
    // plate is the level's ONLY ch-0x06 writer, and the warm route walks the same rooms, so
    // BOTH walks E-fire it: ch 0x06 → OR #295 → ch 0x4E. (The OR's other arm: OUT #294 mirrors
    // ch 0x54 into ch 0x08, but ch 0x54 has NO writer anywhere in this level's data [D the same
    // sweeps] — the old warm-walk ch 0x08 poke modelled a writer that does not exist; deleted.)
    // The route back from the desk: the proven ring legs n26 → n25 (the doorway) → n22 → n21 →
    // n20 → n16 (the n24 side-loop is skipped: the direct n25→n24 line crosses the wall corner
    // at x ≈ −19.6 [D the run dump: cel 7 at 27 %]), then the terminal lane in reverse
    // n15 → n14, then the terminal room and the dome floor south to the plate. The dome's laser
    // beams #270-277 are TOUCH plates out ch 0x0D (locked on ch 0x3B — no writer in the census,
    // so they stay armed): crossing raises ch 0x0D → OR #290 → ch 0x0E, which nothing
    // objective-bearing reads [D the census]; the beams are triggers, not collision.
    {
        for (label, wp) in [
            ("files n26→n25 (the doorway)", Wp::new(-22.18, 47.18, 16.58).r(1.1)),
            ("files n25→n22 (the keypad corner)", Wp::new(-21.14, 47.18, 15.75).r(1.0)),
            ("files n22→n21 (the corridor)", Wp::new(-15.83, 47.18, 15.75).r(1.0)),
            ("files n20", Wp::new(-14.30, 47.18, 15.79).r(1.0)),
            ("files n16", Wp::new(-13.67, 47.18, 6.40).r(1.0)),
            ("files n15", Wp::new(-1.33, 47.18, 6.82).r(1.0)),
            ("files n14", Wp::new(-1.45, 47.18, 3.74).r(1.0)),
        ] {
            w.walk_to(wp, label);
        }
        let spot = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
            .find(|t| t.script.is_some() && t.out == 6 && !t.gone)
            .map(|t| (t.centre(), t.lo, t.hi));
        let mut fired = false;
        if let Some((c, lo, hi)) = spot {
            eprintln!("[0b] the spawned files use-plate at ({:.2},{:.2},{:.2}) box ({:.2},{:.2},{:.2})..({:.2},{:.2},{:.2})",
                c[0], c[1], c[2], lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            let room = w.try_walk_to(Wp::new(-1.0, 47.5, -5.5).r(1.2), "files the terminal room (from n14)");
            let walked = w.try_walk_to(Wp::new(c[0], lo[1] - 0.4, c[2]).r(1.4), "the spawned files use-plate");
            eprintln!("[0b] the files approach: terminal room {} plate walked {}", room, walked);
            // THE SISTER: objective 9 is still open at the writer — ch 0x06 and the OR's output
            // ch 0x4E are OFF before the E fires. The plate is mask 4 = use, so the touch path
            // cannot fire it (mask & 8 == 0 → the Trigger_Hit fallback is false [M 0x1b1fc8:161-163])
            // and no other writer exists [D the census above].
            {
                let sim = w.scene.sim.as_ref().unwrap();
                assert!(!sim.drones.channels.on(6) && !sim.drones.channels.on(0x4e),
                    "objective 9 must be open before the files writer fires [D the census; M 0x1b1fc8 fallback]");
            }
            // The E hold at the plate. KEPT test-setup placement (not a route transit): the
            // walked approach above is the route; these stances enumerate interaction positions
            // AROUND the plate for the E hold (the box sits at console height — the y clamp keeps
            // Bond in the use band), so the subject is the use-plate's channel law, not movement.
            'stances: for d in [[0.0f32, 0.0, 0.0], [0.5, 0.0, 0.0], [-0.5, 0.0, 0.0], [0.0, 0.0, 0.5], [0.0, 0.0, -0.5], [0.0, 0.6, 0.0]] {
                {
                    let sim = w.scene.sim.as_mut().unwrap();
                    sim.bond.pos = [c[0] + d[0], (lo[1] - 0.4 + d[1]).max(lo[1] - 0.6), c[2] + d[2]];
                    sim.bond.vel = [0.0; 3];
                }
                for _ in 0..40 {
                    w.tick(Default::default(), true);
                    if w.scene.sim.as_ref().unwrap().drones.channels.on(6) { fired = true; break 'stances; }
                }
            }
        }
        assert!(fired, "the spawned files use-plate (06000035 s8 @0x1e5, out ch 6, mask 4) never raised ch 6 [D; M 0x1d8968 0xEA arm]");
    }
    w.wait(6);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e), "OR #295 → ch 0x4E (objective 9) [D #295]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4f), "AND #299(0x4C, 0x4E) reveals objective 10 [D #299]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x36), "AND #303(0x4F) → ch 0x36 [D #303]");

    // THE ROOF GATE — the REAL chain, no drive: walk BACK from the files plate (by the
    // terminal) north to the terminal room, then the proven terminal lane n14 → n15 → n16 to
    // E-plate #166 (mask 4 = use, box [-14.82, 46.78, 5.39]-[-13.75, 48.96, 7.58] [D]) → ch 0x35 →
    // SEQ #252(0x36, 0x35) → ch 0x34 → Trigger_Calc 0x1b1900's 0700000b copy → ch 0x2D →
    // the roof cut-scene SP #177.
    let mut ch35_real = false;
    {
        // Both walks end the files section at the spawned plate (by the terminal), so the
        // return is ONE lane: the dome walk north, then the proven n14 → n15 → n16 legs.
        let _ = w.try_walk_to(Wp::new(-1.0, 47.5, -5.5).r(1.2), "return the terminal room (from the dome)");
        for (label, wp) in [
            ("return n14", Wp::new(-1.45, 47.18, 3.74).r(1.0)),
            ("return n15", Wp::new(-1.33, 47.18, 6.82).r(1.0)),
            ("return n16", Wp::new(-13.67, 47.18, 6.40).r(1.0)),
        ] {
            w.walk_to(wp, label);
        }
        w.walk_to(Wp::new(-14.3, 47.4, 6.5).r(1.0), "E-plate #166 side (walked)");
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
    // [−14.71, 58.00, −2.06] [D pack 0790000b]). The deck key is OUTSIDE lift_stop_trigger
    // #245's volume — the trigger's own box is z −1.88..2.15 [D trig dump] and the key's z
    // −2.06 sits 0.18 south of its z face (the port's parsed 12-tri glist matches the box
    // bounds exactly, and the toucher at the key misses the mesh while z −1.5 inside touches
    // it [probe on this tree]) — so the old "INSIDE #245's volume → ch 0x75" reading was
    // wrong and ch 0x75 does not fire at the re-place; the trigger is the shaft-top stop the
    // player crosses on a LIFT ride, not at the cut-scene's deck placement. Its tail writes
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
        // Observation, not a pin: ch 0x75 stays off here — the deck key [−14.71, 58.00, −2.06]
        // [D pack 0790000b] is OUTSIDE #245's box (z −1.88..2.15 [D trig dump]; the port's
        // parsed glist matches the box bounds and misses the key by 0.18 in z, probe-verified
        // on this tree). Whether the DISC raises ch 0x75 for a player who walks from the deck
        // key into the shaft-top volume is a live check (the report's); the walkthrough does
        // not need it (the cut-scene already placed Bond on the roof).
        eprintln!("[0b] ch 0x75 after the re-place: {} (the deck key is outside #245's box [D])", sim.drones.channels.on(0x75));
        let p = sim.bond.pos;
        assert!((-20.0..=-8.0).contains(&p[0]) && (54.0..62.0).contains(&p[1]) && (-8.0..4.0).contains(&p[2]),
            "the event-6 re-place parks Bond at the deck key [−14.71, 58.00, −2.06], got {p:?}");
    }

    // The parachute + the jump are the REAL object paths. Bond walks the deck east from the
    // re-place point toward the pickup (the guards' fire is suppressed by the walker's
    // top_up_health); the take is the walked slot below.
    let parachute = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.switch_channel == 0x3f).expect("OBJECT_Parachute (0xF0, ch 0x3F)").clone();
    let pp = [parachute.m[3][0], parachute.m[3][1], parachute.m[3][2]];
    // The deck east from the re-place point IS the game's route and it WALKS: the walkthrough
    // arrives at the deck below the pickup in both walks (deterministic 60 Hz — once green,
    // always green), so no stand-in: the arrival is a hard assert. (The prior session's wedge
    // transit is gone; the deck's full extent was never a blocker on this tree.)
    w.walk_to(Wp::new(pp[0], pp[1] - 1.4, pp[2] - 5.0).r(1.6), "the deck below the parachute");
    w.wait(10);
    // Take the parachute: the WALKED slot. The game's reach is NOT the north side (the old
    // target z +1.2 ground Bond EAST along the pickup's low box — the run dump's wedge at
    // (3.61, 58.07, 41.34), 1.1 m east of the take zone, mis-read at the time as "the
    // helicopter body's volume": the copter never parks here — it flies its route away from
    // the pad (state 1, (66, 57, 64) at +3600 ticks [D probe on this tree])). The disc's own
    // geometry [D the level's collision triangles]: the pickup #280 (2.53, 58.45, 42.34) hangs
    // over a low box (top ≈57.92, x 2.2-3.8, z 42.6-43.2) inside an open-fronted housing
    // (roof ~59.3-60.9, east wall z 44.04); the housing's south face is OPEN at x ≤ ~2.8
    // above the box, so from the deck at x 2.0-2.5, z 41.0-41.5 the take-ray threads the open
    // face to the pickup centre. The game's touch law [M Pickup_Update 0x18e148:202-220]:
    // `Vec_SqDist3D(pickup+0x80, player+0x30) < (radius+1.1)²` (radius 0.69 [D model bounds]
    // → 1.79) AND `Collide_LineOfSight(player+0x80 → pickup+0x80, 0x721)` — one-sided rays
    // [M Intersect_RayGeom 0x10a8c8:168], surface classes 0xd/0xe skipped. The walk into the
    // slot passes both gates on real input; the take raises ch 0x3F [M Pickup_Handler
    // 0x18e650:330-333 writes switch_channels[param 3] = 1; D #280 p3 = 0x3F].
    w.walk_to(Wp::new(pp[0] - 0.3, pp[1] - 1.4, pp[2] - 1.0).r(1.0), "the parachute take slot (walked)");
    let mut taken = w.scene.sim.as_ref().unwrap().drones.channels.on(0x3f);
    for _ in 0..120 {
        if taken { break; }
        w.tick(nf_game::player::Input { forward: 0.4, ..Default::default() }, false);
        taken = w.scene.sim.as_ref().unwrap().drones.channels.on(0x3f);
    }
    assert!(taken,
        "the walked take from the deck slot must raise ch 0x3F [D #280 p3; M 0x18e148:202-220 + 0x18e650:330-333]");
    // Jump: into the roof-outside band (ch 0x40) — drive OFF the deck with the real
    // input and let the fall cross the band (y 49.7-53.7 [D]; TOUCH fires on volume ENTRY).
    // South is blocked by the pickup's housing (the low box top 57.92 vs the deck 57.03 at
    // z 42-44 [D probe]); EAST is open — the deck ends x ≈ 10-13 and the fall fires ch 0x40 at +143 [D
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
        if fresh {
            assert!(sim.drones.channels.on(100), "mission complete — OUT #304's own ch 100 write [D]; Mission_Update latches ch 100 from any writer [M 0x182c60:23-28] (the carried records' AND cannot fold on a fresh boot [M 0x1827d8:119-123 + 0x182270])");
        } else {
            assert!(sim.drones.channels.on(100), "mission complete — the monitored AND of all 11 records [M 0x1827d8:119-123]");
        }
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

/// The warm mid-mission walk (the carried channels poked, as the RamSave models): the original
/// walkthrough, now the `fresh = false` arm of the shared body.
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
    walk_to_the_mission_end(&mut w, false);
}

/// THE FRESH-SPAWN WALK: a FRESH load of 0b (the level's own boot, no channel pokes) walked to
/// the mission's end on real input. Every objective the level can set is set by the game's own
/// writers: obj 6 the spawn plates [D #176], obj 7 the Q-Worm install [M 0x1aee20], obj 8 the
/// terminal plate [D #163], obj 9 the TERMINAL SCRIPT'S SPAWNED use-plate (06000035 s8 @0x1e5,
/// out ch 6, mask 4 [D; M 0x1d8968 0xEA arm]) → ch 0x06 → OR #295, obj 10 the roof cut-scene's
/// stream 13 [D], obj 11 the parachute SEQ [D #281/#304]. The carried records 1-5 (ch 0x46-0x49)
/// stay OFF — their writers live on 09/0a [M Mission_Init 0x182270 leaves previous parts
/// untouched; L ee_snapshot init values 0] — so the monitored AND cannot fold; the mission end
/// rides OUT #304's own ch 100 write [D] and Mission_Update's latch [M 0x182c60].
#[test]
fn walk_fresh_spawn_to_the_mission_end() {
    let mut w = load_fresh();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 28.92).abs() < 1.5 && (13.0..17.0).contains(&p[1]), "the balcony start [D], got {p:?}");
        // The fresh boot leaves the carried channels OFF — the no-poke proof.
        for c in [0x46u32, 0x47, 0x48, 0x49] {
            assert!(!sim.drones.channels.on(c), "the fresh boot leaves carried ch {c:#x} off [M 0x182270 previous parts untouched; L ee_snapshot 0x2a3f78+: record init values 0]");
        }
        assert!(!sim.drones.channels.on(0x61), "no alarm at the fresh boot [L ee_snapshot: the inverted obj-1 record reads complete with ch 0x61 clear]");
        eprintln!("[0b-fresh] boot owned[0x58] = {}", sim.drones.inventory.owned[0x58]);
    }
    walk_to_the_mission_end(&mut w, true);
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

