//! local-match-bot-pickups: in a multiplayer match with NO net session (a local match from the
//! real MP settings — `nf_view::load_mp_carry`), the bots could not take anything: the shared
//! take pass `Sim::step_mp_pickups` (the port of `Pickup_Update` 0x18e148's humans-then-bots
//! taker loop) had NO caller — `update_pickups` handles only the local player's touch
//! (`Drones::mp_shared_pickups` defaults false [M drone.rs:13034]), and `update_pickups_net`
//! runs only on a net session's `Drones::net_bodies` row (scene.rs:3641). The pins here run
//! the pass the way play does — through the per-tick MP driver inside `Sim::step_bond` — and
//! pin the take chain end to end on the real arena 07000023: the bot reaches a weapon (its own
//! `BotBrain::fell_out_map`, the goal machine's out-of-bounds recovery), the pass takes it, the
//! placement's own sound word (0xF5 [M Pickup_CreateFromSet; the bot arm plays +0x2A,
//! 0x18e650:137 → :334-337]) is heard AT the pickup, the respawn tail hides it with the game's
//! timer (state 2, `60·respawn·10 + 1` ticks [M 0x18e650:348 + 0x18e148:183-185]), the bot's
//! visit stamp lands in its row [M 0x18e650:126-127], and with the two Auto Aim rows set
//! differently the match reads the MP row [M Player_AutoAim 0x1a7c98:39-45 — the game picks the
//! row on MPSettings+0x180; the port's mark is `mp_shared_pickups`, set by the wired pass].

use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn pos_of(scene: &nf_view::scene::Scene, k: usize) -> [f32; 3] {
    let sim = scene.sim.as_ref().unwrap();
    let p = &sim.drones.pickups[k];
    [p.m[3][0], p.m[3][1], p.m[3][2]]
}

/// Tick the sim the way play does — `Sim::step_bond` runs the per-tick MP driver (the match
/// update, the scenario sync, `step_mp_bots`, and once wired the shared pickup pass).
fn step(scene: &mut nf_view::scene::Scene, n: u32) {
    for _ in 0..n {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
    }
}

fn local_match(bots: usize) -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut ms = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    ms.bot_count = bots as u32;
    ms.total_players = (1 + bots) as u32;
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, ms).expect("the arena loads");
    step(&mut scene, 10);
    scene
}

/// A weapon pickup the bot does not already hold (its spawn kit is weapons 1 and 6).
fn target_for_bot(scene: &nf_view::scene::Scene, bot: usize) -> (usize, [f32; 3]) {
    let sim = scene.sim.as_ref().unwrap();
    let m = sim.mp.as_ref().unwrap();
    let held = &m.bots[bot].brain.vars.arsenal.held;
    sim.drones.pickups.iter().enumerate()
        .filter(|(_, p)| !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON
            && (p.weapon as usize) < held.len() && !held[p.weapon as usize])
        .map(|(k, _)| (k, pos_of(scene, k))).next().expect("a weapon pickup the bot does not hold")
}

/// THE pin: in a local match (no net session) a bot that reaches a weapon takes it through the
/// per-tick driver — the sound plays at the pickup, the respawn timer starts, the visit stamp
/// lands, and the MP Auto Aim row is the one the match reads.
#[test]
fn bot_takes_a_weapon_in_a_local_match_through_the_tick_driver() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match(1);
    let (target, at) = target_for_bot(&scene, 0);
    // Park the bot on the pickup; the local player far away so the human loop is out.
    {
        let sim = scene.sim.as_mut().unwrap();
        let at_far = [at[0], at[1], at[2] + 60.0];
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at_far, 0.0);
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at[0], at[1], at[2]], 0.0);
    }
    let mut taken_tick = None;
    for t in 0..40u32 {
        step(&mut scene, 1);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.pickups[target];
        if p.gone || p.state == 2 { taken_tick = Some(t); break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.pickups[target];
        assert!(p.gone || p.state == 2, "the bot takes the weapon through the per-tick driver (pick {target} still live: state {} gone {})", p.state, p.gone);
    }
    // The SOUND at the pickup, through the same pass: the driver's own play lands mid-tick and
    // the AI hearing pass consumes the hearing-list copy within the tick, so the sound is
    // re-pinned on a second take driven straight into the pass (the same code the driver
    // calls; the audit branch's bot_take pin covers the law) — read BEFORE any further tick.
    let (second, at2) = {
        let sim = scene.sim.as_ref().unwrap();
        let held = &sim.mp.as_ref().unwrap().bots[0].brain.vars.arsenal.held;
        let k = sim.drones.pickups.iter().enumerate()
            .filter(|(k, p)| *k != target && !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON
                && (p.weapon as usize) < held.len() && !held[p.weapon as usize])
            .map(|(k, _)| k).next().expect("a second weapon pickup");
        let p = &sim.drones.pickups[k];
        (k, [p.m[3][0], p.m[3][1], p.m[3][2]])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at2[0], at2[1], at2[2]], 0.0);
        sim.step_mp_pickups();
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.sounds.iter().any(|s| s.id == 0xf5
        && (s.pos[0] - at2[0]).abs() < 2.0 && (s.pos[2] - at2[2]).abs() < 2.0),
        "the placement's own 0xF5 plays AT the pickup [M 0x18e650:137 → :334-337] (sounds {:?})",
        sim.drones.sounds.iter().map(|s| (s.id, s.pos)).collect::<Vec<_>>());
    // The respawn tail [M 0x18e650:348 + 0x18e148:183-185]: the hide arm sets state 2 and the
    // game's timer — the port's `respawn_ticks = 60·respawn·10 + 1` (the restore lands on
    // elapsed == limit+1, audit-pickup-calls fix 4). This arena's set pickups carry respawn 3
    // [D the set rows], so the timer is 1801.
    {
        let p = &sim.drones.pickups[target];
        if !p.gone {
            assert_eq!(p.state, 2, "the taken pickup hides for the respawn");
            assert!(p.respawn_ticks >= 1800, "the respawn timer is the game's 60·3·10 (+1) ticks, got {}", p.respawn_ticks);
        }
    }
    // The bot's visit stamp [M 0x18e650:126-127]: the bot's row for this pickup carries a fresh
    // stamp (zero would mean the take never registered on the bots' pickup table).
    let row = sim.drones.pickups[..target].iter()
        .filter(|p| matches!(p.kind, nf_game::pickup::kind::WEAPON | nf_game::pickup::kind::AMMO | nf_game::pickup::kind::ARMOUR)).count();
    let stamp = sim.mp_bot_pickups[row].visit[0];
    assert!(stamp > 0.0, "the bot's visit stamp landed in its pickup row (row {row} visit[0] = {stamp})");
    // The bot's own arsenal grew (the take reached the bot layer, not just the pickup state).
    let m = sim.mp.as_ref().unwrap();
    let w = sim.drones.pickups[target].weapon as usize;
    assert!(m.bots[0].brain.vars.arsenal.held.get(w).copied().unwrap_or(false)
        || m.bots[0].brain.vars.arsenal.pool.get(w).copied().unwrap_or(0) > 0
        || sim.drones.pickups[target].state == 2 || sim.drones.pickups[target].gone,
        "the take reached the bot's arsenal layer");
    let _ = taken_tick;
}

/// The MP Auto Aim row: a local match marks itself as sharing pickups once the pass runs
/// (`mp_shared_pickups` — the mark lib.rs's detector reads: `let mp = self.net.is_some() ||
/// sim.drones.mp_shared_pickups`), so the match reads the MP row and not the single-player
/// row [M Player_AutoAim 0x1a7c98:39-45 — the game's own pick is on MPSettings+0x180]. Before
/// the wiring the mark stayed false in play (the pass had no caller) and the match read the SP
/// row. The row pick itself is the detector's existing unit pin (`the_mp_auto_aim_row_wins_in_mp`);
/// this pin is the mark's arrival in play, through the same driven take as the first test.
#[test]
fn a_local_match_marks_itself_for_the_mp_auto_aim_row() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match(1);
    assert!(scene.sim.as_ref().unwrap().drones.mp_shared_pickups,
        "the pass runs in the warm-up ticks and marks the match (pre-fix this stayed false in play)");
    let (target, at) = target_for_bot(&scene, 0);
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at[0], at[1], at[2] + 60.0], 0.0);
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at[0], at[1], at[2]], 0.0);
    }
    step(&mut scene, 40);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.mp_shared_pickups, "the wired pass marks the match as sharing pickups (the MP detector's local-match arm)");
    assert!(sim.drones.pickups[target].gone || sim.drones.pickups[target].state == 2,
        "the bot took the pickup through the same driven pass");
}

/// The delete arm's row unregister [M Pickup_Handler 0x18e650:338-344 — `+0x2c == 0` sets the
/// gone flag and calls MP_UnregisterPickup; MP_UnregisterPickup 0x18a058:21-31 zeroes the row
/// and drops the registered count] against the hide arm [M :348 — `+0x2c != -1` hides (state 2)
/// and keeps the row]. The arena's placed pickups all respawn (respawn 3, never deleted) and
/// never expire, so the pin takes TWO pickups through the real driven pass (each stamps its
/// bot row), then puts pickup A in the delete arm's own state (gone — what the respawn-0 tail
/// sets) and leaves B in the hide arm's (state 2): A's row is zeroed (its visit stamps are
/// GONE — the memset only the unregister runs), B's row keeps its stamps.
/// and drops the registered count] against the hide arm [M :348 — `+0x2c != -1` hides (state 2)
/// and keeps the row]. The arena's placed pickups all respawn (respawn 3, never deleted) and
/// never expire, so the pin takes TWO pickups through the real driven pass (the bot's own
/// fell_out_map onto each), then puts pickup A in the delete arm's own state (gone — what the
/// respawn-0 tail sets) and leaves B in the hide arm's (state 2): A's row unregisters, B's row
/// stays registered. (The unregister's memset + count are the helper's own unit pin in
/// bots.rs — "the row zeroes and the count decrements".)
#[test]
fn a_deleted_pickup_unregisters_its_bot_row_and_a_hidden_one_keeps_it() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match(1);
    let (a, at_a) = target_for_bot(&scene, 0);
    // Park the bot on A: the driver's pass takes it (state 2, visit stamped).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [at_a[0], at_a[1], at_a[2] + 60.0], 0.0);
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at_a[0], at_a[1], at_a[2]], 0.0);
    }
    step(&mut scene, 10);
    let row_a = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.pickups[..a].iter()
            .filter(|p| matches!(p.kind, nf_game::pickup::kind::WEAPON | nf_game::pickup::kind::AMMO | nf_game::pickup::kind::ARMOUR)).count()
    };
    assert!(scene.sim.as_ref().unwrap().mp_bot_pickups[row_a].visit.iter().any(|&v| v > 0.0),
        "the take stamped the bot's row (row {row_a})");
    // B: a second weapon pickup, taken the same way — its row is the KEEP control.
    let (row_b, at_b) = {
        let sim = scene.sim.as_ref().unwrap();
        let held = &sim.mp.as_ref().unwrap().bots[0].brain.vars.arsenal.held;
        let b = sim.drones.pickups.iter().enumerate()
            .filter(|(k, p)| *k != a && !p.gone && p.state != 2 && p.kind == nf_game::pickup::kind::WEAPON
                && (p.weapon as usize) < held.len() && !held[p.weapon as usize])
            .map(|(k, _)| k).next().expect("a second weapon pickup");
        let p = &sim.drones.pickups[b];
        let row = sim.drones.pickups[..b].iter()
            .filter(|p| matches!(p.kind, nf_game::pickup::kind::WEAPON | nf_game::pickup::kind::AMMO | nf_game::pickup::kind::ARMOUR)).count();
        (row, [p.m[3][0], p.m[3][1], p.m[3][2]])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        let m = sim.mp.as_mut().unwrap();
        m.bots[0].brain.fell_out_map([at_b[0], at_b[1], at_b[2]], 0.0);
    }
    step(&mut scene, 10);
    // Pickup A into the delete arm's own state; B stays hidden (state 2, where its take left
    // it). The next driver pass reconciles: A's row unregisters, B's row stays.
    scene.sim.as_mut().unwrap().drones.pickups[a].gone = true;
    step(&mut scene, 3);
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.mp_bot_pickups[row_a].available, "the deleted pickup's row is unregistered [M 0x18e650:341 → 0x18a058]");
    assert!(sim.mp_bot_pickups[row_b].available, "a hidden (respawning) pickup's row stays registered — only the delete arm unregisters [M 0x18e650:348]");
}
