## Your task: `drone-bump-collision-8`, the 07 east door: the measured push between Bond and the door guards

Continue `origin/s/drone-bump-collision-7` (merge `origin/master`). Your -7 live checks are answered [L PCSX2 fresh boot 07000007, 2026-10-28, audits/live-07-eastdoor.txt, Bond teleported in steps toward the E-spot, 120 frames per step]:
- The door guards do NOT relocate: idle 30 s, d19 (93.74, 44.17, -48.04) and d10 (94.79, 44.17, -48.07) stay within 0.1 of their boots; with Bond 0.5 away d10 shifts to (94.59, -47.88) (0.3).
- The push exists: placed at (94.00, -48.78) Bond settled at (94.08, -48.99).
- Bond STANDS on the E-spot (94.6, 44.4, -47.4) - 0.48 from d10's centre - and stays there 120 frames. The port pins him at the 0.95 r-sum contour.
- Placed at (94.40, -47.86) (between/at the guards) the game moved Bond to (94.34, 44.39, -46.65): out NORTH past the doorway, not back into the hall.
So the port's Bond-vs-door-guard push is wrong at close range: the effective contact radius is well under 0.95 for these guards (their capsule radius, the anim height word, Jointy's resolution direction, or whether the 0x31 guards are armed at all) [M Collide_Jointy 0x1ec180:203-281, NDrone2_Collision 0x140e90:66-101, the radius source]. Port the game's numbers, then turn green probe_07_route, party_door_lock_07000007_live (x2), cutscene_timer_pause_live and walkreal_07000007_live's hall walk. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T16:56Z
