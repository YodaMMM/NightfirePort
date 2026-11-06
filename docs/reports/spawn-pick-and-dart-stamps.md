# spawn-pick-and-dart-stamps — report

Branch `s/spawn-pick-and-dart-stamps` (from `origin/master` c4c00c7ec). 2026-11-06, written
~03:58 UTC. Task: port two drone-AI divergences found reading the decompile, each with a
fail-first test.

## Result in one line

**Both divergences ported with fail-first tests, plus the spawn-chain draws; the spawner-level
and drone.rs-mandated live binaries run green; ONE walk pin broke on the shifted stream
(walkreal_07000005_live's 00agent 05→06 chain) and is documented below as the re-pin that is
left.** No pin removed, no draw removed — the re-pin shipped this session is by route.

## 1. The spawner pick — PORTED [M 0x137cb0:21, 0x137aa0:31-32, :41-56, :88-96]

The game's `DroneSpawner_SpawnDrone` draws `Rand_Rand` over the WHOLE template list
(`DroneSpawner_Init` 0x137aa0:31-32 writes the SAME census count to `+0x8`, the walk bound, and
`+0x2E`, the draw modulus), drawn BEFORE the walk; only USABLE records — unused this epoch, plus
the min-distance screen when `+0x1F` is clear — decrement the drawn rank (:41); a used or
too-close record is walked past without consuming it (:88-91). The (rank+1)-th usable template
spawns; when the rank passes the last usable record the loop falls through and returns 0 —
NOTHING spawns this call, and `Control` retries next tick with a fresh draw. The port drew
`rand(usable.len())` (spawners.rs:194) — always inside the usable set, always spawning — so
waves arrived at the wrong rate whenever any template was used or too close.

Fix (crates/nf-game/src/spawners.rs): draw `rand(self.templates.len())` first, then the usable
walk, `if rank >= usable.len() { return Action::None; }`.

**Fail-first** (module unit tests, `#[cfg(test)]`): `the_pick_draws_over_all_templates_and_a_
missed_rank_spawns_nothing` and `the_draw_rank_indexes_the_usable_walk` both FAILED on the old
code (the modulus assert read 2, the missed rank spawned anyway); both pass ported;
`a_rank_zero_spawns_the_first_usable` passes both models (rank 0 agrees).

## 2. The spawn-chain draws — PORTED [M 0x14b300:382-461, 0x137cb0:58]

The port made none of the game's spawn draws. Ported into the `Action::Spawn` handler
(crates/nf-game/src/drone.rs), in the game's call order:

- the param-14 class switch re-rolls at EVERY spawn (the spawn re-runs the full
  `NDrone2_DefaultInit` — the same block `Drone::new_with_stats` runs at load, there riding the
  pre-snapshot stream, here consuming the live one): two `Rand_FRand` draws per spawn (case 4's
  combat range draws none — the decomp's `goto LAB_0014bb44`), the sight/engage/combat ranges
  written from the same constants [M :382-434];
- the +0xF4 tail draw `Rand_FRand(0)` [M :435] — consumes the stream; the value is +0xEC(4.0) +
  0.0 and the port has no +0xF4 field (noted below);
- the 0x7000011 / 0x7000014 engage/combat overrides [M :437-461; the 0x7000011 decomp condition
  `(uVar9 < 4) || (uVar9 < 7) || (uVar9 != 9)` reduces to `uVar9 != 9`];
- `SpawnDrone`'s own `Rand_Rand(10000)` → obj+0xEC [M 0x137cb0:58-59] — the spawn re-roll of the
  miss-wobble phase. This settles the port's `wobble` field doc ("rewritten after creation by an
  unidentified writer"): on the SPAWNER path the writer is SpawnDrone's own stamp.
- +0xE4 = π/2 re-stamped [M :463, LAB_0014bb44's unconditional write].

Verification: a wave's spawn frames against a live PCSX2 run is a live check (below); the
port-side pins are the whole-binary results — the Drake spawner waves (walkreal_0700001b, 17
tests) still pass with the new stream.

## 3. The dart's Tower stamp block — PORTED [M 0x1469b8:331-357 vs :103-128]

The decomp's `DroneFunc_HandleImpact` case 0x19 (the dart) carries the Tower band block
BYTE-IDENTICAL to the tase's case-7 copy (:331-357 vs :103-128): unsighted → the resume 0x14;
sighted → first-attack latch off (`+0x4F8 &= ~0x02000000`), alt state 0x89 RunToAlarm (`+0x5A4`),
`+0x16 = 0`, alert bits 0 (`+0x4FC`), the `+0xC48 == 100` first-behaviour scratch, mode 9 /
alt mode 7 (`+0xC5/+0xC6`), the sight latch off (`+0x4F8 &= ~0x04000000`), return state 0x14
(`+0x5A2`), resume 0x15. The port's `on_dart_impact` picked only the resume and skipped the body
(the tase ran it whole). Ported in crates/nf-game/src/drone.rs, mirroring the tase arm.

**Fail-first**: `the_dart_stamps_the_tower_alert_block_like_the_tase` (walkreal_07000009_live.rs)
— the challenge scenario IS the sighted arm (the challenger walked to Bond). Pre-fix run quoted:
`state 0x51 mode 9 alt_mode 14 alt_state 0x89 sighted 1 return_state 0x10 resume 0x15 alarm_run
0 alert_bits 0x0 first_attack 1` — alt_mode/return_state/sighted/first_attack all wrong; the
asserts fired. Post-fix: all nine stamp asserts pass [M :337-355].

## Walk re-pins, by route

- **walkreal_07000009_live::the_alarm_switch_fails_objective_1** — the dart stamps (correct per
  the decompile) arm a woken sleeper's alarm run, and the OLD approach hop WEDGED ~3769 ticks at
  the switch's ~1.08 u floor (its "arrival" was the level-exit latch passing the assert truthy),
  gifting the waker his press window. Re-pinned by route: the approach beat-splits with clearing
  beats and accepts the PRESS reach (dxz < 1.4, the arrival assert's own bound) — no wedge, no
  window. Green whole (below).
- **walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage_00agent**
  — NOT re-pinned yet. Death t9526 at (−35.02,38.49,−43.14), 13 hits for 100.0, 4 return-fire
  fights: the shifted stream (05's proximity spawner attempts consume a draw per attempt now)
  made the 00-fights costlier (40.0 hp at t2174, 29.0 at t3118, 23.5 by t4321) and the courtyard
  descent finished Bond. A real-damage 00 re-pin (×1.3 accuracy, the ledger class the earlier
  reports rank hardest) — the next session's first item. The other 13 tests of that binary pass.

## Whole-binary results (quoted, `--test-threads=1`, this branch's tree)

- `nf-game` lib — **686 passed; 0 failed; 5 ignored** (incl. the 3 new spawner unit tests).
- `walkreal_0700001b_live` — **17 passed; 0 failed; 2 ignored** (124.16 s) — the Drake spawner
  waves, the task's named comparison level.
- `walkreal_07000009_live` — **9 passed; 0 failed; 0 ignored** (13.72 s) — incl. the new dart
  stamp pin and the re-pinned alarm test.
- `walkreal_0700000a_live` — **13 passed; 0 failed; 2 ignored** (48.01 s).
- `npc_doors_live` — **6 passed; 0 failed; 1 ignored** (27.74 s) — the drone.rs mandate.
- `fix_guard_misc_live` — **6 passed; 0 failed; 0 ignored** (7.24 s) — the drone.rs mandate.
- `drake_fight_live` — **3 passed; 0 failed; 0 ignored** (15.90 s).
- `walkreal_07000005_live` — **13 passed; 1 failed** (69.53 s) — the 00agent chain above.
- `walkreal_0700000c_live`, `walkreal_0700000d_live`, `walkreal_07000012_live` — built, NOT run
  in the window (the mode-3 stinger pair and the 647 spawner are the next verification).

The full suite did not run in the window (sandbox ~2 h, the setup + three fixes + the re-pin
consumed it); every binary that compiles the changed code and was runnable is listed above.

## What is left

1. The 05 00agent chain re-pin (the ledger above is the starting point).
2. Run walkreal_0700000c/0d/12 whole (built).
3. The port has no +0xF4 field — the tail draw is consumed, the 4.0 value unmapped (the game's
   readers of +0xF4 are untraced; the old port note calls it the "spawn-state field").

## Live checks for the main agent

- **0700001b wave frames (gdbwatch)**: the task's own comparison — a live PCSX2 run of the
  Drake fight's spawner wave, recording each spawned drone's first-visible frame; the port's
  wave (walkreal_0700001b's spawn_seq/spawn_frame logs) should match once the pick draw +
  spawn-chain draws are in. The seeds differ (the port seeds from the level-start snapshot), so
  compare the RATE (frames between spawns) and the COUNT, not the absolute frames.
- **07000009 sleeper wake (fieldprobe)**: poke `obj+0x4F8 |= 0x4000000` on a sleeping tower
  guard, force `+0xCE0` to a short value, and confirm the wake runs the 0x89 alarm run the stamp
  arms — the behaviour the re-pinned alarm test now routes around.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session, so there are no numbers to copy.

## ntfy failed

All three fallbacks attempted in order; all failed (the same measured pattern as the previous
sessions — no outbound route to ntfy.sh from this sandbox):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds` (rc 28)
2. `python3 -c "import urllib.request as u; …"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `timeout 25 wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x`
   → killed by the 25 s timeout, rc 124, no output.

The notification could not be delivered; the queue marker (`done/20-spawn-pick-and-dart-stamps.md`,
report attached — finish.sh printed `OK done 20-spawn-pick-and-dart-stamps`) and this report on
the branch are the delivery of record. Recorded 2026-11-06 ~04:05 UTC.
