## Your task: `satchel-laser-scramble`, lasering the satchel: sparks and scrambled digits

On 0700004a, cutting a planted satchel with the laser watch shows no laser particle effect at the contact point, and the countdown digits never scramble; in the game the digits go jumbled after the laser has been on the satchel for a short time. Find the satchel's laser-hit path [M the laser beam's hit handler, the satchel's digit state, the spark/particle spawn] and the scramble timing, port both with addresses, and pin: sparks spawn at the contact while the beam is on it, the digits scramble after the game's time and the defuse completes. Run satchel binaries, interactables_0700004a_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/satchel-laser-scramble`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

