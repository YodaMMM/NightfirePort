## retune-nb-08 (2026-11-18, s/retune-nb-08)

Task: commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt
attack from the fire states; the 08 whole-mission walks
`walk_the_whole_mission_on_real_input_real_damage_agent` and `_00agent` now die. Read both
ignores, find where the new fights cost the health, and change the route, stance or fight
order there. Helpers stay gated to the level and the difficulty. Real input only, no top-ups,
never lower damage or health, never change guard code. Judge each lever on its own real run.
Remove each ignore when green; if still red, push the measured ledger and the next lever into
the ignore message. Run the whole walkreal_07000008_live binary.

### retune-nb-08 progress

- Reproduced both ignored deaths [L runs, this branch]: Agent d2 t31483 at (112.72,13.49,-177.24),
  4 rockets landed (401.8 of 448.5), dead at the 5th launch — the ledger: 59 machine drips
  (-1.20/-1.40) for 72.6 world damage + the killing blast, and the upstream guard cost only
  ~15.4 (drone #2/#9/#5/#10). 00 Agent d3 t28298 at (112.25,13.09,-179.16): 32.2 in 38 world
  hits (gun chips -2.00 x5 + the -7.78/-8.22 missile blasts), ONE rocket launched (t1269), the
  upstream y-46 hall trio (#14/#11/#2) + secure-room #5 cost 48.0.
- Measured the root shift: the nb-scan-all upstream walk is ~6500 ticks faster (the old green
  fight started ~t35500; now ~t25700-28900 [L the runs — rockets 1/2 identical t49/t229/t275/t428
  across all runs]), so the machine's route phase differs and its passes line up with the
  doorway lane on d2 and fly HIGH overhead (y 25-29) on d3.
- Lever 1 (both sisters, difficulty >= 2): the dodge SIDE bias and the drift walk-back open from
  d3 to >= 2 (the d<2 else branches unchanged — the d1 real-damage Operative timeline byte-exact
  by construction; its test passed in the whole-binary run).
- Lever 2 (Agent, difficulty 2): the fight opens CROUCHED at the slit anchor (110.98, -179.31);
  the movement inputs stand Bond back up into the lintel-window launch geometry (the green run's
  launches leave from the standing eye y 14.22). GREEN: five rockets, 502.5 damage vs hp 448.5,
  best single 101.0, end health 79.0 armour 24.1, 14 hits for 21.0, mission 31070 ticks — the
  ignore REMOVED.
- Levers tried and retired on d3 (each on its own real run, the code reverted): the crouched
  slit anchor (t29087, ZERO launches — the crouched eye's sight stops on the lintel t0.106,
  clip 4 intact), the standing slit + the aim gates relaxed to 0.25 (t27383). The d3 fight code
  is back to its original measured state (death t28298 reproduces exactly [L the whole-binary
  run]); the full ledger + the next lever live in its ignore message.
- Whole walkreal_07000008_live: 6 passed / 0 failed / 1 ignored (the d3 ledger), 153.6 s.
  Only this test file and docs/ changed; no crates/*/src/ and no tests/common/ files touched.
