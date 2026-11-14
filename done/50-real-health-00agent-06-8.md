## Your task: `real-health-00agent-06-8`, 07000006's 00 Agent castle-interior real-damage walk reaches the party

Branch from current origin/master and read the ignore message on walk_the_castle_interior_to_the_party_real_damage_00agent in crates/nf-view/tests/walkreal_07000006_live.rs, plus docs/reports/real-health-00agent-06-7.md on origin/s/real-health-00agent-06-7 (not merged; both of its named levers measured dead, so don't repeat them). The killers are the ch-55 pair (#19/#20) charging the landing from inside their sight and tunnel-roof guard #11 holding the mouth-strip band. Try levers that change the engagement: a route or timing that meets the pair one at a time, cover against #11's lane, or taking #11 first from a spot outside his band. Judge each on its own real run. Helpers stay gated to 07000006 and difficulty 3, so the d1/d2 walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000006_live binary and interactables_07000006_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-06-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


11-13 (master dcf899951, de2695a97): (1) the landing-lane 20 u clear is exposed-only at 00 - the ch-55 pair then falls to the mouth stands (#19 +90, #20 +18) and the walk reaches the courtyard (tick 6255); (2) a live bullet-line scan from the courtyard bottom gives each east post one solo spot - #24 from (10.8, -34.2) (taken, dead at +19), #25 from (25.2, -45.4). Death now t7076 on the south-east corner leg (#23 deck 0x24 34.0 in 2, #25 30.2). Measured dead: #25 from the #24 spot (roof #11 lands 44.0, t6866); walking back down to reach #25 via the south edge (the spot is in #11 lane, 42.2, t6851). Next: #11 first from outside his band, or the corner leg in one push.

Roof #11 first (live scan: his one solo spot from the courtyard bottom is (6.8, -32.6), 19 u, 13.5 u below him): measured dead - the aimed stand duel loses, #11 lands 58.8 in 11 while the taps take him 20 -> 16 (t6982).
