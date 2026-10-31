## Your task: `thrown-satchel-digits`, a thrown satchel's digits use the spawn matrix

origin/s/fix-satchel-digits draws the satchel countdown digits exactly per Bullet_DoTrails 0x12d890 for the 4a drone-dropped charges, but for a satchel thrown by the player and still in flight the digits follow its drawn velocity pose like the mesh does, while the game uses the spawn matrix there [M Bullet_DoTrails 0x12d890:485-553 and the satchel's flight pose]. Continue from origin/s/fix-satchel-digits (merge origin/master first), make thrown satchels' digits and mesh pose like the game in flight and at rest, render before/after headless, pin with a test (sister). Run satchel_digits_4a_live and every satchel/bullet test binary. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/thrown-satchel-digits`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

