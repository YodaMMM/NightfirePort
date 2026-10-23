## Your task: `gondola-windows-08`, the 07000008 gondola's windows must shoot out

Symptom: on 07000008 the gondola's windows cannot be shot out.

1. Identify the gondola's window objects [D placements / the carriage's script entities]: are they class-0x20 breakables (Break_Create, `crates/nf-game/src/breakable.rs`), script-created 0x20 entities riding the carriage's script, or parts of the carriage model? What kills them in the game, and what plays [M Break_Update 0x12bfd8, Break_Kill 0x12b970, Script_CreateEntity 0x1d8968]?
2. Note that the port's breakables are static (the dead pane stays solid, task 983), and task 959b re-lands the weapon-word bullet filter (`filter | 0x106`). A window riding a moving carriage needs its bounds to move with it. Find what the port is missing: the entity is not created as a breakable, its bounds don't follow the carriage, or the bullet pass never tests it.
3. Real-path test: on the ride, Bond fires at a window and it breaks (dead bit, kill sound, pieces). Sister: a miss breaks nothing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gondola-windows-08`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox2 at 2026-10-23T14:22Z

---
## Report (done, 2026-10-23T16:38Z)


---
## Report from the worker (done, 2026-10-23T16:38Z)

Closed by the driver: the report is on `s/995-gondola-windows-08` (docs/reports/995-gondola-windows-08.md); the worker didn't run finish.sh.
