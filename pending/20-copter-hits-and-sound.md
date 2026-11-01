## Your task: `copter-hits-and-sound`, the helicopter dies to one bullet and its sound is too present behind Bond

On the Phoenix Fire levels after 07000011 the attack helicopter is destroyed by a single bullet, and its sound behind Bond is too clear. Find the copter's health and the per-round damage it takes [M the copter class's damage/health path; D its placement/stats], and its 3-D sound falloff and rear attenuation [M Sound_Play3D and the copter's loop], port both with addresses, and pin: rounds-to-kill for the P2K and a rifle, and the rear volume vs front at equal distance. Run the copter binaries and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/copter-hits-and-sound`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

