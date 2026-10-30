## Your task: `level-0a-corner-push-model`, the 0a corner with the push model the game actually uses

A fresh-boot live measurement settles the walk wall model: Bond placed at (-15, 1.2, 16.3) on 0700000a, stick forward 300 frames at four yaws — the game rests at z 15.5904 against the hall wall at yaw pi, the SAME value as the port's current push model on master, and the paths at yaw 0 / pi/2 / -pi/2 match master's to ~0.1 [L 2026-10-30]. The move-clip (NF_WALK_CLIP) is NOT the game's model (on master it walks through walls); do not flip to it. Yet `walkreal_0700000a_live::the_corner_crosses_like_the_game` shows the port needing 870 ticks (with NF_WALK_KILL) where the game crosses in ~60 frames [L 2026-10-26]. With the push model kept, find what differs at that corner [D the corner's faces; M Intersect_CylGeom 0x109da0, Collide_Intersect 0x1ebaa8, the player push / slide] — e.g. a face the port keeps or culls differently, or the multi-face combination — fix it, and un-ignore the test without the kill switch. Every test in the file green; run every walkreal_* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-corner-push-model`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T18:38Z
