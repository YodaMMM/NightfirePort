## Your task: `level-07-rook-doorway-pin`, Rook walks through the 07 party's west doorway on the real path

Branch from origin/master. A user report: Rook (drone 22, skin 0x5000026, spawn (80.71, 44.00, -50.47), WaitSwitch state 0x1 at load) froze in the 07000007 west doorway. Two worker branches (s/level-07-rook-doorway, s/level-07-rook-doorway-2: the per-cel AINetwork_BoundsTest 0x157d80 port and the +0x4F8 bit-0 push-move gate [M DroneMove_SetBoundryFlags 0x14f140:47; NDrone2_DefaultInit 0x14b300:60]) never reached master; master's code has since changed shape (no push_move). Write a real-input walk on master that drives the party to the point where Rook is released (the channel his WaitSwitch waits on [D placement]) and asserts Rook walks THROUGH the west doorway and on to his destination within a bounded time (no position writes, no channel pokes). If he freezes on master, port the -2 branch's findings (read its report) onto master the game's way. Run walkreal_07000007_live and the new test.



**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-07-rook-doorway-pin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
## Report (done by the main agent)
On master 729f9694e: Rook released t2273, walks 14.8 u, never stands >1 tick mid-route; ends Civilian -> DeleteMe. No freeze on master; the pin is in walkreal_07000007_live.
