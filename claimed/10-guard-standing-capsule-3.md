## Your task: `guard-standing-capsule-3`, every level's walks stay green with guards keeping their standing body while firing

Branch from origin/s/guard-standing-capsule-2, merge current origin/master, and read its docs/reports/guard-standing-capsule-2.md. It repaired the 01, 0a and trench walks the standing body moved and recorded seven more casualties with evidence under "07000002, 03, 04, 05, 08, 16, 09". Take them in the report's order, 08 and 05 first, then 02, 16, 04, 03 and 09, each repaired gated to its level and difficulty through real input, never by changing the capsule rule, which is the game's [M NDrone2_Collision 0x140e90:91-121]. Then the walkreal binaries the report did not re-run. Run every walkreal_*_live one level per tool call, never several in one call, so the sandbox disk and the tool cap hold. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-standing-capsule-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-09T07:35Z
