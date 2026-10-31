## Your task: `level-11-route-teleports-to-walks`, replace 07000011's walk stand-in teleports with real input

`walkreal_07000011_route_live.rs` still moves Bond with three `bond.pos` writes (around lines 51, 65, 74). For each: read its comment (why it was a stand-in), find the game's own route there [D collision/placements/nav; M any class the route uses: doors, lifts, ladders, wires, movers], walk it on real input, and delete the write. Where the data shows no route, keep the write but pin the gap with a sister assert and say exactly what live check would settle it. Test-setup placements whose subject is not the route (a measurement spot) may stay — say which and why. Every test in the file green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-route-teleports-to-walks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T00:03Z
