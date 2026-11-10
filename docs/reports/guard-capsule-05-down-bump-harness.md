# guard-capsule-05-down-bump-harness — the 05 down-bump harness runs; the game's 0.3 down ball confirmed on 07000005

**Branch** `s/guard-capsule-05-down-bump-harness`, cut from origin/master (4bdaf3f5).

## What was done

**The 05 harness runs green — no deletion needed.** The first pass (2026-11-09) had ignored
`guard_capsule_radii_live::a_down_guard_bumps_at_the_games_03_ball` with two terrain blockers:
the gatehouse chase ran the courtyard guards downhill (the closing legs chased #6 from y 26.2 to
18.7, the `|dy| < 1.5` same-floor finder never passed), and the Stunner beam's wall gate refused
a target across a ledge drop.

**The census answered both** (the task's `Level::parse` count, run as a scratch probe on
07000005's placements): 07000005 carries **16 kind-0x0F drone placements** [D the parse dump],
and two of them stand ON the courtyard west band — **#5 skin 0x500002e at (61.34, 27.13, 99.50)**
and **#6 skin 0x5000030 at (50.85, 28.27, 93.63)** — the same floor as the established
courtyard-west nav route's end (60.1, 28.4, 96.4) [D; the interact_07000005 ladder/pickup walks
run that route clean, and the spawn floor is the courtyard, start #26 y 25.1]. The walkreal
fights engage these same guards at Bond's floor (the nav 19/20 return-fire kills at
(73.6, 25.0, 98.4) / (63.8, 26.7, 98.1) [L the walkreal_07000005_live run]) — they hold their
post and fight on the west approach; the downhill flee was the gatehouse approach's shape.

**The harness change** (the test file only — no lib, no shared helper): the pre-finder leg is the
courtyard-west route (`walk_nav_to([60.1, 28.4, 96.4], "courtyard west", 1.2)`) instead of the
gatehouse; the same-floor finder, the closing legs, the tase hold, and the creep + latch probe
are the first pass's and the 07000006 sister's proven machinery, unchanged. The first pass's
dead pre-walk bisect block (its own comment said it measured nothing) is removed.

**The measurement** [L the run, deterministic across two runs]:
- The finder picked guard #2 at dxz 2.62 after the walked route.
- The held Stunner fired on the WALKED Bond immediately (`tase_accum 1 clip 99` at hold tick 0)
  — the first pass's "the beam never fires on the walked Bond" blocker never bit on the west
  route: it was the ledge-crossing aim (the pitch converged −0.875 across a ~9 u drop), not a
  walked-view interaction.
- The guard fell into TASER at hold tick 20, then STUNNED with the +0x2f down flag
  [M 0x162b98 case 0xc, 0x162eb0 case 1].
- The creep's latch (the guard's own +0x26 body-contact latch, DealWithObjHit case 3, 0x1405c8)
  **first fired at dxz 0.772** — the game's 0.55 + 0.3 = 0.85 ball, where the old standing-0.4
  capsule lit at ~0.95. The 07000006 sister measured 0.771 [L cap06_run1]; 05 now reproduces the
  same band [L two runs, 0.772 both].

## How it was verified

| binary | result |
|---|---|
| guard_capsule_radii_live (whole) | **2 passed, 0 failed** (3.4 s) — the 05 harness AND the 06 sister |
| walkreal_07000005_live (whole) | **16 passed, 0 failed**, 1 ignored (73.3 s — the ignored one is master's parked notch test, untouched) |
| guard_capsule_live (the sibling capsule binary) | **2 passed, 0 failed** (19.1 s) |
| interact_07000005_live (the courtyard-west route's own file) | **11 passed, 0 failed** (27.2 s) |

The change touches the one test file only — nothing under `crates/*/src/`, nothing under
`tests/common/` — so per the merge rules the binary list above is the touched set (the file
itself, the required walkreal_05, and the two binaries sharing the route/helpers it drives).
Determinism: the latch distance 0.772 reproduced identically on a repeat run (bit-exact RNG,
fixed 60 Hz).

**The fast suite:** `tools/test/suite.py` cannot run in this sandbox — background jobs are
killed between tool calls (attempted again this session: the runner and its rustc children die
within the minute; `suite.out` stays empty). The chunked foreground runs above stand in; the
final tree differs from origin/master by ONE test file, so the suite's exposure is exactly the
four binaries listed.

## What is left

- Nothing on the task's own question: a same-floor 05 guard EXISTS and the harness runs — the
  deletion branch of the task does not apply.
- The remaining `walkreal_*_live` / `interactables_*_live` binaries did not run this session
  (the suite-runner constraint above); the changed tree's exposure is the four binaries run.

## Live checks for the main agent

- None required: the port's 0.3 down ball was already pinned by the 06 sister's live shape, and
  the 05 harness now pins the same measurement through the port's own collision. If wanted for
  completeness: on a fresh boot at 07000005's courtyard west, `gdbwatch.py` on the downed
  guard's +0x26 latch while Bond creeps — the first set should land in the 0.85 band
  (the port reads 0.772 [L]).

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment; the
session wall-clock: 12:59 UTC start, ~14:58 UTC sandbox expiry — the work finished inside the
first half hour).
