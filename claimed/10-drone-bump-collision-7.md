## Your task: `drone-bump-collision-7`, the 07 hall's east door and its two door guards

Continue `origin/s/drone-bump-collision-6` (merge `origin/master`). The west mouth is green; all four red tests (party_door_lock_07000007_live x2, cutscene_timer_pause_live, probe_07_route, plus walkreal_07000007_live's hall walk) now wedge at the east hall door (94.9, -47.2) between the two CIVILIAN_DOOR_GUARD bots d10/d19 (state 0x31) whose push capsules close the doorway. Your -6 report names the gaps: their reaction system never fires in the port (the 0x31 UPDATE gate near_armed_person [M 0x160648:39-57] needs the reaction-acked sight +0x228 & 4, DroneFunc_ReactionTime), and the port's bots have no generic cylinder-vs-world depenetration [M Collide_Update 0x1ea398]. Port what the game does when the player walks up to these guards [M the 0x31 state and its zone reactions] and the bot depenetration if the game uses it there, then turn the reds green on real input - never by disarming the push. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T14:46Z
