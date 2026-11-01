## Your task: `laser-watch-reach`, the laser watch's reach and the satchel contact

In the port each laser-watch round reaches 4.0 units and the satchel's contact is a 0.5 sphere (a stand-in for the game's mesh check), so lasering a landed satchel from more than about 4.5 units does nothing. Find the game's real reach and contact [M Player_LaserPointer 0x1a65d8, the laser watch's weapon record (range) and bullet class, Bullet_DoTrails 0x12d890's satchel hit test (BU+0x102 tail :452-474) — sphere or mesh, what radius], port it with the addresses, and pin: the burn counts down from the game's maximum distance and not beyond it (sister). Also check the space laser's `spawn_impact(0x2d6210, …)` call in drone.rs against `Emitter_Create` 0x17ae00's argument order (def, pos, normal) — 0x2d6210 looks like the normal and the effect id should be 0xc000010; fix and pin it. Run the satchel/laser binaries, interactables_0700004a_live, space_laser_1b_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/laser-watch-reach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-01T06:46Z

---
## Report (done, 2026-11-01T07:14Z)

Closed by the driver: the report is on `s/laser-watch-reach` (docs/reports/laser-watch-reach.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-01T07:15Z)

