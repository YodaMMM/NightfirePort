## Your task: `cutscene-handback-census`, every cut-scene hands Bond back where the game does

`cutscene-kind31-frame-2` (on master) settled the frame rule for unparented case-1 streams (composed before `Script_CameraStart` resets the script base, keys-alone after) and censused the kind-0x31 streams. Extend that census to every SP script on every on-foot level [D]: for each script with an event-6 hand-back (`Script_EventHandler` 0x1d94c0 case 6 -> `Script_Free` 0x1da880:44-48 `Player_Enable`), compute the hand-back position the port produces and check it against the level geometry (on a floor, inside the level, near the scene) [D floor tris / nav]. Fix any scene whose hand-back lands outside the level or in a wall from the game's rule [M], and pin the census as a test so a later change cannot silently move a hand-back into the void.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-census`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

