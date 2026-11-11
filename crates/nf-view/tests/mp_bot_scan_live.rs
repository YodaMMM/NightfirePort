//! mp-bot-scan-others: the bots' per-tick target scan pinned through a real match.
//!
//! `BOT_setOtherPlayerInfo` 0x125390 fills the `BOT_vars+0xB0` rows at
//! `NDrone2_ControlSTANDARD` 0x1490f0:164 (the bots' drone type 0x1E) right before
//! `NDrone2_FindOpponent` 0x1424e0 reads them; the ported pass is
//! `BotBrain::scan_others`, called from the live driver (`scene.rs` `step_mp_bots`) at that
//! point. Pins, from the decompile:
//!
//! - a bot never picks a team-mate in a team game — the pick gate `0x1424e0:371`
//!   (`(uVar3 & 8) == 0`) on the bit-3 team compare the scan writes (`0x125390:99-102`);
//! - a bot never picks a target whose head-to-head line has been world-blocked the whole
//!   match: the rows' seen bit (bit 2) is written only from `NDrone2_CanSeeObject(drone, obj,
//!   5, 0)` (`0x125390:151-153` — `DroneVision_CanSeeObjectFrom` 0x176f18's head-to-head
//!   `Collide_LineOfSight`, no cone and no range) for the ONE slot the `BOT_vars+0x767`
//!   cursor picks per tick (`0x125390:169-176`), so a never-seen slot can never latch the
//!   bit, and the pick gate `0x1424e0:365-367` (bit1 | bit2) can never choose it — the hold
//!   (`0x1424e0:319-321`) only keeps previously valid opponents;
//! - a bot does pick a target in the open (the head-to-head segment clear at the pick tick).
//!
//! The last two ride the latch window: a target walled longer than one cursor period (8
//! slots) loses the bit and is dropped at the next scan; a HELD opponent may outlive that by
//! the game's unseen-hold (`BOT_getAggressionMul` 0x126468 × 20 ticks, up to 3× inside 2.5u),
//! so the walled-pick bound is latch + hold + slack, not the bare window.

use nf_game::mp::MpSettings;
use nf_game::player::Input;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

const ARENA: u32 = 0x0700_0023;
/// The 8-slot cursor period (`0x125390:169-176`) + the ±1 tick position drift of reading the
/// bodies around the step + slack.
const LATCH_WINDOW: u32 = 10;
/// LATCH_WINDOW + the max unseen-hold (`0x126468`: 1.0 × 20, ×3 inside 2.5u → 60) + slack.
const HOLD_WINDOW: u32 = 72;

/// One FFA quick-match body eye per slot, mirroring the driver's `bot_others`: the human rides
/// `bond.pos + EYE` (scene.rs `BOT_EYE`), the bots their `brain.pos` (already the eye).
fn eye_of(sim: &nf_view::scene::Sim, slot: usize) -> Option<[f32; 3]> {
    let m = sim.mp.as_ref()?;
    if slot == 0 {
        if sim.drones.health.health <= 0.0 { return None; }
        let p = sim.bond.pos;
        return Some([p[0], p[1] + nf_game::player::EYE, p[2]]);
    }
    let b = m.bots.iter().find(|b| b.slot == slot)?;
    if b.dead { return None; }
    Some(b.brain.pos)
}

#[test]
fn scan_pinned_through_a_real_match() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = MpSettings::deathmatch(ARENA);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 2;
    s.seed = 0x0bed_0bad;
    s.time_limit = -1;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    let bots = scene.sim.as_ref().unwrap().mp_bot_count();
    assert_eq!(bots, 2, "two bots against the idle human");

    // last_clear[bot][slot] — the last tick the head-to-head segment was world-clear, measured
    // on the pre-step bodies (the scan reads exactly those inside `step_bond`).
    let mut last_clear = [[None::<u32>; 8]; 8];
    // The last tick a target had no body (dead / not yet respawned): an absent row keeps its
    // latched seen bit — `0x125390` LAB_00125544 clears bits 1 and 3 only (`& 0xfffffff5`) —
    // so a target that respawns may be picked off that stale bit until the cursor revisits it.
    let mut last_absent = [None::<u32>; 8];
    let mut prev_opp = [None::<usize>; 8];
    let mut open_picks = 0u32;
    let mut picks = 0u32;
    for t in 0..2700u32 {
        // The scan-time clearness: the pass's own line, brain eye → row eye.
        for i in 0..bots {
            let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
            let my_slot = m.bots[i].slot;
            let Some(me) = eye_of(scene.sim.as_ref().unwrap(), my_slot) else { continue };
            for s in 0..8usize {
                if s == my_slot { continue; }
                let Some(them) = eye_of(scene.sim.as_ref().unwrap(), s) else {
                    last_absent[s] = Some(t);
                    continue;
                };
                if scene.sim.as_ref().unwrap().world.segment_clear(me, them) {
                    last_clear[i][s] = Some(t);
                }
            }
        }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let m = sim.mp.as_ref().unwrap();
        for (i, b) in m.bots.iter().enumerate() {
            if b.dead { continue; }
            let Some(opp) = b.brain.vars.opponent else { prev_opp[i] = None; continue };
            if opp >= nf_game::mp::bots::OBJ_ROW { continue; } // the site row (no DM site here)
            picks += 1;
            assert_ne!(opp, b.slot as usize, "tick {t}: bot {i} picked itself");
            // A FRESH pick (the opponent changed this tick) rides the pick gate
            // [M 0x1424e0:365-367]: the row's bit 2 must be latched. A HELD opponent may
            // outlive its bit by the unseen-hold — that is the game's rule, not a pick.
            if prev_opp[i] != Some(opp) {
                assert!(
                    b.brain.vars.scan_seen >> opp & 1 == 1,
                    "tick {t}: bot {i} picked slot {opp} the scan never saw (no latched seen bit)"
                );
            }
            prev_opp[i] = Some(opp);
            // The walled-target pin: a pick requires a clear line within latch+hold.
            let lc = last_clear[i][opp];
            assert!(lc.is_some(), "tick {t}: bot {i} picked slot {opp} — a target whose line was NEVER clear");
            // the window restarts when the target comes back (the stale-bit case above)
            let age = t - lc.unwrap().max(last_absent[opp].unwrap_or(0));
            assert!(
                age <= HOLD_WINDOW,
                "tick {t}: bot {i} holds walled slot {opp} for {age} ticks (bound {HOLD_WINDOW})"
            );
            if age <= LATCH_WINDOW {
                open_picks += 1; // a fresh-LOS pick — the target was in the open
            }
        }
    }
    eprintln!("picks {picks}, fresh-LOS picks {open_picks}");
    assert!(picks > 0, "the bots pick opponents over 45 s of the real match");
    assert!(open_picks > 0, "a bot picks a target in the open (fresh LOS at the pick tick)");
}

/// The team-game pin: two bots share team 0, the idle human is team 1 — no bot may ever pick
/// its team-mate (`0x1424e0:371`'s bit-3 gate on the scan's team compare `0x125390:99-102`).
#[test]
fn team_game_never_picks_a_teammate() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = MpSettings::deathmatch(ARENA);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 2;
    s.seed = 0x0bed_0bad;
    s.time_limit = -1;
    s.team_game = 1;
    s.scenario = nf_game::mp::scenario::TEAM_DEATHMATCH;
    s.players[0].team = 1; // the human: the bots' enemy
    s.players[1].team = 0; // the bots: team-mates
    s.players[2].team = 0;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    let bots = scene.sim.as_ref().unwrap().mp_bot_count();
    assert_eq!(bots, 2);
    let mut any_pick = false;
    for t in 0..1800u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
        let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
        for b in &m.bots {
            assert_eq!(b.team, 0, "the bots share team 0 [M 0x183a50:143-145]");
            let teammate = m.bots.iter().find(|o| o.slot != b.slot).map(|o| o.slot);
            let Some(teammate) = teammate else { continue };
            if b.dead { continue; }
            if let Some(opp) = b.brain.vars.opponent {
                if opp >= nf_game::mp::bots::OBJ_ROW { continue; }
                any_pick = true;
                assert_ne!(
                    opp, teammate,
                    "tick {t}: bot in slot {} picked its team-mate {teammate}",
                    b.slot
                );
            }
        }
    }
    eprintln!("team-game picks seen: {any_pick}");
}
