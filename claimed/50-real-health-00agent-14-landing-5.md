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
