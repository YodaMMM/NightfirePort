## Your task: `level-0a-office-hatch-4`, 0700000a: the spawn floor, the hall height and the corner

Three ignored tests in `walkreal_0700000a_live` pin the live heights [L fresh boot]: the spawn 1.0891 (the port spawns at 1.1000), the west-hall glide 1.1147 after the doorway-base climb, and the corner at (-17.87, 4.87) walked through at y 1.127 -> 1.120. The main walk `walk_office_floor_qworm_keyboard_hatch` stays ignored.

Known: both sides spawn at the floor ray + 1.1 [M Player_StandAtNewPosition 0x190d88: build_FindCel from pos+0.1, build_PointOnFloor's (0,-1,0)*3.0 ray through Collide_RayIntersect, y += 1.1]. The game's cel-scoped ray finds -0.0109 under the cab; the port's flat ray finds 0.0000 (the cab's own floor at -0.0109 is not in the port's sets at the spawn; the cab placement #1 sits at (-3.9026, 0.69766, 10.9626) [D]). The hall: the 0.1622-tall wall base at x -8.304, z 14.3..16.3 lifts the capsule; the game settles at 1.1147 (1.1147 - 1.1000 = 0.0147 = 0.0256 climb settle - 0.0109 spawn gap).

1. Make the spawn ray find the game's surface (register the cab floor in the spawn cel's chain, or port build_FindCel's chain pick for the spawn ray) [M].
2. Make the walk take the doorway line over the wall base and settle at 1.1147.
3. Un-ignore the three sisters, then the corner and the main walk. The corner may still need `Player_CollisionHandler` 0x1949a8:136-140's push landing [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

