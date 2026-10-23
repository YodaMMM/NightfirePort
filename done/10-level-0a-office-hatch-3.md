## Your task: `level-0a-office-hatch-3`, the 0700000a west-hall glide height and the corner wedge

Start from `s/level-0a-office-hatch-2` (merge it into a fresh branch from master): its walk reaches ring leg 6 and wedges at (-17.87, 1.18, 4.87) heading to (-17.50, 2.50), where the port's y dithers 1.1336 / 1.2019 across the band edge (1.1347), unhooks the ground band and never clears the corner.

Live measurements [L fresh boot, PCSX2, god mode]:
- Walking out of the cab: lobby y 1.0891; at x -7.75 the game climbs (peak 1.18) and then holds **y 1.1147** steady along the west hall to (-11.39, 16.29). The port glides at 1.1336 there.
- From (-17.87, 4.87), holding forward toward (-17.5, 2.5): the game walks straight through to (-17.78, 1.120, 2.92) in about 25 frames, y 1.127 then 1.120. No dither, no wedge.

1. Find why the port's glide height is 0.019 above the game's: the step-up at x -7.75 and the settle after it (`Player_FeetOnPoint` 0x194188's band, the push tail of `Intersect_CylGeom` 0x109da0, the handler's landing at `Player_CollisionHandler` 0x1949a8:136-140). Cite the line that differs and fix it.
2. Pin the measured heights: a real-path test from the cab that reads y in the west hall (1.1147 within 0.003) with a sister, and the corner crossing.
3. Un-ignore `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` and run the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-office-hatch-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox2 at 2026-10-23T21:54Z

---
## Report (done, 2026-10-23T23:12Z)

Closed by the driver: the report is on `s/level-0a-office-hatch-3` (docs/reports/level-0a-office-hatch-3.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-23T23:12Z)


---
## Report from the worker (done, 2026-10-23T23:12Z)

