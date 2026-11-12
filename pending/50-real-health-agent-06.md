## Your task: `real-health-agent-06`, The Exchange castle interior can be walked to the party on Agent with real health

Branch from origin/master. walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_agent is ignored: the pass ended at 3.7 hp; the balcony drop raises ch 0x34 five ticks sooner, the reveal and the courtyard guards shift, and the walker runs dry at the mouth (clip 0, spare 0) and dies to the strip post #25 at tick 17089. Restock before the mouth (the level's ammo pickups [D]) and take #25 from cover. Real input only, never lower damage or health, no top-ups; helpers gated to 07000006 and difficulty 2. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000006_live and interactables_07000006_live.

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

---
claimed-by: main agent at 2026-11-12T10:22:46Z

---
## Main agent's run (2026-11-12, master bb5e63e94 - after the castle chat pair + held-sight look_for_opponent landed in 88051e74f; released back)
The death moved EARLIER than this task's text: the walk now dies at tick 15832 at the mouth (45 hits; #11 skin 0x500002f at (-0.6, 55.5, -51.1) from d 23.4, then #20 at (11.2, 42.0, -35.5) from d 6.4), arriving at 38.4 health. The big loss is the GUARDHOUSE COURT: 94.2 -> 92.3 vs #4 (t2380), then 90.4 -> 49.9 vs #2 (t3236, killed false) - the court trio #2/#3/#4 now engage on held sight (the gate's per-tick rule). Re-derive the court leg first (the Operative rook06 walk passes this court; its approach and kill_438 are the template), then the mouth (restock before it, take #20 before he closes).

---
claimed-by: main agent at 2026-11-12T10:40:01Z
- Main agent, second pass (levers measured, all reverted): the court loss is the walk_to("the guardhouse court") leg's REACTIVE return fire, not fight_through: Bond stands at (33.4, 39.1, -7.9) from t2375 to t3369 while the guardroom pair #3 (35.6, 41.0, -15.8) and #4 (32.9, 40.0, -14.8) fire out of the guardroom (d 7-8.6, 1.93 a hit) and #2 closes to 3.5 u; the reactive fights on #3/#4 end "killed false" (NF_DMG_TRACE shows 29 hits, 94.2 -> 38.4). (a) stand_shooters_down at the hedge end: no shooter yet, no change; (b) clear_the_spot(8.0, exposed_only) in place of fight_through: no change (the loss is earlier, inside walk_to); (c) aimed_fights + switch_dry + back_off 3.0 on the court leg: WORSE (dies t4832). Next idea: a court approach line out of the guardroom openings' view (the pair's rays) or a straight push to door #47 and the doorway pre-clear, rather than standing at the court point.
- Main agent, LOS probe: from the court point (33.4, 39.1, -7.9) to #3 (35.6, 41.0, -15.8), #4 (32.9, 40.0, -14.8) and #2 (31.5, 39.0, -10.8), world.segment and segment_bullet_clear are CLEAR both ways at every height pair (0.6-1.5 / 0-0.9) - no shooting through walls; the court is open to the guardroom. Bond's 'killed false' fights are the walker's fight logic (aim / pick / hold), not blocked lines: check why the reactive fight on #3 (2161) ends with zero damage dealt.
