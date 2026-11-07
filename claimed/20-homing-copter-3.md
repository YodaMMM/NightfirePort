## Your task: `homing-copter-3`, homing rounds lock on to the copter

Branch from origin/s/homing-copter-2, merge current origin/master, and read its docs/reports/homing-copter-2.md. The lock test stays ignored because the arm's own line of sight blocks on the copter's own mesh. The game's `Collide_LineOfSight` carries the ray's subject objects (`Bullet_homing` 0x12f358:92-93 passes the bullet and the candidate), so a ray cannot report the candidate's own collision. The port's `segment_bullet_clear` has no exclusion, and the copter's script entities pose into `set_dynamic`. Build the fix the report names: a bullet-clear in world.rs that excludes by shape hash (the dynamic set's `last_objects` are index-aligned with `geo.objects`), fed the candidate copter's stream hashes (the `hidden_stream_objects` precedent in nis.rs), and call it from the homing's line of sight. Check the guard candidates too, because the same rule applies to them, and the copter's own spot check [M 0x17dbb8:171-172]. Remove the ignore. Pin through real fire: a homing round from the shelf locks and converges on the flying body, and a wall between them still stops the lock. Run the homing binaries, the copter binaries, walkreal_07000008_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/homing-copter-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-07T15:29Z
