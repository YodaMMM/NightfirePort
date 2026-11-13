## Your task: `real-health-00agent-15-8`, 07000015's 00 Agent real-damage walk reaches the omega sector exit

Branch from origin/master. walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent is ignored; the last round moved the death t5790 -> t7985 and cleared the desk corner, door[32] and the #526 lob; it now dies to the west-room grenadier pair's throw cycle - the ledger and next levers are in the ignore message. Run walkreal_07000015_live and interactables_07000015_live. Real input only, no top-ups, never lower damage or health; every lever gated to the level, its difficulty and !w.tops_up(). Remove the ignore when green, otherwise push the ledger and the next lever into it.

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

**Branch:** `s/real-health-00agent-15-8`.

---
claimed-by: sandbox3 at 2026-11-13T11:45Z

11-13 (on master 893dc3252): after the #526 lob the P2K comes back in hand when the gun is nearly empty - the resupply walk east is gone; the death moved t7985 -> t8578 (the door[29] through fight on #11 goes killed-false, #11/#10 throws 33.2 + 25.7). Measured dead: a head-first aimed stand from the threshold (#11 unseen at +200, death t8121).

11-13 (master a4b8a6b14): the west room is CLEARED - a live bullet-line scan at t7347 found #11 seen alone from (5.0, 71.25); from the threshold Bond walks there, head-first stand kills #11 (+20), then #18 and #10 nearest-first, health 34.4 kept. Death t8578 -> t9746 at (-5.66, 41.46, 34.60): the great-hall #5 (0x50000b2) 31.4 in 2 at d 12.7 (ledger lever 3).

Great-hall scan (11-13, at the door[50] threshold, Bond (0.02, 47.73)): live #5 (7.41, 36.21), #4 (-0.35, 27.95), #19 (-3.48, 30.24). No floor spot sees #5 with #4/#19 hidden; 53 spots see #5 at all (nearest (1.0, 40.25), 7.6 u from him) but all are in #4/#19 lines too. The hall needs a multi-guard order (#19/#4 first from a #5-hidden line), not a solo shot.

And the reverse: no floor spot (x -6..6, z 30..48) sees #4 or #19 while hidden from #5 - the great hall is fully mutually visible at the threshold moment. No position answer; the levers left are timing (the throw-cycle windows) or more health carried in (34.4 now).

11-13 (master b164773fb): a 300-tick hold before door[50] (sweep 0/60/120/180/300 -> t9746/11352/9948/9961/20553) crosses the great hall; death t20553 at the door[42] fence leg with 5.4 health and a dry gun (the fence shot resupply found nothing on the path).

After the hall (11-13): Bond holds the P2K at clip 1 spare 0, health 5.4; the hall drops lie at (5.19, 32.22) 0x34, (-0.52, 28.14) 0x1a + 0x1b, (-0.49, 28.00) 0x14 - w.resupply returns true but the P2K stays 1/0 and switch_to_armed finds nothing loaded (the take did not happen). With the extra walk the fence mark leg also hits the backpedal_face cap at (6.66, 39.87). Lever: walk onto (-0.5, 28.1) directly (the AR + SMG pile) before the door[42] threshold.

Correction + measure: the hall pile at (-0.5, 28.1) IS taken when Bond walks onto it (all three gone: 0x1a, 0x1b, 0x14 kind 0/1); Bond already owns the AR so it becomes AR rounds while the empty P2K stays in hand - draw_persist(0x1a) after it gives AR 9/36. Pickups work; the walk only lacked the switch. But at 5.4 health the first hit at the door[42] threshold kills (t11219). The limiter is health through the great hall, not ammo.

FRAGILE: the merged 300-tick hold (b164773fb) is a knife edge - a fine sweep 240/270/330/360/420/480 dies IN the hall every time (t10022-t10233); only 300 crosses. Any upstream change can re-roll it. Treat the hall as unsolved: it needs a real answer to #5 (the throw cycle) or more health in.
