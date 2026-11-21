## Your task: `retune-nb-01`, 01 spawn-to-exit-with-the-contact real-damage walks green again (Operative, Agent, 00 Agent)

Branch from current origin/master. Commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states; 07e6b10 (11-18 07:57) re-routed the escort through the hall's north opening, and the three walks walk_from_the_spawn_to_the_exit_with_the_contact_real_damage, _agent and _00agent are still ignored, each with its measured death in the ignore message. Re-tune them for the new fights, Operative first: read each ignore, find where the new close-range attacks or bunching cost the health, and change the route, stance or fight order there. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-01`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

