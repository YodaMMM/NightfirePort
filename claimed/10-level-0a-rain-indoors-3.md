## Your task: `level-0a-rain-indoors-3`, the last two reds from the rain fix

Start from `origin/s/level-0a-rain-indoors-2` (merge it first; its report has the shared-generator chain [M Rand_* 0x1e36f0..0x1e3780, Env_Update 0x17bc88, AddDrop 0x18c278, UpdateDrops 0x18c580 rain branch]). Rain now steps exactly as the game does, and 5 of 7 reds are green. Still red, each green on master:
- `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`
- `escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`

1. For each, find what the shifted random stream changes on its path (which draw, which drone/door/channel decision), and whether the test's expectation rests on a port-only random value [S] rather than a game rule. Where the walk depends on one lucky draw, make the walk play it the way a player would with real input (never weaken an assert); where the port's draw ORDER still differs from the game's (other per-frame draws: Env_Update's Rand_Rand(20000), the AI's draws, their order within a frame [M]), fix the order.
2. Both green with rain at the game speed; the 5 recovered tests still green; run the FULL suite on the final tree merged with origin/master and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-rain-indoors-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T09:58Z
