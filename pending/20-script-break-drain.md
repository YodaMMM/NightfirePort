## Your task: `script-break-drain`, panes a script creates can really be smashed

`Script_CreateEntity` 0x1d8968's obj_type 0x20 arm calls `Break_Create` 0x12c230. The port's arm in crates/nf-game/src/nis.rs (around line 902) pushes `Objects::pending_breaks`, and the comments there say the sim drains it into `Breaks`, but nothing reads that list on master. Read the queue's done/30-port-map-scripts.md, which records this. Script-created breakables therefore never exist as breakables through that path. The known instance is 07000008's 16 gondola windows (ride script 0x0600004C, streams 7-18/23-26, params [2, 1, 0], parented to carriage stream 2 [D decode]). The windows were fixed once before by another route, so first find how they break on master today and whether that route overlaps this one, and keep a single path. Add the drain after the scripts tick, with `Break_Create`'s kind, health and filter taken from the params, and make the breakable's bounds follow the stream's posed matrix every tick. An existing same-class stream object is re-pointed without a new breakable [M 0x1d8a68-0x1d8a80], so only the create path may add one. Census every script on the disc that creates an obj_type 0x20 entity [D], listing level, script and stream, because the gondola may not be the only one. Pin through the real path on 07000008: Bond rides the gondola, shoots a pane while it moves, and it breaks at its moved position. Pin one more level if the census finds one. Run walkreal_07000008_live, the breakable binaries (grep tests for `Breaks`), the nis tests and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/script-break-drain`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

