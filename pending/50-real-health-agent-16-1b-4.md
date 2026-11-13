## Your task: `real-health-agent-16-1b-4`, 07000016 and 1b can be walked on Agent with real health

Branch from origin/s/real-health-agent-16-1b-3, merge current origin/master, and read that branch's report plus the two Agent ignore messages. Restore the report files onto the branch if the merge drops them, as the last round did. 1b is the closer one, so start there: extend Drake's r < 8 aim band across the first 500 ticks of his fight, because his measured hits landed at 9.3 u outside it, or re-score cover against the ring's live positions (cannot be seen from where the ring is, not from where it spawns). Then 16's lever from that report. Do not repeat the measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000016_live and walkreal_0700001b_live binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-16-1b-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000016_live: Bond died at tick 1020 (frame 1020) at (12.23,0.94,-89.00) armour 0.0
walkreal_07000016_live: Bond died at tick 14833 (frame 14833) at (-7.56,-22.28,-172.78) armour 0.0
walkreal_07000016_live: Bond died at tick 2176 (frame 2176) at (-0.19,7.96,-79.44) armour 0.0
walkreal_0700001b_live: Bond died at tick 11385 (frame 11385) at (46.01,-14.08,-3.00) armour 0.0
walkreal_0700001b_live: Bond died at tick 5619 (frame 5619) at (31.92,-1.68,-0.59) armour 0.0
- Main agent, named: the t1020 death is walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent - the upper-ring guards (y 6.7: #5/#2 skin 0x50000b2 and #3 0x500008c, weapon 0x1a) land 7.0 a hit from 22-28 u onto the floor-0 approach (the level_fights field doc's ring ambush); t14833 is the Operative real-damage walk (#14 at 1.6 u, weapon 0x14, the pit); t2176 the Agent walk.

---
Update 2026-11-13T17:05Z (master c94c1043e): the 16 Agent walk now dies at tick 2176 at (-0.19, 7.96, -79.44), 38 hits for 100.0 - #5 (skin 0x50000b2, 0x1a, at (-9.5, 6.7, -91.4), 15 u) 43.3 in 11 and #3 (0x500008c, 0x1a, at (-4.7, 6.7, -110.5), 31 u) 31.5 in 15; five return-fire fights, none closed. The 1b Agent walk is still red too.

---
Update 2026-11-13T17:11Z (master 40f33b3d3): the 1b AGENT walk latches all 8 consoles and ends the console run at 2.3 health, then dies at the waves-wait first tick (frame 11385, (46.0, -14.1, -3.0)) to a world hit of 2.32 - at the instant every missile reaches deploy state 2 (likely the launch blast), not a hurt volume (hurt-avoidance in hide_and_snipe for d2 changed nothing). Budget: 28 hits for 100 - the world 27.7 in 5 (13.84 at frame 8502), astronauts #3 17.5, #7 14.0, #4 10.5, #11 10.5 (0x6a). Levers: a waiting spot away from the silos when the deploy completes; the frame-8502 world hit during the console run.

Also: the frame-8502 world hit (13.84 on Bond) is one blast centred (20.69, -3.89, -0.80) that also hits a dozen astronauts (#7 10.4, #13 11.3, #8 8.4 ...); find its source (NF_HEALTH_TRACE prints blast Some(..) with no owner) - a console or prop blowing up, or a grenade - and keep Bond out of its radius.

Both 1b Agent world hits are the SAME blast kind (a spawn_blast log on master): weapon type 0x6b, radius 20, damage 20, owner usize::MAX-1 (not a drone), script 0x60007c4 - f8502 at (20.69, -3.89, -0.80) (debris 0x6) and the fatal f11385 at (46.07, -13.81, -2.76), i.e. AT Bond's standing spot (46.0, -14.1, -3.0) after he held it for the 1316-tick missile wait. Identify what spawns them (script 0x60007c4) and either keep Bond moving through the waits or out of its aim.

The 0x6b blasts are explosive BOLT rounds (weapon_data+0x70 bolt bits -> script 0x60007c4, Bullet_CollisionHandler 0x12ce80:205-224) fired by a shooter with NO drone behind it (drone.rs blast_at maps b.owner == usize::MAX to usize::MAX-1) - a turret or scripted gun aimed at Bond's position. Lever: no long stationary holds in its line (keep the waits moving or out of its sight).

Update 2026-11-13T17:18Z: master 1a0270f44 shifts cover in the 1b Agent waits; the walk now reaches Drake (2.3 hp) and fails at the Drake fight - Bond never fires (no clear shot from the held cover). Start there.

Update 2026-11-13T17:21Z: the 1b AGENT walk is GREEN on master 95a45f308 (main agent). Only the 16 Agent walk is left in this task.

16 Agent detail (master 95a45f308): the loss is the ramp-foot return-fire fights at (12.8, 0.93, -90.35): #4 (t530), #0 (t874, 95.8 -> 74.8), #1 (t1313, -> 26.5), #2 (t1435, -> 18.1) - while those run, the gallery riflemen #5 (-9.5, 6.7, -91.4) and #3 (-4.7, 6.7, -110.5) fire into Bond; he dies on the ring west leg at t2176. Lever: fight the ramp-foot four from a stand out of the gallery line (the lane south / door[7] side), then take the ramp.

Tried (reverted): the d1 route tactics on d2 (finish_fights + back_off 3.5 + hurting_first + level_fights) - WORSE, death t1571 at (13.62, 6.81, -78.60).

Tried (reverted): answer the exposed gallery rifleman (#3, 32.4 u) from the lane before the ramp foot (shoot_bounded 1200) - WORSE, death t1258 at (7.75, 0.84, -84.60); the ground four close in during the long-range trade.

Tried (reverted): a d2 hold in the door[7] corridor before the lane, answering visible ground guards (y < 2.5, < 25 u) - none came (quiet 400 ticks at 100 health): the ramp-foot four hold their posts; the walk then died on the ramp at t1882 (14.37, 3.37, -84.92). Three measured levers now closed for 16 (d1 tactics, gallery-first, door hold); the fight has to be taken TO the ramp-foot four from a stand the gallery cannot see.

COVER MAP (Agent, t600, bullet rays from eye height to the three gallery riflemen #3 (-10.5, 6.7, -111.0), #4 (-10.4, 6.7, -85.5), #5 (-8.4, 6.7, -112.2)): the ramp foot and the whole ramp (x 13-16, z -81..-91) see ALL THREE; the only all-blocked pockets near the route are x 14-18 / z -80..-83 (the ramp top east) and x 19-20 / z -92..-98. There are no ground guards at the ramp foot at t600 - the "ramp-foot fights" were the gallery trio. So the ramp exposure is unavoidable; the trio has to be fought: from the x 19-20 / z -92..-98 pocket (blocked to all three - peek out and back), or from the ramp-top east pocket after a fast climb.
