## Your task: `results-stats-key`, the results page Stats key

On the mission results page the game binds 0x5e (Stats) to its own key; the port has none — Extra (Q) stands in for it, and the on-screen hint's keycap reads "Del", which nothing binds (the pad meaning of actions 0x20/0x21 is open). Find the game's results-page key handling and the hint's keycap source [M the results page handler and its message ids], bind the port's keyboard and pad to the same actions, and make the hint show the key that actually works. Test: pressing the bound key on the results page opens Stats; the hint text names that key.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/results-stats-key`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

