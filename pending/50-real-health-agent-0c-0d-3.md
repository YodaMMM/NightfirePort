## Your task: `real-health-agent-0c-0d-3`, 0700000c and 0700000d can be walked on Agent with real health

Branch from origin/s/real-health-agent-0c-0d-2, merge current origin/master, and read that branch's docs/reports/real-health-agent-0c-0d-2.md plus the ignore messages on `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage_agent` and `walk_arrival_to_the_c5_door_real_damage_agent`. Take its levers in order: the pitch-convergence fix in the aimed stand for 0d (one shared-helper change, opt-in, default off), then the early band sweep again, then 0c's #30 stand east of the hall edge or #31's fire-window pre-emption. Do not repeat its measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000c_live and walkreal_0700000d_live binaries, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-0c-0d-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_0700000c_live: Bond died at tick 6148 (frame 6150) at (54.02,6.01,-157.98) armour 0.0
walkreal_0700000d_live: Bond died at tick 3749 (frame 3749) at (213.07,17.37,-234.51) armour 0.0
walkreal_0700000d_live: Bond died at tick 5020 (frame 5020) at (213.07,17.37,-234.51) armour 0.0

---
Update 2026-11-13T17:05Z: the 0c walk (walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage_agent) is green on master c94c1043e and un-ignored; only the 0d walk (walk_arrival_to_the_c5_door_real_damage_agent) is left.

---
Update 2026-11-13T17:06Z (master c94c1043e): walk_arrival_to_the_c5_door_real_damage_agent dies at tick 3868 at the ladder #435 top (213.07, 17.37, -234.51), 6 hits for 100.0, zero return-fire fights: the west-roof snipers #32/#33 (0x1e, at (151.4, 22.7, -252.8)/(-255.8), 65 u) land 28-damage rounds during the 863-tick ladder climb (frames 2680-3868), plus deck #20 28.0 at frame 3168. They are not present at the pre-wake roof checks (line ~150) and the ladder-top sweep runs only after the climb. Lever: answer them before the climb (from the ladder base) or climb out of their line.

---
Update 2026-11-13T17:07Z: the west-roof pair is NOT present at the ladder base either (a capped base shot found no guard near (151.4, -249..-256)) - they appear DURING the climb, so a pre-climb answer cannot work; the lever is the climb itself (its timing vs their wake, or a route that is not on their line).

Correction 2026-11-13T17:24Z: the west-roof pair #32/#33 WAKE at the platform drop (t1452, Bond (178.1, 8.0, -207.8)) and settle at x ~151, z -241..-248 (a temporary present/state watch); at the ladder base #33 IS present at (151.35, 22.70, -245.52) state 0x93, but a 300-tick base answer at d 63 with the carried gun (clip 2 spare 30) does nothing (health 10 left; the second line blocked 0.970 at the roof edge). So the base answer fails on reach, not presence: the climb needs cover from their line, or a long gun for the 63 u shot.

At the ladder base Bond OWNS the sniper rifle 0x1e (owned 0x1 0x2 0x1e 0x4a 0x4e 0x50 0x54; two more 0x1e lie on the route at (194.4, 12.7, -232.1) and (161.4, 5.8, -234.5)). The lever: a SCOPED 0x1e shot at #32/#33 (x ~151, z -241..-248, d 63) from the ladder base before the climb - the far-box helper fired the held gun (clip 2 spare 30) unscoped and landed nothing in 300 ticks.

Tried (reverted): the held 0x1e at the ladder base through the far-box helper (unscoped, 400 ticks x2) - the aim never converged (crosshair offset ~(-0.29, 0.33)) and #33 stepped to z -244 behind the roof edge (ray blocked 0.970). The base has a brief line only; a scoped stand (aiming + zoom) or cover on the climb is what is left.

Tried (reverted): the 0x1e drawn + zoom held through shoot_far_aim at the ladder base - the crosshair pins at the aim box edge (cross [-0.296, 0.324]) and the view never turns onto #33 (the helper's pinned-edge turn does not engage), and #33's line is then blocked 0.970 at the roof edge. A base shot needs Bond turned to face the roof before aiming (a plain turn/look to the target, then the aim), not the cursor walk.

Tried (reverted): face_toward the roof + a pitch walk, then the scoped far-box act - the crosshair still pins at x -0.296 and #33 is behind the roof edge within ~120 ticks (blocked 0.980). CONCLUSION: the ladder base has no lasting line on the west-roof pair; drop the base-shot idea - the climb needs cover from their line (or a different route up).

The climb is game-rate: Player_Climb moves +0x14 by fwd*0.02 per tick = 1.2 u/s [M 0x196310; player.rs climb_tick], so the ~16 u ladder is ~800 ticks of exposure whatever the input - no cover exists on it. The remaining idea: take the pair right after their wake (t1452, triggered as Bond drops off the platform at (178.1, 8.0, -207.8)) while they walk to x ~151 (they sit at (145.8, 22.7, -237.4)/(145.7, 22.7, -247.3) at the wake, ~44 u off), before they reach the edge-covered spots.

ROOT CAUSE of every failed far shot here (5 measured attempts 11-13, all reverted): shoot_far_aim enters aim mode and steers the CURSOR, and the cursor pins at the aim box edges (cross ~(-0.29, +0.39), the +-0.296/+-0.4 box) while the view does not turn - pitch stays at its entry value (0.231 -> 0.243 over 400 ticks). A target outside the box at entry is never reached. The wake-window try had the line CLEAR at d 33 (t+147) and still never fired a landing round (clip 3 -> 2). Fix the helper first: pre-aim the VIEW (yaw + pitch to the target with plain turn/look, aiming off) until the target sits inside the box, then enter aim mode - or turn on the pinned-edge rule. Then retry the wake window (d ~33, lines clear in stretches).

Pre-aim measured (reverted): turning the view onto the target first (aim off; bond.pitch/look are in pi/2 units - a radians pre-aim overshoots) brings the cursor in (cross ~(0.07, 0.00)) and Bond fires (clip 3 -> 0), but at d ~33 no round lands on #32 in 2 x 400 ticks while the pair's return fire takes Bond 94 -> 38. The wake-window trade loses at Agent. Every gun answer to the west-roof pair is now measured and closed; what is left is NOT being shot: a route that skips ladder #435 or a timing that climbs before the pair reaches its spots (they arrive x ~151 by ~t1989; the climb starts ~t2900).
