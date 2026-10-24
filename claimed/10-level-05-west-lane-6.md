## Your task: `level-05-west-lane-6`, 07000005: the band gap, then the notch grab, the ride and the exit

On master: the west lane, the crawl #299 and the slab crossing walk green (real input); the creepwall half-length is the world-sphere radius x scale at the world-sphere centre [M Control_BuildWorldSph 0x133070]. Still ignored: `the_cable_legs_the_gap_the_notch_and_the_ride_parked` and the notch sister.

The band gap: the band's west pieces (x -19.75..-15.75 at z 46-47.5, including the (-24..-23, 45-47) cable-ledge step [D]) are separated from the diagonal's west end by a void/2-cell gap (x -16..-9.5, z 45.5-48). The walked crossing drops ~3.5 onto a y-29.6 roof whose south-west run descends to 22.6 and dead-ends [L port walk].

1. Find the game's link across the gap: a thin or one-way surface the down-ray misses (check every tri in the gap's column, including back-facing and pass-flagged ones), a jump distance the game allows [M the player's jump arc], or another route. If only a live read can settle it, name it (fresh boot, walk the band's west end westward, watch the player position) and stop there.
2. Then the notch grab at icon #310 (-23.80, 32.74, 12.33), the ride, the south ledge, the window-ledge crawl #35 (-34.10, 37.28, -31.65) and the exit door #112. The old round-3 legs (git history 96cebefd1) arrived by XZ only — re-verify heights.
3. Un-ignore both tests.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-24T02:06Z
