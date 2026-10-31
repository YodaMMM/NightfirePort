## Your task: `enemy-accuracy`, enemies hit as often and as hard as the game's

Continue `origin/s/wip-fix-enemy-accuracy` (merge `origin/master` first). Several walkthroughs keep Bond alive with a per-tick health top-up (0700001b's astronaut waves, 07000002's fights). Check the enemy fire path against the decompile: drone aim/accuracy and its difficulty scaling (GameState+0x28 consumers), reaction and burst timing, spread, hit tests on Bond's hitbox, damage per weapon and body part, armour absorption [M Player pain path 0x1902c8 and the drone fire states; D weapon data]. Fix divergences with citations; pin hits-per-10-s and damage-per-hit at 5/10/20 u per difficulty (sister: old numbers fail); report which walkthroughs can drop their top-up (try walkreal_0700001b_live::walk_arrival_to_drakes_defeat without it). Run nf-game, every *drone*/*combat*/*damage*/*weapon* binary, walkreal_0700001b_live, walkreal_07000002_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/enemy-accuracy`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

