## Your task: `zerog-channel-75`, remove the misread zero-g channel rule

The port maps switch channel 75 to "enter zero-g substate 9". The function it was modelled on (0x216388) is the cheat menu's ZEROG checkbox handler and its 0x4b argument is a menu message, not a channel (found during the Equinox walk work, origin/s/fix-1b-walk — read its report notes in the 0700001b test and docs/levels/0700001b.md). On the real route it only flips substate 8 to 9. Remove the misread rule, make the four older tests that relied on it start zero-g the game's way (the level loads in substate 8 [D/M]), and keep every 1b test green. Continue from origin/s/fix-1b-walk (merge origin/master first). Run walkreal_0700001b_live, interactables_0700001b_live, every zero-g/1b binary and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/zerog-channel-75`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

