## Your task: `level-4a-ride-hand-props`, the lift-ride cut-scene's guards hold their weapons

In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289) the guards' hand props (their weapons) are missing. Find how the game attaches props to script-animated actors [M the script entity create Script_CreateEntity 0x1d8968, the anim object's attachment/bone-socket path, the pack's prop entities and their parent links] [D the pack's streams and objects] and port it so each prop draws in its actor's hand for the whole scene. Test: in the scene, each prop exists and its pose follows its parent bone [M/D]; the 07000014 scenes stay unchanged.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-ride-hand-props`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-27T16:32Z
