## Your task: `level-07-rook-doorway-2`, Rook still freezes in the 07 west doorway: the clamp's wall probe

Start from `origin/s/level-07-rook-doorway` (merge it, then origin/master; its report has the chain: the undrawn raw slide [M Drone_Control 0x139268:75-96], the boundary clamp [M :76-83], DroneMove_SetBoundryFlags 0x14f140 ported with MoveTest = AINetwork_BoundsTest [M 0x155330:86-108]). Its target test `the_mission_walker_completes_his_doorway_leg` is ignored: Rook (drone 22, Path_3) still freezes in the west doorway (79.53, 44.0, -50.32) because the port's one-cel wall flattening probes walls the game's per-cel BLINK lists would not.

1. Port the game's per-cel bounds test exactly [M AINetwork_BoundsTest and the cel BLINK lists it walks, the cel the drone stands in] so the probe sees only the walls the game's does; Rook crosses the doorway in real play.
2. Un-ignore the test; npc_doors_live (the 01 shoji) and the 01/07 walks stay green; run the full suite on the merged tree; report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-rook-doorway-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T19:37Z

---
## Report (done, 2026-10-25T20:42Z)

Closed by the driver: the report is on `s/level-07-rook-doorway-2` (docs/reports/level-07-rook-doorway-2.md); the worker didn't run finish.sh.
