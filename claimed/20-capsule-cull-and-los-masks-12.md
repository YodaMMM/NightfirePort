## Your task: `capsule-cull-and-los-masks-12`, the cull/masks branch onto master: the last walk red

Continue `origin/s/capsule-cull-and-los-masks-11` (merge `origin/master` - it now carries the vision-bit census and the ally fixes). mp_perf_16 is closed there. The one red left: walkreal_07000001 `following_mayhew_down_completes_the_level` jams at (-33.44, 4.72, 12.03) on the plate descent. Your -11 decompile reading shows the port's walk-down (the 0.1 glue band [M Player_FeetOnPoint 0x194188:106], gravity per riser, the soffit cull arm) matches the game on every cited term, so the jam is the test's approach line into the stairwell, not a port rule. Re-derive the follow leg's descent on the main walk's proven hall-corner + stairwell-head shape (the line `walk_from_the_spawn_to_the_exit_with_the_contact` walks green), with the cite; Mayhew must still park on the plate and the level must end. Run the FULL suite on the merged tree and quote its total line - every red accounted for.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-12`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T12:38Z
