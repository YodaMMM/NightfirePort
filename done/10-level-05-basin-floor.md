## Your task: `level-05-basin-floor`, The Exchange's valley basin has its floor east of the lip

Branch from origin/master (after s/level-05-valley-hurt lands, read docs/reports/level-05-valley-hurt.md; until then read it on that branch). Measured there: past the lip at about (132.0, 14.17, 125.37) every point resolves find_cel = room 14, and room 14's collision build covers the east basin but has NO floor quads there, so Bond cannot walk down into the valley the game lets him enter; the west basin floor is room 15 and works. The basin is hurt #3's band (x 80.78..168.38, z 96.66..144.68, y ~5.1). Find why the port's room-14 build drops that floor: compare 07000005.bin's collision data for the room [D] against the port's parse, and the game's FindCel relink [M build_FindCel 0x1c8cb0] against the port's ScopeGraph. Fix the build, with a test that fails on the old code (a down ray in the basin hits the floor). Then un-ignore walkreal_07000005_live::the_valley_plane_hurts_bond_walked if the walk reaches the plane. Run walkreal_07000005_live, interactables_07000005_live, and every walkreal_*_live binary if the shared collision build changes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T09:06Z

---
## Report (done, 2026-11-10T10:07Z)

Closed by the driver: the report is on `s/level-05-basin-floor` (docs/reports/level-05-basin-floor.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-10T11:01Z)

