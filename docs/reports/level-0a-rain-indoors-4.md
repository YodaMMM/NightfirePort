# level-0a-rain-indoors-4 — the bathhouse shout after the rain fix

Branch `s/level-0a-rain-indoors-4` = origin/s/level-0a-rain-indoors-3 merged with origin/master
(clean; master's tip c528468b includes the level-13 merge). The task's one red —
`bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too` (07000002) —
is **green** on this branch; the recovered reds from rounds 2-3 all re-run green; the full suite
started in the background on the merged tree (see the Suite appendix for the cutoff state).

One note on the start point: master does NOT carry the rain-2 fix (`git branch -r --contains
origin/s/level-0a-rain-indoors-2` lists only the rain-indoors branches), so this merged tree runs
the game-true rain while master's tree does not — that is why the red is back here after
master's later merges had "un-red" it on the level-13 tree (docs/reports/level-13-security-room.md,
commit bfb48b04: those pins held on a tree WITHOUT the rain fix).

## Which draw decides the shout, and what the old window actually pinned

The shout chain [M]: a vt-1 guard speaks the attack shout only at an Attack enter —
`NDrone2_AttackTalk` 0x13eac0 (work/decomp/ACTION.ELF/0013eac0) picks
`Rand_Rand(4) + 0x33` (the voice line, SfxInfo entries 0x33..0x36 → sfx ids 115/112/111/113,
repeat 120 [D Drone_SfxInfo; docs/systems/under-bathhouse-voices.md]) and then speaks it on the
second `Rand_Rand(2)` odds draw [M]; both draws happen per call when the guard is not voice-busy
or dead (the decompile's early returns precede the switch). The shout sites are the FirstAttack /
AimStandFire (0x68) / CombatNoMove (0x59) ENTERs [M]. The level's first-sight line is the silent
entry 0x21 for vt 1 (sfxId 0 [D]) — the shout can only come from a later Attack enter. The mixer
half is the same chain the bunker test pins: the stream keys (STREAM pitch 0x759 = 1881 [M
`StartSample` 0x3524]) and reaches the mixer at non-zero volume — the game's `SFXSetup` 0x4220
cannot reject it (priority 99 · 100 dwarfs the threshold [M 0x4220]).

The old choreography pinned ONE coin. Probed with
`crates/nf-view/examples/probe_02_shout_window.rs` (a dump, not a test — the rain-3 probe
pattern), which replays the choreography and dumps per tick: the EE sound pool's live ids
(`drones.audio.live_ids()`), the driver's live voice ids, every stream Play, each guard's
`voice`/state-log transitions, and the channel rise ticks:

- The floor plate #341 (out ch 0x15 [D]) rises on Bond's first tick on it (cycle 0 tick 1), and
  the ch-0x15 five (guards #0-4, `switch_channel` 0x15 — the probe's dump) enable — and PATROL:
  their logs go 0x1 → 0x6 (INIT_PATROL) → 0x4/0x5/0x7 with **no Attack enter and therefore no
  AttackTalk roll** in the whole window.
- The choreography's "away" spot (−47.0, 6.0, −23.3) IS the bunker plate #389 [D — the same
  position the mixer test's own bunker test and the voices doc's live check use]. When Bond
  steps there, ch 0x81 rises and the BUNKER TRIO (guards #13-15, `switch_channel` 0x81) enables
  with start state ATTACK — each rolls AttackTalk's rand(4)/rand(2) **once** at the enable-time
  FirstAttack (TouchOnce = one wake per boot).
- So the whole 4×300 window rode the trio's single enable-time coin, and the game-true rain's
  extra shared-generator draws moved that coin's stream position [M Rand_* 0x1e36f0..0x1e3780]:
  in this tree's stream all three guards land silent (the port is bit-exact — the coin is
  deterministic, and on this stream it lands tails ×3).
- The voices twin (`bathhouse_voices_live`, same choreography) passed on the same tree only
  because its extra 30-tick phase shifts the trio's enable 30 frames → a different coin (its
  hit: id 112, cycle 1 tick 2, from the probe's verbatim replica) — and because it polls the EE
  pool (a node lives ~5 ticks, START_GRACE [M sound.rs]) instead of requiring the mixer's
  one-tick stream Play batch.

No assert was wrong here — the asserts ([M]-backed: a shout voice keys, its stream Play reaches
the mixer at non-zero volume) were fine. The WINDOW was the unbacked part: 4×300 has no
decompile basis; it held on the pre-rain stream by luck.

## The fix — the player's walk, no assert touched

The decompile-justified window is the player's own walk over the level's wake plates: Bond steps
onto EVERY oDroneTrigger wake plate in turn (out channels 0x15/0x81/0x17/0x18/0x19/0x08 [D
placements/docs/levels/07000002.md — the 0x15 five, the 0x81 trio, and the yard groups, 20+
guards]), standing up to 1200 ticks on each (the bunker test's proven window). Each wake's
guards roll AttackTalk at their Attack enters [M 0x13eac0] — a stream shift moves WHICH guard
shouts, never whether the chain carries a shout to the mixer; a port break (the SFXSetup reject,
a speech-gate drift, a mixer loss) silences every group and fails the test. A plate behind an
unfired lock chain simply doesn't wake its group (the AND/lock gates [D docs]) and that group's
rolls are skipped; the rest carry the pin. The asserts are byte-for-byte the old ones (ids
111..115 live + a pitch-1881 Play in the same batch; `l > 0.0 && r > 0.0`), and the test name is
unchanged.

Result: **the test is green (2/2 in the binary, 3.66 s)** — the shout keys inside the floor
group's phase (the patrol guards' notice → Attack → AttackTalk chain, ~300-400 ticks in; with
the old 300-tick windows the walk moved on before it and the later plates' streams landed
silent — the window length itself does not change the stream up to tick 300, which is exactly
why the short window was the red).

## Verification

- `cargo test --offline --release -p nf-view --test bathhouse_shout_mixer_live --
  --test-threads=1` → **ok. 2 passed** (3.66 s) on the merged tree with the rain at game speed.
- The recovered reds from rounds 2-3, run directly on this tree (final binaries,
  `--test-threads=1`): sight_frustum_live **2/2**, doublecross_transitions_live **6/6**,
  escort_hide_findrun_live **3/3**, fix_pickups_ammo_live **4/4**,
  hostage_killer_07000002_live **5/5**, fix_0a_rain_indoors_live **2/2**,
  interactables_0700000a_live **7/7** (the armour walk inside), escort_follow_07000001_live
  `escort_at_walking` **1/1** — 30 tests, all green.
- Probe example `probe_02_shout_window.rs` committed (the rain-3 dump pattern; it builds clean
  and is not a test).

## What's left

- **The full suite's total line.** The suite daemon started 14:56 UTC on the merged tree —
  BEFORE the test fix (the fix touches one test file; the suite's `bathhouse_shout_mixer_live`
  result is the pre-fix binary's and is stale — the direct run above is the proof). At the
  15:35 checkpoint it had run 417 binaries. Projected finish ~16:46 UTC is past the sandbox
  expiry (~16:32), so the total line will not land here; the next session should read
  `/tmp/suite4_daemon.log` (the daemon's stdout — note: it is NOT in `suite4.out`, the launch
  wrapper routed it) and `/home/z/suite.log` first, then re-run the suite on the final tree.
- **`interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire` is STILL red on
  this tree** (direct run: 1 failed, 1.98 s; also red in the suite run). This is the round-3
  report's second stream-shift red — the same class (a pinned fire window riding the shared
  generator's position), explicitly outside this task's one-red scope. It passes on trees
  WITHOUT the rain fix (level-13's run) and returns wherever the game-true rain is present.
  Next-session work, with the same method: find the draw the window rides [M], re-pin to the
  rule or drive the real trigger.
- A deeper question this task surfaced (not needed for the green, worth a look next session):
  after a blind enable (no sight at the wake), the trio dropped to NO_OPPONENT (0x60) at the
  enable tick + 1 and never re-acquired Bond standing 0.5-6 units away for 900+ ticks (the
  probe's phase-1 logs) — while the bunker test's own boot DOES engage. The 0x60 exit reads the
  +0x170 pointer alone [M 0x1650a0:27-47] and FindOpponent re-points it at Bond per tick [M
  0x1424e0:100-121]; whether the re-point is sight-gated in a way that can starve here is
  unverified. Everything observable in this task was explained without touching it; the
  multi-plate walk does not depend on it (the floor group's engagement goes through the patrol
  notice path, and the trio test's own boot engages).
- The environment's OOM kills (exit −9: net_*/mp_*/frame_cost/big walks) are sandbox limits,
  not test failures — the same signature every session shows.

## Live checks for the main agent

Nothing visual is claimed — this task is a test-choreography fix; the port's audio chain
(EE pool → driver → mixer) is already covered by the bunker test's pins. If wanted:

1. PCSX2, 07000002, fresh boot: walk onto the bathhouse floor plate #341 (−25.0, 4.4, 42.6) and
   stand ~7 s — EXPECT the ch-0x15 floor guards (five around the bathhouse floor) to patrol;
   when one notices Bond and enters Attack, EXPECT a shout (sfx 111..115) audible. gdbwatch on
   `Drone_SfxInfo+0x08` (0x2812d0 + 0x33·32): EXPECT lastFrame stamped at the shout [M 0x141b00].
2. Same session: step onto the bunker plate #389 (−47.0, 6.0, −23.3) — EXPECT ch 0x81, the trio
   wakes and (on the game's stream) its AttackTalk coins decide who shouts [M 0x13eac0].

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).

## Suite appendix (state at the 15:35 UTC checkpoint)

- Daemon started 14:56 UTC on the merged tree (rain-3 + master) with the double-fork wrapper
  (exported PATH/PKG_CONFIG_PATH/NF_ISO first); build `799 test binaries` finished ~14:55.
- At 15:35: 417 binaries run, no total line yet. Results so far:
  - FAILED (6): the 4 documented slim-ISO reds (`identity_disc` + 3 `disc_golden` — the
    driver's rule 5 allows exactly these), `bathhouse_shout_mixer_live::…` (**stale — the
    pre-fix binary; the fixed test passes 2/2 directly**), and
    `interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire` (documented above).
  - CRASH (exit −9, the sandbox OOM set): net_match_end_clients_live, frame_cost_live,
    mp_online_failures, net_soak_faults_live (+ the usual net_*/mp_* set as the run proceeds).
- The daemon keeps running until the sandbox dies; the next session should read
  `/tmp/suite4_daemon.log` and `/home/z/suite.log` before re-running anything.
