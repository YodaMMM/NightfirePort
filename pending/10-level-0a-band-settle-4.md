## Your task: `level-0a-band-settle-4`, 0700000a's jamb slide, with its writer named

Start from `origin/s/level-0a-band-settle-3` (merged to master) and read its report. Its two live checks were run on the real game (PCSX2, 0700000a through the game's loader, god mode, yaw -1.5708 west, stick forward from (-8.0, 1.12, 16.29)) [L 2026-10-26]:
1. **The writers of HITTEST+0x10 (BL+0x750) through the jamb pass:** a write watchpoint caught only `Collide_Intersect` 0x1ebaa8 + 0x644 (PC 0x1ec0ec, 32 hits = the DeltaP add-back) and `Vec_Zero` (0x1e2d60/64/68, the per-tick clear). No other writer — the slide comes out of the DeltaP that `Intersect_CylGeom` 0x109da0 accumulates per face, so the tangential part must be in that per-face push.
2. **The per-frame values** (frame: pos | +0x10 | +0x50): f2 (-7.77607, 1.12437, 16.43305) | (0.01938, 0, 0.00566) | (0.24493, 0.04521, 0.14305); f3 (-7.77937, ., 16.44391) | (0.0345, 0, 0.01086); f4 (-7.79265, ., 16.48133) | (0.05402, 0, 0.0214); f6 (-7.80424, ., 16.50845) | (0.059, 0, 0.02712); f7 (-7.90382, ., 16.54156) | (0.06088, 0, 0.03312); f8 (-7.93165, ., 16.58067) | (0.05955, 0, 0.03911); f9 (-7.87678, ., 16.62515) | (0.05487, 0, 0.04449); f10 (-7.92087, ., 16.67337) | (0.04681, 0, 0.04822); f11 (-8.07472, ., 16.72227) | (0.02329, 0, 0.04506); f13 (-8.13789, ., 16.80307) | (0.01132, 0, 0.03574); f14 (-8.2344, ., 16.82433) | (0.00271, 0, 0.02126); f15 on: 0 (free walk, z settles 16.82875 -> 16.82952). +0x50 accumulates exactly the pushes (0.22555,0.04521,0.13739 -> 0.73368,0.04521,0.53961). y stays 1.12437.
So the per-face push carries a growing +z component (up to 0.048/tick) while the x part peaks at 0.061: port `Intersect_CylGeom`'s per-face DeltaP exactly (0x109da0:300-460, the edge/vertex regions included) so the port's pushes reproduce this frame table, then un-ignore the three walkreal_0700000a tests and keep every other walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

