## Your task: `level-13-guards-start-alerted-6`, the vision-bit census with Double Cross Part 1 still finishable

Continue `origin/s/level-13-guards-start-alerted-5` (merge `origin/master`). Its five families are green, but on the merged tree the full suite has two reds, both on 07000001 - the level the player must be able to finish: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` ("the contact must walk his route to the LoadPointActivator plate", :406) and `following_mayhew_down_completes_the_level` ("the AND-touch must fire with Bond AND the parked contact on the plate", :891). Both are green on master. With the vision bit on the escort/ally states [M 0x15e4f0:24, 0x15e698:21, 0x15eda8:20, 0x15ec08:18, ...] the contact now sees, and his route ends somewhere other than the plate (AllyLeadHide on a live armed guard [M 0x15ec08 case 3 + NotNearArmedDrone 0x177640], or the band release). Find what the game's contact does at the stall [M] and fix the port, or re-derive the walk's fight/route with the cite; Mayhew must still park on the plate and the level must end. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-28T11:01Z
