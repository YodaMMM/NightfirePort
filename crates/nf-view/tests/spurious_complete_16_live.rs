//! spurious-complete-16: a mission-complete banner popped up on 07000016 while the mission could
//! not actually continue.
//!
//! ROOT CAUSE [L ee_start_0700000a.bin @ 0x2a3f78 — the game's own `.data` at a mid-mission level
//! start]: at 0700000a's start, 07000009's prior-part record holds the FIRED ch-0x47 record at
//! state 4 (complete), the unfired ch-0x46 record at 3 (announced), the fail record at 2. The
//! port's `Mission::init` fabricated EVERY prior-part record as announced (3) — so any load
//! without a mission snapshot (a cold load: the pause-menu Restart, the driving return, boot
//! level-select; the whole pre-map-tower2 campaign flow) re-completed every carried record on the
//! first monitor tick: N× "OBJECTIVE COMPLETE" banners stack at the arrival while the mission is
//! still open — the spurious completion.
//!
//! The fix (as of 2026-10-13) was `Mission::init`'s prior-part fabricator. objectives-0b-start
//! (2026-10-17) REPLACED it: the fabricator contradicted the disc ([D] every record ships state
//! 0 — a cold boot must announce the priors, [L] a PCSX2 fresh 0700000b load) and the
//! warm arrivals it modelled now carry the real snapshot (`with_mission` + `restore`, the game's
//! persisted `.data` [M 0x1cd720/0x1cd880]). This test's carry therefore rides the snapshot —
//! the same assertions, the game-faithful shape.
//!
//! Tests: the 07000016 arrival with the carried 07000014/15 chain posts exactly ONE "OBJECTIVE
//! COMPLETE" (the spawn plate's ch 0x8e completing record 21) and no type-3 "Mission Complete"
//! banner; the real end (the KikoDeathButton #693 → ch 0x90 → mux #618 → ch 100 [D]) posts the
//! banner. Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::{Input, RamSave};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    // A fresh Bond with only the carried gates — and the mission snapshot the game's own warm
    // exit carries (`Player_RamSave`'s mission block, LS_MakeMission's shape [M 0x1cd720];
    // objectives-0b-start: the old channels-only shape leant on the removed `Mission::init`
    // fabricator — the snapshot IS the game's persisted `.data` at a real arrival, the same
    // states the [L ee_start_0700000a.bin] shape holds: 07000014's five records folded COMPLETE
    // during 07000014's play (their condition channels 0x47/0x48/0x4a/0x54/0xce ride the save
    // [L c07000014]), 07000015's announced (its ch 0x8e still unfired — the spawn plate's fold
    // is what this test watches), 07000016's sitting at the init shape (1/1, enable-gated).
    let mut m = nf_game::mission::Mission::new();
    m.base_map = 0x0700_0014;
    m.order = 2;
    m.states[20] = vec![4; 5];
    m.states[21] = vec![3];
    m.states[22] = vec![1, 1];
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
        .with_mission(&m)
}

/// The arrival: the carried chain (0x47/0x48/0x4a/0x54/0xce + 0x4b) rides the save; the spawn
/// plate #252 fires ch 0x8e. The game's own `.data` at this moment holds record 20's five
/// objectives COMPLETE (they folded during 07000014's play [L the snapshot's shape]) — the
/// arrival queues exactly ONE "OBJECTIVE COMPLETE" (record 21's ch-0x8e fold) and ONE announce
/// ("Reach Drake's launch silo"), and no type-3 "Mission Complete". FAIL-FIRST: the fabricator
/// used to re-complete every carried record on the first monitor tick (the burst).
#[test]
fn the_arrival_posts_one_objective_complete_and_no_mission_banner() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load 07000016");
    // Tick-by-tick through the arrival (the intro + the spawn plate): track the banner queues.
    // The queue drains one banner per 180 ticks, so the burst stays observable as simultaneous
    // entries; `max_completes` catches it however the intro spreads the ticks.
    let mut max_completes = 0usize;
    let mut max_total = 0usize;
    let mut mission_banner_ticks = 0u32;
    for _ in 0..3400u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let completes = sim.hud.objective_msgs.iter().filter(|m| m.status == 0x0200_0053).count();
        max_completes = max_completes.max(completes);
        max_total = max_total.max(sim.hud.objective_msgs.len());
        if !sim.hud.mission_msgs.is_empty() { mission_banner_ticks += 1; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #252 fired ch 0x8e [D placement #252]");
        assert!(sim.drones.mission.states[20].iter().all(|&s| s == 4),
            "the carried record 20 sits complete [L ee_start_0700000a.bin @ 0x2a3f90+0x14 = 4], got {:?}", sim.drones.mission.states[20]);
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4), "the plate completed record 21");
        assert_eq!(sim.drones.mission.states[22][0], 3, "'Reach Drake's launch silo' announced");
        assert_eq!(max_completes, 1,
            "exactly ONE 'OBJECTIVE COMPLETE' at the arrival (record 21's 0x8e fold) — the game's own count [L the snapshot]; the fabricator's re-completion burst posts the carried five again");
        assert_eq!(mission_banner_ticks, 0, "no type-3 'Mission Complete' banner before the end condition");
        let _ = max_total;
    }
}

/// The real end: the launch chain (ch 0x81 [D-driven] → the pack cinema → ch 240/242 → the
/// spawners) spawns the chaser; her defeat raises nothing (829); the KikoDeathButton #693 raises ch 0x90 → mux #618 →
/// ch 100 [D #271] → MissionState 6 — and NOW the type-3 "Mission Complete" banner posts
/// [M 0x182c60:85-122]. Sister: no mission banner before the ch 100 tick.
#[test]
fn the_real_end_condition_posts_the_mission_complete_banner() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load 07000016");
    let mut mission_banner_ticks = 0u32;
    let mut banner_before_end = 0u32;
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
        for _ in 0..200 { sim.step_bond(Input::default(), false, false, false); }
        // The silo plate #253 reveals the Kiko objective (ch 0x8f [D]).
        sim.bond = nf_game::player::Bond::new([2.5, 1.2, -153.9], std::f32::consts::PI);
        let mut input = Input::default();
        input.forward = 1.0;
        for _ in 1..=120 { sim.step_bond(input, false, false, false); if sim.drones.channels.on(0x8f) { break; } }
        // The launch chain's head: ch 0x81 (no writer on the disc data [D scans]) — driven the
        // way fix_finale_live drives it.
        sim.drones.channels.set(0x81, 1, sim.drones.frame);
    }
    let chaser = {
        let mut found = None;
        // The launch exhaust gate: 0x0600090a stream 6 raises ch 0xf3 (243) at script tick 1611 and
        // drops it at 2170 [D scriptlog], BEFORE the spawner writes (ch 0xf0 at 2180 [D]) — the
        // #309 HurtPoly (gate 0xf3, 1000 [D]) is live in between, and the cinema's snap leaves Bond
        // inside it on the silo floor. The mission only completes from a live Bond (Mission_Update
        // 0x182c60 case 1: the ch-0x62 fail arm — Player_SetHealth(0), MissionState 3
        // [M 0x182c60:21-43] — outranks the ch-100 completion arm), so the game's player leaves the
        // pit during the launch; the test re-seats her on the upper floor once the cinema hands
        // control back, the same route the descent came up.
        let mut re_seated = false;
        for _ in 0..4000u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            if !re_seated && !scene.sim.as_ref().unwrap().disabled {
                re_seated = true;
                scene.sim.as_mut().unwrap().bond.pos = [2.5, 1.2, -153.9];
            }
            let sim = scene.sim.as_ref().unwrap();
            if !sim.hud.mission_msgs.is_empty() { banner_before_end += 1; }
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0x1e && d.dtype == 0xf && !matches!(d.death_channel, 23..=26)) {
                found = Some(i);
                break;
            }
        }
        found.expect("the launch chain spawns the chaser (spawner #257, group 6 [D])")
    };
    // Her defeat through the game's explosive path (the ninja chain walks her; re-read per attempt).
    {
        let sim = scene.sim.as_mut().unwrap();
        let mut died = false;
        for _ in 0..3 {
            let p = sim.drones.list[chaser].pos;
            sim.drones.grenades.push(nf_game::grenade::Grenade {
                pos: p, velocity: [0.0, 0.0, 0.0], speed: 0.0,
                travelled: 59.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34,
                mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false,
                resting: false, stuck: false, seek_target: None, phase: None,
            });
            for _ in 0..300u32 {
                sim.step_bond(Input::default(), false, false, false);
                if sim.drones.list[chaser].corpse { died = true; break; }
            }
            if died { break; }
        }
        assert!(died, "the chaser dies at the silo");
    }
    // kiko-16-defeat (829): her death raises nothing (param 12 = 0 [D #261]) — no banner yet.
    for _ in 0..120u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().hud.mission_msgs.is_empty() { banner_before_end += 1; }
    }
    assert!(!scene.sim.as_ref().unwrap().drones.channels.on(0x90), "the chaser's death raises no ch 0x90 [D #261 param 12 = 0]");
    // The real end: Bond's E on the KikoDeathButton #693 (p0 = ch 0x90, gate 0x8d [D]) —
    // fix_finale_live's press (the re-seat beside the button is that test's reach; 911 owns the
    // on-foot route).
    let at = [7.02f32, 1.16, -155.61];
    for k in 0..240u32 {
        if scene.sim.as_ref().unwrap().drones.channels.on(0x90) { break; }
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = [7.02, 1.2, -154.4];
            sim.bond.vel = [0.0; 3];
            let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
            let to = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
            sim.bond.yaw = (at[0] - sim.bond.pos[0]).atan2(at[2] - sim.bond.pos[2]);
            sim.bond.pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, k % 6 == 0);
        if !scene.sim.as_ref().unwrap().hud.mission_msgs.is_empty() { banner_before_end += 1; }
    }
    // ch 0x90 → mux #618 → ch 100 → MissionState 6 → the banner [M 0x182c60:85-122].
    let mut complete = false;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if !sim.hud.mission_msgs.is_empty() { banner_before_end += 1; mission_banner_ticks += 1; }
        if sim.drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x90), "the KikoDeathButton raises ch 0x90 [D #693 p0]");
        assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 ← 0x90) fires ch 100 [D #618]");
        assert!(complete, "the final mission completes (MissionState 6 [M 0x182c60:85-116])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
        assert!(mission_banner_ticks > 0, "the type-3 'Mission Complete' banner posted at the real end");
        // The banner binds the game's 0x0200_0006 text.
        let want = sim.lang.as_ref().and_then(|l| l.bind(0x0200_0006)).map(|t| t.to_string());
        if let Some(want) = want {
            assert!(sim.hud.mission_msgs.iter().any(|m| m.text == want), "the banner is the game's 'Mission Complete' text, got {:?}",
                sim.hud.mission_msgs.iter().map(|m| m.text.clone()).collect::<Vec<_>>());
        }
        banner_before_end = banner_before_end.saturating_sub(mission_banner_ticks);
        assert_eq!(banner_before_end, 0, "no mission banner before the end condition");
    }
}
