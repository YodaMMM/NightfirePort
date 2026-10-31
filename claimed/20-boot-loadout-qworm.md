## Your task: `boot-loadout-qworm`, the level-start gadgets match the disc

level-0b-teleports-to-walks found the disc's level load owns the Q-Worm (`owned[0x58]`, load=1) on 0700000b while the port's boot loadout misses it. Find how the game builds the level-start inventory [M the loadout/inventory init the level load runs; D each level's loadout rows], fix the port so every on-foot level starts with the disc's gadgets and weapons (Q-Worm included where the data grants it), and pin it per level with a test that reads the disc rows (and a sister that fails on a missing gadget). Run nf-game, every *loadout*/*inventory*/*gadget* binary and walkreal_0700000b_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/boot-loadout-qworm`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-31T12:00Z
