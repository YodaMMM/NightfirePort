## Your task: `guard-capsule-radii-2`, guards' bodies collide exactly as the game's do in every state

Branch from origin/s/guard-capsule-radii, merge current origin/master, and read its docs/reports/guard-capsule-radii.md. Take "What's left" in order. Land the real-input down-bump proof that is still an ignored test: 07000006's start guards, or corner a fleeing guard. Then the two unmodelled arms: the dead 0x200 arm's bone-based volume, and the +0x570 & 2 anim-driven no-collide bit. Read each in full with addresses and port it in the game's order, keeping the Rand count, and pin each through real play. Then the walkreal binaries the report did not run, one level per tool call, repairing any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries (grep tests for NDrone2_Collision) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-capsule-radii-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

