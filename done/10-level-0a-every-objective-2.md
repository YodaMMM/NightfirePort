## Your task: `level-0a-every-objective-2`, Tower B office floor: past the ring mouth to the hatch

Continue `level-0a-every-objective` (on master; read its report). `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` is ignored at the ring's mouth: after the static wall at x -10.70 (z -16..-13.1), the walk stops at (-12.06, -13.65) with every static ray clear - a dynamic blocker (door #540 office_b_door's leaf). Find what the game does there [M Door_* / the leaf collision (the port's kind-0xDB doors carry no leaf collision; the game picks doors through the ForcedList [M build_LinkToRoom 0x1c9680:63-66, Collide_Pick 0x1eb320:369-375]); L if needed], fix the port, walk the ring legs east and the hatch E on real input, and un-ignore the test. Also the Q-Worm cubicle collision blocker (objective 4's stand-in): find why every eye ray to the PC is blocked in the port and fix it [M/D]. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-29T15:17Z

---
## Report (done, 2026-10-29T16:46Z)

