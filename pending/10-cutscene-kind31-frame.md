## Your task: `cutscene-kind31-frame`, the general rule for unparented kind-0x31 script streams

`level-14-capture-placement` (on master) fixed Countdown's capture-scene hand-back by excluding script 0x0600062F's unparented kind-0x31 streams from the SP base compose (they key in world frame once `Script_CameraStart` 0x1d9140 has reset the script base to identity). It is scoped to that one script on purpose: 07000001's intro 0x06000087 s5/s11 (also kind-0x31, hash-0, in a camera script) still uses the composed reading. Find the general rule the game applies [M Script_Interp 0x1d9a08 case 1 :91-138, Script_CameraStart's base reset, Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48] and replace the per-script scope with it, so every cut-scene's markers and hand-backs land where the game puts them. Census every SP script's kind-0x31 streams [D] and check each scene's hand-back position against the level geometry; the 01 intro and 14 capture tests stay green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-kind31-frame`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

