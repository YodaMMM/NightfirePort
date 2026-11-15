## Your task: `real-health-00agent-14-landing-5`, 07000014's 00 Agent infiltration walk reaches the delta exit

Branch from current origin/master.

Master fcd3ca397: the escape touch is reached at 26.6 health; the walk dies at t41509 at (-15.85, 1.13, -22.71) on the Rook legs. Read the landing-4 ignore ledger on master. Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-14-landing-5`.

---
claimed-by: sandbox3 at 2026-11-15T09:20Z

Main 11-15 (on fcd3ca397): the escape-room armour (-27.3, 0.9, 31.0) right after the touch (now reached at 26.6): death at the room door t40134 - worse than t41509.

Main 11-15 (on c39691b2d, idle re-roll sweeps, ~80 runs): an idle injected late (36400-38200, 60-200 ticks) re-rolls the last legs - the best rolls reach the Rook doors (23.5-27.4, -22.3) and die there to #50/#46 (0x500008c, 0x1a) at 1.5-3.8 u on 5-9 health (#50 stands behind the slide door, unseen until it opens). Waits at the load (< 150 ticks) change nothing. Measured DEAD on those rolls: the escape-room armour (-27.3, 31.0) and the west armour (-41.3, 36.5) detours (deaths at their room doors), a pre-door visible-guard clear (nobody visible before the door), and a delta-approach stop at (-18.0, -22.6) with a visible-guard clear (radius 15/22; #40/#51 outgun the stop). The walk needs ~10-15 more health into the door fight.

Main 11-15 (2): two-stage idles (36600:150 or 37400:120, then 41000-42000:30-90) put Bond PAST the Rook door at (26-27.3, -19 to -21) - the stretch beyond is a gauntlet of 0x500008c 0x1a guards (#50 behind the door, #53, #46 (29.3, -18.4), #47 at the second door (27.8, -19.6), #57 (27.9, -4.7)), entered on 8.6 health. Measured no help there: the damage-log opt-ins (hurting_first/aimed/precise/unseen/finish/eye_ray/world_ray, round_dodge off) - hurting_first only reaches (27.30, -19.22); a kiko_hold-style door hold at the leg-5 end (150-3600 ticks: the guards come out and kill Bond in the hold). The fix has to save ~15 health before leg 5 (the touch 26.6 -> leg 5 start 19.1 after #22 at 6.4 u by the escape room).

Main 11-15 (3): on master c39691b2d the costs after the Kiko hold (52.5): #20 (0x5000090, 0x14) 15.0 at 2.8 u + #31 3.44 in the east lane (t31075-83), #22 2.5 at the touch, #43 3.5, then #24 (0x500008c) - his fight at t40001 ends unkilled, he closes to 1.0 u for 7.0 + 7.0 - then #40/#41 on the delta approach. Damage-log opt-ins set from leg 5 (hurting_first, aimed, precise, unseen, finish_fights, eye_ray, world_ray, escalating_cooldowns, lane_fights off, back_off off, sniper_stands): best finish_fights t41810 (vs t41509). finish_fights + an idle 150 at t36600 kept 19.3 health to leg-5 node 8 (-28.1, 4.7) but the #37 fight there emptied the gun (clip 19 spare 8 -> 0, out of ammo at +807) and the leg wedged at the 6000 cap - AMMO is the next limit on that roll. Resupply there measured dead: w.resupply and a direct nav to the escape-room 0x1a boxes (-35.2, 1.9, 32.2) both route west through (-42.2, 29.3) and die there. Next: a resupply source on the leg-5 path itself (a dropped 0x1a from #25/#43/#36 near nodes 2-8), or ammo discipline in the leg-5 fights (bounded shots instead of the 807-tick determined pass on #37 at 9.4 u).

Main 11-15 (4): ammo on leg 5 is solved by scooping the corpses' drops - leg 5 split at (-24.2, 0.9, 10.7) then walk_nav_to each live pickup within 8 u (dy < 1.5): with finish_fights + an idle 150 at t36600, the scoop leaves 0x1a clip 25 spare 35 at 15.8 health and the walk reaches the Rook door again (death t42795 at (27.04, -20.24)). The remaining gap is health into the Rook-door gauntlet (~10-15 more). Probe code: a leg-5 split/scoop env hook, NF_WAIT_AT idles in walker_b2 tick_inner.

---
## Report (done, 2026-11-15T11:30Z)

