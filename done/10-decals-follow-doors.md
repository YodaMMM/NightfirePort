## Your task: `decals-follow-doors`, bullet holes follow doors, draw by room, debris slots

Continue `origin/s/wip-decals-2` (it builds on origin/s/fix-stairwell-decals; merge `origin/master` first). Three gaps: (1) in the game a bullet hole on a door moves with the door — in the port it stays put, so holes on opening doors float (07000011's #192 and #213, any level); (2) the game draws a hole only while its room is visible; (3) the flying-debris pool shifts every entry when full, so interpolated frames draw debris between two pieces (the decals had the same bug; their fix keeps each entry in its pool slot for life). Find the decal owner/matrix fields and their per-frame update, the draw-time room check, and the debris pool rule [M]; fix; render before/after (NF_INTERP_SHOT renders the in-between frame); tests with sisters. Run every bullet/decal/debris/door/interp binary and walkreal_07000011_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/decals-follow-doors`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T19:58Z

---
## Report (done, 2026-10-31T20:53Z)

