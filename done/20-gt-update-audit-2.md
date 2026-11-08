## Your task: `gt-update-audit-2`, a 0700000b turret wrecked while deployed behaves as the game's

Branch from origin/s/gt-update-audit, merge current origin/master, and read its docs/reports/gt-update-audit.md. Its pins run from the DORMANT state, so a 0b turret wrecked while deployed is unpinned (see its geometry note). Read that path in `GT_Update` 0x180498 and `GT_ApplyDamage` 0x1816d8 and pin it through real play on 0700000b: Bond shoots a deployed turret until it is wrecked, and the fold, the smoke and the stop happen as in the game. Then settle the class-5 gate on the 0x4a/0x4c disable from the disc: is any turret script object on 0a or 0b class 5 [D]? If the disc answers it, replace the [S] stand-in with the real class read; if only a live read can, list it under the report's live checks. Run the turret binaries, walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gt-update-audit-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T09:06Z

---
## Report (done, 2026-11-08T10:44Z)


---
## Report from the worker (done, 2026-11-08T10:44Z)

