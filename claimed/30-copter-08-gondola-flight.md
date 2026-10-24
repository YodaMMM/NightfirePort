## Your task: `copter-08-gondola-flight`, the 07000008 gondola helicopter flies its attack, not a hover

Symptom: on 07000008, the helicopter that attacks the gondola flies in, then holds one spot in the air, and the mission completes. The port's copter hit walk (Copter_Update 0x17dbb8:277-320) and the gondola completion on real shots are on master; the flight itself is the open part.

1. Trace the port: from the ride's start, log the attacking copter's position, its route node, `Copter_Update`'s case and its target each tick until the kill. Find where it stops moving (a route with no further nodes, a keyless script stream holding its pose, a case that never advances).
2. Port the game's flight from the decompile: `Copter_Move` 0x17d628 (the path follow), `Copter_Update` 0x17dbb8 (the case machine: approach, attack runs, strafes, the hit reactions), the stream/route data it reads [M + D]. Cite each case's exit.
3. Tests, real input from the ride: the copter's position keeps changing between attack passes (a moving-window distance check per N ticks, with the numbers from the decompile's speeds), it dies on real shots, and the mission completes only after the kill. Sister: an unshot copter never completes the mission.
4. Name the PCSX2 read that would confirm the flight (fresh boot, the copter object's +0x30 over time during the ride).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/copter-08-gondola-flight`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T04:44Z
