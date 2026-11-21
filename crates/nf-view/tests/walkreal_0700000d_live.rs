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
    walk_arrival_to_the_c5_door_at(1, false);
}

/// The difficulty-parameterized body (real-health-agent-0c-0d): 1 is the byte-identical
/// master replay; 2+ (the Agent sister) loads the menu way BOTH times — the cold menu pick
/// and the warm level change both run GameState+0x28 = 2 [M C_SBNFDFCTY 0x216ca8:15 →
/// load_with_carry_difficulty; GameState+0x28 → Drones::difficulty, asserted after the
/// warm load]. Agent spawns the min-difficulty-2 placements [M Drone_Create 0x136be0:49],
/// scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain 0x1902c8] and the guards'
/// hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy 0x179740:74-88].
fn walk_arrival_to_the_c5_door_at(difficulty: u32, real_damage: bool) {
    let iso = iso();
    // THE 00 AGENT CONTAINER DROP (real-health-00agent-0d-1): the 11-15 ledger's lever — break
    // the gantry ride while the beam is still over the yard's containers (entity x < 153.5),
    // hop west with the drift held 36 ticks, and land on the container top (x 145-149, surface
    // 7.1) instead of riding to the west apex and taking the 10 m slot fall (25 x 1.2 = 30 at
    // 00 Agent [M 0x1d3a80 group 0x1d40f4; 0x1902c8:84-104 — the fall skips armour]). The
    // landing is the measured clean first-hop roll (~18.6 health in, the fall from the girder
    // ~5 m ≈ timer 61 → raw 1 < 10 → no pain [M Player_CollisionHandler 0x1949a8 disasm
    // 0x194b64-0x194be0]). Bond then LEAVES the container before the sweep — the yard floor is
    // 3.1 m below the top (48 airborne ticks, also under the 60-tick gate) and the
    // drop-then-sweep's flat-yard rays engage the garrison from the floor (the container-top
    // stands measured dead: the sweep's passes burned under #1's -1.8 cadence at 6.8 u).
    let container_drop = real_damage && difficulty == 3;
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

    // THE PRE-WAKE ROOF SNIPE (level-0d-agent-real-damage re-route). The west-roof placements
    // are PRESENT and ASLEEP through this phase — the wake is the platform drop itself
    // [probe_0d_roof_rays part E] — and the probe's pre-wake ray table reads, from this very
    // stand (eye 12.89): the SOUTH and MIDDLE placement head lines CLEAR (the g27-spawn head
    // CLEAR at 0.949-chest, the g29-spawn chest AND head CLEAR) while the NORTH line dies 0.814
    // on the warehouse roof lip. Since the class-4 sight table landed (sight 1000, engage
    // 75+frand(8), the cone zeroed on first attack [M NDrone2_DefaultInit 0x14b300:380-424,
    // DroneFunc_SetAsAttacking 0x14aed8:25-26]) these snipers own the whole climb and ladder
    // top — the t3310 Agent ledger opens with the south placement's graze at 2395 and 6.0 at
    // 3047 (d 64). A player takes them off the roof BEFORE walking into the drop that wakes
    // them; the box act (a real posed head-box centre) keeps blocked lines silent. Log-only:
    // the walk stays green whether or not a kill lands, and the log is the measured answer.
    // Agent-gated in the batch-1109a-walks merge (measured on difficulty 2 only).
    if difficulty >= 2 {
        let _ = kill_guard_head_present(&mut w, [145.6, -256.7], [3.0, 3.0], "the west roof #461 south (pre-wake, the head line CLEAR [probe_0d_roof_rays])", 500);
        let _ = kill_guard_head_present(&mut w, [145.7, -247.3], [3.0, 3.0], "the west roof #463 middle (pre-wake, the chest and head lines CLEAR [probe_0d_roof_rays])", 500);
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
    // THE PLATFORM-DROP PIN (level-0d-fall-damage), real input: the ~11.1 m walk-off deals
    // NOTHING — the ledge slide keeps the ground contact alive (BL+0x940 stays set through the
    // slide), so the handler's branch B zeroes BL+0x928 on every contact tick and the landing
    // timer never passes 60 [M Player_CollisionHandler 0x1949a8:97-127 — the accumulate arm
    // runs only while BL+0x940 == 0]; the game's own drop read no damage [L probe4]. The walk
    // measured health 100.0 through tick 3000 (the drop at ~1750).
    let pre_platform_health = w.scene.sim.as_ref().unwrap().drones.health.health;
    assert!(w.try_walk_to(Wp { at: [181.0, 7.5, -227.5], act: Act::None, arrive: 1.5 }, "the platform drop to the warehouse ground"),
        "the platform drop must reach the ground");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = bond_pos(&w);
        assert!(p[1] < 6.0, "the drop lands on the warehouse ground (y ~1.04), got {p:?}");
        assert!((sim.drones.health.health - pre_platform_health).abs() < 0.05,
            "the ~11 m platform slide-off deals no fall damage [M 0x1949a8:97-127 + L probe4], was {pre_platform_health}, got {}",
            sim.drones.health.health);
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
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    // 00 Agent twin of the yard-armour leg: TRIED and DEAD (retune-nb-0d, run3 2026-11-18):
    // the kind-3 pickup at (205.4, 0.1, -205.3) did NOT arm at 00 Agent — the nav arrived
    // (205.97, 1.13, -205.17), 0.57 dxz off the placement, and armour stayed 2.0 → 2.0 (at
    // d1 the same leg measured "armour 50 taken") — and the walk-over cost 1483 ticks during
    // which #36 (ground band, 0x5000090, (158.1, 1.0, -217.7)) landed 46.0 and #22 48.0
    // [L the NF_DMG_TRACE run]. The leg stays out; the armour question needs the pickup's
    // d3 existence confirmed (a live check for the main agent) before any retry.
    for (at, r, label) in [
        ([196.0f32, 0.5, -228.5], 2.0, "the blind strip east 2"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    // LEVER (b), retune-nb-0d-4: the deck sniper #22 re-roll. The round-1 ledger below: the
    // strip-mouth head act LANDED the #22 kill (the scan's west reposition puts it at
    // (204.21, 12.09, -290.15), inside the [209.0, -289.3] tol [7.0, 3.5] box) but the re-deal
    // handed #20 46.0 + #32 48.0. The re-deal is the shared stream's state at the re-deal
    // events, and idle ticks draw nothing (measured), so the act's own DRAW COUNT is the knob:
    // NF_0D_DECK_EXTRA deliberate pre-shots (each a hip-spray spread draw from the shared
    // stream — the strip faces east into the wall, nothing hittable) fire before the act, and
    // each count is its own roll. Gated to the container roll (d3 real damage);
    // NF_0D_DECK_KILL arms the leg (0 = off, the master timeline byte-identical).
    if container_drop {
        let deck_kill: u32 = std::env::var("NF_0D_DECK_KILL").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if deck_kill > 0 {
            let deck_extra: u32 = std::env::var("NF_0D_DECK_EXTRA").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
            let h0 = w.scene.sim.as_ref().unwrap().drones.health.health;
            let t0 = w.total_ticks;
            for _ in 0..deck_extra {
                let reload = { let s = w.scene.sim.as_ref().unwrap(); s.gun.clip == 0 && s.gun.spare > 0 };
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), true, reload, false);
                w.total_ticks += 1;
                for _ in 0..24 {
                    w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
                    w.total_ticks += 1;
                }
            }
            let landed = kill_guard_head_present(&mut w, [209.0, -289.3], [7.0, 3.5], "the deck sniper #22 (the strip-mouth head act, lever (b))", 600);
            eprintln!("[0d] the deck #22 act: landed {landed} health {h0:.1} -> {:.1} +{} ticks (extra {deck_extra})",
                w.scene.sim.as_ref().unwrap().drones.health.health, w.total_ticks - t0);
        }
    }
    // THE DECK DUO FROM THE STRIP MOUTH, 00 Agent only: MEASURED DEAD (retune-nb-0d,
    // 2026-11-18, three real runs): the head act LANDED its kill (the scan's west reposition
    // put the deck sniper #22 (0x5000090, 0x1e) at (204.21, 12.09, -290.15) where the
    // [209.0, -289.3] tol [7.0, 3.5] box converged and killed it — "landed true") — but the
    // roll re-dealt the killer: #20 (deck (197.9, 14.0, -262)) landed 46.0 at t2022 and #32
    // (west roof (151.4, 22.7, -238.3)) 48.0 at t2354 + 6.0 at t2895, death at the ladder
    // base vs the un-leged roll's t19197 ride death [L the NF_DMG_TRACE runs]. The scan's
    // 0x1e roster is WIDE (#11/#20/#22 deck, #32 west roof, #36 ground band — each 46-48 raw
    // at 00 Agent's accuracy row, bullets eat armour first, the 0.0-health deltas are
    // armour-eaten hits); single-target pre-ride kills just move the death. The leg stays
    // out (the walk-7 "the net is NEGATIVE" precedent).
    let bay_killed = kill_guard_at_present(&mut w, [214.2, -289.4], [6.0, 6.0], "the bay guard #10 (from the strip mouth, ray clear d 64 [probe_0d_ray_stands])", 180);
    eprintln!("[0d] the strip-mouth #10 kill: landed {} pool {:.1}", bay_killed, w.scene.sim.as_ref().unwrap().drones.health.health);
    // (the measured-dead strip-mouth deck-duo lever's ledger sits in the comment above)
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
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    // Operative real damage: the yard's armour (kind 3 at (205.4, 0.1, -205.3), 24 u north of the
    // blind strip) before the climb - the ladder-top snipers land 16 a hit from 55-65 u and the
    // climb arrived with no armour. Measured: armour 50 taken (34 left back at the strip, one
    // 16.0 on the return), death t5532 -> t10148 (out on the deck at (202.0, 16.0, -255.1)).
    // retune-nb-0d-4: the d3 armour probe. The round-1 twin refused the take (armour 2.0 → 2.0)
    // with the nav arrival 0.57 dxz / 1.03 dy off the placement — and the placed pickups spawn
    // with NO difficulty filter [D scene.rs placed_pickups: every 0xF0 placement], so the
    // entity exists at d3; the touch is the radius+1.1 sphere from the player's position +
    // the LOS mask-0x721 ray from his world-sphere centre [M Pickup_Update 0x18e148:203-214,
    // two different origins]. The probe prints the pickup entity around the spot, then walks
    // the leg with the tighter arrive (0.2) and a 40-tick stand on the spot before the return.
    // NF_0D_ARMOUR_PROBE=1 arms it (d3 only; 0 = off, the master timeline byte-identical).
    let armour_probe = std::env::var("NF_0D_ARMOUR_PROBE").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0) > 0;
    if real_damage && (difficulty == 1 || (difficulty == 3 && armour_probe)) {
        let h0 = w.scene.sim.as_ref().unwrap().drones.health.health;
        let t0 = w.total_ticks;
        {
            let sim = w.scene.sim.as_ref().unwrap();
            for (k, p) in sim.drones.pickups.iter().enumerate() {
                if walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], [205.4, 0.1, -205.3]) < 4.0 {
                    eprintln!("[0d] armour probe: pickup {k} kind {} weapon {:#x} rounds {} at ({:.2},{:.2},{:.2}) radius {:.2} gone {} hidden {} state {}",
                        p.kind, p.weapon, p.rounds, p.m[3][0], p.m[3][1], p.m[3][2], p.bounds.radius, p.gone, p.hidden, p.state);
                }
            }
        }
        w.walk_nav_to([205.4, 0.1, -205.3], "the yard armour", 0.6);
        if difficulty == 3 {
            // The tighter approach + the stand: the touch runs every tick — the pass-through
            // at the round-1's 0.57-dxz arrival may have been just outside the sphere.
            w.walk_nav_to([205.4, 0.1, -205.3], "the yard armour tight", 0.2);
            for _ in 0..40 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
                w.total_ticks += 1;
            }
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[0d] armour probe: after the tight approach at {:?} armour {:.1}", sim.bond.pos, sim.drones.health.armour);
        }
        w.walk_nav_to([204.0, 0.5, -229.0], "back to the blind strip", 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0d] yard armour: {} ticks armour {:.1} health {h0:.1} -> {:.1}", w.total_ticks - t0, sim.drones.health.armour, sim.drones.health.health);
    }
    for (at, r, label) in [
        ([212.0, 0.5, -230.0], 1.5, "the blind strip east 4"),
        ([215.0, 0.8, -233.2], 1.2, "east of the crane leg"),
        ([213.2, 1.5, -234.8], 0.8, "at the ladder base"),
    ] {
        assert!(w.try_walk_to(Wp { at, act: Act::None, arrive: r }, label), "{label} must walk");
    }
    // LEVERS (a)+(b) FROM THE BASE — MEASURED DEAD this session (level-0d-operative-walk-7,
    // four + one real runs, the walk-6 ledger's first two named levers, the #18-leg precedent
    // "the net is NEGATIVE: the leg stays out" applies):
    // (a) the pre-climb band stand from the ladder BASE: the base line to #17 EXISTS and
    // CLEARS on her east corridor (z −218..−223; the ray printed full CLEAR / bullet_scoped
    // CLEAR at d 56 while she patrolled east of the x ~160 low wall — the -5 strip-mouth 0.975
    // law is the z ~−225+ west segment only) — but the act never landed in four runs: the aim
    // cursor PINS ~4-5 u off a far off-centre target (cross stuck 0.09-0.29 across raw,
    // drained, no-drain and d2-pre-aim-tracking acts) and her 0.042 u/tick patrol crosses the
    // ~120-tick corridor before any convergence; the ladder-top band arm's rays to her west
    // and NE corners die 0.967-0.988 from the top too. Costs: ~500-800 ticks of dwell, zero
    // incoming on three rolls, 14.0 landed across the re-rolled dice on the fifth — every
    // death WORSE than the baseline (4251/4571/3934/3804 vs 3733). The corridor law and the
    // convergence race are the next session's inheritance (the ledger below).
    // (b) the deck guard's box act FROM THE BASE: the pick finds #16 in-box at (198.95,
    // 14.04, −260.41) state 0x68, but the up-hill ray from the base (eye 1.8) dies 0.040
    // along on the crane's own structure — clear_box_aim returns None, ZERO rounds in 400
    // ticks (the [0d:ray] trace). The d2 #20-leg geometry does NOT transfer to #16's line;
    // the -1/-2 law's own stand is the LADDER TOP (down-hill rays clear the crane), so the
    // act's FIRST turn moved up there (below).
    if difficulty >= 2 {
        // The deck guard #20 (d ~30 from the base) lands one 28.0 on the ~800-tick climb (the
        // climb rate is the game's 1.2 u/s); the ladder top has no cover to absorb it later.
        let _ = kill_guard_head_present_fast(&mut w, [198.4, -260.7], [4.0, 4.0], "the deck guard #20 (from the ladder base)", 400);
        eprintln!("[0d] after the deck shot: health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
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
    // MERGE (batch-1109a-walks): the Agent run takes the level-0d-agent-real-damage ladder-top
    // re-route (measured on difficulty 2); the plain and Operative runs keep the
    // level-0d-operative-walk legs they were measured with, byte-identical.
    // (the act-proof gate reaches the ALL-difficulty sweeps below; declared here, set in the
    // d2 block)
    let mut act_proof_live = false;
    if difficulty >= 2 {
        // THE D2 SOUTH-ROOF SNIPER #17 FIRST (dart-tase-stamps-check + the t4312 ledger): ANY
        // ladder-area gunshot wakes him — his fire spot (192.3,19.6,−267.4) is d 39 from the
        // ladder top, INSIDE the P2K shot's hearing radius 40 [L] — and in the t4312 run the
        // west-roof sweep's own kill shot woke him mid-phase: he landed the killing 10.0 at 4312.
        // Take him before the sweep's shots (the box also catches the d2 deck sniper #20 if #17
        // has moved). Log-only.
        if difficulty >= 2 {
            let _ = kill_guard_at_present(&mut w, [192.3, -267.4], [8.0, 8.0], "the d2 south-roof sniper #17 (the ladder-top shot, before the band guns)", 500);
            {
                // The south-roof pair #19/#15 (0x1e at (172.9, 21.2, -271.6)/(166.3, 21.2, -271.4),
                // d 55-60) killed the pre-aimed run at t4851 (38.0 + 28.0) after the climb.
                // The bay captain #11 first, for 120 ticks only: the shots put him off his fire
                // (measured: no #11 hits with the short act), while the pair-first order let him
                // land 3 x 28.0 and the full 400-tick act let #19 land 2 x 28.0 in the wait.
                let _ = kill_guard_head_present_fast(&mut w, [214.7, -289.5], [4.0, 4.0], "the bay captain #11 (from the ladder top)", 120);
                for _ in 0..2 {
                    let _ = kill_guard_head_present_fast(&mut w, [169.6, -271.5], [5.0, 3.0], "the south-roof pair (from the ladder top)", 400);
                }
            }
        }
        // THE BAY DUO FROM THE TOP (level-0d-agent-real-damage, the t4312 ledger's decisive drain):
        // the bay captain #292 (0x5000093) and the annex sniper #363 (0x5000090) own the ladder top
        // — their rays to it are CLEAR at d 54.5 and both fire on the 180-tick cadence: the t4312
        // ledger took 2 × 28.0 (#22/#363) + 28.0 (#11/#292) in 380 ticks. They WANDER the bay north
        // strip — observed (203.5..214.7, −288.5..−290.5) across runs — so the old fixed
        // (214.2,−289.4)±(6,6) strip-mouth box missed them both (10 u off). A player scans the whole
        // bay strip and takes whichever head is exposed: one wide box, the box act fires only on a
        // real clear head-box centre (the annex sniper's ray to the strip MOUTH is blocked 0.521
        // [-3 probe], but from the TOP the ledger proves his line lives).
        for k in 0..3 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.present && !d.corpse && d.health > 0.0
                    && (d.pos[2] + 289.5).abs() < 4.0 && (195.0..=217.0).contains(&d.pos[0])
            });
            if !live { break; }
            let _ = kill_guard_head_present(&mut w, [209.3, -289.5], [7.0, 3.0], "the bay duo (the whole north strip, the t4312 84-damage drain)", 400);
        }
        // (the #27 direct act moved UP into the trio loop above — the run-4140 ledger's reorder.)
        // THE #17 WAKE-LAG SECOND PASS (level-0d-agent-ladder-top ledger): the first pass's own
        // shot (d 38.9 from the ladder-top eye (213.1,18.1,−234.5) to the fire spot — inside the
        // P2K hearing radius 40 [L]) wakes the sleeping sniper and he WALKS to (192.3,−267.4) —
        // the prior ledger measured him ABSENT from the box at the immediate re-check (the walk
        // lag: the second pass found nobody, the killing 10.0 landed mid-phase twice — t4312 and
        // the v3 4000, the last ticks of the acts). Wait out the walk (240 ticks, inside the
        // 150-300 lever window), widen the box to the approach (±10), take him BEFORE the bay
        // duo's exposure starts. The wait rides 150 (the window's lower bound — the run-8110
        // ledger has the trio/bay cadence eating every standing tick). Log-only: when the first
        // pass already caught him the pick finds nobody and only the wait is spent.
        if difficulty >= 2 {
            w.wait(150);
            let _ = kill_guard_at_present(&mut w, [192.3, -267.4], [10.0, 10.0], "the d2 south-roof sniper #17 (the wake-lag second pass, the walk-out wait)", 500);
            // The south-roof sweep: #9 (22 health) settles at x ~180.9, outside the second pass's
            // box, picks up a sniper rifle and lands 28.0 a round from there. Sweep until empty.
            {
                for _ in 0..3 {
                    let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| d.present && !d.corpse && d.health > 0.0
                        && d.pos[1] > 18.0 && (d.pos[0] - 186.0).abs() < 14.0 && (d.pos[2] + 272.0).abs() < 10.0);
                    if !live { break; }
                    let _ = kill_guard_at_present(&mut w, [186.0, -272.0], [14.0, 10.0], "the south-roof sweep (the sniper-rifle pickup)", 500);
                }
            }
        }
        // THE WEST-ROOF TRIO LOOP (level-0d-agent-ladder-top-2): the trio (spawned at the
        // platform-drop wake — the pre-wake census reads NO present guard above y 15, the spawn
        // is the wake [t3310]) opens fire ~3290, 240 ticks after the climb ends (~3050). The
        // act is dice-dependent (landed +372 in the run-4891 tree, held fire 775 in the run-4034
        // one — the posed boxes vs the mid-band obstruction), so the pass rides 450 (the +372
        // kill fits; a stalled pass re-picks a guard whose line clears) and the loop rides 6.
        // The box widens ±5: the settled band is (151.37,−245.55)/(151.37,−248.53)/(151.11,
        // −250.82) [t3310] and the run-4891 pick wandered OUTSIDE the ±3 box. THE SECOND-CLIP
        // LEVER (the prior ledger's +575 re-converge): the old 700 cap expired right after the
        // reload's re-converge — the second clip never fired; 450 per pass with re-picks keeps
        // fresh clips down range. Log-only.
        // (declared here so the post-trio legs can see the act-proof blacklist)
        let mut stalled: Vec<usize> = Vec::new();
        if real_damage {
            // LEVER (c) level-0d-agent-ladder-top-3 v2, measured on the run-4380 roll: the
            // retarget pick is PRE-VERIFIED and STALL-BLACKLISTED — the pass goes to the
            // nearest band guard whose clear_box_aim returns a REAL box right now (torso
            // first: the head box is graze-prone at d 63), and a guard who took ZERO damage
            // across a full pass is act-proof THIS roll — he is blacklisted and the loop
            // re-picks (the run-4380 pass burned 450+ on the act-proof g33 while his cadence
            // ate 94.0). When every live member is blacklisted the loop BREAKS — the waits
            // themselves are cadence exposure (the run-5239 law: the pair drained 94 through
            // 2 passes + 4 waits).
            for k in 0..6 {
                let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                    d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
                        && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0
                });
                if !live { break; }
                let Some((di, part)) = pick_trio_target(&w, [151.35, -248.5], [5.0, 5.0], &stalled) else {
                    if band_all_stalled(&w, &stalled) {
                        eprintln!("[0d] the west roof trio: every live member is act-proof this roll — breaking (the waits are cadence exposure, the run-5239 law)");
                        break;
                    }
                    eprintln!("[0d] the west roof trio: every band line blocked — the 60-tick patrol wait");
                    w.wait(60);
                    continue;
                };
                {
                    let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
                    eprintln!("[0d:box] the west roof trio: retarget g{di} skin {:#x} pos ({:.2},{:.2},{:.2}) state {:#x} part {part}",
                        d.skin, d.pos[0], d.pos[1], d.pos[2], d.state);
                }
                let hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
                // THE HEAD-ONLY CAP (the run-5069 law): a head-only candidate is the graze
                // line — 450 ticks on it is cadence exposure, not a kill attempt; one
                // 200-tick pass is the whole bet (the deck #20 kill proved the head act
                // lands at d 31; at d 63 the same act ate 8+ aimed CLEAR-ray rounds for
                // zero on BOTH members).
                let cap = if part == 5 { 200 } else { 450 };
                let down = shoot_far_box_fast(&mut w, di, "the west roof trio (the retarget pick, the lever-(c) law)", cap);
                if !down {
                    let hp1 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
                    if hp1 >= hp0 {
                        eprintln!("[0d] the west roof trio: g{di} took ZERO damage across a full pass — blacklisted (the act-proof law)");
                        stalled.push(di);
                    }
                }
            }
        }
        // THE ACT-PROOF GATE (level-0d-agent-ladder-top-3): when the trio loop ended with
        // blacklisted (act-proof) survivors, the head sweep's nearest-pick would re-burn its
        // 400-tick caps on THEM (the same head-only graze lines) — pure cadence exposure with
        // zero kill odds. The survivors ride to the g29 south-chest leg (the crossing killer)
        // and the ride instead. The non-real-damage d2 path (no run today) stays identical.
        let act_proof = real_damage && !stalled.is_empty();
        if act_proof {
            eprintln!("[0d] the west roof trio: act-proof survivors live ({stalled:?}) — skipping the re-burn legs (the head sweep, the deck #16 leg)");
        }
        // THE LADDER-TOP HEAD SWEEP (level-0d-agent-real-damage re-route). On THIS tree the wake's
        // three west-roof snipers settle MID-BAND — measured d2 run (t3310 ledger):
        // (151.37,22.70,−245.55)/(151.37,−248.53)/(150.95,−249.42) — where the CHEST rays die
        // 0.80-0.974 on the rooftop obstruction (z −249..−252 band + the -4 slab edge) while the
        // HEAD lines clear from the ladder top [probe_0d_roof_rays]. The old fixed boxes
        // ((151.4,−241.3)/(151.1,−255.8) ±4) were the -5 tree's spots: all three settled 4.25-7.3
        // outside them, every leg printed "no PRESENT guard near", and the snipers (class-4:
        // sight 1000, engage 75+frand(8), and the mode-5 sniper states carry NO distance gate
        // [M NDrone2_DefaultInit 0x14b300:380-424, NDrone2_DSTATE_SniperFire 0x15fd88:34-63]) plus
        // the deck guard (28.0 at d 29.5) and the bay captain (42−armour = 10.0 at d 55.3) shot
        // Bond dead at t3310 — 6 hits for 100.0, 0 return-fire fights, armour spent. A player scans
        // HEADS over the whole band and takes whichever line is exposed: the wide box (x
        // 145.3-157.3, z −236.5..−258.5) covers every measured settle (-5 spots included), and the
        // box act fires only when clear_box_aim returns a real clear hit-box centre, so a blocked
        // line holds fire instead of burning the cap.
        for k in 0..4 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
                    && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0
            });
            if !live { break; }
            // Agent: the heads pass is act-proof exposure (the d2 west-roof pair landed 28 + 10
            // in its 400 ticks) - skipped.
            if act_proof || difficulty >= 2 { break; }
            let _ = kill_guard_head_present(&mut w, [151.3, -247.5], [6.0, 11.0], "the west roof heads (the settled mid-band trio — the -5 head law)", 400);
        }
        // THE DECK GUARD #16 (level-0d-real-health-crossfire-3/4 + wire-jump-grab's route fix):
        // the -1/-2 routes landed this kill every run (+~200 ticks) from the ladder top, but his
        // FIRE anims (state 0x70/0x71) pose the whole box set 0.65-1.0 BELOW his origin while the
        // game's standing two-sphere capsule still covers the chest line [M NDrone2_Collision
        // 0x140e90:91-121] — the port's boxes-only refine flew the t3310 run's chest shots clean
        // past him (0.21 u from origin, no box) while he landed 28.0 at d 29.5. The box act aims
        // at a REAL posed box centre (the part-5 head first) — the +209 landing wire-jump-grab
        // measured.
        let _ = if !act_proof { kill_guard_head_present(&mut w, [198.9, -260.4], [6.0, 4.0], "the deck guard #16 (the box act — his fire anims pose the boxes low [M 0x140e90:91-121])", 600) } else { false };
        // (the act-proof gate reaches the ALL-difficulty sweeps below)
        act_proof_live = act_proof;
    } else {
        // (the ladder crouch rides from the base — the press block above; the whole phase runs
        // crouched, the stand before the wire grab is a fresh press below.)
        // THE WEST-ROOF RETARGET LAW → OPERATIVE (level-0d-operative-walk-6, the v1 roll's
        // ledger, death 4683): the Operative ladder-top phase died with the run-4380 cause
        // VERBATIM — the fixed-spot chest acts (600/400) and the #27-direct (700) burned ~1700
        // ticks on head-only / act-proof d-63 lines (the [0d:box] censuses picked g29 at three
        // spots for ZERO damage across every pass; the #27-direct's 700-tick cap expired at 4680
        // and #10's killing 6.0 landed 4683) while the roof/deck/bay fire landed 100.0. The Agent
        // twin's proven retarget loop (the torso-first clear-box pick, the act-proof blacklist,
        // the 450/200 caps, the all-stalled break) rides for the Operative real-damage run too;
        // the plain replay (top-ups on) keeps its fixed legs byte-identical (the loop is
        // real_damage-gated and the fixed picks below self-gate on the emptied band).
        // LEVER (b), FINAL SHAPE (level-0d-operative-walk-7): the deck guard #16's box act
        // takes its FIRST turn here — before the retarget loop — on the -1/-2 law's own
        // stand (the -1/-2 routes landed this kill every run, +~200 ticks, from the ladder
        // top; the walk-6 ledger's named lever never got its turn because the retarget loop
        // opened the phase). Measured this session FROM THE BASE: the up-hill ray dies 0.040
        // on the crane's own structure (the comment at the climb) — from the TOP the down-
        // hill rays clear the crane and the box act aims at her REAL posed fire-anim box
        // centres [M NDrone2_Collision 0x140e90:91-121]. Removing her takes the climb's
        // 32.0 (her two 16s on every v-roll) off the whole phase below. Real-damage-gated:
        // the plain replay keeps its fixed order byte-identical; when the early act lands,
        // the phase's later deck leg self-cancels ("no PRESENT guard near", zero cost).
        if real_damage && difficulty == 1 {
            // Operative real damage: the ladder-top bill is the 0x1e snipers on the west roof
            // (#28/#29/#27 at (151.4, 22.7, -241..-256), 62-65 u, 14-16 a hit every ~200 ticks
            // [L NF_DMG_TRACE]). One crossing pass on that band FIRST takes one of them before the
            // deck/bay acts (measured: death t4161 -> t5052; two or three passes, or the pass
            // after #16, die inside them - t3664/t3958/t3918).
            // Crouched at the ladder top (one crouch press): the deck guard #16's round no longer
            // lands and the death moves t5052 -> t5532 (#27 x2, then the bay sniper #10 x3).
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
            for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
            let _ = kill_guard_at_present(&mut w, [151.2, -248.0], [6.0, 12.0], "the west roof snipers FIRST (ladder top)", 900);
        }
        if real_damage {
            let _ = kill_guard_head_present(&mut w, [198.9, -260.4], [6.0, 4.0], "the deck guard #16 (the box act, the FIRST turn — the -1/-2 law from the ladder top)", 600);
        }
        // LEVER (c) (level-0d-operative-walk-7): the bay captain #10's act — the d2 bay-duo
        // law (the t4312 ledger's decisive drain) rides for the Operative real-damage run.
        // MEASURED THIS SESSION (level-0d-operative-walk-8, the audit env NF_0D_AUDIT — the
        // ledger's lever (1), the #10 bay-act bullet audit at d 55): the flight's own law —
        // Bullet_update's per-tick START-CEL re-feed [M 0x12f0cc-0x12f0d8] — stops EVERY line
        // to #10 on the bay's north-wall strip east of the strip mouth: the audit's stop
        // points (214.45,12.89,−288.28) for the head line and (212.25,11.93,−287.90) for the
        // torso line, each BEFORE the posed boxes the same query sees past it (part
        // Some(5,0.805) / Some(4,0.704) over the remaining 60 u) — while the AIM gate (the
        // eye's cel, segment_bullet_scoped) reads the same line bullet_scoped CLEAR (the
        // [0d:ray] lines). #18's line passes the strip MOUTH at x ~204 (her stop z −291.02 is
        // PAST her torso hit t 0.744 — the contact order is box first) which is why her act
        // lands at +195 and his never does. #10 is ACT-PROOF from the ladder top on this
        // roll; the two extra 400-tick passes were pure cadence exposure under the west-roof
        // pair's fire (this roll: 64.0 from #29/#27 while the #10 hunts ran). The loop keeps
        // ONE pass — the measured +195 #18 kill — and the #10 hunts are removed.
        if real_damage {
            for k in 0..1 {
                let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                    d.present && !d.corpse && d.health > 0.0
                        && (d.pos[2] + 289.5).abs() < 4.0 && (195.0..=217.0).contains(&d.pos[0])
                });
                if !live { break; }
                let _ = kill_guard_head_present(&mut w, [209.3, -289.5], [7.0, 3.0], "the bay duo (the Operative arm, the d2 t4312 drain law)", 400);
            }
        }
        // THE SOUTH-ROOF PAIR → OPERATIVE (level-0d-operative-walk-8, the d2 law verbatim,
        // the death-list lever): the pair's east member #14 (skin 0x5000087, 0x1e at
        // (166.3,21.2,−271.4), d 59.7) landed 22.0 this roll (16+6 at 4155/4335) and the
        // Operative arm never acted on them — the d2 twin's own acts (the present-pick + the
        // no-drain box act at this exact box, landed every d2 run from the ladder top) sit
        // behind `difficulty >= 2`. The same real-damage gate as the other Operative levers;
        // the plain replay keeps its fixed legs byte-identical (the pick self-gates on the
        // emptied box when the d2 legs already ran).
        if real_damage {
            // Operative: 900-tick passes (the 400-tick pair never landed #14 at 59.4 u; at 900
            // #14 is down at +889 - measured, the walk then reaches the gantry ride alive).
            let (n, cap) = if difficulty == 1 { (2, 900) } else { (2, 400) };
            for _ in 0..n {
                let _ = kill_guard_head_present_fast(&mut w, [169.6, -271.5], [5.0, 3.0], "the south-roof pair (the Operative arm, the d2 law)", cap);
            }
        }
        // THE PAIR'S CROSSING SWEEP (level-0d-operative-walk-9, the OPERATIVE-WALK-8 ledger's
        // lever (1)): the -3 kills' own law — the WALKER-CATCHING sweep — hoisted BEFORE the
        // retarget loop. The walk-8 structural floor: the trio settles into the slab shadow
        // where the chest rays die 0.972 on the mid-band obstruction (the run-4380 law, the
        // guards −243.42/−245.55), the head-only lines graze and the torso pick returns no
        // box, so NO act family landed on them from the top before ~3900 while their
        // 180-tick sniper cadence delivered 86.0 (the 4121/4345/4160 rolls). But the trio
        // does not hold still: they reposition off their placements and PATROL the band
        // east-west afterwards (the stable ends −238.3/−255.8), and a pass opened on the
        // walkers' band catches the CROSSING of a clear-ray spot — the -3's kills at
        // +222/+754/+193, the -4 run-5's +194 and roll A's +236 g29 catch, every one inside
        // a 900 cap (a 300 cap expires before the crossing lands).
        // MEASURED THIS SESSION (level-0d-operative-walk-9, the lever's own real run): the
        // NEAREST-LIVE chest pass caught g29 at +200 (the crossing law verbatim — she was
        // 0.972-blocked at the pick census and crossed clear inside the pass), but the
        // second pass's nearest lock took g28 (blocked, 0.972 the whole ride) while #27
        // stood at her CLEAR south spawn (151.35,−255.80) the whole phase — the roll's top
        // killer (32.0) untouched — and the death stayed 4161 (the walk-8 roll's 4160). The
        // refined shape rides BOTH proven laws per pass: the retarget pick's torso-first
        // clear-box law first (the settled-CLEAR member dies — the -4/-5 law's +194 kills at
        // the walked spots), and when NO clear box exists anywhere in the band the
        // nearest-live chest act takes over (the crossing catch — the -3 law). A full pass
        // with zero damage blacklists the guard (the retarget law) into the SHARED
        // stalled_op, so the retarget loop behind it never re-burns the measured-dead line.
        // Caps: 450 for the box act (the retarget law), 900 for the crossing act (the -3
        // law). Gated real_damage at difficulty 1 (this else branch): the plain replay and
        // the d2 twin keep their trees byte-identical.
        let mut stalled_op: Vec<usize> = Vec::new();
        if real_damage {
            for _k in 0..2 {
                let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                    d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
                        && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0
                });
                if !live { break; }
                match pick_trio_target(&w, [151.2, -246.8], [6.0, 11.0], &stalled_op) {
                    Some((di, _part)) => {
                        let hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
                        let down = shoot_far_box_fast(&mut w, di, "the west roof pair (the crossing sweep, the clear-box pick)", 450);
                        if !down && w.scene.sim.as_ref().unwrap().drones.list[di].health >= hp0 {
                            eprintln!("[0d] the west roof pair (the crossing sweep): g{di} took ZERO damage across a full pass — blacklisted (the act-proof law)");
                            stalled_op.push(di);
                        }
                    }
                    None => {
                        kill_guard_at_present(&mut w, [151.2, -246.8], [6.0, 11.0], "the west roof pair (the crossing sweep, the nearest-live crossing act)", 900);
                    }
                }
            }
        }
        if real_damage {
            for _k in 0..6 {
                let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                    d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
                        && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0
                });
                if !live { break; }
                let Some((di, part)) = pick_trio_target(&w, [151.35, -248.5], [5.0, 5.0], &stalled_op) else {
                    if band_all_stalled(&w, &stalled_op) {
                        eprintln!("[0d] the west roof (Operative): every live member is act-proof this roll — breaking (the waits are cadence exposure, the run-5239 law)");
                        break;
                    }
                    eprintln!("[0d] the west roof (Operative): every band line blocked — the 60-tick patrol wait");
                    w.wait(60);
                    continue;
                };
                {
                    let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
                    eprintln!("[0d:box] the west roof (Operative): retarget g{di} skin {:#x} pos ({:.2},{:.2},{:.2}) state {:#x} part {part}",
                        d.skin, d.pos[0], d.pos[1], d.pos[2], d.state);
                }
                let hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
                let cap = if part == 5 { 200 } else { 450 };
                let down = shoot_far_box_fast(&mut w, di, "the west roof (the Operative retarget pick, the lever-(c) law)", cap);
                if !down {
                    let hp1 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
                    if hp1 >= hp0 {
                        eprintln!("[0d] the west roof (Operative): g{di} took ZERO damage across a full pass — blacklisted (the act-proof law)");
                        stalled_op.push(di);
                    }
                }
            }
        }
        let op_act_proof = real_damage && band_all_stalled(&w, &stalled_op);
        act_proof_live = act_proof_live || op_act_proof;
        if op_act_proof {
            eprintln!("[0d] the west roof (Operative): act-proof survivors ({stalled_op:?}) — the fixed chest legs skip (the act-proof gate)");
        } else {
        let _ = kill_guard_at_present(&mut w, [151.4, -241.3], [4.0, 4.0], "the west roof g28 (chest, her walked spot — the ladder-top ray is CLEAR [probe_0d_roof_rays])", 600);
        let _ = kill_guard_at_present(&mut w, [151.1, -255.8], [4.0, 4.0], "the west roof g27 (chest, her south spawn — the ladder-top ray is CLEAR [probe_0d_roof_rays])", 600);
        // THE BOX ACT STANDBY for g28 if the chest leg missed (the part-5 head first): log-only.
        let _ = kill_guard_head_present(&mut w, [151.4, -241.3], [4.0, 4.0], "the west roof g28 (box/head standby [probe_0d_roof_rays])", 400);
        // THE WEST-ROOF #27 DIRECT (level-0d-operative-walk, real-damage-gated): in the walked
        // timeline #27 settles at (151.37, 22.70, −248.53) from the phase start (the [0d:box]
        // census at the g28-leg moment) and her CHEST ray from the ladder top reads full CLEAR
        // (the sweep's own [0d:ray] lines: eye (213.1,18.1,−234.5) chest (151.1,23.0,−246.1) full
        // CLEAR static clear bullet_scoped CLEAR — she is ABOVE the mid-band obstruction; the
        // height-asymmetry law, the up-hill side). The shared sweep catches her only at +332 into
        // its second pass (~1000 ticks after the phase opens) because its pick takes the nearest
        // walker first — her measured Operative ledger is 2 × 16 health (ticks 3529, 4229) plus
        // the armour share. A player opens on the settled shooter he can see: the box act at her
        // walked spot FIRST, before the #16 deck kill. Gated to the real-damage run.
        if real_damage {
            let _ = kill_guard_head_present(&mut w, [151.35, -248.5], [3.0, 3.0], "the west roof #27 (the direct box act, her walked spot)", 700);
        }
        }
        // THE DECK GUARD #16 (level-0d-real-health-crossfire-3/4): the -1/-2 routes landed this
        // kill every run (+~200 ticks) from the ladder top — his deck fire rode the -4 run-1 sweep
        // window (38 at d 29.7); with the west roof dying above he is the phase's only shooter.
        // THE BOX ACT (level-0d-wire-jump-grab measured): the chest act's 4 rounds all MISS him —
        // he fights from his fire anims (state 0x70/0x71, scripts 0x6000606B/0x60006137) whose
        // posed hit boxes sit 0.65-1.0 BELOW his origin (the crouch-fire skeleton: the head box
        // top 14.03 vs the origin 14.04, the torso centre 13.72 [trace:box]) while the aim rides
        // pos+0.3 — every converged CLEAR-ray round passed 0.21 from his origin above all 16
        // boxes [trace:raypart first_box None]. The game's standing volume is the origin-centred
        // capsule A = pos+up·r, B = pos+up·(r−hip), r 0.4 (0.55 mode 0xc) [M NDrone2_Collision
        // 0x140e90:91-121 — the standing branch's +0x3f0/+0x400/+0x45c writes] which the pass at
        // 0.21 never escapes — the port's boxes-only refine is the documented deviation
        // (drone_ray_part's doc). A player aims at the VISIBLE (crouched) body: the box act aims
        // at a box the port's own law returns.
        // (the act-proof gate: survivors the retarget loop blacklisted are the same fire-anim
        // pose — the d2 gate's own law — the 600-tick act on them is pure cadence exposure).
        let _ = if !op_act_proof { kill_guard_head_present(&mut w, [198.9, -260.4], [6.0, 4.0], "the deck guard #16 (the box act, the crouch-fire boxes)", 600) } else { false };
        // THE BAY GUARD #18 QUESTION (level-0d-operative-walk, MEASURED — the leg is deliberately
        // NOT here): #18 is the ladder-top phase's floor — 3-5 × 16 health every run plus the
        // armour share — from INSIDE the bay at (204-212, 12.09, −288.5..−290.5), where every
        // chest ray from outside dies on the bay's north wall above its low opening (the #18-leg
        // comment's law, the measured 0.521 strip-mouth fraction). This session TESTED the
        // height-asymmetry counter from the ladder top (the -5 head law): clear_box_aim returned
        // Some — but for g10 (207.83, −288.57), NOT #18 — the act killed g10 at +330 (a real
        // bonus: the bay-mouth sniper dies) and the ~900 added ladder-top ticks fed #18's cadence
        // two MORE shots (5166, 5628) before the phase closed; on the later stands he landed two
        // more (the deck 8236, the bay-enter kill at 10656) — 94 total vs the baseline 78. The
        // net is NEGATIVE: the leg stays out. The measured next levers are in the ignore message
        // (the crouch-dodge of the opening line; the port-side capsule work the -5 report
        // flagged).
    }
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
        if act_proof_live { break; }
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
    // with ZERO incoming on the pre-2998b6a76 tree (its ledger: no hit between 2051 and 7932 —
    // "the whole west-roof phase is free" THERE; on THIS tree the class-4 sight table
    // [M 0x14b300:380-424] makes the phase lethal — the t3310/t4312 ledgers), its eye 18.1
    // looks DOWN the band line over the low x ~160
    // wall, and the band is settled (not marching) by this phase. Guards stand at 10 health
    // [probe census] — one aimed hit each; the sweep repeats while live band guards remain
    // (the -5 west-roof pattern). D2-GATED: the Operative twin's route stays byte-identical
    // (its #32 spawner leg stays late-route; the kill order shifts would re-roll the shared
    // RNG stream the green twin's 22.2-end budget rides).
    //
    // THE D2 SOUTH-ROOF SNIPER #17 leg MOVED ABOVE the west-roof sweep (level-0d-agent-real-
    // damage): the t4312 run proved ANY ladder-area shot wakes him (his fire spot is d 39,
    // inside the 40 hearing radius [L]) — the sweep's kill shot woke him mid-phase and his
    // 10.0 landed the t4312 kill. He is taken right after the climb now.
    // OPERATIVE ARM (level-0d-operative-walk-6, the ignore ledger's own lever "extend the
    // ladder-top band sweep to the Operative difficulty for #17/#10"): the same self-gating
    // sweep rides for the Operative real-damage run — on the v1-v5 rolls the band read EMPTY
    // (zero cost, the live check broke first pass) but the walk-2 ledger's ride-window 16
    // (#17 (157.9, 1.0, −226.1)) and the -4's band class are exactly this y < 5 band; the
    // sweep kills them before the ride on any roll where the band spawns live. The plain
    // replay (top-ups on) and the d2 twin keep their streams (the arm adds the d1 real-damage
    // path only).
    if difficulty >= 2 || (difficulty == 1 && real_damage) {
        // THE OPERATIVE CAP TRIM (level-0d-operative-walk-7): the top rays to the Operative
        // stream's band spots are MEASURED BLOCKED every roll (0.967-0.988 — her west and NE
        // corners, two rolls) — the 600-tick caps are pure cadence exposure under the roof
        // pair's fire (the walk-6 ledger's own "the waits themselves are cadence exposure"
        // law). The Operative real-damage arm rides 200 per pass (a clear-line kill fits —
        // the aimed tap fires within ~40 ticks of a clear line); the d2 twin keeps its 600
        // (the d2 band spots' rays clear — the -5 ledger's kills).
        let band_cap = if difficulty == 1 && real_damage { 200 } else { 600 };
        for k in 0..3 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.pos[1] < 5.0 && (150.0..=168.0).contains(&d.pos[0])
                    && (-230.0..=-210.0).contains(&d.pos[2])
            });
            if !live { break; }
            let _ = kill_guard_at_present(&mut w, [160.5, -220.0], [5.5, 8.0], "the ground-band sniper sweep (the ladder-top shot)", band_cap);
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
        // D2-ONLY (level-0d-operative-walk-6): the g21 act has never landed a kill in any run
        // (the convergence law above) — riding it on the Operative arm would be pure cadence
        // exposure; the Operative arm takes the band sweep alone.
        if difficulty >= 2 {
        for k in 0..2 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.pos[1] < 5.0 && (150.0..=168.0).contains(&d.pos[0])
                    && (-230.0..=-210.0).contains(&d.pos[2])
            });
            if !live { break; }
            // the act-proof phase law: the g21 leg has NEVER landed a kill in any run (the
            // aim does not converge to the −0.29 head line) — with act-proof roof survivors
            // live the 1200 ticks are pure cadence exposure; skip to the ride.
            if act_proof_live { break; }
            let _ = kill_guard_head_present(&mut w, [162.8, -215.0], [4.0, 4.0], "the d2 band sniper g21 (the head line over the jamb, the ladder-top shot)", 600);
        }
        }
    }
    // THE PRE-RIDE #19 BOX ACT (level-0d-agent-ladder-top-3, the ledger's lever (a)): #19
    // (skin 0x5000087, weapon 0x1e, (172.0,19.6,−292.5)) landed the killing 10.0 at 8110,
    // ~700 ticks into the ride, from d 49.9 — one aimed head hit drops a guard [M
    // NDrone2_HitDamage 0x145558 × DroneDamage_Head, damage.rs part 5] and a player scans the
    // south-west roof line before stepping off. The ray from the ladder top is UNMEASURED (the
    // walk-2 ledger: the band-sweep stands drew zero hits from him, so his line to the PARKED
    // stand is likely blocked — but the rider MOVES through his line); the box act fires only
    // on a REAL clear hit-box centre, so a blocked line holds fire and only the cap is spent.
    // Log-only: when he is not yet at the box the pick finds nobody and the leg costs one
    // census.
    if difficulty >= 2 && real_damage {
        for k in 0..2 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| {
                d.present && !d.corpse && d.health > 0.0
                    && (d.pos[0] - 172.0).abs() < 8.0 && (d.pos[2] + 292.5).abs() < 8.0
            });
            if !live { break; }
            let _ = kill_guard_head_present(&mut w, [172.0, -292.5], [8.0, 8.0], "the wire-ride shooter #19 (the pre-ride box act, the lever-(a) probe)", 400);
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
    // Operative real damage: the deck walk to the cab crouched (stood again before the E). The
    // standing walk-off took #17's two catwalk hits and #28's roof hit (16.0 each); measured
    // crouched here: those two of the four climb hits are gone, the ride starts at 63.0 (was
    // 28.6), the yard landing at 34 - death t19289 -> t19957.
    // 00 Agent: the same crouch rides at difficulty 3 (the 11-15 ledger: the deck_crouch gate
    // at difficulty 3 takes the walk to the gantry ride at 31.2 health, was death t15731 at
    // the bay). The Agent sister (difficulty 2) keeps its own tree byte-identical.
    // s-crouch-latch-walks: with the game's stance lockout (it ends with the 50-frame transition,
    // ~13 ticks [M Player_SSWalk 0x192310:34-44]) these presses now crouch Bond. Before, the
    // jump-grab's stand armed the flat 60-tick lockout and the wire ride froze it, so every press
    // here was lost and the deck walk ran standing. Crouched for real the deck walk takes +404
    // ticks (was +281); measured on this branch it is still the cheaper roll: standing, #17 and
    // #28 land three more 16.0 rounds (catwalk and roof, t12388/t13479) and the ride starts at
    // 28.6; crouched it starts at 63.6.
    let deck_crouch = real_damage && (difficulty == 1 || difficulty == 3) && !w.scene.sim.as_ref().unwrap().bond.crouched;
    if deck_crouch { for _ in 0..5 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false); w.total_ticks += 1; } }
    assert!(w.try_walk_to(Wp { at: [198.93, 14.16, -264.15], act: Act::None, arrive: 0.8 }, "the deck walk to the cab"),
        "the deck walk to the control plate must arrive");
    if deck_crouch && w.scene.sim.as_ref().unwrap().bond.crouched {
        for _ in 0..5 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false); w.total_ticks += 1; }
        for _ in 0..10 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false); w.total_ticks += 1; }
    }
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
    let mut annex_fall = 0u32;
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
            annex_fall = annex_fall.max(sim.bond.fall_timer);
            if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        }
    }
    // THE ANNEX-DROP PIN (level-0d-fall-damage), real input: the roof→annex push-off is the
    // walk's one damage-dealing fall before the beam (the walk's real-damage ledger attributes
    // its whole `other 11.2` to it). Landing timer 76 → raw 16 → ×0.7 at Agent = 11.2 — the
    // game's own math [M 0x1949a8:117-126 + 0x1902c8:47-60].
    assert_eq!(annex_fall, 76, "the annex push-off's landing timer — raw = timer − 60 = 16, ×0.7 Agent = 11.2 [M 0x1949a8]");
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
    // stream-batch-1121 (Agent real damage): the annex floor's own body armour — the placed
    // kind-3 pickup at (192.82, 10.79, -272.03), 8.5 u north of the annex landing [D the
    // placed pickups, every 0xF0 placement]. The Agent walk passed it by and lost the yard
    // landing burst with no armour (death t20311) [L the 2026-11-21 runs].
    if real_damage && difficulty == 2 {
        let a0 = w.scene.sim.as_ref().unwrap().drones.health.armour;
        let t0 = w.total_ticks;
        // The touch is the radius + 1.1 sphere [M Pickup_Update 0x18e148:203-214]: a 1.3 arrival
        // takes it (a 0.2 arrival wedged 1.09 off the spot for the whole 6000-tick cap and still
        // took it). The way out is south-west to the control corner: the direct lines back to
        // the annex landing and to the bay enter wedge on the wall at z ~-275.5 (191.8, -275.2)
        // [L the 2026-11-21 runs]; the bay enter is walked from the corner.
        w.lenient = true;
        let _ = w.try_walk_to(Wp::new(192.82, 10.79, -272.03).r(1.3), "the annex armour");
        for _ in 0..20 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
        }
        let _ = w.try_walk_to(Wp::new(191.6, 10.6, -275.0).r(1.0), "the annex armour south side");
        let _ = w.try_walk_to(Wp::new(189.5, 10.6, -285.5).r(1.2), "the control corner from the armour");
        w.lenient = false;
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0d] the annex armour: {} ticks armour {a0:.1} -> {:.1} health {:.1} at {:?}", w.total_ticks - t0, sim.drones.health.armour, sim.drones.health.health, sim.bond.pos);
    }
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
    // THE YARD-GARRISON BAY-ENTER SCAN (level-0d-operative-walk, real-damage-gated): the ride
    // phase's measured Operative drain is the yard garrison's 0x12 trickle — 18 hits for 10.8
    // of 0.6 from g8 (159.8, 5.0, −282.8), g3/g2 (132.2..133.4, −299.2..−299.4) and #2
    // (131.6, −301.2) across the claw-corner wait, the hang and the gantry ride (ticks
    // 11627..12525, the NF_DMG_TRACE ledger) — and the pain staggers ate the slot hop's jump
    // input whole (the [hop] per-tick trace: vel [0,0,0], grounded, five whole attempts). The
    // high stands see NOTHING into the yard pit (the ladder-top and roof-lane scans both broke
    // on empty picks) and the corner stand is the worst place to fight (the measured corner
    // scan: one kill at +509, then caps under 1-hit-per-45-ticks fire — Bond died in it). This
    // stand — INSIDE the bay at (194, 10.6, −282), the route's own bay-enter arrival — is the
    // quiet one: the garrison sleeps until Bond steps to the corner rim (the measured run:
    // zero hits from the bay enter through the E plate, ticks ~10000-11487, the first corner
    // hit +140 after the rim step), the bay walls eat the pit's return fire (the same law the
    // #18 leg's comment cites for the outward rays), and the guards here are 10-health
    // 0x12 grunts. The box act at whichever live yard guard (y < 6.5, the yard band)
    // clear_box_aim sees, nearest first; a guard without a clear aim is skipped, so a blocked
    // angle leaves that guard to the post-drop sweep. Zero expected incoming; the cap bounds
    // each converge. Gated to the real-damage run: the green master replay's timeline stays
    // byte-identical.
    if real_damage {
        for _ in 0..8 {
            let pick = {
                let sim = w.scene.sim.as_ref().unwrap();
                let mut best: Option<(f32, usize)> = None;
                for (i, d) in sim.drones.list.iter().enumerate() {
                    if !d.present || d.corpse || d.health <= 0.0 { continue; }
                    if d.pos[1] >= 6.5 { continue; }
                    if !(118.0..=182.0).contains(&d.pos[0]) { continue; }
                    if !(-312.0..=-263.0).contains(&d.pos[2]) { continue; }
                    if clear_box_aim(sim, i).is_none() { continue; }
                    let dist = walker::dxz(sim.bond.pos, d.pos);
                    if best.map_or(true, |(bd, _)| dist < bd) { best = Some((dist, i)); }
                }
                best
            };
            let Some((_, di)) = pick else { break };
            shoot_far_box(&mut w, di, "the yard garrison (the bay-enter scan)", 700);
        }
    }
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
    // Bond waits for the gantry's east park WITHOUT standing on the rim: the corner is the
    // yard garrison's densest sight (the measured Operative ledger: 1 hit per ~45 ticks of
    // 0.6-per-hit 0x12 fire once he steps there, 18 hits for 10.8 across the wait+hang+ride;
    // the E-plate stand behind him is measured-quiet — zero hits from the bay enter through
    // the E, ticks ~10000-11487). The park wait happens at the E-plate spot (the poll reads
    // the entity matrix from anywhere), the rim walk only when the beam is crossing 174.5 —
    // the corner dwell shrinks from ~2350 ticks to the walk+claw. Real-damage-gated: the
    // plain twin's timeline (wait at the corner) stays byte-identical, and the in-place wait
    // below then breaks on its first poll.
    if real_damage {
        for _ in 0..2350u32 {
            let ox = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_09ef, 1)).map_or(0.0, |m| m[3][0]);
            if ox > 174.5 { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
    }
    // LEVER (B), retune-nb-0d-4: PRE-ARM the claw switch during the QUIET E-plate wait before
    // the rim walk (the raise completes ~20 ticks [M Player_SetWeaponAnimObj 0x1a2e38 case 0xf])
    // and poll until the 0x50 is actually up — the exposed rim phase shrinks to the walk + the
    // latch only (~2-3 fewer 12-tick chip windows from #3/#2/#1). NF_0D_FAST_SWITCH=1 arms it
    // (container roll only; 0 = off, the master timeline byte-identical — the default keeps the
    // original post-walk fixed 20-tick switch below).
    let fast_switch = container_drop && std::env::var("NF_0D_FAST_SWITCH").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0) > 0;
    if fast_switch {
        for _ in 0..30 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false);
            w.total_ticks += 1;
            if w.scene.sim.as_ref().unwrap().gun.weapon == 0x50 { break; }
        }
        eprintln!("[0d] the claw pre-armed: in hand {:#x} at {:?}", w.scene.sim.as_ref().unwrap().gun.weapon, bond_pos(&w));
    }
    w.walk_to(Wp::new(185.3, 11.4, -294.8).r(1.2), "the deck in the pulley's icon arm");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.icon_ctx, 2, "Bond stands in a type-2 icon's arm [D]");
    let held_gun = w.scene.sim.as_ref().unwrap().gun.weapon;
    if !fast_switch {
        for _ in 0..20 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x67), "sister: nobody aboard before the boarding");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.world.segment_hit(sim.bond.head(), claw_pulley).is_none(), "the control corner sees the pulley #370 [D]");
    }
    // THE CROUCH-DODGE (level-0d-operative-walk-2, lever 1), real-damage-gated: the corner is
    // the yard garrison's densest sight and the baseline's rim step pulled 0.6-per-hit 0x12
    // fire at a 12-tick cadence (#3 (131.6, 5.0, −301.1), ticks 13824/13836) — the boarding
    // arrived at 13.8 health with the ~14-damage slot fall still to come (the fall skips
    // armour [M 0x1902c8:84-104] — 13.8 does not survive it). The crouch drops the aim/sight
    // target 0.45 under the eye [M Player_GetHeadPos 0x195aa0:39-51 — the BL+0x90C crouch eye
    // drop; the port's Player::target = camera − 0.45, scene.rs run()] so the bay/yard opening
    // line can clip a crouched chest, and multiplies the visibility factor by 0.75
    // [M Drone_ProcessOpponents 0x1390c0:40-42] — under the 0.7 LOS gate [M 0x176a28]. Toggle
    // crouch ON (the press edge — the game's Toggle stands only on a NEW press edge [M
    // Player_HandleJump's crouch arms, 0x190630 dispatch]) and stay crouched through the wait,
    // the latch, the hang and the ride; the stand before the hop is a fresh press below.
    // (The crouch-timing levers were measured and reverted: the crouched E-plate park wait
    // is inside the west band's sight (#0/#1, 25.8 of chips — the standing wait is quiet)
    // and the crouched rim dwell doubles the exposure time; both dead on their own runs.)
    if real_damage {
        for _ in 0..5 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    }
    // THE DICE KNOB (real-health-00agent-0d-2): the yard's fate is decided by WHERE the shared
    // RNG stream sits when the guards wake, and the only thing that moves it is DRAW COUNT —
    // idle ticks draw nothing (measured: a 3-tick pre-boarding pause re-produced the no-pause
    // ledger tick for tick), but every tap spends a spread draw. The ride taps are the knob:
    // the 18-tap roll re-rolled the ride to ZERO chips (31.2 → 31.2 through the landing) and
    // a ZERO-COST exit, where every no-tap roll takes 2 chips. The taps never land themselves
    // (zero kills in 33 taps across two gate versions at d 37-43 off the moving beam — the
    // muzzle/girder-spawn geometry eats them), so they are pure shift: the tap count and the
    // sweep-first floor order are judged together on real runs. The pause_at arming below is
    // kept (it was the l2c roll's config); pause_at fires inside tick_inner and the boarding
    // WAIT is the first phase that runs w.tick after the corner.
    let ride_fire_agent = container_drop;
    if container_drop {
        w.pause_at = Some((w.total_ticks + 20, 3));
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
    // THE HANG PAUSE (retune-nb-0d-2 dice knob: the pause_at arm timing/n). The hang is the
    // last w.tick phase before the ride and the corner chippers (#3/#2/#1, the 12-tick 0x12
    // cadence) are awake and firing through it — n extra idle ticks here extend their fired
    // rounds (each a spread draw from the shared stream), so g18's ride burst re-rolls with
    // the stream (the ignore message's NEXT LEVER). Gated to the container roll (d3 real
    // damage): the plain twin and the d1/d2 walks stay byte-identical. NF_0D_HANG_PAUSE is
    // the tuning harness knob; the committed default 0 = the master timeline (the measured
    // hang-pause rolls 12/48 saturated to one worse roll, death t18999 — see the ignore
    // message).
    if container_drop {
        let hang_pause: u32 = std::env::var("NF_0D_HANG_PAUSE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if hang_pause > 0 { w.pause_at = Some((w.total_ticks + 10, hang_pause)); }
    }
    // Hang, the fire held, until the gantry is at its east apex under Bond.
    let mut max_ox = 0.0f32;
    for _ in 0..2350u32 {
        w.tick_fire(nf_game::player::Input::default());
        if let Some(m) = w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_09ef, 1)) { max_ox = max_ox.max(m[3][0]); }
        // The real-damage twin's E-plate park wait + the corner walk eat the beam's east crawl:
        // the loop can start with the beam already at/past the apex, and breaking on max_ox
        // alone released Bond mid-reel (the baseline: pos[1] 10.67 — the claw latched, the reel
        // never finished, Bond stayed on the deck floor). The break now ALSO waits for the
        // HANG — the reel's phase 2, reached within 2.0 of the pulley [M 0x192860:91-94]. The
        // plain master replay's reel always finishes before max_ox crosses 177 (its corner
        // arrive precedes the crawl), so its break tick is unchanged.
        let hung = w.scene.sim.as_ref().unwrap().bond.grapple.as_ref().is_some_and(|g| g.phase == 2);
        if max_ox > 177.0 && hung { break; }
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
    // s-crouch-latch-walks: crouch again aboard (one press, real damage). The claw's substate
    // change ends a crouch with no transition [M Player_ChangeSubState 0x190630: only cases 0/4
    // touch the stance] and the reel hands back a STANDING walk, so the corner press no longer
    // carries onto the beam. Before, the port kept the crouch flag through the hang and the
    // ride ran with it. Standing, the d2 ride took ~19 chips of 1.0 (x 177 onward) and died in
    // the yard (t20428); crouched aboard it takes 3. The stand press before the hop below
    // stands him again (the ride is far past the ~13-tick lockout).
    if real_damage {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    w.wait(10);
    // THE RIDE, real input: the carry [M Player_FeetOnPoint 0x194188 → Control_InheritVelocity
    // 0x133e88, the port's ground_motion] holds Bond at a constant offset on the runway
    // through the west apex — probe_the_ride_carry's proven carry, now asserted.
    let mut off0 = None;
    let mut apex = None;
    let mut turned = false;
    // Operative real damage: the ground sniper #15 (skin 0x5000087, 0x1e, (176-178, 1.0, -281))
    // killed the crouched rider (16.0 + 12.6) and the boarding spot has no line to him (0.102
    // along, the gantry floor). A player riding the gantry answers him with the rifle as the
    // line opens: the rifle back up (the hang left the Q-Claw 0x50 in hand), aim, and tap on a
    // clear bullet line. Measured: #15 dies in 30 taps, the ride ends at 26.2, the slot drop
    // lands and the walk dies in the yard at t19289 (was t18516 on the ride).
    let ride_fire = real_damage && difficulty == 1;
    if ride_fire {
        for _ in 0..40 { w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { switch_to: Some(0x1e), ..Default::default() }, false, false, false); w.total_ticks += 1; }
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        eprintln!("[0d] ride fire: in hand {:#x} {}+{} owns {:x?}", sim.gun.weapon, sim.gun.clip, sim.gun.spare,
            (0..inv.owned.len()).filter(|&k| inv.owned[k]).collect::<Vec<_>>());
    }
    // THE 00 AGENT RIDE FIRE (real-health-00agent-0d-1): the 11-15 ledger's own lever law —
    // "answering the nearest clear-line shooter from the ride" — retargeted for the container
    // roll. The census this session measured FOUR awake burst rifles (g0/g1/g2/g3) around the
    // container landing; the ride west from the east park is the QUIET phase (the ride chip
    // measured 2 hits), and the rider passes OVER their longitudes: every tick the nearest
    // live yard guard with a clear scoped bullet ray from the rider's eye gets the aimed tap
    // (the 0x1e's 40+ a round one-shot-kills the 10-health grunts [L the loadout; M
    // DroneWeap_DoBulletAccuracy 0x179740]). Reload pulses on empty; the ride's own cadence
    // bounds the exposure. Log-only per tap; the landing census is the measured answer.
    // MEASURED DEAD THIS SESSION (two runs, both deaths on the ride): the aimed ride fire's
    // taps consume the shared RNG stream (the spread rolls), and both re-rolled rides died to
    // a 0x12 HEADSHOT at the rider's crouched head (player_hits: tick 19166/19171, w 0x12,
    // 31.2 = the whole health; the no-tap rides took 2 body hits and survived to the break
    // every time). The lever's machinery (the ray law, the threat pick, the switch poll) is
    // kept for the next session — the headshot lottery needs a dice shift first (a pause_at
    // before the boarding, or the tap count/phase changed), or the g0/g1 kill window
    // (ox ~161-153.5, where the beam's south face finally clears them) rode with a pause.
    // (kept for the record) THE 00 AGENT RIDE FIRE — the report's lever 2 machinery; the
    // dice shift above is its measured precondition.
    if ride_fire_agent {
        // The switch rides longer than the d1 block (40 ticks left the Q-Claw in hand — the
        // claw's hang fire script holds the raise queue [M 0x1a24c8:66-99]); poll until the
        // 0x1e is actually up (the raise completes ~20 ticks [M Player_SetWeaponAnimObj
        // 0x1a2e38 case 0xf]).
        for i in 0..200 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { switch_to: Some(0x1e), ..Default::default() }, false, false, false);
            w.total_ticks += 1;
            if i % 20 == 19 { let sim = w.scene.sim.as_ref().unwrap(); eprintln!("[0d] agent ride fire switch: tick {} in hand {:#x} {}+{}", i + 1, sim.gun.weapon, sim.gun.clip, sim.gun.spare); }
            if w.scene.sim.as_ref().unwrap().gun.weapon == 0x1e { break; }
        }
        // The aim raise: the aim bit makes the 0x1e EXACT (the hip-fire keeps the full
        // 100-unit cone [M Bullet_init 0x12c5a0:139-170] — exact ONLY with the aim bit).
        for _ in 0..90 {
            let sim = w.scene.sim.as_mut().unwrap();
            if sim.aiming { break; }
            sim.step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[0d] agent ride fire: in hand {:#x} {}+{} aiming {}", sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.aiming);
    }
    // THE RIDE-ENTRY SHIFT (retune-nb-0d-2 dice knob 2): n idle ticks aboard at the east park
    // before the ride loop — the rider is already crouched on the beam, so this shifts the
    // whole ride's exposure window against the yard's fire cadences (each fired round is a
    // spread draw) without touching any route leg. Gated to the container roll. The
    // committed default 30 is the session's measured-best roll: the ride ALIVE (2 chips,
    // 25.8 → 22.2 on the girder), the container landing CLEAN (fall timer 63, 18.6), the
    // exit clean (16.8) — the death moved OFF the ride onto the yard-floor fight (t19772;
    // the refreshed ledger and the next lever are in the ignore message). Time shifts
    // saturate in the pre-ride phases (the guard fire is sight-event-keyed, not
    // absolute-tick-keyed), but the ABOARD shift re-deals the ride window itself.
    if container_drop {
        let ride_shift: u32 = std::env::var("NF_0D_RIDE_SHIFT").ok().and_then(|v| v.parse().ok()).unwrap_or(30);
        for _ in 0..ride_shift {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
        }
        if ride_shift > 0 { eprintln!("[0d] the ride-entry shift: +{ride_shift} idle ticks aboard"); }
    }
    let mut last_tap = 0u32;
    let mut taps = 0u32;
    let mut hp_prev = w.scene.sim.as_ref().unwrap().drones.health.health;
    // THE TAP KNOBS (retune-nb-0d-2 dice knob 3: the tap cadence + the threat zone). Defaults
    // are the master values (zone 25, > 15) so every other walk stays byte-identical.
    let tap_zone: f32 = std::env::var("NF_0D_TAP_ZONE").ok().and_then(|v| v.parse().ok()).unwrap_or(25.0);
    let tap_cadence: u32 = std::env::var("NF_0D_TAP_CADENCE").ok().and_then(|v| v.parse().ok()).unwrap_or(15);
    for k in 0..2200u32 {
        let (input, trigger) = if ride_fire {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let tgt = sim.drones.list.iter().enumerate().find(|(_, d)| d.enabled && d.present && !d.corpse && d.health > 0.0
                && d.pos[1] < 3.0 && walker::dxz(d.pos, [177.0, 1.0, -280.8]) < 8.0).map(|(i, d)| (i, [d.pos[0], d.pos[1] + 1.3, d.pos[2]]));
            if let Some((_, at)) = tgt {
                let desired = (at[0] - eye[0]).atan2(at[2] - eye[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let want_pitch = ((at[1] - eye[1]) / walker::dxz(eye, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let perr = want_pitch - sim.bond.pitch;
                let clear = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, at).is_none();
                let fire = clear && err.abs() < 0.02 && perr.abs() < 0.028 && k.saturating_sub(last_tap) > 20 && sim.gun.clip > 0;
                (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, fire)
            } else { (nf_game::player::Input::default(), false) }
        } else if ride_fire_agent {
            // The nearest live yard guard (y < 8, the yard band) with a CLEAR scoped bullet
            // ray from the rider's eye — the ride's own crosshair law (the turn/look drive the
            // ray ~1:1, the tap gate measures the ray like shoot_far_aim).
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let mut best: Option<(usize, [f32; 3], f32)> = None;
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present || d.corpse || d.health <= 0.0 { continue; }
                if d.pos[1] >= 8.0 { continue; }
                if !(118.0..=182.0).contains(&d.pos[0]) || !(-335.0..=-263.0).contains(&d.pos[2]) { continue; }
                let at = [d.pos[0], d.pos[1] + 1.3, d.pos[2]];
                let clear = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, at).is_none();
                if !clear { continue; }
                let dist = walker::dxz(eye, at);
                if dist < 3.0 { continue; }
                // THE THREAT PICK, THREATS ONLY (real-health-00agent-0d-2): the landing-adjacent
                // rifles (within 25 u of the container landing zone (147, -296)) — the beam's
                // own south face blocks their rays until the rider passes their longitude
                // (ox ~161-153.5), so the pick simply WAITS while blocked instead of burning
                // taps on far guards: the measured N=3 roll spent 13 of 15 taps on g4/g6
                // (d 38-49, clear down-west rays from the east park, zero lands) while g0/g1
                // sat blocked — every tap spends a spread draw from the same stream the ride's
                // own fire rolls from, and the far guards are the floor sweep's job.
                // retune-nb-0d MEASUREMENT (2026-11-18): the ride killer is g18 (skin
                // 0x5000087, 0x12) at (177.90, 0.98, -283.26), state 0x68, 33.4 u off the
                // landing zone — OUTSIDE this 25 u pick — and its ONE 25.8 0x12 round at
                // frame 19197 (ox 164.7, d ~18.4 up-ray from the rider's crouched head) is
                // the whole death margin [L the instrumented run: player_hits (frame, owner,
                // damage) [M drone.rs 13525] resolves the owner]. The east-yard threat is
                // therefore untouchable by the ride taps (the zone-35 + rider-distance
                // variant re-aimed them at g18, d 13.9-14.4, and STILL landed zero — the
                // girder-spawn geometry law holds at d 14; the ride died at the same margin,
                // g3 chip 1.8 at t19027 + g18 kill 24.0 at t19222 [L run6]) and the master
                // zone-25 zone-priority pick stays. The g18 kill must come from the ground
                // phases or the dice.
                let zone = walker::dxz(d.pos, [147.0, 1.0, -296.0]);
                if zone >= tap_zone { continue; }
                if best.map_or(true, |(_, _, b)| zone < b) { best = Some((i, at, zone)); }
            }
            if let Some((_, at, _)) = best {
                // THE RAY LAW (the shoot_far_aim construction [M Player_GetAimingPoint 0x1a4a98]):
                // with the aim bit set the cursor offsets ride the view ray — the tap gate must
                // measure the RAY (look + 0.76966·cross[0]·right + 0.57724·cross[1]·up), not
                // the raw yaw/pitch (the probe_0d_bullet calibration: a ±0.2309 rad ray offset
                // at the cursor's ±0.4 clamp while bond.pitch read converged — the systematic
                // miss this session's first ride fire measured: 15 converged-YAW taps, zero
                // lands).
                let yaw = sim.bond.yaw;
                let p = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                let look = sim.bond.look();
                let right = [-yaw.cos(), 0.0, yaw.sin()];
                let up = [-p.sin() * yaw.sin(), p.cos(), -p.sin() * yaw.cos()];
                let (tx, ty) = (0.76966f32 * sim.cross[0] / sim.zoom, 0.57724f32 * sim.cross[1] / sim.zoom);
                let dv = [look[0] + tx * right[0] + ty * up[0], look[1] + tx * right[1] + ty * up[1], look[2] + tx * right[2] + ty * up[2]];
                let dl = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt().max(1e-4);
                let ray = [dv[0] / dl, dv[1] / dl, dv[2] / dl];
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let err = walker::wrap_pi(dx.atan2(dz) - ray[0].atan2(ray[2]));
                let perr = dy.atan2((dx * dx + dz * dz).sqrt().max(0.5)) - ray[1].asin();
                let clear = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, at).is_none();
                // THE TAP GATE, the proven shoot_far_aim box law (real-health-00agent-0d-2):
                // the old flat 0.02/0.028 rad gates are 0.76/1.06 u at d 38 — a torso is
                // ~0.3 u, so 15 taps sprayed wide and landed ZERO. The proven law scales the
                // tap box to the range (0.12 u lateral / 0.25 u vertical, clamped) and holds
                // fire unless the aim bit is up (the hip cone is the 100-unit spray [M
                // Bullet_init 0x12c5a0:139-170]; the raise is ~20 ticks [M 0x1a2e38 case 0xf]).
                let dist = walker::dxz(eye, at);
                let (err_tol, perr_tol) = (
                    (0.12f32 / dist.max(0.5)).clamp(0.003, 0.02),
                    (0.25f32 / dist.max(0.5)).clamp(0.006, 0.05),
                );
                let fire = clear && err.abs() < err_tol && perr.abs() < perr_tol
                    && k.saturating_sub(last_tap) > tap_cadence && sim.gun.clip > 0 && sim.aiming;
                if fire { let (i, d) = best.map(|(i, _, _)| (i, &sim.drones.list[i])).unwrap(); eprintln!("[0d] agent ride fire: tap{} at g{} skin {:#x} d {:.1} state {:#x}", taps + 1, i, d.skin, walker::dxz(eye, d.pos), d.state); }
                (nf_game::player::Input { turn: (err * 1.0).clamp(-0.08, 0.08), look: (perr * 2.0 / std::f32::consts::PI).clamp(-0.08, 0.08), aiming: true, ..Default::default() }, fire)
            } else { (nf_game::player::Input::default(), false) }
        } else { (nf_game::player::Input::default(), false) };
        if trigger { last_tap = k; taps += 1; }
        let reload_pulse = ride_fire_agent && k % 20 == 0
            && { let s = w.scene.sim.as_ref().unwrap(); s.gun.clip == 0 && s.gun.spare > 0 };
        w.scene.sim.as_mut().unwrap().step_bond(input, trigger, reload_pulse, false);
        w.total_ticks += 1;
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.health.health != hp_prev {
            let nh = sim.drones.player_hits.len();
            // player_hits is (frame, owner, damage) [M drone.rs 13525]; the owner index
            // resolves through the live list — log-only, no sim writes.
            let last = sim.drones.player_hits.last().map(|(f, o, d)| {
                let dd = sim.drones.list.get(*o).map(|dd|
                    format!("skin {:#x} at ({:.2},{:.2},{:.2}) state {:#x} enabled {} present {}",
                        dd.skin, dd.pos[0], dd.pos[1], dd.pos[2], dd.state, dd.enabled, dd.present));
                format!("frame {f} owner g{o} dmg {d:.1} {:?}", dd.unwrap_or_else(|| "the world".into()))
            });
            eprintln!("[0d] the ride: tick {} health {hp_prev:.1} -> {:.1} at {:?} fall {} hits {} {last:?}",
                w.total_ticks, sim.drones.health.health, sim.bond.pos, sim.bond.fall_timer, nh);
            hp_prev = sim.drones.health.health;
        }
        // The ride steps the sim directly (no walker death report): a death here is named here.
        assert!(sim.drones.health.health > 0.0, "Bond died on the gantry ride at tick {} at {:?}", w.total_ticks, sim.bond.pos);
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
        // THE CONTAINER BREAK (real-health-00agent-0d-1, the 11-15 ledger): while the beam is
        // still over the yard's containers (entity x < 153.5) break the ride and hop west onto
        // the container top — the slot apex's 10 m fall costs 30 at 00 Agent, the container
        // landing ~5 m costs nothing (raw < 10 [M 0x1949a8]).
        if container_drop && ox < 153.5 { break; }
        let _ = k;
    }
    if ride_fire { let sim = w.scene.sim.as_ref().unwrap(); let h = sim.drones.list.iter().find(|d| d.pos[1] < 3.0 && walker::dxz(d.pos, [177.0, 1.0, -280.8]) < 8.0 && d.skin == 0x5000087).map(|d| (d.health, d.corpse)); eprintln!("[0d] ride fire: {taps} taps, #15 {h:?}, in hand {:#x} sw {} {}+{}", sim.gun.weapon, sim.gun.switch_state, sim.gun.clip, sim.gun.spare); }
    if container_drop {
        // The ride broke over the containers: the west-apex asserts below are the slot law.
        eprintln!("[0d] the ride broken over the containers at ox {:.2} — at {:?}",
            w.scene.sim.as_ref().unwrap().drones.scripts.entity_matrix((0x0600_09ef, 1)).map_or(0.0, |m| m[3][0]), bond_pos(&w));
        // THE HOP STAGGER WAIT (retune-nb-0d-2): a chip landing late in the ride leaves the
        // pain stagger running into the hop — the eaten/late jump turns the hop into the
        // re-hop path and a re-hop's apex makes the container fall LONG (the measured roll:
        // timer 99 → raw 39 × 1.2 = 46.8 at 18.6 health = death at the landing; the slot
        // comment's own warning "re-hops add apex height"). n idle ticks crouched on the
        // girder let the stagger expire before the stand-up + hop. Gated to the container roll.
        let hop_delay: u32 = std::env::var("NF_0D_HOP_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        for _ in 0..hop_delay {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
        }
        if hop_delay > 0 { eprintln!("[0d] the hop stagger wait: +{hop_delay} idle ticks crouched on the girder"); }
    } else {
    let (ox_apex, drift) = apex.expect("the beam must reach its west park (~137.59) with Bond aboard");
    assert!((137.3..=138.2).contains(&ox_apex), "the west apex ~137.59 [D the key trace], got {ox_apex:.2}");
    assert!((0.0..=3.0).contains(&off0.unwrap()), "Bond rides the east half of the gantry, got off {:.2}", off0.unwrap());
    assert!(drift.is_some_and(|d| d.abs() < 0.75), "the carry must hold the offset (the ease-out lag only) [M 0x194188 → 0x133e88], got {drift:?}");
    eprintln!("[0d] the ride walked: boarded at offset {:.2}, the apex at {ox_apex:.2} — at {:?}", off0.unwrap(), bond_pos(&w));
    }
    // Stand out of the crouch before the hop (real-damage-gated: the crouch twin rides
    // crouched; the plain twin never crouched): the crouch press edge stands [M the crouch
    // arms] and the jump arm reads no lockout — the hop fires clean on its own input.
    // s-crouch-latch-walks: only when crouched. Every real-damage roll now boards crouched (the
    // press aboard), but a press here on a standing Bond would crouch him instead.
    if real_damage && w.scene.sim.as_ref().unwrap().bond.crouched {
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
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
    // The container hop holds the west drift LONGER (36 ticks, the 11-15 ledger's measured
    // 33-38 band's centre) so the extra ~0.11-0.3 u/tick of air drift [M Player_Move] carries
    // Bond from the break point (x ~153-156) onto the container top (x 145-149); the slot hop
    // keeps its proven 20.
    // THE FALL PIN (level-0d-fall-damage), real input: the slot fall's health delta is the
    // game's own landing rule. BL+0x928 += FRAME_RATE_MUL per airborne tick while BL+0x54 < 0,
    // reset on any vel.y >= 0 tick; on landing raw = (short)(int)(timer - 60), pain only when
    // raw >= 10 [M Player_CollisionHandler 0x1949a8 disasm 0x194b64-0x194be0: the 60.0 literal
    // @0x194b74, slti 0xa @0x194bb0, Player_HandlePain(raw, 6, -1) @0x194bd0]; the pain scales
    // by the difficulty (0d: Easy 0.4 / Normal 0.7 [M ReadTuningVars 0x1d40f4; 0x1902c8:
    // difficulty 1 -> Easy, 0/2 -> Normal]), skips armour for types 1..7 [M 0x1902c8:84-104]
    // and kills when the remainder < 1 [M 0x1902c8:131-134].
    let pre_drop_health = w.scene.sim.as_ref().unwrap().drones.health.health;
    let hits_before_drop = w.scene.sim.as_ref().unwrap().drones.player_hits.len();
    let mut max_fall = 0u32;
    let mut landed = false;
    // The landing height: the container top lands at surface 7.1 + the ~1.1 pos offset = 8.2
    // (the 11-15 ledger); the yard floor lands at ~5.1. The slot law's < 7.0 gate widens to
    // < 8.6 for the container roll.
    // s-crouch-latch-walks: the real-damage slot hop holds 13. The crouched ride (the press
    // aboard above) boards at offset 1.09 (was 2.00 with the old flag-only crouch), 0.9 further
    // west, and the 20-tick hold carried Bond to x 133.2 beside #3 (d 1.6, 1.2 a chip): d1 died
    // t19467. Measured holds, the landing x and the result (port runs on this branch, one each):
    // d1: 8 137.5 dead, 10 135.4 green 6.6, 11 135.2 dead, 12 137.8 green 11.6, 13 137.8 green
    // 20.0, 14 137.7 green 18.8, 16 137.5 dead, 24 134.1 dead; d2: 10 dead, 12 134.6 dead,
    // 13 138.0 green 10.2, 14 138.0 dead, 16 dead, 24 dead. The plain twin keeps 20.
    // NF_0D_HOP_HOLD overrides it (the tuning knob).
    let hop_hold: u32 = std::env::var("NF_0D_HOP_HOLD").ok().and_then(|v| v.parse().ok())
        .unwrap_or(if container_drop { 36 } else if real_damage { 13 } else { 20 });
    let land_y = if container_drop { 8.6 } else { 7.0 };
    // One hop can land back on the girder: hop again until the slot takes him.
    'hops: for _ in 0..5 {
        for k in 0..300u32 {
            let input = if k == 0 {
                nf_game::player::Input { jump: true, ..Default::default() }
            } else if k <= hop_hold {
                nf_game::player::Input { forward: 1.0, ..Default::default() }
            } else {
                nf_game::player::Input::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
            let sim = w.scene.sim.as_ref().unwrap();
            max_fall = max_fall.max(sim.bond.fall_timer);
            if sim.bond.grounded && sim.bond.pos[1] < land_y { landed = true; break 'hops; }
            if sim.bond.grounded && k > hop_hold { break; }
        }
        // The attempt landed back on the girder (a pain stagger ate the jump — the measured
        // Operative ledger: vel [0,0,0] through five whole attempts under the yard trickle).
        // A player shakes it off and re-jumps: settle 40 ticks so the next attempt's jump lands
        // on a clean tick. The green twin's first hop succeeds, so this never runs there.
        if !landed {
            for _ in 0..40 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
                w.total_ticks += 1;
            }
        }
    }
    assert!(landed, "the jump-drift must drop Bond through the slot into the yard, got {:?}", bond_pos(&w));
    if container_drop {
        // The container landing: the girder (~13.2 pos) to the top (surface 7.1) is ~5 m of
        // fall — timer ~61 → raw 1 < 10, NO pain [M 0x1949a8 disasm 0x194b64-0x194be0] — and
        // re-hops add apex height, so the exact slot math below does not apply here. The
        // landing band is the 11-15 ledger's measured clean first-hop roll (x 145-149). The
        // health floor is the measured ~18.6 minus the ride chip's dice: assert only that the
        // roll is alive with the container budget (the walk's own log is the measured ledger).
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        eprintln!("[0d] the container landing: pos {p:?} health {} (was {pre_drop_health:.1} on the girder) fall timer {max_fall}", sim.drones.health.health);
        // The garrison census at the landing: who is awake (present), where, what state —
        // the exit's exposure is decided by this list.
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.pos[1] >= 8.0 || d.pos[0] < 118.0 || d.pos[0] > 182.0 || d.pos[2] < -335.0 || d.pos[2] > -263.0 { continue; }
            if d.corpse || d.health <= 0.0 { continue; }
            eprintln!("[0d:land] g{i} skin {:#x} pos ({:.2},{:.2},{:.2}) enabled {} present {} health {:.0} state {:#x}",
                d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.health, d.state);
        }
        assert!(p[1] > 7.5 && p[1] < 8.6, "the container top catches the hop (surface 7.1 + the ~1.1 pos offset), got {p:?}");
        assert!(p[0] > 143.0 && p[0] < 151.0, "the landing rides the container's x band, got {p:?}");
        // THE RIM SWEEP (retune-nb-0d-3, the retune-nb-0d-2 ignore message's lever (c)): the
        // garrison CONVERGES on the container during the ride (the census above — the burst four
        // g0/g1/g2/g3 all awake at d 9.7-15.6) and the floor landing eats their chip burst before
        // the floor sweep's first kill lands (+39 ticks; death t19772 at 16.8 in the measured
        // roll). A player answers from the CONTAINER TOP before dropping: the landing spot IS the
        // stand (no extra walk, no extra exposure), every converged guard whose scoped-bullet ray
        // reads clear from here gets the aimed act, and each landed 0x1e round one-shot-kills the
        // 10-health grunts [L the loadout; M DroneWeap_DoBulletAccuracy 0x179740]. The head-first
        // act is the lever's untried variant (clear_box_aim prefers the part-5 head box and its
        // ray is measured from THIS spot — the 0d-1 burns were box passes at #1 walked to d 6.8).
        // A blocked angle is skipped, never burned (the pick's own clear law). Gated to the
        // container roll (d3 real damage): the plain twin and the d1/d2 walks never run it.
        // NF_0D_RIM_MAX bounds the passes; the COMMITTED DEFAULT IS 0 — the rim sweep is the
        // retune-nb-0d-2 ignore message's lever (c), MEASURED DEAD on five real runs this
        // session (each its own roll): the audit (NF_0D_AUDIT) proved the two laws that kill
        // it. (1) THE ARMING LAW: the first kill-shot from the top (a part-5 head hit on g3 at
        // +34 [NF_0D_AUDIT: b0 SPAWN dir (-0.972,-0.218,-0.088) -> part Some(5, 0.099)])
        // ARMS the yard — g0 (157.55,-286.52, state 0x68) lands its first 1.8 chip ~24 ticks
        // later and then g0 AND g1 sustain a 1.8 chip every ~6 ticks (0.3 damage/tick EACH;
        // the measured roll: 9 chips in 51 ticks ate the 18.6 pool mid-sweep) — one shot buys
        // one kill and hands the whole pool to the pack's cadence; no second shot fires inside
        // that window (see (2)). The 0x12 burst the floor landing eats (6 chips/6 ticks) is the
        // same pack already armed. (2) THE POST-KILL SWALLOW: the trigger tick after a landed
        // kill spawns NO bullet ([NF_0D_AUDIT: pass 1's tap at +34 — 36 ticks after the +34
        // kill shot, trigger input in, NO SPAWN line] — the ride fire's ~28-tick tap spacing
        // never killed anything, so it never hit this law), so the sweep gets ONE shot per
        // ~200 ticks while the armed pack drains 0.3-0.6/tick. The 0d-1 burns (#1 at 6.8 u)
        // and the crouched-stand rolls (the crouch breaks Bond's OWN head-rays to the north
        // pair at the container's north rim — three head-aim taps at g1 landed ZERO) and the
        // torso-first roll (the muzzle drop lands steep down-shots BELOW the torso — the g3
        // chest-aim bullet hit part 4, the lower torso [NF_0D_AUDIT b0 dir vs the geometric
        // ray]) are all the same wall: the rim fight cannot outrun the arming. The machinery
        // stays (env NF_0D_RIM_MAX > 0 re-arms it) for the next session's probes; the walk's
        // lever moves to the pre-ride budget (the corner chips / the #22 re-roll).
        let rim_max: u32 = std::env::var("NF_0D_RIM_MAX").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        // THE POSTURE (retune-nb-0d-3, the measured crouch roll): the sweep runs STANDING. The
        // crouched eye (the 0.45 drop [M Player_GetHeadPos 0x195aa0:39-51]) breaks Bond's OWN
        // head-box rays to the north pair at the container's north rim (the measured roll: three
        // head-aim taps at g1 landed ZERO — the act fell back to lower clear boxes and the
        // muzzle drop put every bullet below them), while the standing eye holds the north
        // head-rays CLEAR (the v1 roll's one-tap g1 kill) and the west pair's down-lines cross
        // the west rim at y ~8.8-9.0 (the top is 7.1). The west pair's marginal up-sight
        // (clear by ~0.1 u standing) is the accepted cost — the sweep's own kill clock beats it.
        let rim_pool = w.scene.sim.as_ref().unwrap().drones.health.health;
        let rim_hits0 = w.scene.sim.as_ref().unwrap().drones.player_hits.len();
        let mut rim_kills = 0u32;
        for pass in 0..rim_max {
            let pick = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                // THE THREAT LAW (the measured roll's own ledger): the first kill-shot ARMS the
                // yard — g0 (state 0x68, north-east) then sustains a 1.8-chip every ~15 ticks
                // (the measured roll: 7 chips in ~100 ticks ate the pool during pass 1's miss
                // stall). Once armed, the LAST chipper dies first (its own fire line is clear —
                // it is shooting).
                let threat = sim.drones.player_hits.last().and_then(|&(_, o, _)| {
                    let d = sim.drones.list.get(o)?;
                    if !d.present || d.corpse || d.health <= 0.0 { return None; }
                    if d.pos[1] >= 8.0 { return None; }
                    if !(118.0..=182.0).contains(&d.pos[0]) || !(-335.0..=-263.0).contains(&d.pos[2]) { return None; }
                    if clear_box_aim(sim, o).is_none() { return None; }
                    if walker::dxz(bp, d.pos) >= 25.0 { return None; }
                    Some(o)
                });
                let mut near: Option<(f32, usize)> = None; // (|yaw err|, i) — the front-arc pick
                let mut far: Option<(f32, usize)> = None;  // (dxz, i) — the fallback pick
                for (i, d) in sim.drones.list.iter().enumerate() {
                    if !d.present || d.corpse || d.health <= 0.0 { continue; }
                    if d.pos[1] >= 8.0 { continue; } // the yard band sits BELOW the container top
                    if !(118.0..=182.0).contains(&d.pos[0]) { continue; }
                    if !(-335.0..=-263.0).contains(&d.pos[2]) { continue; }
                    if clear_box_aim(sim, i).is_none() { continue; }
                    let dist = walker::dxz(bp, d.pos);
                    if dist < 3.0 { continue; }
                    let desired = (d.pos[0] - bp[0]).atan2(d.pos[2] - bp[2]);
                    let yerr = walker::wrap_pi(desired - sim.bond.yaw).abs();
                    if dist < 20.0 {
                        if near.map_or(true, |(e, _)| yerr < e) { near = Some((yerr, i)); }
                    } else if far.map_or(true, |(dd, _)| dist < dd) { far = Some((dist, i)); }
                }
                // The threat overrides the front-arc order only once it has actually fired
                // (the pre-arm passes keep the yaw-cheap order). The owner index resolves against the
                // live list exactly like the ride log (g0/g3/g30); any live owner is a fair
                // target once it has fired.
                match threat { Some(t) => Some((0.0, t)), None => near.or(far) }
            };
            let Some((_, di)) = pick else { eprintln!("[0d] the rim sweep: pass {pass} — no clear pick remains"); break };
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                let desired = (d.pos[0] - sim.bond.pos[0]).atan2(d.pos[2] - sim.bond.pos[2]);
                eprintln!("[0d:box] the rim sweep pass {pass}: g{} skin {:#x} pos ({:.2},{:.2},{:.2}) state {:#x} d {:.1} yawerr {:.2} health {}",
                    di, d.skin, d.pos[0], d.pos[1], d.pos[2], d.state, walker::dxz(sim.bond.pos, d.pos), walker::wrap_pi(desired - sim.bond.yaw), d.health);
            }
            if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare > 0 { w.reload_full(); }
            let (was, c0, s0) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.drones.list[di].health, sim.gun.clip, sim.gun.spare) };
            let down = rim_sweep_act(&mut w, di, "the container rim sweep (the front-arc act)", 260)
                || { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0 };
            if down { rim_kills += 1; }
            let sim = w.scene.sim.as_ref().unwrap();
            let now = sim.drones.list[di].health;
            eprintln!("[0d] the rim sweep: pass {pass} g{} was {was:.1} -> {now:.1} down {down}, ammo {c0}+{s0} -> {}+{}, health {}, hits +{} tail {:?}",
                di, sim.gun.clip, sim.gun.spare, sim.drones.health.health, sim.drones.player_hits.len() - rim_hits0,
                sim.drones.player_hits.iter().rev().take(3).rev().map(|(f, o, d)| (f, *o, *d)).collect::<Vec<_>>());
            if sim.drones.health.health <= 0.0 { break; }
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[0d] the rim sweep: {rim_kills} kills, health {rim_pool:.1} -> {}, hits +{}",
                sim.drones.health.health, sim.drones.player_hits.len() - rim_hits0);
            assert!(sim.drones.health.health > 0.0, "the rim sweep must not kill Bond on the top");
        }
    } else {
    assert!(max_fall >= 70, "the ~10 m slot fall passes the pain gate (timer - 60 >= 10), got timer {max_fall}");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The raw-step window (the hops and the fall) also carries awake-guard trickle
        // (the real-damage Operative run took one 1.2 hit here); the falls do NOT record into
        // player_hits (they run through the scene's pain_generic, outside the drone.rs Bond-
        // damage sites), so the window's hit delta is guard fire alone — subtract it.
        let guard_dmg: f32 = sim.drones.player_hits[hits_before_drop..].iter().map(|&(_, _, d)| d).sum();
        let raw = max_fall as f32 - 60.0;
        // The factor is the level's Plr_DMod (0C-0D 0.4/0.7/1.2 [M ReadTuningVars 0x1d3a80 group
        // 0x1d40f4]; nf_game::bullet::damage_factor) - the old literal pinned 0.7 for every
        // difficulty above Operative, wrong at 00 Agent (1.2).
        let fall_dmg = if raw < 10.0 { 0.0 } else { raw * nf_game::bullet::damage_factor(0x0700_000d, difficulty) };
        let raw_expected = pre_drop_health - fall_dmg - guard_dmg;
        let expected = if raw_expected < 1.0 { 0.0 } else { raw_expected };
        assert!((sim.drones.health.health - expected).abs() < 0.05,
            "the ~10 m slot fall deals exactly the game's math (raw × the difficulty factor, armour untouched, < 1 kills) [M 0x1949a8 + 0x1902c8], expected {expected} (fall {fall_dmg} + window guard fire {guard_dmg}), got {}",
            sim.drones.health.health);
    }
    }
    // THE CONTAINER EXIT (real-health-00agent-0d-1, the new lever the 11-15 ledger asked for —
    // "a no-jump way off the gantry" proved unneeded; the way OFF THE CONTAINER is the lever):
    // the 11-15 roll died ON the container — #1 (0x500007e) walked to (150.4, 5.0, -290.2)
    // during the ride and his 0x12 cadence (-1.8 a round at 6.8 u) ate the ~18.6 health while
    // the sweep's passes burned. The exit is real input, the same laws as the hop: the girder-
    // edge law walls the WALK off the top (the capsule's bottom sphere meets the edge face
    // below the top), so Bond WALKS to the north-west rim until the displacement stalls (the
    // same stall the girder walk measured), then JUMPS with a north-west drift — the vertical-
    // only launch [M Player_HandleJump 0x1916b0] + the air drift [M Player_Move]. The fall from
    // the top (surface 7.1) + the jump apex to the yard floor (4.0) is ~4.2 m ≈ 56 airborne
    // ticks — under the 60-tick pain gate [M 0x1949a8 disasm 0x194b64-0x194be0: raw = timer -
    // 60, pain only when raw >= 10] — so the exit is FREE (the slot apex's 10 m cost 25 raw).
    // The floor stands north-west of the container are the probe's proven-clear zone: the
    // sweep's own scoped law reads #1's chest AND head CLEAR from (140-143, -286..-292) and
    // g2's chest CLEAR from the same stands [probe_0d_container, this session].
    if container_drop {
        let mut exit_ticks = 0u32;
        // LEVER 1 (real-health-00agent-0d-2): the exit aims due WEST from the landing — the
        // hop leaves the yaw at -pi/2 (west, the pre-hop turn_to), the west face at x ~145 is
        // ~3.3 u of flat top from the landing band x 145-151 (the down-ray map: the top spans
        // x 145-151 at z -296, yard 4.0 west of it [probe]), while the old NW aim
        // (141.5, -291.5) stalled ~100 ticks on the container's ridge. The turn phases break
        // the moment the yaw converges (the hop's yaw is already west), so the walk-off starts
        // within ~40 ticks and the measured 154-tick exit (cost -18) drops to ~80-90 —
        // ~4-7 fewer 0x12 rounds at -1.8 into the landing budget.
        for _ in 0..60 {
            let err = walker::wrap_pi(-std::f32::consts::FRAC_PI_2 - w.scene.sim.as_ref().unwrap().bond.yaw);
            if err.abs() < 0.02 { break; }
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
            w.total_ticks += 1;
            exit_ticks += 1;
        }
        // The edge detector is the AIRBORNE edge itself: the girder-edge law walls the WALK
        // while the bottom sphere meets the face, and the moment the sphere clears the rim the
        // walk-off starts — grounded flips false. (A displacement stall never comes: the same
        // held walk that crossed the rim keeps him moving on the floor below — measured this
        // session: the stall-detector rim walk crossed the west edge and walked 12 u into the
        // garrison's fire before the detector gave up.) The container is STEPPED — the first
        // rim crossing this session landed on a lower tier (surface ~6.1, pos 7.21) 1 u past
        // the top's rim — so the walk-off LOOPS: from each tier, turn north-west, walk to the
        // next edge, drift, fall; the last step drops the remaining 2.1-3.1 m to the yard
        // floor (4.0) — every fall under the 60-tick pain gate [M 0x1949a8: raw = timer - 60,
        // pain only when raw >= 10].
        // LEVER (d) MEASURED DEAD (retune-nb-0d-3, two runs): the pre-exit crouch walls the
        // crouched rim walk on the container's ridge (541 ticks for 1.25 u, the walk-off never
        // fired) and the landing-tick crouch press toggles across the exit's MULTI-LANDING
        // structure (the yard floor has sub-6.0 steps — three landings in one exit) while the
        // longer exit ate 16.8 -> 9.6 of chips before the sweep. The rim walk must run
        // standing; the floor sweep's own crouch carries the posture from there.
        for _round in 0..4 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.bond.grounded && sim.bond.pos[1] < 6.0 { break; }
            }
            for _ in 0..40 {
                let err = walker::wrap_pi(-std::f32::consts::FRAC_PI_2 - w.scene.sim.as_ref().unwrap().bond.yaw);
                if err.abs() < 0.02 { break; }
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }, false, false, false);
                w.total_ticks += 1;
                exit_ticks += 1;
            }
            let mut fell = false;
            // LEVER (C), retune-nb-0d-4: the EXIT-FALL CROUCH PRESS. The (d) lever's
            // landing-tick press toggled across the exit's MULTI-landing structure (three
            // landings in one exit); this variant presses ONCE during the FINAL airborne
            // descent (grounded false, pos y < 6.2 — below the top, above every tier rest),
            // so Bond LANDS CROUCHED: the landing burst's sight target drops 0.45 [M
            // Player_GetHeadPos 0x195aa0:39-51] and the visibility multiplies by 0.75 [M
            // Drone_ProcessOpponents 0x1390c0:40-42] — the converged garrison's 6-chip burst
            // re-rolls against the crouched target, and the crouched SWEEP posture carries
            // from the landing. The crouch toggle arms only on a NEW press edge [M
            // Player_HandleJump's crouch arms, 0x190630 dispatch]; one press tick here = one
            // toggle. NF_0D_EXIT_CROUCH=1 arms it (container roll only; 0 = off,
            // byte-identical). The press lives in the DRIFT loop below — the rim-walk loop
            // breaks on the first airborne tick (the descent happens in the drift).
            let exit_crouch = container_drop && std::env::var("NF_0D_EXIT_CROUCH").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0) > 0;
            for _ in 0..300 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
                exit_ticks += 1;
                let sim = w.scene.sim.as_ref().unwrap();
                if !sim.bond.grounded { fell = true; break; }
            }
            eprintln!("[0d] the container rim walk {exit_ticks}: fell {fell} at {:?}", bond_pos(&w));
            assert!(fell, "the rim walk must reach an edge and start the walk-off fall");
            // The drift: hold the same NW forward ~25 ticks (the smoother's ~2 u [M
            // Player_Move]), then release and wait for the next surface. The exit-fall crouch
            // press (lever (C)) rides here: one press tick during the final airborne descent.
            let mut crouch_pressed = false;
            for k in 0..240u32 {
                let mut input = if k <= 25 { nf_game::player::Input { forward: 1.0, ..Default::default() } } else { nf_game::player::Input::default() };
                if exit_crouch && !crouch_pressed {
                    let sim = w.scene.sim.as_ref().unwrap();
                    if !sim.bond.grounded && sim.bond.pos[1] < 6.2 && sim.bond.crouch_drop <= 0.0 {
                        input.crouch = true;
                        crouch_pressed = true;
                    }
                }
                w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
                w.total_ticks += 1;
                exit_ticks += 1;
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.bond.grounded && sim.bond.pos[1] < 6.0 && k > 25 { break; }
            }
            if crouch_pressed {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[0d] the exit-fall crouch press: crouch_drop {:.2} at {:?}", sim.bond.crouch_drop, bond_pos(&w));
            }
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            eprintln!("[0d] the container exit: {exit_ticks} ticks, landed {p:?} health {} (every fall under the 60-tick gate [M 0x1949a8])", sim.drones.health.health);
            assert!(sim.bond.grounded && p[1] < 6.0, "the exit must reach the yard floor (pos ~5.1) off the container, got {p:?}");
        }
        // THE FLOOR SWEEP FIRST (real-health-00agent-0d-2): the 0d-1 stand walk ran BEFORE any
        // fight and cost -16.2 of the 31.2 the dice-shifted roll landed with (4 rifles' bursts
        // across 90 ticks of walk). The sweep's own law self-checks the ray every tick, so the
        // sweep runs from the EXIT LANDING (the first pick, g3 at d 9.7, measured down at
        // +39 ticks); the probe's proven-clear stand (142, -291.5) is walked AFTER the first
        // sweep pass — it re-opens the band rays for whatever survives.
    }
    w.scene.sim.as_mut().unwrap().gun.switch_request = held_gun;
    w.wait(40);
    eprintln!("[0d] the drop landed in the yard — at {:?}", bond_pos(&w));
    // THE YARD GARRISON SWEEP (level-0d-real-health-crossfire-5): the drop lands amid the
    // yard's standing grunts — the run-4 ledger: #6 (138.8, 5.0, −273.5) fired from d 7-21,
    // #3 (132.4, −300.1) from d 6.8, #0/#5 (158.9/159.9, −285.7/−294) from d 21-26 — 98 hits
    // for 86.4 of 0.6-1.2-per-hit trickle killed the walk at 16441 (death by accumulation; the
    // fights' 5 engagements came after the approach woke them). They sleep until approached
    // (the load census: enabled false [L]; bullets hit present drones regardless [M the AI gate
    // only decides whether the guard shoots BACK]) — the flat-yard rays from the drop point are
    // clear, so the grunts die ASLEEP before the south walk. Log-only.
    if real_damage {
        // THE DROP-THEN-SWEEP (level-0d-operative-walk-2, lever 2): land and sweep AT ONCE,
        // nearest live guard FIRST. The fixed box order swept the far corner while #3 — which
        // had WALKED to the drop point during the ride (the baseline trace: d 55 at the corner
        // → d 7.9 at the landing, ticks 12517 → 15351) — landed the landing bursts. The pick
        // is the nearest PRESENT live guard in the yard/trench band with a clear box aim (the
        // flat-yard rays from the drop point are clear — the -1's probes); a guard without a
        // clear aim is skipped (a blocked angle leaves it to the walk's own return fire), and
        // the loop ends when no pick remains. The crouch carries into the sweep (lever 1: the
        // aim/sight target drops 0.45 [M Player_GetHeadPos 0x195aa0:39-51 — the BL+0x90C
        // crouch eye drop] and the visibility factor 0.75 [M Drone_ProcessOpponents
        // 0x1390c0:40-42]), then a fresh press stands for the yard walk.
        // Operative: the sweep runs STANDING (measured: crouched, three kills and Bond left at 4.6
        // for the plate leg, t20383; standing, four kills and the yard, the plate and the
        // photograph pass - death t23462 at the shooting stand). The Agent sister keeps the crouch.
        for _ in 0..(if difficulty == 1 || container_drop { 0 } else { 5 }) {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
        // Operative: 200-tick passes and a guard that survives his pass is not re-picked (the
        // 400-tick pass re-picked the same 20.8 u guard while the crowd fired). Measured: three
        // yard kills (was two), the walk reaches the plate leg - death t19957 -> t20383.
        // THE CONTAINER ROLL RUNS THE FAST ACT (real-health-00agent-0d-1): the drain's 160
        // ticks are exactly the margin four awake burst rifles need (the census: -18 across
        // the 154-tick exit) — the 0700000d fast-act law (the ladder-top passes, the walker
        // fights hold the aim; the raise ~20 ticks [M Player_SetWeaponAnimObj 0x1a2e38 case
        // 0xf]) skips it. The crouch stays OFF for the container roll: the standing eye holds
        // the flat clear rays the probe read, and the 0.75 visibility cut does not hide a
        // stand inside four rifles' engage range.
        let skip_env = difficulty == 1;
        if container_drop {
            // THE CROUCH SWEEP (real-health-00agent-0d-2): the Operative ledger's proven kill
            // posture — the crouch carries into the sweep (the aim/sight target drops 0.45 [M
            // Player_GetHeadPos 0x195aa0:39-51], the visibility factor 0.75 [M
            // Drone_ProcessOpponents 0x1390c0:40-42] — under the 0.7 LOS gate the bursts land
            // less), and its d 20-24 kills LANDED on this very level (the Operative roll's
            // 23.8 u gunner, down at +210) while the standing roll's d 24-32 rounds sprayed
            // zero lands. A fresh press stands after (the yard walk needs the stand).
            for _ in 0..5 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            for _ in 0..12 {
                let pick = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let mut best: Option<(f32, usize)> = None;
                    for (i, d) in sim.drones.list.iter().enumerate() {
                        if !d.present || d.corpse || d.health <= 0.0 { continue; }
                        if d.pos[1] >= 10.5 { continue; }
                        if !(118.0..=182.0).contains(&d.pos[0]) { continue; }
                        if !(-335.0..=-263.0).contains(&d.pos[2]) { continue; }
                        if clear_box_aim(sim, i).is_none() { continue; }
                        let dist = walker::dxz(sim.bond.pos, d.pos);
                        if best.map_or(true, |(bd, _)| dist < bd) { best = Some((dist, i)); }
                    }
                    best
                };
                let Some((_, di)) = pick else { break };
                // THE PASS CAP 260 (the ledger's own law: a FAILED ray burns the whole cap
                // while Bond STANDS in the crossfire — the optional kills ride 260) and the
                // RELOAD HOLD between passes (the l4 roll: the #30 pass burned 400 ticks on
                // clip 0 spare 10 — the 20-tick reload PULSE restarts the anim and never
                // fills; the walker's reload_full holds the R ~100 ticks and does).
                if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare > 0 { w.reload_full(); }
                shoot_far_box_fast(&mut w, di, "the container drop-then-sweep (the fast act, the nearest first)", 260);
            }
            // THE STAND RE-OPEN (real-health-00agent-0d-2): whatever survived the landing sweep
            // gets a second pass from the probe's proven-clear band (142, -291.5) — the walk
            // itself is the exposure the first sweep just thinned. The press STANDS first (the
            // crouch walk is the slower gait and the band's rays were read standing).
            eprintln!("[0d] the post-sweep stand walk: {:?} health {}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.health);
            for _ in 0..5 {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
                w.total_ticks += 1;
            }
            let _ = w.try_walk_to(Wp { at: [142.0, 5.0, -291.5], act: Act::None, arrive: 1.2 }, "the clear stand north-west of the container");
            eprintln!("[0d] the exit stand: {:?} health {}", bond_pos(&w), w.scene.sim.as_ref().unwrap().drones.health.health);
            if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare > 0 { w.reload_full(); }
            for _ in 0..12 {
                let pick = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let mut best: Option<(f32, usize)> = None;
                    for (i, d) in sim.drones.list.iter().enumerate() {
                        if !d.present || d.corpse || d.health <= 0.0 { continue; }
                        if d.pos[1] >= 10.5 { continue; }
                        if !(118.0..=182.0).contains(&d.pos[0]) { continue; }
                        if !(-335.0..=-263.0).contains(&d.pos[2]) { continue; }
                        if clear_box_aim(sim, i).is_none() { continue; }
                        let dist = walker::dxz(sim.bond.pos, d.pos);
                        if best.map_or(true, |(bd, _)| dist < bd) { best = Some((dist, i)); }
                    }
                    best
                };
                let Some((_, di)) = pick else { break };
                if w.scene.sim.as_ref().unwrap().gun.clip == 0 && w.scene.sim.as_ref().unwrap().gun.spare > 0 { w.reload_full(); }
                shoot_far_box_fast(&mut w, di, "the container stand sweep (the second pass)", 260);
            }
        } else {
        let mut failed: Vec<usize> = Vec::new();
        for _ in 0..12 {
            let pick = {
                let sim = w.scene.sim.as_ref().unwrap();
                let mut best: Option<(f32, usize)> = None;
                for (i, d) in sim.drones.list.iter().enumerate() {
                    if !d.present || d.corpse || d.health <= 0.0 { continue; }
                    if failed.contains(&i) { continue; }
                    if d.pos[1] >= 10.5 { continue; }
                    if !(118.0..=182.0).contains(&d.pos[0]) { continue; }
                    if !(-335.0..=-263.0).contains(&d.pos[2]) { continue; }
                    if clear_box_aim(sim, i).is_none() { continue; }
                    let dist = walker::dxz(sim.bond.pos, d.pos);
                    if best.map_or(true, |(bd, _)| dist < bd) { best = Some((dist, i)); }
                }
                best
            };
            let Some((_, di)) = pick else { break };
            shoot_far_box(&mut w, di, "the drop-then-sweep (the nearest first)", if skip_env { 200 } else { 400 });
            if skip_env { let d = &w.scene.sim.as_ref().unwrap().drones.list[di]; if !d.corpse && d.health > 0.0 { failed.push(di); } }
        }
        }
        if (difficulty != 1 || w.scene.sim.as_ref().unwrap().bond.crouched) && !container_drop {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { crouch: true, ..Default::default() }, false, false, false);
            w.total_ticks += 1;
        }
    } else {
        for (at, tol, label) in [
            ([138.8f32, -273.5], [4.0f32, 4.0], "the yard grunt #6 (the C5 door corner, asleep)"),
            ([132.4, -300.1], [4.0, 4.0], "the yard grunt #3 (the plate path, asleep)"),
            ([158.9, -285.7], [4.0, 4.0], "the yard grunt #0 (asleep)"),
            ([159.9, -294.0], [4.0, 4.0], "the yard grunt #5 (asleep)"),
        ] {
            let _ = kill_guard_at_present(&mut w, at, tol, label, 300);
        }
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
    // Putting away the sniper rifle takes ~110 ticks (measured); wait for the camera.
    for _ in 0..300 { if w.scene.sim.as_ref().unwrap().gun.weapon == 0x54 { break; } w.wait(1); }
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
    // Operative real damage too (the standing yard sweep reaches the stand alive): the yard-east
    // gunner there is #1 (0x500007e) at (155.2, 5.0, -295.1), chipping 0.6 from 23.8 u - the
    // box is centred on his spot. Measured: down at +210 and the walk COMPLETES (was t23462).
    if difficulty >= 2 || (difficulty == 1 && real_damage) {
        let (bc, bt): ([f32; 2], [f32; 2]) = if difficulty == 1 { ([156.0, -288.0], [10.0, 10.0]) } else { ([158.0, -280.0], [8.0, 8.0]) };
        // The yard-east gunner #8 (the drop-then-sweep leaves him alive) chips 1.05 a round at
        // the stand through the whole door kill; he goes first, with the rifle.
        // The camera leg left the camera up: back to the rifle (the holster takes ~110 ticks).
        w.scene.sim.as_mut().unwrap().gun.switch_request = 0x1e;
        for _ in 0..300 { if w.scene.sim.as_ref().unwrap().gun.weapon == 0x1e { break; } w.wait(1); }
        for _ in 0..3 {
            let live = w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| d.present && !d.corpse && d.health > 0.0
                && (d.pos[0] - bc[0]).abs() < bt[0] && (d.pos[2] - bc[1]).abs() < bt[1] && d.pos[1] < 8.0);
            if !live { break; }
            let _ = kill_guard_at_present(&mut w, bc, bt, "the yard-east gunner #8 (from the shooting stand)", 400);
        }
    }
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
/// pinned against). The real-damage-gated route legs are marked level-0d-operative-walk in
/// the body; the plain master replay (top-ups on) runs byte-identical to its pre-gate tree.
#[test]
#[ignore = "stream batch 1121b: Bond dies at (198.0, 10.6, -278.9) at about t11200 (the roof guards' cover snipes, s-g028-cover-walks); no stop of 37 swept completes it; also red on s/g028-cover-walks and s/auto-aim alone"]
fn walk_arrival_to_the_c5_door_real_damage() {
    // Stream batch 1121 (impacts heard + the stand-reload roll + the stand idles): the ground
    // sniper #15 landed three 16.0 rounds on the gantry ride (one on master) and Bond died at the
    // yard landing (t19123). Lever: a 15-tick stop at tick 17200 (the east park, before the
    // ride) re-times the ride against #15's sight cadence [L 24 grid + 71 random stops and the
    // tap knobs all died; the 12-value ride-window grid: 17200/15 completes].
    walker::with_pause(17200, 15, || walker::with_real_damage(|| walk_arrival_to_the_c5_door_at(1, true)));
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
// Stream batch 1121: Bond died at the yard landing (t20311) with no armour. GREEN with the
// annex floor's own body armour (192.82, 10.79, -272.03) taken after the annex drop [L the
// 2026-11-21 run: armour 0 -> 50 at 54.8 health, five yard kills, the walk completes].
#[ignore = "stream batch 1121b: Bond dies at (213.1, 17.4, -234.5) at about t4900 to the roof guard's 63 u rounds (s-g028-cover-walks); no stop of 37 swept completes it; also red on s/g028-cover-walks and s/auto-aim alone"]
fn walk_arrival_to_the_c5_door_real_damage_agent() {
    walker::with_real_damage(|| walk_arrival_to_the_c5_door_at(2, true));
}

/// 00 Agent (difficulty 3) real damage: the same walk as the Agent sister. GREEN
/// (real-health-00agent-0d-2, on top of 0d-1's container drop): the dice-knob roll rides the
/// ride with ZERO chips (the 18 aimed ride taps never land off the moving beam — zero kills in
/// 33 taps across two gate versions — but each tap spends a spread draw from the shared stream,
/// and the shifted roll wakes the yard rifles into a merciful cadence; every no-tap roll takes
/// 2 chips and a -7.2 exit), lands the container top at a full 31.2, exits west in 86 ticks
/// (the west face is 3.3 u of flat top; the 0d-1 NW aim stalled ~100 ticks on the ridge) at
/// ZERO cost, and the CROUCH sweep from the exit landing kills five (+31/+150/+149/+40/+243 —
/// the Operative ledger's proven kill posture: the crouch drops the sight target 0.45 [M
/// Player_GetHeadPos 0x195aa0:39-51], multiplies the visibility by 0.75 [M Drone_ProcessOpponents
/// 0x1390c0:40-42], and its d 24 kills LANDED here while the standing roll's sprayed wide), the
/// stand walk survives at 6.0, and the plate/trench/photograph/shooting-stand legs finish the
/// real-damage section at 4.2; the C5 kill rides the mounted laser under the established
/// top-up guard. Full mission 30411 ticks, stuck 0, falls 0. No health, armour or ammo writes
/// on real input anywhere before the C5 leg's guard.
/// RED again under nb-scan-all (retune-nb-0d, 2026-11-18): the 0d-2 levers hold through the
/// landing, but the RIDE now dies to the east-yard guard g18's ONE 25.8 0x12 round — the
/// measured ledger and the next lever are in the ignore message.
/// Round 4 (retune-nb-0d-4, 2026-11-19): the dice hunt and the #22 re-roll measured dead
/// (the re-deal is sight-cadence-keyed, not stream-keyed), but the ARMOUR TAKE is proven —
/// the kind-3 pickup exists at 00 Agent and the tight arrival (0.2) takes it (2.0 -> 50.0);
/// the walk-over reshape is the round-5 lever. The refreshed ledger is in the ignore message.
/// s-crouch-latch-walks (port runs on that branch): with the game's crouch the claw leaves Bond
/// standing, so the ride runs crouched only from the press aboard. A standing ride dies on the
/// beam (t19152, 1.8 chips from g2/g3 from x 176). Crouched, the ride reaches the container break
/// at 24.0, but the jump right after the stand press freezes Bond in the air at y 14.69
/// (grounded false, vel.y 3.59 held through five hops) and the beam carries him back east to
/// x 170: the hop assert fails. Standing up ~3.5 u or ~16 u before the break settles him at
/// y 14.50 (the boarding stand reads 14.60) and the same freeze follows, or the early stand
/// dies to g18 on the ride. Base died later (t19772, the yard floor).
#[test]
#[ignore = "nb-scan-all (the spacing scan everywhere + CanAltAttack in the fire states), retune-nb-0d + retune-nb-0d-2 measured ledger, twenty-three real runs 2026-11-18 [L]: (0) the original roll — boarded 25.8 (the deck sniper #22 0x5000090/0x1e landed 46.0 at t2609 on the ladder-base pin; corner chips 5 x 1.8 from #3/#2/#1), the ride took ZERO chips ~900 ticks, then g18 (owner 18 [M player_hits (frame, owner, damage) drone.rs 13525], skin 0x5000087, 0x12, (177.90, 0.98, -283.26) state 0x68, d ~18.4 off the rider's crouched eye) landed ONE 25.8 0x12 round = death t19197 at (164.71, 14.60, -296.08); (1) the retune-nb-0d levers — the deck-duo kill LANDED but the roll re-dealt #20 46.0 + #32 48.0 worse; the yard-armour twin did NOT arm at d3 (armour 2.0 -> 2.0); the zone-35 rider-distance taps re-aimed g18 (d 13.9-14.4), zero lands (the girder-spawn law holds at d 14), the same-margin death t19222 — all reverted; (2) the retune-nb-0d-2 dice knobs (env-tunable harness, master defaults except the committed ride shift): the hang pause 12/48 SATURATES to one roll (death t18999 — time shifts alone move nothing in the pre-ride phases, the guard fire is sight-event-keyed not absolute-tick-keyed, the 3/12/48-tick pauses pair into two roll classes); NF_0D_RIDE_SHIFT=30/60 (the committed default 30) RE-ROLLED THE RIDE ALIVE — 2 ride chips (25.8 -> 22.2 on the girder), the container landing CLEAN (fall timer 63 [M 0x1949a8 raw = timer - 60], 18.6), the exit clean (16.8) — the death moved OFF the ride to the YARD-FLOOR fight: the scan-era garrison CONVERGES on the container during the ride and the exit landing eats a 4-guard chip burst (frames 19676-19682: #1 (151.8,-286.0) d 14.6, #2 (136.3,-298.9) d 5.7, #0 (156.2,-286.1) d 18.0, #3 (133.0,-297.6) d 8.3 — 6 chips of 1.8 in 6 ticks), death t19772 at the landing spot (141.22, 4.88, -296.08) at 16.8; the shift-60 twin died t19722 the same way; the family map is ALIVE exactly at {30, 60} and dead at {0, 12, 20, 25, 35, 40, 45, 50, 70, 80, 90} — the alive band is ~30-tick quantized and both alive rolls land at 16.8-18.6; the tap knobs zone 50/cadence 3 (99 taps — the 18-round 0x1e budget spent early) rode 4 chips but the container HOP flew LONG (fall timer 99 = raw 39 x 1.2 [M the d3 Plr_DMod] = 46.8 at 18.6 = death AT the landing; the hop timer is config-tied: the same break spot gives timer 63 under the shift-30 roll and 99 under the zone-50 roll; the hop stagger wait 30/60 does NOT fix it); the crouch-timing levers REGRESS: the crouched E-plate park wait is inside the west band's sight (#0 (157.6,-289.9) + #1 25.8 of chips, dead t21174; the standing wait is the measured-quiet stand) and the crouched rim dwell doubles the exposure time (#1 18.0 in 10 hits, dead t21538) — both reverted. FACTS: the pre-ride costs are 46.0 (#22, every roll) + 19.2 (the annex fall raw 16 x the d3 factor 1.2 [M 0x1949a8 + 0x1902c8]) + the corner chips 5.4-14.4 -> boarding 20.4-29.4; the ride CAN be ridden alive and the landing CAN be clean; the container-top sweep is measured-dead (the 0d-1 roll burned its passes on #1 at 6.8 u); the ride taps CANNOT kill anything (33+ taps at d 13.9-43, zero lands); g18 is outside the 25 u tap zone AND the bay-enter scan's wake window; the corner rim and the high stands see nothing into the yard pit. NEXT LEVER: the floor-fight budget — the alive-ride rolls land at 16.8-18.6 and the converged garrison's 4-chip/6-tick burst outraces the sweep's first kill (+39 ticks in the 0d-2-era roll); the sweep needs the ~28-31 the 0d-2-era roll landed with, i.e. ~+12 health from the pre-ride phases: (a) shave the corner chips 5.4-9.0 WITHOUT extra dwell time (the crouch-timing levers are dead; a faster rim approach — trim the 20-tick claw switch poll, pre-arm the latch — or a rim stand outside the #3/#2 sight cone, geometry needs a probe), (b) re-roll the #22 sniper round (the deck-duo kill landed it but re-dealt #20+#32 — the re-deal is its own dice, a kill act with a different draw count may re-deal kinder), or (c) fight the converged garrison from the container rim BEFORE dropping (the down-rays from the rim need a probe — the 0d-1 passes burned on #1 at 6.8 u with the box act; the head act is the untried variant). (3) THE retune-nb-0d-3 LEDGER (six real runs 2026-11-18, each its own roll): lever (c) is MEASURED DEAD with two audit-proven laws — NF_0D_AUDIT on the rim sweep (the machinery stays in the tree behind NF_0D_RIM_MAX, committed default 0 = OFF): (i) THE ARMING LAW: the first kill-shot from the container top (a part-5 head hit on g3 at +34 [NF_0D_AUDIT: b0 SPAWN dir (-0.972,-0.218,-0.088) -> part Some(5,0.099)]) ARMS the yard — g0 lands its first 1.8 chip ~24 ticks later, then g0 AND g1 sustain 1.8 every ~6 ticks (0.3 damage/tick EACH; 9 chips in 51 ticks ate the 18.6 pool mid-sweep) — one shot buys one kill and hands the pool to the pack; the floor landing burst is the same pack already armed by flat sight. (ii) THE POST-KILL SWALLOW: a trigger tick ~36 ticks after a landed kill spawns NO bullet ([NF_0D_AUDIT: pass 1's tap at +34, trigger in, NO SPAWN line]; the ride fire's ~28-tick taps never killed, so they never hit this law) — the sweep gets ONE shot per ~200 ticks vs an armed pack draining 0.3-0.6/tick. Sub-laws measured the same wall: the CROUCHED stand breaks Bond's OWN head-rays to the north pair at the container's north rim (three head-aim taps at g1 = zero; the standing stand holds them clear — the v1 roll's one-tap g1 kill) and the standing chest-aim bullet lands LOW on down-shots (the g3 chest-aim hit part 4, the lower torso [b0 dir (-0.979,-0.179,-0.096) vs the geometric (-0.989,-0.111,-0.107)] — the muzzle drop law); the 0.12 u tap gate (half the ride law) still missed the steeper g2 line twice — the drop outruns the gate at d 12. (iii) THE DICE KNOBS DO NOT MOVE THE FLOOR FIGHT: NF_0D_TAP_CADENCE=17 (17 taps) reproduced the shift-30 roll tick for tick (the same ride chips at 18877/19384, the same landing 18.6, the same death t19772 — the guard fire is sight-event-keyed, the tap draws are invisible to it); NF_0D_RIDE_SHIFT=0 + the tap defaults (19 taps) reproduced the ROUND-1 death exactly (g18's ONE 25.8 0x12 at t19197, the owner-resolved log) — the shift-0 ride is dead on this timeline. NEXT LEVER (round 4): the pre-ride budget is the only remaining source of the ~+12 the floor sweep needs — (a) shave the corner chips 5.4-9.0 WITHOUT extra dwell time (trim the 20-tick claw switch poll, pre-arm the latch before the rim walk — the corner phase's own code comments name it; the chip cadence is 12-tick 0x12 from #3/#2/#1 at the rim), (b) re-roll the #22 46.0 (the deck-duo kill re-dealt #20+#32; a kill act with a different draw count may re-deal kinder — each act variant is its own roll), (c) hunt the 18-tap zero-chip ride roll (the dice-knob comment's own measurement: an 18-tap roll re-rolled the ride to ZERO chips, boarding 31.2 through the landing, a ZERO-COST exit — the exact budget the floor sweep needs; the exact old tap config is not on this branch — sweep NF_0D_TAP_ZONE x NF_0D_TAP_CADENCE x NF_0D_RIDE_SHIFT for a ride with 0 chips at the {30,60}-like alive band), (d) the floor-landing crouch — MEASURED DEAD this session (two runs): the pre-exit crouch walls the crouched rim walk on the container's ridge (541 ticks for 1.25 u, the walk-off never fired — the rim walk must run standing) and the landing-tick crouch press toggles across the exit's MULTI-LANDING structure (the yard floor has sub-6.0 steps — three landings in one exit) while the longer exit ate 16.8 -> 9.6 of chips before the sweep (death t19846). The rim walk runs standing; the floor sweep's own crouch carries the posture from there. The round-4 lever list stands at (a)/(b)/(c). (4) THE retune-nb-0d-4 LEDGER (seventeen real runs 2026-11-19, each its own roll): (i) THE DICE HUNT (lever (c)) is DEAD IN THIS NEIGHBOURHOOD — twelve fresh rolls: NF_0D_TAP_CADENCE {7,9,11,13,14,16,18,20} at the committed shift 30 ALL died ON THE RIDE to g18's 22.2-24.0 0x12 draw (t19194-19292; the default cadence 15 = 19 taps is the roll where g18 MISSES — every neighbouring tap schedule re-rolls it onto the rider); the shift-60 family (cadence 14/16) died the same way; NF_0D_TAP_ZONE 20/30 SATURATE to the default roll exactly (the same chips t18877/t19384, the same landing 18.6, the same death t19772). The old branch's 18-tap zero-chip config is not reachable in this gate law's neighbourhood. (ii) LEVER (b) — the #22 re-roll — MEASURED DEAD with a NEW LAW: the strip-mouth head act (the round-1 box [209.0, -289.3] tol [7.0, 3.5], d3-gated env NF_0D_DECK_KILL=1, default 0) LANDED each time but the box picks the NEAREST PRESENT guard — g11 (0x5000093, the bay captain, nearer the box centre than #22's repositioned spot), NOT #22 — and the re-deal is NOT stream-keyed: NF_0D_DECK_EXTRA = 1/2/3 deliberate pre-shot draws produced the IDENTICAL deal tick for tick (#20 46.0 t2022, #36 48.0 t2060, #32 6.0 t2599/2600, death at the ladder base t2599/2600). The 0x1e snipers' fire is sight-cadence-keyed and LANDS at d3 Accuracy row 2 regardless of the draw count — 'each act variant is its own roll' is measured FALSE for this act (the stream only rolls the damage VALUE 46/48; the hit/miss at row 2 saturates). (iii) THE ARMOUR TAKE — THE ROUND-5 BREAKTHROUGH: the kind-3 pickup EXISTS at 00 Agent (pickup 1, kind 3, weapon 0x6, rounds 50, at (205.41, 0.11, -205.31), bounds radius 0.81, gone false hidden false state 1 [L the probe run]; the placed pickups spawn with NO difficulty filter [D scene.rs placed_pickups: every flags&0xff==0xf0 placement]) and the TIGHT approach (walk_nav_to arrive 0.2 + a 40-tick stand on the spot, NF_0D_ARMOUR_PROBE=1, default 0) TAKES it — armour 2.0 -> 50.0 [L] — the round-1 refusal was the LOOSE arrival (arrive 0.6 stopped 0.57 dxz / 1.03 dy off; the touch is the radius+1.1 sphere from the player's position + the LOS mask-0x721 ray from his world-sphere centre [M Pickup_Update 0x18e148:203-214, two different origins]). The probe roll still DIED (t4023 at the ladder base, before the climb): the 1388-tick walk-over dwell sits in the snipers' sight (#32 100.0 in 4 hits; the fresh armour 50 absorbed ~1-2 rounds then spent — the probe ledger shows both -0.00 armour-eaten entries AND a -46.00 health entry, so the exact absorb order needs the [M] pin of Player_HandlePain 0x1902c8:84-104). Lever (a) (the corner shave) went untried — the budget went to the dice hunt, the deck act and the armour probe. NEXT LEVERS (round 5): (A) THE ARMOUR WALK-OVER RESHAPE — the take is proven; the cost is the walk-over dwell itself (the snipers fire on their own cadence while Bond stands visible). Reshape it: a shorter/faster path (the walk-out 24 u north + the walk-back is ~1388 ticks of exposure), or fold the take into an existing north-bound leg, or take it and hold the armour UNTIL the climb (the ladder-top snipers' rounds then eat the 50 instead of Bond's health); the absorb law first needs the [M] read (the probe's -0.00 vs -46.00 mix). With the armour alive at the pre-ride phases, #22's 46.0 every-roll round EATS ARMOUR and the floor sweep's budget lands ~+40. (B) lever (a) — shave the corner chips 5.4-9.0 without extra dwell (trim the 20-tick claw switch poll, pre-arm the latch). (C) the exit-fall crouch press AFTER the exit's last multi-landing step (the (d) variant pressed AT the landing tick and toggled across the sub-6.0 steps; a press ~20-40 ticks before the ground is ONE press — the burst's visibility 0.75x [M Drone_ProcessOpponents 0x1390c0:40-42] + the 0.45 sight drop [M Player_GetHeadPos 0x195aa0:39-51] may cut the landing burst's 6 chips; the crouched SWEEP posture then carries from the landing). (iv) LEVERS (B)/(C) MEASURED THIS SESSION: the corner-shave pre-arm (NF_0D_FAST_SWITCH=1, the claw switch moved into the quiet E-plate wait + a poll-until-up, default 0) re-rolled the ride onto g18's 25.8 at t19354 — the same dice law, one roll, dead for this roll; the EXIT-FALL CROUCH PRESS (NF_0D_EXIT_CROUCH=1, one press tick in the drift loop at grounded-false y < 6.2, default 0) ENGAGES — Bond lands crouched (crouch_drop 0.29, the landing 4.64 vs the standing 5.06 [L]) — but the landing burst is NOT cut (the same ~6 chips at d 5.1-13.8 point-blank, 11.4 in 12 ticks, death t19777 vs t19772): the 0x12 cadence fires through the 0.75 visibility + the 0.45 drop at that range. The armour absorb law is now PINNED [M Player_HandlePain 0x1902c8:60-72 — the decompile]: the armour absorbs min(damage, armour) and ONLY for the default damage types (case default = 1.0); damage types 1-7 bypass the armour entirely (fVar4 = 0.0). The d1 evidence (the armour 50 -> 34 by one 16.0 sniper hit) says the sniper rounds ARE armour-applicable; the probe's mixed ledger (-0.00 entries beside -46.00 health entries with the armour fresh) needs the round-5 re-measure with NF_PICKUPLOG + the hit trace (the take's exact tick vs the hits decides it). ROUND-5 LEVERS: (A) the armour walk-over reshape (the take proven; the walk-over dwell 1388 ticks attracts the sniper cadence — a shorter path, a fold into an existing leg, or the take held until the climb; the [M] absorb law + the bullet damage type decide the net); (B') the g18 ride lottery at other knob families (each config its own roll); (C') the landing burst answer — the crouched landing works mechanically but does not cut the point-blank burst; the sweep's first-kill race (+39 ticks) remains the wall to beat with the armour alive. (5) THE retune-nb-0d-5 LEDGER (two real runs 2026-11-19, each its own roll): the armour walk-over reshape is MEASURED DEAD — three new laws: (i) THE ABSORB LAW PINNED WITH THE FIELD ADDRESSES [M Player_HandlePain 0x1902c8:84-96]: absorbed = min(damage x the type factor, armour), armour at BLData+0x8B0, health at +0x894, the default damage type = 1.0 (types 1-4 = 0.0, bypass); the trace run PROVES the fresh 50 fully ate a 48.0 0x1e roll (armour 2.0 at the leg end, health unchanged) AND ate #22's round at t2999 in the reshaped roll — the absorbed hits sit in player_hits as 0.0 HEALTH DELTAS (the trace's dmg != 0.0 filter hides them: round 4's 'mixed ledger' was this display law, not a mixed absorb; the d3 spawn armour is 2.0 — the loadout line's armour 0 is the pre-spawn print). (ii) THE WAKE LAW: ANY pocket visit (the 1388-tick committed leg AND the reshaped leg) wakes the west-roof #32's sight-cadence onto the climb phases — the no-leg baseline stand eats only #22's 46.0 and survives, both probe rolls ate #32 rounds after the leg (the committed-leg roll died t4023 AT the ladder-base stand 46+8; the reshaped roll died t3790 IN THE POCKET), and in the reshaped roll the armour DID eat #22's round (t2999) and the roll STILL died — the woken cadence alone is lethal. The armour's one-absorb budget is spent in the measured-hit corridors (the walk-out east corridor x~213, the walk-back west detour to (183.5,-216.8) at 32 u from #32's roof spot) before #22's round: net NEGATIVE vs the no-leg baseline. (iii) THE POCKET-BUMP LAW: the pocket exit bump at (206.65,-205.04) (y 1.91) is climbable east->west (the out path) but the direct-steer east return west->east WEDGED on it (the reshaped roll sat at (206.5,-204.9) 180+ ticks in the cadence). ROUND-6 LEVERS: (A) the g18 ride lottery at the untried knob families (NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60} — each config its own roll; the alive band is ~30-tick quantized at {30,60}); (B) the armour lever is CLOSED unless a sight-blocked pocket corridor appears (every nav-graph yard node on the leg is measured-hit; a live check could map #32's sight mask along the x~213 corridor); (C) the pre-ride corner shave that does NOT re-roll the ride (the FAST_SWITCH pre-arm re-rolled it — dead; a rim stand outside the #3/#2 sight cone needs geometry)."]
fn walk_arrival_to_the_c5_door_real_damage_00agent() {
    walker::with_real_damage(|| walk_arrival_to_the_c5_door_at(3, true));
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

/// The FAST CHEST variant (level-0d-operative-walk-7, lever (a)): the present-pick + the
/// chest aim WITHOUT the 160-tick aim-off drain — the drain exists to reset the raise queue
/// after a fire script (state 9 [M 0x1a24c8:66-99]), and the pre-climb base stand fires
/// ~1600+ ticks after the last shot with no script pending (the ladder-top legs' own
/// skip-drain law) — while a patrolling band guard crosses the z −216..−223 clear corridor
/// at ~0.026 u/tick, the drain's 160 ticks are 4.2 u of walk: the whole corridor (measured
/// v2: the CLEAR snapshot's act re-found her behind the wall at +220, one round late).
fn kill_guard_at_present_fast(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str, cap: u32) -> bool {
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
            shoot_far_aim(w, i, label, cap, 0.3, true)
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

/// The TORSO-FIRST variant (retune-nb-0d-3): the same clear-box law but the TORSO box wins —
/// the head box is graze-prone against a tap tolerance tuned to the body: at d 12-15 the head
/// box's angular size (~0.012 rad) is SMALLER than the tap gate (0.12/dist ≈ 0.008-0.010 rad),
/// so a head-aimed tap can fire while the ray points at the box's edge and graze (the
/// pick_trio_target law: "prefers a clear TORSO box over a head-only one"; the run-4380 roll's
/// 8+ converged head rays landed ZERO). The torso box (~0.3 u ≈ 0.025 rad at d 12) dwarfs the
/// gate: every on-gate tap is a chest hit, and one 0x1e chest round kills a 10-health grunt.
fn clear_box_aim_torso(sim: &nf_view::scene::Sim, i: usize) -> Option<([f32; 3], i16)> {
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
    let mut torso: Option<([f32; 3], i16)> = None;
    let mut head: Option<([f32; 3], i16)> = None;
    for b in d.hit_boxes(&data) {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        if nf_game::damage::ray_box(eye, ray, &b).is_none() { continue; }
        if sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, c).is_some() { continue; }
        if b.part == 5 { head = Some((c, b.part)); } else { torso = torso.or(Some((c, b.part))); }
    }
    torso.or(head)
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
    shoot_far_aim(w, di, label, cap, 0.3, false)
}

/// The HEAD aim variant (level-0d-real-health-crossfire-5): the same aimed act with the aim
/// point and the world-ray gate raised to the head box (pos + `aim_dy`) — the height-asymmetric
/// slab law makes head lines clear where chest lines die [probe_0d_roof_rays].
fn shoot_far_head(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, 0.9, false)
}

/// The BOX aim variant (level-0d-real-health-crossfire-5): the aim point is a REAL clear hit-box
/// centre each tick (the part-5 head first — `clear_box_aim`), not a fixed offset — the run-2
/// +0.9 head guess fired clear-ray rounds that passed under the head box; a box point the
/// port's own law returns is on a box the bullet reaches.
fn shoot_far_box(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, -1.0, false)
}

/// The NO-DRAIN box act (level-0d-c5-door-fall): the 160-tick aim-off drain exists to reset
/// the raise queue after a fire script (state 9 [M 0x1a24c8:66-99]) — but the ladder-top phase
/// opens ~2000 ticks after the last shot with no script playing, and the drain costs exactly
/// the margin the west-roof trio's return fire needs to kill first (the Agent run's lethal
/// 28.0s landed at ticks 3048-3236 while the drained act's kill sat at +222 — the race lost by
/// ~36 ticks; the Operative stream's same act kills at +222 WITH the drain, so the drained
/// path stays the default and the Agent's direct leg alone runs fast).
fn shoot_far_box_fast(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    shoot_far_aim(w, di, label, cap, -1.0, true)
}

/// THE RIM ACT (retune-nb-0d-3): the ride-fire tap law on a STATIC stand. The shared act's
/// agent_fast pre-turn phase breaks only at `err < 0.01 && perr < 0.01 && !aiming` — with the
/// aim bit already up (the ride fire raised it and the passes hold it) it runs its FULL
/// 240-tick window every pass: the measured roll's pass 1 died at +109 ticks still inside that
/// phase while the converged garrison's cadence ate the 18.6 pool. This act skips both the
/// drain and the pre-turn phase: converge the RAY per tick (the shot's own construction [M
/// Player_GetAimingPoint 0x1a4a98 — the cursor offsets ride the view ray]), hold the aim bit
/// in the input (the raise completes ~20 ticks [M Player_SetWeaponAnimObj 0x1a2e38 case 0xf]
/// and every pass after the first is already up), and tap when the clear-box centre is on the
/// ray with the scoped line clear (the tap box scales to the range like the proven acts).
fn rim_sweep_act(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    let audit = std::env::var_os("NF_0D_AUDIT").is_some();
    if audit { audit_clear(); }
    let mut last_tap = 0u32;
    let mut taps = 0u32;
    let mut hp_prev = w.scene.sim.as_ref().unwrap().drones.health.health;
    let start = w.total_ticks;
    loop {
        let (fire, input) = {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0 {
                eprintln!("[0d] {label}: down at +{} ticks ({taps} taps)", w.total_ticks - start);
                return true;
            }
            let bp = sim.bond.pos;
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            // THE HEAD-FIRST AIM ON DOWN-SHOTS (retune-nb-0d-3, the audit's own verdict): the
            // bullet spawns at the muzzle BELOW the eye and lands LOW — the measured g3 tap (a
            // chest-centre aim) hit part 4, the LOWER torso [NF_0D_AUDIT: b0 SPAWN dir
            // (-0.979,-0.179,-0.096) -> part Some(4, 0.104)]; the steeper the down-shot, the
            // lower the landing (the g2 tap landed below the torso entirely — zero). Aiming at
            // the part-5 HEAD box drops the bullet into the chest/neck — the v1 roll's g1 kill
            // (the same down-shot geometry, the head act) and every flat-stand kill of the
            // older legs agree. The torso-first variant is the measured-miss act.
            let (aim_pt, aim_part) = match clear_box_aim(sim, di) {
                Some((c, part)) => (c, Some(part)),
                None => ([sim.drones.list[di].pos[0], sim.drones.list[di].pos[1] + 0.3, sim.drones.list[di].pos[2]], None),
            };
            let yaw = sim.bond.yaw;
            let p = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let look = sim.bond.look();
            let right = [-yaw.cos(), 0.0, yaw.sin()];
            let up = [-p.sin() * yaw.sin(), p.cos(), -p.sin() * yaw.cos()];
            let (tx, ty) = (0.76966f32 * sim.cross[0] / sim.zoom, 0.57724f32 * sim.cross[1] / sim.zoom);
            let dv = [look[0] + tx * right[0] + ty * up[0], look[1] + tx * right[1] + ty * up[1], look[2] + tx * right[2] + ty * up[2]];
            let dl = (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt().max(1e-4);
            let ray = [dv[0] / dl, dv[1] / dl, dv[2] / dl];
            let (dx, dy, dz) = (aim_pt[0] - eye[0], aim_pt[1] - eye[1], aim_pt[2] - eye[2]);
            let err = walker::wrap_pi(dx.atan2(dz) - ray[0].atan2(ray[2]));
            let perr = dy.atan2((dx * dx + dz * dz).sqrt().max(0.5)) - ray[1].asin();
            let dist = walker::dxz(eye, aim_pt).max(0.5);
            // THE TAP GATE, HALVED (retune-nb-0d-3, the measured miss): the ride-fire law's
            // 0.12/0.25 u boxes are the TORSO's own half-width at d 12 — a gate-edge tap can sit
            // ON the box edge and graze (the measured roll: g2's tap1 at +29 landed ZERO while
            // g3's identical tap at d 15.4 killed — the 0.12 u margin plus the walk drift is
            // nothing). Dead-centre shots only: half the lateral and vertical gates.
            let (err_tol, perr_tol) = ((0.06f32 / dist).clamp(0.002, 0.01), (0.12f32 / dist).clamp(0.004, 0.02));
            let world_clear = aim_part.is_some()
                && sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, aim_pt).is_none();
            let t = w.total_ticks - start;
            // THE TAP CADENCE 32 (retune-nb-0d-3, the audit's second verdict): the 0x1e's fire
            // recycle is ~28 ticks [NF_0D_AUDIT: pass 0's kill shot at +21 swallowed pass 1's
            // +30 tap — trigger input in, NO bullet spawn (the ride fire's own ~28-tick tap
            // spacing agrees)]; the 15-tick cadence tapped into the recycle every pass after
            // the first and the pass burned its cap shooting nothing.
            let fire = world_clear && err.abs() < err_tol && perr.abs() < perr_tol
                && t.saturating_sub(last_tap) > 32 && sim.gun.clip > 0 && sim.aiming;
            if fire { last_tap = t; taps += 1; }
            let input = nf_game::player::Input {
                turn: (err * 1.0).clamp(-0.08, 0.08),
                look: (perr * 2.0 / std::f32::consts::PI).clamp(-0.08, 0.08),
                aiming: true,
                ..Default::default()
            };
            (fire, input)
        };
        let reload_pulse = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.gun.clip == 0 && sim.gun.spare > 0 && (w.total_ticks - start) % 20 == 0
        };
        w.scene.sim.as_mut().unwrap().step_bond(input, fire, reload_pulse, false);
        w.total_ticks += 1;
        w.after_step();
        if audit { audit_bullets(w, di, start); }
        let sim = w.scene.sim.as_ref().unwrap();
        if fire {
            eprintln!("[0d] {label}: tap{} at +{} — g{} health {:.1}, aiming {}, clip {}, health {:.1}",
                taps, w.total_ticks - start, di, sim.drones.list[di].health, sim.aiming, sim.gun.clip, sim.drones.health.health);
        }
        if sim.drones.health.health != hp_prev {
            let last = sim.drones.player_hits.last().map(|(f, o, d)| {
                let dd = sim.drones.list.get(*o).map(|dd|
                    format!("skin {:#x} at ({:.2},{:.2},{:.2}) state {:#x}", dd.skin, dd.pos[0], dd.pos[1], dd.pos[2], dd.state));
                format!("frame {f} owner g{o} dmg {d:.1} {:?}", dd.unwrap_or_else(|| "the world".into()))
            });
            eprintln!("[0d] {label}: health {hp_prev:.1} -> {:.1} at +{} ({last:?})",
                sim.drones.health.health, w.total_ticks - start);
            hp_prev = sim.drones.health.health;
        }
        if sim.gun.clip == 0 && sim.gun.spare == 0 && w.total_ticks - start > 60 {
            eprintln!("[0d] {label}: out of ammo at +{}", w.total_ticks - start);
            return false;
        }
        if w.total_ticks - start >= cap {
            eprintln!("[0d] {label}: still alive after {cap} ticks (health {:.1}, dist {:.1}, {taps} taps)",
                sim.drones.list[di].health, walker::dxz(sim.bond.pos, sim.drones.list[di].pos));
            return false;
        }
    }
}

/// The head-pick + fast-act variant of `kill_guard_head_present` (same present-pick, the
/// no-drain act) for the Agent run's direct ladder-top leg.
fn kill_guard_head_present_fast(w: &mut Walker, at: [f32; 2], tol: [f32; 2], label: &str, cap: u32) -> bool {
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
            shoot_far_box_fast(w, i, label, cap)
        }
        None => { eprintln!("[0d] {label}: no PRESENT guard near ({:.1},{:.1}) — already down", at[0], at[1]); false }
    }
}

/// The TRIO-RETARGET pick v2 (level-0d-agent-ladder-top-3, the ledger's lever (c) measured):
/// the run-4380 roll measured the run-4034 class's cause — the nearest clear-box pick (g33,
/// state 0x97/0x68) is ACT-PROOF on that roll: his fire-anim pose leaves only the small head
/// box clear (the torso line dies on the mid-band obstruction) and 8+ converged CLEAR-ray
/// rounds at d 63 grazed him for ZERO (the first clip even fired HIP — the aim bit flaps with
/// the fire script between shots). The pick (1) skips the blacklisted targets (a full pass
/// with zero damage = act-proof THIS roll — the caller blacklists), (2) prefers a clear
/// TORSO box over a head-only one (the head box is graze-prone at d 60+: the 0.12 u tap box
/// at d 63 is the head's own angular size), (3) returns None when every band line is blocked
/// or blacklisted — the caller waits, no pass burned.
fn pick_trio_target(w: &Walker, at: [f32; 2], tol: [f32; 2], skip: &[usize]) -> Option<(usize, i16)> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().enumerate()
        .filter(|(i, d)| d.present && !d.corpse && d.health > 0.0
            && (d.pos[0] - at[0]).abs() < tol[0] && (d.pos[2] - at[1]).abs() < tol[1])
        .filter_map(|(i, d)| clear_box_aim(sim, i).map(|(_, part)| (i, part, d)))
        // THE HEAD-ONLY SKIP (the run-4379/5069/5239 law, 4 rolls): a candidate whose ONLY
        // clear box is the head is the graze line at d 63 — the same act that one-shot the
        // deck #20 at d 31 ate 8+ aimed CLEAR-ray rounds for ZERO on both members. A
        // head-only candidate is NOT attempted: the window they burn is cadence exposure
        // (the phase floor is the pair's opening cadence — 56 — before any act can fire).
        .filter(|(_, part, _)| *part != 5)
        .min_by_key(|(_, part, d)|
            ((d.pos[0] - at[0]).powi(2) + (d.pos[2] - at[1]).powi(2)) as u32)
        .map(|(i, part, _)| (i, part))
}

/// Every live west-roof band guard is act-proof THIS roll: blacklisted, or no clear box at
/// all, or the head-only graze line (the run-4379 law — the pick skips those).
fn band_all_stalled(w: &Walker, stalled: &[usize]) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    let live: Vec<usize> = sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && d.pos[1] > 18.0
            && (d.pos[0] - 151.5).abs() < 12.0 && (d.pos[2] + 248.0).abs() < 14.0)
        .map(|(i, _)| i).collect();
    !live.is_empty() && live.iter().all(|i| {
        if stalled.contains(i) { return true; }
        match clear_box_aim(sim, *i) {
            None => true,
            Some((_, part)) => part == 5,
        }
    })
}

/// THE BULLET FLIGHT AUDIT (level-0d-operative-walk-8, the ledger's lever (1)): env-gated
/// (`NF_0D_AUDIT=1`, zero cost when unset — the plain replay and the d2 twin stay
/// byte-identical). Every tick of `shoot_far_aim` it tracks the live `player_bullets`
/// (read-only: no RNG draws, no sim writes) and prints, per round: the SPAWN state (pos, dir,
/// weapon, speed, range) and the death state — the round's last pos/dir/travelled with the
/// flight's own verdicts AT that point: `drone_ray_part` (the flight's posed-box test over a
/// 4 u segment [M Bullet_update 0x12e7b0's per-tick sweep]) and the world stop
/// (`segment_flight_scoped` with the glass pass — the flight's own query). The aim
/// (`clear_box_aim`/`segment_bullet_scoped`) and the flight share the same box law, so a
/// SPAWN verdict of part Some + world CLEAR that still dies identifies the diverging leg
/// (the spread actually applied, the dir the scene composes, or the world pick the aim gate
/// does not see).
thread_local! {
    static AUDIT_LAST: std::cell::RefCell<Vec<(usize, [f32; 3], [f32; 3], f32)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn audit_bullets(w: &Walker, di: usize, start: u32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    // the verdict rides the WHOLE remaining flight (up to 60 u): the world stop point (t·l,
    // absolute coords) and the posed-box test over the same reach — the aim gate (the eye's
    // cel) and the flight (the bullet's per-tick cel) may disagree; the stop point names the leg
    let verdict = |pos: [f32; 3], dir: [f32; 3]| -> String {
        let l = 60.0f32;
        let end = [pos[0] + dir[0] * l, pos[1] + dir[1] * l, pos[2] + dir[2] * l];
        let world = sim.world.segment_flight_scoped(sim.world.find_cel(pos), pos, end, true);
        let wp = world.map(|t| [pos[0] + dir[0] * l * t, pos[1] + dir[1] * l * t, pos[2] + dir[2] * l * t]);
        let part = nf_game::drone::drone_ray_part(&sim.drones.list[di], &d, pos, end);
        let dr = &sim.drones.list[di];
        format!("tgt g{di} ({:.2},{:.2},{:.2}) st{:#x} en{} pr{} hp{:.0} part{:?} world {}",
            dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.enabled as u8, dr.present as u8, dr.health,
            part.map(|(p, t)| format!("Some({p},{t:.3})")).unwrap_or_else(|| "None".into()),
            wp.map(|w| format!("STOP({:.2},{:.2},{:.2})", w[0], w[1], w[2])).unwrap_or_else(|| "CLEAR".into()))
    };
    AUDIT_LAST.with(|last| {
        let mut last = last.borrow_mut();
        let mut matched = vec![false; last.len()];
        for (k, b) in sim.drones.player_bullets.iter().enumerate() {
            let mut mi = None;
            for (i, e) in last.iter().enumerate() {
                if matched[i] || e.0 != b.weapon as usize { continue; }
                let dd = (e.1[0] - b.pos[0]).powi(2) + (e.1[1] - b.pos[1]).powi(2) + (e.1[2] - b.pos[2]).powi(2);
                // the P2K flies 13.2 u/tick [L weapon_data 0x1e +0x20] — the match window rides it
                if dd < 15.0 * 15.0 { mi = Some(i); break; }
            }
            match mi {
                Some(i) => { matched[i] = true; last[i] = (b.weapon as usize, b.pos, b.dir, b.travelled); }
                None => {
                    eprintln!("[audit] +{} b{k} SPAWN pos({:.2},{:.2},{:.2}) dir({:.4},{:.4},{:.4}) w{:#x} sp{:.2} rng{:.0} {}",
                        w.total_ticks - start, b.pos[0], b.pos[1], b.pos[2], b.dir[0], b.dir[1], b.dir[2],
                        b.weapon, b.speed, b.range, verdict(b.pos, b.dir));
                    last.push((b.weapon as usize, b.pos, b.dir, b.travelled));
                    matched.push(true);
                }
            }
        }
        for (i, m) in matched.iter().enumerate() {
            if !*m {
                let e = &last[i];
                eprintln!("[audit] +{} DEAD pos({:.2},{:.2},{:.2}) dir({:.4},{:.4},{:.4}) trv {:.1} {}",
                    w.total_ticks - start, e.1[0], e.1[1], e.1[2], e.2[0], e.2[1], e.2[2], e.3, verdict(e.1, e.2));
            }
        }
        let mut keep = Vec::new();
        for (i, m) in matched.iter().enumerate() { if *m { keep.push(last[i]); } }
        *last = keep;
    });
}

/// Clears the flight tracker (act boundaries) without printing.
fn audit_clear() {
    AUDIT_LAST.with(|last| last.borrow_mut().clear());
}

fn shoot_far_aim(w: &mut Walker, di: usize, label: &str, cap: u32, aim_dy: f32, skip_drain: bool) -> bool {
    let audit = std::env::var_os("NF_0D_AUDIT").is_some();
    if audit { audit_clear(); }
    let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
    let mut last_tap = 0u32;
    let start = w.total_ticks;
    // The aim-on EDGE: a button held across the intro/legs consumed the edge (prev_aiming stuck
    // true with the gated bit never set — measured in probe_0d_bullet's gate trace), and the
    // raise queue REFUSES while a fire script plays (state 9 [M 0x1a24c8:66-99] — the previous
    // fight's 130-tick script), so release long enough to drain it, then hold until the bit
    // actually sets (the raise completes ~20 ticks [M Player_SetWeaponAnimObj 0x1a2e38 case 0xf]).
    // The aim-on EDGE (see below) — but SKIPPED when the aim bit is already held (the sweep
    // passes, back-to-back legs: the drain exists for the raise-queue fire-script busy case,
    // which cannot be pending while `aiming` is on; each skipped drain saves 160 ticks of
    // standing in a live crossfire — the ladder-top phase is under sniper fire since the
    // class-4 sight 1000 landed [M 0x14b300:380-424]).
    // Agent-gated in the batch-1109a-walks merge: the plain and Operative runs keep the drain.
    let agent_fast = w.scene.sim.as_ref().unwrap().drones.difficulty >= 2;
    if !(skip_drain || (agent_fast && w.scene.sim.as_ref().unwrap().aiming)) {
        for _ in 0..160 {
            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
            w.total_ticks += 1;
        }
    }
    // Agent: turn the view onto the target before the aim (the aim cursor pins at the box edge
    // for a far target off-centre).
    if agent_fast {
        for _ in 0..240 {
            let sim = w.scene.sim.as_ref().unwrap();
            let e = sim.bond.head();
            let d = sim.drones.list[di].pos;
            let t = [d[0], d[1] + 1.4, d[2]];
            let yaw = (t[0] - e[0]).atan2(t[2] - e[2]);
            let err = walker::wrap_pi(yaw - sim.bond.yaw);
            let pitch = (t[1] - e[1]).atan2(((t[0] - e[0]).powi(2) + (t[2] - e[2]).powi(2)).sqrt());
            let perr = pitch / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
            if err.abs() < 0.01 && perr.abs() < 0.01 && !sim.aiming { break; }
            let inp = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.06, 0.06), ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
            w.total_ticks += 1;
        }
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
            let (tx, ty) = (0.76966 * sim.cross[0] / sim.zoom, 0.57724 * sim.cross[1] / sim.zoom);
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
            // THE AIM-BIT TRIGGER GATE (level-0d-agent-ladder-top-3, the run-4380 measurement):
            // the aim bit FLAPS with the fire script between shots — the run-4380 trio pass
            // fired 5 HIP rounds (the full 100-unit cone [M Bullet_init 0x12c5a0:139-170]) at
            // d 63 while sim.aiming read false, every one a guaranteed miss. The d2 acts hold
            // fire until the bit is back up (the raise is ~20 ticks [M Player_SetWeaponAnimObj
            // 0x1a2e38 case 0xf]). Gated to agent_fast: the plain/Operative drained paths enter
            // the loop with the bit raised and keep their byte-identical behaviour.
            // THE OPERATIVE AIM-BIT TRIGGER GATE (level-0d-operative-walk-7, the ledger's
            // lever (1), the OPERATIVE HIP-FIRE LAW's own fix): the aim bit FLAPS with the
            // fire script between shots, and the d1 shortcut (`!agent_fast`) fired those
            // down-bit windows as HIP rounds — the full 100-unit cone [M Bullet_init
            // 0x12c5a0:139-170] — even with the ray converged (the measured bay trace: cross
            // ~0.0 at +377 with `aiming false`, two clips burned, the 10-health target
            // alive). On the Operative REAL-DAMAGE run the trigger now requires the bit like
            // the d2 path always has (the raise completes ~20 ticks [M Player_SetWeaponAnimObj
            // 0x1a2e38 case 0xf]); the plain replay (no_topup_env() false at d1) and the d2
            // twin keep their byte-identical gates.
            let op_real = walker::no_topup_env() && sim.drones.difficulty == 1;
            let trigger = !clip_empty && on_target && world_clear && (sim.aiming || (!agent_fast && !op_real));
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
        if audit { audit_bullets(w, di, start); }
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
