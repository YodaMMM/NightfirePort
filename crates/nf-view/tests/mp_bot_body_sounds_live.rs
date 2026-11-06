//! mp-bot-body-sounds: the MP bots' jump launch and landing sounds stay SILENT.
//!
//! The decompile says the game's bots never play either sound. `Player_HandleJump` 0x1916b0 is
//! the ONLY code in ACTION.ELF that plays them (the launch 0xA3 [M :184, disasm `addiu a0,
//! zero,0xa3` at 0x1918d8 → `Sound_Play3D` 0x191924]; the landing 0x5FB/0xA3 [M :195-198,
//! 0x1919e4/0x191a14]) — a whole-binary immediate scan of every `Sound_Play3D` 0x1dd548 and
//! `Sound_Play` 0x1dd880 call site finds no other producer, in SLUS_205.79 or DRIVING.ELF
//! either. Its only caller is `Player_SSWalk` (jal 0x192474), whose only caller is
//! `Player_Update`'s substate-0 arm (jal 0x193964) — and `Player_Update` is the movement
//! handler of object classes 3 and 0x12 ONLY (`control_funcs` 0x26f890 +0x24/+0xd8, read as
//! `control_funcs[obj+0xff·0xc]` by `control_movement_object_handler` 0x1336f8:62). The MP
//! bots' objects are class 2 drones: `MP_Start` 0x183a50 creates them through `BOT_init`
//! 0x1249e8 (`param_4 == 0` → `Drone_Create` 0x136be0, a zeroed DIVars) → `NDrone2_CreateObj`
//! 0x14d7c8 writes `obj+0xff = 2`; the only class-3 write outside `Player_Init` 0x1912c4 is
//! `MP_ReSpawn` 0x185474/0x185578, whose class-3/0x12 arm runs only for objects ALREADY class
//! 3/0x12 (a bot's class-2 object takes the reposition arm 0x185368:69-76). Whole-binary `sb
//! …,0xff(…)` scan: 3 → `Player_Init`+`MP_ReSpawn` only, 0x12 → `Player_CheckForDeath` 0x1909ec
//! (the human corpse) only. Class 2 (and the disabled shell's 0x11, `BOT_respawn` 0x124f7c)
//! dispatch `Drone_Control` (control_funcs +0x18/+0xcc) — the NDrone2 anim/root-motion mover,
//! never the player walk. So a bot never reaches `Player_HandleJump`: neither the launch nor
//! the bump plays for it, on any machine.
//!
//! The port's bot body runs the human movement core as its documented stand-in
//! (`BotBrain::body`, bots.rs), so the core queues `(0xA3, pos)` on a launch [M player.rs, the
//! `Player_HandleJump` 0x1916b0:118-147 arm] exactly as it does for the humans' bodies — and
//! `walk_body` clears the queue every tick (bots.rs, the cited `sfx_out.clear()`), which is the
//! game-accurate silence. The humans keep their drains: the local body into `drones.audio`
//! (scene.rs) and the remote bodies into `mp_fx.body_sounds` → `mpfx_play3d` (slot usize::MAX).
//! The jump itself (a bot that jumps drops its launch sound) is pinned by
//! `a_bot_that_jumps_plays_no_launch_sound` (nf-game brain_tests): the real `walk_body` with a
//! wedged walk — this arena's own bots never wedge within the match (their route follower
//! rounds every obstacle), so the live pin holds the match-long silence instead.
//!
//! This pin runs the Quick Start shape crowded to the perf16 roster (14 bots, the same arena
//! the perf16 run uses) for 3600 ticks and asserts: no 0xA3/0x5FB body sound ever reaches the
//! match's sound stream (any slot, any channel) and every bot's `sfx_out` is empty after each
//! step.

use nf_game::mp::MpSettings;
use nf_game::player::Input;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

/// The menus' Quick Start shape, crowded to the perf16 roster (14 bots — the same arena the
/// perf16 run uses): the densest real bot traffic this arena hosts.
fn quick_game() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 14;
    s.seed = 0x0bed_0bad;
    s
}

#[test]
fn a_bot_that_jumps_plays_no_launch_or_landing_sound() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let s = quick_game();
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");

    let mut airborne_bots = 0u32; // observed bot bodies off the ground (falls count; no launches)
    let mut bump_like: Vec<(u32, u32)> = Vec::new(); // (tick, id) — any 0xA3/0x5FB anywhere
    let mut seen = 0usize;
    for t in 0..3600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), t % 37 == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        // The bots' queue is dropped inside `walk_body` every tick: nothing survives a step.
        if let Some(m) = sim.mp.as_ref() {
            for b in &m.bots {
                assert!(b.brain.body.sfx_out.is_empty(),
                    "tick {t}: a bot's body queued {:?} — walk_body must drop the queue every tick (the game's bots never reach Player_HandleJump 0x1916b0)",
                    b.brain.body.sfx_out);
                if !b.brain.body.grounded {
                    airborne_bots += 1;
                }
            }
        }
        // No jump/landing id reaches the match's sound stream through any channel (the
        // remote bodies' `mpfx_play3d` records here; only player.rs's launch/bump arms
        // produce these ids, and the bots' queue is dropped before any drain).
        for &(_, id, _, _) in &sim.mp_fx.started[seen..] {
            if id == 0xa3 || id == 0x5fb {
                bump_like.push((t, id));
            }
        }
        seen = sim.mp_fx.started.len();
    }

    eprintln!("== bot body sounds over 3600 ticks (14 bots): {airborne_bots} airborne bot samples, {} jump/bump sounds ==",
        bump_like.len());
    assert!(bump_like.is_empty(),
        "no 0xA3/0x5FB reached the match's sound stream (the bots never play them [M 0x1916b0 is class-3/0x12-only]): {bump_like:?}");
}
