## Your task: `findcel-tie-break`, objects on 07000002 are placed in the cel the game picks

Branch from origin/s/markers-world-objects, merge current origin/master, and read its docs/reports/markers-world-objects.md, item 3 of "What's left". 07000002 has 13 class-0xD8 placements that overlap more than one cel, and the port does not port `build_FindCel`'s collision-ray tie-break that decides which cel each one belongs to. Read the tie-break in full, with addresses, and port it. Print the 13 placements with the cel the port picks before and after and the game's rule, and check every other level for multi-cel placements [D]. Pin through real play on 07000002: those objects collide, draw-cull and wake with the cel the game picks. Run the worldobj binaries, walkreal_07000002_live, interactables_07000002_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/findcel-tie-break`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

