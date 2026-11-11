## Your task: `guard-leaf-pushout`, guards slide off a closed door leaf as in the game instead of freezing

Branch from origin/s/guard-stuck-14-corner, merge current origin/master, and read its docs/reports/guard-stuck-14-corner.md, item 1 of "What is left". 07000014's drone #0 (GO_TO_GOAL) freezes at (50.38, 1.00, 52.34) against the closed 0x1A leaf's west face: the port's probe_move rejects the step every tick, where the game moves first and then pushes out in the contact pass [M 0x140968:204-208, Collide_Intersect 0x1ebaa8:353-373]. Read both in full with addresses and port the game's order (move, then the DeltaP push-out), keeping the Rand count. Test it red on the old code, and pin it through real play: that guard reaches his goal. This touches every level's walks: run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai binaries, npc_doors_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-leaf-pushout`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

