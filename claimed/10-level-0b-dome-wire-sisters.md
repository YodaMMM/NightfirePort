## Your task: `level-0b-dome-wire-sisters`, the four 0700000b real-damage walks cross the dome wire again

Branch from origin/s/walker-sidestep-decay, merge current origin/master, and read its docs/reports/walker-sidestep-decay.md, the walkreal_0700000b_live repair and ledger. The sidestep decay is right and stays; with it, the four 0b real-damage sisters fail at the dome-wire crossing, because every stuck event upstream now resolves differently. Take its lever: re-time the dome-wire crossing against the fusebox windows on the decayed ladder (the 0b-diag block prints them). One walk per tool call. Helpers stay gated to 0700000b and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the temporary diagnostics before the final commit. Run the whole walkreal_0700000b_live binary and interactables_0700000b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-dome-wire-sisters`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: level-0b-dome-wire-sisters at 2026-11-11T08:23Z
