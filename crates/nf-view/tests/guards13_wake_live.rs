//! level-13-guards-start-alerted-9: the REAL-path wake pin — the two Phoenix Fire lobby guards
//! wake through the game's own chain and NEVER fire before their first sight ack.
//!
//! The real path (the way a player reaches 07000013): 07000012 walked → its exit loads 0700004A →
//! the spawn drop onto touch #277 (ch 0x4D) → the RamSave carry → 07000013. The level-start data
//! [D census m41]: the 0xEA TouchOnces **#58 (out 0x28)** at (2.17, −368.63, 11.13) and **#419
//! (out 0x4A)** at (2.14, −368.55, 11.35) — Bond's spawn (2.157, −368.334, 10.960 [L port runtime,
//! the level's own start record]) sits INSIDE both volumes, so the spawn touch IS the game's own
//! wake stimulus, and the two lobby guards (placements #318/#319, skin 0x05000008, wait ch 0x28,
//! behaviour word 15 = 3 → prop 0x20 = 3 [D bitDescs 0x40: block+4 &3, mask 3] → alertness
//! table[3] = 1.0 [L ee_start_05 @0x2f5cc8; M DoModeSettingsNEW 0x14c600:70-77]) wait disabled in
//! WaitSwitch (ENTER = `NDrone2_Enable(0)` [M 0x15b960:12-14]) until it fires.
//!
//! The wake chain, every step game-cited: WaitSwitch case 3 → `Enable(1)` + the start state
//! [M 0x15b960:39-56]; the start state for +0xc5 = 7 is EnemyRunToPoint 0x85 (DroneTypeSettings
//! row 7 start 0x85 [L ee_snapshot @0x29aee8+7·12]; the -1 → mode-7 default [M 0x14cbd8:113-121]);
//! 0x85's ENTER sets the vision bit, and the Tower-mission clause (07000013 in {11,12,13,41},
//! 0x4000000 clear) keeps the +0x5A2 resume at ATTACK 0x56 because +0x50c = 1.0
//! [M 0x168910:17-43]; FindRunToPoint → GoToGoalPosition 0x63 or straight to the resume
//! [M 0x168910:44-56]; Attack's FirstAttack with no sight bit and no sound bits returns 0 and the
//! fallback lands Combat 0x58 [M 0x164210:24-90 + 0x148010]; the combat router holds the player
//! opponent for team ≠ 2 [M 0x1424e0:100-120] and parks 0x5f/0x5c while +0x274 grows
//! [M 0x148338:67-75].
//!
//! THE PIN (the player-visible rule in all of this): **no guard fires before its own first sight
//! ack** — every combat fire arm gates the request on the +0x228 & 4 seen flag
//! [M 0x148338:108-113, 0x164c08 case 1's +0x3b = 0, 0x17a080:24-25], and the seen flag itself
//! rises only behind the reaction gate on a LOS tick [M 0x176a28:216-231 → 0x175ef0:20-25]. Bond's
//! health must not drop while no guard has `ever_seen`.
use nf_game::player::Input;

#[path = "common/tower2.rs"]
mod tower2;
#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_lobby_guards_wake_through_the_game_chain_and_never_fire_before_their_first_sight() {
    let iso = iso();
    // ---- 07000012 walked (the real approach, Agent difficulty) ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
        tower2::walk_12(&mut w);
        tower2::walk_12_exit(&mut w);
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    // ---- 0700004A: the spawn drop onto the 0x4D touch ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
        w.wait_enabled(3000, "the 4A intro");
        let mut landed = false;
        for _ in 0..600 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
        }
        assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
        for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };

    // ---- 07000013: the lift arrival ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48) && sim.drones.channels.on(0x49) && sim.drones.channels.on(0x4d),
            "the previous parts' channels carry through the warm load [M 0x182270]");
        let guards: Vec<usize> = (0..sim.drones.list.len())
            .filter(|&i| sim.drones.list[i].present && sim.drones.list[i].skin == 0x0500_0008)
            .collect();
        assert_eq!(guards.len(), 2, "the Agent-difficulty lobby roster is the two skin-0x05000008 placements [D census #318/#319]");
        // Bond spawns INSIDE the wake touch [D #58 at (2.17,−368.63,11.13); L spawn (2.16,−368.33,10.96)].
        assert!(sim.bond.pos[2] > 10.0 && sim.bond.pos[2] < 12.5,
            "Bond's spawn sits in the 0x28/0x4A touch band z 10.6..11.8 [D #58/#419 volumes]");
    }
    w.wait_enabled(3000, "the 13 intro");
    w.wait(30);
    let guards: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let g: Vec<usize> = (0..sim.drones.list.len())
            .filter(|&i| sim.drones.list[i].present && sim.drones.list[i].skin == 0x0500_0008)
            .collect();
        assert!(sim.drones.channels.on(0x28), "the spawn touch #58 writes ch 0x28 [D]");
        for &i in &g {
            let dr = &sim.drones.list[i];
            assert_eq!(dr.alert, 1.0, "the lobby guards' +0x50c starts at table[prop 0x20=3] = 1.0 [M 0x14c600:70-77; D param 15; L 0x2f5cc8[3]]");
            assert!(dr.vision_on, "the wake chain's 0x85 ENTER sets the vision bit [M 0x168910:17]");
            // The wake states are the game's own chain (0x85/0x63/0x56/0x58/0x5f/0x5c/0x68) — never
            // Alert 0x05 (the Tower clause KEEPS the 0x56 resume at +0x50c = 1.0 [M 0x168910:34-43]).
            assert_ne!(dr.state, 0x05, "the wake never lands in Alert 0x05 [M 0x168910:34-43]");
            assert_ne!(dr.state, nf_game::drone::state::GLOBAL, "the guards woke into the start-state chain [M 0x15b960 case 3]");
        }
        g
    };

    // THE PIN: 600 quiet ticks (Bond stands at the spawn — the way the player regains control),
    // then the first two walk legs. Bond's health must never drop while NO guard has ever_seen:
    // the fire request is gated on the +0x228 & 4 seen flag in every combat arm
    // [M 0x148338:108-113; 0x17a080:24-25], and the seen flag rises only behind the reaction gate
    // on a LOS tick [M 0x176a28:216-231].
    let mut wounded_without_sight: Option<u32> = None;
    let start_health = w.scene.sim.as_ref().unwrap().drones.health.health;
    let mut last_health = start_health;
    for _ in 0..600 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let any_ever_seen = guards.iter().any(|&i| sim.drones.list[i].ever_seen);
        if sim.drones.health.health < last_health && !any_ever_seen {
            wounded_without_sight = Some(sim.drones.frame);
            break;
        }
        last_health = sim.drones.health.health;
    }
    assert!(wounded_without_sight.is_none(),
        "a guard wounded Bond at frame {:?} with NO guard ever having sighted him — the fire request must be gated on the +0x228 & 4 seen flag [M 0x148338:108-113]",
        wounded_without_sight);
    // The first two walk legs (the player moving in quietly — no shots, no use): the same pin holds.
    w.walk_to(walker::Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let any_ever_seen = guards.iter().any(|&i| sim.drones.list[i].ever_seen);
        if !any_ever_seen {
            assert!(sim.drones.health.health >= start_health,
                "Bond wounded during the quiet approach with no guard ever sighted [M 0x148338:108-113]");
        } else {
            eprintln!("[13-wake] a guard sighted Bond during the quiet approach (ever_seen) — the wake hunt reached his LOS [M 0x148338 router]");
        }
    }
    eprintln!("[13-wake] pin green: health {:.3} -> {:.3} over the quiet window", start_health, last_health);
    let _ = Input::default();
}
