## Your task: `guard-capsule-radii-4`, the far-guard no-physics arm lands without freezing guards the game moves

Branch from origin/s/join-1110-guards (guard-capsule-radii-3 + a fall-speed fix, 43b2739f5) and merge current origin/master. A full suite with it has 4 reds master lacks: abseil_death_live (both: a guard killed on the rope never lands), abseil_damage_live::real_rounds_kill_the_roping_guard_through_the_repro (the corpse never lands), hostage_rescue_msgs_live::first_rescue_posts_one_of_two_second_posts_all (rescue channel 74 never rises). All four pass with `no_physics_now` forced false, so the cause is that arm. The game's arm [M NDrone2 0x149ec8:16-31] keeps physics on while obj+0xFC (the object's drawn byte, written by the render pass) is set; the port stands in `drawn` = a line-of-sight ray from Bond to the head, which is stricter (an occluded guard in view range reads undrawn). Find obj+0xFC's writer [M], model it the game's way (what makes the game draw a drone: room visibility, frustum, distance), and keep the arm; the four tests must go green with it on, and ai_live's 05 gate-guard pin must still hold. Run abseil_death_live, abseil_damage_live, hostage_rescue_msgs_live, ai_live, guard_capsule_radii_live, and every walkreal_*_live binary one level per tool call.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-capsule-radii-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: local-agent (lane nf-agent1) at 2026-11-10T09:07Z
