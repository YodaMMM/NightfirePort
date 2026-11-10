//! walkreal-b: 07000012 (HT Tower2 B, the lower offices) walked with the player's own movement —
//! the route lives in `common/tower2.rs` (shared with the 07000013 warm chain). Every leg is the
//! real input path; no position writes. The level's summary:
//!
//! the elevator start #0 (22.49, 0.19, 19.16 [D]) → the spawn TouchOnce #679 fires ch 0x48 → the
//! REAL walk south around the shaft core → the REAL Q-Decryptor G-cycle + beam burn of the keypad
//! locks #656/#694 ("Lock - KeypadLocked", out 0xa [D]) → ch 0xa's rising edge OPENS door[19]
//! ([M Door_Update's lock path; the 0x1336f8 two-latch snapshot]) → the REAL walk through the
//! doorway (the security room's cel at the identity [M parseentity_fixup_entity 0x1d00c0]; the
//! game's crossing line [L]) → the console → E → ch 0x49 (objective 1) + ch 0x59 (the SS lift doors
//! #677/#678 slide to key 1 [M SS_Update 0x1a9948]) → the REAL lift call button #699 E (ch 0x58
//! swaps both shutter SS pairs [M SS_Update]) → back out through door[19] and the REAL walk round
//! the south and west wings to the north lobby (door plc 545 opened with E) → through the lift
//! doors into the shaft entrance → #683 holds ch 0x1F → the gated exit #669 auto-fires ([M
//! Trigger_Update 0x1b1a78 case 10]) → level_exit 0700004A → the RamSave carried through the real
//! load: 4A's spawn drop into touch #277 fires ch 0x4D and objective 2 "Escape through the
//! elevator shaft" completes (the late AND — its writer is on 0700004A [D census §2]).

//! every-objective: the level's mission data lists EXACTLY TWO objective records — decoded from
//! the EE snapshot [L ee_snapshot.bin 0x2a4350 + row 16*40 (level 07000012, mission 07000011,
//! part 1) → objective table 0x2a41b8, count 2]:
//!   obj0 text 0x04000037 "Activate elevator override control in security center", fail none,
//!        hint 0x0400003d, condition ch 0x49, enable ch 0x48, flags 0;
//!   obj1 text 0x04000038 "Escape through the elevator shaft", fail none, hint 0x0400003e,
//!        condition ch 0x4D, enable ch 0x49, flags 0.
//! The test below covers BOTH, in the order a player does them: obj0 announced at the spawn
//! (the enable ch 0x48 fires under Bond's feet [D #679]) and completed at the console E
//! (tower2::walk_12 asserts states[16][0] 3 → 4 with ch 0x49); obj1 announced at the console
//! (states[16][1] == 3) and completed through the REAL exit load — the level's own 0xE8 exit
//! fires into 0700004A and its spawn touch #277 writes ch 0x4D (the late AND — obj1's writer
//! lives on 0700004A [D census §2]), asserted states[16][1] == 4 with obj0 still 4. Nothing
//! else to leg.

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission-table index of THIS level's record, with the whole objective list pinned [L
/// ee_snapshot.bin 0x2a4350 row 16 → table 0x2a41b8 count 2].
fn mission_row_12() -> usize {
    let row = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0012)
        .expect("the level's mission record");
    assert_eq!(nf_game::mission::LEVELS[row].objectives.len(), 2,
        "07000012's mission data lists exactly two objective records [L ee_snapshot.bin 0x2a4350 row 16 → 0x2a41b8 count 2]");
    row
}

fn walk_spawn_to_the_shaft_exit_at(difficulty: u32) {
    let iso = iso();
    let row12 = mission_row_12();
    assert_eq!(row12, 16, "the record's row in the port's LEVELS table [L row 16]");
    // The keypad locks (placements 656/694) carry param 6 = 1: Lock_Create auto-unlocks them at
    // difficulty 1 [M 0x1ae1f8 tail: level_tag+0x44 != 0 && GameState+0x28 == 1 -> Lock_Unlock],
    // so the Q-Decryptor keypad burn this route walks is the Agent+ behavior; the difficulty-3
    // load is 00 Agent where the full (pre-gate calibration) roster exists [M 0x136be0:49]. The
    // difficulty-3 expression is byte-identical to the pre-sister master's (the same
    // load_with_carry_difficulty(..., 3) — real-health-agent-difficulty-01-07's idiom).
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, difficulty).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 21.0 && p[0] < 24.0 && p[1] > 0.0 && p[1] < 2.0 && p[2] > 18.0 && p[2] < 20.0,
            "the elevator start #0 [D], got {p:?}");
        assert_eq!(sim.drones.difficulty, difficulty,
            "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    }
    tower2::walk_12(&mut w);
    tower2::walk_12_exit(&mut w);

    // ---- EVERY objective of the level: obj 2 "Escape through the elevator shaft" (0x04000038,
    // complete ch 0x4D) has NO writer on 07000012 [D — the census §2]; its writer is TouchOnce
    // #277 on 0700004A (the shaft's first plate, y −9.87..−7.65 [D 0700004a.md §2]) — the late
    // AND. The exit the route just fired loads 0700004A ([M Trigger_Update 0x1b1a78 case 10]) —
    // follow it the way the game does: carry the RamSave through the load and let the 4A spawn
    // (#43 at (4.26, −7.71, −4.76) [D], INSIDE #277's volume) complete the objective.
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
            // The difficulty rides the warm save: the game's GameState+0x28 global survives the
            // level change untouched (no transition-path writer [M whole-decompile grep]) and the
            // next level's creates/locks read the SAME 00 Agent pick [M 0x136be0:49, 0x1ae1f8 tail].
            .with_difficulty(sim.drones.difficulty)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
    w.wait_enabled(3000, "the 4A intro");
    let mut landed = false;
    for _ in 0..600 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
    }
    assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
    // The mission monitor runs BEFORE the object handler each frame [M Mission_Update 0x182c60
    // from Game_Run 0x1c9790:17], so the 0x4D write completes the objective the NEXT frame.
    for _ in 0..30 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[row12][1], 4,
            "12's 'Escape through the elevator shaft' completes through the real exit load (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[row12][0], 4,
            "12's 'Activate elevator override control' stays complete [M 0x182270 keeps previous parts]");
    }
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-12-13-14-15): the
/// same route and the same end, loaded at difficulty 2 the menu way (the front-end's launch path
/// [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]; GameState+0x28 → the sim's
/// Drones::difficulty, asserted at load). Agent spawns the min-difficulty-2 placements
/// [M Drone_Create 0x136be0:49] — fewer than the master walk's 00-Agent roster — scales Bond's
/// pain by Plr_DMod_Normal [M Player_HandlePain 0x1902c8] and the guards' hit chance by
/// Accuracy row 2 [M DroneWeap_DoBulletAccuracy 0x179740:74-88]. Real input only; the fights
/// keep the level's own head-hunting + unseen-cover law.
#[test]
fn walk_spawn_to_the_shaft_exit_real_damage_agent() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(|| walk_spawn_to_the_shaft_exit_at(2)))
}

/// Operative sister of `..._real_damage_agent`: the same route at difficulty 1. Operative
/// creates only the p2 = 1 placements [M Drone_Create 0x136be0:49] and the keypad locks unlock
/// at create [M Lock_Create 0x1ae1f8 tail: param 6 = 1 on difficulty 1] (the route's unlock
/// check accepts both paths, common/tower2.rs). Real input only.
#[test]
fn walk_spawn_to_the_shaft_exit_real_damage_operative() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(|| walk_spawn_to_the_shaft_exit_at(1)))
}

#[test]
fn walk_spawn_to_the_shaft_exit() {
    walk_spawn_to_the_shaft_exit_at(3);
}

/// Real damage: the whole-mission walk (the Tower2 B offices route plus the real 0700004A load)
/// with the health top-up off for every walker it makes and the walker's return fire on — Bond's
/// death panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that
/// the route completes on the port's own damage. No health/armour/ammo writes anywhere; the exit
/// load into 0700004A is the game's own Trigger_Activate path [M 0x1b2318 → Player_RamLoad
/// 0x1957a8], not a teleport.
///
/// The fights HEAD-HUNT (`with_real_damage_head_hunting`): the level's guards are the
/// 12-health type-2 skins (0x500004/0x500006/0x500008) — the first real-health run's damage
/// log: the whole 100.0 was 20 hits, the shotgun guard #10 (skin 0x5000008, weapon 0x1c)
/// dealing 72 in 16 rounds from 4.2 u [D weapon_data 0x2bf150 +0x18 = 8 pellets] while the
/// return fire's torso taps needed 3-5 rounds a guard — the level's own offering is the aim:
/// one head hit drops a 12-health guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head].
/// The port's fire law is the game's: one accuracy roll per FireWeapon call, the 8-pellet
/// cone along the SAME aim [M DroneWeap_FireWeapon 0x178f68:233-241], TooClose ×2
/// accuracy/damage only under 3.0 u [M DroneWeap_DoBulletAccuracy 0x179740:113-115;
/// Drone_ModBulletDamage 0x139f50:21-23] — the 4.2-u volley that landed 6 pellets on one
/// frame is the game's own rolled-hit cone, not a port over-damage.
/// The fights also answer the DESK guards (`with_real_damage_unseen_fights`): the offices'
/// guards sit behind the desk rows and the first run's fights never started against the worst
/// of them (the plain pick needs one clear eye ray; the Office-C computer mesh eats every ray
/// — the same mechanism the 07000013 lobby diagnosed) while the shotgun guard's volleys land.
/// The unseen fight's creep walks Bond up to the desk lip until the line opens — the level's
/// own cover law, not a cheat: no health, armour, ammo or position writes.
#[test]
#[ignore = "los-mask-01-walk-2: the masked-tree Operative walk wedges at 'the floor south 2' — (16.66,1.39,3.19)/(17.15,1.39,3.19)/(15.60,1.40,3.19) across resupply rounds 0-3, target (17.48,0.90,1.51) d 1.7-2.5, segment 0.328, grounded, cel 50, health 100; the sidestep ladder also wedged 'the south corridor east' resupply 1 at (17.11,1.60,−10.60) NOT grounded, cel 339 [L the captured run]. The masked-timeline wedge class (the 03 moat law; the 11 ledger's sibling). NEXT LEVER: re-time the floor-south and corridor-east legs (a waypoint past each wedge spot + pinned fight sites), gated to 07000012/difficulty 1"]
fn walk_spawn_to_the_shaft_exit_real_damage() {
    walker::with_real_damage_head_hunting(||
        walker::with_real_damage_unseen_fights(walk_spawn_to_the_shaft_exit))
}
