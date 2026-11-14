## Your task: `real-health-00agent-08-bleed-5`, 07000008's 00 Agent whole-mission real-damage walk wins the Hind fight

Branch from current origin/master; if origin/s/real-health-00agent-08-bleed-4 is not on master, merge it in first. Read docs/reports/real-health-00agent-08-bleed-4.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Take the ride's rocket-ammo pickups the measured roll left untaken and the drain-class lever the ledger names; keep every new block gated to difficulty 3 (an ungated form regressed the d1 sister last round). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary and interactables_07000008_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-08-bleed-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-14T14:18Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): on the merged bleed-4 stream the Hind fight (t0 at frame 20137, health 24, armour 50 from the secure-room + ledge armours) dies at t20428 to "world" -2.0 hits every ~6 ticks from frame 20332 (the trace flags 0x1C 0x27 0x28 0x06), Bond at (111.17,13.52,-179.81) = the bleed-4 ANCHOR; line_ok false the whole fight, ZERO rocket launches. Moving the start spot / the ANCHOR to (110.98,-179.31) or (110.6,-179.0): the same -2.0 cadence, deaths t20379-t20530, still 0 launches. Open question first: is this -2.0 the terminal volume or the Hind gun attributed to the world (it starts ~190 ticks into the fight); the old master stream (t34277) launched rocket 1 at t531 from (110.98,-179.31).

Main 11-14 answer to the open question: the -2.0 / 6-tick hits ARE the Hind gun (copter.rs fire_gun_tick fires every 6th call, the unit test fire_gun_fires_every_sixth_call; the round is attributed to the world in the damage log). On this stream the Hind sits SOUTH of the gondola (z -193..-208) while the firing slit is the cabin WEST doorway: the launch line from the anchor is blocked by the cabin wall at t 0.11-0.72 (the hind t300/t400 scans), so line_ok never opens while the gun has Bond. Next: a fight spot that sees south (the doorway edge), or wait the gun out in cover until the circuit brings the Hind west.

Main 11-14: a cover-anchor lever (on each gun hit, re-anchor to the nearest cabin point whose chest-to-Hind segment_bullet_scoped is blocked) picks Bond s own spot (111.1,-180.0)/(111.1,-179.6) as blocked while the gun keeps landing there - the Hind gun rounds do not obey that world trace (check fire_gun_tick s round path / collision scope before using sight lines for cover). Death t20415, 0 launches. Not shipped.

Main 11-14: the Hind gun is legit (rounds via the west doorway, see done/20-level-08-hind-gun-through-cabin). An eye-only launch line (instead of the eye + both gun spots) never opens either (0 line_ok samples): the Hind stays above the lintel line for the whole ~290 ticks Bond lasts. Anchors (109.5/109.0,-179.4) are not reachable (Bond stays at x ~110.95 - he is AT the doorway already). The fight needs either more health into it (24 + 50 armour now) or a way to stay out of the doorway s downward line until the circuit brings the Hind low.

Main 11-14 (6ffd3f07d merged: crouched deep anchor, death t26671, rocket 1 landed 100.7): the eye-only launch line from the deep anchor is byte-identical (1 launch) - the binding gate is slit_clear / the aim gates, not the 3-spot line. Bond dies to the Hind missile blasts (-7.13 at 25838, -10.58 at 26552).

Main 11-14 (6ffd3f07d): the fire-gate census from the deep crouched anchor (every 50 ticks, t>300): yaw/pitch/range pass almost always; machine_lane_clear is FALSE in every sample and eye_sees false in ~85% - from (112.5,-179.4) the Hind is mostly out of sight (the deep anchor that hides Bond from the gun also hides the Hind). A shorter slit (1.5/2.2 u) changes nothing (byte-identical). Next: a two-spot dance - wait deep while the gun cycles, step to the doorway anchor (111.09,-179.72) only when eye_sees from there, fire, step back.

Main 11-14: the timed deep/doorway anchor cycle (cycle,door = 600,200 / 400,200 / 900,300; deep whenever the gun just hit) - 0 launches, deaths t24509/t22511/t24508 (worse than the plain deep anchor t26671, 1 launch). The doorway leg must be triggered by the Hind being in sight from the doorway, not by a timer.
