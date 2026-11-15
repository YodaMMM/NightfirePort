## Your task: `real-health-00agent-14-landing-3`, 07000014's 00 Agent infiltration walk reaches the delta exit

Branch from current origin/master.

Master: death t29197 at (-30.20, 1.16, 24.28) after the escape touch. The bleed: the south-loop corner pair #14/#15 (35.0 in 40 ticks at 1.0 u, t11354), the Kiko hold (ends at 27.9) and the second hold (27.9 -> 7.9); the killing blows are 0x14 explosive rounds (-17.9/-22.9 blasts, one from an already-dead guard's round in flight). Measured worse this round: the armours at (-41.3, 36.5) and (-27.3, 31.0) by every route (the routes cross the squad), a node-42 corner fight (saves 12.5 but the corridor re-rolls into the blast), dodging 0x14 rounds in the holds, hold lengths 450/750, skipping the second hold. Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-14-landing-3`.
