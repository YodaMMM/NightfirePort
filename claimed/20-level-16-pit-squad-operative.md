## Your task: `level-16-pit-squad-operative`, the 16 chase completes on Operative with real damage through the pit squad

Branch from origin/master. walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage (Operative) is ignored: the pit squad's ninjas fire at accuracy byte 0 [M 0x14b300:752]; after the chaser at 46.4 Bond dies at tick 15337 to #12 at 0.4 u. First confirm the accuracy byte path (where 0x14b300:752 reads it, what a 0 does in DroneWeap_DoBulletAccuracy 0x179740) so the walk tunes against the game's rule; then build a stand for the pit fight on real input (distance, cover, the level's ammo/armour pickups [D]). Never lower damage or health, no top-ups; helpers gated to 07000016 and difficulty 1. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000016_live and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-16-pit-squad-operative`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Main agent's check (2026-11-12): the accuracy byte path is confirmed
- NDrone2_DefaultInit 0x14b300:745-752: class (+0xC5) 0x0D drones get props 0x3c/0x36/0x35, health 100, +0x8f0 8.0, +0xEC 2.0 and +0xB4 = 0 [M].
- DroneWeap_DoBulletAccuracy 0x179740:52: chance = 100 - (+0xB4)·5, so byte 0 = 100 before modifiers; then x TooClose_Accuracy under TooClose_Distance (:53-55, again :95-97), x TargetFirstMoved_Accuracy when +0x1dd (:56-58), x TargetMoving_Accuracy when Bond moves (+0x1dc, :78-80), x the difficulty factor (Easy on Operative, :81-93); a hit needs FRand(100) < chance (:100-101) AND the new-sighting time-to-hit past +0x270 beyond TooClose (:103-109); under 3.0 every round hits (:113-115).
- So the stand's levers are the game's own: keep Bond MOVING while exposed (the moving-target factor), stay out of 3.0 u, and break first sight (the time-to-hit gate). The death at 0.4 u is inside the auto-hit band.

---
claimed-by: sandbox5 at 2026-11-12T20:01Z
