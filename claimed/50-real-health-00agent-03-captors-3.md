## Your task: `real-health-00agent-03-captors-3`, 07000003's 00 Agent real-damage walk completes every objective

Branch from origin/master. walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent is ignored; the last round worked the #5 doorway wake ambush - its ledger and next levers are in the ignore message. Run walkreal_07000003_live and interactables_07000003_live. Real input only, no top-ups, never lower damage or health; every lever gated to the level, its difficulty and !w.tops_up(). Remove the ignore when green, otherwise push the ledger and the next lever into it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-03-captors-3`.

---
claimed-by: sandbox1 at 2026-11-13T11:19Z

11-13 measured on master 893dc3252: death t9585 at (11.55, 6.29, -15.74), one 40.0 from the east-wing sniper #6 (d 22.3). Bond reaches the lane mouth line at exactly 40.0: the mid-courtyard "through the doorway" fights cost 92.8 -> 40.0 (#9 killed false, #8 twice, #11). At the mouth line stand_shooters_down_03 engages nobody (no exposed shooter hit Bond in the last 600 ticks); holding Bond on the mouth line between stands changed nothing. Lever: the mid-courtyard doorway trade (the 52.8 drain), not the mouth.

Measured dead (11-13): an aimed pre-clear from the mid-courtyard doorway line (lane side, after the doors open). It kills #9 cleanly (+0 damage) but the stand time on the lane side feeds the lane shooters #11 (machine gun, 56.8 in 5) and #10 (30.6 in 17): death t8720 (t8472 when the pick also took the upper-row #8 at y 9.1). The quick crossing stays the better trade; the 52.8 drain needs a crossing that does not linger on either side, or the lane shooters dead first.

Live solo-line scan at the mid-courtyard doors (11-13, Bond (10.66, -21.21)): 8 enabled hostiles within 30 u (#6, #7, #8 upper row, #9, #10, #11, #12, #17); only #6 (12.0, -19.75) and #9 (8.0, -29.25) have one solo spot each - the courtyard/lane is mutually visible, no position answer. Levers left: timing or fewer awake guards (the wake order).
