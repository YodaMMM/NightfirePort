## Your task: `level-05-west-lane-4`, 07000005: from the west lane to the cable and the exit

On master the west lane walks green (`walkreal_07000005_live`, real input). `walk_the_west_ledge_and_cross_the_cable` stays ignored at two unfound approaches: the tower-top west parapet (x ~26.5-27.2, z 58-73; the walker stalls near x 27.22, y 37.1) and the drop from the lane plateau (y ~34-36) onto the creepwall shelf at y ~30.9 around (25, 71). The creepwall #299 "Wall Hug Short" (25.79, 31.78, 67.76) half-length is the object's own bound half-span along local x, the port's stand-in for the game's +0x8c [S].

Live [L fresh boot]: standing on the shelf at (25.51, 32.11, 71.02), one action press enters the wall-hug view of #299 (third-person, back to the wall, the crawl icon). The entry works; only the route onto the shelf is open.

1. Map the parapet and the shelf at 0.25-unit resolution (x 24-27, z 58-76) and find where a real player gets from the plateau onto the shelf (a walk-off, a step, a gap).
2. Then the crawl through #299 with the real use press, the slab-end to cable-ledge step (cells around (-24..-23, 45-47) [D]), the notch grab at icon #310 (-23.80, 32.74, 12.33), the ride, the south ledge, the window-ledge crawl #35 and the exit leg.
3. Settle the creepwall half-length/axis from `control_create_object`'s +0x8c write [M] if it can be read; otherwise keep the [S] mark.
4. Un-ignore the test. If a live read is needed, name it (fresh boot, what to watch) and stop there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

