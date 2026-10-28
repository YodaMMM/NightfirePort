//! drake-fight-1b: the 0700001b final fight's mechanics — the missile launch sequence
//! (`Create_SpaceMissile` 0x1a9ef0 / `SpaceMissile_Update` 0x1aa118's five states), the
//! `MissileDeploy` bytes (0x30cd70), and the win/fail conditions (AND #139 → ch 9; the launch
//! branch's p1 ch 99 @t1000 → the mission fail flow [M 0x182c60:23-33]).
//!
//! Drives the REAL path (`nf_view::load` + `Sim::step_bond`). The launch-order channel 19 is
//! driven directly: it is AND #202's output (ch 0xc1 & 0xc2 → 19 [D #202]) and 0xc2 has **no
//! writer** in the level's 77 scripts or any placement [D sweep 2026-10-17] — the same shape as
//! fix_finale_live's ch 0x0a drive (the gate whose chain head the disc data cannot reach).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// With the G-047 hold (a negative delay holds the script): the MissileDissarm scripts
/// (0x060007e2..0x060007ea) pause on their stream-1 Event 3 p0 0 at t650 (`SCRIPTINFO+0x9C = -1`
/// holds - `Script_Update` 0x1da9c4 runs only a zero delay [M disasm]) until `SP_UnPause`
/// 0x1dca58 zeroes it (SP_Update's damage tail at the hit points [M 0x1dc1c0-0x1dc330]; the
/// scenery-shot stat names exactly these scripts [M 0x1dc290-0x1dc2c8] - the player shoots them).
/// The test stands in for the shots with `SP_UnPause`'s two effects (the one-tick `+0x2c`
/// activation and delay 0) on every held one [S trigger only - the shot itself is not simulated].
fn shoot_held_disarm_sps(sim: &mut nf_view::scene::Sim) {
    for p in sim.drones.scripts.players.iter_mut() {
        if !(212..=220).contains(&p.placement) { continue; }
        let cur = p.cur;
        let held = (0x0600_07e2..=0x0600_07ea).contains(&p.scripts[cur]);
        if let Some(s) = p.info[cur].as_mut() {
            if s.delay != 0 && held {
                s.delay = 0;
                p.activated = true;
            }
        }
    }
}

fn free(sim: &mut nf_view::scene::Sim, ticks: u32) -> bool {
    for _ in 0..ticks { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { return true; } }
    !sim.disabled
}

/// The walk allowance (the walkreal harness's own rule): the station's defence AI still runs —
/// the zero-g skins' ASTRONAUT_LAUNCH/COMBAT/COMBAT_MOVE ENTERs carry the game's sight bit
/// [M 0x16dd28:17, 0x174458:16, 0x174550:18] and DroneMove_AstronautCombat fires on the sight
/// flag [M 0x154618:78] — the player stands it off by fighting; this scripted-sequence stand-in
/// cannot, so the shots cannot kill Bond here (the health top-up, the same allowance the
/// walkreal 01 walk documents in its header).
fn step_alive(sim: &mut nf_view::scene::Sim) {
    sim.step_bond(Input::default(), false, false, false);
    if sim.drones.health.health < 100.0 { sim.drones.health.health = 100.0; }
}

/// The launch sequence's phases on the game's trigger, then the disarm branch:
/// ch 19 → state 0 arms every missile (`MissileDeploy[k] = 1`, the p3 seeds are all < the 512.0
/// step [M 0x1aa1ec-238]) → the p0 scripts write their gate channels 31..38 at t359 (missile 4's
/// 06000a30 writes ch 192 at t1 instead [D]) → the MissileDissarm SPs #213–220 (gates 31..38 [D])
/// play the console scripts, which write the console channels 1..8 at t682 [D] → each p0 end →
/// state 2 takes the disarm branch (`switch_channels[p4]` set: `MissileDeploy[k] = 2`, obj+0xf6 = 0
/// [M 0x1aa25c-338]) → state 3 writes the per-missile channels 21..28 [M 0x1aa354-398] → AND #139
/// (21 & 22 & 27 → ch 9 [D #139]) completes "Disable the missiles" [L record 1b].
/// Sister: ch 199 (the abort channel) never rises while every missile disarms [M state 3's 0xf6==0
/// arm writes p6, not p5]. Missile 4's console chain is driven with ch 4 [S]: its gate 34's only
/// writer is 06000928 (SP #200, gated on ch 191 ← SP #199 out, gated on 190) and 190 has no writer
/// in the level's scripts [D sweep] — the drive stands in for that unproven chain head.
#[test]
fn the_missile_launch_sequence_reaches_each_phase_and_disarms() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free after the shuttle arrival");
        assert!(sim.drones.space_missiles.len() == 8, "the level carries the eight kind-0x39 missiles [D #129/#131–137]");
        // The launch order (AND #202's output; the test drives it — see the module doc).
        sim.drones.channels.set(19, 1, sim.drones.frame);
    }
    // Phase 1 — state 0 arms on the first gated tick (every p3 seed 0..105 < the 512.0 step).
    for _ in 0..3 { step_alive(scene.sim.as_mut().unwrap()); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.missile_deploy.iter().all(|&b| b == 1),
            "state 0 → MissileDeploy[k] = 1 for all eight [M 0x1aa210-238], got {:?}", sim.drones.missile_deploy);
        assert!(sim.drones.space_missiles.iter().all(|m| m.state == 1), "every missile is in state 1 [M 0x1aa21c]");
    }
    // Phase 2 — the p0 scripts' gate writes at t359 [D]; missile 4 wrote ch 192 at its t1 [D].
    for _ in 0..370 { step_alive(scene.sim.as_mut().unwrap()); }
    {
        let sim = scene.sim.as_ref().unwrap();
        for g in [31u32, 32, 33, 35, 36, 37, 38] {
            assert!(sim.drones.channels.on(g), "the p0 script's t359 gate write ch {g} [D]");
        }
        assert!(sim.drones.channels.on(192), "missile 4's 06000a30 writes ch 192 at t1 [D]");
    }
    // The console chain: the port's SPs #213–220 play on gates 31..38, the console scripts write
    // ch 1..8 at their t682 [D] (gate 34's ch 34 rise — SP #216 — is disc-positive in the port).
    let mut console = Vec::new();
    for _ in 0..2000u32 {
        step_alive(scene.sim.as_mut().unwrap());
        shoot_held_disarm_sps(scene.sim.as_mut().unwrap());
        let sim = scene.sim.as_ref().unwrap();
        for k in 1..=8u32 { if sim.drones.channels.on(k) && !console.contains(&k) { console.push(k); } }
        if console.len() == 8 { break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(console.len(), 8, "the console scripts write ch 1..8 at t682 [D], got {console:?}");
    }
    // Phase 3 — each p0 end (state-1 tick L−1 [D t3299/1799/2399]) → state 2 → the disarm branch
    // (ch[k] set) → MissileDeploy[k] = 2; the state-3 p2 waits [D per-missile 1170..1710] →
    // ch 21..28; AND #139 → ch 9.
    let mut disarmed = false;
    for _ in 0..7000u32 {
        step_alive(scene.sim.as_mut().unwrap());
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.missile_deploy.iter().all(|&b| b == 2) { disarmed = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(disarmed, "state 2's disarm branch sets MissileDeploy[k] = 2 [M 0x1aa2cc-338], got {:?}", sim.drones.missile_deploy);
        assert!(sim.drones.space_missiles.iter().all(|m| m.launch_flag == 0), "obj+0xf6 = 0 on the disarm path [M 0x1aa2bc]");
    }
    let mut ch9 = false;
    for _ in 0..3000u32 {
        step_alive(scene.sim.as_mut().unwrap());
        if scene.sim.as_ref().unwrap().drones.channels.on(9) { ch9 = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(ch9, "state 3 writes ch 21..28 [M 0x1aa354-398] → AND #139 (21&22&27) → ch 9 [D #139]");
        for c in [21u32, 22, 27] {
            assert!(sim.drones.channels.on(c), "AND #139's input ch {c} [D p6]");
        }
        // Sister: ch 24 rose (missile 4's state 3) and is then cleared by the speech script
        // 060009a4's own ch-24 stream [D decode: "ch 24 @t1200/1510-ish"] — the game's AND watches
        // only 21/22/27, so the clear is disc behaviour, not a port bug.
        assert!(!sim.drones.channels.on(199), "sister: the abort channel stays clear on the disarm path [M 0x1aa354 vs 0x1aa360]");
        // The objective monitor folds ch 9 into objective 1 the tick after (the monitor runs at the
        // top of the tick [M 0x1827d8; fix-objective-banner]).
        for _ in 0..2 { step_alive(scene.sim.as_mut().unwrap()); }
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.mission.states.iter().flatten().any(|&s| s == 4),
            "objective 1 'Disable the missiles' completes [L record 1b: text 0x04000059, ch 9]");
    }
}

/// The launch branch's fail flow: with ch 19 driven and the MissileDissarm SPs #213–220 removed
/// (the test setup — the proven precondition for the [M] state-2 launch branch:
/// `switch_channels[p4]` clear), EVERY missile's state 2 takes the launch branch
/// (`MissileDeploy[k] = 3`, obj+0xf6 = 1 [M 0x1aa25c-338]) and its p1 script writes
/// `ch 99 = 1` @p1-t1000 [D 060007c6/060007e1 stream 14] — the game's mission-fail channel:
/// `Mission_Update` 0x182c60:23-33 (internal 1) reads it → MissionState 2 (the type-3 fail msg,
/// TimeOut 300 [M :45-57]). The missiles 3/4/5 get there first (p0 1800 → state-1 tick 1799
/// [D] + p1 1000), i.e. armed+2799.
///
/// The mission's fail lands on ch 99, NOT on ch 199: the monitor's ch-199 arm
/// (`Mission_MonitorObjectives` 0x1827d8:113-115, FailLabel 0x0200_005a) runs only from
/// Mission_Update internal 1 [M 0x182c60:22; the decompile caller sweep — no other caller],
/// which requires ch 99 clear — dead from armed+2799. The state-3 ch 199 still lands at p1
/// t1200 = armed+2999 [M 0x1aa360-3c4 + D END 1200], inside the 300-tick fail window (only the
/// player pauses [M 0x182c60:124]), with FailLabel at the init sentinel 0 [M 0x182270:27].
/// The old test pinned `fail_label == 0x0200_005a` here and was WRONG — fixed against the
/// decompile.
#[test]
fn the_missile_launch_fails_the_mission_on_ch_99() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free");
        sim.drones.channels.set(19, 1, sim.drones.frame);
    }
    // Test setup: remove the MissileDissarm SPs #213–220 (0-based 212..219) so the console
    // channels 1..8 never rise — the proven precondition for the [M] state-2 launch branch.
    scene.sim.as_mut().unwrap().drones.scripts.players.retain(|p| !(212..=219).contains(&p.placement));
    // ch 99 at armed+2799 (missiles 3/4/5: p0 state-1 tick 1799 + p1 t1000) [D]. Mission_Update
    // runs ahead of the missile tick in the frame [M Game_Run 0x1c9790's order], so the fail flow
    // observes ch 99 on the tick after the write — settle 3 ticks before the state pins.
    let mut failed = false;
    for _ in 0..4000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(99) { failed = true; break; }
    }
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(failed, "the p1's ch 99 = 1 @t1000 fails the mission [D 060007c6/060007e1 + M 0x182c60:23-33]");
        assert_eq!(sim.drones.mission.status(), 2, "MissionState 2 on ch 99 [M 0x182c60:45-57]");
        assert_eq!(sim.drones.mission.fail_label, 0,
            "FailLabel stays the init sentinel — the monitor only runs in internal 1 [M 0x182c60:22, 0x182270:27]");
        assert_eq!(sim.drones.missile_deploy, [1, 1, 3, 3, 3, 3, 1, 3],
            "missiles 3/4/5 (p0 1800) and 6/8 (2400) took the launch branch by the fail; 1/2/7 (3300) are still armed in state 1 [D P0_LEN + M 0x1aa31c/0x1aa210-238]");
        assert_eq!(sim.drones.space_missiles.iter().find(|m| m.index == 4).unwrap().launch_flag, 1,
            "missile 4's obj+0xf6 = 1 on the launch path [M 0x1aa31c]");
        for c in 21..=28u32 {
            assert!(!sim.drones.channels.on(c),
                "no done channel rises on the all-launch path [M 0x1aa354's 0xf6==0 arm never runs]");
        }
    }
    // The state-3 ch 199 still lands at armed+2999, inside the fail window [M 0x1aa360-3c4 +
    // D END 1200] — and the dead monitor leaves FailLabel 0: the 0x200005a arm is unreachable.
    let mut abort = false;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(199) { abort = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(abort, "the missile's state-3 end writes ch[p5] = 199 at p1 t1200 [M 0x1aa360-3c4 + D END 1200]");
        assert_eq!(sim.drones.mission.fail_label, 0,
            "sister: the ch-199 FailLabel 0x0200_005a arm is unreachable — the monitor is gated to internal 1 [M 0x182c60:22]");
        assert!(!sim.drones.mission.menu_open,
            "the fail menu waits out the 300-tick window [M 0x182c60:123-130, 77-81]");
    }
}

/// Drake's fight init [M 0x14b300:236-243 + :722-739]: the zero-g skins (0x05000078/94/ba) set
/// skin kind 0x18, which forces mode +0xC5 = 0x1d (DroneTypeSettings row 29: start 0xbd
/// AstronautLaunch [L 0x29b300 + 0xbd·4]) and the health `DroneInit_TakeXHits(hits, 0x33)` =
/// weapon_data[0x33]+0xc × (difficulty==1 ? 1 : 2), ×3 for Drake [M 0x14d928, :734-739] — 60.0 at
/// the port's boot difficulty 2. Before this force the port ran Drake as a generic mode-0 guard
/// (none of the fight's mechanics worked) — he entered Idle and
/// fell through the station in gravity. The zero-g drift/thruster movement itself
/// (`DroneMove_Astronaut*`) is s/astronaut-guards-1b's family, not ported here.
#[test]
fn drake_initializes_as_the_zero_g_boss() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free");
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    let drake = {
        let mut found = None;
        for _ in 0..600u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
            let sim = scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                found = Some(i);
                break;
            }
        }
        found.expect("the gate spawns Drake (spawner #187 [D])")
    };
    // Settle past the spawn tick (the first tick's GLOBAL ENTER routes to the start state).
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[drake];
        assert_eq!(d.mode, 0x1d, "skin kind 0x18 forces mode +0xC5 = 0x1d [M 0x14b300:722]");
        // The astronaut family runs the launch UPDATE's
        // route-done exit → 0xC1 AstronautCombatMove [M 0x16dd28:54] on the first tick (the port's
        // spawn-point hold is "arrived"), so after the settle he is in 0xbd or its successor 0xc1.
        assert!(matches!(d.state, 0xbd | 0xc1), "the start state is AstronautLaunch [M 0x14b300:723; L 0x29b300+0xbd·4] → 0xc1 [M 0x16dd28:54], got {:#x}", d.state);
        assert!((d.health - 60.0).abs() < 1e-3, "TakeXHits(2, 0x33) ×3 = 60.0 at difficulty 2 [M 0x14b300:726-739, 0x14d928], got {}", d.health);
        assert!(d.enabled, "the spawner enables him (his wait-switch 10 fired with the gate [D])");
    }
}
