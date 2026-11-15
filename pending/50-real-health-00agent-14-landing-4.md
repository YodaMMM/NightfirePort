## Your task: `real-health-00agent-14-landing-4`, 07000014's 00 Agent infiltration walk reaches the delta exit

Branch from current origin/master.

Master: death t29197 at (-30.20, 1.16, 24.28) to #30 (0x5000087, 0x14) with 2.9 health after the escape touch. Read the landing-3 report and the done/ task notes for 14 (landing-2/-3): the corner pair #14/#15 (35.0 in 40 ticks at 1.0 u; a node-42 stop kills both at 50.4 health but the corridor re-rolls into a dead guard's in-flight 0x14 blast), the Kiko hold (ends 27.9) and the second hold (27.9 -> 7.9), the middle-lane crossing closed by #29's sightline; measured worse: the armours (-41.3, 36.5) / (-27.3, 31.0) by every route, 0x14-round dodges in the holds, hold lengths 450/750, no second hold. Idea not yet measured: a dodge of the in-flight 0x14 rounds only outside the holds' fights (cf. the 1b laser_dodge that yields when a hostile is close). Goal: the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-14-landing-4`.

Main 11-15 (merged ad2d99954 - master now carries the named idea): walker_b2 round_dodge (step perpendicular to an incoming 0x14 round whose line passes within 3 u, 1.5-20 u out, unless a hostile is within 5 u): the Kiko hold ends at 52.5 (was 27.9), death t29197 -> t31711 at (-45.68, 1.07, 26.88) by the second hold. Rebase on master before measuring.

Main 11-15 (merged 9f5b13b6e): with the round dodge the second hold is dropped - death t31711 -> t35208 at (-13.09, 1.12, -23.10) past the escape touch on the Rook wake legs. Rebase on master.
