//! walkreal-b: 07000012 (HT Tower2 B, the lower offices) walked with the player's own movement —
//! the route lives in `common/tower2.rs` (shared with the 07000013 warm chain). Every leg is the
//! real input path except the TWO documented stand-ins: the door[19] doorway crossing (the corner
//! pincer — see the report's live checks) and the north-wall funnel into the shaft lobby (the
//! 0.88 pinch — measured, see tower2.rs's comment and the probe12 evidence). The level's summary:
//!
//! the elevator start #0 (22.49, 0.19, 19.16 [D]) → the spawn TouchOnce #679 fires ch 0x48 → the
//! REAL walk south around the shaft core → the REAL Q-Decryptor G-cycle + beam burn of the keypad
//! locks #656/#694 ("Lock - KeypadLocked", out 0xa [D]) → ch 0xa's rising edge OPENS door[19]
//! ([M Door_Update's lock path; the 0x1336f8 two-latch snapshot — the fix this task landed]) →
//! the doorway stand-in → the REAL walk to the security-centre console → E → ch 0x49 (objective
//! 1) + ch 0x59 (the SS lift doors #677/#678 slide to key 1 [M SS_Update 0x1a9948]) → the REAL
//! lift call button #699 E (ch 0x58 swaps both shutter SS pairs [M SS_Update]) → the funnel
//! stand-in → the REAL walk down the lobby ramp into the shaft entrance → #683 holds ch 0x1F →
//! the gated exit #669 auto-fires ([M Trigger_Update 0x1b1a78 case 10]) → level_exit 0700004A →
//! the RamSave carried through the real load: 4A's spawn drop into touch #277 fires ch 0x4D and
//! objective 2 "Escape through the elevator shaft" completes (the late AND — its writer is on
//! 0700004A [D census §2]).

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_spawn_to_the_shaft_exit() {
    let iso = iso();
    // The keypad locks (placements 656/694) carry param 6 = 1: Lock_Create auto-unlocks them at
    // difficulty 1 [M 0x1ae1f8 tail: level_tag+0x44 != 0 && GameState+0x28 == 1 -> Lock_Unlock],
    // so the Q-Decryptor keypad burn this route walks is the Agent+ behavior; the load is 00 Agent
    // where the full (pre-gate calibration) roster exists [M 0x136be0:49].
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 21.0 && p[0] < 24.0 && p[1] > 0.0 && p[1] < 2.0 && p[2] > 18.0 && p[2] < 20.0,
            "the elevator start #0 [D], got {p:?}");
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
        assert_eq!(sim.drones.mission.states[16][1], 4,
            "12's 'Escape through the elevator shaft' completes through the real exit load (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][0], 4,
            "12's 'Activate elevator override control' stays complete [M 0x182270 keeps previous parts]");
    }
}
