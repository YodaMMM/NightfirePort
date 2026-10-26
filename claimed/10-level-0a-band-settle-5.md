## Your task: `level-0a-band-settle-5`, the per-face push with its live operands

Start from `origin/s/level-0a-band-settle-4` (merge it, then origin/master) and read its decode. Its three live checks were run on the real game (PCSX2, 0700000a, god mode, Bond walking west from (-8.0, 1.12, 16.29), a breakpoint at 0x10a540 in `Intersect_CylGeom`'s radial block; 12 consecutive hits, all on ONE contact tick with Bond at (-7.8138, 1.1216, 16.4328); s1 = the HITTEST stack copy 0x1fffa30, sp 0x1fff800) [L 2026-10-26]:
1. **copy+0x50 = Dist2Tri's closest point Q** (your finding 2 confirmed): per face (-8.304, 0, 16.2788), (-8.2249, 0, 16.1597), (-8.304, 0.1622, 16.2221), (-8.304, 0.1622, 16.2788), (-7.8138, 0, 16.2788), (-7.8138, 0, 16.4328), (-7.5129, 0, 16.8199), (-6.9183, 0, 16.0449), (-8.304, 2.0206, 16.2787) x2, (-8.304, 0.8562, 15.9498), (-8.304, 0.1793, 16.1899).
2. **copy+0x60 is constant across every face of the tick: (-22.0782, 0.3832, 12.2086)** — it does NOT track pos + the pushes; it is the accumulator's transformed image (the add-back target), unchanged within a tick.
3. **96(sp), the query point, rides Bond's spine:** (-7.8138, 0.6368, 16.4328) on most faces = pos.y - 0.4848 (the B end), (-7.8138, 1.3966, 16.4328) on the two y-2.02 faces = pos.y + 0.275 (the A end), and (-7.8138, 1.0941, 16.4328) on one face = the on-edge projection onto the spine between A and B. x/z = Bond's exactly.
Port the per-face push with these operands (query = the spine point, Q = Dist2Tri's closest point, the accumulator image only in the add-back), reproduce round 3's jamb frame table, then un-ignore the three walkreal_0700000a tests and keep every other walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T22:02Z
