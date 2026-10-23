## Your task: `find-opponent-exact`, NDrone2_FindOpponent's team-2 scan: the exact alive filter and the per-frame LOS budget

The port has the team-2 opponent pick (`Drones::team2_opponent_picks` in drone.rs) with two approximations:

- **The alive filter.** The game's candidate test is the +0xE0 record's +0x4F8 flags (0x600 clear, 0x100 set), health +0xAC > 0, class +0xFF != 0x11, and the +0xFE & 1 death bit [M 0x1424e0:210-226, and the held-opponent re-check :157-166]. The port uses present && enabled && !corpse. Port the exact test.
- **The LOS budget.** The scan stops on `uVar1 < DAT_00299092` [M 0x1424e0:259-262], a per-frame CanSeeObject counter shared by all drones. The port uses one LOS per drone per tick. Find what DAT_00299092 is, where it is reset and incremented (CanSeeObject 0x?), and port the shared budget.

Tests: an ally skips a guard with 0x600 set, and one at health 0; sister: a live guard in range with LOS is picked. Budget: with N allies scanning in one frame, only the game's count of LOS probes run (cite the number). Keep walkreal_07000008_live's ally tests green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/find-opponent-exact`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

