## Your task: `level-0a-band-settle-3`, 0700000a's walk: the HITTEST+0x50 / +0x10 live values

Start from `origin/s/level-0a-band-settle-2` (merge it, then origin/master) and read its report. Its two live reads were run on the real game (PCSX2, player obj 0x1b78640, BL 0x1b78750, god mode, stick forward, yaw -1.5708 = west) [L 2026-10-26]:
1. **HITTEST+0x50 (BL+0x790) is never reseeded on a walk.** Fresh level load at the spawn: (0, 0, 0), +0x10 (0, 0, 0). Placed at (-8.0, 1.12, 16.29) and walked west: sliding round the doorway jamb it grew (0.22555, 0.04521, 0.13739) -> (0.66049, 0.04521, 0.3834) -> (0.73368, 0.04521, 0.53961) by frame 20 (z 16.83), then stayed EXACTLY constant for the whole free walk down the hall (frames 20-60+, x -8.93 -> -17.1). After a second level load (the game's ResetMap reload) it started at (-0.06453, 0.00522, -23.21946) — it carries over a reload.
2. **Pinned at the west hall wall (x -45.856/-45.956, z 16.8288, y 1.11671):** HITTEST+0x10 (BL+0x750) = **(0.1, 0, 0)** on each contact tick (0.01056 on the first touch, 0 on a non-contact tick between), and +0x50 grows by exactly that every contact tick: 0.27723 -> 0.47722 -> 0.67721 -> ... -> 3.27709 over 30 ticks (x only; y 0.08088 and z -22.77985 unchanged). Bond's x alternates -45.856 / -45.956 (the 0.1 step in, the 0.1 push out).
So the push equals the walk step along the wall normal, and +0x50 is a running sum of every push (the add-back), not a per-tick reference. Port the ref exactly this way, then land the radial-dir rule, drop the NF_WALK_KILL gate if the kill becomes unnecessary (or fix its 01 regression), un-ignore the three walkreal_0700000a tests with their [L] pins, and keep every other walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-26T18:05Z

---
## Report (done, 2026-10-26T19:18Z)

