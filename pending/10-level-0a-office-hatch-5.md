## Your task: `level-0a-office-hatch-5`, 0700000a: the spawn floor the game's ray finds at -0.0109

The game spawns Bond at root 1.0891 = its floor ray's -0.0109 + 1.1 [L ee_start snapshot; M Player_StandAtNewPosition 0x190d88]; the port spawns at 1.1000 (its ray finds 0.0000). The branch `s/level-0a-office-hatch-4` (not merged; its hatch4_probe1-4 dumps are there to reuse) searched every STATIC placed tri and the stored plane records around the spawn column (-3.93, 10.40): only 0.0000 faces (map37 obj458 Room-M_lifts tri36/tri253, obj0 Player_1 tri2), and obj458's planes store d = 0.

1. The search has not covered the MOVING set at spawn time: the lift cab is a scripted SS object (the cab placement #1 at (-3.9026, 0.69766, 10.9626) [D]) whose mesh is posed by its script/keys at load. Pose it as the game does on the spawn tick [M SS_Create 0x1a9c58 / SS_Update 0x1a9948 + the stream keys] and test the vertical ray against it: does a cab floor face sit at -0.0109?
2. Check `build_PointOnFloor` 0x1c95a8's query flags (0x70c?) and the HITTEST+0x98 mask against the port's floor query: a node class the port skips could carry the surface.
3. If neither finds it, report exactly; the live read (the HITDATA of the spawn ray: hit point, cel, object) is queued on the game side.
4. When the spawn matches 1.0891, re-run the hall glide (game 1.1147) and the corner, and un-ignore the three sisters and the main walk in `walkreal_0700000a_live`.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

