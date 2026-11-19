## Your task: `retune-nb-08-3`, 08 whole-mission 00 Agent real-damage walk green again (round 3)

Branch from current origin/master; if origin/s/retune-nb-08-2 is not on master, merge it in first. Read docs/reports/retune-nb-08-2.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-19T03:39Z

---
## Report (done, 2026-11-19T04:52Z)


# retune-nb-08-3 — report

Branch `s/retune-nb-08-3` = `origin/master` 73822d3a0 + `origin/s/retune-nb-08-2` merged in
(fast-forward to 876a97f85; 08-2 was not on master). Task: the 00 Agent (difficulty 3)
whole-mission real-damage walk `walk_the_whole_mission_on_real_input_real_damage_00agent`
stays ignored; take the next untried lever from its ignore message and judge each lever on
its own real run.

## Done

- **Lever (1) shipped: the hurting-first pick's unseen-cover fallback** (`crates/nf-view/
  tests/common/damage_log.rs`, `return_fire`): the hurting-first arm filtered VISIBLE only
  and returned None when nobody exposed — the #27-class bursters behind the half-open door
  leaves were never engaged. The arm now falls back to the same unseen-cover pick the plain
  and lane arms run (the shooter actually hurting Bond first), opt-in via the UNSEEN_FIGHTS
  thread-local: the d1 real-damage and d2 Agent walks set no UNSEEN_FIGHTS and stay
  byte-exact (proved by the whole-binary run below).
- **Measured effect [L the NF_DMG_TRACE lever run]**: the BOARDING LEDGE IS PASSED with
  ZERO #27 damage — #27 (skin 0x500002f) never lands a round (he burst 21.0 in 7 hits in
  the round-2 run) — and the walk reaches the CABIN fight at (111.94, 13.09, −179.18), the
  route's deepest point: death t21777, 61 hits for 100.0 (17 return-fire fights) — #16
  28.0/5, the y-46 trio 36.0 (#14 16.0/4 + #11 12.0/3 + #2 8.0/2), world 18.0/34 with the
  last −2.00 drips at frames 20809-21778 closing a zero-health walk. The upstream bleed now
  dominates, which is exactly the ledger's lever (2) target. Reproduced exactly on the
  re-check run after lever (2)'s revert (deterministic, same ledger).

## Levers tried and retired (each on its own real run)

- **Lever (2) the mid-fight scoop — TRIED and DEAD on three real runs.** The mechanism the
  ledger named ("the fights' 60-tick-per-tap reload cycles are the exposure engine") is
  real but the fights are not DRY: the trace shows the transit trio fights running clip 3-7
  spare 27-41 [L the NF_DMG_TRACE+NF_RF_TRACE run] — the spares never run out; the 61-tick
  burn between taps is the exposure, and each 10-hp guard took ~230 ticks while the armed
  fights (the 0xa clip 30 after the deck-head scoop) kill the same guards in single-digit
  ticks at ZERO cost (the walkway pair: two fights, 18 ticks total). Three variants measured:
  1. The clip+spare < 10 gate (the fights' own low gate): NEVER FIRED — the fights are not
     low; death t21777 unchanged.
  2. The clip < 8 gate with the pass's corridor widened to 14 u: the widened scan walked
     Bond to a PLACED box 9.9 u off the line ((156.1, 45.9, −85.7)) — 190 ticks of
     open-deck walking into the trio's own lane; death t1325.
  3. The clip < 8 gate at the master 9 u corridor: walked to the (150.7, 46.3, −96.1) drop
     mid-fight; death t1759.
  Every walk-under-fire variant costs more than the armed tail saves. The trio's own drops
  sit 9-11 u off the walked line, outside the master corridor, and the arm-up that would
  matter lands only at the deck head, after the transit is paid. The scoop code is reverted;
  the tree carries lever (1) only. The ledger and the next lever live in the ignore message
  (commit 3fcf23d91).

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live (whole) | 6 passed / 0 failed / 1 ignored (the d3 ledger), 153.26 s — d1/d2 byte-exact |
| walkreal_07000003_live (glass fights) | 5 passed / 0 failed / 5 ignored, 416.13 s |
| walkreal_07000011_live (unseen fights) | 4 passed / 0 failed, 51.55 s |
| walkreal_07000012_live (unseen fights) | 4 passed / 0 failed, 25.32 s |

The d3 test ran five times this window (the lever-1 run, the three lever-2 variants, the
post-revert re-check), each with NF_DMG_TRACE=1; the ledgers live in the ignore message and
the comment block above the test. The other walkreal binaries (01-07, 09, 0a, 0b, 0c×4, 0d,
11_route, 13-16, 1b, 41×2, 4a×2) did NOT run: the sandbox could not build them (see below).
The changed helper path they share is the hurting-first arm + its UNSEEN_FIGHTS-gated
fallback; 03/11/12 above are the binaries whose walks set those flags, plus the 08 binary's
own d1/d2 tests — the flag matrix (glass only / unseen only / both / neither via the 08 d3
wrapper stack) is covered by the runs that did fit.

## What did not run

- The full four-crate suite does not fit this sandbox, twice over: the background
  `tools/test/suite.py` was killed between tool calls twice (empty suite.out, no target
  formed — the same kill the previous window recorded), and the foreground four-crate
  `cargo test --no-run` build filled the 9.9 GB disk to 92% at 04:15 UTC (`target/` at
  3.9 GB, 808 MB free) before nf-view's own binaries finished. Freeing the browser caches
  (1.2 GB) and a stale `target-scoped/` (785 MB) kept the per-binary builds alive; the run
  list above is what the disk could afford. No `crates/*/src/` files were touched by this
  task (one test-file plus one tests/common helper file plus docs), so no library test can
  move.

## What's left

- The Hind outlast (the ledger's lever (3)): pin the machine's route period from the flight
  log and outlast the high phase into the low passes — the d2 phase's y 13-24 passes gave
  five standing-slit launches in 2100 ticks. The walk now reaches the cabin fight with a
  spent ledger, so the fight itself is the last chokepoint.
- The full-suite pass on the merged final tree, when a sandbox with disk for it exists.

## Live checks for the main agent

For the Hind fight's route-period pin (the next lever), during a fresh-boot 08 playthrough
at the cabin fight (Bond inside the cabin at ~(112.5, 13, −179.4)):

1. `gdbwatch` the machine's position (the Hind entity's pos triple) at 60 Hz for ~30 s of
   fight time, access type read, expected: the y band alternates between the HIGH passes
   (y 25-29, the nb-scan-all circuit) and the LOW passes (y 13-24, the d2 circuit) — the
   period of that alternation is the lever's target.
2. `gdbwatch` the machine's yaw/heading over the same window, access type read, expected:
   the turn reversals bound each pass; the reversal count per 30 s pins the route period.
3. `fieldprobe` poke Bond's pitch to the standing-slit sight line during a LOW pass,
   expected: `line_ok` flips TRUE within the pass window (the d2 measurement) and FALSE in
   the HIGH phase — confirming the outlast tactic has launch windows.

## Usage

not visible to me — this session's driver does not expose token, cost or context-window
figures to the worker.
