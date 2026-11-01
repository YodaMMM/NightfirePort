## Your task: `level-0d-beam-boarding`, 0d: onto the moving beam without a teleport

`walkreal_0700000d_live`'s full walk teleports Bond onto the moving beam: the surfaces it can be boarded from are unreachable with the port's collision. Find how the game's player reaches and boards it [D collision, the beam's mover path and keys; M the mover class, the platform ride / FeetOnPoint 0x194188, any jump or ladder on the way], fix the port gap with the address cited and walk it on real input. Run walkreal_0700000d_live, interactables_0700000d_live, every *0700000d* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-beam-boarding`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

