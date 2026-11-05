//! mp-bot-voices: the MP bots grunt when hit and call out when they die.
//!
//! Headless, the Quick Start's 3-bot local Deathmatch on the first arena (mp_bots_live.rs's
//! harness), 3600 fixed ticks of the bots fighting each other. Every registered hit on a bot
//! barks the pain pool and every bot death barks the death pool through `Sim::mp_bot_bark`
//! (nf-view mpfx.rs) — `BOT_soundEffect` 0x126618's production caller: trigger 0x1f after the
//! damage write [M `DealWithObjHit` 0x13fff0 case 0x27 → 0x1401d0, `addiu a1,zero,31`], trigger
//! 1 at the death states' ENTER before the kill is scored [M 0x1755d8 / 0x175540,
//! `addiu a1,zero,1`; `MP_PlayerKilled` 0x188f70 at 0x175658 comes after].
//!
//! The pool ids come from the loaded `Drone_SfxInfo` [D ELF 0x2812d0]: pain (survived ==
//! trigger 0x1f) 0x14d..0x151 → 950/948/943/596/138, death (trigger 1) 0x154..0x156 →
//! 949/947/139, all repeat 0. The audit-voice-triggers table row had pain and death swapped;
//! the disassembly settles it [M 0x126104 → 0x126158-0x126164 died, 0x126120 →
//! 0x126138-0x126154 survived-gated] and `bot_pain_pools_agree_with_bot_sound_effect`
//! (nf-game) pins the cross-check.

use nf_game::mp::MpSettings;
use nf_game::player::Input;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The menus' Quick Start: a local Deathmatch on the first arena with Drake/Kiko/Rook enabled
/// (mp_bots_live.rs pins the same settings).
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
fn bot_hit_barks_the_pain_line_and_death_barks_the_death_line() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = quick_game();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");

    // The bot pools straight from the loaded table — the wiring resolves through these rows.
    let voices = scene.sim.as_ref().unwrap().voices.as_ref().expect("Drone_SfxInfo loads");
    let pain: Vec<u32> = (0x14d..=0x151).map(|i| voices[i].sfx).collect();
    let death: Vec<u32> = (0x154..=0x156).map(|i| voices[i].sfx).collect();
    assert_eq!(pain, vec![950, 948, 943, 596, 138], "the pain rows [D ELF Drone_SfxInfo]");
    assert_eq!(death, vec![949, 947, 139], "the death rows [D ELF Drone_SfxInfo]");

    let humans = 1usize; // the bots' slots follow the humans' [S mp-16-slots]
    let mut seen = 0usize;
    let mut pain_barks: Vec<(u32, u32, usize)> = Vec::new(); // (frame, sfx, slot)
    let mut death_barks: Vec<(u32, u32, usize)> = Vec::new();
    for t in 0..3600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let started = &sim.mp_fx.started[seen.min(sim.mp_fx.started.len())..];
        for &(f, id, _, slot) in started {
            if slot < humans { continue; }
            if pain.contains(&id) { pain_barks.push((f, id, slot)); }
            if death.contains(&id) { death_barks.push((f, id, slot)); }
        }
        seen = sim.mp_fx.started.len();
    }

    let sim = scene.sim.as_ref().unwrap();
    let deaths: u32 = (0..sim.mp_bot_count()).map(|i| sim.mp_bot_deaths(i)).sum();
    eprintln!("== bot voices over 3600 ticks: {} pain barks {:?}.., {} death barks {:?}.., {deaths} deaths ==",
        pain_barks.len(), &pain_barks[..pain_barks.len().min(4)], death_barks.len(), &death_barks[..death_barks.len().min(4)]);

    // A bot takes a hit and the pain line plays.
    assert!(!pain_barks.is_empty(), "at least one hit on a live bot barks the pain pool (950/948/943/596/138)");
    // A bot dies and the death line plays.
    assert!(!death_barks.is_empty(), "at least one bot death barks the death pool (949/947/139)");
    assert!(deaths > 0, "the match's bots die (mp_bots_live's law)");
    // The barks sit at bot slots, never the human's.
    assert!(pain_barks.iter().chain(death_barks.iter()).all(|&(_, _, slot)| slot >= humans),
        "the barks are at the bots' slots");
    // The pools do not bleed into each other: a death tick's bark is a death id, a hit's a pain id
    // (the sets are disjoint [D ELF]; both disjoint from every other voice line the bots could play).
    let played_pain: Vec<u32> = pain_barks.iter().map(|&(_, id, _)| id).collect();
    let played_death: Vec<u32> = death_barks.iter().map(|&(_, id, _)| id).collect();
    assert!(played_pain.iter().all(|id| pain.contains(id)), "pain barks only from the pain rows");
    assert!(played_death.iter().all(|id| death.contains(id)), "death barks only from the death rows");
}

/// The bark draws belong to the shared match stream: the same seed twice produces the same bark
/// sequence (frame, sfx, slot) — the draws are taken in the sim's damage flow, not from any
/// per-machine state. (The online hash-equality pin is `mp_world_sync_live`'s
/// `skyrail_with_bots_hashes_agree`, which runs the same wiring on two machines.)
#[test]
fn the_bark_sequence_is_bit_exact_across_runs() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut runs: Vec<Vec<(u32, u32, usize)>> = Vec::new();
    for _ in 0..2 {
        let s = quick_game();
        let arena = format!("{:08x}.bin", s.level_code);
        let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
        let mut barks: Vec<(u32, u32, usize)> = Vec::new();
        let mut seen = 0usize;
        for t in 0..1500u32 {
            scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
            let sim = scene.sim.as_ref().unwrap();
            for &(f, id, _, slot) in &sim.mp_fx.started[seen.min(sim.mp_fx.started.len())..] {
                if id == 950 || id == 948 || id == 943 || id == 596 || id == 138
                    || id == 949 || id == 947 || id == 139 {
                    barks.push((f, id, slot));
                }
            }
            seen = sim.mp_fx.started.len();
        }
        runs.push(barks);
    }
    eprintln!("== run 0: {} barks, run 1: {} barks ==", runs[0].len(), runs[1].len());
    assert!(!runs[0].is_empty(), "the bots bark within 1500 ticks (25 s of bot fire)");
    assert_eq!(runs[0], runs[1], "the same seed barks the same lines at the same ticks");
}

/// The second site: a HUMAN's round barks the bot through the shared combat (`mp_hurt`'s bot arm
/// — the port's `NDrone2_HitDamage` 0x145558 → `BOT_handlePain` 0x125ed8 arm). The shot
/// description rides `net_tick` (mp_combat_weapons_live's shape): slot 0 tracks the nearest live
/// bot and fires; a tick where that bot's health dropped AND a pain-pool bark started at its slot
/// pins the site.
#[test]
fn the_humans_round_barks_the_bot_through_the_shared_combat() {
    use nf_net::NEUTRAL;
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = quick_game();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    let (w6, wrec) = (6u8, scene.sim.as_ref().unwrap().weapons.records[6].clone());
    let w6_dmg = f32::from_le_bytes(wrec[0x0c..0x10].try_into().unwrap());
    eprintln!("== weapon 6 damage {w6_dmg} ==");
    let mut paired = 0usize;
    let mut tracked = 0usize;
    for t in 0..600u32 {
        let sim = scene.sim.as_ref().unwrap();
        let m = sim.mp.as_ref().unwrap();
        let me_head = sim.bond.head();
        // The nearest live bot with a clear shot line from the eye.
        let target = m.bots.iter().enumerate()
            .filter(|(_, b)| !b.dead)
            .filter(|(_, b)| sim.world.segment(me_head, b.brain.pos).is_none())
            .min_by_key(|(_, b)| {
                let d = b.brain.pos;
                ((d[0] - me_head[0]).powi(2) + (d[1] - me_head[1]).powi(2) + (d[2] - me_head[2]).powi(2)) as i64
            })
            .map(|(_, b)| b.brain.pos);
        let Some(target) = target else { continue };
        let mut row = [NEUTRAL; nf_net::MAX_AGENTS];
        row[0].held = w6;
        row[0].shot = true;
        row[0].shot_weapon = w6;
        let d = [target[0] - me_head[0], target[1] - 0.2 - me_head[1], target[2] - me_head[2]];
        row[0].shot_aim = nf_game::mp::combat::aim_from_dir(d);
        let healths_before: std::collections::HashMap<usize, f32> =
            m.bots.iter().map(|b| (b.slot, b.health)).collect();
        nf_view::net_tick(scene.sim.as_mut().unwrap(), &row, 0);
        let sim = scene.sim.as_ref().unwrap();
        // A pain bark this tick at a bot whose health dropped this tick.
        let dropped: Vec<usize> = sim.mp.as_ref().unwrap().bots.iter()
            .filter(|b| healths_before.get(&b.slot).is_some_and(|h| *h - b.health > 0.5))
            .map(|b| b.slot).collect();
        tracked += dropped.len();
        for &(_, id, _, slot) in &sim.mp_fx.started {
            if slot < 1 { continue; }
            if (id == 950 || id == 948 || id == 943 || id == 596 || id == 138) && dropped.contains(&slot) {
                paired += 1;
            }
        }
        if paired > 0 && t > 60 { break; }
    }
    eprintln!("== shared-combat pin: {tracked} bot health drops, {paired} paired with a pain bark ==");
    assert!(tracked > 0, "the human's rounds reached a bot ({tracked} health drops)");
    assert!(paired > 0, "a human-round hit barked the pain line at the victim bot's slot through mp_hurt");
}
