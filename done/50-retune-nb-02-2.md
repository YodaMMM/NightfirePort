## Your task: `retune-nb-02-2`, 02 castle 00 Agent real-health walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-02 is not on master, merge it in first. Read docs/reports/retune-nb-02.md there and the ignore message on walk_the_castle_all_objectives_real_health_00agent. The geisha phase is green now; the walk dies past the stair bottom. Take the next lever from the report and fix that fight. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000002_live binary and interactables_07000002_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-02-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-18T19:18Z

---
## Report (done, 2026-11-18T20:28Z)


---
## Report from the worker (done, 2026-11-18T20:29Z)

# retune-nb-02-2 — 02 castle 00 Agent real-health walk, round 2

Branch `s/retune-nb-02-2` (origin/master 73822d3a0 + the merge of origin/s/retune-nb-02,
which was not on master). The task: the geisha-1 phase was green after round 1; the walk
died past the stair bottom (t27653, the lane crossing, 7.8 health, the sniper's −48.0
body round). This round took the next lever from `docs/reports/retune-nb-02.md`, judged
every variant on its own real run, and moved the ledger one full phase forward.

**The named death spot is green: Bond now crosses the courtyard lane alive.** The walk
still ends red one phase later — in the bunker-ambush courtyard fight — and the refreshed
ledger + the next lever are in the ignore message on
`walk_the_castle_all_objectives_real_health_00agent`.

## What was done (lever 2-a, landed)

`crates/nf-view/tests/walkreal_07000002_live.rs`, all inline inside the level's own test
file, every change behind the existing `difficulty >= 3` gates — no shared helper under
`crates/nf-view/tests/common/` was touched, so the d1/d2 walks and every other level's
walks are byte-identical (the whole walkreal_07000002_live binary's four other tests
still pass, unchanged).

1. The post-stand-off `clear_pursuers(16.0)` sweep became an inline vantage-shaped duel
   loop (the sweep's own targeting: nearest live hostile within 16 u, hold_for_sight 600,
   approach only a close unseen guard, survivors passed over) with the vantage duel's
   stand (settle_aim + stand_and_fire 900, back_off 8.0) and NO between-fight loot walks —
   resupply only when the wheel is DRY, through the capped scoop_quiet. The repro showed
   the plain sweep's cost was inflated by its loot walks: the fight-0 loot walk wedged at
   the 3000-tick cap TWICE (the nav web hands a 4.9-u drop a route that detours north and
   stalls on a rail) [L the 2026-11-18 repro1 run].

2. The stale ignore message was replaced with the refreshed ledger and the next lever.

## Measured result [L the 2026-11-18 run-3 ledger; run 10 on the final tree is ledger-identical]

- t10952 the geisha-1 stand-off round 0: 87.0
- t15664 the post-stand-off duel 0: 73.2 (cost 13.8 vs the old sweep fight's 36.0)
- t15996/t16434 the drop-scoop's return fire kills the chasers #8 and #6: free
- t30445 the west-wing armour detour done: **49.8 + 50.0 armour** (round 1: 7.8 + 50.0)
- t33543/t33683 the stair legs and the stair bottom: 24.8 (the descent's in-view fights)
- t33925 **the lane crossing: alive at 21.2** — the round-1 death spot passed
- t34062 the lane mouth: 19.4; the door legs + the band stand (the sniper already down):
  12.2
- t39952 DEATH at (−47.84, 6.37, −21.34), the bunker-ambush courtyard fight, 12.2 health,
  a 1-shell shotgun: the door legs' shotgun fights killed the sniper and a door guard but
  burned the shells 4 → 1, the funnel's one-tap mechanic dried after a single kill, and
  the hp-10/12 funnel's point-blank volleys outpaced the dry cycle.

## Levers judged on their own real runs and rejected

- The quiet stair-top arm (arm the silenced P99 for the descent): the P99 is OFF the wheel
  by then (the post-stand-off duels spend it dry; the game drops an empty gun —
  "0x2 not carried", runs 4/5), and the fallback loud leg drew a rifle miss round onto the
  scared civilian #464 → ch 0x60 → FAIL_CIVILIANS at the stair-bottom pin (run 5). The
  run-3 leg had fired 43 of the same gun and gotten lucky — a lottery, not a plan.
- The descent hold-fire (cooldowns across the stair leg): Bond walked the leg UNANSWERED
  and the converged set's volleys took 50.0 armour + 6.2 health (run 6).
- The lane-mouth rifle arm: the arm's wheel-cycling dwelled inside the sniper's shot
  window — the old 02-6 "no dwell" rule — death at the mouth (run 7).
- The arm_fullest lane-crossing arm: arm_fullest raced the draw and landed on the
  bare-hands 0x47 read (0 rounds); Bond crossed on the cuffs and the mid-lane fight had no
  gun (death t34000, run 8).
- The dedicated 0x12 crossing arm: the crossing itself was fine, but the door legs' rifle
  fights then wandered west after the drawn pair (the shoot loop's reposition — the run-4
  x −85 lesson again; death t35082 at (−65.57, 5.91, −0.34), run 9).

The lane shotgun arm (the 02-6 shape) therefore STAYS; the comments in the test file
record each rejection with its run.

## What is left (the next lever, for the next message)

The bunker-ambush courtyard fight needs ~25+ health+armour at its start; Bond arrives
with 12.2 and a 1-shell shotgun. Options recorded in the ignore message: scoop the
funnel's own drops between the stand-off rounds (the kills land at the fight floor), or
arm the fullest gun at the lure when the shells are < 2, and shave the door legs' −7.2.
All d3-gated, judged on their own real runs, per the task rules.

## How it was verified

- `walkreal_07000002_live` (whole binary): **4 passed, 0 failed, 1 ignored** (the re-tune
  target is the ignored one) [L 2026-11-18, 64.9 s].
- `interactables_07000002_live` (whole binary): **18 passed, 0 failed** [L 2026-11-18,
  11.3 s].
- The ignored target itself, run with `--ignored` for every measurement above (red at the
  ambush, as described). Runs 1–10 are the per-lever judgement trail (repro → lever 2-a →
  the five rejected variants → the final tree's confirmation run, whose status/health
  ledger is byte-identical to run 3 — the sim is deterministic and the final diffs are
  comments only).
- `nf-game`: **1233 passed, 0 failed** (--test-threads=1).
- `nf-formats`: **109 passed, 3 failed** — the 3 failures are the slim-ISO golden-manifest
  trio (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`), the only failures the task permits on
  the slim ISO.
- `nf-audio`: **35 passed, 0 failed**.
- The full `suite.py` fast runner was not run: this sandbox is the same 9.9 GB class the
  round-1 report documented — the full nf-view test build (1111 binaries at ~13-20 MB
  each) does not fit, and background jobs are killed between tool calls here, so the
  suite's parallel build cannot survive anyway. The crates' test binaries were built and
  run in foreground chunks instead (the counts above are those runs).
- No shared helper changed (diff scope: the one test file + docs), so per lesson 7 the
  other levels' walkreal binaries were not required to re-run; the task's two named
  binaries were run whole.

## Live checks for the main agent

None required for this round: every figure above is a sim measurement of the real-input
walk. Round 1's live-check request (the 07000002 captain step-aim siege watch) stands if
the main agent wants to pin the 3412a89a3 spacing-scan behaviour as game-true.

## Sandbox notes

Fresh sandbox (the previous session's tree was gone): rebuilt Rust 1.100 (rustup), fetched
work-slim.7z + Nightfire-onfoot.iso.7z (size verified 369823203) and extracted them with
py7zr (no 7z binary; py7zr rejects `-y`), installed libasound2-dev + libasound2 into the
user prefix /home/z/alsa (PKG_CONFIG_PATH + RUSTFLAGS -L), created the branch + the
WORKLOG entry before anything else, and pushed checkpoints after each verified step.

## Usage

Environment figures for this session: input/output token and context-window figures are
not visible to me; cost is not visible to me.
