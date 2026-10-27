## Your task: `level-01-exit-after-the-wait`, Double Cross Part 1 can be finished after Mayhew's wait

On Double Cross Part 1 (07000001), playing through the real path on the current build, Mayhew now waits at the door correctly (the ally-wait hold, node 33 [M NDrone2_ReachedDestNode 0x153358:85-94]) and leads on after the ambush, but the level cannot be finished at the end: Bond at (-33.44, 6.47, 8.16) at the bottom stairwell with Mayhew walking down ahead (frame ~7138), and the exit never completes. Reproduce it on real input from a fresh load (play it the way a player does - clear the ambush, follow Mayhew down), find which link of the exit chain never fires (the contact reaching the LoadPointActivator plate, the AND-touch with Bond and the contact on it, the channel/objective the exit waits for, Mayhew's route after the release) [M the escort states and the exit's trigger/objective chain] [D placements], and fix the port. Test it end to end on that real path (not only the walker's tuned route): the level completes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-exit-after-the-wait`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

