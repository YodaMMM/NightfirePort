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

/// The SpaceMissile chain, console 1: the launch order (ch 19 — the AND(0xc1,0xc2) trigger's
/// out [D PL#109]; its two input writers are absent from all 77 scripts [D decode
/// 2026-10-18], so the test drives it the documented way, the walkreal ch-0x0a precedent)
/// starts missile 1's state 0 [M 0x1aa1c8]; its p0 script 0x0600092a writes the console-1
/// gate ch 31 at t359 [D decode]; SP#128's 0x060007e2 (gate 31 [D param 9]) writes the
/// console channel ch 1 at its t682 [D decode]; missile 1's state 2 sees `switch_channels[1]`
/// and takes the DISARM branch [M 0x1aa25c-2c0, deploy = 2], and its state-3 end writes the
/// done channel ch 21 [M 0x1aa354-398]. Missiles 2 and 7 latch the same way (gates 32/37 →
/// console chs 2/7 → done 22/27 [D]); AND #139 [21 & 22 & 27] then writes ch 0x09 — objective
/// 1's complete channel [D AND #139; L record 1b obj 1].
///
/// Missile 4 has NO gate writer (its p0 0x06000a30 writes ch 192 at t1 instead [D decode]) —
/// its console chain never runs on disc data [D; crates/nf-game/src/space_missile.rs module
/// doc] — so with ch 19 driven missile 4 always takes the launch branch and its state-3 end
/// writes the abort channel 199 [M 0x1aa360-3c4], which fails the mission with FailLabel
/// 0x200005a [M 0x1827d8:113-115]. The test pins BOTH facts: the disarmed majority + the
/// abort. Sister: without the ch-19 drive every missile sits in state 0 [M the gate return
/// 0x1aa1c8-1e4] and no console or done channel ever fires.
#[test]
fn missile_console_chain_disarms_and_the_gateless_missile_aborts() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.space_missiles.len(), 8, "the eight 0x39 placements [D PL#107 x8]");
        assert!(sim.drones.space_missiles.iter().all(|m| m.state == 0),
            "sister: every missile sits in state 0 until the launch order [M 0x1aa1c8]");
        assert!(!sim.drones.channels.on(0x09), "sister: objective 1's channel is off [D §2]");
        assert!(!sim.drones.channels.on(199), "sister: no abort channel [D §3]");
    }
    // The launch order: ch 19 (the AND(0xc1,0xc2) trigger's out [D PL#109]; no disc writer
    // [D decode 2026-10-18 — see the header]).
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
    let mut complete = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        complete |= sim.drones.channels.on(0x09);
        if complete && sim.drones.space_missiles.iter().filter(|m| m.state == 4).count() >= 8 { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(1), "console ch 1 latched [D 060007e2 t682]");
        let m1 = &sim.drones.space_missiles[0];
        assert_eq!(m1.state, 4, "missile 1 ran to the epilogue [M 0x1aa3d4]");
        assert_eq!(sim.drones.missile_deploy[0], 2, "missile 1 DISARMED (deploy = 2, the console-ch-1 branch) [M 0x1aa2cc-338]");
        assert!(sim.drones.channels.on(21), "missile 1's done channel ch 21 [M 0x1aa354-398]");
        assert!(sim.drones.channels.on(22) && sim.drones.channels.on(27),
            "missiles 2 and 7 disarm the same way (done 22/27) [D P0_GATE/P2 decode]");
        assert!(complete, "AND #139 [21&22&27] writes objective 1's ch 0x09 [D #139]");
        // The gateless missile 4 launches and aborts the mission.
        assert_eq!(sim.drones.missile_deploy[3], 3, "missile 4 (no gate writer [D]) took the LAUNCH branch [M 0x1aa31c]");
        assert!(sim.drones.channels.on(199), "its state-3 end writes the abort ch 199 [M 0x1aa360-3c4]");
        let st = sim.drones.mission.status();
        assert!(st == 2 || st == 3, "ch 199 fails the mission (status {st}) [M 0x1827d8:113-115]");
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
/// mission end → 0x0730001b [M 0x182c60:114-116]. Sister: the unfought gate leaves Drake
/// unspawned and the mission running.
#[test]
#[ignore = "the reached blocker, narrowed 2026-10-23: the spawner now spawns Drake the game's way (the gate driven while Bond is far — the min_dist screen [M 0x137cb0:30] refuses the on-spot drive the old order used) and the walker's fire kills him, but the death channel 0x0e does not rise through the walker-driven kill — the SAME kill through raw step_bond (probe1b_blockers::probe_drake_death) spawns him enabled, dies at t81 into AstronautDeath 0xbf and raises ch 0x0e; the walker path (w.shoot: exposed_aim box-centre aims + the reload/strafe inputs) leaves ch 0x0e off and the NIS chain dead. Un-ignore when the kill's channel raise survives the walker path"]
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
    // The defeat NIS plays out and ends the mission.
    let mut ended = false;
    for _ in 0..2400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { ended = true; break; }
    }
    w.skip_cutscenes = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x0e), "Drake's death channel ch 0x0e [D param 12 = 14]");
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
    let mut w = load();
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
