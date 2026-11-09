## Your task: `guard-standing-capsule-2`, a firing guard has the game's standing body on every level's walk

Branch from origin/s/guard-standing-capsule, merge current origin/master, and read the report in the queue's blocked/10-guard-standing-capsule.md. The body now follows the game's rule [M `NDrone2_Collision` 0x140e90:91-121; `NDrone2_DoCollision` 0x149ec8; `DroneFunc_NearPlayerCollision` 0x14b190], and the change is global, so two walks moved. Take its residuals in order. First, walkreal_07000001's `following_mayhew_down_completes_the_level`: the contact dies at t15517 to guard #15's burst after the push shifts Bond's follow spots. Repair the walk's spots or fight order, gated to 07000001, without hacking the contact's rules. Second, walkreal_0700000a's `west_doorway_repro`: the mission fails during the ring walk at ai node 34 south. Find what fired and at which spot, and repair it gated to 0700000a. Then run every other walkreal_*_live one level per tool call, building with -j2, plus npc_doors_live and fix_guard_misc_live, and repair any walk that moves, gated to its level. Real input only, no top-ups, never lower damage or health. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-standing-capsule-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-09T03:36Z
