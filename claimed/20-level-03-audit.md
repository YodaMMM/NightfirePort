## Your task: `level-03-audit`, the 07000003 behaviour audit: every placed object and character against the game

The walk test for 07000003 passes, but real play still finds missing behaviour (characters walking through Bond, guests that never talk, doors that never lock, consoles whose action does nothing, cut-scenes with broken cameras). This task lists every gap on the level.

1. For EVERY placement on 07000003's main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts, special objects): name its creator in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, its update and its use/damage/event handlers [M], and what the port does with it (created? updated? drawn? usable? reacts to Bond/guards/damage? plays its sounds?). Do the same for the level's scripts (every stream's ops, the camera streams, the hand-backs) and the mission records (objectives, the channels each writes, fail conditions).
2. Write the table to `docs/levels/07000003-audit.md`: one row per placement/script/record, the game's behaviour [M/D], the port's, and a GAP column.
3. Fix the small gaps directly [M] with a test each. For each large gap, write a section in the report (what, where, the [M] chain) so it can become its own task. Draw-only gaps: name the mesh/hash/matrix.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-audit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-25T22:26Z
