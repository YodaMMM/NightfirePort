# retune-nb-05 — the 05 door-band exit to 06 on 00 Agent green again

**Branch** `s/retune-nb-05` (from origin/master 5144d18a8; merged origin/master before the final
runs — "Already up to date", the branch point IS current master).

## Done

`the_door_band_exit_chains_to_07000006_walked_real_damage_00agent` is **un-ignored and green**.
The whole `walkreal_07000005_live` binary runs **17 passed / 0 failed / 0 ignored** in 72.2 s
(`--release --test-threads=1`, slim ISO, the final tree).

## The measured causes and the fixes (all levers 00 Agent only)

The nb-scan-all ignore ledger ("Bond dies t10830 at (43.01, 45.19, −73.11): every gun dry at the
walkway pair [L 2026-11-18 run]") reproduced exactly, with a fuller ledger:

1. **The master weapon cycle livelocks on the dry 0x16.** The walk arrives at the strip's west
   corner holding only a dry Q-CLAW (0x50) and a 0x16 with `clip 6 spare 0` — plus a **0x24 with
   `clip 5` picked up on the ditch-sniper flank walk** [L the 2026-11-18 state census: owned
   `0x24 clip 5 carried 0` at the strip]. `Walker::switch_to_armed` presses `weapon_cycle` and
   waits 60 ticks; the 0x24's draw outlasts the wait, so every press re-queued the cycle from the
   old gun — the measured run printed 12 cycle lines all `weapon 0x16 clip 0 spare 0`, the 0x24
   never armed, and the answer burned 732 ticks dry while #13's bursts knocked Bond off the strip.
   **Fix:** a level-local `cycle_armed_slow` in the 05 test file (one press, waits up to 400 ticks
   for the in-hand id to change before the next press, same armed predicate). The answers lead
   with the 0x24 — **30.0 damage per round [L ee_start_05.bin weapon_data 0x2bf150 +
   0x24·0x10C]** vs the 0x16's 2.75 — one round drops a 10-hp guard, where the 0x16's grazes
   measured 1.6-1.7 and #13 survived six of them at 0.1 [L the 2026-11-18 run].
2. **44.0 of the pre-strip cost was #3**, the level's only 20-hp guard [L the state census:
   (29.86, 35.23, 71.12) hp 20.0 weapon 0x16]: 4 hits of 11.0 (the 0x16's headshot tier), 2 of
   them BEFORE the return-fire fight opened (health 100 → 78 → 56 [L the repro]). **Fix:** a
   lane-approach pre-emptive stand in `walk_to_the_slab_east_end` (the shared route's strip-west
   leg, 27 u out, inside the P99's 50 range [L ee_start_05.bin record 2]), gated
   `sim.drones.difficulty >= 3`. The pick is #3's **health signature (> 10.0)**, not a spot: the
   first lever run used a radius-5 spot pick, found nothing (the ditch/wall stands' gunfire had
   already moved him), and the 44.0 landed unchanged [L the 2026-11-18 lever run].
3. The strip answers run for every difficulty (master's shape, unchanged for d1/d2); a third
   answer after the walkway south 1 leg is d3-only (the door guards #271/#272 [D placements
   (41.5/46.4, 46, −69.7)] stand where the legs pass within 1.2-1.6 u).

**The passing run's ledger** [L the 2026-11-18 pass]: 6 hits for 24.8 the whole walk, end health
75.2, #3 killed at t2928 (health after 83.5), #12 one-shot at the west corner (6.9 u), #13
one-shot at the walkway south 1 answer (1.3 u), ch 0x1E written and `level_exit == 0x07000006`
at t11844.

## Gating / byte-identity

All changes are inside `crates/nf-view/tests/walkreal_07000005_live.rs` (the 05 file) — no
`crates/*/src/`, no `tests/common/` files, no shared file rewrites. The d1/d2 sisters pass
`slow = difficulty >= 3` = false, so they keep the master cycle and their proven rolls; the
whole-binary run above passed them unchanged (17/0/0). No guard code touched; no damage or
health constants changed; no top-ups; all input real (weapon-cycle presses, reloads, walks,
stands through the walker's own shot path).

## What was run

- `walkreal_07000005_live` (the whole binary, final tree): **17 passed / 0 failed / 0 ignored**
  in 72.2 s.
- The lib suites (final tree): **nf-game 796 passed / 0 failed / 4 ignored**; **nf-formats
  92 / 0 / 0**; **nf-audio 29 / 0 / 0**. No FAILED/TIMEOUT lines anywhere.
- **The full 422-binary suite did not fit this sandbox**: the all-crates `cargo test --no-run`
  filled the disk (the linker died with `No space left on device` at `copter_hits_12_live`
  after ~5.3 GB of nf-view test executables); after freeing them only the lib binaries plus the
  05 walk binary fit the remaining window. Per the merge lessons this is reported rather than
  skipped silently: my change touches one nf-view test file only, and that binary plus the
  three lib suites ran whole.

## What's left

Nothing for this task's goal. Notes for the next worker: the temp state-census probe used for
the evidence was deleted after use (its numbers are quoted in the test comments as
"[L the 2026-11-18 state census]"); the walkway answer's `prefer 0x24` leaves 1 spare 0x24
round after the door guards, so a fourth engagement past the walkway would be dry — the route
has none before the exit.

## Live checks for the main agent

None required. (Optional: a PCSX2 fresh-boot spot check that a 07000005 guard's dropped 0x24
picks up with `clip 5` when crossed — the port's equip path [M Player_EquipWeapon 0x1a8a58]
gives the drop's own rounds; the walk's strip fight depends on it.)

## Usage

Not visible to me: this session shows no token, credit or context-window figures in its
environment; nothing to copy.
