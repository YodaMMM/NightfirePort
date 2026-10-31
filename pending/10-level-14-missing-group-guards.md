## Your task: `level-14-missing-group-guards`, 14: three of the release group's guards never exist

Continue `origin/s/level-14-door-guard-release` (merge `origin/master` first; read its report). It found the port spawns six guards on the ch 0x87+0x88 release where group 0xd has seven templates (#721 #722 #723 #724 #783 #784 #785, all p4 0xd, wake 0x8C) plus the group-9 pair — #723 (-32.78, 28.48), #784 (-36.18, 27.76) and #785 (-31.74, 25.16) are absent from `drones.list` at load. FIRST rule out the game's own create gate: `Drone_Create` 0x136be0 makes nothing for a placement whose level tag (parameter 2) exceeds the load's difficulty (GameState+0x40) — the port mirrors that gate (ghost-guards-990) — so read those three placements' parameter 2 [D] and the difficulty the test loads at. If they are difficulty-gated, the port is right at that difficulty: pin it (present at the higher difficulty, absent at the lower) and say so. Otherwise find where the parse/attach drops them (`Drones::attach_level`, the placement row parse) and fix it with the address cited. Then pin the release in `door_guards_locked_door_07000014_live` (the guard count and states per difficulty; the ambush trio stays 0x38 [M 0x161560]). Run door_guards_locked_door_07000014_live, every *07000014* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-missing-group-guards`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

