## Your task: `glass-pierce-2`, guards' rounds and glass follow the game's material rule

Branch from origin/s/glass-pierce, merge current origin/master, and read its docs/reports/glass-pierce.md. Take "What's left" in order. First arm the guard-side glass pass from the weapon record at the five spawn sites, and repair `walkreal_07000008_live`'s real-damage sisters for guards firing through intact station glass: the documented tick-25178 death is the walk standing where the game makes it unsafe, so the walk is what changes. Then flip the pass gate from kind 2 to material 0x10 [M 0x12cf54-0x12cf60] and re-pin the 01 breach test, whose crossing half becomes a stop pin because the 0x02 pane takes the round. Then the rest of that list. Never lower damage or health. Pin through real input on 07000008 and 07000001. Run glass_pass_every_level_live, dead_pane_live, the bullet binaries, walkreal_07000001_live, walkreal_07000008_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/glass-pierce-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

