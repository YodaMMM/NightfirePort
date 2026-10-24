## Your task: `level-0a-office-hatch-6`, the spawn settle without the 05 truck-ride regression, then the 0a hall

Start from `origin/s/level-0a-office-hatch-5` (not merged): its spawn arrival settle reproduces the 0700000a rest 1.0891111 bit-exactly [L ee_start_0700000a], but it moves every level's spawn by -0.0109 and `walkreal_07000005_live::walk_to_the_gatehouse_and_ride_the_truck` no longer parks on the ch-0x70 plate. Its four-tick hold is modelled from the observable, not read from a decompile line.

1. **The 05 regression first.** Run the truck ride on both trees, log Bond's root and the ch-0x70 plate box at the park tick, and name which side he misses by. Fix it so both hold: either the ride route is re-derived at the settled height, or the settle is wrong somewhere [M]. A branch that turns any currently-green test red is not merged.
2. **The hold.** Find the gate that holds the placed root for the boot's first ticks (the boot/level-start path after `Player_StandAtNewPosition` 0x190d88, the first `Player_Update` 0x193600 calls, `GS_PausePlayer`, the fade-in) [M]. If it cannot be found, keep the observable model but mark the hold `// [S] unverified` and keep that part out of pass/fail.
3. Then the hall glide (game 1.1147 after the doorway-base climb at x -8.304) and the corner at (-17.87, 4.87), and un-ignore the sisters and the main walk in `walkreal_0700000a_live` as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox1 at 2026-10-24T05:08Z

---
## Report (done, 2026-10-24T06:06Z)

