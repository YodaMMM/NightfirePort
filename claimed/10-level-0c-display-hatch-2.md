## Your task: `level-0c-display-hatch-2`, 0700000c: the route into the display case, with the live answers

Start from `origin/s/level-0c-display-hatch` (merge it, then origin/master) and read its blocked report. Its live checks were run on the real game (PCSX2, 0700000c loaded through the game's own loader, god mode, stick forward) [L 2026-10-26]:
1. **Placed at (5.0, 16.4, -110.5) facing south (yaw 3.1416):** the game settled Bond at (5.3328, 15.7852, -111.0017) — inside the bay, not on the walkway — and he FELL: frame 10 (5.3947, 14.2643, -111.3391) with **ch 27 (0x1b) = 1** for that sample (the skylight touchplate fired as he dropped past ~14.1), frame 20 onward resting at **(5.42, 12.8773, -111.38)**, ch 27 back to 0. So the bay floor reads 12.877 in the game (the report's stand map said 11.84), and the touchplate pulses on a body passing through its volume.
2. **Placed on the roof at (5.75, 17.6, -110.6), walking south:** y 17.6 -> 17.4666 (z -112.62) -> 17.3904 (-113.58) -> 17.1399 (-114.38) -> 16.7153 (-115.18) -> 16.1164 (-115.98) -> 15.6536 from z -116.78 on (to -120.53). Ch 27 stayed 0 the whole walk: walking on the roof does NOT fire the skylight.
3. (the SS #179 hinge-burn blocker watch: not run.)
4. **Placed on the mid floor at (3.0, 11.4, -114.0) walking north (yaw 0):** settled (3.0948, 11.4248, -113.54), then held at z -112.83/-112.93 for 120 frames (dithering 0.1) — the game does NOT let Bond drop off the mid floor's north edge into the 6.27 pocket.
Use these to find the real route into the case (the touchplate fires when Bond's body passes through its volume — e.g. dropping into the bay from the walkway side), port whatever the port lacks, and un-ignore the two tests with their legs re-derived from these numbers.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-display-hatch-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T16:52Z
