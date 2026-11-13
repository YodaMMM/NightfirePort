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

---
claimed-by: sandbox5 at 2026-11-13T13:02Z

Measured on master 631388054: the walk now stops EARLIER - the landing west drop (line ~812) wedges at (33.70, 1.19, 35.21), health 100. A d3-only guard g6 (28 health, state 0x4) stands at (32.2, 1.8, 33.2) inside the drop band, so drop_aim_z steers to z 34.9 into the overhang and Bond never moves. Bond reaches the landing with clip 0 / spare 0, so shooting g6 wakes him (he lands 70 in 14 hits, death t5203). Blocked-aim z 34.0 walks into him (death t3860); 34.5/35.3/35.8 wedge. Lever: arrive with ammo (find where the d3 route spends it) or a drop line that skirts g6 without waking him.

Ammo trace (11-13): the route reaches the trench guard kill with clip 3 spare 9 (12 rounds total) and that kill spends all of them; the path-clear stand on #1 (killed false) fires nothing more. Skipping the path-clear stand moves Bond off the landing (no wedge) but he arrives clip 0 spare 0 (gun 0x50) and the awake #6 (0x14, 28 hp) kills him point-blank (t3726; with a forced clear t3829, 0 return-fire fights - no rounds). Lever: ammo upstream - take the trench guard #0 drop / a box on the stairwell lane before the landing, or melee is not modelled for the walker.

LANDED on master 615e0239b: the drop-band guard #6 is shot from the landing with the pistol (switch_to_armed - Bond held the vent-lid item, the clip 0 spare 0 reading was that item), no hits taken; the drop aims z 33.4 past his body. The wedge is gone; the walk now dies at t7051 in the corridor lane north (node 57) to the crowd, 0 return-fire fights (#9 65.0 in 13 at d 1.2). Next: why the fights do not answer at d 1.2.

11-13 later (master 55add8c4c nearest-first clear, 4d1c87d11 fusebox cable aim - the t21466 death was the walk cutting the WRONG cable: group 1 = 0x71 = generator #388 kill word, 500/r10): the walk now dies at t23511 at the switch band. Budget: 100 -> 82.9 (landing clear) -> 72.9 -> 42.9 at the south loop node 42 door (-23.0, -19.6): the walk opens it for 157 ticks while #14/#15 fire through it (#14 killed false at node 42, both killed at node 44) -> 37.9 -> 22.9 -> #32 at the switch band (32.9 in 4, killed true at 23375) and his grenade lands 17.91 after his death (23511). Next: answer #14/#15 before the node-42 door opens (they are the 30.0), then the switch band with ~50.

Measured (not merged): a nearest-first clear (r 16, 900) inside the (-19.3,-22.8) door before the node-42 door: #14 engaged at d 10.6, #15 then drains 45.0 in 9 (was 30) - death t23890 (vs t23511). A clear placed before the (-19.3,-22.8) door wedged node 41 (the door cycles state 0x10 after the 900-tick stand). The #14/#15 pair needs a stand they cannot both reach, not a longer fight.

Live solo-line scan before the node-42 door (Bond (-21.45, -21.08); #14 (-25.95, -11.43), #15 (-25.94, -6.38)): (-19.75, -20.25) sees both with neither line back (the bullet ray is one-way there). Standing there killed both, but #14 still landed 30.0 first and the walk then fought the (-19.3, -22.8) door for ~6000 ticks; the death is the same switch-band spot later in time (t29484, #32 grenade 37.9). Not merged.
