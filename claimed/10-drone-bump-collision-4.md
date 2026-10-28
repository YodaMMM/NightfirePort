## Your task: `drone-bump-collision-4`, the guard push with the party and Mayhew tests green

Start from `origin/s/drone-bump-collision-3` (the arming ported from the disassembly: ext+0x464 kind word rewritten every control tick [M NDrone2_Collision 0x140e90], NDrone2_DoCollision 0x149ec8, the +0x154 near slots on the bot's own control pass [M 0x148db0:35-48], the frozen-system zero [M 0x140968:71-75], the disabled-player skip [M 0x193600:80-88, 0x195f70:12-18], the capsule bottom from ext+0xa0) merged with `origin/master`. Still red: party_door_lock (x2), cutscene_timer_pause, probe_07_route (the 07000007 hall's south band - the walker wedges at the 0.95 contact against the armed party crowd), mayhew_hold_live (the 07000001 flank legs wedge on armed combat guards). The push is the game's [M]; what the tests lack is a player's answer to a crowd. Give the walker the real player's response - wait for the crowd to clear the line, step around the contact (a sidestep the way a player steers), or take the west mouth on the measured timing - with the decompile cite for every assumption about the bots, never by disarming them. Run the FULL suite and quote its total line; every red accounted for.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T06:57Z
