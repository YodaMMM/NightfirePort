## Your task: `subtitle-flush-camera-scripts`, a new cut-scene clears the subtitles already on screen

`Script_Play` 0x1db588 calls `Text_FlushAllSubtitles` 0x1b15c8 on every camera script start (`+0xAA8 & 0x40`), not only on the three death NISs: the music event is gated, the flush that follows it is not. The port flushes only on the skip arm (drone.rs `subtitle_track.flush`, around line 16705), and the comment near nis.rs:1688 says the flush rides the view side; find out whether any view code does, and correct the comment either way. If nothing does, a line still showing when a camera script starts carries over into the scene. Add the flush at the `Script_Play` site for every camera script. Read 0x1b15c8 and match what it deletes (the type-4/5 messages) and leaves. Check that the start of a script does not eat the first line of that same script: the order of the flush against the script's own first subtitle must follow the game. Pin it through the real path on a level where a subtitle is live as a camera script starts, and pin that a non-camera script leaves it alone. Run the subtitle and cut-scene binaries (grep tests for `subtitle_track`), the nis tests and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/subtitle-flush-camera-scripts`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-05T13:15Z

---
## Report (done, 2026-11-05T15:07Z)

