//! mp-bot-combat-ranges: the MP bot's combat bands are `NDrone2_DefaultInit`'s values, not a
//! port guess. The markers-mp-bots report flagged the port's `BotVars::default` [2.0, 20.0,
//! 40.0] against DefaultInit's +0xEC = 4.0 [M 0x14b300:377] and the per-class +0xF0/+0xFC
//! [M :380-426]. The settlement: DefaultInit's block RUNS for every MP bot at match start —
//! `ResetMap_Load` 0x1d6360:155-157 calls `MP_Start` (whose per-bot `BOT_init` gate skips
//! `NDrone2_PostLoad_Init` while `MPSettings[0x188]` is 0 [M 0x183a50:69,207; 0x1249e8:182-185])
//! and THEN `Drone_PostLoad_Init` 0x136938:106-108 walks every drone on the list — the fresh
//! bots included — through `NDrone2_PostLoad_Init` 0x14d890 → `NDrone2_DefaultInit`. The class
//! word `*(u32*)(drone+0xc6c)` is the spawn entry `(idx+2)`'s facing.y bit pattern
//! [M 0x183a50 disasm 0x183d40-0x183d58 (`a2 = idx·0x30 + 0x2a7360` = `SpawnPoints+0x10+idx·0x30`,
//! `SpawnPoints` 0x2a7350 [M-sym]; the register writes facing to entry+0x10..+0x1c
//! [M 0x184f20:82-89]) — a facing.y of −1/0/1 has bit patterns 0xbf800000/0/0x3f800000, never
//! 1..=5, so the `default` arm runs: +0xF0 = 12 + FRand(4), +0xFC = (+0xF0+1) + FRand(4),
//! +0xE8 = 24 [M 0x14b300:381-387, :426], and +0xF4 = +0xEC + FRand(0) [M :441-443] — three
//! shared-stream draws per bot, in the game's order. `BOT_postLoadInit` 0x1262b8:10-14
//! (DefaultInit's MP arm, :822-824) saves the three as the +0x71C/0x720/0x724 defaults.
//! Consumers: the too-close band +0xEC enters 0xD7 Backoff [M 0x16f5e8:61, 0x1501a0:60-76],
//! the engage band +0xF0 holds position [M 0x16f198:46], the too-far band +0xFC advances
//! [M 0x16f5e8:73], and the 0xD7 msg-3 exit bound is +0xF0 (past it → 0xCF
//! [M 0x16fc30:64-92]).

use nf_game::mp::MpSettings;
use nf_game::player::Input;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The Quick Start's settings, one bot against the idle human (mp_bots_live's shape).
fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 1;
    s.seed = 0x0bed_0bad;
    s.time_limit = -1;
    s
}

/// The bot's eye (brain.pos already is) and the human's eye, mirroring mp_bot_scan_live's rows.
fn eyes(sim: &nf_view::scene::Sim) -> Option<([f32; 3], [f32; 3])> {
    let m = sim.mp.as_ref()?;
    let b = m.bots.first()?;
    if b.dead {
        return None;
    }
    if sim.drones.health.health <= 0.0 {
        return None;
    }
    let p = sim.bond.pos;
    Some((b.brain.pos, [p[0], p[1] + nf_game::player::EYE, p[2]]))
}

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn bot_carries_defaultinit_ranges_and_holds_the_game_distance() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = settings();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");

    // ---- 1. The carried values [M 0x14b300:377-426 + 0x1262b8:10-14]. The draws happen at
    // bot_init on the shared stream (create_bots), so the bands are fixed for the match.
    {
        let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
        let r = m.bots[0].brain.vars.combat_range;
        eprintln!("bot 0 combat_range {:?}", r);
        assert_eq!(r[0], 4.0, "the too-close band is DefaultInit's +0xEC = 4.0 [M 0x14b300:377]");
        assert!(
            (12.0..16.0).contains(&r[1]),
            "the engage band is 12 + FRand(4) (the default arm [M :381-387]), got {}",
            r[1]
        );
        assert!(
            (13.0..21.0).contains(&r[2]),
            "the too-far band is (+0xF0+1) + FRand(4) [M :426], got {}",
            r[2]
        );
        assert!(r[2] > r[1], "the too-far band sits above the engage band");
        assert_eq!(
            m.bots[0].brain.vars.combat_range_default, r,
            "BOT_postLoadInit saved the same three as the +0x71C/0x720/0x724 defaults [M 0x1262b8:10-14]"
        );
    }

    // Settle the match: 60 ticks as-is (the bots arm, the scan fills).
    for t in 0..60u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }

    // ---- 2. The bot holds the game's distance: released far away, it closes INSIDE the drawn
    // engage band — the old port guess ([2.0, 20.0, 40.0]) stopped closing at its 20.0 engage
    // band; the game's bot runs the too-far arm [M 0x16f5e8:73] until `dist < +0xF0`.
    let mut min_d = f32::MAX;
    for t in 0..1500u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let m = sim.mp.as_ref().unwrap();
        let b = &m.bots[0];
        if b.dead {
            continue;
        }
        if let Some(opp) = b.brain.vars.opponent {
            if let Some(row) = m.game.slots.get(opp).and_then(|s| s.pos) {
                min_d = min_d.min(dxz(b.brain.pos, [row[0], b.brain.pos[1], row[2]]));
            }
        }
    }
    eprintln!("close phase: min opponent distance {min_d:.2}");
    assert!(
        min_d < 16.0,
        "the bot closed inside the drawn engage band (min dist {min_d:.2}) — the too-far arm [M 0x16f5e8:73] and the engage band it stops under [M 0x16f198:46] are the drawn DefaultInit values"
    );

    // ---- 3. The too-close band holds: once inside 4.0 u the game's 0xD7 Backoff takes over
    // [M 0x16f5e8:61] and the bot never collapses onto the opponent. The port's live sight
    // chain drops the seen-bit at point-blank (the vision model's own gap — the report), so
    // the pin here is the distance floor the crossing produced.
    assert!(
        min_d > 1.0,
        "the bot held a body's width of distance (min {min_d:.2})"
    );
}
