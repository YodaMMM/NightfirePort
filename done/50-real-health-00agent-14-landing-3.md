## Your task: `real-health-00agent-14-landing-3`, 07000014's 00 Agent infiltration real-damage walk gets past the landing

Branch from current origin/master; if origin/s/real-health-00agent-14-landing-2 is not on master, merge it in first. Read docs/reports/real-health-00agent-14-landing-2.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Measure the middle-lane crossing (z 34.0-34.5) the report proposes: #29's sightline from (-49.1, 30.2) to that lane, while keeping clear of #20's pocket; then carry the walk on. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-14-landing-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


Measured 11-15 (sandbox5, landing-3, master f3c20899a): the baseline re-verified byte-identical (t29197, #30 0x5000087 -2.91 on the 2.9, 43 hits for 100, 17 return-fire fights).

Measured 11-15 (probe14_kiko_sight, deterministic d3 load, 3 green tests): the middle-lane crossing (z 34.0-34.5) is CLOSED BY GEOMETRY - #29's sightline from his measured firing spot (-49.1, 30.2) is CLEAR across x -50..-48 at every z 34.0-34.5 (d 3.8-4.4, CLOSER than the north lane's 6.1 kill), blocked only west of x -51 (t 0.44-0.9) and east of x -47 (t 0.65-0.8); #20's pocket line from (-51.2, 32.6) is blocked to x -52..-50 (hide cover, t 0.21-0.28) but CLEAR to x -49..-48; the only band clear of both lines is x <= -51.5 (inside #20's pocket itself, inside the 3.0 u auto-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115]) or the east cover at x <= -47 (d 1.3-1.5 from #21's hidden spot (-47.9, 33.5)). The hold route runs (-48.64, 0.40, 32.13) -> (-48.63, 0.40, 30.49) - the node-0 pass is d 2.7 from the pocket. No walkable z 34.0-34.5 line keeps clear of both shooters; the guards track (the z 34.4 run's t27096 death). The lever is closed without another run.

Measured 11-15: the frag-lob precondition is NEGATIVE - the port does NOT implement the player's grenade throw (player.rs/gun.rs carry the 0x34/0x37 as ammo bookkeeping only, clip[0x45] the grenade-belt pseudo-weapon [M 0x1a16c0:14-15]; the only throw path is the drones', DroneAnim_EventFunc datum 1 -> pending_grenade -> DroneWeap_ThrowGrenade 0x179558 [M drone.rs]). Next lever: implement the player's throw the game's own way (the anim-state 0x49 throw release feeding the 0x179558 arc/fuse, the law the drone path already runs), then the lob from the Kiko hold (-51.3, 37.9) onto the pocket kills #20 (hp 10) AND #29 (hp 14) inside the frag's radius 8 / damage 100 [M 0x12cd98/0x12c5a0 records] (both sit 5.3-5.6 u from the hold spot), with the thrower retreating west through the opened door #151 out of the 8 u band before the 210-tick fuse pops (the hold spot is d ~5.5, inside the band).

---
claimed-by: sandbox5 at 2026-11-15T04:01Z

---
## Report (done, 2026-11-15T05:03Z)

