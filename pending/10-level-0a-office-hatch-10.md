## Your task: `level-0a-office-hatch-10`, the capsule push's overlap gate, then the 0a hall, corner and office walk

Read `docs/reports/level-0a-office-hatch-9.md` on `origin/s/level-0a-office-hatch-9`. It named the step that differs: the game's capsule push does not fire when the capsule AXIS already overlaps a wall plane ([L fresh boot] the game glides the z 16.29 line with its capsule 0.011 inside the z = 16.279 plane, no shove), while the port's ungated contact fires +0.539/tick there. The decompile gates the push on `0 < dot(hitpoint - HITTEST+0x50, plane)` [M `Intersect_CylGeom` 0x109da0:422-431]; implementing it with B = the spine's bottom (`NF_CAPS_GATE=1` on that branch, off by default) makes the corner cross but misses the exit-height pin both ways. The open point is the reference vector HITTEST+0x50 (BL+0x790): never written per tick in the decompile — only moved by the DeltaP adds [M `Collide_Intersect` 0x1ebaa8:353-373], zeroed nowhere per tick (`Collide_Update` 0x1ea398:50 zeroes only +0x10).

1. Settle what HITTEST+0x50 holds [M — its boot write (Player_Init / the hittest init), every writer, and what the DeltaP adds accumulate], port the gate with that exact reference, and pin it: the glide holds 1.0891 -> peak 1.18 -> settle 1.1147 and the corner exits at 1.127 -> 1.120 (+-0.01) [L].
2. Make sure nothing else regresses: the gate touches every capsule contact, so run the full suite; every walkreal test green on master must stay green.
3. Un-ignore `the_west_hall_glide_height_matches_the_game`, `the_corner_crosses_like_the_game` and then `walk_office_floor_qworm_keyboard_hatch` as they go green; remove the env switch once the gate is the default.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


