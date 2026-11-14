## Your task: `real-health-00agent-16-ramp`, 07000016's 00 Agent chasers walk survives the corridor approach

Branch from current origin/master and read the ignore message on walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent in crates/nf-view/tests/walkreal_07000016_live.rs, plus docs/reports/real-health-00agent-16-1b-4.md on origin/s/real-health-00agent-16-1b-4, "What's left" item 2. Bond dies in the corridor approach cluster near tick 1030 at (12.12, 0.93, -88.14). The cheap levers are measured there; take the next one: a cover-shaped approach from the ring that meets the ramp-foot cluster a few at a time, or an armour pickup the level really has before the ramp. Judge each on its own real run. Helpers stay gated to 07000016 and difficulty 3, so the other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-16-ramp`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


11-13 cover map (master eb89478f5, d3, Bond held at the start (5.17, -65.30) for 600 ticks, segment_bullet_clear from each live shooter to eye height, 0.75 u grid): live shooters #0 (11.43, 6.71, -76.65), #1 (10.37, 0.80, -113.09), #2 (2.39, 6.71, -106.68), #3 (-10.51, 6.71, -111.00), #4 (-10.39, 6.71, -85.45), #5 (-8.45, 6.71, -112.24). The approach middle (z -77..-100, x -6..12) is seen by 2-6 everywhere; the EAST edge is the only low-exposure band - x ~15-17 at z -77..-80 (seen by 0-1) and x ~8-13 at z -96..-100 (seen by 0); the ramp between (z -86..-92) was not sampled (floor above the 1.2 band). The current death (t1020 at (12.23, -89.00)) is mid-ramp with #2/#5/#3 (0x1a) landing 82.5. Next: route down the east edge, measure the ramp band with the floor filter widened, and hold at the 0-spots between volleys.

CORRECTION (east-edge probe, 0.75 u grid x 10..17.5, z -76..-91, top-down floor + shooter lines): the ground lane (x 10-12, floor -0.20) is seen by 4-5 shooters, the east ramp (x 13-16, floor rising 0.0 -> 5.7 from z -90 to -80) by 4-6, the ring (5.71) by 5-6. The earlier "0-exposure" patches are off this path (south of the ramp foot at z -96..-100, and the x 15-17 / z -77..-80 cells sit under the ring). There is NO covered approach to the ramp - the lever is thinning the gallery shooters (#2/#3/#5, 0x1a), not a route.

LANDED (master a793c19fa): the d2 gallery-pocket dash now runs at 00 Agent - corridor-approach death t1020 gone; pocket entered at 72.0, its fight kills #0/#5/#2/#3 and ends at 4.7; death t4379 at (20.49, 0.91, -94.04). Next: health through the pocket fight (the gallery peek leg wedged its 3000-tick cap).

Measured 11-13 (dead levers on the pocket fight, both on a793c19fa):
- clear ground hostiles (radius 12) before each peek cycle: death t4423 vs t4379 base, no gain.
- hold the pocket and shoot the nearest visible guard instead of the peek walk: death t2236 (worse) - the
  south ground pair (#2 weapon 0x1a from (21.4,-103.2), #1 from (11.3,-111.1)) see into the pocket.
The health loss (72 -> 4.7) lands during the wedged peek walk (d 2.09 from (19.0,-92.2), 3000-tick cap).
Next: a peek spot that is reachable (or a shorter cap), not a hold.

Merged 11-13 (my-16b): the pocket fight is survived - peek (19.3,-93.6) + the close pair fought in the pocket; death t4379 -> t12054 at the pit chaser fight with a dry gun (#8 took the last rounds). Next: ammo into the pit (take the pocket drops before the back-out, or a resupply before the pit floor).

Measured 11-13 (dead ammo levers, on ced98413c): resupply from the pocket drops after the close pair - every step east of the pocket is exposed, death t4126/t3468; walking the pit floor west AR (-16.8,-167.9) / 0x15 (-15.5,-170.8) before the hand-back - neither was taken (gun unchanged 0x11 1+3) and the pit pulse killed the walking Bond, t10795. The pit-floor ARs exist (kind 1 0x1a 50 rounds at (18.8,-173.4) and (-16.8,-167.9)); check their pickup state before trying again (they may only arm at the hand-back).

Measured 11-13: at the chaser fight start Bond stands at (-14.5,-170.2) with 16.0 health; the pit AR (-16.8,-167.9, 50 rounds, state 1) and armour (-14.4,-172.7) are untaken. Walking both (return fire held) takes the AR (clip 30 + 50 carried) but the walk costs 16 -> 3.5 (#6 skin 0x5000087 0x14 at 1 u) and the armour does not apply; death t12157 (no gain). The real gap is health: 61.5 after the gallery, 26.5 after the close pair, 16 at the pit.
