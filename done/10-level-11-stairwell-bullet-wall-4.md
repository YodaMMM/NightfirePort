## Your task: `level-11-stairwell-bullet-wall-4`, 07000011: the remaining invisible walls in the stairwell

Start from `origin/s/level-11-stairwell-bullet-wall-3` (merge it, then origin/master). Its pass rule [M Collide_Update 0x1ea398:216-229, the 0x40 surface/glist pass bit] removed some of the invisible bullet walls but not all. Real play on that build, Bond (22.40, 30.09, 1.42, yaw -1.505, pitch -0.035) frame 2944: firing into the well, the bullet holes hang IN MID-AIR and trace flat planes — one diagonal plane under the stair flight, one upright plane across the middle of the well. Those decals sit on collision surfaces that draw nothing.

1. From that spot, cast rays across the decal planes and list every surface the port's bullet trace stops on: owner object/placement, glist flags (incl. 0x10 render-only), surface byte, collision class, and whether the object has any DRAWN geometry [D]. For each, find why the game's round passes it [M — the remaining pass arms: HITDATA+0x50 bit 0x40, the 0x80 random roll, collision classes the bullet's mask excludes (HITTEST+0x94 node class 0x101/0x201, +0x9a surface mask), objects not in the bullet pick at all (Collide_PickObj 0x1eae80 class/mask filters)]. Fix the port.
2. Real-input test from that spot: rounds fired across the well reach the far wall / the rope guards with no mid-air contact; a sister into a real wall still stops. Full suite on the merged tree; report the total line. Name headless renders of the spot (decals) before/after in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-stairwell-bullet-wall-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-25T18:51Z

---
## Report (done, 2026-10-25T20:09Z)

Closed by the driver: the report is on `s/level-11-stairwell-bullet-wall-4` (docs/reports/level-11-stairwell-bullet-wall-4.md); the worker didn't run finish.sh.
