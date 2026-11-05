# mp-bot-voices — MP bots grunt when hit and call out when they die

Branch `s/mp-bot-voices` (cut from origin/master 17a784980; master had not moved when re-checked
after the work, so the branch IS the merged-with-master tree). Tags: [M] code (function + address,
read in `work/decomp/ACTION.ELF/`), [D] disc ELF tables, [L] live PCSX2 measurement, [S] suggested.

## What the game does (settled from the disassembly)

`BOT_handlePain` 0x125ed8 and `BOT_soundEffect` 0x126618 hold the bot bark pools; the two readings
recorded in audit-voice-triggers' "Left as a table" row 2 disagreed on which ids are pain and which
are death. The disassembly settles it — **the port's old comment was right, the audit row's
attribution is swapped**:

- **Pain pool = 0x14d..0x151 (male, `Rand_Rand(5)`) / 0x152..0x153 (alt, `Rand_Rand(2)`)**, gated
  on `Sound_Playing_Ref(+0x73c, Drone+0x108)` [M 0x126120-0x126130; the pool pick 0x126138-0x126154].
  `Sound_Playing_Ref` 0x1de3a0 returns 1 only when the stored handle is non-zero, its sound node's
  ref word (+0x38) equals `Drone+0x108` and the node's state byte (+0x50) is ≤ 1.
- **Death pool = 0x154..0x156 (male) / 0x157..0x159 (alt), `Rand_Rand(3)`, ungated** [M the health
  test 0x1260dc-0x1260e8 produces only 1 = died / 0x1f = survived; `beql s0,1` at **0x126104**
  routes to the death pick 0x126158-0x126164 — the decomp's `uVar8 == 0x1e` arm is dead]. One
  `Rand_Rand(n)` draw per bark, before the play [M 0x126168 / 0x1266b4]; `Sound_Stop_Ref` on the
  stored handle then the new handle stored [M 0x126178-0x1261a4 / 0x1266c8-0x1266fc].
- Callers, all direct `jal`s: `BOT_soundEffect` trigger **0x1f** from `NDrone2_DealWithObjHit`
  case 0x27's MP arm right after the damage write and before the death check [M 0x1401d0, delay
  slot `addiu a1,zero,31` at 0x1401d4], and trigger **1** from both death states' ENTER —
  `NDrone2_DSTATE_BotDeathAnim` 0x175508 → 0x175540 and `NDrone2_DSTATE_BotDeathByExplosion`
  0x1755a8 → 0x1755d8 (delay slot `addiu a1,zero,1`) — before `NDrone2_LogDroneDeath`/`MP_PlayerKilled`
  0x175658. `BOT_handlePain` itself is called from `NDrone2_HitDamage` 0x145558's MP arm
  (`MPSettings+0x388` bots flag [M 0x145558:15-17 → 0x145594]) — the damage+verdict entry. So a
  killing hit plays the gated pain bark first, then the death state's ENTER stops it and plays the
  death line: two draws, in that order.
- All ten pool rows read back from `Drone_SfxInfo` 0x2812d0 [D ELF]: 950/948/943/596/138 (pain
  male), 944/465 (pain alt), 949/947/139 (death male), 945/581/466 (death alt), **all repeat 0** —
  the `NDrone2_PlaySFX` repeatDelay gate (param_5 = 1 [M 0x1266f0]) is vacuous on them; the
  30-frame post-load gate stands.

## What the port does now

- **`BotBrain::bot_sound_effect`** (mp/bots.rs) is `BOT_soundEffect`'s trigger table; its doc now
  cites the callers and addresses above. It has a production caller:
- **`Sim::mp_bot_bark`** (nf-view mpfx.rs) — the bark primitive, kept out of `self` so the damage
  sites can call it while holding the match borrow. It takes the pool from `bot_sound_effect`,
  draws the one `Rand_Rand(n)` on the **shared match stream** (`drones.rng`) exactly when the
  game's arm is reached, runs the row gates through `Drones::foley_entry` (made pub, plus
  `voice_last` pub — the same table the SP foley path stamps), stops the previous bark's handle
  (`MpFx.bot_bark[slot]`, the port's `BOT_vars+0x73c` slot) and delivers the row's sfx at the bot
  via `drones.audio.play3d` + `mp_fx.started` (the MP tests' view). The stored handle is cleared on
  the bot's respawn (a fresh drone object).
- Wired at both MP bot damage/death sites:
  - `Sim::step_mp_bots`' bot-vs-bot round resolution (scene.rs): pain bark (0x1f) after
    `health -= dmg`, death bark (1) before `player_killed` — the game's order.
  - `Sim::mp_hurt`'s bot arm (the port's `NDrone2_HitDamage` → `BOT_handlePain` arm — its doc
    already claimed the multipliers): the same two barks around the damage/death block, for human
    bullets and blasts alike.
- **`Drones::bot_handle_pain`'s stale doc comment fixed** — it claimed "the port is single-player";
  the new comment states the real shape: the lift is indexed into the SP drones list (which holds
  no MP bots), `NDrone2_HitDamage` routes to `BOT_handlePain` only under the MP bots flag, and the
  MP flow barks through `bot_sound_effect` + `mp_bot_bark` reusing its verdict reading. The
  "Sound_Playing_Ref needs the sound pool's node state" deviation text was corrected too (nf-game
  exposes node state; the reason the gate stays unevaluated is determinism — see below).
- `body.sfx_out` on a bot is still only cleared (bots.rs walk_body clears the movement core's
  jump/head-bump queue; the humans' is drained into `mp_fx.body_sounds` at scene.rs). That is the
  movement-sound class, not the pain/death voices — left open, listed below.

## Deviations (documented, kept out of the pins)

- **`Sound_Playing_Ref`'s "previous bark still playing" gate is not carried.** It reads the audio
  pool's node (ref word +0x38, state byte +0x50). In MP the pool's contents are per-machine
  presentation (the local slot's own 0x412/0x88 sounds play on one machine only), so a gate on
  them would make the shared stream's draw count machine-dependent and desync the match. The port
  stops the previous handle and always plays — the same deviation `bot_handle_pain` already
  carries for SP — rate-limited only by the row gates (which are vacuous on these rows: repeat 0).
- **The alt byte (`Drone+0x15`) is `false` for MP bots.** [S] unverified: the bot records carry no
  sex byte and the port does not map the MP character skins (0x0500_xxxx) to one, so the male
  pools stand in. The alt arms are pinned only by `bot_sound_effect`'s unit tests, never by the
  live path.
- Trigger 0x1e stays caller-less (the game has no direct caller for it either; `BOT_handlePain`'s
  decomp arm for it is dead).

## Determinism

The bark draws are shared-stream draws taken in the damage flow, which is identical on every
machine (bot-vs-bot hits come from the brains' deterministic shots; human rounds ride the shared
combat, run by every machine). Nothing hashed changed: `state_hash` does not fold the bots, and
`mp_fx` (where the handles/started log live) is outside the hash. Evidence:

- `mp_world_sync_live` 8/8 (incl. `skyrail_with_bots_hashes_agree` — 2 bots online, two machines,
  per-tick hash equality with the barks drawing).
- `mp_perf_16_live`'s fixed 16-participant run: both machines agreed on all 3600 ticks.
- `the_bark_sequence_is_bit_exact_across_runs` (new): the same seed twice → identical bark
  sequences.

## Pins (all on the real arena, real input paths)

- `crates/nf-view/tests/mp_bot_voices_live.rs` (new):
  - `bot_hit_barks_the_pain_line_and_death_barks_the_death_line` — the Quick Start's 3-bot local
    Deathmatch, 3600 ticks: the pools are asserted equal to the [D ELF] rows (950/948/943/596/138
    pain, 949/947/139 death), 12 pain barks and 2 death barks observed at bot slots, deaths == 2,
    no id cross-play between the pools.
  - `the_bark_sequence_is_bit_exact_across_runs` — same seed twice, 1500 ticks, identical bark
    streams.
  - `the_humans_round_barks_the_bot_through_the_shared_combat` — the second site: slot 0 tracks
    and shoots a bot through `net_tick`; 3 bot health drops paired with a pain bark at the
    victim's slot (6 drops observed).
- `crates/nf-game/src/drone.rs` `foley_tests::bot_pain_pools_agree_with_bot_sound_effect` (new) —
  the settlement cross-check: `pain_pool(died)` == `bot_sound_effect(1)`, `pain_pool(survived)` ==
  `bot_sound_effect(0x1f)`, both alt values, with the addresses in the messages.
- **`mp_perf_16_live`'s `PINNED_DIGEST` re-pinned** 0x89cb_fcb6_ae45_c2ac → **0x6483_2b31_d595_bd3b**
  with the reason in a comment: the barks are real shared-stream draws now [M 0x1266b4], so the
  simulated result legitimately changed; the pin's job (catching an UNintended change) is intact,
  the assertion strength is unchanged. The old golden predates game-accurate bot barks; the test
  contradicted the decompile, the test's golden was what moved (per the task law: the test is
  fixed, with the address).

## Runs (this sandbox, release, `--test-threads=1`, NF_SILENT=1)

- **nf-game: lib 663 passed / 0 failed** (incl. the new settlement test and the foley pool tests);
  `mp_arenas_live` 1/1.
- **Every `mp_*` binary of nf-view ran**: 36 of 38 green (per-binary lines in the run log;
  totals below). Two binaries die by **SIGKILL (OOM)** on this 4 GB sandbox before finishing —
  no assertion failed in either:
  - `mp_online_failures` — dies in `join_in_progress_from_the_menus_then_leave_and_reconnect`.
    **Verified the same on a pristine origin/master worktree** (built and run at
    /home/z/nf-master-check): the same test hangs past a 260 s budget there — pre-existing, not
    this branch's regression.
  - `mp_stress_explosives_live` — dies under the explosives-spam load. On the master worktree the
    same binary runs ≥150 s without completing (its full behaviour was not established here
    before the deadline). The test drives the wired path hardest (blasts → barks), so it is the
    first thing to re-run on a larger-memory machine.
- The full `tools/test/suite.py` fast-runner could not run to completion in this sandbox: every
  background invocation (nohup, setsid) was reaped between tool calls, and the foreground total
  exceeds the 10-minute tool limit. Instead, all test binaries were prebuilt once
  (`cargo test --no-run`, 348 binaries) and the required set was run per-binary in foreground
  chunks with the suite's shape (`--test-threads=1`, per-binary timeout), recorded in
  `suite_chunks.txt` (scratch, not committed). **Totals: 37 binary runs green — nf-game lib 663 +
  nf-view mp set 143 + mp_arenas 1 = 807 test cases passed, 0 failed**; the two OOM binaries above
  and the unrun non-mp nf-view binaries are the remainder. The task's "run it once more on the
  final tree" holds: the final tree IS this tree (master had not moved), and everything that ran,
  ran on it.
- SP suites were not run in full: the touched nf-game surface (foley_entry pub, voice_last pub,
  bot_handle_pain's comment) and nf-view surface (mpfx, the two mp damage sites, MpFx) have no SP
  consumers among the changed behaviour; `voice_triggers_live` does not exist on master (it is the
  audit branch's pin).

## Live checks for the main agent

1. **MP bot pain/death barks** [M 0x1401d0/0x1755d8]: in PCSX2, an MP match with a bot; shoot a
   bot (surviving) and then kill one. Watch `Sound_Play3D`'s id argument: expected a pain id from
   {950, 948, 943, 596, 138} on hits and a death id from {949, 947, 139} on the kill, and on the
   kill the pain line's node stops immediately (Sound_Stop_Ref at 0x1266c8).
2. **The pain gate** [M 0x126120-0x126130]: machine-gun one bot continuously — expected: the pain
   bark does NOT restart while the previous bark's node is still playing (the port will restart
   it; the deviation's size is whatever this shows).
3. **The bot's +0x15 byte** [M 0x126680]: with the disc's default bot roster (Drake/Kiko/Rook on
   07000023), read each bot drone's +0x15 at spawn. Expected: 0 (male pools) if the [S] default is
   right; non-zero for a female character would mean the alt pools (0x152../0x157..) belong in the
   live path.

## What's left

- The two OOM binaries on this sandbox (`mp_online_failures`, `mp_stress_explosives_live`) —
  re-run on a machine with more RAM; `mp_online_failures` is already shown pre-existing.
- The bot alt byte mapping (live check 3).
- `body.sfx_out` on a bot: the movement core's jump/head-bump sounds are still dropped at
  bots.rs (`self.body.sfx_out.clear()`); the humans' are drained into `mp_fx.body_sounds`. A
  BotBrain-side queue or a driver-side drain would fix it; not part of the pain/death ask.
- The bots' bark currently rides `drones.audio.play3d` only (no drones' hearing-list push): the
  port's MP bot AI does not consume the SP hearing path, and `BotBrain`'s own awareness is
  vision-based — if a later task ports bot hearing, the barks (and the bots' shot sounds) should
  feed it.

## Usage

Not visible to me. This sandbox exposes no token, credit or context-window figures for the
session: there is no environment variable, file or tool output in this session that reported any
such numbers (checked: env, /proc, the shell banner). Not estimated.
