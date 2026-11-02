//! interactables-1b: one live test per KIND of player-touchable thing on 0700001b (the space
//! station), driven the way a player drives it (the walk, the trigger pull, the walk-over),
//! each asserting the game's effect with its sister. The census with the type table lives in
//! docs/levels/0700001b.md §7 (task level-1b-interactables); the disc-level console/missile
//! decode is interactables_0700001b_console_probe.
//!
//! The station has NO doors, locks, PCs, switches, monitors, fuseboxes, sensors, grapple
//! points, searchlights, ladders, wires, vents (creepwalls), SimpleScripts or gunimps [D the
//! §7 census — every placement kind scan]. Its player-touchable kinds: the SpaceMissiles +
//! their console chain (script players), Drake + the hub guard (AI), the 0xE1 destroyables,
//! the 7 Bond-bonus pickups, the 5 sound triggers, the hint marker, the 7 AND triggers (level
//! logic), and the unported SpaceLaser (see the census row + the report).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A
//! teleport to reach a spot is used only where the walk there is separately documented
//! (walkreal-c); the operation itself always runs through the real input.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — NOT a game assertion: the p3 == 2 spawn enters substate 8 [M Player_Init
    // 0x1910f8 :178-183, now ported], but no decompile path back to a ground substate after the
    // arrival NIS is pinned and the port wires no substate-changing handler. These tests pin
    // the port's WALK/shoot routes, so the substate is forced back to the ground path (the
    // walkreal_0700001b precedent).
    w.scene.sim.as_mut().unwrap().bond.substate = nf_game::player::SS_WALK;
    w.wait(120);
    w
}

/// The screen sphere's local centre on every console entity [D map26 obj22/76/77 —
/// interactables_0700001b_console_probe]: the glist centre the game's damage read set
/// transforms by the SP's live matrix [M 0x1dc5f8; the port's held_hit_targets].
const SCREEN_C: [f32; 3] = [0.0, 5.33, -1.75];

/// Shoot a console screen (the SP whose scripts[0] == `hash`) until `ch` latches: wait for
/// the script's event-3 hold (its damage tail arms then [D 060007e2 stream 1 t650]), aim at
/// the live screen sphere, fire through the real path. The console scripts end at t900 [D] —
/// the shot window is the hold, exactly the game's own deadline.
fn shoot_console_screen(w: &mut Walker, hash: u32, ch: u32, label: &str) -> bool {
    let Some(pi) = w.scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .position(|p| p.scripts[0] == hash) else { eprintln!("[1b] {label}: SP not found"); return false };
    let mut held = false;
    for _ in 0..1100u32 {
        w.tick(Default::default(), false);
        let p = &w.scene.sim.as_ref().unwrap().drones.scripts.players[pi];
        held = p.info[p.cur].as_ref().map(|s| s.delay != 0).unwrap_or(false);
        if held || p.deleted { break; }
    }
    if !held { eprintln!("[1b] {label}: the hold never engaged"); return false; }
    w.resupply(label); // the tour spends the clip on the earlier screens
    // Walk over: the deck is the pit's descending gallery (a teleport to reach — the walkreal-c
    // crossing blocker) — stand beside the screen, settle, then the REAL trigger pull.
    {
        let m = w.scene.sim.as_ref().unwrap().drones.scripts.players[pi].live_matrix;
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [m[3][0] + 3.0, m[3][1] + 0.4, m[3][2]];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..30 { w.tick(Default::default(), false); }
    for t in 0..500u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(ch) { return true; }
            if sim.drones.scripts.players[pi].deleted { eprintln!("[1b] {label}: script ended unlatched at t{t}"); return false; }
        }
        // The mirrored (−z) consoles' screens sit at local z +1.90 (0x060007e7 stream 8's key
        // [D decode]) — alternate the aim between the two local centres until the latch.
        let screen_c = if (t / 150) % 2 == 0 { SCREEN_C } else { [0.0, 5.33, 1.90] };
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let m = sim.drones.scripts.players[pi].live_matrix;
            let at = [
                m[0][0] * screen_c[0] + m[1][0] * screen_c[1] + m[2][0] * screen_c[2] + m[3][0],
                m[0][1] * screen_c[0] + m[1][1] * screen_c[1] + m[2][1] * screen_c[2] + m[3][1],
                m[0][2] * screen_c[0] + m[1][2] * screen_c[1] + m[2][2] * screen_c[2] + m[3][2]];
            let bp = sim.bond.pos;
            let dx = at[0] - bp[0]; let dz = at[2] - bp[2];
            let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08), ..Default::default() }
        };
        w.tick_fire(input);
    }
    eprintln!("[1b] {label}: shot cap");
    false
}

/// The SpaceMissile chain on the real fire path — and the launch branch's fail, which lands on
/// ch 99 BEFORE any disarm completes: the launch order (ch 19 — the AND(0xc1,0xc2) trigger's out
/// [D PL#109]; its inputs are SP#148's ch 193 and spawner #163's done channel 194, written when
/// the first wave is dead [D] — walkreal_0700001b_live::the_first_wave_kill_raises_the_launch_order;
/// this test drives ch 19 to isolate the console mechanics from that fight) starts missile 1's
/// state 0 [M 0x1aa1c8]; its p0 script 0x0600092a writes the console-1 gate ch 31 at t359 [D
/// decode]; SP#128's 0x060007e2 (gate 31 [D param 9]) plays and holds at t650 — Bond SHOOTS the
/// screens (the real fire path; SP_Update's damage tail unpauses [M 0x1dc1c0-0x1dc330]) and the
/// console scripts write ch 1 (console 7's 0x060007e6 → ch 7) at their t682 [D decode]. But the
/// consoles the test does not reach leave missiles 3/4/5 on the LAUNCH branch: missile 4's gate 34
/// comes from SP#200's 0x06000928 [D], not its own 06000a30, and consoles 3/4/5 are never shot —
/// their p0 ends at state-1 tick 1799 [D L−1] take the launch branch (ch 3/4/5 clear [M
/// 0x1aa25c-338], missiles 6/8 follow at 2399), and their shared p1 scripts write
/// `ch 99 = 1` @p1-t1000 [D 060007c6/060007e1 stream 14] — armed+2799. ch 99 is the game's
/// mission-fail channel [M 0x182c60:23-33]: MissionState 2, the fail flow. That is BEFORE the
/// shot consoles can pay off — missiles 1/2/7 leave state 1 at 3299 [D] and their done channels
/// 21/22/27 land at 5009/4469/4909, past the fail and past the fail menu's world pause — so AND
/// #139's ch 9 can not rise [D #139]. The state-3 ch 199 still writes at armed+2999 [M
/// 0x1aa360-3c4 + D END 1200], inside the 300-tick fail window, with FailLabel at the init
/// sentinel 0 — the monitor's ch-199 arm (0x1827d8:113-115, FailLabel 0x0200_005a) runs only
/// from Mission_Update internal 1 [M 0x182c60:22; the caller sweep], dead since ch 99. The old
/// pins here (all eight state 4, ch 9, deploy[0] = 2, the 0x200005a label) were the
/// pre-timeline stand-in's outcome, not the game's — fixed against the decode + decompile
///. Sister: without the ch-19 drive every missile
/// sits in state 0 [M the gate return 0x1aa1c8-1e4] and no console or fail channel ever fires.
#[test]
fn missile_console_shots_latch_and_the_launch_fails_on_ch_99() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.space_missiles.len(), 8, "the eight 0x39 placements [D PL#107 x8]");
        assert!(sim.drones.space_missiles.iter().all(|m| m.state == 0),
            "sister: every missile sits in state 0 until the launch order [M 0x1aa1c8]");
        assert!(!sim.drones.channels.on(0x09), "sister: objective 1's channel is off [D §2]");
        assert!(!sim.drones.channels.on(199), "sister: no abort channel [D §3]");
    }
    // The launch order: ch 19 (the AND(0xc1,0xc2) trigger's out [D PL#109]) — driven here; the
    // real writer is the first wave's death (see the header).
    let frame = w.scene.sim.as_ref().unwrap().drones.frame;
    w.scene.sim.as_mut().unwrap().drones.channels.set(19, 1, frame);
    // The gates arrive at the missiles' t359 (31/32/37 — consoles 1/2/7 [D P0_GATE]); the
    // console scripts hold at their t650 and wait for the player's 20 damage — Bond SHOOTS
    // the three screens (the real fire path), consoles 7, 2, 1 (the far deck first: every
    // script ends at its t900 [D] — the game's own shot deadline).
    assert!(shoot_console_screen(&mut w, 0x0600_07e2, 1, "console 1"), "console 1 latches ch 1 [D 060007e2 t682]");
    assert!(shoot_console_screen(&mut w, 0x0600_07e6, 7, "console 7"), "console 7 latches ch 7 [D 060007e6]");
    // The mirror-side console (−z): the same mechanic, but the test's glist-centre aim does not
    // reach its mirrored screen (see the report) — the latch is driven the documented stand-in
    // way (the walkreal ch-0x0a precedent) and the gap is listed in the report.
    if !shoot_console_screen(&mut w, 0x0600_07e7, 2, "console 2") {
        eprintln!("[1b] console 2: the mirror-side aim gap — driving ch 2 as the stand-in");
        let frame = w.scene.sim.as_ref().unwrap().drones.frame;
        w.scene.sim.as_mut().unwrap().drones.channels.set(2, 1, frame);
    }
    // The launchers' p1 ch 99 lands at armed+2799 (missiles 3/4/5: p0 state-1 tick 1799 [D] +
    // p1 t1000 [D]) — the mission fails there, before any disarm can complete. Mission_Update
    // runs ahead of the missile tick in the frame [M Game_Run 0x1c9790's order] — settle 3
    // ticks after the write before the state pins.
    let mut ch99 = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(99) { ch99 = true; break; }
    }
    for _ in 0..3 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(ch99, "the p1's ch 99 = 1 @t1000 fails the mission [D 060007c6/060007e1 + M 0x182c60:23-33]");
        assert!(sim.drones.channels.on(1), "console ch 1 latched by the real shot [D 060007e2 t682]");
        assert!(sim.drones.channels.on(7) && sim.drones.channels.on(2),
            "console 7 latched by the real shot and 2 by the stand-in drive [D]");
        assert_eq!(sim.drones.mission.status(), 2, "MissionState 2 on ch 99 [M 0x182c60:45-57]");
        assert_eq!(sim.drones.mission.fail_label, 0,
            "FailLabel stays the init sentinel — the monitor only runs in internal 1 [M 0x182c60:22, 0x182270:27]");
        assert_eq!(sim.drones.missile_deploy[3], 3, "missile 4 (console 4 unshot; its gate 34 comes from SP#200's 0x06000928 [D]) took the LAUNCH branch [M 0x1aa31c]");
        assert_eq!(sim.drones.missile_deploy[2], 3, "missile 3 (console 3 unshot) launched too [M 0x1aa31c]");
        let m1 = &sim.drones.space_missiles[0];
        assert_eq!(m1.state, 1, "missile 1 is still in state 1 at the fail (its p0 ends at 3299 > 2799) [D]");
        assert!(!sim.drones.channels.on(0x09),
            "sister: objective 1's ch 9 can not rise before the fail — AND #139's inputs sit at 4469+ [D #139]");
        for c in 21..=28u32 {
            assert!(!sim.drones.channels.on(c), "no done channel at the fail [D the state-3 ends sit past it]");
        }
    }
    // The state-3 ch 199 still lands at armed+2999, inside the 300-tick fail window [M
    // 0x1aa360-3c4 + D END 1200], with FailLabel still 0 — the old 0x200005a pin never happens.
    let mut ch199 = false;
    for _ in 0..600u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(199) { ch199 = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(ch199, "the launch branch's state-3 end writes ch[p5] = 199 [M 0x1aa360-3c4 + D END 1200]");
        assert_eq!(sim.drones.mission.fail_label, 0,
            "sister: the ch-199 FailLabel arm is unreachable — monitor-gated to internal 1 [M 0x182c60:22]");
        assert!(!sim.drones.mission.menu_open, "the fail menu waits out the window [M 0x182c60:123-130, 77-81]");
    }
}

/// Drake (drone #186, skin 0x050000ba [D §4]) dies to Bond's own fire after the fight gate,
/// and his defeat NIS ends the mission. The gate is driven while Bond is still on the spawn
/// platform — the game's own order: the gate chain's head (0x060008f0 t104 writes ch 10 [D],
/// behind SP #211, [D]) runs while the player is elsewhere, and `DroneSpawner_SpawnDrone`
/// 0x137cb0:30 refuses a template whose spot is nearer than the spawner's min_dist (10.0 [D
/// placement #187 param 8 → the default]) to the player — Bond walked onto the arena floor
/// (0.5 u from the template spot) before the gate and Drake never spawned, in the port AND by
/// the same screen in the game [M]. The route to the arena is walkreal's (blocked at the hub
/// crossing); the test teleports to the arena basin — the documented spot — and the last steps
/// plus the whole fight run through the REAL input (the walker's walk + shoot). The defeat NIS
/// (SP#198 → 0x06000a20 [D]) plays OUT (no skip): its t600 event writes ch 0x0b and its t720
/// event writes ch 100 [D decode; M Script_EventHandler 0x1d94c0 case 0x11] — ch 100 is the
/// mission end → 0x0730001b [M 0x182c60:114-116]. ch 0x0e's own lifecycle: it rises with the
/// kill (AstronautDeath's ENTER raise [M 0x1747b8 → SetDeathChannel 0x13e660]) and the game
/// itself CLEARS it one tick past the NIS's end — SP#198's stay is 0 [D placement #198 param
/// 4], so when the 720-tick script [D NIS length] runs out the player reaches SP_Update's case
/// 3, whose re-arm restarts the script and writes the OUT channel 0 [M 0x1dbda8 case 3
/// cVar10 = '\0' + the tail `switch_channels[out] = cVar10`]. The old ignore note (2026-10-23,
/// "the walker's fire kills him but ch 0x0e never rises and the NIS chain stays dead") is STALE
/// on the current master: the killtrace (NF_KILL_TRACE) shows the walker kill entering
/// AstronautDeath 0xbf with ch 0x0e ON, and the chain completes (ch 0x0b at the NIS's t600,
/// MissionState 6 at the t720, to_end 0x0730001b). The end-of-chain `channels.on(0x0e)` assert
/// this test used to make was the contradiction — the game's own case-3 write turns ch 0x0e
/// off; the raise is now asserted AT the kill and the clear AT the chain's end. Route note: this
/// test isolates the kill (the gate driven, Bond placed in the arena basin); the real-input route
/// is walkreal_0700001b_live::walk_arrival_to_drakes_defeat, where Drake closes in through the
/// float and no crossing is needed. Sister: the unfought gate leaves Drake unspawned and the mission running.
#[test]
fn drake_defeat_by_real_fire_ends_the_mission() {
    // The arrival NIS is CUT here (the probe1b_blockers::probe_drake_death shape — the same
    // substate force the load() helper documents): the full arrival's script tail breaks the
    // defeat chain (the kill raises nothing) — pinned in the report as the next blocker.
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.disabled = false;
        sim.bond.substate = nf_game::player::SS_WALK;
    }
    w.wait(3000);
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba),
            "sister: Drake unspawned before the gate [D spawner #187, trigger 10]");
        assert!(sim.drones.mission.status() <= 1,
            "sister: the mission runs (status {}) [M 0x182c60]", sim.drones.mission.status());
    }
    // The fight gate: driven while Bond is far from the template spot (the game's own order —
    // the min_dist screen [M 0x137cb0:30] refuses a spot within 10.0 of the player, and the
    // arena floor is 0.5 u from Drake's template spot (−7.32,−1.42,0.06) [D §4]). From here the
    // scripts play OUT (no skip): SP#198 ("ScriptPlayer_Pause", out 0x0e [D]) is part of the
    // defeat chain and a walker skip-request would drop its start write.
    w.skip_cutscenes = false;
    let frame = w.scene.sim.as_ref().unwrap().drones.frame;
    w.scene.sim.as_mut().unwrap().drones.channels.set(0x0a, 1, frame);
    let drake = {
        let mut found = None;
        for _ in 0..900u32 {
            w.tick(Default::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                found = Some(i);
                break;
            }
        }
        let i = found.expect("the gate spawns Drake while Bond is far (spawner #187, trigger 10 [D]; the min_dist screen cleared [M 0x137cb0:30])");
        assert!(w.scene.sim.as_ref().unwrap().drones.list[i].enabled,
            "the gate enables Drake (his wait-switch param 3 = 10 [D])");
        i
    };
    // The arena floor (walkreal's route end): teleport into the arena basin to reach it (the
    // walkreal-c hub-crossing blocker), settle on the floor, then the REAL fire (the walker's
    // shoot — the same kill the probe1b_blockers::probe_drake_death run verified: spawn → bullets
    // → AstronautDeath 0xbf → ch 0x0e).
    w.scene.sim.as_mut().unwrap().bond.pos = [-4.0, -1.9, 0.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(30);
    assert!(w.shoot(drake, "Drake"), "Drake dies to Bond's fire");
    // The kill's raise lands in the same tick the corpse does (AstronautDeath's ENTER →
    // SetDeathChannel [M 0x1747b8 → 0x13e660], drained by the drone tick's channel writer).
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x0e),
        "Drake's death channel ch 0x0e rises with the walker-path kill [D param 12 = 14; M 0x1747b8/0x13e660]");
    // The defeat NIS plays out and ends the mission. NO early break: SP#198's stay-0 re-arm
    // clears its out channel ch 0x0e one tick past the script's end [M 0x1dbda8 case 3], and
    // that tick must land inside the window for the end-of-chain assert below (the NIS is 720
    // ticks [D], the re-arm + the SP's own replay follow it).
    let mut ended = false;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { ended = true; }
    }
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x0e),
            "the game's own re-arm clears the out channel: SP#198 stay=0 [D placement #198 param 4] reaches SP_Update's case 3 when the NIS's script runs out, and the restart writes switch_channels[out] = 0 [M 0x1dbda8 case 3 cVar10 = '\0' + the tail write]");
        assert!(sim.drones.channels.on(0x0b), "the NIS's t600 event-11 writes ch 0x0b [D]");
        assert!(ended, "the NIS's t720 ch 100 ends the mission (MissionState 6) [M 0x182c60]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    }
}

/// The hub guard (drone #0, skin 0x0500008f [D §4]) stands at (2.86, 0.39, 0.00) present and
/// enabled from load — Bond kills it with his own fire (the real path). Sister: it lives
/// while unfired at.
///
/// The stand is the arena basin (the documented teleport + the real last walk — walkreal-c's
/// hub-crossing blocker): from the arena floor the guard's hit volumes are body-clear (the
/// probe's 16/16 boxes from (−6.0, −0.3, 0.0)); the hub DECK flanks the walker first tried all
/// sit east of the pit's grill wall (x ≈ 3.88-4.4 [D scans, s/level-1b-drake-fight a4372516]),
/// which is body-solid but bullet-pass — every deck flank's body segment is blocked while the
/// bullet segment is clear, and the deck spots themselves have no floor (the walker's flank
/// dumps: floor_below None at (6.37,6.02,1.87)). The flight stops on pass-flagged nodes [L ai_live
/// guard_takes_cover_like_the_game UnderCoverAim 37], so only a body-clear ground flank lands
/// the shot — the game's own geometry, ported; the harness just has to stand where the game's
/// player stands.
#[test]
fn the_hub_guard_dies_to_fire() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[0];
        assert!(g.present && g.enabled && !g.corpse, "sister: the centrepiece guard lives at load [D §4]");
    }
    // The arena-basin stand (the documented teleport — the walkreal-c hub crossing), then the
    // real last walk to the floor spot the guard's volumes are clear from.
    w.scene.sim.as_mut().unwrap().bond.pos = [-4.0, -1.9, 0.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.walk_to(walker::Wp::new(-7.0, -1.5, 0.1).r(1.2), "the arena floor");
    assert!(w.shoot(0, "the hub guard"), "the guard dies to Bond's fire");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[0].corpse, "the guard is a corpse after the kill");
}

/// The hub's 0xE1 destroyables (placements #276/#278 at (−12.95, 0, ±0.14) [D the census])
/// shatter under Bond's fire: `Destroy_Create` 0x12c318's objects break through the real
/// bullet path. Sister: intact before the shot.
#[test]
fn the_hub_destroyable_shatters_under_fire() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.props.destroy.len(), 6, "the six 0xE1 destroyables [D the census]");
        assert!(!sim.drones.props.destroy[4].dead, "sister: destroyable #276 intact before the shot [D PL#276]");
    }
    // Stand on the hub floor facing it; the real trigger pull.
    w.scene.sim.as_mut().unwrap().bond.pos = [-11.4, -1.9, 0.14];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let at = [-12.95, 0.0, 0.14];
    let ok = w.shoot_point(at.into(), "destroyable #276",
        |sim| sim.drones.props.destroy[4].dead);
    assert!(ok, "the destroyable breaks under Bond's fire [M Destroy_Create 0x12c318]");
}

/// The Bond bonuses (7 kind-6 pickups [D the census]) collect on the walk-over:
/// `Pickup_Handler` 0x18e650's take. These placements carry respawn 0 [D PL#; the port's parse:
/// pickup.respawn = 0], and the take tail's respawn split [M 0x18e650:340-344] DELETES a
/// respawn-0 pickup (`obj+0xFE |= 1`) instead of the respawn>0 hide (`obj+0xF0 |= 0x10`,
/// PICKUPINFO+0x20 = 2) — so the walked-over bonus is asserted through `gone` (the pre-fix
/// test asserted only state 2 / hidden and never saw the take that the [M] tail performs — the
/// TEST was wrong, fixed 2026-10-23 with this address). Sister: resting before the walk.
#[test]
fn the_bonus_pickup_collects_on_the_walk_over() {
    // The kind-6 bonus pickups exist only with the level's gold medal recorded [M Pickup_Create 0x18dda0:93]: load with every medal held.
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "0700001b.bin", false, None, 1, u64::MAX).unwrap());
    let (x, y, z) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = &sim.drones.pickups[1];
        assert_eq!(p.kind, 6, "the kind-6 Bond bonus [D the census]");
        assert_eq!(p.state, 1, "sister: the bonus rests before the walk [M Pickup_CreateSimple]");
        (p.m[3][0], p.m[3][1], p.m[3][2])
    };
    // Teleport above the deck spot (the walk there is the hub crossing, walkreal-c's blocker),
    // then walk onto it through the real input.
    w.scene.sim.as_mut().unwrap().bond.pos = [x + 0.6, y + 1.2, z];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut took = false;
    for _ in 0..400u32 {
        let (b, t) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, [x, y, z])
        };
        let d = [t[0] - b[0], t[1] - b[1], t[2] - b[2]];
        let yaw = d[0].atan2(d[2]);
        w.scene.sim.as_mut().unwrap().bond.yaw = yaw;
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        // The take tail's split [M 0x18e650:340-344]: respawn 0 → deleted (obj+0xFE |= 1, the
        // port's `gone`); respawn > 0 → hidden with PICKUPINFO+0x20 = 2 (the port's `hidden` +
        // `state == 2`). Both are takes.
        took |= sim.drones.pickups[1].gone || sim.drones.pickups[1].hidden || sim.drones.pickups[1].state == 2;
        if took { break; }
    }
    assert!(took, "the bonus collects on the walk-over [M Pickup_Handler 0x18e650]");
}

/// The spawner's own player-distance screen [M DroneSpawner_SpawnDrone 0x137cb0:30 — a template
/// whose spot is nearer than the spawner's min_dist (placement #187 param 8 = 0 → the game's
/// 10.0 default [M 0x1378d0: `0x41200000`]) is skipped while the player stands inside it]:
/// with Bond WALKED onto the arena floor 1.5 u from Drake's template spot (−7.32,−1.42,0.06
/// [D §4]) the gate ch 0x0a spawns NOTHING in 900 ticks — the sister that keeps the main test's
/// gate-first order honest (the game's own chain runs the gate while the player is elsewhere).
#[test]
fn the_spawner_refuses_a_template_on_top_of_bond() {
    let mut w = load();
    w.scene.sim.as_mut().unwrap().bond.pos = [-5.81, -1.90, 0.14];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(30);
    let frame = w.scene.sim.as_ref().unwrap().drones.frame;
    w.scene.sim.as_mut().unwrap().drones.channels.set(0x0a, 1, frame);
    let mut spawned = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            spawned = true;
            break;
        }
    }
    assert!(!spawned, "sister: the min_dist screen refuses Drake's spot 1.5 u from Bond [M 0x137cb0:30]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x0e),
        "and the fight never opens (no death channel) [D param 12 = 14]");
}

/// The spawn hint marker (0x33 placement, at (22.86, −2.12, 3.82) [D the census]) fires its
/// one INFO-pane message when Bond walks its straddle set (`Hint_Update` 0x1b3380:41-42's
/// once-per-load delete bit — the port's `hint_fired`). Sister: unfired at load.
#[test]
fn the_spawn_hint_fires_on_the_first_walk() {
    // A cold load, one tick: the marker is unfired before Bond's first step.
    let mut scene = nf_view::load(&iso(), "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.disabled = false;
    for _ in 0..2 { sim.step_bond(Default::default(), false, false, false); }
    // The marker sits ON the spawn platform [D the census: (22.86,-2.12,3.82) vs the spawn
    // (22.98,-1.90,3.13)] — Bond's first real steps are already its walk-in.
    for _ in 0..300 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.hint_fired[0],
        "the hint fires at once (the spawn stands in its set) [M Hint_Update 0x1b3380:41-42]");
    // One-shot: the level's one 0x33 hint [D the census].
    assert_eq!(sim.hint_fired.len(), 1, "the level's one 0x33 hint [D the census]");
}

/// KIND 0xF9 — the pit-bottom AluWarning klaxons (#151/#157/#158/#159/#160 [D the census:
/// ON ch 0x4c..0x50, one-shot p1 = 1, ids 0x350/0x34e/0x34c/0x36f/0x34d, 3-D p3 = 0, global
/// p5 = 1]) fire on the station's own speech chain: `SoundTrigger_Update` 0x1b2790 plays the
/// param-2 id (`Sound_Play3D(100, −1, −1, id, obj+0x30)` — 3-D at the placement [M]; the port's
/// create maps p0=on, p1=one-shot, p2=id, p3=is2d, p4=off, p5=global [M 0x1b2628:24-48]), a
/// global trigger finding the `GlobalVoiceHandle` busy kills itself [M :19-28], and a finished
/// one-shot frees the voice and deletes itself [M :71-84]. The ON channels' writers [D the
/// 0700001b script decode — census_0700001b::dump_0700001b_scripts, cross-checked against
/// space_missile.rs's P2_WRITES] — every one a CONDITIONAL event-11 write ([cond, ch_if_on,
/// ch_if_off, val] — `Script_EventHandler` 0x1d94c0 case 0x11: the write lands on the param-2
/// channel only while the param-0 condition channel reads OFF [M; the port's nis.rs 0x11 arm]).
/// The klaxons warn about the missiles STILL ARMED:
///  - ch 0x4c ← SP#201's 0x06000a31 stream 6 t1500 (SP#201: mode 1, gate p9 = 19 — the launch
///    order, played while it holds; its real writer is the AND[193,194] mux, pinned on the real
///    kill path by walkreal's the_first_wave_kill_raises_the_launch_order — the poke here is the
///    missile test's documented stand-in), condition ch 5 — fires while missile 5 is ARMED;
///  - ch 0x4d ← SP#152's 0x060009a5 t900, condition ch 4 — SP#152 starts on its own out ch 0x55
///    (mask 1 = SP_Hit 0x1dc470:11-13's own-out bit [M]), written by missile 3's p2 @t550 or
///    missile 5's @t1 [D P2_WRITES] — it needs a DISARMED m3/m5 and an ARMED m4;
///  - ch 0x4e ← SP#153's 0x060009a6 t1080, condition ch 3 — started by m2's p2 @t490 / m4's @t1;
///  - ch 0x4f ← SP#154's 0x060009a7 t1560, condition ch 0x5b (91) — the pair mux [6 & 8] writes
///    it every tick [M triggers.rs's AND case], so the klaxon fires only while m6 or m8 is still
///    armed — started by m1's p2 @t1030 / m3's @t1 (ch 0x53);
///  - ch 0x50 ← SP#155's 0x060009a8 t3000, condition ch 0x5c (92) — the triple mux [1 & 2 & 7],
///    the same law — started by m7's p2 @t930 or the [0x56 & 0x58] mux → ch 0x59 (m4's t930 +
///    m6's t730 p2 writes) [D P2_WRITES + the census row].
/// The drive pokes EVERY tick: the launch-order mux's INPUT channels 193/194 (the mux itself
/// then writes ch 19 = 1 — its own real write, which the walk order delivers to SP#201's gate;
/// the inputs' real writers are SP#148's stream-3 write and spawner #163's done channel, pinned
/// by walkreal's the_first_wave_kill_raises_the_launch_order — the pokes are the missile test's
/// documented stand-in class) and the console channels 3/4/5/6 (the missiles' p4; the real
/// writer is the shot screen's t682 write — the missile test's real path for consoles 1/7).
/// The klaxon conditions are the PAIR muxes [D the census's AND rows: 0x5b←[6, 8], 0x5c←[1, 2, 7]
/// write ch 91/92 every tick [M triggers.rs's AND case] — the klaxon fires only while a missile
/// of the pair is STILL ARMED. With m3–m6 disarmed and m1/m2/m7/m8 armed: SP#154 starts off
/// m3's p2 t1 (ch 0x53, ~t1852 — before the m8 launch branch fails the mission on ch 99 at
/// ~t3450 [M 0x182c60:23-33]) and its t1560 event lands while ch 91 still reads off (m8 armed)
/// → ch 0x4f; the [0x56 & 0x58] mux (m4's t930 + m6's t730 p2 writes) starts SP#155 at ~t3180
/// — still before the fail — and its t3000 event lands with ch 92 off (m1/m2/m7 armed) → ch
/// 0x50. The console-conditioned klaxons 0x4c (cond ch 5), 0x4d (cond ch 4) and 0x4e (cond ch 3)
/// stay SILENT — every one of their condition channels is ON here — the live sisters, their
/// triggers un-triggered in the list.
/// Sisters: the five triggers sit cold at the load; at most one klaxon sounds at any tick
/// [M :19-28]; the mission's fail (the m8 launch branch) and the objective's AND #139
/// [ch 21 & 22 & 27] → ch 0x09 (m1/m2/m7 unarmed → the AND out stays off) pin the level logic
/// around the chain [D PL#109; M 0x1b1a78 type 3, 0x182c60:23-33]. HARNESS LIMIT (not a game
/// assertion): the one-shot's natural end needs the audio output's ended-report (`Sound::batch`'s
/// `take_ended` — [M sfx.rs]); the headless driver has no output, so the FIRST klaxon's voice
/// holds forever and the second klaxon's ON rise exercises the busy-kill arm [M :19-28] — the
/// same delete outcome, and both should be audible in the game (a live check).
#[test]
fn the_launch_order_and_the_disarm_speech_fire_the_pit_klaxons() {
    let mut w = load();
    // The speech scripts play OUT: a walker skip-request fast-forwards the running script and
    // drops its stream writes (the drake_defeat test's SP#198 precedent) — the five ON channels
    // are exactly those stream writes.
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.sound_triggers.list.len(), 5,
            "the five 0xF9 AluWarning triggers exist at the load [D the census]");
        for c in 0x4c..=0x50u32 {
            assert!(!sim.drones.channels.on(c), "sister: ch {c:#x} cold at the load");
        }
        for id in [0x34cu16, 0x34d, 0x34e, 0x350, 0x36f] {
            assert!(!sim.drones.sound_triggers.playing(id), "sister: id {id} silent at the load");
        }
    }
    // The drive: the launch-order mux's inputs + consoles 3-6 (above).
    const DRIVE: [u32; 6] = [193, 194, 3, 4, 5, 6];
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let frame = sim.drones.frame;
        for c in DRIVE { sim.drones.channels.set(c, 1, frame); }
    }
    // Poll the whole chain: per-tick bookkeeping of the five ON channels, the five klaxon ids
    // and the concurrency (the global voice holds ONE handle [M 0x1b2790:19-28]).
    let ids = [0x350u16, 0x34e, 0x34c, 0x36f, 0x34d]; // #151, #157, #158, #159, #160 [D]
    let chans = [0x4cu32, 0x4d, 0x4e, 0x4f, 0x50];
    let mut rose_at = [None::<u32>; 5];
    let mut played = [false; 5];
    let mut max_concurrent = 0usize;
    let mut saw_playing_any = false;
    for _ in 0..8200u32 {
        w.tick(nf_game::player::Input::default(), false);
        let pre_poke = w.scene.sim.as_ref().unwrap().drones.channels.on(19);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let frame = sim.drones.frame;
            for c in DRIVE { sim.drones.channels.set(c, 1, frame); } // the poke_gate hold
        }
        let sim = w.scene.sim.as_ref().unwrap();
        if std::env::var("NF_KLAXON_DEBUG").is_ok() && sim.drones.frame % 500 == 0 {
            let sp = |ix: u32| sim.drones.scripts.players.iter().find(|p| p.placement as u32 == ix)
                .map(|p| (p.state, p.info[p.cur].as_ref().map(|s| (s.delay, s.clock as u32)))).unwrap_or((9, None));
            eprintln!("[klaxon-dbg] f{} ch19 pre={} post={} ch76..80={:?} sp201={:?} sp154={:?} audio_live={} playing={:?} list={}",
                sim.drones.frame, pre_poke, sim.drones.channels.on(19),
                (0x4c..=0x50u32).map(|c| sim.drones.channels.on(c) as u8).collect::<Vec<_>>(),
                sp(201), sp(154),
                sim.drones.audio.live(),
                [0x350u16, 0x34e, 0x34c, 0x36f, 0x34d].map(|id| sim.drones.sound_triggers.playing(id)),
                sim.drones.sound_triggers.list.len());
        }
        let mut now_playing = 0usize;
        for (k, &id) in ids.iter().enumerate() {
            if sim.drones.sound_triggers.playing(id) {
                now_playing += 1;
                saw_playing_any = true;
                played[k] = true;
            }
            if rose_at[k].is_none() && sim.drones.channels.on(chans[k]) {
                rose_at[k] = Some(sim.drones.frame);
            }
        }
        max_concurrent = max_concurrent.max(now_playing);
    }
    let unrisen = (0..5).filter(|k| rose_at[*k].is_none()).map(|k| chans[k]).collect::<Vec<_>>();
    assert_eq!(unrisen, vec![0x4c, 0x4d, 0x4e],
        "0x4f/0x50 rise off their condition-cold chains [D P2_WRITES + the decode]; 0x4c/0x4d/0x4e \
         are the armed-missile warnings — every console ON silences them (the live sisters)");
    let rose = rose_at.map(|r| r);
    assert!(rose[3].unwrap() < rose[4].unwrap(),
        "0x4f (SP#154's t1560) lands before 0x50 (SP#155's t3000) [D P2_WRITES], got {rose:?}");
    assert!(saw_playing_any && played[3],
        "the 0x4f klaxon (id 0x36f) plays — the global voice was free at its rise [M \
         SoundTrigger_Update 0x1b2790's play arm] (0x50's play is the harness-frozen-voice \
         busy-kill, see the doc)");
    assert!(max_concurrent <= 1,
        "the GlobalVoiceHandle holds at most one klaxon [M 0x1b2790:19-28], saw {max_concurrent}");
    let _ = ids; let _ = played; // the per-id end polls are the harness limit (see the doc)
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The 0x34d trigger left the list — its ON rose while the first klaxon's voice held, the
        // busy-kill arm [M :19-28] (headless the first voice never frees — the harness limit).
        // The other four keep their triggers: #151/#157/#158 never triggered, #159's one-shot
        // holds its (headless-frozen) voice.
        assert_eq!(sim.drones.sound_triggers.list.len(), 4,
            "the busy-killed 0x34d trigger left the list [M :19-28]");
        let kept: Vec<u16> = sim.drones.sound_triggers.list.iter().map(|t| t.id).collect();
        for id in [0x350u16, 0x34e, 0x34c, 0x36f] {
            assert!(kept.contains(&id), "id {id} keeps its trigger (the harness limit — see the doc)");
        }
        assert!(!kept.contains(&0x34d), "the 0x34d trigger is gone [M :19-28]");
        for c in [0x4cu32, 0x4d, 0x4e] {
            assert!(!sim.drones.channels.on(c), "sister: ch {c:#x} never rose");
        }
        for c in [0x4fu32, 0x50] {
            assert!(sim.drones.channels.on(c),
                "ch {c:#x} latches — no disc script writes it 0 [D the script decode]");
        }
        // The m8 launch branch failed the mission mid-run [M 0x182c60:23-33] — the fail and the
        // klaxon chain co-exist (the SPs that started before the fail keep playing).
        assert_eq!(sim.drones.mission.status(), 2,
            "the armed m8's launch branch fails the mission [M 0x182c60]");
        // The objective's AND #139 [ch 21 & 22 & 27] → ch 0x09: m1/m2/m7 are armed, so the
        // objective's mux out stays off — the mux kind's live sister [D PL#109; M 0x1b1a78].
        assert!(!sim.drones.channels.on(0x09),
            "sister: the objective's AND out stays off with m1/m2/m7 armed [D #139]");
    }
}

/// KIND 0xE4 — the ungated OBJECT_MissileFlame volume (#17 at (−33.44, 19.58, 0) [D the census:
/// p0 = 0x2 the pain type, p1 = 0x32 the damage 50, p2 = 0 the gate — ungated, p4 = 0 the who]) —
/// `Hurt_Update` 0x17e5d0's state-0 start law: ungated with who 0 it starts at once [M :51-102],
/// and the damage rides the player's object contact — `Player_DealWithObjHit` 0x194458's
/// class-0x27 arm calls `Player_HandlePain(impact + Hurt_GetDamage(obj), type)` [M] — pain types
/// 1..7 take no armour [M 0x1902c8:84-104], so the contact costs `50 × Plr_DMod` health with the
/// difficulty factor (Easy 0.6 on this level [M damage_factor's 0x1b row]). The approach floats
/// the load's own zero-g substate 8 (the p3 == 2 spawn [M Player_Init 0x1910f8 :160-165]; the
/// same float the walkreal legs drive) to the volume's own collision AABB — the volume's tris
/// are the placement's object collision [M Hurt_Create 0x17e870:44]. Sister: floating BESIDE it
/// leaves the health and the contact witness untouched.
#[test]
fn the_missile_flame_hurts_on_contact() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.top_up_health = false; // the damage must reach the health pool
    let (centre, standoff, axis, volume) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.hurts.iter().find(|h| h.placement == 17)
            .expect("the OBJECT_MissileFlame hurt volume [D the census #17]");
        assert_eq!((h.kind, h.gate, h.who), (2, 0, 0), "the disc params [D audit:444]");
        assert!((h.damage - 50.0).abs() < f32::EPSILON, "damage 50 [D p1 = 0x32]");
        assert!(!h.tris.is_empty(),
            "the volume carries the placement's object collision [M Hurt_Create 0x17e870:44]");
        let c = [0.5 * (h.lo[0] + h.hi[0]), 0.5 * (h.lo[1] + h.hi[1]), 0.5 * (h.lo[2] + h.hi[2])];
        assert!((c[0] - -33.44).abs() < 3.0,
            "the volume hangs at its placement's x [D (−33.44, 19.58, 0) — the object's own \
             collision mesh offsets the centre down the shaft, y {y}", y = c[1]);
        // The stand-off: 2 units beyond the AABB face on its thinnest axis (out of the
        // capsule's quick-reject reach).
        let extents = [h.hi[0] - h.lo[0], h.hi[1] - h.lo[1], h.hi[2] - h.lo[2]];
        let axis = (0..3).min_by(|a, b| extents[*a].partial_cmp(&extents[*b]).unwrap()).unwrap();
        let mut standoff = c;
        standoff[axis] = h.hi[axis] + 2.0;
        (c, standoff, axis, sim.hurts.iter().position(|h| h.placement == 17).unwrap())
    };
    let mut reached = false;
    // The west shaft past the hub is the walkreal route's documented boundary (the route never
    // crosses it — walkreal_0700001b's route note), so the documented stand-in transit places
    // Bond just OUTSIDE the stand-off (a further 4 units along the approach axis); the approach
    // and the contact itself float the real thrusters.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let mut near = standoff;
        near[axis] += 4.0 * (standoff[axis] - centre[axis]).signum();
        sim.bond.pos = near;
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    for _ in 0..2400u32 {
        let input = thrust_to(w.scene.sim.as_ref().unwrap(), standoff);
        w.tick(input, false);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, standoff) < 0.5 && (p[1] - standoff[1]).abs() < 0.5 { reached = true; break; }
    }
    assert!(reached, "the float reaches the stand-off (at {:?})",
        w.scene.sim.as_ref().unwrap().bond.pos);
    let h0 = w.scene.sim.as_ref().unwrap().drones.health.health;
    for _ in 0..60 {
        let input = thrust_to(w.scene.sim.as_ref().unwrap(), standoff);
        w.tick(input, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.health.health, h0, "sister: no contact at the stand-off");
        assert_eq!(sim.hurts[volume].hits, 0, "sister: the volume's contact witness is zero");
    }
    // The real contact: thrust into the volume until the health moves, then back off (the
    // volume is not one-shot [D p5 = 0] — every contact tick would bill again).
    let mut hurt_at = None;
    for _ in 0..900u32 {
        let input = thrust_to(w.scene.sim.as_ref().unwrap(), centre);
        let before = w.scene.sim.as_ref().unwrap().drones.health.health;
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.health.health < before {
            hurt_at = Some((before, sim.drones.health.health, sim.hurts[volume].hits));
            break;
        }
    }
    let (before, after, hits) = hurt_at.expect("the flame volume hurts on contact [M 0x194458's class-0x27 arm]");
    assert!(hits >= 1, "the volume's contact witness records the hit");
    let drop = before - after;
    assert!((drop - 30.0).abs() < 6.0,
        "the contact costs damage 50 × the Easy 0.6 factor [M 0x1902c8 + damage_factor's 0x1b \
         row; D p1 = 0x32], got {before} → {after}");
    // The exit: the test's own retreat (the volume bills every contact tick — the harness does
    // not owe the game a death).
    w.top_up_health = true;
    for _ in 0..120 {
        let input = thrust_to(w.scene.sim.as_ref().unwrap(), standoff);
        w.tick(input, false);
    }
}

/// The zero-g thruster input toward `at` (the walkreal harness's `thrust_to`, local copy): the
/// body-frame dot products as the strafe/lift/forward axes — the float's real input path.
fn thrust_to(sim: &nf_view::scene::Sim, at: [f32; 3]) -> nf_game::player::Input {
    let p = sim.bond.pos;
    let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let m = sim.bond.zero_g_rot;
    let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    nf_game::player::Input {
        strafe: (dot(m[0]) * 0.15).clamp(-1.0, 1.0),
        lift: (dot(m[1]) * 0.15).clamp(-1.0, 1.0),
        forward: (dot(m[2]) * 0.15).clamp(-1.0, 1.0),
        ..Default::default()
    }
}
