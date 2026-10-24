## Your task: `level-0a-office-hatch-9`, 0700000a: the hall glide, the corner, then the office walk to the hatch

Master now has the level-start spawn settle (the 1.0891 rest [L ee_start_0700000a]). Still ignored in `walkreal_0700000a_live`: `the_west_hall_glide_height_matches_the_game` (the walker wedges at the doorway base line, bond (-9.15, 1.09, 15.92); the game glides at 1.1147 after the doorway-base climb at x -8.304 [L fresh boot]), `the_corner_crosses_like_the_game` (the port dithers at (-17.87, 1.17, 4.87); the game walks through at y 1.127 -> 1.120 [L]) and `walk_office_floor_qworm_keyboard_hatch` (wedges at the ai lane into the N-S corridor, the 0.5-wide mouth).

1. Dump the doorway base's triangles at x -8.5..-8.0 and the corner's at (-17.87, 4.87); log the port's per-tick capsule contacts and pushes there; compare with the game's rules — the slope-blind capsule push added every tick [M `Intersect_CylGeom` 0x109da0, `Player_CollisionHandler` 0x1949a8 disasm 0x194c60-0x194c70], the feet band [M `Player_FeetOnPoint` 0x194188], the head-on cull [M 0x109da0:302-322]. Name the step that differs and fix it [M].
2. Un-ignore the glide and the corner as they go green, then take the main walk through the corridor mouth to the hatch, un-ignoring it when it walks.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


