## Your task: `level-1b-real-health`, Equinox without the health top-up

`walkreal_0700001b_live::walk_arrival_to_drakes_defeat` finishes the mission on real input but keeps the walker's per-tick health top-up; without it Bond dies around the fly-by to the later astronaut waves (origin/s/fix-1b-walk — read the walk and its notes). Make a real-health variant pass: play like a player (auto-aim on as the app defaults, cover, ammo management, the console-shooting order) and check the astronauts' accuracy/damage against the game [M the drone fire path, difficulty scaling] — fix port gaps with evidence. Keep the existing walk green; put the real-health walk in its own test. Continue from origin/s/fix-1b-walk (merge origin/master first). Run walkreal_0700001b_live, interactables_0700001b_live and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-real-health`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

