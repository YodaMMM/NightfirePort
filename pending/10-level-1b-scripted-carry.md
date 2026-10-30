## Your task: `level-1b-scripted-carry`, how the game carries Bond past the 1b boundary wall

`walkreal_0700001b_live::walk_arrival_to_drakes_defeat` is ignored: the west wall plc #128 OBJECT_Boundary_1 (x 4.433) is body-solid [M], so the game's crossing must be scripted. The candidate is the SP#211 chain (0x060007fd -> 0x060008f0): it pulses ch 75 (the float entry) and its tail pulses ch 74 (0x600096e, t392-399 [D]). Decode those scripts fully [D the streams; M the script player/event handlers 0x1d94c0, 0x1d8548]: find any player-position write, attach or teleport event, and which entity the player follows. Port what is missing, then make the walk cross on real input (the script doing the carry) and un-ignore it. Every test in the file green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-1b-scripted-carry`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

