## Your task: `mouse-look-sensitivity`, mouse look got much faster between releases

Mouse look became much more sensitive between v0.1.23 and the build after it (same settings file). Find the cause: diff everything that feeds the look rate between tags v0.1.23 and v0.1.24 and master — the input path (crates/nf-view: mouse delta handling, raw input, DPI/scale, the per-frame vs per-tick accumulation, the interpolation/render-rate changes that could multiply deltas at high frame rates such as 240 Hz), the settings load/defaults/migration (settings.rs, the controls/sensitivity options and their front-end rows), and the player's look integration (nf-game player.rs). Reproduce it with a test that feeds the same mouse deltas through the app path at 60 and 240 frames per second on both versions and compares the resulting yaw/pitch, fix the regression, and pin it (sister: the old buggy path fails). Run every *input*/*mouse*/*settings*/*look* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mouse-look-sensitivity`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

