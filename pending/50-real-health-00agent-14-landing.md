## Your task: `real-health-00agent-14-landing`, Countdown's 00 Agent infiltration walk passes the 1.72 landing

Branch from origin/master. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent wedges (not a death): the '1.72 landing' resupply leg floats Bond 0.41 over the floor at (33.84, 0.82, 30.95) at tick 26340 and the leg cap panics. A floating Bond is a collision question first: find why the port holds him 0.41 above that floor [M the player floor snap] and fix it if the port is wrong (test red on the old code); otherwise take the ignore's own lever (a re-ground before the leg). Real input only, never lower damage or health. Coordinate with 50-real-health-14-00agent-10 if it is claimed. Run walkreal_07000014_live and interactables_07000014_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-14-landing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000014_live: Bond died at tick 4104 (frame 4104) at (31.42,2.35,33.43) armour 0.0

Merged 11-13 (my-14d): the switch band guards are answered from node 626 (three bounded stands, nearest inside 15 u); death t23511 -> t26213 at (-45.31,1.07,27.50) on the escape-touch legs: #20 15.0 in 3 at node 2, #29 7.9 at 3.7 u, health 27.9 -> 10.4 across nodes 0-3. Next: answer #20/#21/#29 from the Kiko corridor before those legs.

Merged 11-13 (42aff3fcb): the Kiko corridor squad answered while exposed (nearest first, 300 each); death t26213 -> t27134 at node 4 of the escape-touch legs, health 27.9 -> 7.9 by node 2. Next: a second stand at node 0.

Measured 11-13 (on 42aff3fcb): a second exposed-only stand at node 0 (-49.3,36.3) t27071 (22.9 left after it), at node 1 (-49.4,33.2) t26823 - no gain; the squad on nodes 2-4 outlasts 27.9 health.

Measured 11-13 (97c74df39): the #14/#15 room behind the node-42 door costs 30.0 (72.9 -> 42.9, t11150-11401). A nav stop + stand at (-21.4,-21.1) keeps 60.4 past it but the walk re-rolls: the later timeline reaches the Kiko corridor DRY (clip 1 spare 0) and dies at the same escape-touch legs (t41666, #20 25.0 in 9); at (-22.5,-20.0) t29274. Not shipped. The Kiko stretch needs ammo as well as health.

Measured 11-13: a no-stop push (return fire off) through the escape-touch legs dies sooner, t26418 at node 3. Not shipped.

Merged 11-13 (d0b707fbf): the Kiko corridor end is held 600 ticks (short stands + grenade side-step); six of the squad die on the legs; death t27517 at (-35.17,26.39), 2 u from the escape touch, to #31 (0x5000090, 0x14 at (-28.8,27.0)) with 7.9 health. Next: #31 before the last leg.

Merged 11-13 (51fc82754): a second 300-tick hold at escape-touch node 4 (-44.2,27.6); the escape touch is reached; death t29197 at (-30.20,24.28) on the Rooks-wake legs to #30 (0x5000087, 0x14) with 2.9 health. Next: health into the Kiko corridor (27.9 there).

Measured 11-13 (51fc82754): an extra hold-and-dodge at node 626 after the band stands (300/600) dies sooner at the door (t25726/t26118). Not shipped.

Measured 11-13 (51fc82754): a hold at the escape touch (300/600) dies there at 2.9 (t29452/t29378); hold-length sweep (Kiko/second): 600/150 t27685, 600/450 t29335, 450/300 t29157 (12.9 after the second hold, same death area), 900/300 t26998. No stable gain; the shipped 600/300 stays. The 16 pattern that finished its squad (short 120-tick stands on close hostiles inside the slice loop) is the next thing to try on the Rooks-wake legs.

Measured 11-13: the Rook leg split into its 33 nav nodes with a 120-tick close answer (visible, radius 8/12/18) before each node - byte-identical death t29197 at node 1 (nobody visible; 2.9 health). The gap is health before the escape touch.

Measured 11-13 (NF_DMG_TRACE): between the Kiko hold (27.9) and the second hold (7.9), #20 (0x5000090, 0x14) hidden at (-51.2,32.6) lands 15.0 in 3 at 2.4 u as Bond passes node 1 (t26993-27001), #23 2.5, #22 2.5. A 300/600-tick stand on #20 from node 0 (-49.8,36.1) or node 1 (-49.0,33.4) dies inside the stand at t26998. Next: a line on #20 from the Kiko hold spot side, or crossing node 1 faster.

Measured 11-13: crossing to the second hold with return fire held dies sooner, t26923. Not shipped.

Measured 11-13: a bullet-line scan at the Kiko hold end finds #20 (at (-51.33,32.42)) in clear line from the whole x -53..-46, z 30..37 band, including the hold spot (-51.33,37.87) at 5.5 u; a direct shoot_bounded on him from there (300/900) loses the duel - death t26998 at the hold spot with 27.9 going in. Open for a worker: the levers tried tonight are all in this file.

Measured 11-13: a hold-and-dodge in the switch band before the door west face (300/900) bleeds to 7.9/2.9 and dies at the door (t25847/t26050). Not shipped.

Measured 11-13 (c9b24271d): the node-42 door stand (60.4 kept) re-rolls the later stream: a clear_hostiles return leg ("back along the clears trail nav 1/2") wedges two 6000-tick caps at (-58.27,35.49) and Bond reaches the Kiko hold DRY (0x11 clip 0 spare 0, no armed gun to switch to) - death t41862. If the door stand is used, the clear return leg needs fixing first (nav to a reachable node, or lenient).

Measured 11-13: skipping the clears trail return at d3 (nav straight home) - alone t26476 (Kiko hold ends at 15.6); with the node-42 door stand (77.9 at the door) t26461 near the Kiko corridor. Both worse than the shipped t29197; the late stream re-rolls with any upstream change.
