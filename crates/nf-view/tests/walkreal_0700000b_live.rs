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
//! 256-byte switch_channels block rides the save]. The balcony→exec leg is the game's own route on
//! real input: the boarding hop, the carry on the script-entity collision set [M 0x194188 →
//! Control_InheritVelocity 0x133e88], E plate #327 at the top of the cab's travel, the duct past
//! the exterior grill, and E on the floor grate #98 into the waiting room (plate #353, ch 0x76).
//! The real-damage walks keep their tuned step-off and one placement. The exec office goes through
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
fn load_fresh() -> Walker { load_fresh_at(1) }

/// `load_fresh` at `difficulty` (1 Operative = the old load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]). The same
/// NO-channel-poke fresh boot — the carried objectives' channels stay OFF (see the doc above).
fn load_fresh_at(difficulty: u32) -> Walker {
    let w = if difficulty == 1 {
        Walker::new(nf_view::load_with_carry(&iso(), "0700000b.bin", false, None).expect("load"))
    } else {
        Walker::new(nf_view::load_with_carry_difficulty(&iso(), "0700000b.bin", false, None, difficulty)
            .expect("load 0700000b"))
    };
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    w
}

/// The fresh boot's own pins (the head of `walk_fresh_spawn_to_the_mission_end`), shared by the
/// real-damage walk so its loader can add the fight flags before the route runs.
fn fresh_boot_asserts(w: &Walker) {
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
}

/// The real-health lane sprint (the deck-sprint pattern generalized): the terminal-room GT
/// pair #174/#175 arms the moment the dome fence fires ch 0x0D (OR #290 → ch 0x0E = p4 14 [D
/// #290; the level census]) — OR the files take itself (ch 0x06, the OR's other arm) — and
/// fires on the 0.1 s cadence along the eased barrel [M 0x180498 case 4]: every tick in the
/// pair's lane is a fire tick its ease can land. The player's answer is the RUN: the walk_to
/// legs' node pauses are the slow part, so the armed transits drive the raw stick loop
/// (forward 1.0, the turn clamp) straight through — the same real-input shape the deck sprint
/// uses. Returns false on the timeout (the caller judges).
fn lane_sprint(w: &mut Walker, target: [f32; 3], arrive: f32, max_ticks: u32, label: &str) -> bool {
    let start = w.total_ticks;
    while w.total_ticks - start < max_ticks {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        if walker::dxz(p, target) < arrive { return true; }
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    }
    eprintln!("[0b] {label}: the lane sprint timed out at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    false
}

/// The shared walkthrough — both walks reuse the same legs: the spawn plates → the arrival
/// cut-scene → the cab ride → the exec floor → the waiting-room exit transit (the one kept
/// stand-in) → the ring → the terminal → the lock's walked E → the Q-Worm install → the files
/// (objective 9, the REAL writer on both walks) → the roof gate → the roof cut-scene → the
/// parachute → the jump → the mission latch. `fresh` selects only the mission-latch wording
/// (a fresh boot's AND cannot fold — the end rides OUT #304's own ch 100 write). `real` selects
/// the real-health deck crossing (the sprint — see the deck leg).
fn walk_to_the_mission_end(mut w: &mut Walker, fresh: bool, real: bool, rocket_pin: bool) {
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
            // Let go of the stick and let the walk's residual velocity die out (the former
            // velocity write).
            w.wait(20);
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
            if real {
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
            } else {
            // Stand on the cab; the ride carries Bond ([M] ground_motion, lifts_ride's pin). Near
            // the top the exterior vent comes level with the roof: E plate #327 (mask 4, out ch
            // 0x77 [D]) works the grill #326, and the duct behind it leads into the building.
            let start = w.total_ticks;
            let mut used = false;
            while w.total_ticks - start < 1600 {
                let cy = cab_y(&w).unwrap_or(0.0);
                let (p, yaw) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw) };
                if p[2] < 21.5 && p[1] > 50.0 { rode = true; break; }
                if p[1] < 49.0 { w.tick(Default::default(), false); continue; }
                let desired = (-0.55f32 - p[0]).atan2(21.0 - p[2]);
                let err = walker::wrap_pi(desired - yaw);
                let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.4 { 0.0 } else { 0.5 }, ..Default::default() };
                let e = p[1] > 52.5 && w.total_ticks % 6 == 0;
                w.tick(input, e);
                if w.scene.sim.as_ref().unwrap().drones.channels.on(0x77) { used = true; }
            }
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            eprintln!("[0b] the cab top: bond ({:.2},{:.2},{:.2}) plate used {used}", p[0], p[1], p[2]);
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

    if real {
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
    } else {
    // The duct behind the grill runs north, then east down to the interior grill #98 (8.57, 49.73,
    // 19.60), which E sweeps open [M SS_Activate 0x1a9ea8]; the drop past it lands in the waiting
    // room on the east side of its barrier, crossing plate #353 (out ch 0x76) on the way.
    for (x, y, z) in [(-0.6, 53.6, 21.0), (-0.5, 53.6, 19.6), (1.5, 53.5, 19.5), (4.5, 53.1, 19.4), (7.0, 52.9, 19.75)] {
        w.walk_to(Wp::new(x, y, z).r(0.45), "the duct");
    }
    // On the grate Bond looks down at it before E: looking level the use point sits 1.8 above
    // the grate, past the refine's 1.0 [M Collide_Intersect 0x1ebaa8:186-189 →
    // Intersect_PointGeom 0x1e92d8; use_pick_census_live's #98 row]. He looks level again
    // once he has dropped through.
    for t in 0..400u32 {
        let (p, yaw, pitch) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.pos, sim.bond.yaw, sim.bond.pitch) };
        if p[1] < 48.6 && w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        let on_grate = p[0] > 9.2 && p[1] < 51.2;
        let want = if on_grate { -std::f32::consts::FRAC_PI_2 } else { std::f32::consts::FRAC_PI_2 };
        let err = walker::wrap_pi(want - yaw);
        let look_want = if on_grate { -0.7 } else { 0.0 };
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: (look_want - pitch).clamp(-0.05, 0.05), forward: if on_grate || err.abs() > 0.3 { 0.0 } else { 1.0 }, ..Default::default() };
        w.tick(input, on_grate && err.abs() < 0.3 && pitch < -0.6 && t % 6 == 0);
    }
    for _ in 0..30 {
        let pitch = w.scene.sim.as_ref().unwrap().bond.pitch;
        if pitch.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { look: (-pitch).clamp(-0.05, 0.05), ..Default::default() }, false);
    }
    eprintln!("[0b] the duct walked: bond {:?} ch76 {}", w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.channels.on(0x76));
    w.wait(10);
    }
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
        for _ in 0..200 { if w.scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; } w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    }
    // A keypad opens by its typed code or the decryptor; the laser arm is kind 1 only [M 0x1ae3d8:47, :162].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(li) = sim.drones.locks.iter().position(|l| l.out == 47 && !l.unlocked) {
            let code = sim.drones.key_codes.code(sim.drones.locks[li].code);
            sim.drones.keypad = Some((li, nf_game::lock::Keypad::arm(code, false)));
            for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
        }
    }
    // A keypad opens by its typed code or the decryptor; the laser arm is kind 1 only [M 0x1ae3d8:47, :162].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(li) = sim.drones.locks.iter().position(|l| l.out == 47 && !l.unlocked) {
            let code = sim.drones.key_codes.code(sim.drones.locks[li].code);
            // The stand-in arm writes the solved state straight in, with no page shown: the
            // page's pad hold (`keypad_page_drawn`) stays off for it, so the aim below runs
            // from its first tick as on the proven route.
            sim.drones.keypad_page_drawn = false;
            sim.drones.keypad = Some((li, nf_game::lock::Keypad::arm(code, false)));
            for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
        }
    }
    let mut burned = false;
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().drones.locks[lk].unlocked { burned = true; break; }
        let (head, target) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.bond.head(), lp) };
        let dir = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-4);
        // Aimed with the stick (turn + look), not a yaw/pitch write; the trigger held throughout.
        let (turn, look) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (walker::wrap_pi(dir[0].atan2(dir[2]) - sim.bond.yaw),
             (dir[1] / l).asin() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch)
        };
        let input = nf_game::player::Input { turn: turn.clamp(-0.08, 0.08), look: look.clamp(-0.08, 0.08), ..Default::default() };
        w.scene.sim.as_mut().unwrap().step_bond(input, true, false, false);
    }
    assert!(burned, "the typed code opens the keypad [M Lock_Update 0x1ae3d8 case 4]");
    w.scene.sim.as_mut().unwrap().drones.keypad_page_drawn = true;
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
    ] {
        w.walk_to(wp, label);
    }
    // THE ROCKET JOG (explode-hurt-radius; the level-0b-roof-sprint-4 office jog, difficulty-gated
    // so the d1/warm streams stay byte-identical): the exec office's west strip holds the disc's
    // PICKUP_RocketLauncher #349 (p0 = 0 weapon, p1 = 46 = 0x2E, p2 = 8 rounds [D
    // docs/levels/0700000b-audit.md:191]) at (-22.5, 47.96, 29.30). The straight drives all wedge
    // on the office furniture [L sprint-4 agent_v2/v3]; the level's own nav web routes the strip
    // (nodes 0.33/0.32/0.29 [L the sprint-4 probe_0b_execfloor_nav dump]) and the take fires on
    // the creep past the pickup. The jog returns through the doorway so the entry legs re-run
    // unchanged — sprint-4 proved the desk approach lands byte-identical [L agent_base2 vs
    // agent_v10].
    if w.scene.sim.as_ref().unwrap().drones.difficulty >= 2 {
        lane_sprint(w, [-20.58, 47.18, 25.46], 1.0, 300, "jog 0.33 (office centre-south)");
        lane_sprint(w, [-24.09, 47.18, 27.41], 0.9, 300, "jog 0.32 (the west strip south)");
        lane_sprint(w, [-22.5, 47.96, 29.30], 0.9, 240, "the rocket pickup #349 (the office jog)");
        let mut took = false;
        for _ in 0..60 {
            if w.scene.sim.as_ref().unwrap().drones.inventory.owned.get(0x2e as usize).copied().unwrap_or(false) { took = true; break; }
            w.tick(nf_game::player::Input { forward: 0.4, ..Default::default() }, false);
        }
        eprintln!("[0b] rocket #349 take: owned 0x2e = {}", took);
        lane_sprint(w, [-23.94, 47.18, 29.90], 0.9, 240, "jog 0.29 (back out)");
        lane_sprint(w, [-24.09, 47.18, 27.41], 0.9, 240, "jog 0.32 (back south)");
        lane_sprint(w, [-20.58, 47.18, 25.46], 0.9, 240, "jog 0.33 (back centre)");
        w.walk_to(Wp::new(-22.18, 47.18, 16.58).r(1.0), "jog back through the doorway (n25)");
    }
    for (label, wp) in [
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
    if rocket_pin {
        // THE BLAST-RADIUS PIN (explode-hurt-radius), on the real path: Bond walked here from the
        // spawn (no teleport after the load) and owns the rocket 0x2e (8 rounds [D #349 p2]). The
        // disc's row: radius 8.0, damage 100.0 [L ee_snapshot.bin @0x2c2178 = weapon_data base
        // 0x2bf150 [M action-symtab.tsv] + 0x2e*0x10c]. The game's hurt reads the sphere EDGE:
        // picked iff centre < radius+obj+0x8C [M 0x1eae80:221-228], amount =
        // damage*(1 - max(dist-0x8C,0)/radius), below 0.0002 nothing [M 0x17c5e8-0x17c620].
        // The walls are picked by the sim itself (world.segment from Bond's eye), so the pin
        // leans on no hand-copied geometry: the far wall >= 10.5 u out (edge >= 9.4 > 8.0 -> the
        // falloff is 0), the near wall <= 7.5 u (edge <= 6.45 < 8.0 -> hurt).
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.gun.weapon = 0x2e;
            sim.gun.switch_request = 0x2e;
        }
        w.wait(30);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(sim.drones.inventory.owned.get(0x2e as usize).copied().unwrap_or(false),
                "[0b-rocket] the office jog did not take pickup #349");
            assert_eq!(sim.gun.weapon, 0x2e, "[0b-rocket] the rocket is not the armed weapon");
        }
        // The walls are candidates the sim itself measures with the rocket's own flight filter;
        // the rocket's DETONATION POINT is then read back from the live blast
        // (`drones.explosions` — (pos, radius, damage, attacker, weapon)) and every assert keys
        // on the true detonation distance, so a script-entity object that eats the round
        // mid-flight (the office furniture did exactly that on the first attempt — the sprint-4
        // v10 story) just disqualifies that ray and the next candidate takes over.
        let mut far_cands: Vec<[f32; 3]> = Vec::new();
        let mut near_cands: Vec<[f32; 3]> = Vec::new();
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let cell = sim.world.find_cel(eye);
            for k in 0..24 {
                let yaw = k as f32 * std::f32::consts::TAU / 24.0;
                let dir = [yaw.sin(), 0.0, yaw.cos()];
                let end = [eye[0] + dir[0] * 40.0, eye[1], eye[2] + dir[2] * 40.0];
                let Some(t) = sim.world.segment_flight_scoped(cell, eye, end, false) else { continue };
                let d = t * 40.0;
                if d < 1.5 { continue; } // muzzle furniture: not a wall to fire at
                let p = [eye[0] + dir[0] * d, eye[1], eye[2] + dir[2] * d];
                if d >= 10.5 { far_cands.push(p); }
                if d <= 7.5 { near_cands.push(p); }
            }
        }
        assert!(!far_cands.is_empty(), "[0b-rocket] no wall >= 10.5 u from the desk for the far leg");
        assert!(!near_cands.is_empty(), "[0b-rocket] no wall <= 7.5 u from the desk for the near leg");
        // The far candidates try LONGEST first: the longest measured clear ray is the one most
        // likely to keep its detonation past the radius (a mid-flight object that eats a shorter
        // ray detonates inside the radius and the game's own falloff then hurts — legal, but it
        // spends health this walk cannot spare for the near leg).
        let bp0 = w.scene.sim.as_ref().unwrap().bond.pos;
        far_cands.sort_by(|a, b| {
            let da = (a[0] - bp0[0]).powi(2) + (a[2] - bp0[2]).powi(2);
            let db = (b[0] - bp0[0]).powi(2) + (b[2] - bp0[2]).powi(2);
            db.partial_cmp(&da).unwrap()
        });
        eprintln!("[0b-rocket] far candidates {}, near candidates {}", far_cands.len(), near_cands.len());
        let sph = w.scene.sim.as_ref().unwrap().drones.player_sph;
        // One fire + wait cycle; returns (blast pos, world-owned hurt this window).
        let fire_and_wait = |w: &mut Walker, target: [f32; 3]| -> (Option<[f32; 3]>, Vec<f32>) {
            let ph = w.scene.sim.as_ref().unwrap().drones.player_hits.len();
            let clip0 = w.scene.sim.as_ref().unwrap().gun.clip;
            let fired = shoot_at(w, target, 60.0, 600, "the radius pin rocket", &|sim| sim.gun.clip < clip0);
            assert!(fired, "[0b-rocket] the rocket never left (clip {clip0})");
            let mut blast = None;
            for _ in 0..150 {
                w.tick(nf_game::player::Input::default(), false);
                let sim = w.scene.sim.as_ref().unwrap();
                if let Some(&(p, _, _, _, wt)) = sim.drones.explosions.iter().find(|(_, _, _, _, wt)| *wt == 0x2e) {
                    blast = Some(p);
                }
                if blast.is_some() { break; }
            }
            let hits: Vec<f32> = w.scene.sim.as_ref().unwrap().drones.player_hits[ph..].to_vec()
                .into_iter().filter(|(_, o, _)| *o == usize::MAX).map(|(_, _, d)| d).collect();
            (blast, hits)
        };
        // The near leg runs FIRST (Bond at full walked health — a near blast costs at most the
        // full 80 the Agent pain factor gives, so he survives it); the far leg's law then holds
        // for EVERY candidate, and die_panics is off so a lawful mid-flight detonation (the
        // office furniture eats rockets on the shorter rays) cannot panic the walker before the
        // asserts read their ledger. No assert reads Bond's health after this point.
        w.dmg.die_panics = false;
        // THE NEAR LEG: a wall inside the radius: Bond MUST take the blast, the amount the
        // straight-line falloff gives (x the level's pain factor: Agent 0.8 [M Player_HandlePain
        // 0x1902c8; tuning.rs 09-0b group]).
        let target = near_cands[0];
        let (blast, hits) = fire_and_wait(&mut w, target);
        let bp = blast.expect("[0b-rocket] the near rocket never detonated");
        let d3 = ((sph[0] - bp[0]).powi(2) + (sph[1] - bp[1]).powi(2) + (sph[2] - bp[2]).powi(2)).sqrt();
        assert!(!hits.is_empty(),
            "[0b-rocket] the near blast (d3 {d3:.2}) did not reach Bond — the port's reach falls short of the disc radius");
        {
            let total: f32 = hits.iter().sum();
            let want = 100.0 * (1.0 - (d3 - 1.0576).max(0.0) / 8.0) * 0.8;
            assert!((total - want).abs() < 3.0,
                "[0b-rocket] the near blast took {total}, the straight-line law wants {want} (sphere {sph:?}, blast {bp:?}, d3 {d3})");
            eprintln!("[0b-rocket] near leg green: took {total} (law {want}) at d3 {d3:.2}");
        }
        // THE FAR LEG (the sprint-4 v10 shape: fire with the target over 10 u out): for EVERY
        // candidate the law must hold — a detonation past the sphere-edge 8.0 (the disc's
        // radius) hurts NOBODY; one inside hurts exactly the falloff. The level-0b-roof-sprint-4
        // report's ~48 "at 13.7 u" is the refuted reading: that fire's rocket detonated on the
        // office furniture ~4.7 u out (the falloff there IS ~44), not at the d-13.7 aim point.
        let mut far_ledger = String::new();
        let mut far_ok = false;
        for (i, &target) in far_cands.iter().enumerate().take(4) {
            let (blast, hits) = fire_and_wait(&mut w, target);
            let Some(bp) = blast else { far_ledger += " no-detonation"; continue; };
            let d3 = ((sph[0] - bp[0]).powi(2) + (sph[1] - bp[1]).powi(2) + (sph[2] - bp[2]).powi(2)).sqrt();
            let edge = d3 - 1.0576;
            let total: f32 = hits.iter().sum();
            far_ledger += &format!(" cand{i} d3 {d3:.2} edge {edge:.2} hurt {total:.2}");
            if edge > 8.05 {
                assert!(hits.is_empty(),
                    "[0b-rocket] the far blast hurt Bond from edge {edge:.2} u past the 8.0 radius ({hits:?}) — the port's reach passes the disc radius");
                far_ok = true;
                eprintln!("[0b-rocket] far leg green: detonation edge {edge:.2} u > 8.0, no hurt");
                break;
            }
            // Detonated inside the radius (a mid-flight object ate the round): the game's own
            // falloff may hurt — the law assert above already covers the amount window through
            // the near leg; this ray just cannot serve as the past-the-radius proof.
        }
        assert!(far_ok, "[0b-rocket] no far ray detonated past the radius:{far_ledger}");
        return;
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
    // The aim sits on the PC's own mesh (y ±0.34 about the placement [D the glist's collision
    // bounds]): the Q-Worm fires only with the PC first under the crosshair [M 0x1a5460:531-545].
    let installed = shoot_at(&mut w, [pc[0], pc[1] + 0.2, pc[2]], 2.6, 1800, "the Q-Worm C shot", &|sim| sim.drones.channels.on(0x4c));
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
            // REAL HEALTH: the fence has fired ch 0x0D by now (the walk crossed it en route) —
            // the terminal-room GT pair #174/#175 is ARMED (OR #290 → ch 0x0E = p4 14 [D]), so
            // the dome-floor transit south runs the lane sprint (see lane_sprint); the warm
            // walks keep the walked approach.
            let room = w.try_walk_to(Wp::new(-1.0, 47.5, -5.5).r(1.2), "files the terminal room (from n14)");
            // The turret pair #174/#175 sits at (2.26/-4.60, 49.10, −21.60) [D] — flanking the
            // plate lane 7.7 u south. The straight north-south lane hands them a head-on,
            // barrel-eased target for the last 8 u (6 × 3.2 = 19.2 [L the real_damage_agent
            // ledger, frames 12761-12804]). The pair eases its barrel along the 0.1 s cadence
            // [M 0x180498 case 4], so a fast ~90° crossing of the lane makes the ease chase:
            // run WEST to x −7.0 at the plate's z, then dash EAST into the plate. [L] the
            // same-cadence pair lands 6/6 on the head-on lane — the crossing is the reroute.
            let walked = if real {
                // [L the reroute measurements] head-on lane: 6 × 3.2 on the approach, re-place
                // 80.8. West crossing (x −7.0): approach 6.4 — but the return then runs under
                // barrels pre-trained east: re-place 49.2 (head-on return) / 36.4 (west return).
                // The head-on pair is the best measured lane; the deck two-phase is the lever.
                lane_sprint(w, [c[0], lo[1] - 0.4, c[2]], 1.4, 900, "the spawned files use-plate")
            } else {
                w.try_walk_to(Wp::new(c[0], lo[1] - 0.4, c[2]).r(1.4), "the spawned files use-plate")
            };
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
                // The use pick is the head + view-dir sphere [M 0x1a5170:100-108] and the E fires
                // on the PRESS EDGE only [M 0x1a5460:396-403] — face the plate from each stance
                // and give every stance its own fresh press (the old loop held use across all
                // six: one edge, stance 1 only, and the east-dash arrival yaw missed the pick).
                for _ in 0..30 {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let p = sim.bond.pos;
                    let err = walker::wrap_pi((c[0] - p[0]).atan2(c[2] - p[2]) - sim.bond.yaw);
                    if err.abs() < 0.05 { break; }
                    w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
                }
                for k in 0..40 {
                    w.tick(Default::default(), k == 0);
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
        // REAL HEALTH: the pair is armed (the fence + the take both feed OR #290), so the
        // whole return lane runs the sprint — the plate → the room → n14 → n15 → n16.
        if real {
            // [L the reroute measurements] the straight north return is the clean one once the
            // barrels end north-trained (the head-on approach); the west detours trained them
            // onto Bond and the return bled 44.4 / 56.4. Head-on both ways: re-place 80.8.
            lane_sprint(w, [-1.0, 47.5, -5.5], 1.2, 900, "return the terminal room (from the dome)");
            lane_sprint(w, [-1.45, 47.18, 3.74], 1.0, 600, "return n14");
            lane_sprint(w, [-1.33, 47.18, 6.82], 1.0, 600, "return n15");
            lane_sprint(w, [-13.67, 47.18, 6.40], 1.0, 900, "return n16");
            // (level-0b-roof-sprint-4: the rocket is taken in the exec office instead — the
            // first cut of this detour (n16 → the pickup south-west) crossed the keypad-corner
            // guards #2/#3 at d 8-12 and bled 23 hits for 100 in the z 15-16 corridor [L the
            // agent_v1 run]; the baseline return lane itself is clean [L agent_base2 — zero
            // corridor hits]. The office jog after the desk approach carries the rocket over
            // the whole roof gate with no new exposure.)
        } else {
            let _ = w.try_walk_to(Wp::new(-1.0, 47.5, -5.5).r(1.2), "return the terminal room (from the dome)");
            for (label, wp) in [
                ("return n14", Wp::new(-1.45, 47.18, 3.74).r(1.0)),
                ("return n15", Wp::new(-1.33, 47.18, 6.82).r(1.0)),
                ("return n16", Wp::new(-13.67, 47.18, 6.40).r(1.0)),
            ] {
                w.walk_to(wp, label);
            }
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
    // [D streams 6/13/14]. Everything after the gate is the real disc script. The wait runs
    // until the GAME re-enables Bond (the cut-scene's PlayerDisable tail; the arrival
    // cut-scene's wait_enabled pattern) — the fixed 2900 stood Bond 450 extra ticks at the deck
    // key under the roof guards' fire with no control ([L the death run: 11 hits, 23.8 → 0.0,
    // ticks 16473-16652, at the re-place point, no fight ever started]).
    w.wait_enabled(3400, "the roof cut-scene tail (control returns)");
    // [0b-roof-sprint DIAG] the deck facts at control return: the world-hurt ledger so far, the
    // fuseboxes' wire state, the hurt volumes engaged, and the roof squad's positions/states.
    if real {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[0b-diag] frame {} world hits so far:", sim.drones.frame);
            for (f, o, a) in &sim.drones.player_hits {
                if *o == usize::MAX { eprintln!("[0b-diag]   frame {f}: -{a:.2}"); }
            }
            for (fi, fb) in sim.drones.fuseboxes.iter().enumerate() {
                eprintln!("[0b-diag] fusebox{fi}: placement {} script {:#x} ch {:?} g0 alive {} g1 alive {}",
                    fb.placement, fb.script, fb.ch,
                    fb.groups[0].slots.iter().filter(|s| s.is_some()).count(),
                    fb.groups[1].slots.iter().filter(|s| s.is_some()).count());
            }
            for h in sim.hurts.iter() {
                if h.hits > 0 || (h.lo[1] > 44.0 && h.lo[1] < 60.0) {
                    eprintln!("[0b-diag] hurt placement {} kind {} dmg {} gate {} who {:#x} one_shot {} hits {} box ({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})",
                        h.placement, h.kind, h.damage, h.gate, h.who, h.one_shot, h.hits,
                        h.lo[0], h.lo[1], h.lo[2], h.hi[0], h.hi[1], h.hi[2]);
                }
            }
            let p = sim.bond.pos;
            eprintln!("[0b-diag] bond at ({:.2},{:.2},{:.2}) hp {:.1} armour {:.1}", p[0], p[1], p[2], sim.drones.health.health, sim.drones.health.armour);
            for (ti, t) in sim.drones.gt_turrets.iter().enumerate() {
                eprintln!("[0b-diag] turret{ti}: placement {} pos ({:.2},{:.2},{:.2}) dead {}",
                    t.placement, t.pos[0], t.pos[1], t.pos[2], t.hp);
            }
            for (pi, pk) in sim.drones.pickups.iter().enumerate() {
                eprintln!("[0b-diag] pickup{pi}: kind {} weapon {:#x} ch {:#x} at ({:.2},{:.2},{:.2}) gone {}",
                    pk.kind, pk.weapon, pk.switch_channel, pk.m[3][0], pk.m[3][1], pk.m[3][2], pk.gone);
            }
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.present && d.team == 1 && !d.corpse && d.health > 0.0 {
                    eprintln!("[0b-diag] guard #{i} ({:.2},{:.2},{:.2}) hp {:.1} state {:#x} weapon {:#x} dist {:.1} los {} seen {}",
                        d.pos[0], d.pos[1], d.pos[2], d.health, d.state, d.weapon, d.dist, d.los, d.seen);
                }
            }
        }
    }
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

    // The parachute + the jump are the REAL object paths. The deck crossing to the pickup is
    // the walked leg (the warm walks) or the real-health sprint (see the deck leg below); the
    // take is the walked slot below.
    let parachute = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.switch_channel == 0x3f).expect("OBJECT_Parachute (0xF0, ch 0x3F)").clone();
    let pp = [parachute.m[3][0], parachute.m[3][1], parachute.m[3][2]];
    if real {
        // THE DECK SPRINT-THROUGH — the player's crossing on real health. The deck key stands
        // 30-42 u from four roof guards [D placements; L the death runs' damage logs]; the K5's
        // dart cannot touch a guard in full combat [M 0x164210 — Attack carries no case 0x19] and
        // the sleep it does land buys `RecoverTime(0x43) = 36·(200−+0xB9)` ≈ 144 ticks on these
        // guards before a team-1 wake [M 0x147188 k=60; L the ring sleeps #1/#3 — both back
        // shooting inside ~150 ticks] — standing to dart them is the stall that bled the first
        // un-ignoring runs. A moving target cuts the guards' roll instead [M 0x179740 —
        // `DroneFiring_TargetMoving_Accuracy`], so Bond RUNS the diagonal (the same real-input
        // pattern the jump leg drives; no fight stops) straight through the reduced fire.
        // THE SCRAPE-SHIFT ANSWER (level-0b-walk-after-scrape-draw): s/fx-scrape-and-impacts'
        // own draws shifted the shared stream, and the sprint that used to end at the deck slot
        // (pp−5.0) then cost its stop there: the walked take-slot leg's return-fire gate picked a
        // dartable deck guard whose aim ray the housing blocks, and `stand_and_fire` held the
        // blocked stand (no dart flies, the 300-tick no-round window never closes) while drone #4
        // (skin 0x5000006, weapon 0x18) stood 4.5 u away chewing 58.8 health into the run's last
        // 80 frames [L the real_damage_agent death log, tick 16047 at (2.22,58.08,36.54)]. A
        // player under that fire does not stand and trade — the objective is 9 u further and the
        // take fires from the RUN (the touch law is a per-tick distance+LOS check [M
        // 0x18e148:202-220], it needs no stand) — so the sprint now runs THROUGH the take slot
        // [pp−0.3, −1.0], the take raising ch 0x3F on the pass, and the jump leg's east drive
        // (raw ticks, no return-fire gate) follows with no stop between. The tranq fights stay
        // on the legs where their target is reachable; the deck's end is pure movement.
        // THE DECK LEG (level-0b-roof-sprint-3, the chained lane): the deck's NW quadrant is
        // walled by a 45° diagonal face (normal (−0.71,−0.70), top 58.2 — above the standing
        // capsule) running (−8.2, 23) → (−14, 28.8), plus a flat south wall at z 22.28 spanning
        // x −7..−3 with a second piece at x 2 [L the ray probe on this tree]; the only north
        // opening is the GAP x −2..1 (north rays clear, 57.0 floor at z 22-26). Sprint-2's west
        // hug held 87.2 hp through the #6/#7/#8 band (they park at x ≥ 8.5 [D the diag]) but its
        // P3 pressed the diagonal and slid Bond NW into the corner (hp 64.8 → 8.8 [L the
        // 0b-stall trace]); the straight WIP line threaded the gap but crossed the whole open
        // middle under the east band and #4 on-line (the on-line #4 pass alone 36.4 [L]). The
        // chain keeps the hug's west-band margin, runs EAST south of the flat wall, goes north
        // THROUGH the gap, then passes #4 (2.6, 58.0, 32.7) wide on the final dash — the wide
        // pass sprint-2 measures at ~16.8. Phases advance only on arrival (dxz < 1.6) and never
        // re-aim backward (the sprint-2 re-steer pathology: a backward re-aim walked Bond back
        // into #4's cone, hp 34.0 → 14.4 [L]).
        let target = [pp[0] - 0.3, pp[1] - 1.4, pp[2] - 1.0];
        // ARM FIRST (level-0b-roof-sprint-3): Bond reaches the deck holding the DRY 0x58
        // (clip 0 spare 0 [L the stall trace]) while the wheel's 0x43 sits loaded (clip 5
        // [L the inventory trace]) — every fire-on-the-move shot was a blank; 0x4a (clip 100) spawned no rounds — a non-bullet gadget [L the clip stayed 100 through the drop stand]. A player switches
        // to the loaded weapon: the real switch path (Player_WeaponChange) until it lands.
        // SPRINT-4 (the sisters only): that stand cost ~20 hp (87.2 → 67.6 [L sprint-3]) —
        // Player_WeaponChange needs no stand, so the Agent/00Agent runs issue the switch INTO
        // the P1 run below (arm-on-the-run, target the ROCKET 0x2e for the gap-mouth drop)
        // and skip this block; the d1 walk keeps the stand-first arm byte-identical.
        let agent_run = w.scene.sim.as_ref().unwrap().drones.difficulty >= 2;
        if !agent_run {
            for _ in 0..120 {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.gun.weapon == 0x43 && sim.gun.clip > 0 { break; }
                let sw = sim.gun.switch_request;
                w.tick(nf_game::player::Input {
                    switch_to: if sw == 0x43 { None } else { Some(0x43) },
                    ..Default::default()
                }, false);
            }
        }
        // SPRINT-5 (the sisters): P4 runs THROUGH the take zone to the deck's east edge
        // (9.8, 41.3) and P5 is dead for the d1 walk — the wake volley math: the dart lands
        // at d ~5.5 (t16305 [L agent00_run5]), the guard shoots again ~180 ticks later (144
        // RecoverTime [M 0x147188 k=60] + the get-up), and the slot→edge drive from a take-
        // slot stop is 60-80 exposed ticks — the 00 Agent run died IN that drive (4 × 7.0,
        // tick 16536 [L agent00_run1]). The dash covers slot + edge in the same sleep window
        // (13 u + 7.6 u at the measured 0.14 u/tick), the take firing ON THE PASS (the touch
        // law is a per-tick distance+LOS check [M 0x18e148:202-220]), and the jump leg only
        // has the last 0.2-3 u before the fall. The P4 aim point (1.9, 40.9) sits west of
        // the housing's south-west column — the straight SW approach wedges on it at
        // (2.02, 40.59) with the touch sphere 0.07 out of reach and the take never fires
        // [L agent00_run5 take-dist 1.861] — while (1.9, 40.9) is d 1.62 of the pickup,
        // inside the 1.79 sphere with the take-ray threading the open south face. The d1
        // walk keeps the slot lane (its 5th entry is unreachable — the take-break fires
        // first) byte-identical.
        let lane: [[f32; 2]; 5] = if agent_run {
            [
                [-11.8f32, 18.0f32], // P1 the west band, south of the diagonal (the measured heights)
                [0.0, 21.0],         // P2 the east run aims through #5's gap-mouth drift
                [-1.5, 30.5],        // P3 north through the x −2..1 gap (clear at capsule height)
                [1.9, 40.9],         // P4 the take-zone approach, west of the housing column
                [11.3, 41.3],         // P5 the deck's east edge — the wake volley lands past it
            ]
        } else {
            [
                [-11.8f32, 18.0f32], // P1 the west band, south of the diagonal (the measured heights)
                [-1.5, 20.3],        // P2 the east run south of the z 22.28 wall, to the gap mouth
                [-1.5, 30.5],        // P3 north through the x −2..1 gap (clear at capsule height)
                [target[0], target[2]], // P4 the final dash past #4 to the take slot
                [target[0], target[2]], // (dead for d1 — the take-break fires at the slot)
            ]
        };
        let mut phase = 0usize;
        let start = w.total_ticks;
        let mut stall_diag = 0u32;
        let mut drop_start = 0u32;
        let mut close_start = 0u32;
        // SPRINT-4 (the sisters): the gap-mouth rocket attempt's bookkeeping — retired there,
        // REVIVED by sprint-6 as the ROCKET WINDOW's state (below): rocket_done closes the
        // window, rocket_start bounds it, rocket_cool stamps the last shot, rockets_fired
        // counts the shots (0 = the band shot pending, 1 = the #4 shot pending, 2 = spent).
        let mut rocket_done = false;
        let mut rocket_start = 0u32;
        let mut rocket_cool = 0u32;
        let mut rockets_fired = 0u32;
        while w.total_ticks - start < 2400 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            // The take fires ON THE PASS — for the sisters the pass continues to the deck's
            // east edge (the P5 lane) before the loop hands over to the jump drive; the d1
            // walk breaks at the take exactly as before.
            if sim.drones.channels.on(0x3f) && (!agent_run || p[0] > 9.0) { break; }
            // [0b-stall] every 30 ticks: where the body sits against the face, its speed, and the
            // take-sphere distance (the NF_PICKUPLOG take-test only logs when the gate passes).
            if w.total_ticks - stall_diag >= 30 {
                stall_diag = w.total_ticks;
                let dx = pp[0] - p[0]; let dy = pp[1] - p[1]; let dz = pp[2] - p[2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                // SPRINT-6: `mv` is the accuracy read itself — the move quad the guards latch
                // [M 0x179a48:34-41 → 0x179740:71-73, Player_Move 0x191dd0:166-170 stores]. The
                // sprint-5 ledger's "vel (0,0,0) while running" read BL+0x50 (the collision vel,
                // zeroed while grounded), NOT this quad — the print now shows both.
                eprintln!("[0b-stall] t{} bond ({:.2},{:.2},{:.2}) vel ({:.2},{:.2},{:.2}) mv ({:.2},{:.2}) yaw {:.2} hp {:.1} take-dist {:.3} phase {} gun w{:#x} clip {} spare {} d4hp {:.1}",
                    w.total_ticks, p[0], p[1], p[2],
                    sim.bond.vel[0], sim.bond.vel[1], sim.bond.vel[2],
                    sim.bond.move_local[0], sim.bond.move_local[1], sim.bond.yaw,
                    sim.drones.health.health, dist, phase,
                    sim.gun.weapon, sim.gun.clip, sim.gun.spare,
                    sim.drones.list[4].health);
                // [0b-stall] the roof squad's live law (sprint-5): which of #4/#5/#7/#8 are alive,
                // where they stand relative to Bond, and their STATE (the dart law is
                // state-gated [M 0x1469b8:315-361] — a dart lands a sleep only on the states
                // that carry case 0x19; d4state tells whether the close can take #4 at all).
                // SPRINT-6: #6/#9 ride along (the 00 Agent ledger's long-range tollers) and
                // each guard prints its OWN moving latch (Drone+0x1DC [M 0x179a48:34-41]) —
                // the run-proof that a running Bond reads MOVING to every firing guard.
                for &gi in &[4usize, 5, 6, 7, 8, 9] {
                    let g = &sim.drones.list[gi];
                    if g.present && !g.corpse {
                        eprintln!("[0b-stall]   g{gi} ({:.2},{:.2},{:.2}) hp {:.1} state {:#x} w {:#x} d {:.1} dartable {} moving {}",
                            g.pos[0], g.pos[1], g.pos[2], g.health, g.state, g.weapon,
                            walker::dxz(g.pos, p),
                            nf_game::drone::Drone::dart_impact_state_ok(g.state),
                            g.target_motion.moving);
                    }
                }
                let inv = &sim.drones.inventory;
                let owned: Vec<String> = inv.owned.iter().enumerate()
                    .filter(|(_, &o)| o)
                    .map(|(w, _)| format!("{w:#x}:clip {}", inv.clip.get(w).copied().unwrap_or(0)))
                    .collect();
                eprintln!("[0b-stall]   carried {:?} owned [{}]", inv.carried, owned.join(" "));
            }
            // SPRINT-6: THE ROCKET WINDOW — measured, NOT shipped. The P1/P2 long-range
            // toll's intended answer was one carried 0x2e round into the east band's
            // anchor #6 (the disc placements hold #6/#7/#8/#9 within 6.1 u of him [D
            // drone-props; L a00_run1 stalls]; the disc falloff 100·(1−gap/8) pays 31-100
            // at those gaps vs hp 10-20 [L ee_snapshot weapon_data+0x2e·0x10c+0x08/0x0C;
            // M Explode_Propagate 0x17c518]) — three real-input attempts, three misses:
            // (1) a flat shot from control return struck a SW deck structure at
            // (−6.5, 58.6, 7.9), 5.2 u from Bond (−47.37) [L a00_run3]; (2) the flat shot
            // from the P1 point hit the lower floor band at d ~14 — the round ARCS, 1.33 u
            // below the straight line at 14 u [L a00_run4: impact y 57.6 at (5.0, 22.3)];
            // (3) drop-compensated shots (quadratic then linear fits) flew past the
            // cluster — the linear fit over-aimed at d 35 [L a00_run5: only #8's −4.2;
            // a00_run6: no hit at all]. The round's own flight law (speed + gravity) is
            // not pinned: the next window probes Bullet_init 0x12c5a0's speed/fuse fields
            // for 0x2e [M the record walk] before re-arming the window. Also measured:
            // the port's blast obeys the disc law exactly — detonation d 5.2, radius 8.0,
            // damage 100.0 → 47.37 = 100·(1−4.21/8) [L a00_run3] — the sprint-4
            // "hurt at 13.7" was a mis-attributed detonation distance, NOT a radius
            // divergence (the explode-hurt-radius task should re-measure the blast POS).
            // The a00_run5 post-shot switch back to 0x43 took ~100 ticks (w0x2e t16157 →
            // t16277) and left the sprint-5 close cold — Bond died at d 1.0 of #4 under
            // ×2 TooClose rounds [L a00_run5: 45.5 in 5 at d 1.0; M 0x139fac-0x139fc8];
            // any re-armed window must fire at control return AND ride the switch-back on
            // the fire tick [L a00_run6: the early shot + the fire-tick request got the
            // gun home, the arc still missed].
            // SPRINT-5 (the sisters): CLOSE ON #4 BEFORE THE P4 TURN — the sprint-4 report's
            // second option. #4 comes down the gap column to meet the lane (the agent_run2
            // trace: d 14.8 → 7.8 → 5.0 as Bond ran P3) and is dartable the whole way [L
            // agent_run2 state 0x68/0x73], so the north leg steers AT him once he is inside
            // the fire-on-move's proven 9 u (the trigger rides the K5 on the run — the
            // sprint-3 funnel STAND is what killed the sisters: it held Bond at d 2.6 for
            // ~150 ticks while its err hovered 0.05-0.14 and #4/#5/#7/#8 chewed 39.6 → 0.0
            // [L agent_run1]). The phase holds until he is asleep (the dart family — the
            // dart's own win state [M 0x163430]) or the 3.4 floor (under 3.0 u every drone
            // round hits [M 0x179740:113-115]) hands the lane to the P4 dash.
            let mut closing = false;
            let mut close_ticks = 0u32;
            if agent_run && phase == 2 {
                let g4 = &sim.drones.list[4];
                if g4.present && !g4.corpse && g4.health > 0.0
                    && !nf_game::drone::Drone::dart_family_state(g4.state) {
                    let d4 = walker::dxz(g4.pos, p);
                    if d4 < 9.0 {
                        if d4 > 3.4 {
                            closing = true;
                            if close_start == 0 { close_start = w.total_ticks; }
                            close_ticks = w.total_ticks - close_start;
                            // Bounded: a close that cannot converge (the err never opens the
                            // 0.05 gate against a strafing guard) must not hold the lane
                            // forever — the dash goes and eats the old wide-pass cost.
                            if close_ticks > 180 { closing = false; phase = 3; }
                        } else {
                            phase = 3;
                        }
                    }
                } else {
                    // asleep or gone — the dash goes while the sleep holds (the agent_run2
                    // trace: the roof guard's 0x50 held 260+ ticks, well past the take).
                    phase = 3;
                }
            }
            while phase + 1 < lane.len()
                && !closing
                // SPRINT-5 (the sisters): the P4→P5 advance waits for the TAKE — the 1.6
                // arrival fires ~1.6 u short of the slot and the veer to the east edge then
                // grazes the touch sphere at 1.98 (the take never fires, the loop never
                // breaks, the wake volley kills the stall [L agent_run4/agent00_run3 — both
                // died oscillating at x 9.7-10.1 with the parachute untaken]). Steering AT
                // the slot walks Bond into the sphere (d 1.11 [L the slot geometry]) and the
                // take fires on the arrival tick; the d1 walk is untouched (its take-break
                // fires before this advance can matter).
                && (phase != 3 || !agent_run || sim.drones.channels.on(0x3f))
                && ((p[0] - lane[phase][0]).powi(2) + (p[2] - lane[phase][1]).powi(2)).sqrt() < 1.6 {
                phase += 1;
            }
            if phase == 3 && drop_start == 0 { drop_start = w.total_ticks; }
            let aim = if closing {
                let g4 = &sim.drones.list[4];
                [g4.pos[0], g4.pos[2]]
            } else {
                lane[phase]
            };
            let desired = (aim[0] - p[0]).atan2(aim[1] - p[2]);
            let mut aim_err = walker::wrap_pi(desired - sim.bond.yaw);
            // (level-0b-roof-sprint-4: the gap-mouth drop attempts are RETIRED — the rocket's
            // detonation hurt Bond ~48 at d 13.7 against the disc's own 8.0 blast radius [L
            // ee_snapshot weapon_data+0x2e·0x10c+0x08], and the shotgun shells landed ZERO on
            // #4 at d 13.7 while the stand bled 25 hp in 90 ticks [L agent_v11] — both levers
            // recorded in the report; the sprint-3 funnel drop below stays the sisters'
            // fallback once 0x43 is up.)
            // THE STAND-AND-DROP (level-0b-roof-sprint-3, "dropping #4 before the final dash"):
            // #4 parks at (2.6, 58.0, 32.7) 4-5 u east of the gap column — no lane heading ever
            // brings him near the crosshair (err 0.7-1.1 [L the runs]), and his 0x18 rounds
            // (5.6/hit) chew the P4 dash AND the jump leg behind it (the 15890-15902 backshots
            // killed the previous run at 14.4 hp [L]). Past the gap (z > 29) a player stops in
            // the funnel — the diagonal west, the flat wall south — turns and drops him, then
            // dashes. Position-gated (the phase bookkeeping missed it when the run drifted);
            // bounded: if he will not die in 240 ticks, the dash goes anyway.
            // SPRINT-5 (the sisters): the stand is the d1 walk's lever only — on the sisters
            // it is the measured DEATH (it held Bond at d 2.6 from #4 for ~150 ticks — inside
            // the 3.0 all-hit band — while its own err hovered 0.05-0.14 and the squad chewed
            // 39.6 → 0.0 [L agent_run1]; the dart it did land left #4 asleep at state 0x50
            // while the stand kept holding, because a sleeping guard's hp never moves). The
            // sisters' close above lands the dart ON THE RUN instead, and the P4 dash follows
            // the sleep.
            if p[2] > 29.0 && !agent_run {
                let sim = w.scene.sim.as_ref().unwrap();
                let g = &sim.drones.list[4];
                let dxg = g.pos[0] - sim.bond.pos[0];
                let dzg = g.pos[2] - sim.bond.pos[2];
                let dg = (dxg * dxg + dzg * dzg).sqrt();
                if g.present && !g.corpse && g.health > 0.0 && dg < 8.0 {
                    if drop_start == 0 { drop_start = w.total_ticks; }
                    if w.total_ticks - drop_start < 240 {
                        let desired_g = dxg.atan2(dzg);
                        let err_g = walker::wrap_pi(desired_g - sim.bond.yaw);
                        let at = [g.pos[0], g.pos[1] + 0.3, g.pos[2]];
                        let want_pitch = ((at[1] - (sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop))
                            / dg.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                        let aim = nf_game::player::Input {
                            turn: err_g.clamp(-0.08, 0.08),
                            look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08),
                            ..Default::default()
                        };
                        if err_g.abs() < 0.05 { w.tick_fire(aim); } else { w.tick(aim, false); }
                        continue;
                    }
                }
            }
            // FIRE ON THE MOVE (level-0b-roof-sprint-3): opportunistic ONLY — the trigger rides
            // the real fire path when a live target is already near the crosshair (|err| < 0.18;
            // #7 (10.0, 21.1) is nearly on-heading through the whole east run) but the TURN
            // NEVER leaves the lane: tracking a target while running yanked Bond off the lane
            // twice (the sprint-2 re-steer pathology — he drifted point-blank into #4 and died
            // [L the runs]). The run continues regardless.
            let mut trigger = false;
            let mut look = 0.0f32;
            // SPRINT-4: the opportunistic fire rides the SHOTGUN only — the sisters hold the
            // rocket 0x2e through P1/P2 and its rounds are reserved for the gap-mouth drop
            // (a wasted rocket or a < 9 u blast is the risk the gate removes). On the d1 path
            // the gun is always 0x43 (the stand-first arm above), so the gate changes nothing
            // there — the stream stays byte-identical.
            let gun_shotgun = sim.gun.weapon == 0x43;
            for &gi in &[5usize, 4, 7] {
                let g = &sim.drones.list[gi];
                if !gun_shotgun { continue; }
                if !g.present || g.corpse || g.health <= 0.0 { continue; }
                // SPRINT-5 (the sisters): a guard already in the dart family ignores further
                // darts (the stun families carry no case 0x19 [M 0x1469b8:315-361]) — do not
                // spend the clip on a sleeping body (d1 keeps its stream byte-identical).
                if agent_run && nf_game::drone::Drone::dart_family_state(g.state) { continue; }
                let dxg = g.pos[0] - p[0];
                let dzg = g.pos[2] - p[2];
                let dg = (dxg * dxg + dzg * dzg).sqrt();
                if dg > 9.0 { continue; }
                // SPRINT-5 (the sisters): #4's dart is held until d 5.0 — the sleep buys 180
                // ticks (144 RecoverTime [M 0x147188 k=60] + the get-up) and the latch chain
                // (the dash + the edge hop + the fall to ch 0x40) runs ~190 ticks from the
                // dart: released at d 7.2 the wake volley started 60 ticks BEFORE the band
                // and killed the 00 Agent run in the drive (28.0, tick 16536 [L
                // agent00_run3/4]); released at d ~5 the wake lands ~35 ticks after ch 0x40
                // has already latched the mission. The close's steering has the crosshair
                // converged long before 5.0 (the err rides 0.02-0.04 from d ~6 [L
                // agent_run3]), so the gate opens on the first window tick.
                if agent_run && gi == 4 && dg > 5.5 { continue; }
                let err_g = walker::wrap_pi(dxg.atan2(dzg) - sim.bond.yaw);
                // SPRINT-5 (the sisters): the 0.18 gate FIRES while the aim is still swinging
                // — a dart released at err 0.18 lands 0.6-1.6 u off the body and every one of
                // those shots missed [L agent_run1/2: the clip ran 5→3 across P2/P3 and no
                // guard slept until the funnel stand's 0.05-converged dart]. The sisters fire
                // only on the converged crosshair (0.05 ≈ 0.25-0.45 u at dart range); the
                // close above brings the aim to the guard so the gate actually opens.
                let err_gate = if agent_run { 0.05 } else { 0.18 };
                if err_g.abs() < err_gate {
                    trigger = true;
                    let at = [g.pos[0], g.pos[1] + 0.3, g.pos[2]];
                    let want_pitch = ((at[1] - (p[1] + sim.bond.eye - sim.bond.crouch_drop))
                        / dg.max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                    look = (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08);
                    break;
                }
            }
            let mut input = nf_game::player::Input {
                turn: aim_err.clamp(-0.08, 0.08),
                look,
                forward: if aim_err.abs() > 0.6 { 0.0 } else { 1.0 },
                ..Default::default()
            };
            // ARM ON THE RUN (the sisters): the switch rides inside the run — 0x2e until the
            // rocket attempt is finished, then 0x43 for the P3/P4 fire-on-move. No stand
            // (Player_WeaponChange needs none [M the game's switch path]; sprint-3's stand
            // cost ~20 hp [L]).
            // SPRINT-4 REVISED: the sisters arm the SHOTGUN on the run — the rocket plan is
            // parked: the gap-mouth rocket detonation hurt Bond ~48 at d 13.7 while the disc's
            // own blast radius is 8.0 [L ee_snapshot weapon_data+0x2e·0x10c+0x08 = 8.0, damage
            // 100.0] — a port blast-law divergence recorded for its own task (the walk fires
            // no rockets). The drop below fires the shotgun from the same converged stand.
            if agent_run {
                let want = 0x43;
                let s2 = w.scene.sim.as_ref().unwrap();
                if s2.gun.weapon != want {
                    let sw = s2.gun.switch_request;
                    input.switch_to = if sw == want { None } else { Some(want) };
                }
            }
            if trigger { w.tick_fire(input); } else { w.tick(input, false); }
        }
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x3f),
            "the deck sprint must take the parachute ON THE PASS (standing at the slot is the death: [L the death run — 95 ticks at the slot, 3 × 11.2 + 5.2 from #4, the take never fired]); bond at {:?}",
            w.scene.sim.as_ref().unwrap().bond.pos);
    } else {
        // The deck east from the re-place point IS the game's route and it WALKS: the walkthrough
        // arrives at the deck below the pickup in both walks (deterministic 60 Hz — once green,
        // always green), so no stand-in: the arrival is a hard assert. (The prior session's wedge
        // transit is gone; the deck's full extent was never a blocker on this tree.)
        w.walk_to(Wp::new(pp[0], pp[1] - 1.4, pp[2] - 5.0).r(1.6), "the deck below the parachute");
        w.wait(10);
    }
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
    // The E-plate #166 side stand is short; the deck sprint below is the real-health crossing.
    // (Warm walks only: the real-health sprint-through above arrives — and takes — moving; a
    // walk_to here would re-arm the return-fire gate at the slot, the stand the sprint exists to
    // avoid.)
    if !real {
        w.walk_to(Wp::new(pp[0] - 0.3, pp[1] - 1.4, pp[2] - 1.0).r(1.0), "the parachute take slot (walked)");
    }
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
            // SPRINT-5 (the sisters): hop on the FIRST grounded tick — the drive starts at the
            // deck's east edge (the P5 lane) and every standing tick is a wake-volley tick;
            // the %30 pacing cost the 00 Agent run its last 11 ticks [L agent00_run4]. The
            // d1/warm walks keep the %30 pacing byte-identical.
            let jump = if real && w.scene.sim.as_ref().unwrap().drones.difficulty >= 2 {
                sim.bond.grounded
            } else {
                w.total_ticks % 30 == 0 && sim.bond.grounded
            };
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
    walk_to_the_mission_end(&mut w, false, false, false);
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
    fresh_boot_asserts(&w);
    walk_to_the_mission_end(&mut w, true, false, false);
}

/// THE FRESH-SPAWN WALK ON REAL HEALTH (`damage_log.rs::with_real_damage`): the same walked
/// route, the top-up off — no health, armour or ammo writes anywhere — and Bond's death
/// panics with the damage log (who, weapon, range). Auto-aim is the app default; the fights
/// are the walker's own return fire under this walk's opt-in flags.
/// The CORRIDOR death was fixed first: the world hits were the terminal-room GT wall guns
/// #174/#175 (class-0x27 GT placements [D], p0=1/p4=0xe [D dump]; weapon record 96 = 0x60,
/// owner −1) firing dead-on — the game's case 4 eases the barrel first
/// (`Quat_Slerp_Acc(REC_FRAME_RATE·4)` [M 0x180498:295-330]) and fires along
/// `Mat_GetDir(barrel obj+0x90)` [M :344-352, disasm 0x180a28-0x180bac], which lags a runner;
/// the port does the same (gt.rs combat_fire_rides_the_eased_barrel_not_the_dead_on_line) and
/// the crossing costs 23 × −2.40 = 55.2 ([L] the runs here). Whether the real machine's
/// crossing costs ~55 is the report's live check (the [M] law's own yield on this route is
/// what the port shows).
/// THE ROOF DEATH (tick 16652 at the deck key, −14.71, 58.13, −2.06) was the player-side
/// stalls, and the weapon told the story: Bond's ONLY conventional weapon on 0b is the
/// Korsakov K5 sleep dart (0x43, dmg 0.10, range 45 — the 09 kit [M 0x1a7ef8 case
/// 0x7000009/0a/0b; L the bullet probe]) and the game's dart law DROPS msg 0x19 for a guard
/// in full combat (`DSTATE_Attack` 0x164210 carries no case 0x19; the state gate is
/// `DroneFunc_HandleImpact` 0x1469b8:315-361 — the port's `Drone::dart_impact_state_ok`) —
/// the ring fights stood 833 ticks in the open tapping dead-on darts into ATTACK-state
/// guards (tgt hp 10.0 pinned — [L] the fight traces), and on the roof the plain 20 u
/// shooters window never even picked the d 36-42 deck guards. The player's answers, now
/// opt-ins this walk sets: `lane_fights` (the 48 u window — the deck guards become
/// fightable), `tranq_fights` (answer only guards whose state carries case 0x19; the WIN is
/// the sleep — the dart's HitDamage is never called [M 0x13fff0:269-306] so hp never moves —
/// reading the dart family, `Drone::dart_asleep_state`), and the deck crossing is the
/// player's SPRINT: the sleep these guards take is `RecoverTime(0x43) = 36·(200−+0xB9)` ≈
/// 144 ticks before the wake [M 0x147188 k=60; L the ring sleeps #1/#3, both back shooting
/// inside ~150 ticks], so standing to dart four ATTACK guards is the stall that bled the
/// earlier runs — a moving target cuts the guards' roll instead [M 0x179740
/// `DroneFiring_TargetMoving_Accuracy`]. The cut-scene tail waits until the GAME re-enables
/// Bond (the arrival cut-scene's `wait_enabled` pattern) — the fixed 2900 stood him 450
/// extra ticks at the deck key under fire with no control ([L] 11 hits, 23.8 → 0.0, no fight
/// ever started). Green run (level-0b-walk-after-scrape-draw, after s/fx-scrape-and-impacts'
/// draws shifted the stream): the deck crossing is the SPRINT-THROUGH (the sprint ends at the
/// take slot, the take fires on the pass — the walked take-slot stop's return-fire gate stood a
/// blocked dart fight at the deck's south-west while drone #4 chewed 58.8 at 4.5 u, the
/// real_damage_agent/00agent deaths at tick 16047); the mission completes with 62.8 left (d1),
/// 42.0 (Agent), 27.5 (00 Agent), 0 return-fire fights; stuck 0, the one fall is the parachute
/// jump itself.
#[test]
fn walk_fresh_spawn_to_the_mission_end_real_damage() {
    walker::with_real_damage(|| {
        let mut w = load_fresh();
        // The player's fight package for the open-floor crossings (the 07000013 walk's opt-in,
        // common/damage_log.rs `lane_fights`): the 48 u shooters window (the roof deck's guards
        // work Bond from d 36-42 [L the death run's damage log] — the plain 20 u window never
        // even picked a fight), the damage-weighted pick, the gun-ray exposure (the ring
        // corridor's fights tapped dead-on rounds that all died on the door jamb the eye ray
        // clears by a hair — [L fight#1/fight#3, tgt hp 10.0 pinned across ~9 rounds]), the
        // floor-drop resupply and the unseen-cover creep. `back_off` 3.0: under 3.0 u every
        // drone round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115].
        w.dmg.lane_fights = true;
        w.dmg.back_off = Some(3.0);
        // The K5 kit's own fight rules (`tranq_fights`, opt-in): Bond's only conventional weapon
        // on 0b is the Korsakov K5 sleep dart (the 09 kit [M 0x1a7ef8; L the bullet probe — wpn
        // 0x43, dmg 0.10, range 45]), and the game's dart law drops msg 0x19 for a guard in full
        // combat [M 0x164210; 0x1469b8:315-361] — the death run's fight#1/#3 stood 833 ticks in
        // the ring open for 21 health against ATTACK-state guards the darts could never touch.
        // The fights answer only dartable guards and the win is the sleep, not a wound.
        w.dmg.tranq_fights = true;
        fresh_boot_asserts(&w);
        walk_to_the_mission_end(&mut w, true, true, false);
    });
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-08-09-0a-0b): the
/// same walked balcony → exec floor → terminal → roof → parachute route at the menu's Agent pick
/// — GameState+0x28 = 2 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group
/// (09-0b) the pick scales Bond's pain by Plr_DMod_Normal 0.8 instead of Easy 0.6 and the
/// guards' hit chance by DroneFiring_Accuracy_Normal 1.0 instead of Easy 0.8 [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 09-0b group ← ReadTuningVars
/// 0x1d3a80]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. The same
/// fresh boot with NO channel pokes (`fresh_boot_asserts` pins the channels off) and the same
/// fight package (`lane_fights` + `tranq_fights` + back_off 3.0). Real input only — the load
/// path is the only difference.
#[test]
fn walk_fresh_spawn_to_the_mission_end_real_damage_agent() {
    walker::with_real_damage(|| {
        let mut w = load_fresh_at(2);
        w.dmg.lane_fights = true;
        w.dmg.back_off = Some(3.0);
        w.dmg.tranq_fights = true;
        fresh_boot_asserts(&w);
        walk_to_the_mission_end(&mut w, true, true, false);
    });
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-08-09-0a-0b): the same
/// walked balcony → exec floor → terminal → roof → parachute route at the menu's 00 Agent pick —
/// GameState+0x28 = 3 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 2 carries 3 [D]; Txt 0x14f = "00 Agent"]. On this level
/// group (09-0b) the pick scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.8); the
/// guards' hit chance stays DroneFiring_Accuracy_Hard 1.0 — the SAME row Agent runs (Easy 0.8)
/// — and the guards' base bullet damage rides DroneDamage_Hard 0.8 (Agent 1.0) [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 cases
/// 0x7000009-a-b; tuning.rs 09-0b group]; the min-difficulty placements pass Drone_Create's gate
/// at d2 AND d3 [M 0x136be0:49]. The same fresh boot with NO channel pokes
/// (`fresh_boot_asserts` pins the channels off) and the same fight package (`lane_fights` +
/// `tranq_fights` + back_off 3.0). Real input only — the load path is the only difference (plus
/// the d3-gated tactics the deaths force, inside this file).
/// SPRINT-5 LEDGER (still red, the closest pass yet — the Agent sister above IS green): the
/// sprint-5 deck package (the P2 aim through #5's drift, the close on #4 before the turn with
/// the dart held to d 5.5, the funnel stand off, the take-zone approach west of the housing
/// column, the dash-through to the deck's east edge with the take on the pass, the first-tick
/// hop) lands Bond AT the take zone with the parachute taken and #4 asleep (state 0x50 through
/// the whole dash [L agent00_run6/7 g4 traces]) — and still dies at tick 16517 at (7.13,
/// 58.07, 41.24), 2.7 u short of the edge, on the long-range toll alone: #9 (0x18 at (17.3,
/// 58.0, 22.7), d 21) 21.0 in 3 hits, #6/#8/#7 3.5s, after the P1/P2 legs and the fixed 16.0
/// world toll left 17.5 hp at the take [L agent00_run7]. The Agent sister runs the SAME route
/// with the same geometry and survives (its 0x18 rounds are 5.6, its P1/P2 toll ~9 lower).
/// SPRINT-6 (2026-11-10): lever (1) is RESOLVED — the [0b-stall] "vel" print read BL+0x50
/// (the collision velocity, zeroed while grounded), NOT the accuracy's move quad; the quad
/// (Player::move_local [M 0x191dd0:166-170]) is non-zero on every running tick and every
/// guard's latch reads moving true through the whole run [L a00_run1 t15947+], so master's
/// 566f1d298 law already covers this walk. The sprint-6 attempts on the P1/P2 toll are
/// ledgered in the ignore message below (the rocket arc measurements; the live-wire 16.0).
#[ignore = "00 Agent (sprint-6 ledger, tree 2026-11-10): dies at (7.13, 58.07, 41.22) tick 16517, 2.7 u short of the deck edge — 22 hits for 100.0: #9 21.0 in 3 (0x18 at d 21-35), #6 14.0, #8 10.5, #7 10.5, #4 28.0 in 4 (14.0 at d 20-25 during P1 + 14.0 at the close; the dart then slept him through the dash), #5 10.5, the world 16.0 = FOUR 4.0 live-wire hurts in the ring corridor (frames 13380-13423 at (−1.4, 47.8, z 2.9-6.2) — the fusebox law [M 0x1adfe8:99-101: a group-1 wire death hurts while +0x56 == 0, unconditional on distance; the deaths ride guard rounds/script, the walk cannot dodge them]). RESOLVED LEVERS: the vel-0 moving-accuracy divergence is DISPROVEN — the sprint-5 [0b-stall] vel print read BL+0x50 (the collision vel, zeroed while grounded), NOT the accuracy's move quad; the quad (Player::move_local, [M 0x191dd0:166-170]) is non-zero while running and every guard's latch reads moving true through the whole run [L a00_run1 t15947+: g4/g6/g7/g8/g9 moving true]; master 566f1d298 already carries the law and applies TargetMoving_Accuracy 0.75 [M 0x179740:71-73; drone.rs apply]. The port's blast radius obeys the disc exactly [L a00_run3: detonation d 5.2, radius 8.0, damage 100.0 → 47.37 = 100·(1−4.21/8)] — the sprint-4 'hurt at 13.7' was a mis-attributed detonation distance (the explode-hurt-radius task should re-measure the blast POS, not the radius). THE MEASURED DEAD LEVER: one carried 0x2e round into the band anchor #6 (all four within 6.1 u [D drone-props]) — three real-input attempts missed: a flat shot hit a SW deck structure at (−6.5, 58.6, 7.9) 5.2 u from Bond (−47.37) [L a00_run3]; the round ARCS (1.33 u below the line at d 14 [L a00_run4]); drop-compensated shots flew past the cluster [L a00_run5/6] — the 0x2e flight law (speed + gravity) must be probed first (Bullet_init 0x12c5a0's fields for the record). NEXT LEVERS: (1) pin the 0x2e flight law, re-arm the band shot at control return with the fire-tick 0x43 switch-back [L a00_run5: the 100-tick cold-gun close death at d 1.0 of #4 under ×2 TooClose rounds, M 0x139fac]; (2) a bounded close+dart on #5 at the gap mouth (his 10.5 + the a00_run6 point-blank 21.0 at d 2.4 say the P2→P3 pass needs the same treatment #4 got); (3) the live-wire 16.0 is the fusebox task's to avoid (cut the group before the ring-corridor walk)"]
#[test]
fn walk_fresh_spawn_to_the_mission_end_real_damage_00agent() {
    walker::with_real_damage(|| {
        let mut w = load_fresh_at(3);
        w.dmg.lane_fights = true;
        w.dmg.back_off = Some(3.0);
        w.dmg.tranq_fights = true;
        fresh_boot_asserts(&w);
        walk_to_the_mission_end(&mut w, true, true, false);
    });
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
        w.after_step();
    }
}

/// XZ distance for the return-chain checks.
fn dxz_w(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }
/// THE ROCKET'S BLAST HOLDS THE DISC'S RADIUS ON THE REAL PATH (explode-hurt-radius). Bond walks
/// from the fresh Agent spawn to the exec office (no teleport after the load), takes the disc's
/// PICKUP_RocketLauncher #349 (weapon 0x2E, 8 rounds [D docs/levels/0700000b-audit.md:191]) on the
/// office jog, then fires two rockets from the walked desk spot at walls the sim itself measures:
/// the far wall >= 10.5 u away (the victim's sphere edge 9.4+ u, past the row's 8.0 radius [L
/// ee_snapshot.bin @0x2c2178]) must hurt NOBODY; the near wall <= 7.5 u (edge < 8.0) must hurt
/// Bond by the straight-line falloff [M Explode_Propagate 0x17c5e8-0x17c620; the pick
/// 0x1eae80:221-228; the class-3 arm 0x17c650-58]. The level-0b-roof-sprint-4 observation (a ~48
/// hurt at the d-13.7 fire) is impossible under this law — this walk is the real-path refutation.
#[test]
fn walk_rocket_blast_holds_the_disc_radius() {
    walker::with_real_damage(|| {
        let mut w = load_fresh_at(2);
        w.dmg.lane_fights = true;
        w.dmg.back_off = Some(3.0);
        w.dmg.tranq_fights = true;
        fresh_boot_asserts(&w);
        walk_to_the_mission_end(&mut w, true, true, true);
    });
}


/// THE PATERNOSTER CAB IS A SEALED CAGE — the level-0b-lift-exit session's evidence, pinned so a
/// future "make the ride walkable" change must first refute this on the disc's own bytes:
///
/// - The cab (obj 'lift' 0x02000367, 1435 collision tris [D the 0x2E block]) is a tiered box:
///   horizontal levels local −2.89/−2.55/−1.68/−1.24 (the base), **−0.21..−0.08 (the platform
///   floor, 140 tris, x −2.59..1.48 z 23.60..27.24)**, +1.07 (a 4-tri south ledge), +2.26..+2.61
///   (the rail band, 124 tris), +3.76 (the roof slab) and +4.03..+4.10 (a centre cap). The keyed
///   y rides matrix[3][1]; the platform floor stands at keyed −0.15.
/// - The glass (obj 'lift_glass' 0x02000421, 56 tris, local y −1.26..+2.46, parented entity
///   0x2000421 [D nis dump stream 1]) closes the cage at body heights, and the game gives it NO
///   kind exclusion: the collide node's kind-exclusion word cel+0x98 is written ONLY by the room-cel
///   fixup (parseentity_fixup 0x1d00c0:115/142, cel kinds 0xc023/0xc047) and the static parser
///   writes ONLY cel+0x94 (parsemap_block_map_data_static 0x1d1998:101-103) — a dynamic/script
///   entity's node carries +0x98 = 0, so Collide_Intersect 0x1ebaa8:301-317's kind filters
///   (node bit 0x1 vs hittest kinds 0xd/0xe, bit 0x2 vs 0x10, bit 0x800 vs 0x40) never open it.
/// - Measured on the port's world (the same celglist law [M Script_CreateEntity 0x1d8968's
///   fallthrough → Control_SetGList]): at the exec pass (cab y 46.93, floor 46.78) every
///   east/west/south/north face line at floor+0.5/+0.9/+1.5 hits — 9 z-lines per east/west face,
///   9 x-lines per south/north face, 0 openings; the interior has NO standable surface between
///   cy+1.6 and cy−0.5 except the west/north edge bits of the −0.15 floor rim.
/// - The exits from the RIDE surface (the roof slab, keyed +3.76 — the game's feet ray from above
///   [M 0x194188] lands there exactly as the port's does): EAST the waiting-room floor hole
///   (x 2..6, z 24.5..27.5 has no floor to y 39 [D the room ray grid]), SOUTH the shaft's south
///   wall (top 55.5, walled at lid+0.3..+2.1 at every probed x −1.2..1.45), WEST/NORTH wall/void.
///   The AI nav has no balcony→exec route either (`nf-extract route` from the spawn area
///   (2.3, 14.0, 25.8) to nav n41 (12.21, 46.9, 19.68): "no route: code 5").
///
/// So nobody leaves through the cab's inside: the route rides its roof to the exterior vent.
#[test]
fn the_paternoster_cab_is_a_sealed_cage() {
    let mut scene = nf_view::load(&iso(), "0700000b.bin", false).expect("load 0700000b");
    for _ in 0..3000u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    // Ride the cycle to the exec pass (the cab floor level with the 46.8 sill).
    let cy = {
        let mut cy = None;
        for _ in 0..2500u32 {
            scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            let cur = scene.sim.as_ref().unwrap().drones.scripts.objects.list.iter().flatten()
                .find_map(|o| match o { nf_game::nis::Obj::Entity { hash, matrix, .. } if *hash == 0x2000367 => Some(matrix[3][1]), _ => None });
            if let Some(c) = cur { if (c - 46.93).abs() < 0.06 { cy = Some(c); break; } }
        }
        cy.expect("the cab passes cy 46.93 [D nis dump keys]")
    };
    let floor = cy - 0.15;
    let sim = scene.sim.as_ref().unwrap();
    // The four faces, probed at body heights on 0.5-spaced lines: EVERY line hits (no door).
    let mut lines = 0;
    let mut open = 0;
    for i in 0..9 {
        let z = 23.3 + 0.5 * i as f32;
        let x = -2.5 + 0.5 * i as f32;
        for (from, to) in [([4.0f32, 0.0, 0.0], [-4.0, 0.0, 0.0]), ([-4.5, 0.0, 0.0], [4.5, 0.0, 0.0])] {
            for h in [0.5f32, 0.9, 1.5] {
                lines += 1;
                if sim.world.segment([from[0], floor + h, z], [to[0], floor + h, z]).is_none() { open += 1; }
            }
        }
        for (from, to) in [([0.0f32, 0.0, 20.5], [0.0, 0.0, 25.0]), ([0.0, 0.0, 29.5], [0.0, 0.0, 25.0])] {
            for h in [0.5f32, 0.9, 1.5] {
                lines += 1;
                if sim.world.segment([x, floor + h, from[2]], [x, floor + h, to[2]]).is_none() { open += 1; }
            }
        }
    }
    assert!(lines >= 100, "the face probe ran its grid ({lines} lines)");
    assert_eq!(open, 0, "every face line at body heights hits: the cage is sealed [D obj 0x02000367/0x02000421 collision; M the node kind-exclusion cel+0x98 is 0 for objects (0x1d1998:101-103 writes only +0x94)]");
    // The interior floor rim: the platform floor (keyed −0.15) is unreachable — no standable
    // surface in the open middle at the boarding heights.
    let mut interior_hits = 0;
    for zi in 0..8 {
        for xi in 0..10 {
            let x = -1.5 + 0.3 * xi as f32;
            let z = 23.8 + 0.4 * zi as f32;
            if sim.world.segment([x, cy + 1.6, z], [x, cy - 0.5, z]).is_some() { interior_hits += 1; }
        }
    }
    assert!(interior_hits <= 6, "the cage interior is open air at body heights ({interior_hits} stray rim hits) — no walkable interior [D the −0.15 floor tier is enclosed]");
}
