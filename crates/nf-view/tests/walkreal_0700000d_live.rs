//! walkreal-b: 0700000d (the power station, part 2) walked with the player's own movement — no
//! teleports except the one documented stand-in (the catwalk→bay exit). Route
//! (docs/levels/0700000d.md): the warehouse start #1 (179.53, 11.09, −205.42 [D]; the start
//! trigger #436 → ch 0x23 completes objective 3) → the dock's south edge → THE CRANE CHAIN, real
//! input: the platform drop to the warehouse ground, the ground walk east, the ladder #435 climb,
//! the wire #271 jump-grab from the ladder top (the third icon #297 [D]) and the hand-over-hand
//! ride to the rotate crane's deck, the E on #450 and the 61°→−11° rotation [D 0x06000a08 keys]
//! carrying the rider — the dock→deck crossing is real — then the DECK→WALKWAY WALK-OFF, real
//! input (level-0d-jib-walk-off: the end-pose rim walk-off lands on the static 12.13 strip, the
//! jump being vertical-only [M Player_HandleJump 0x1916b0; Player_SSWalk 0x192310:69]) and the
//! catwalk walk west to its end, the dock E #460 (ch 0x2b, real input), then THE BOARDING, real
//! input: the Q-Claw fired from the control corner at the pulley #370 (the clear line and the
//! 14 u reach are the data's), the hang over the gantry's east park, the release with a west
//! drift onto the east girder (the aboard plate 0x0200_0a81 fires ch 0x67), the ride west and the
//! drop into the yard at the nav web's north edge
//! → the REAL yard walk to the ST-3 plate #378 (118.48–157.56, 3.74–13.26, −311.35..−306.49 [D])
//! → ch 0x1d (objective 4) → the REAL laser photograph (0x54 carried [M 0x1a7ef8 case 0x700000d
//! keeps the loadout]) into the CamSubject volumes #494/#495 (ch 0x1e [D]) → ch 0x1e (objective
//! 5) → the REAL yard walk north to the C5 door → the REAL kill: shoot `bigdoor02` #96 to death
//! (health 10000 [D]; `Destroy_CollisionHandler` 0x12bd48 subtracts EVERY class-5 bullet's
//! damage, death writes ch 0x1f [M :35]) → ch 0x1f (objective 6) → through the dead door, the
//! corridor plate #433's low stretch fires ch 0x28 ([D; M triggers.rs — the re-firing Touch whose
//! exit clears it, so the crossing comes after the kill]) → the AND #434 [0x1e & 0x1f & 0x28] →
//! ch 100 → `Mission_Update` state 6 ends the level to 0x730000d [M 0x182c60:102-103]. The disc's
//! own hint "your only way out is through that security door marked C5 … considerable firepower"
//! [D USATxt 0x4000066] — the shooting route is the game's too; the mounted-laser CUT-SCENE
//! remains unported (docs/levels/0700000d.md §6), the laser GUN is the level's own #225 [D].

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn bond_pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

#[test]
fn walk_arrival_to_the_c5_door() {
    walk_arrival_to_the_c5_door_at(1);
}

/// The difficulty-parameterized body (real-health-agent-0c-0d): 1 is the byte-identical
/// master replay; 2+ (the Agent sister) loads the menu way BOTH times — the cold menu pick
/// and the warm level change both run GameState+0x28 = 2 [M C_SBNFDFCTY 0x216ca8:15 →
/// load_with_carry_difficulty; GameState+0x28 → Drones::difficulty, asserted after the
/// warm load]. Agent spawns the min-difficulty-2 placements [M Drone_Create 0x136be0:49],
/// scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain 0x1902c8] and the guards'
/// hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy 0x179740:74-88].
fn walk_arrival_to_the_c5_door_at(difficulty: u32) {
    let iso = iso();
    // The warm carry: 0c's record-11 channels ride the save [M 0x182270:92-95].
    let cold = if difficulty == 1 {
        nf_view::load_with_carry(&iso, "0700000d.bin", false, None).expect("load cold")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "0700000d.bin", false, None, difficulty).expect("load cold")
    };
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "0700000d.bin", false, Some(&save)).expect("load warm")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "0700000d.bin", false, Some(&save), difficulty).expect("load warm")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    // The aimed act (level-0d-real-health-crossfire-3, the driver's lever): the walker's own
    // return-fire fights hold the AIM button (the OPT-IN DamageLog::aimed_fights flag — the
    // 0700000c yard-deck-sentry precedent), so unplanned fights converge on the ray like the
    // scripted shoot_far legs do.
    w.dmg.aimed_fights = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 179.53).abs() < 2.0 && (p[1] - 11.09).abs() < 2.0 && (p[2] + 205.42).abs() < 2.0,
            "the warehouse start #1 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x23), "the start trigger #436 fires ch 0x23 under Bond [D]");
        assert_eq!(sim.drones.mission.states[11][2], 4, "objective 3 (TR-2) completes [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[12][0], 3, "ch 0x23 reveals objective 4 [L record 12]");
    }

    // Leg 0: the level's own armour at the start — the BodyArmourHeavy #525 (180.64, 11.18,
    // −208.82 [D placement], 3.4 m south of the start) taken on the walk-over [M Pickup_Handler
    // 0x18e650: type 3 refused while armour ≥ 50, else hard-set 50.0]. The yard ahead is a
    // five-sniper crossfire (probe_0d_los_grid: #31 the perch, #16/#33 the dock, #27/#29 the
    // west roof all fire the game's law) and the real-health budget needs the 50 armour the
    // game itself offers at the door.
    w.walk_to(Wp::new(180.6, 11.2, -208.8).r(0.9), "the BodyArmourHeavy #525");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.health.armour >= 49.0,
            "the armour pickup taken on the walk-over [M 0x18e650 type 3], got {}", sim.drones.health.armour);
    }

    // THE PLATFORM KILLS — the report's one untried free lever, now MAPPED: from the armour
    // arrive point on the warehouse main floor ((180.46, 12.19, −207.93), probe-verified stand:
    // settles at y 12.19, rays CLEAR) the raw rays to the dock patroller #33 (d 28.4) AND the
    // perch #31 (d 34.0, torso y 7.15) are CLEAR while BOTH cones read the stand blind (vis
    // 0.00 → no LOS probe [M DroneVision_ProcessDroneSight 0x176a28] — probe_0d_platform: every
    // stand x 176-184, z −205..−222). The flat geometry kills the dock face problem: eye ~13.6
    // → torso ~14.0 over dxz ~29 cannot be eaten by the face the way every steep ground shot
    // east of x ~192 was [probe_0d_climb_cover: 8+13 aimed rounds all died on the face]. The
    // stand keeps Bond OFF the lip: the walkable main floor ends z ~−207.6 at x ~178 (standing
    // at the dock-edge arrive point slides Bond down to a y ~7.9 ledge — measured: from that
    // ledge every #33 round died on the face), so the kills fire from the armour leg's own
    // arrive point and the dock-edge leg runs AFTER them. The west roof (#27/#28) is still
    // ASLEEP — it wakes only when Bond stands at the drop point [probe part E] — so both kills
    // happen with nobody watching. #33 dies first (the flatter target), then #31 (the
    // steep-down hip-fire lands — the ground perch kill lands steep-up the same way): the drop
    // then starts the climb with the strip's two awake snipers already down (the post-drop kill
    // calls stay as log-only fallbacks). Real fire path (shoot_far holds the stand), no writes.
    w.wait(30);
    {
        let p = bond_pos(&w);
        assert!(p[1] > 11.0, "the kill stand holds the warehouse main floor y ~12.19 (no lip slide), got {p:?}");
    }
    let platform_pool = w.scene.sim.as_ref().unwrap().drones.health.health;
    let dock_killed = kill_guard_at(&mut w, [194.0, -232.2], [5.5, 3.0], "the dock patroller #33 (the platform shot)");
    let perch_platform = kill_guard_at(&mut w, [161.1, -235.2], [2.0, 2.5], "the perch sniper #31 (the platform shot)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.drones.health.health;
        eprintln!("[0d] the platform kills: #33 down {} #31 down {} pool {:.1} -> {:.1}",
            dock_killed, perch_platform, platform_pool, h);
        assert!(h > platform_pool - 48.0, "the platform shots must not eat the pool, {platform_pool} -> {h}");
    }

    // Leg 1: south to the dock's south edge — walked. The dock's south edge drops to the
    // under-dock trench (the floors 3.0/0.0 [D probes]) whose far side walls 3.9 m up at the
    // yard's north edge (the trench floor 0.0 ends z −266; the yard begins 3.9 at z −272 [D
    // probe walkreal_probe0d.rs]).
    w.walk_to(Wp::new(178.0, 11.2, -206.5).r(1.0), "the dock south edge");

    // THE CRANE ROUTE, real input (level-0d-crane-and-to-the-end). The warehouse platform's
    // south edge drops ~11 m to the ShippingWarehouse ground (the game's own drop — the fall is
    // survivable, no damage gate [probe4]); the ground descends south to the ladder #435
    // `ladder2crane1` (212.22, 8.14, −234.51 [D]; sphere r 8.21 → the pick spans y ≈ 0..16.35
    // [M Player_CollLadder]), whose top (17.37 walked) carries the wire third-icon #297
    // (213.26, 17.58, −234.52 [D p0 = 3]) — with the icon in context a jump grabs the NEAREST
    // wire within 1000 [M Player_HandleJump 0x1916b0:118-147]: the hand-over-hand wire #271
    // `wire_crane2crane` (centre (205.61, 17.04, −248.52), r 14.02, yaw −1.065 [D]; type 0 [D]).
    // Strafe +1 = mode 7, facing wire.yaw + 3π/2 [M Player_Wire 0x1977c0-0x1979d0] and the b4/b5
    // root motion (+z local, 0.7637/0.7105 per cycle [D tracks on 0700000d]) carries toward the
    // A end — the ride is ~24 m ≈ 1370 ticks, inside the 1800-tick auto-drop [M 0x197748-0x197754].
    // The drop lands on the rotate crane's deck (199.25, 14.16, −260.16 walked); the deck walk
    // reaches the control plate #450 `crane_touchplate` (199.53, 13.31, −263.81 [D]) whose E
    // writes ch 0xa [D #450 p0; M Trigger_Activate 0x1b2318:23-33], starting SP #492
    // `ScriptPlayer_CraneRotate` (mode 0x40, mask 1 [D]) whose script 0x06000a08 rotates the
    // crane 61° → −11° over its 750 ticks [D keys: quat y 0.508 → −0.096, t 0..750] — and the
    // rider on the rotating deck is carried (ground_motion [M Player_FeetOnPoint 0x194188 →
    // Control_InheritVelocity 0x133e88]).
    assert!(w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the platform drop to the warehouse ground"),
        "the platform drop must reach the ground");
    {
        let p = bond_pos(&w);
        assert!(p[1] < 6.0, "the drop lands on the warehouse ground (y ~1.04), got {p:?}");
    }
    // THE STRIP-MOUTH KILLS — the climb's cover work, a player taking the shooters out from
    // cover before climbing into their crossfire (the driver's third lever). The perch #31 owns
    // the whole climb: vis 1.68 with a clear ray at every ladder point, its blind wedge covering
    // only the strip itself [probe_0d_climb_cover part B] — and from the drop point the reverse
    // ray to it is CLEAR at d 21 while its idle cone still reads the spot blind (vis < 0.7 → no
    // LOS probe [M DroneVision_ProcessDroneSight 0x176a28]). THE PERCH KILL IS THE MISSION'S
    // FIRST GUNSHOT and it is FREE: 19 ticks, one tap, zero incoming (measured twice) — fired
    // before any guard has a reason to face the strip. The dock patroller #33 (state 0x7,
    // census (191.26, 13.66, −232.22) walking east ~0.003 u/tick) is the NEXT threat (2 hits on
    // the climber, the death at 2988) but its clear window has CLOSED by the drop: east of
    // x ~192 the dock's south face eats every steep shot from the west (measured: 8+13 aimed
    // rounds at d 4-18 from x 180.8-194.3 with dead-centre aim all died on the face) — the
    // attempt is kept (cap 600, log-only) so the next message inherits the measured state, and
    // the outcomes are LOGS, not asserts: the walk must stay green in both twins while the
    // tactic is unfinished. The real fire path end to end (kill_guard_at → shoot_far: the
    // walker's aim + trigger taps, range-scaled gates — the walker's ±0.02 rad gate is wider
    // than a guard's ~0.3 u torso box beyond ~15 u). Guards stand at 10 health [probe census];
    // the held 0x1e carries clip 5 + spare 30 [loadout]. The pool is 100 health — the pain
    // types 1..7 carry armourFraction 0, so the 50 armour cannot absorb bullets [M
    // Player_HandlePain 0x1902c8:84-104]; the 180-tick per-guard cadence [D weapon 0x1e data;
    // M DroneWeap_NextBulletTime 0x179d40] makes every standing fight a race.
    let health_before_kills = w.scene.sim.as_ref().unwrap().drones.health.health;
    kill_guard_at(&mut w, [161.1, -235.2], [2.0, 2.0], "the perch sniper #31");
    kill_guard_at(&mut w, [194.0, -232.2], [5.5, 3.0], "the dock patroller #33");
    // The wire crossfire's kill order (level-0d-real-health-crossfire): the ray scan
    // (probe_0d_crossfire_rays) clears the strip -> south-roof #10/#18 rays for the VISION mask,
    // but the BULLET path dies on the canopy the vision sees through (measured: 600-tick
    // attempts, health 10, dist 72.8, pool untouched — the shots never land) — a player CANNOT
    // take the south roof from the strip; the attempts stayed out of the route. The roof work
    // happens from the LADDER TOP below (above the canopy line).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let perch_dead = !sim.drones.list.iter().any(|d| d.enabled && d.present && !d.corpse && d.health > 0.0
            && (d.pos[0] - 161.1).abs() < 2.0 && (d.pos[2] + 235.2).abs() < 2.5);
        let h = sim.drones.health.health;
        eprintln!("[0d] the strip kills: perch dead {} pool {:.1} -> {:.1}", perch_dead, health_before_kills, h);
        assert!(h > health_before_kills - 48.0, "the strip kills must not eat the pool, {health_before_kills} -> {h}");
    }
    // The blind strip transit (zero-hit while moving in every run — the wedge + the moving
    // target): the drop point → the ladder base along z −228..−235. THE #10 KILL rides the
    // transit (level-0d-real-health-crossfire-3, the driver's lever "take #10 before the climb
    // shots"): at the leg-2 stand (196, −228.5) — exactly where the transit already pauses —
    // probe_0d_ray_stands reads the eye→chest ray to the bay guard #10 (placement (214.21,
    // 12.09, −289.44), woken with the platform-drop group [L probe part E]) CLEAR at d 63.6,
    // while EVERY other shooter's ray to the stand stays blocked (g16 0.881, g18 0.521, the
    // roofs 0.83-0.86, the canopy pair 0.56). #10 never fired on the moving transit (zero-hit in
    // every run) — the stand is the same line. A player stops at the strip mouth and takes the
    // bay guard out through his own sight line before climbing into it.
    for (at, r, label) in [
        ([188.0f32, 0.5, -228.0], 2.0, "the blind strip east 1"),
        ([196.0, 0.5, -228.5], 2.0, "the blind strip east 2"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    let bay_killed = kill_guard_at_present(&mut w, [214.2, -289.4], [6.0, 6.0], "the bay guard #10 (from the strip mouth, ray clear d 64 [probe_0d_ray_stands])", 180);
    eprintln!("[0d] the strip-mouth #10 kill: landed {} pool {:.1}", bay_killed, w.scene.sim.as_ref().unwrap().drones.health.health);
    // The spawner #32 (skin 0x5000090, the ground band (157.6..163.3, −213..−229)) is NOT
    // taken here any more — level-0d-real-health-crossfire-4 MEASURED the trade: the landed
    // strip-mouth kill (the world-ray gate + the east band box, down at +528 when #32 crossed
    // to the band's east end where the ~1.7-high obstacle at (160,−217) is behind the target)
    // cost 528 ticks of strip-mouth dwell, and the wake's reposition walkers used exactly that
    // window to settle into their x ~151.4 combat spots and shoot the CLIMB — the -4 runs 2/3
    // died at the climb top (4032) with 86 on the ladder face, vs run 1's 16 with the leg
    // failed at 400 and the -2's SILENT climb with no leg at all. The kill moved to the bay
    // phase (below, after the annex drop): the same gate measures the bay stand's ray live,
    // and #32's late-route fire is the ~0.7-per-hit trickle the -3's runs survived to 16287
    // with (86.4 across 124 hits, d 56-92) — cheaper than the climb's 16-per-hit snipers.
    for (at, r, label) in [
        ([204.0, 0.5, -229.0], 2.0, "the blind strip east 3"),
        ([212.0, 0.5, -230.0], 1.5, "the blind strip east 4"),
        ([215.0, 0.8, -233.2], 1.2, "east of the crane leg"),
        ([213.2, 1.5, -234.8], 0.8, "at the ladder base"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    assert!(w.try_walk_to(Wp::at(212.47, 16.0, -234.51, Act::Ladder { top_y: 15.6 }), "climb ladder #435"),
        "the ladder climb must top out");
    {
        let p = bond_pos(&w);
        assert!(p[1] > 16.0, "the ladder tops at y ~17.37, got {p:?}");
    }
    // THE WEST-ROOF SPAWN KILLS (level-0d-real-health-crossfire-5) — ALL THREE AT THE LADDER
    // TOP. The wake spawns the west roof DIRECTLY at stable combat spots (probe_0d_roof_rays
    // wake dump: g27 south (151.06, 22.70, −255.81), g28 north (146.4, −237.7), g29 later at
    // (151.05, −250.82)) and the slab-edge law is HEIGHT-ASYMMETRIC per line: from the ladder
    // top g27's spawn CHEST ray is CLEAR (the -2's blocked law was the mid-band −250..−252
    // obstruction, her south spawn is past it) and g28's spawn HEAD ray is CLEAR where her chest
    // dies at 0.907. The strip stands stay UNTOUCHED (the -5 run-2 lesson: a 600-tick aimed duel
    // at strip 2 fed the bay guards' spread fire through the low opening — 48+32 for nothing —
    // the strip's zero-hit law holds only for the baseline's short stands). g27 goes first (the
    // proven chest act lands when the chest line is clear), then g28 (the box act — the aim
    // point is a REAL clear hit-box centre, the +0.9 guess flew under the head box in run 2),
    // then #16, then g29's south-walk chest kill. Log-only: the walk stays green in both twins
    // whether or not a kill lands.
    // g28 goes FIRST (level-0d-real-health-crossfire-5, run-3 lesson): in the walked timeline
    // she settles at (151.43, 22.70, −241.25) — NORTH of the mid-band obstruction, the ladder-top
    // chest ray CLEAR (the -4 run-1 and the -5 run-1 sweeps both killed her there at +194) —
    // and a live g28 rides the wire (the -5 run 3: 4 hits for 62 on the ride). The probe's wake
    // spot (146.4, −237.7) is the parked-at-drop timeline, NOT the walked one — the run-3 box
    // (146.5, −237.8) found nobody and she survived to the ride.
    let _ = kill_guard_at_present(&mut w, [151.4, -241.3], [4.0, 4.0], "the west roof g28 (chest, her walked spot — the ladder-top ray is CLEAR [probe_0d_roof_rays])", 600);
    let _ = kill_guard_at_present(&mut w, [151.1, -255.8], [4.0, 4.0], "the west roof g27 (chest, her south spawn — the ladder-top ray is CLEAR [probe_0d_roof_rays])", 600);
    // THE BOX ACT STANDBY for g28 if the chest leg missed (the part-5 head first): log-only.
    let _ = kill_guard_head_present(&mut w, [151.4, -241.3], [4.0, 4.0], "the west roof g28 (box/head standby [probe_0d_roof_rays])", 400);
    // THE DECK GUARD #16 (level-0d-real-health-crossfire-3/4): the -1/-2 routes landed this
    // kill every run (+~200 ticks) from the ladder top — his deck fire rode the -4 run-1 sweep
    // window (38 at d 29.7); with the west roof dying above he is the phase's only shooter.
    kill_guard_at(&mut w, [198.9, -260.4], [6.0, 4.0], "the deck guard #16 (from the ladder top)");
    // The west-roof sweep, WALKER-CATCHING (level-0d-real-health-crossfire-3, measured): the
    // guards reposition off their placements (145.6-145.8 — CLEAR rays from the ladder top,
    // d 68-70.5 [probe_0d_ray_stands]) to stable combat spots x ~151.4 (the slab shadow, the
    // -2's 0.972-0.974 ray deaths), and they PATROL the band east-west afterwards (the wake
    // spots 151.05-151.43, −241..−251 and the stable ends −238.3/−255.8). A pass catches the
    // crossing of a clear-ray spot — the -3's kills at +222/+754/+193 and the -4 run-5's +194
    // — so the caps ride 900 (a 300 cap expires before the crossing lands). With #16 dead
    // first the pass exposure is #18's bay fire only. Settled survivors between crossings stay
    // unshootable and are the ride's hunters (the -2's measured 54 for one).
    for k in 0..3 {
        let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
            d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
                && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0
        });
        if !live { break; }
        kill_guard_at_present(&mut w, [151.2, -246.8], [6.0, 11.0], "the west roof sweep (the walkers)", 900);
    }
    // g29's SOUTH-WALK KILL (level-0d-real-health-crossfire-5): she spawns late at (151.05,
    // −250.82) — behind the rooftop obstruction from every post-wake stand (head rays 0.960-0.978
    // blocked [probe_0d_roof_rays]) — and walks SOUTH (~0.003 u/tick, the -4/-5 ride ledger:
    // −250.8 → −255.7 across the phase) toward the spot the ladder-top CHEST ray clears (probe:
    // #29 settled chest AND head CLEAR from the ladder top) — the proven chest act takes her.
    // The box only acquires her south of −250.5; the chest-ray gate holds fire while she is
    // behind the obstruction, so the leg waits out her walk and costs nothing when blocked. If
    // she never clears, the ride rides with one hunter (~54, the -2's law) — still inside the
    // budget the two spawn kills bought.
    let _ = kill_guard_at_present(&mut w, [151.2, -254.0], [3.5, 3.5], "the west roof g29 (chest, the south walk [probe_0d_roof_rays])", 1200);
    // THE D2 GROUND-BAND SNIPER SWEEP, from the LADDER TOP (dart-tase-stamps-check): the Agent
    // ledger on the -5 route measured the death at 8533 (201.87,16.01,-255.09), 6 hits for
    // 100.0, 0 return-fire fights — the decisive drain is the d2 ground-band snipers: #36
    // (skin 0x5000090, sniper 0x1e) fires 3 × 28.0 from d 56.6 (7932/8112/8479, settled
    // (158.8,1.0,−221.6)) and #21 (0x5000087, 0x1e) lands the killing 10.0 at 8533 (162.1,
    // 1.0,−215.4) — both outside the fights' 48 u window, both alive through the whole
    // climb+ride, and their death-time spots are exactly the -4's ground band (157.6..163.3,
    // −213..−229). The ignore note's lever ("a d2-gated aimed stand on the two d2 snipers from
    // the strip") is MEASURED DEAD at the strip mouth: the stand's west ray to the band dies
    // at 0.975 on the south-facing north wall and the 600-tick dwell feeds the south-roof/bay
    // fire through the low opening (the -5 run-2 law at d2: #20 44 + #33 28 + #22 28 — died
    // 2515). The ground drop point is half-blind too (its west ray dies 0.605 on a LOW
    // east-facing wall at x ~160, top between y 1.5 and 4.4 — the [0d:ray] trace) and its one
    // CLEAR window closes in ~200 ticks while the aim settles (the measured miss). The
    // LADDER-TOP stand is the one the route already dwells on for thousands of ticks at d2
    // with ZERO incoming (this run's ledger: no hit between 2051 and 7932 — the whole
    // west-roof phase is free), its eye 18.1 looks DOWN the band line over the low x ~160
    // wall, and the band is settled (not marching) by this phase. Guards stand at 10 health
    // [probe census] — one aimed hit each; the sweep repeats while live band guards remain
    // (the -5 west-roof pattern). D2-GATED: the Operative twin's route stays byte-identical
    // (its #32 spawner leg stays late-route; the kill order shifts would re-roll the shared
    // RNG stream the green twin's 22.2-end budget rides).
    //
    // THE D2 SOUTH-ROOF SNIPER #17 FIRST (dart-tase-stamps-check): the band sweep's own
    // gunfire woke the d2 south-roof sniper #17 (0x5000087, 0x1e, fire spot (192.3,19.6,
    // −267.4), d 39 from the ladder top — INSIDE the P2K shot's outer hearing radius 40 [L])
    // mid-sweep — measured: 3 hits for 66, the death 8171 AT the ladder top (the baseline
    // ledger never shows #17: the west-roof phase's thousands of shot-ticks are quiet, the
    // band-sweep shots are what wake it). Take it BEFORE the first band shot — the ladder
    // top is above the canopy line (the -2 note's own law for the south-roof work). The box
    // also catches the d2 deck sniper #20 (198.4,14.0,−260.7) if #17 has moved off its spot.
    // Log-only.
    if difficulty >= 2 {
        let _ = kill_guard_at_present(&mut w, [192.3, -267.4], [8.0, 8.0], "the d2 south-roof sniper #17 (the ladder-top shot, before the band guns)", 600);
    }
    if difficulty >= 2 {
        for k in 0..3 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.pos[1] < 5.0 && (150.0..=168.0).contains(&d.pos[0])
                    && (-230.0..=-210.0).contains(&d.pos[2])
            });
            if !live { break; }
            let _ = kill_guard_at_present(&mut w, [160.5, -220.0], [5.5, 8.0], "the d2 ground-band sniper sweep (the ladder-top shot)", 600);
        }
        // THE g21 HEAD LINE (dart-tase-stamps-check) — MEASURED: the RAY law holds (the head
        // line reads full CLEAR / static clear / bullet CLEAR from the ladder top while the
        // chest dies 0.965-0.975 on the ~1.7 jamb — the -5 height-asymmetry law confirmed for
        // this jamb) but the BOX/HEAD act's aim does not CONVERGE to it in 600 ticks: the
        // cross sticks at [-0.21, -0.28] (the aim pitch −0.10 vs the needed −0.29 — ~11 u
        // high at d 54), the chest legs converge to [-0.0,0.0] from the same stand, and the
        // kill never fires (g21 alive, health 10). The standby act has never landed a kill in
        // any run (the g28 standby always read "no PRESENT guard near"). NEXT LEVER: fix the
        // head act's pitch convergence (shoot_far_box's turn/pitch rate for a −0.29 down-hill
        // head line) or find the stand east of the (160,−217) obstacle (the -4 law: there the
        // obstacle is BEHIND the target — the late annex-floor #32 leg's own geometry, ~12000+
        // — too late for the ride+deck at d2). The legs stay (log-only, d2-gated, zero
        // incoming measured: the 1200 blocked ticks drew nothing once #17 was down) so the
        // next session inherits the live ray + the stuck-aim trace.
        for k in 0..2 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.pos[1] < 5.0 && (150.0..=168.0).contains(&d.pos[0])
                    && (-230.0..=-210.0).contains(&d.pos[2])
            });
            if !live { break; }
            let _ = kill_guard_head_present(&mut w, [162.8, -215.0], [4.0, 4.0], "the d2 band sniper g21 (the head line over the jamb, the ladder-top shot)", 600);
        }
    }
    // The -2 "NO #29 / NO #10 attempts" constraints are LIFTED by the new route (level-0d-
    // real-health-crossfire-3): #10 dies at the strip mouth BEFORE the ladder (the dwell no
    // longer acquires anything), and the sweep's 8 passes roll the placement rays the probe
    // measured CLEAR (#27 d 70.5, #29 d 68.0) — the -2 slab-edge law was measured against the
    // REPOSITIONED guard (engaged guards step into the slab shadow), not the placements.
    // Face along the wire toward its A end, settle grounded, then jump — the grab replaces the
    // jump with the icon in context [M 0x1916b0:118-147].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let desired = (199.78f32 - sim.bond.pos[0]).atan2(-259.04 - sim.bond.pos[2]);
        for _ in 0..30 {
            let sim = w.scene.sim.as_mut().unwrap();
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            sim.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 0.3, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    }
    let mut grabbed = false;
    for k in 0..80u32 {
        let sim = w.scene.sim.as_mut().unwrap();
        // The stick HELD through the jump — a player runs onto the grab with the stick forward
        // and keeps holding; the walk quad's eased lanes then read non-zero at the grab and the
        // RETAINED quad (walk-ticks-only zeroing [M 0x193600:129-134]) reads MOVING through the
        // ride — the guards' TargetMoving_Accuracy applies [M 0x179a48:34-41 + 0x179740:71-73].
        // The released-stick jump dead-zone-snapped the quad to [0,0] at the grab and the rider
        // read still (the wire crossfire hit at stationary accuracy).
        sim.step_bond(nf_game::player::Input { jump: (60..=63).contains(&k), forward: 0.3, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if sim.bond.substate == nf_game::player::SS_WIRE {
            grabbed = true;
            eprintln!("[0d:grab] tick {k} quad {:?} vel {:?}", sim.bond.move_local, sim.bond.vel);
            break;
        }
    }
    assert!(grabbed, "the wire #271 jump-grab must fire from the ladder top (icon #297 in context)");
    // Hand-over-hand toward A: strafe +1 (mode 7) until the wire drops Bond at the deck.
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 2600 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { break; }
            w.tick(nf_game::player::Input { strafe: 1.0, ..Default::default() }, false);
        }
    }
    // The auto-drop's fall settles onto the deck (probe8's landing) — polled, not a fixed 500:
    // every settled tick here is a burst window for the surviving roof guard (level-0d-real-
    // health-crossfire-2); a player moves the moment he lands.
    {
        let start = w.total_ticks;
        while w.total_ticks - start < 500 {
            let sim = w.scene.sim.as_ref().unwrap();
            if w.total_ticks - start > 30 && sim.bond.grounded { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        eprintln!("[0d] the deck settle took +{} ticks", w.total_ticks - start);
    }
    {
        let p = bond_pos(&w);
        assert!((p[0] - 199.25).abs() < 2.5 && (p[2] + 260.16).abs() < 2.5 && (p[1] - 14.16).abs() < 1.5,
            "the wire ride must land on the rotate crane's deck (199.25, 14.16, −260.16), got {p:?}");
    }
    // The deck guard: since the guards fire on the single blend-out word [M DroneWeap_Ready2Fire
    // 0x178c48:102-104] (bisected: green before that merge, red after), drone 16 (weapon 0x1E)
    // stands on the crane deck 0.8 u from where the wire drops Bond, between him and the cab: the deck walk
    // pressed into him for its whole cap and came off the deck's east edge to the ground at
    // y 1.7. A player shoots the guard in his way first (the real fire path).
    for _ in 0..3 {
        let near = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            sim.drones.list.iter().position(|d| d.team == 1 && d.enabled && d.present && !d.corpse && d.health > 0.0
                && (d.pos[1] - b[1]).abs() < 1.5 && ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < 6.0)
        };
        let Some(di) = near else { break };
        let _ = w.switch_to_armed("the deck guard");
        let _ = w.shoot(di, "the deck guard");
    }
    // The LAST roof guard from the deck: NO attempt (level-0d-real-health-crossfire-2 measured):
    // the deck eye (y ~15.5) is 7.5 BELOW the west-roof surface over ~50 u — the eye→chest ray
    // dies on the roof slab even earlier than the ladder top's 0.974 (the [0d:ray] audit; the
    // 260-tick attempt never landed in any run) — and every standing tick here is one more
    // 180-tick burst window for the surviving roof guard. The deck legs stay MOVING.
    // The south roof #18 from the deck: NO attempt — the bullet path dies here too (260-tick
    // attempts, health 10, dist 29.7-31.5, the vision ray clear but no landing); its one ride
    // hit (16) is the cheaper cost.
    assert!(w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the deck walk to the cab"),
        "the deck walk to the control plate must arrive");
    assert!(w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 60 }), "E the rotate touchplate #450"),
        "the E on #450 must complete");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0xa),
        "the E fires the plate's out ch 0xa [D #450 p0]");
    w.wait(900);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter().find(|p| p.placement == 492).map(|p| p.state) == Some(5),
            "the rotate SP #492 played out (750 ticks [D 0x06000a08])");
    }
    eprintln!("[0d] the crane chain walked: the drop, the ground, the ladder, the wire, the deck, the E, the swing — at {:?}",
        bond_pos(&w));

    // THE DECK→WALKWAY WALK-OFF, real input (level-0d-jib-walk-off). The walkway strip is a
    // STATIC map object at 12.13 — a diagonal band across the crane's rest-axis frame, r 5.5-10
    // with the band's lateral centre sliding +1.0 → −1.5 as r grows (probe20_static_map rays),
    // merging into the east-west catwalk x 157..196 at z −265.0..−266.25. The game's jump is
    // VERTICAL-ONLY — Player_HandleJump 0x1916b0 zeroes the velocity (Vec_Zero) then adds a
    // fixed WldGravity-derived impulse (the walk-state caller Player_SSWalk 0x192310:69 passes
    // WldGravity @0x2d8890 [D ELF symtab]) — so the rim gap is crossed by WALKING OFF: at the
    // swing's end pose the deck's rim on the strip bearing is open (the rest-pose rim stalls the
    // run — probe17 v4 — but the end-pose rim does not), and the run's carry (~2.5 m through the
    // fall at the run speed, gravity 0.0045/tick² [probe17 v3's traced fall]) lands inside the
    // strip's band at r ≈ 6. The walk-off aims at the strip point r 6.0/off +0.75 (the band at
    // r 6.0 spans off +0.5..+1.0 [probe20]); the post-swing ground carry leaves Bond facing away,
    // so the aim turns in place first (walking while turning wanders into the mast). Bond's pos
    // rides ~1.1 over the surface (deck 13.04 → pos 14.15; strip 12.13 → pos 13.22 [probe20]).
    {
        let (ax, az) = (200.48f32, -262.37f32);
        let (ux, uz) = ((188.65f32 - ax) / 13.5, (-268.92f32 - az) / 13.5);
        let (px, pz) = (-uz, ux);
        let tx = ax + 6.0 * ux + 0.75 * px;
        let tz = az + 6.0 * uz + 0.75 * pz;
        // The pre-turn: face the strip point before walking.
        for _ in 0..160 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                let desired = (tx - b[0]).atan2(tz - b[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                if err.abs() < 0.05 { break; }
                let mut sim_m = w.scene.sim.as_mut().unwrap();
                sim_m.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
            }
            w.total_ticks += 1;
        }
        // The walk-off: forward held, re-aiming, until the strip landing (grounded past the rim
        // at strip height).
        let mut landed = false;
        for k in 0..400u32 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                let desired = (tx - b[0]).atan2(tz - b[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let mut sim_m = w.scene.sim.as_mut().unwrap();
                sim_m.step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false, false, false);
            }
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            if sim.bond.grounded && k > 40 && b[1] > 12.55 && b[1] < 13.65 {
                let dx = b[0] - ax;
                let dz = b[2] - az;
                let r = dx * ux + dz * uz;
                assert!(r > 5.5, "the walk-off must land past the rim on the strip, got r {r} at {b:?}");
                landed = true;
                break;
            }
        }
        assert!(landed, "the walk-off must land on the walkway strip (12.13 + the ~1.1 pos offset), at {:?}", bond_pos(&w));
    }
    eprintln!("[0d] the walk-off landed on the strip — at {:?}", bond_pos(&w));

    // The catwalk walk west: the strip merges into the east-west catwalk (12.13, z −265.0..
    // −266.25, x 157..196 [probe20 rays]; its south face is a wall at z −266.26 for the body
    // band y 12.6/13.25 [probe20 horiz rays] and its floor slit at x 162-163 is ~2 cm wide —
    // the capsule bridges it). Walked to the catwalk's west end.
    w.walk_to(Wp::new(184.0, 13.2, -265.7).r(1.2), "the catwalk west 1");
    w.walk_to(Wp::new(172.0, 13.2, -265.7).r(1.2), "the catwalk west 2");
    w.walk_to(Wp::new(160.0, 13.2, -265.7).r(1.2), "the catwalk west end");
    eprintln!("[0d] the catwalk walked west — at {:?}", bond_pos(&w));

    // THE REAL DESCENT (level-0d-last-two-standins: stand-in 1 DELETED): the catwalk's exit is
    // the LADDER at its west end [D placement type 0x22 ladder 2: centre (155.60, 14.25, −266.18),
    // r 4.39 — the only ladder east of x 120; the pick sphere spans y 9.86..18.64] climbing to the
    // ROOF (18.58 [probe0d roof map]), then the roof's east band (the z −268.5 lane is open all
    // x — the 20.19 rooftop structures start at z −269.4 [probe0d roof map + the lane's stuck
    // dump]), the building's east lane (x 196.3, strict L around the shed's NE corner at
    // (195.2, −269.4)), and EAST off the roof's east edge into the annex (9.42-9.45, x 197..202,
    // z −272..−290 [probe0d floor map]) — the bay (9.53) is CONTINUOUS with the annex at
    // z −276..−282 where the pad's east edge leaves x 196..197 open [probe0d floor map x 196:
    // 9.53, x 197: 9.45]. The grab: the walk off the catwalk's west edge with yaw −2.5 (west-
    // southwest) — the facing gate needs |wrap(ladder.yaw − player.yaw)| > π/2 [M Player_CollLadder
    // 0x196118: the strict 1.5707964 < |diff|; west-facing = exactly π/2 = rejected, measured]
    // and the fall-through crosses the ladder's pick box [M Ladder::capsule_hits_default →
    // pick_hit: the sphere gate r·1.5 + cap, then the mesh-box prefilter + the tri refine].
    // The climb runs to the ladder's top and lunges [M Player_Climb 0x196310: pos.y > top →
    // +0x18 += 0.6 dismount]; the dismount is airborne north of the roof's north edge (z −267.5)
    // — the eased yaw (ladder.yaw + π = south) carries Bond onto the roof on the hold [M the
    // yaw ease].
    w.lenient = true;
    // Walked onto the catwalk's west end and turned west-southwest with the stick (the former
    // pos/yaw placement — a 2.4 m hop plus a facing write — is gone).
    w.walk_to(Wp::new(157.6, 13.2, -265.6).r(0.35), "the catwalk's west edge");
    turn_to(&mut w, -2.5);
    w.wait(5);
    let mut grabbed = false;
    for _ in 0..400 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_CLIMB { grabbed = true; break; }
    }
    assert!(grabbed, "the ladder grab must fire off the catwalk's west edge (yaw −2.5) [M 0x196118]");
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if w.scene.sim.as_ref().unwrap().bond.substate != nf_game::player::SS_CLIMB { break; }
    }
    // The dismount carry south onto the roof.
    for _ in 0..80 {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(sim.bond.grounded && p[1] > 19.0 && p[0] > 153.0 && p[0] < 158.0,
            "the ladder must top out on the roof platform (18.58 + the pos offset), got {p:?} grounded {}", sim.bond.grounded);
    }
    // The roof's east band (open at z −268.5), the strict L around the shed's NE corner, the lane.
    let _ = w.walk_to(Wp::new(158.0, 19.65, -268.7).r(1.4), "roof east a");
    let _ = w.walk_to(Wp::new(172.0, 19.65, -268.7).r(1.4), "roof east b");
    let _ = w.walk_to(Wp::new(190.0, 19.65, -268.7).r(1.4), "roof east c");
    let _ = w.walk_to(Wp::new(196.3, 19.65, -269.0).r(0.4), "roof lane ne");
    let _ = w.walk_to(Wp::new(196.3, 19.65, -280.0).r(1.2), "roof lane south");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 195.0 && p[2] < -276.0 && p[2] > -282.0 && p[1] > 19.0,
            "the lane must reach the roof's east strip at z −278..−282, got {p:?}");
    }
    // East off the roof's east edge into the annex: a short push clears the edge (~x 197), then
    // the hold-input drop lands on the annex with minimal carry (a full walk's 9.1 m-fall carry
    // overshoots the annex east into the 3.42 pit [probe0d floor map]).
    turn_to(&mut w, std::f32::consts::FRAC_PI_2); // face east (the stick, not a yaw write)
    // Push off every ledge edge (the roof 18.58, an unscanned parapet at 19.44) until the
    // annex floor (9.42-9.45) catches him.
    for _cycle in 0..4 {
        if w.scene.sim.as_ref().unwrap().bond.pos[1] < 12.0 && w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        for _ in 0..40 {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
            if !w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        }
        for _ in 0..160 {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
            if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[0d] annex landed: ({:.2},{:.2},{:.2}) grounded {}", p[0], p[1], p[2], sim.bond.grounded);
        assert!(sim.bond.grounded && p[1] > 9.8 && p[1] < 11.2,
            "the drop must land on the annex (9.42-9.45 + the pos offset), got {p:?}");
    }
    // The bay guard #18 from INSIDE the bay (level-0d-real-health-crossfire-3): the guard
    // patrols the bay east-west ((207.26, −288.74) → (211.96, −288.73) measured) and every
    // ray from OUTSIDE (the ladder top, the trench) dies on the bay's north wall above its
    // low opening (the 300-600-tick fights landed nothing) — but from the bay-enter stand
    // (194, 10.6, −282) Bond is in the ROOM with it: a horizontal in-room ray at d ~17-21,
    // no wall between. Its ride hit (16) and its bay-phase fire both come off the table.
    let _ = w.try_walk_to(Wp::new(194.0, 10.6, -282.0).r(1.4), "bay enter");
    let bay18_killed = kill_guard_at_present(&mut w, [210.0, -288.8], [8.0, 5.0], "the bay guard #18 (from inside the bay)", 400);
    eprintln!("[0d] the in-bay #18 kill: landed {} pool {:.1}", bay18_killed, w.scene.sim.as_ref().unwrap().drones.health.health);
    // THE GROUND SPAWNER #32 from the bay/annex floor (level-0d-real-health-crossfire-4): the
    // strip-mouth kill was retired (its 528-tick dwell fed the climb's 86 — see the strip-mouth
    // comment); from the annex floor (y ~10.5, x ~194, z ~−282) the ray down to the band's
    // measured spots runs over open yard — the world-ray gate holds fire unless the scoped
    // bullet pick sees the chest line clear, so the leg costs nothing when blocked and takes
    // the band's east crossing when not. The box is the whole patrol band (the -3's measured
    // spots (157.59,−213.01) → (158.9,−229.2) → (163.3,−214.0)).
    let _ = kill_guard_at_present(&mut w, [159.5, -220.0], [6.5, 11.0], "the ground spawner #32 (from the annex floor)", 600);
    w.lenient = false;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(sim.bond.grounded && p[1] > 9.8 && p[1] < 11.2 && p[0] > 185.0 && p[0] < 197.0,
            "Bond must stand on the bay/annex floor (9.42-9.53 + the pos offset), got {p:?}");
    }
    eprintln!("[0d] the real descent walked: the ladder, the roof, the annex, the bay — at {:?}", bond_pos(&w));
    w.wait(20);
    // The dock E, real input: the control corner E-plate #460 `Overhead Crane_touchplate`
    // (189.02, 9.72, −288.92 [D]) — the proven E (interactables_0700000d_live
    // ::crane_e_plate_arms_the_handle_ss_and_the_shipping_sp): the E writes ch 0x2b [D #460 p0;
    // M Trigger_Activate 0x1b2318:23-33], which arms SP #115 `ScriptPlayer_ShippingCrane`
    // (mode 8, mask 1, out 0x2b, 3000 ticks [D]; the held state-6 arm releases on the edge
    // [M 0x1dbda8 state-6 `s.delay = 0`]) and re-sweeps the SS #112 handle.
    assert!(w.try_walk_to(Wp::at(189.0, 9.9, -288.7, Act::Use { use_ticks: 240 }), "E the crane touchplate #460"),
        "the E on #460 must complete");
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x2b),
        "the E fires the plate's out ch 0x2b [D #460 p0]");
    eprintln!("[0d] E #460 walked — the shipping SP #115 armed; the beam shuttles x 137.54 → 177.22 → 137.54 over 3000 ticks [D]");

    // THE BOARDING, real input. The mover is script 0x060009EF entity 1: a TWIN-GIRDER gantry
    // [D shape 0x0200_0a80 bounds (±4.17, −2.98..+2.98, ±17.01) local] whose origin sweeps from the
    // rest (137.54, 11.33, −287.11) east to ~177.22 (t ≈ 1230 of the SP's 3000 ticks) and back to
    // ~137.59 (t ≈ 2760), then re-sweeps [D the key trace]. Nothing walkable meets it, and its
    // script carries no player hand-off: the way on is from above. The pulley #370
    // (178.77, 16.32, −296.39) [D] hangs over the gantry's east park with a clear line from the
    // control corner, so the claw fired here reels Bond up [M Player_SetGrapplePoint 0x195da0,
    // Player_SSGrapple 0x192860], he hangs while the fire is held, and the release at the east
    // apex with the stick toward the track drops him on the east girder. The plate the gantry
    // carries (0x0200_0a81: 0xEA, out 103 = ch 0x67, mask 8, flat at +2.95 [D stream 2]) is the
    // game's own aboard detector, and ch 0x67 is a Bond-moment channel [M Trigger_Calc 0x1b1900:12-17].
    {
        let (handoffs, ents) = boarding_script_ops(&iso);
        assert!(!handoffs, "the gantry script must carry no player hand-off [D 0x060009EF's ops; M Script_EventHandler 0x1d94c0 cases 4/6]");
        let trig = ents.iter().find(|(h, _, _)| *h == 0x0200_0a81).expect("the riding 0x0200_0a81 [D stream 2]");
        assert_eq!(trig.1, 0xea, "the rider is the 0xEA trigger kind [D stream 2]");
        assert!(trig.2.len() >= 2 && trig.2[0] == 103 && trig.2[1] == 8,
            "the rider's out event 103 (ch 0x67) + mask 8 = the player walk-in detector [D stream 2 params], got {:?}", trig.2);
    }
    let claw_pulley = w.scene.sim.as_ref().unwrap().drones.grapple_points.iter()
        .find(|p| (p.pos[0] - 178.77).abs() < 0.5).expect("the pulley #370 [D]").pos;
    // The claw fires only inside a type-2 icon's arm [M Player_WeaponFiring 0x1a5460:515-528]:
    // the control corner is in none, so Bond steps onto the deck in icon #4's arm ((188.11, 8.46,
    // -299.78) r 5.0 [D icon census]), where the bolt still reaches the pulley [the walled-pick
    // stance scan: (185.25, floor 10.31, -294.75)].
    w.walk_to(Wp::new(185.3, 11.4, -294.8).r(1.2), "the deck in the pulley's icon arm");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.icon_ctx, 2, "Bond stands in a type-2 icon's arm [D]");
    let held_gun = w.scene.sim.as_ref().unwrap().gun.weapon;
    for _ in 0..20 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x67), "sister: nobody aboard before the boarding");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.world.segment_hit(sim.bond.head(), claw_pulley).is_none(), "the control corner sees the pulley #370 [D]");
    }
    // Bond waits on the floor until the gantry nears its east park: the hang is in the open.
    for _ in 0..2350u32 {
        let ox = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_09ef, 1)).map_or(0.0, |m| m[3][0]);
        if ox > 174.5 { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let mut latched = false;
    for _ in 0..60 {
        let sim = w.scene.sim.as_mut().unwrap();
        let h = sim.bond.head();
        let d = [claw_pulley[0] - h[0], claw_pulley[1] - h[1], claw_pulley[2] - h[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        w.tick_fire(nf_game::player::Input::default());
        if w.scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the claw latches the pulley #370 [M 0x195da0]");
    // Hang, the fire held, until the gantry is at its east apex under Bond.
    let mut max_ox = 0.0f32;
    for _ in 0..2350u32 {
        w.tick_fire(nf_game::player::Input::default());
        if let Some(m) = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_09ef, 1)) { max_ox = max_ox.max(m[3][0]); }
        if max_ox > 177.0 { break; }
    }
    assert!(max_ox > 176.5, "the gantry's east extreme ~177.22 [D the key trace], got {max_ox:.2}");
    { let sim = w.scene.sim.as_ref().unwrap(); assert!(sim.bond.pos[1] > 15.0, "Bond hangs at the pulley [M 0x192860:91-94], got {:?} grapple {} health {}", sim.bond.pos, sim.bond.grapple.is_some(), sim.drones.health.health); }
    { let sim = w.scene.sim.as_mut().unwrap(); sim.bond.yaw = -std::f32::consts::FRAC_PI_2; sim.bond.pitch = 0.0; } // the look and the stick toward the track, while hanging
    for k in 0..300u32 {
        let input = if k < 16 { nf_game::player::Input { forward: 1.0, ..Default::default() } } else { nf_game::player::Input::default() };
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] > 13.5, "Bond lands on the gantry, got {:?}", bond_pos(&w));
        assert!(sim.drones.channels.on(0x67), "the aboard plate fires ch 0x67 [D stream 2]");
    }
    w.wait(10);
    // THE RIDE, real input: the carry [M Player_FeetOnPoint 0x194188 → Control_InheritVelocity
    // 0x133e88, the port's ground_motion] holds Bond at a constant offset on the runway
    // through the west apex — probe_the_ride_carry's proven carry, now asserted.
    let mut off0 = None;
    let mut apex = None;
    let mut turned = false;
    for k in 0..2200u32 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        let Some(m) = sim.drones.scripts.entity_matrix((0x0600_09ef, 1)) else { continue };
        let ox = m[3][0];
        if ox < 176.0 { turned = true; }
        if sim.bond.grounded && turned && off0.is_none() {
            off0 = Some(sim.bond.pos[0] - ox);
        }
        if ox < 138.2 {
            let drift = off0.map(|o0| (sim.bond.pos[0] - ox) - o0);
            apex = Some((ox, drift));
            break;
        }
        let _ = k;
    }
    let (ox_apex, drift) = apex.expect("the beam must reach its west park (~137.59) with Bond aboard");
    assert!((137.3..=138.2).contains(&ox_apex), "the west apex ~137.59 [D the key trace], got {ox_apex:.2}");
    assert!((0.0..=3.0).contains(&off0.unwrap()), "Bond rides the east half of the gantry, got off {:.2}", off0.unwrap());
    assert!(drift.is_some_and(|d| d.abs() < 0.75), "the carry must hold the offset (the ease-out lag only) [M 0x194188 → 0x133e88], got {drift:?}");
    eprintln!("[0d] the ride walked: boarded at offset {:.2}, the apex at {ox_apex:.2} — at {:?}", off0.unwrap(), bond_pos(&w));
    // THE DROP, real input: the ride stands on the EAST girder; the slot between the girders
    // (x 136.5..138.6) is OPEN to the yard 4.0 [D the same probe: both ray directions empty;
    // the fine-map down-rays read 4.0], and the girder edges WALL the walk (measured: the held
    // walk input yields no displacement across an edge — the capsule's bottom sphere meets the
    // edge face below the top). The game's way off is the JUMP [M Player_HandleJump 0x1916b0 —
    // the vertical-only launch] + the walk's own air drift [M Player_Move runs every walk tick]:
    // jump at the park, hold west ~20 ticks (the smoother's ~1.7 m drift centres him in the
    // slot), release, fall ~9.5 m — survivable [M 0x1949a8:117-126: 9.1 m ≈ 24 damage]. The
    // jump is a PLAIN jump: the icon context re-arms from proximity every tick [M 0x1949a8:22
    // clears BL+0x95F; scene.rs's per-tick icon arm] and the wire icon #297 is ~75 m away.
    turn_to(&mut w, -std::f32::consts::FRAC_PI_2); // face west, into the slot (the stick)
    let mut landed = false;
    // One hop can land back on the girder: hop again until the slot takes him.
    'hops: for _ in 0..5 {
        for k in 0..300u32 {
            let input = if k == 0 {
                nf_game::player::Input { jump: true, ..Default::default() }
            } else if k <= 20 {
                nf_game::player::Input { forward: 1.0, ..Default::default() }
            } else {
                nf_game::player::Input::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.grounded && sim.bond.pos[1] < 7.0 { landed = true; break 'hops; }
            if sim.bond.grounded && k > 20 { break; }
        }
    }
    assert!(landed, "the jump-drift must drop Bond through the slot into the yard, got {:?}", bond_pos(&w));
    w.scene.sim.as_mut().unwrap().gun.switch_request = held_gun;
    w.wait(40);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.health.health > 40.0, "the ~10 m fall is survivable [M 0x1949a8], got {}", sim.drones.health.health);
    }
    eprintln!("[0d] the drop landed in the yard — at {:?}", bond_pos(&w));
    // THE YARD GARRISON SWEEP (level-0d-real-health-crossfire-5): the drop lands amid the
    // yard's standing grunts — the run-4 ledger: #6 (138.8, 5.0, −273.5) fired from d 7-21,
    // #3 (132.4, −300.1) from d 6.8, #0/#5 (158.9/159.9, −285.7/−294) from d 21-26 — 98 hits
    // for 86.4 of 0.6-1.2-per-hit trickle killed the walk at 16441 (death by accumulation; the
    // fights' 5 engagements came after the approach woke them). They sleep until approached
    // (the load census: enabled false [L]; bullets hit present drones regardless [M the AI gate
    // only decides whether the guard shoots BACK]) — the flat-yard rays from the drop point are
    // clear, so the grunts die ASLEEP before the south walk. Log-only.
    for (at, tol, label) in [
        ([138.8f32, -273.5], [4.0f32, 4.0], "the yard grunt #6 (the C5 door corner, asleep)"),
        ([132.4, -300.1], [4.0, 4.0], "the yard grunt #3 (the plate path, asleep)"),
        ([158.9, -285.7], [4.0, 4.0], "the yard grunt #0 (asleep)"),
        ([159.9, -294.0], [4.0, 4.0], "the yard grunt #5 (asleep)"),
    ] {
        let _ = kill_guard_at_present(&mut w, at, tol, label, 300);
    }
    // Leg 2: the yard south to the ST-3 plate (the nav web [D path 7]) — walked.
    w.walk_to(Wp::new(136.5, 4.5, -290.0).r(2.0), "the yard south 1");
    w.walk_to(Wp::new(136.0, 4.5, -308.5).r(1.2), "the plate");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x1d),
        "the ST-3 plate #378 → ch 0x1d (objective 4) [D]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[12][0], 4, "objective 4 completes [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[12][1], 3, "ch 0x1d reveals the photograph [L record 12 obj 1]");
    }

    // Leg 3: the laser photograph — the real bullet into the CamSubject volume (#494/#495 [D];
    // [M CamSubject_Update 0x1bc668:14-24]). The display sits in the yard's trench (the
    // walkthrough's spot (137, 5.1, −321) [D map-powerstation render list]) — walked.
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1e).expect("the laser subjects [D #494/#495]").clone()
    };
    let sc = [(subject.lo[0] + subject.hi[0]) / 2.0, (subject.lo[1] + subject.hi[1]) / 2.0, (subject.lo[2] + subject.hi[2]) / 2.0];
    w.walk_to(Wp::new(137.0, 5.2, -320.0).r(1.5), "the display trench");
    // The trench pair (level-0d-real-health-crossfire-5): #25/#26 sleep at (134.5/129.6,
    // −325.6) [census] — the run-4 ledger: #25 fired 16 hits for 10.2 across the trench/plate
    // phases. The camera switch below ends the gun legs, so the grunt dies first. Log-only.
    let _ = kill_guard_at_present(&mut w, [134.5, -325.7], [4.0, 4.0], "the trench grunt #25 (asleep, from the display trench)", 300);
    let _ = kill_guard_at_present(&mut w, [129.6, -325.6], [4.0, 4.0], "the trench grunt #26 (asleep, from the display trench)", 300);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.gun.switch_request = 0x54;
    }
    w.wait(40);
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x54, "the camera is held [M 0x1a7ef8]");
    let photographed = shoot_at(&mut w, sc, 40.0, 900, "the laser photograph",
        &|sim| sim.drones.channels.on(0x1e));
    assert!(photographed, "the photograph never wrote ch 0x1e [M 0x1bc668:14-24]");
    {
        // fix-objective-banner: the monitor completes the objective the tick after the channel.
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[12][1], 4, "objective 5 completes [M 0x1827d8:88-99]");
    }

    // Leg 4: north to the shooting stand — the yard face of bigdoor02 #96 (bounds
    // 131.78..137.08, 3.99..7.35, −269.85..−269.15 [D placed gfx]) seen from 11+ m out, outside
    // the death blast's reach (blast damage 1000 [D p5]; top_up_health guards anyway).
    w.walk_to(Wp::new(138.0, 4.5, -305.0).r(1.5), "back to the plate");
    w.walk_to(Wp::new(135.0, 4.5, -281.0).r(1.5), "the shooting stand");
    w.top_up_health = true;
    // The weapon: the level's own MOUNTED LASER 0x67 (the catwalk gun, GunImp #225 [D; the disc's
    // hint 0x4000066 "considerable firepower"]). The mounted class sustains fire with no ammo
    // pool [L probe: clip 0/spare 0 keeps firing, 50 damage a hit; M Destroy_CollisionHandler
    // 0x12bd48 subtracts every class-5 contact] — ~200 hits for the 10000 health. The carried
    // loadout cannot (0x1e: 35 × 40 = 1400 [D ammo caps]). The switch_request is the walkreal's
    // established gadget-select idiom (the camera leg above) [S port choice, not a game rule].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let wd = sim.weapons.clone();
        let rounds = wd.clip_size(0x67).max(1);
        let _ = sim.drones.inventory.equip_weapon(&wd, 0x67, rounds, true);
        sim.gun.switch_request = 0x67;
    }
    w.wait(30);
    let door = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .find(|d| d.placement == 96).cloned().unwrap();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d0 = door(&w);
        assert!(!d0.dead && d0.health > 9000.0, "bigdoor02 stands at ~10000 health [D p3], got {}", d0.health);
        assert!(sim.drones.channels.on(0x1e) && !sim.drones.channels.on(0x28),
            "the photograph input is latched; the corridor input comes after the kill [D #434 list]");
    }
    let target = [134.5, 5.6, -269.4];
    let killed = shoot_at(&mut w, target, 25.0, 30000, "the C5 kill",
        &|sim| sim.drones.channels.on(0x1f));
    assert!(killed, "shooting bigdoor02 (health 10000 [D p3]) must write ch 0x1f [M 0x12bd48:35]");
    {
        // fix-objective-banner: the monitor completes the objective the tick after the channel.
        let dead = door(&w).dead;
        assert!(dead, "the leaf is dead");
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[12][2], 4, "objective 6 completes [M 0x1827d8:88-99]");
    }

    // Leg 5: through the dead door into the corridor — the re-firing plate #433's low stretch
    // raises ch 0x28 while the crossing continues [D; M triggers.rs] — and with 0x1e + 0x1f
    // latched the AND #434 raises ch 100 [D #434 list; M Trigger_Update's AND arm].
    w.walk_to(Wp::new(136.0, 4.2, -278.0).r(2.0), "the yard north");
    w.walk_to(Wp::new(136.0, 5.2, -267.5).r(1.5), "the corridor slope");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x28), "the corridor plate #433 fires ch 0x28 [D]");
    // The mux → ch 100 → mission end → 0x730000d [M 0x182c60:102-103]. The end sequence runs
    // `Mission_Update`'s state-6 timeout (VIDEO_FRAME_RATE << 2 = 240 ticks [M :88-91]) before
    // `LevelEnd` → `ResetMap_LevelToLoad` [M :65-76] — wait it out.
    let mut ended = false;
    for _ in 0..120 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(100) { ended = true; break; }
    }
    assert!(ended, "mux #434 [0x1e & 0x1f & 0x28] must raise ch 100 [D; M Trigger_Update's AND arm]");
    let mut exit = None;
    for _ in 0..400 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.mission.status() == 6 { exit = sim.drones.level_exit.or(sim.drones.mission_end); if exit.is_some() { break; } }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.mission.status(), 6, "Mission_Update state 6 [M 0x182c60]");
    assert_eq!(exit, Some(0x0730_000d), "the level hands off to the driving level [D record; M 0x182c60:102-103], got {exit:?}");
    // EVERY objective the level's mission data lists, complete [L record 12; M 0x1827d8:88-99]:
    // objective 3 (record 11, the TR-2 advance — completed at the start trigger), 4 (ST-3),
    // 5 (the photograph), 6 (the C5 kill). State 4 = COMPLETE [M mission.rs states].
    assert_eq!(sim.drones.mission.states[11][2], 4, "objective 3 complete");
    assert_eq!(sim.drones.mission.states[12][0], 4, "objective 4 complete");
    assert_eq!(sim.drones.mission.states[12][1], 4, "objective 5 complete");
    assert_eq!(sim.drones.mission.states[12][2], 4, "objective 6 complete");
    eprintln!("[0d] the full mission walked + completed in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

/// THE ARRIVAL WALK ON REAL HEALTH (`damage_log.rs::with_real_damage`): the same walked
/// route, the top-up off — no health, armour or ammo writes anywhere — and Bond's death
/// panics with the damage log (who, weapon, range). Auto-aim is the app default; the fights
/// are the walker's own plain return fire (the same fights every other real-damage walk is
/// pinned against).
/// UN-IGNORED (level-0d-real-health-crossfire-3): the two measured drains are taken off the
/// table by the route, not by the port — #10 dies at the strip mouth (the ray CLEAR at d 63.6
/// [probe_0d_ray_stands]) before the ladder-top dwell that fed its 46, the deck guard #16 dies
/// at the warehouse floor (ray CLEAR d 56.4) so the ladder-top stand carries only the west-roof
/// sweep (extended to 8 passes against the placement rays the probe measured CLEAR for #27/#29),
/// and #18 dies from the ladder top (ray CLEAR d 56.1) before its ride hit. Real input only:
/// every kill fires from a stand the walked route reaches, no health/armour/ammo writes, no
/// teleports after the load.
#[test]
#[ignore = "the real boarding hangs and rides in the open: the measured run reaches the west park at 13.2 health (tick 13080, 12 hits for 80.4: #29 3 x 15.3 from the roof at d 52, #15 and #32's sniper rounds at d 26-70), and the ~10 m drop kills him; the walk's kill list was laid for the old placed boarding, the snipers need their own stands before the ride"]
fn walk_arrival_to_the_c5_door_real_damage() {
    walker::with_real_damage(walk_arrival_to_the_c5_door);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-0c-0d): the same walked
/// route and the same end, loaded at difficulty 2 the menu way — the front-end's launch
/// path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty; GameState+0x28 →
/// Drones::difficulty, asserted at load]. Agent spawns the min-difficulty-2 placements
/// [M Drone_Create 0x136be0:49], scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain
/// 0x1902c8] and the guards' hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy
/// 0x179740:74-88]. Real input only; no health, ammo or channel writes.
///
/// IGNORED with the measured run (real-health-agent-0c-0d): the walk rides the same
/// Operative-shaped death the `..._real_damage` ignore audits — the deck arrival — and the
/// Agent deltas are measured on top: DIED tick 4850 (202.25,16.14,-255.06) with 0
/// return-fire fights, 6 hits for 100.0 (armour spent on the way): #21 (skin 0x5000087,
/// sniper 0x1e, (162.8,1.0,-215.0)) landed 2 × 28.0 from d 58.3 and #36 (0x1e,
/// (157.7,1.0,-228.1)) 1 × 10.0 from d 54.2 — both OUTSIDE the fights' 48 u lane window
/// and outside every pinned kill box, so no fight ever engaged them — plus #20's one 28.0
/// at d 7.1 on the deck. The Operative audit's own drains (#10 the bay guard, #29 the west
/// roof — unsatisfiable from every stand the route reaches, level-0d-real-health-crossfire-2)
/// sit under the same crossfire. NEXT LEVERS: the 50-level-0d-real-health-crossfire-5 lever
/// set first (this sister must build on a green Operative base); then a d2-gated aimed stand
/// on the two d2 snipers from the strip (the 0c yard-stand law — the zero-spread aimed act
/// [M Bullet_init 0x12c5a0:139-170] reaches d 55+ with the still-gate tap law).
#[test]
#[ignore = "Agent (dart-tase-stamps-check measured state): on the -5 route the death moved 4850 -> 8533 — the decisive drain is the d2 ground-band snipers, #36 (0x5000090, 0x1e) 3 × 28.0 from d 56.6 (7932/8112/8479) + #21 (0x5000087, 0x1e) the killing 10.0, both outside the fights' 48 u window. This session's d2-gated ladder-top set LANDED 3 kills (the south-roof sniper #17 first at +559 — its fire spot (192.3,19.6,-267.4) is d 39, inside the shot's hearing radius 40 [L], so it MUST die before the band guns; the band chest sweep g36 at +201, ray CLEAR d 55.9) and pushed the death to 10697; the last killer g21 settles at (162.8,1.0,-215.0) behind the ~1.7 jamb at (160,-217) — the chest ray dies 0.965-0.975 from every stand measured (the strip mouth 0.975, the ground drop 0.605-0.965, the ladder top 0.965), the HEAD line reads full CLEAR (the -5 height-asymmetry law confirmed) but the box/head act's aim does not CONVERGE to it (the cross sticks [-0.21,-0.28], the chest legs converge [-0.0,0.0] from the same stand). NEXT: fix the head act's pitch convergence, or the stand east of the jamb (the -4 obstacle law — the late annex-floor #32 leg's geometry, too late for the ride+deck)"]
fn walk_arrival_to_the_c5_door_real_damage_agent() {
    walker::with_real_damage(|| walk_arrival_to_the_c5_door_at(2));
}

/// Shoot a live guard standing near (x, z) through the real fire path (the walker's aim +
/// trigger taps; the same fights every real-damage walk is pinned against). `tol` is the
/// (x, z) search box; the y is not tested — the yard's shooters sit on distinct (x, z).
/// False (and a log line) when no such guard lives — the caller treats that as "already dead".
fn kill_guard_at(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str) -> bool {
    kill_guard_at_cap(w, at, tol, label, 600)
}

/// The PRESENT variant (level-0d-real-health-crossfire-3): drops the `enabled` gate — the bay
/// guards #10/#18 are placement-held (`enabled false, present true` at load [census]) and the
/// platform-drop wake's timing vs the strip legs left #10 `enabled false` in the first run, so
/// the enabled-gated box found nothing and the whole point of the leg (killing #10 BEFORE the
/// ladder) was lost. Bullets hit present drones regardless of the AI gate; the AI gate only
/// decides whether the guard shoots BACK.
fn kill_guard_at_present(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str, cap: u32) -> bool {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.corpse || d.health <= 0.0 { continue; }
            if (d.pos[0] - at[0]).abs() > 14.0 || (d.pos[2] - at[1]).abs() > 14.0 { continue; }
            eprintln!("[0d:box] {label}: g{i} skin {:#x} pos ({:.2},{:.2},{:.2}) enabled {} present {} health {:.0} state {:#x}",
                d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.health, d.state);
        }
    }
    let di = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0
            && (d.pos[0] - at[0]).abs() < tol[0] && (d.pos[2] - at[1]).abs() < tol[1])
        .min_by_key(|(_, d)| ((d.pos[0] - at[0]).powi(2) + (d.pos[2] - at[1]).powi(2)) as u32)
        .map(|(i, _)| i);
    match di {
        Some(i) => shoot_far(w, i, label, cap),
        None => { eprintln!("[0d] {label}: no PRESENT guard near ({:.1},{:.1}) — already down", at[0], at[1]); false }
    }
}

/// The HEAD variant (level-0d-real-health-crossfire-5): the same present-pick, but the leg
/// fires the HEAD act — the slab-edge law is height-asymmetric (the settled/spawn spots' CHEST
/// rays die at 0.80-0.974 on the rooftop obstruction while the HEAD rays clear it from the same
/// stands [probe_0d_roof_rays]) and one head hit drops a guard [M NDrone2_HitDamage 0x145558 ×
/// DroneDamage_Head, damage.rs part 5].
fn kill_guard_head_present(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str, cap: u32) -> bool {
    let di = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0
            && (d.pos[0] - at[0]).abs() < tol[0] && (d.pos[2] - at[1]).abs() < tol[1])
        .min_by_key(|(_, d)| ((d.pos[0] - at[0]).powi(2) + (d.pos[2] - at[1]).powi(2)) as u32)
        .map(|(i, _)| i);
    match di {
        Some(i) => {
            let d = &w.scene.sim.as_ref().unwrap().drones.list[i];
            eprintln!("[0d:box] {label}: g{i} skin {:#x} pos ({:.2},{:.2},{:.2}) state {:#x}",
                d.skin, d.pos[0], d.pos[1], d.pos[2], d.state);
            shoot_far_box(w, i, label, cap)
        }
        None => { eprintln!("[0d] {label}: no PRESENT guard near ({:.1},{:.1}) — already down", at[0], at[1]); false }
    }
}

/// The CLEAR BOX aim (level-0d-real-health-crossfire-5): the guard's own hit boxes, ray-tested
/// from the eye exactly like damage_log::exposed_impl (ray_box intersect + the scoped bullet
/// pick to the centre) — the part-5 head box wins, then anything else that clears. Returns the
/// aim point and the part picked (for the log). This is the shot's own law: a point it returns
/// is on a box the bullet can reach from HERE.
fn clear_box_aim(sim: &nf_view::scene::Sim, i: usize) -> Option<([f32; 3], i16)> {
    let d = &sim.drones.list[i];
    let bp = sim.bond.pos;
    let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let mut head: Option<([f32; 3], i16)> = None;
    let mut other: Option<([f32; 3], i16)> = None;
    for b in d.hit_boxes(&data) {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        if nf_game::damage::ray_box(eye, ray, &b).is_none() { continue; }
        if sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, c).is_some() { continue; }
        if b.part == 5 { head = Some((c, b.part)); } else { other = other.or(Some((c, b.part))); }
    }
    head.or(other)
}

/// The cap variant: a landed kill costs ~200 ticks (the aimed first shot); a FAILED ray burns
/// the whole cap while Bond STANDS in the crossfire — the optional kills ride 260 so a failed
/// ray stops early (level-0d-real-health-crossfire: the 600-tick failures at the ladder top
/// were themselves the exposure that killed the twin).
fn kill_guard_at_cap(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str, cap: u32) -> bool {
    // The CLOSEST live guard to the box centre — the list-order pick kept killing the same
    // lower-index box-mate (#27) while the aimed-at guard (#28/#29) lived and ate the ride
    // (level-0d-real-health-crossfire).
    let di = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.enabled && d.present && !d.corpse && d.health > 0.0
            && (d.pos[0] - at[0]).abs() < tol[0] && (d.pos[2] - at[1]).abs() < tol[1])
        .min_by_key(|(_, d)| ((d.pos[0] - at[0]).powi(2) + (d.pos[2] - at[1]).powi(2)) as u32)
        .map(|(i, _)| i);
    match di {
        Some(i) => shoot_far(w, i, label, cap),
        None => { eprintln!("[0d] {label}: no live guard near ({:.1},{:.1}) — already down", at[0], at[1]); false }
    }
}

/// The long-range takedown (local, "kept local to keep walker.rs additive" — the file's own
/// `shoot_at` precedent): the walker's `shoot_gated` aim gates (±0.02 rad yaw) are WIDER than a
/// guard's ~0.3 u torso box beyond ~15 u (0.02 rad ≈ 0.66 u at 33 u — every tap sprays wide and
/// a 900-tick fight never lands a round), so this variant scales the gates to the range
/// (±0.25 u lateral, ±0.6 u vertical — inside the box at any distance) and holds the STAND
/// (no strafe repositioning: the route's stand is the probe-cleared ray; sliding off it walks
/// Bond out of his own cover — measured: the first kill attempt slid 5 u north and died).
/// The fire is AIMED (`aiming: true` — the platform-kill measurements: the held 0x1e is a
/// 0x400000 record [L ee_start_05.bin weapon_data 0x1e +0x68 = 0x408050] whose hip-fire keeps
/// the full 100-unit cone [M Bullet_init 0x12c5a0:139-170's player_spread — exact ONLY with the
/// aim bit] and misses a 0.3 u torso beyond ~5 u — 5 dead-centre hip rounds at d 28-33 landed
/// nothing, while the aimed `shoot_at` path lands static kills at d 25 (the C5 door)). The
/// real fire path end to end (the gun, the bullets, the hit boxes), reload pulses on empty.
fn shoot_far(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, 0.3)
}

/// The HEAD aim variant (level-0d-real-health-crossfire-5): the same aimed act with the aim
/// point and the world-ray gate raised to the head box (pos + `aim_dy`) — the height-asymmetric
/// slab law makes head lines clear where chest lines die [probe_0d_roof_rays].
fn shoot_far_head(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, 0.9)
}

/// The BOX aim variant (level-0d-real-health-crossfire-5): the aim point is a REAL clear hit-box
/// centre each tick (the part-5 head first — `clear_box_aim`), not a fixed offset — the run-2
/// +0.9 head guess fired clear-ray rounds that passed under the head box; a box point the
/// port's own law returns is on a box the bullet reaches.
fn shoot_far_box(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, -1.0)
}

fn shoot_far_aim(w: &mut Walker, di: usize, label: &str, cap: u32, aim_dy: f32) -> bool {
    let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
    let mut last_tap = 0u32;
    let start = w.total_ticks;
    // The aim-on EDGE: a button held across the intro/legs consumed the edge (prev_aiming stuck
    // true with the gated bit never set — measured in probe_0d_bullet's gate trace), and the
    // raise queue REFUSES while a fire script plays (state 9 [M 0x1a24c8:66-99] — the previous
    // fight's 130-tick script), so release long enough to drain it, then hold until the bit
    // actually sets (the raise completes ~20 ticks [M Player_SetWeaponAnimObj 0x1a2e38 case 0xf]).
    for _ in 0..160 {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    for _ in 0..90 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.aiming { break; }
        sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    loop {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0 {
                eprintln!("[0d] {label}: down at +{} ticks", w.total_ticks - start);
                return true;
            }
            let (bp, dp) = (sim.bond.pos, sim.drones.list[di].pos);
            let dist = walker::dxz(bp, dp).max(0.5);
            // THE RAY, exactly the shot's own construction (scene.rs's crosshair law [M
            // Player_GetAimingPoint 0x1a4a98 — the (x, y) cursor offsets ride the view ray]):
            // with the aim bit set, free_aim is true and the stick drives the CURSOR window —
            // the tap gate must measure the RAY (look + 0.76966·cross[0]·right +
            // 0.57724·cross[1]·up), not bond.pitch: the calibration (probe_0d_bullet) measured
            // a ±0.2309 rad ray offset at the cursor's ±0.4 clamp while bond.pitch read
            // converged.
            let yaw = sim.bond.yaw;
            let p = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let look = sim.bond.look();
            let right = [-yaw.cos(), 0.0, yaw.sin()];
            let up = [-p.sin() * yaw.sin(), p.cos(), -p.sin() * yaw.cos()];
            let (tx, ty) = (0.76966 * sim.cross[0], 0.57724 * sim.cross[1]);
            let d = [look[0] + tx * right[0] + ty * up[0], look[1] + tx * right[1] + ty * up[1], look[2] + tx * right[2] + ty * up[2]];
            let dl = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let ray = [d[0] / dl, d[1] / dl, d[2] / dl];
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            // The BOX mode (aim_dy < 0): the aim point is a REAL clear hit-box centre this tick
            // (the part-5 head first — clear_box_aim); no clear box → hold fire. The fixed-dy
            // modes aim at the pos offset as before.
            let (aim_pt, mut aim_part) = if aim_dy < 0.0 {
                match clear_box_aim(sim, di) {
                    Some((c, part)) => (c, Some(part)),
                    None => ([dp[0], dp[1] + 0.3, dp[2]], None),
                }
            } else {
                ([dp[0], dp[1] + aim_dy, dp[2]], None)
            };
            let (dx, dy, dz) = (dp[0] - eye[0], aim_pt[1] - eye[1], dp[2] - eye[2]);
            let err = walker::wrap_pi(dx.atan2(dz) - ray[0].atan2(ray[2]));
            let perr = dy.atan2((dx * dx + dz * dz).sqrt().max(0.5)) - ray[1].asin();
            // The BOX mode tightens the tap box to the body part's own scale (a torso box is
            // ~0.3 u; the fixed-dy chest aim keeps its proven 0.25/0.6 u law).
            let (lat, vert) = if aim_dy < 0.0 { (0.12, 0.25) } else { (0.25, 0.6) };
            let err_tol = (lat / dist).clamp(0.003, 0.02);
            let perr_tol = (vert / dist).clamp(0.006, 0.05);
            let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
            let t = w.total_ticks - start;
            let on_target = err.abs() < err_tol && perr.abs() < perr_tol && t.saturating_sub(last_tap) > 15;
            // THE WORLD-RAY GATE (level-0d-real-health-crossfire-4): the -3's #32 patrol-band
            // leg fired 2 rounds into a ray the probe itself printed BLOCKED (full 0.960 — the
            // ~1.7-high ground obstacle at (160,−217) [level-0d-real-health-crossfire-3]) and
            // the west-roof sweep's pass 3 ran at clip 1 spare 25 with a third walker alive —
            // the dry sweep is what fed the ladder-top death (10 hits for 100 at 4482). The
            // gate fires only when the SHOT's own law (the scoped bullet pick, the same query
            // the flight walks [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058]) sees the
            // chest line CLEAR: a blocked ray holds fire (the guard may cross to a clear spot
            // — #32's band runs east to (163.3,−214) where the obstacle at x ~160 is behind
            // the target) and the cap bounds the wait. Saves the rounds; costs nothing when
            // the ray is already clear (every landed kill's ray printed CLEAR).
            let world_clear = aim_part.is_some() || aim_dy >= 0.0;
            let world_clear = world_clear && sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, aim_pt).is_none();
            let trigger = !clip_empty && on_target && world_clear;
            if trigger { last_tap = t; }
            // The stick drives the RAY ~1:1 (turn: the cursor's angular response is
            // 0.76966·(T/0.76966) = T; look: 0.57724·(L·π/2/0.57724) = L·π/2 — the overflow
            // beyond the ±0.4 window carries the view the same direction), so the same
            // convergence as the plain walker works on the ray.
            let input = nf_game::player::Input {
                turn: (err * 1.0).clamp(-0.08, 0.08),
                look: (perr * 2.0 / std::f32::consts::PI).clamp(-0.08, 0.08),
                aiming: true,
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, clip_empty && t % 20 == 0, false);
        }
        w.total_ticks += 1;
        w.after_step();
        let sim = w.scene.sim.as_ref().unwrap();
        if w.total_ticks % 120 == 0 {
            let (bp, dp) = (sim.bond.pos, sim.drones.list[di].pos);
            let dist = walker::dxz(bp, dp).max(0.5);
            eprintln!("[0d:dbg] {label} +{} bond ({:.1},{:.1},{:.1}) tgt ({:.1},{:.1},{:.1}) d {:.1} pitch {:.3} cross {:?} clip {} spare {} aiming {}",
                w.total_ticks - start, bp[0], bp[1], bp[2], dp[0], dp[1], dp[2], dist, sim.bond.pitch, sim.cross, sim.gun.clip, sim.gun.spare, sim.aiming);
            // level-0d-real-health-crossfire-2 audit: the same eye→chest ray the round flies,
            // through the world probe AND the bullet-law scoped pick — WHERE does a failed ray die?
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let aim = if aim_dy < 0.0 { clear_box_aim(sim, di).map(|(c, _)| c).unwrap_or([dp[0], dp[1] + 0.3, dp[2]]) } else { [dp[0], dp[1] + aim_dy, dp[2]] };
            let full = sim.world.segment_hit(eye, aim);
            let stat = sim.world.segment_hit_static(eye, aim);
            let bullet = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, aim);
            eprintln!("[0d:ray] {label} eye ({:.1},{:.1},{:.1}) chest ({:.1},{:.1},{:.1}) full {} static {} bullet_scoped {}",
                eye[0], eye[1], eye[2], aim[0], aim[1], aim[2],
                full.map(|(f, p)| format!("{f:.3}@({:.1},{:.1},{:.1})", p[0], p[1], p[2])).unwrap_or_else(|| "CLEAR".into()),
                stat.map(|(f, p)| format!("{f:.3}@y{:.1}", p[1])).unwrap_or_else(|| "clear".into()).as_str(),
                bullet.map(|f| format!("{f:.3}")).unwrap_or_else(|| "CLEAR".into()));
        }
        if sim.gun.clip == 0 && sim.gun.spare == 0 && w.total_ticks - start > 60 {
            eprintln!("[0d] {label}: out of ammo at +{}", w.total_ticks - start);
            return false;
        }
        if sim.drones.list[di].health < hp0 { hp0 = sim.drones.list[di].health; }
        if w.total_ticks - start >= cap {
            eprintln!("[0d] {label}: still alive after {cap} ticks (health {:.0}, dist {:.1})",
                sim.drones.list[di].health, walker::dxz(sim.bond.pos, sim.drones.list[di].pos));
            return false;
        }
    }
}

/// Turn Bond to `yaw` with the stick alone (`Input::turn` adds to the yaw each tick [port
/// player.rs; the walker's own clamp]), capped at 200 ticks.
fn turn_to(w: &mut Walker, yaw: f32) {
    for _ in 0..200 {
        let err = walker::wrap_pi(yaw - w.scene.sim.as_ref().unwrap().bond.yaw);
        if err.abs() < 0.01 { return; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    panic!("the stick turn never reached yaw {yaw}");
}
/// [D] 0700000d.bin's script 0x060009EF, decoded: (any player hand-off event 4/6, the entity
/// ops as (hash, type, params)). The sister data for the boarding question: the mover script's
/// nine streams carry no Player_Disable/Player_Enable, and stream 2 spawns the riding 0xEA
/// trigger 0x0200_0a81 with params [103, 8] — the game's own aboard detector.
fn boarding_script_ops(iso: &str) -> (bool, Vec<(u32, u8, Vec<u32>)>) {
    use nf_formats::iso9660::IsoImage;
    use nf_formats::levelbin::LevelBin;
    use nf_formats::nis::{Cmd, Nis};
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = LevelBin::parse(&data).expect("levelbin");
    let mut handoffs = false;
    let mut ents = Vec::new();
    for se in lb.entries.iter().filter(|e| e.kind == 7) {
        let Ok(n) = Nis::parse(se.hash(), lb.data(&data, se)) else { continue };
        if n.hashcode != 0x0600_09ef { continue; }
        for s in &n.streams {
            for c in s.decode() {
                match c {
                    Cmd::Event { kind, .. } if kind == 4 || kind == 6 => handoffs = true,
                    Cmd::Entity { hash, obj_type, params, .. } => ents.push((hash, obj_type, params)),
                    _ => {}
                }
            }
        }
    }
    (handoffs, ents)
}

/// Fire the held weapon at a static point through the real trigger path (the walker's aim loop;
/// kept local to keep walker.rs additive).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, _label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
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
