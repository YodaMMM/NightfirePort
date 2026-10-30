## Your task: `level-08-music-plate-ride`, 08's music plates reached the game's way

level-08-interactables-2 (merged) found mux #102 is an AND fold (out 0x68 = 90 AND 91 AND 92 [M Trigger_MultiplexIn 0x1b2c68:15]) and left open how the player reaches plates #99/#101 (their volumes sit at y 46-49 over x 117-131 [D]) — the gondola ride is the candidate. Find the game's route to each plate [D placements/collision; M the gondola/mover class], walk it on real input, and complete the mux test's fold + map-sound legs without placing Bond. Fix any port gap with evidence. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-08-music-plate-ride`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T22:06Z
