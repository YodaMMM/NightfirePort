## Your task: `level-0b-objective9-writer`, 0b: objective 9 completes on the warm walk

level-0b-teleports-to-walks (merged; read its report's open items) left the disc writer of ch 0x54/0x06 (objective 9's OR input) unresolved on the warm walkthrough — only the fresh walk's path writes it — and the ch 0x75 touch after the cut-scene re-place does not fire in the walkthrough window. Find every writer of those channels [D the trigger/SS/script census of 0700000b; M Trigger_Update 0x1b1a78 and the script event handler], determine which one a player hits on the warm route, fix any port gap, and make `walkreal_0700000b_live` assert objective 9 completes on both walks (sister: it stays open before the writer fires). Run walkreal_0700000b_live and interactables_0700000b_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-objective9-writer`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-31T07:06Z
