## Your task: `level-16-stuck-overlook`, 16: no way on from the overlook by the control-room glass

On 07000016 at Bond (-17.86, 6.81, -97.16) facing yaw 6.98 (the walkway beside the big control-room window, around tick 3700 of a real run) the player cannot continue the level, and the held rifle's view model visibly clips into the glass/wall. Find what the game expects here [D the route on from this walkway: doors, triggers, the channel that opens the way; M the classes], reproduce on real input from the level start with the port's walker, fix any port gap with the address cited, and pin a walk that passes this spot. Also check the view model's near-clip / wall push-back at that spot against the game [M the view-model draw path]. Run walkreal_07000016_live, interactables_07000016_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-stuck-overlook`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-01T02:37Z
