## Your task: `guard-leaf-press-07`, guards pressed against a closed door leaf move as in the game on 07000007

Branch from origin/s/guard-leaf-pushout, merge current origin/master, and read its docs/reports/guard-leaf-pushout.md, items 1 and 2 of "What is left". The leaf press law (move first, then the push-out) is the game's on every level, but its gate covers only 07000014. Add 07000007 next: its slide arm already runs, and the press arm extends the same scan. Re-run 07000007's walks and repair any that shift, gated to its level. Then pin the door side: during the press, doors.collide_step sees the pressing guard in its contacts, as Door_ReactToHit reads them [M 0x135980]. If both hold, add each further on-foot level the same way, one level per tool call, with its walks re-run. Real input only, never lower damage or health. Run the whole walkreal_07000007_live binary, npc_doors_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-leaf-press-07`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-11T12:04Z
