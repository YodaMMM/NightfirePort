## Your task: `capsule-cull-and-los-masks-11`, the last two cull/masks reds

Continue `origin/s/capsule-cull-and-los-masks-10` (merge `origin/master`). Two reds remain, each with its evidence chain in the -10 report:
1. walkreal_07000001 `following_mayhew_down_completes_the_level` (cull): the plate descent pins the capsule between the z-11 tread edge and the z-12 soffit at (-33.44, 4.72, 12.03). Port the game's walk-DOWN behaviour from the decompile [M Player_Collision 0x194df0's ground snap / step-down path, Player_StandAtNewPosition 0x190d88, the walk's floor scope] - what distance the game snaps down a stair riser while walking and whether the head-on cull applies to the descent's soffit - and turn the walk green without re-deriving the leg unless the decompile shows the leg itself was non-game.
2. mp_perf_16 `sixteen_participants_fixed_run_digest_is_pinned` (both): dump the per-tick net_state_hash streams of a master build and this tree, name the first diverging tick's mover, and re-pin only with that mover cited.
Double Cross Part 1 must stay finishable. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T15:42Z

---
## Report (done, 2026-10-28T16:55Z)

Closed by the driver: the report is on `s/capsule-cull-and-los-masks-11` (docs/reports/capsule-cull-and-los-masks-11.md); the worker didn't run finish.sh.
