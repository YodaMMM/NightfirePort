//! fix-spawner-p7 NF_ISO tests + fix-script-start-tick: the spawner's param 7 is the KILL channel,
//! measured live on 07000005 (docs/fields/hitdata-spawner-ally.md "Live result"): ch 0x22 spawns the
//! two gate guards; ch 0x43 (the wall-hug plate) runs SP 298's script (0x06000994 — event 5 freezes
//! the NPCs on the TOUCH TICK, the game's one-walk order), and when the script's end re-enables the
//! NPCs the kill latch removes both guards and the spawner — the placed pair behind the gate swaps
//! in at the same moment. Plus 07000011's stairwell (the fix-actors case): the keep-level
//! split keeps a drawn guard. Every step goes through the real input path: `step_bond` with Bond
//! teleported into the disc's touch volumes. Opt-in: NF_ISO.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn touch_centre(sim: &nf_view::scene::Sim, out: u8) -> [f32; 3] {
    let t = sim.drones.triggers.iter().find(|t| t.out == out)
        .unwrap_or_else(|| panic!("the ch {out:#x} touch exists [D placement 305/301]"));
    [(t.lo[0] + t.hi[0]) * 0.5, (t.lo[1] + t.hi[1]) * 0.5, (t.lo[2] + t.hi[2]) * 0.5]
}

fn group2(sim: &nf_view::scene::Sim) -> Vec<usize> {
    sim.drones.list.iter().enumerate().filter(|(_, d)| d.group == 2).map(|(i, _)| i).collect()
}

fn present(sim: &nf_view::scene::Sim, tpls: &[usize]) -> usize {
    tpls.iter().filter(|&&i| sim.drones.list[i].present).count()
}

/// The live measurement end to end, in the game's visible order [L docs/fields/hitdata-spawner-ally.md
/// "Live result"]: ch 0x22 → the two gate guards spawn; the wall-hug plate runs SP 298's script
/// (0x06000994 — the disc's placement data for placement 298; the live run's loadable-pack variant is
/// 0x06000AD4), whose event 5 freezes the NPCs IN THE TOUCH TICK and the whole ~1,400-tick scene —
/// the gate pair stays present the whole time, the two placed guards behind the gate stay disabled —
/// and when the script's end re-enables the NPCs the kill latch fires: both guards removed, the
/// spawner gone, and the placed pair switch on. The player never sees an empty gate.
#[test]
fn the_wall_hug_scene_holds_the_gate_pair_and_swaps_the_pairs_at_its_end() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    // The placed pair behind the gate: drones at (32.4, 37.0, 49.9) / (27.6, 37.0, 49.7) — the live
    // read's (27.6, 37.2, 49.9) / (32.3, 37.2, 50.1), object flags 0x8b0 (disabled) during the scene
    // and 0x880 after [L hitdata-spawner-ally]. The port's `enabled` is the obj+0xF0 gate's stand-in.
    let placed = |sim: &nf_view::scene::Sim| -> Vec<usize> {
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| (d.pos[0] - 30.0).abs() < 4.0 && (d.pos[2] - 50.0).abs() < 4.0)
            .map(|(i, _)| i).collect()
    };
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        let tpls = group2(sim);
        assert!(!tpls.is_empty(), "the gate spawner's group-2 templates exist [D placement 268]");
        assert!(tpls.iter().all(|&i| !sim.drones.list[i].present), "the spawner holds them back [M 0x137aa0]");
        assert_eq!(placed(sim).len(), 2, "the two placed guards behind the gate exist [L hitdata-spawner-ally]");
        // The trigger: ch 0x22 comes from the OR gate (placement 0xEF #25) over the three
        // level-start touches (placements 21/22/23, out 0x23/24/25 [D probe]) — Bond onto the
        // first one's volume, the real input path to the gate spawner's trigger channel.
        sim.bond.pos = touch_centre(sim, 0x23);
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(sim.drones.channels.on(0x22), "the gate touch fires ch 0x22 (via the OR) [D]");
        // Step Bond away from the spawn spots so the stealth checks (p8 = 4.0 [D]) let them through.
        sim.bond.pos = [104.9, 25.7, 108.8];
    }
    let mut spawned = 0;
    for _ in 0..24u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        spawned = { let sim = scene.sim.as_ref().unwrap(); present(sim, &group2(sim)) };
        if spawned == 2 { break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(spawned, 2, "the two gate guards spawn on ch 0x22 [L: 15 drones, two at the gate]");
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 2).expect("the gate spawner");
        assert!(!sp.gone, "the spawner is alive");
    }
    // Sister-in-advance: without ch 0x43 they stay.
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(present(sim, &group2(sim)), 2, "no kill channel: both guards stay");
    }
    // The kill channel: Bond onto the wall-hug plate (placement 301, out 0x43 [D]) — with the
    // combat event QUIET, as the PINE probe had it (a fresh room, nobody attacking):
    // wait out any yard alert so `SP_Update`'s state-0 gate lets the script start [M 0x1dbda8:96-100].
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [25.0, 34.0, 60.0]; // off the yard's sight lines while the alert decays
    }
    let mut quiet = false;
    for _ in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.prev_event2 == 0 { quiet = true; break; }
    }
    assert!(quiet, "the combat event decayed (the PINE probe's quiet room) [M 0x114e70 clears per frame]");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x43);
    }
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    // THE TOUCH TICK, in the game's walk order [M control_movement_object_handler 0x1336f8:54-68:
    // one newest-first walk runs every control func — the plate #301 writes ch 0x43, then SP #298
    // (placement 298) triggers and `SP_LoadScript`'s trailing `Script_Update` [M 0x1dbc08:56] runs
    // its first commands (event 5 → `Drone_EnableAll(0)` [M 0x1d94c0 case 5]), then the spawner
    // (placement 268) returns at `NPCGlobals == 0` [M 0x137e68:21-23] with its latch still off]:
    // the freeze holds the SAME tick, the gate pair is still present, the spawner alive.
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x43), "the wall-hug plate fires ch 0x43 [D]");
        assert!(!sim.drones.npc_enabled, "the freeze lands on the touch tick (event 5 in the same walk) [L]");
        assert_eq!(present(sim, &group2(sim)), 2, "the gate pair survives the touch tick — the spawner was frozen first [L: no empty gate]");
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 2).expect("the gate spawner");
        assert!(!sp.gone && !sp.kill_latch, "the spawner is alive with its kill latch still off [L: +0x20 stays 0 through the scene]");
        assert!(placed(sim).iter().all(|&i| !sim.drones.list[i].enabled), "the placed pair is disabled by the freeze (obj flags 0x8b0) [L]");
    }
    // MID-SCENE (~300 ticks in, the scene is 1,400 [D script 0x06000994 length]): the same picture.
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.npc_enabled, "the freeze holds through the scene [L]");
        assert_eq!(present(sim, &group2(sim)), 2, "the gate pair is present mid-scene [L: 15 drones throughout]");
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 2).expect("the gate spawner");
        assert!(!sp.kill_latch, "the latch waits (Control returns at its first line) [M 0x137e68:21-23]");
        assert!(placed(sim).iter().all(|&i| !sim.drones.list[i].enabled), "the placed pair stays hidden mid-scene [L]");
    }
    // The script's end re-enables the NPCs (stream 8's event 7 at clock 1398 → `Script_Free`'s 0x80
    // flag [M 0x1da880:41-43]); then the latch fires and the kill scan removes both guards [L: 15 → 13].
    let mut freed = None;
    for k in 0..1500u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.npc_enabled { freed = Some(k); break; }
    }
    assert!(freed.is_some(), "the script's end re-enabled the NPCs (event 7 → Script_Free 0x80) [L]");
    let mut gone_ticks = None;
    for k in 0..30u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if present(sim, &group2(sim)) == 0 { gone_ticks = Some(k); break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(gone_ticks.is_some(), "both gate guards removed once Control runs again [L: 15 → 13]");
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 2).expect("the gate spawner");
        assert!(sp.gone, "the spawner deletes itself [M 0x137e68: obj+0xFE |= 1]");
        assert!(placed(sim).iter().all(|&i| sim.drones.list[i].enabled), "the placed pair switched on at the re-enable (0x8b0 → 0x880) [L]");
    }
    // And it never spawns again.
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(present(sim, &group2(sim)), 0, "the spawner never spawns again");
}

/// The real-play case (trace `tests/traces/05-gate-swap.trace` 174328→174329): the player stepped on the
/// ledge plate #301 (ch 0x43) while the courtyard sniper had been cycling SNIPER_AIM/SNIPER_FIRE on him
/// since frame 173533 (the 173599 hit) — `NDrone2_SeenAndAttacking` (0x145150) had the combat-music event
/// live (EventList[2] = 5). `SP_Update`'s state-0 start checks [M 0x1dbda8:80-106] refuse the ledge
/// cut-scene for exactly this script: `script == 0x6000994 && Get_Music_Event(2) != 0` → the player goes
/// to state 5 and deletes itself (stay=1), so NO Drone_EnableAll(0) ever runs: the plate's only effects
/// are the spawner's kill (the spawned pair removed on the plate tick) and the placed pair's
/// WAIT_SWITCH→IDLE at their posts (trace: state 1 → 4, same spots), and the AI walks on. Here the
/// combat event comes through the level's own alarm path [M DroneFunc_CheckAlarmRaised 0x1453a8: a
/// death on ch 0xA4 posts event 2 = 3 for the first 58 ticks]: Bond kills ch-0xA4 death-channel guard
/// #65 through the real weapon path. Sister:
/// `the_wall_hug_scene_holds_the_gate_pair_and_swaps_the_pairs_at_its_end` — with the event QUIET (the
/// hand-written ch 0x43 probed over PINE) the same script DOES start and freezes.
///
/// Tick order on the plate tick, why the spawned pair is ALREADY gone when the step returns [M
/// control_movement_object_handler 0x1336f8:54-68 — one newest-first walk runs every control func;
/// Trigger_Calc 0x1b1900's pre-walk pass takes type 2 only, so the type-1 TouchOnce plate #301 fires
/// in `Trigger_Update` at its own walk slot — docs/systems/triggers.md's type table]: the placed pair
/// (placements 302/303) walk first and read ch 0x43 still off; the plate (301) writes ch 0x43; SP #298
/// (298) refuses; the spawner (268) reads its p7 kill channel and — the same `DroneSpawner_Control`
/// call — latches and runs the kill scan [M 0x137e68:50-52 then 235-288]. The trace's frame 174328
/// still shows the pair because the trace's frame sample sits mid-walk between the plate's write and
/// the spawner's Control (frame 174329 then has the removal AND the placed pair's next-tick wake —
/// they walk before the plate, so the channel reaches them one walk later). The earlier draft of this
/// test asserted the pair present at the tick boundary — that state only exists mid-walk; the test was
/// wrong, not the port (the old ignore text's "stray death during the yard wait" was this same-tick
/// removal misread: nothing damages the pair — hp stays at the game's type-1 default 10 [M
/// 0x14c600:81-95, `drone_stats[type][1]` word 2 else 10; 07000005's guards have no record [L]]).
#[test]
fn the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    // The gate pair, through the real spawn path (the trace's 173756/173757: touch 0x23 → the OR → ch 0x22).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x23);
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    scene.sim.as_mut().unwrap().bond.pos = [104.9, 25.7, 108.8];
    let mut spawned = 0;
    for _ in 0..24u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        spawned = { let sim = scene.sim.as_ref().unwrap(); present(sim, &group2(sim)) };
        if spawned == 2 { break; }
    }
    assert_eq!(spawned, 2, "the gate pair spawned on ch 0x22 (the trace's 173758) [D]");
    // A patrolling guard to watch across the plate (the trace's two state-7 walkers).
    let walker = scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .find(|(_, d)| d.present && !d.corpse && d.state == nf_game::drone::state::PATROL)
        .map(|(i, _)| i).expect("a state-7 walker after the spawn (the trace's 2/12 moving) [D]");
    // The combat event, live the way real play had it (the trace: the courtyard sniper cycled
    // SNIPER_AIM/FIRE on Bond from 173533 — `NDrone2_SeenAndAttacking` 0x145150 posting
    // `Music_Event(2, 5)` — while Bond walked the yard): stand where the yard's guards see him
    // until the attack gate posts, through the real AI path only.
    let mut combat_at = None;
    {
        let sim = scene.sim.as_mut().unwrap();
        let v = sim.drones.list[walker].pos;
        sim.bond.pos = [v[0] + 1.2, v[1], v[2]]; // beside the walker, in the yard's sight lines
    }
    for k in 0..300u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.prev_event2 != 0 { combat_at = Some(k); break; }
    }
    assert!(combat_at.is_some(), "the yard's attack gate posted the combat event (EventList[2] = 5) [M 0x145150; D trace 173533+]");
    // The pair rides the wait out untouched — nothing in the yard damages it (the spawned guards'
    // health stays the game's type-1 default 10 [M 0x14c600:81-95]; guards only ever shoot Bond).
    assert_eq!(present(scene.sim.as_ref().unwrap(), &group2(scene.sim.as_ref().unwrap())), 2,
        "the gate pair intact after the yard wait — no stray death [L probe: hp 10 throughout]");
    scene.sim.as_mut().unwrap().bond.pos = [104.9, 25.7, 108.8]; // the walk-to-the-ledge stand-in
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    assert!(scene.sim.as_ref().unwrap().drones.prev_event2 != 0, "the event is still live on the walk to the ledge [D trace: 800 frames of the sniper's cycle]");

    // Onto the plate within the 58-tick alarm window, through the touch volume.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x43);
    }
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    // THE PLATE TICK (the trace's 174328): the channel is on, the script does NOT start (no
    // freeze), and the spawner's kill lands IN THE SAME WALK (see the tick-order note above):
    // by the tick boundary the spawned pair is removed and the spawner is gone.
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x43), "the plate fired ch 0x43 [D trace 174328]");
        assert!(sim.drones.prev_event2 != 0, "the combat event is live at the plate [D trace 174328: the sniper's cycle]");
        assert!(sim.drones.npc_enabled, "no Drone_EnableAll(0) on the plate tick — the refusal, not a freeze [D trace 174328: the walkers move on; M 0x1dbda8]");
        assert_eq!(present(sim, &group2(sim)), 0, "the spawned pair removed on the plate tick — the spawner's Control walks after the plate (301) and kills in the same call [M 0x137e68:50-52 then 235-288]");
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 2).expect("the gate spawner");
        assert!(sp.gone && sp.kill_latch, "the spawner latched (p7) and deleted itself in that Control call [M 0x137e68:50-52, 286]");
        let waiting: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.switch_channel == 0x43 && d.present).map(|(i, _)| i).collect();
        assert_eq!(waiting.len(), 2, "the placed pair (placements 302/303) present but still held [D]");
        for &i in &waiting {
            assert_eq!(sim.drones.list[i].state, nf_game::drone::state::WAIT_SWITCH,
                "the placed pair still WAIT_SWITCH on the plate tick — they walk BEFORE the plate (302/303 > 301, newest-first) and read ch 0x43 one walk later [M 0x1336f8:54-68]");
        }
    }
    // THE NEXT TICK (the trace's 174329): the placed pair activates at their posts (WAIT_SWITCH 1 →
    // IDLE 4) — the visible swap completes; the script player deleted itself after the refusal;
    // the AI-walks-on pin follows below over a window.
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(present(sim, &group2(sim)), 0, "the spawned pair stays gone [D trace 174329]");
        let placed: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.switch_channel == 0x43 && d.present).map(|(i, _)| i).collect();
        assert_eq!(placed.len(), 2, "the placed pair (placements 302/303) present [D]");
        for &i in &placed {
            assert_eq!(sim.drones.list[i].state, nf_game::drone::state::IDLE,
                "the placed pair active: WAIT_SWITCH → IDLE at their posts [D trace 174329: state 1 → 4]");
            assert!(sim.drones.list[i].enabled, "the placed pair enabled with the wake (obj flags 0x8b0 → 0x880) [L hitdata-spawner-ally]");
        }
        assert!(sim.drones.npc_enabled, "the AI never froze [D trace: no state stall through +1,500 frames]");
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 298).expect("SP #298");
        assert!(sp.deleted || sp.state == 5, "SP #298 refused the script and deletes itself [M 0x1dbda8 state 5]");
    }
    // And the scene never plays late: walk out, wait past the alarm window, no freeze ever lands.
    // The AI "walks on" pin is a WINDOW, not one tick: the yard guards are mid-combat with Bond
    // standing on the plate, and an aiming guard legitimately stands still for a tick [D trace:
    // 2/10 moving per frame, not all]. Over 120 ticks the yard's live AI keeps producing motion;
    // a state stall (the old freeze bug) produces none at all.
    let live_pos = |sim: &nf_view::scene::Sim| -> Vec<([f32; 3], usize)> {
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && !d.corpse && d.enabled && d.state != nf_game::drone::state::WAIT_SWITCH)
            .map(|(i, d)| (d.pos, i)).collect()
    };
    let before = live_pos(scene.sim.as_ref().unwrap());
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let after = live_pos(sim);
        let moved = after.iter().any(|(p, i)| {
            before.iter().find(|(_, j)| j == i).is_some_and(|(q, _)| (p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2) > 0.05 * 0.05)
        });
        assert!(moved, "the yard's AI walks on — some live guard moves within 120 ticks of the plate [D trace: 2/10 moving after]");
        assert!(sim.drones.npc_enabled, "no late freeze [D trace: 1,500 frames after the plate, no stall]");
        assert_eq!(present(sim, &group2(sim)), 0, "the swap is final [D]");
    }
    for _ in 0..180 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.npc_enabled, "no late freeze [D trace: 1,500 frames after the plate, no stall]");
    assert_eq!(present(sim, &group2(sim)), 0, "the swap is final [D]");
}

/// The sister: a script started on a tick whose first command is a WAIT starts its clock on the
/// start tick. SP 298's script stream 12 is `Wait(369)` → `Sound(368)` [D 0x06000994 decoded]: the
/// sound lands exactly 369 ticks after the touch (the start tick), not 370 — the old pre-fix player
/// (first commands one tick after `start`) played it at touch+370.
#[test]
fn the_wait_first_stream_starts_its_clock_on_the_start_tick() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x43);
    }
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x43), "the wall-hug plate fires ch 0x43 [D]");
        assert!(!sim.drones.npc_enabled, "the script started and ran its first commands this tick [M 0x1dbc08:56]");
        let at = |sim: &nf_view::scene::Sim| sim.drones.script_sounds.iter().any(|s| s.0 == (0x0600_0994, 12));
        assert!(!at(sim), "stream 12's Wait(369) holds the sound back on the start tick [D 0x06000994 stream 12]");
    }
    // 368 ticks later: still waiting (369 − 1 for the start tick's own update); the 369th tick after
    // the touch plays it.
    for _ in 0..368 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.script_sounds.iter().any(|s| s.0 == (0x0600_0994, 12)), "the wait is not done at touch+368 [D]");
    }
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.script_sounds.iter().any(|s| s.0 == (0x0600_0994, 12)),
        "the sound lands at touch+369 — the clock started on the start tick [D; pre-fix: touch+370]");
}

/// The sister: the same spawn sequence without the kill channel — the guards stay.
#[test]
fn without_the_kill_channel_the_gate_guards_stay() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x23); // the level-start touch feeding the ch-0x22 OR [D]
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    scene.sim.as_mut().unwrap().bond.pos = [104.9, 25.7, 108.8];
    for _ in 0..120u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if present(sim, &group2(sim)) == 2 { break; }
    }
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(present(sim, &group2(sim)), 2, "no ch 0x43: both gate guards remain");
    assert!(sim.drones.spawners.iter().find(|sp| sp.group == 2).map(|sp| !sp.gone).unwrap_or(false),
        "the spawner stays");
}

/// A spawner gated while the NPCs are disabled does nothing until they are re-enabled: with
/// `npc_enabled` forced off (what the game's `Drone_EnableAll(0)` does), the kill channel fires and
/// nothing happens; the tick Control runs again (the script's `Script_Free`), the removal lands.
#[test]
fn the_kill_channel_waits_for_the_npcs_to_be_re_enabled() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = touch_centre(sim, 0x23); // the level-start touch feeding the ch-0x22 OR [D]
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    scene.sim.as_mut().unwrap().bond.pos = [104.9, 25.7, 108.8];
    for _ in 0..120u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if present(sim, &group2(sim)) == 2 { break; }
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(present(sim, &group2(sim)), 2, "both guards spawned");
        sim.drones.npc_enabled = false; // the game's Drone_EnableAll(0) state [M 0x1383e8]
        sim.bond.pos = touch_centre(sim, 0x43);
    }
    for _ in 0..60 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x43), "the kill channel is on");
        assert!(!sim.drones.npc_enabled, "the script froze the NPCs again");
        assert_eq!(present(sim, &group2(sim)), 2, "the spawner's Control returns at its first line: no removal [M 0x137e68:21-23]");
    }
    // The script frees the NPCs (flag 0x80) and the removal lands.
    let mut freed = false;
    for _ in 0..6000u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.npc_enabled { freed = true; break; }
    }
    assert!(freed, "Script_Free re-enabled the NPCs");
    for _ in 0..30u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if present(sim, &group2(sim)) == 0 { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(present(sim, &group2(sim)), 0, "the removal lands once Control runs [L]");
}

/// 07000011, map-tower2's stairwell (the fix-actors case): the group-1 spawner (mode 3,
/// waves 2, trigger ch 0x15 ← TouchOnce #182, stop ch 0x16 ← #183, kill ch 0x1D ← Touch #285 [D])
/// brings its guard in on the trigger, and the kill channel behaves as the keep-level split says
/// [M 0x137e68:235-262]: a guard the game is drawing (`obj+0xFC` != 0 — the port's per-tick
/// `drawn` [M `lbu $v0, 0xfc($s0)` 0x13805c]) STAYS; once he is not drawn any more the next scan
/// removes him and the spawner deletes itself. Sister-in-advance: without ch 0x1D he stays.
#[test]
fn tower2_stairwell_the_kill_channel_keeps_the_drawn_guard() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // the 00-Agent load: the scenario was calibrated on the full roster, which Drone_Create's min-difficulty gate [M 0x136be0:49] makes the 00-Agent roster (the game holds back the param2>=2 placements at 1/2): the stairwell spawner's group-1 templates are param2>=2 placements (Agent/00 Agent
    // only), so the Operative load never creates them and the spawner has nothing to spawn.
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000011.bin", false, None, 3).expect("load 07000011");
    // The level-start cut-scene holds the NPCs; wait it out (the fix-actors `free` shape).
    for _ in 0..400 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); if !scene.sim.as_ref().unwrap().disabled { break; } }
    let tpls: Vec<usize> = scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .filter(|(_, d)| d.group == 1).map(|(i, _)| i).collect();
    let present = |s: &nf_view::scene::Sim| tpls.iter().filter(|&&i| s.drones.list[i].present).count();
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(!tpls.is_empty(), "the group-1 templates exist [D]");
        assert_eq!(present(&sim), 0, "the spawner holds them back [M 0x137aa0]");
        // The trigger: TouchOnce #182 (out 0x15 [D], centre of lo [6.02, 47.09, -4.65] /
        // hi [10.48, 49.16, 0.10]) — the real input path to the spawner's start channel.
        sim.bond.pos = [8.25, 48.13, -2.28];
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x15), "touch #182 fires ch 0x15 [D]");
        assert!(sim.drones.spawners.iter().find(|sp| sp.group == 1).map(|sp| sp.active).unwrap_or(false),
            "the stairwell spawner is active on its trigger [M 0x137e68:53-64]");
    }
    // A spot where the stealth checks pass (the templates far and behind: p8 ≈ 0 [D], the
    // visibility half of spot_ok does the work). Facing +z, away from every template spot.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-30.0, 48.0, 40.0];
        sim.bond.yaw = 0.0;
    }
    let mut spawned = 0;
    for _ in 0..24u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        spawned = present(&scene.sim.as_ref().unwrap());
        if spawned == 2 { break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(spawned, 2, "both guards of the wave arrive (mode 3's permission holds until the rollover [M 0x137e68:176-203])");
    }
    let guard = tpls.iter().copied().find(|&i| scene.sim.as_ref().unwrap().drones.list[i].present).expect("the arrived guard");
    // Sister-in-advance: no kill channel yet, the guard stays. (Short — this level's end-level
    // script chain freezes the NPCs a minute in, and the drawn step below needs him enabled.)
    for _ in 0..12 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert_eq!(present(&scene.sim.as_ref().unwrap()), 2, "no ch 0x1D: the arrived guards stay");
    // Bond right next to him: `visible_to`'s l < 2 arm → drawn (the game's renderer sets
    // obj+0xFC = 2 for every drawn object [M 0x1e62d8:71]).
    {
        let sim = scene.sim.as_mut().unwrap();
        let p = sim.drones.list[guard].pos;
        sim.bond.pos = [p[0] + 1.0, p[1], p[2]];
    }
    scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[guard].drawn, "Bond next to the guard: the game would draw him (obj+0xFC = 2 [M 0x1e62d8:71])");
    }
    // The stairwell kill touch #285 (out 0x1D [D], centre of lo [13.62, 31.29, -2.86] /
    // hi [15.69, 35.75, -0.78]). The latch fires and the kill scan reads LAST tick's drawn
    // (the spawner's Control runs before this tick's drone pass) → the drawn guard is KEPT
    // [M 0x138060 `bnez $v0` → 0x138098: the kept count].
    scene.sim.as_mut().unwrap().bond.pos = [14.65, 33.52, -1.82];
    let mut latched = false;
    for _ in 0..6 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 1).unwrap();
        if sim.drones.channels.on(0x1d) && sp.kill_latch { latched = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 1).unwrap();
        assert!(latched, "the stairwell touch fires ch 0x1D and latches the kill [M 0x137e68:50-52]");
        assert_eq!(present(&sim), 1, "the keep level keeps the drawn guard [M 0x137e68:235-262 — pre-fix-actors this is where he died]");
        assert!(!sp.gone, "the spawner lingers while a drawn guard remains [M 0x1380bc `bnez $s6`]");
    }
    // The level's end-level script chain freezes the NPCs (event 5; the port's model of this
    // level's chain never frees — the Script_Free stand-in, the same direct-state technique the
    // 07000005 tests use). The freeze clears the guard's enabled (obj+0xF0 |= 0x10|0x20
    // [M 0x149dd0:8-15]), so he is no longer drawn — exactly the game's undrawn arm.
    scene.sim.as_mut().unwrap().drones.npc_enabled = true;
    scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI; // face away from the guard
    let mut removed = false;
    for _ in 0..40u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if present(&scene.sim.as_ref().unwrap()) == 0 { removed = true; break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|sp| sp.group == 1).unwrap();
        assert!(removed, "the undrawn guard is removed on the next scan [M 0x13805c: obj+0xFC == 0]");
        assert!(sp.gone, "once none remain the spawner deletes itself [M 0x1380b4-0x1380cc: obj+0xFE |= 1]");
    }
    // And it never spawns again (mode 3 or not — the latched path never reaches the spawn
    // machinery [M 0x137e68:86-233 runs only while +0x20 is clear]).
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert_eq!(present(&scene.sim.as_ref().unwrap()), 0, "the latched spawner never spawns again");
}
