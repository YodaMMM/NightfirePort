//! mp-bots-in-match (704): the Quick Game's 3 bots must actually PLAY.
//! Headless, from the menus' `mp_match_settings` through `load_mp_carry` (the same load the
//! Quick Start serves), 3600 fixed ticks of the arena. Per bot: route movement (positions
//! change), acquisition + fire at the human and other bots, damage + death, respawn per the
//! game's rules, scores going up. Sister: `bot_count = 0` leaves only the human in the match.
//!
//! Evidence: [M] work/decomp/ACTION.ELF (function address = filename prefix), [L] EE snapshot,
//! [S] suggested. The bot brain is the ported `nf_game::mp::bots::BotBrain` (the BOT_update
//! 0x126448 family); the match core is `nf_game::mp` (MP_Start 0x183a50, MP_Update 0x184bd0).

use nf_game::mp::MpSettings;
use nf_game::player::Input;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The menus' Quick Start: a local Deathmatch on the first arena with Drake/Kiko/Rook enabled
/// (`mp_menu_flow.rs`:233 pins the same settings — bot_count 3 rides `mp_match_settings`).
fn quick_game() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 3;
    s.seed = 0x0bed_0bad;
    s
}

#[test]
fn quick_game_bots_play_the_match() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = quick_game();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s.clone()).expect("arena loads");

    // The bots exist: `MP_Start` 0x183a50:134-160 creates one bot per enabled record in slots
    // 4..4+count, after every human's create.
    let bots = scene.sim.as_ref().unwrap().mp_bot_count();
    assert_eq!(bots, 3, "the Quick Start's 3 bots exist in the match [M 0x183a50:138-140]");

    // 3600 fixed ticks = 60 s of the match. The human stands (neutral input) — the bots must do
    // everything else themselves.
    let mut moved = vec![0.0f32; bots];
    let mut last = scene.sim.as_ref().unwrap().mp_bot_positions();
    for t in 0..3600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let cur = sim.mp_bot_positions();
        for i in 0..bots {
            moved[i] += ((cur[i][0] - last[i][0]).powi(2) + (cur[i][2] - last[i][2]).powi(2)).sqrt();
        }
        last = cur;
    }

    let sim = scene.sim.as_ref().unwrap();
    eprintln!("== per-bot stats after 3600 ticks ==");
    for i in 0..bots {
        eprintln!("bot {i}: walked {:.1}, shots {}, deaths {}, respawns {}, score {}",
            moved[i], sim.mp_bot_shots(i), sim.mp_bot_deaths(i),
            sim.mp_bot_respawns(i), sim.mp_bot_score(i));
    }

    for i in 0..bots {
        assert!(moved[i] > 5.0, "bot {i} moves its route (walked {:.1} units over 3600 ticks)", moved[i]);
    }
    // The bots fire as a match [M the brain's fire decision 0x127c78's combat block]: no
    // decompile law makes EVERY bot pull within 60 s — which bot fires is emergent over the
    // shared stream. mp-respawn-fists-draw: the stream shifted because the spawn tick no longer
    // plays the held weapon's IDLE script (its frame-1 op-1 sound, 0x060004b9 → sound 0x1, every
    // 37 ticks) — the game plays the DRAW first (the equip's anim trio [M 0x1855f8:50-53] →
    // Player_WeaponSelect 0x1a2cc0 → view state 3 [M 0x1a2e38 case 3]), so the idle op-1 cannot
    // fire at the spawn tick; the bots' hearing rolls moved with it.
    let shots: u32 = (0..bots).map(|i| sim.mp_bot_shots(i)).sum();
    assert!(shots > 0, "the bots fire ({shots} shots)");
    // The match plays: the bots fight each other (damage + death + respawn per BOT_respawn
    // 0x124e38) and the score table moves [M MPGame+slot*0x30+0x18].
    let deaths: u32 = (0..bots).map(|i| sim.mp_bot_deaths(i)).sum();
    let score: i32 = (0..bots).map(|i| sim.mp_bot_score(i)).sum();
    assert!(deaths > 0 || score > 0, "the bots score frags or die — deaths {deaths}, score {score}");
    assert!(deaths > 0, "at least one bot takes damage and dies (deaths {deaths})");
}

/// Sister: with the bots disabled, only the human is in the match — no bot entities, no bot
/// slots (`MP_Start` creates nothing for bot_count 0 [M 0x183a50:134-160's count gate]).
#[test]
fn bots_disabled_leaves_only_the_human() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = quick_game();
    s.bot_count = 0;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    assert_eq!(scene.sim.as_ref().unwrap().mp_bot_count(), 0, "no bot entities when the bots are disabled");
    for t in 0..600u32 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.mp_bot_count(), 0, "still no bots after 600 ticks");
    assert!(sim.drones.health.health > 0.0, "the human stands alone and unhurt");
}

/// The bots roam and hunt (they used to stand still): 2 bots + the idle human on the first
/// arena, 3600 ticks. Per bot: the path length, the farthest point from its spawn, the distinct
/// 4-unit grid cells it stood in, and the closest it came to the human. No tick's step may pass
/// through a wall (the eye-height segment between consecutive positions stays world-clear).
#[test]
fn bots_roam_and_hunt() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = quick_game();
    s.bot_count = 2;
    // No time limit: the default's limit runs out inside the 60 s and `MP_ReSpawn`'s gate then
    // holds a killed bot down [M 0x185368:16-20] — with the armed bots (mp-lobby-settings) bot 1
    // died just before the time-up and stood dead for the rest of the run.
    s.time_limit = -1;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    let bots = scene.sim.as_ref().unwrap().mp_bot_count();
    assert_eq!(bots, 2);
    let nav_paths = scene.sim.as_ref().unwrap().nav.paths.len();
    eprintln!("nav paths {nav_paths}: {:?}", scene.sim.as_ref().unwrap().nav.paths.iter().map(|p| (p.name.clone(), p.kind, p.nodes.len())).collect::<Vec<_>>());
    let start = scene.sim.as_ref().unwrap().mp_bot_positions();
    let mut last = start.clone();
    let mut walked = vec![0.0f32; bots];
    let mut far = vec![0.0f32; bots];
    let mut near_human = vec![f32::MAX; bots];
    let mut cells: Vec<std::collections::BTreeSet<(i32, i32, i32)>> = vec![Default::default(); bots];
    let mut through_wall = 0u32;
    let mut big_drop = 0u32;
    for t in 0..3600u32 {
        let respawns: Vec<u32> = (0..bots).map(|i| scene.sim.as_ref().unwrap().mp_bot_respawns(i)).collect();
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let cur = sim.mp_bot_positions();
        let human = sim.bond.pos;
        for i in 0..bots {
            if sim.mp_bot_respawns(i) != respawns[i] { last[i] = cur[i]; continue; }
            let step = ((cur[i][0] - last[i][0]).powi(2) + (cur[i][2] - last[i][2]).powi(2)).sqrt();
            walked[i] += step;
            if step > 0.0 && sim.world.segment(last[i], cur[i]).is_some() { through_wall += 1; }
            if (cur[i][1] - last[i][1]).abs() > 1.0 { big_drop += 1; }
            far[i] = far[i].max(((cur[i][0] - start[i][0]).powi(2) + (cur[i][2] - start[i][2]).powi(2)).sqrt());
            cells[i].insert(((cur[i][0] / 4.0).floor() as i32, (cur[i][1] / 2.0).floor() as i32, (cur[i][2] / 4.0).floor() as i32));
            let dh = ((cur[i][0] - human[0]).powi(2) + (cur[i][2] - human[2]).powi(2)).sqrt();
            near_human[i] = near_human[i].min(dh);
        }
        if t % 600 == 599 {
            eprintln!("t{} human {:.1?} bots {:.1?}", t + 1, human, cur);
        }
        last = cur;
    }
    let sim = scene.sim.as_ref().unwrap();
    for i in 0..bots {
        eprintln!("bot {i}: walked {:.1}, farthest {:.1}, cells {}, closest to human {:.1}, shots {}, deaths {}, score {}",
            walked[i], far[i], cells[i].len(), near_human[i], sim.mp_bot_shots(i), sim.mp_bot_deaths(i), sim.mp_bot_score(i));
    }
    eprintln!("steps through a wall {through_wall}, height jumps > 1 unit {big_drop}");
    // Before the body walk (one straight-line stand-in step, then standing fire): 5.2 and 9.9
    // units walked, 2 and 4 cells. The bots now roam the arena and come to the human.
    for i in 0..bots {
        assert!(walked[i] > 100.0, "bot {i} roams the arena (walked {:.1} units in 60 s)", walked[i]);
        assert!(far[i] > 15.0, "bot {i} leaves its spawn area (farthest {:.1})", far[i]);
        assert!(cells[i].len() >= 20, "bot {i} covers ground ({} cells)", cells[i].len());
        assert!(sim.mp_bot_shots(i) > 0, "bot {i} fires");
    }
    assert!(near_human.iter().any(|&d| d < 8.0), "a bot comes to the idle human (closest {near_human:?})");
    assert_eq!(through_wall, 0, "no bot step crosses world geometry");
    assert_eq!(big_drop, 0, "no bot height snaps by more than a unit in one tick");
}
