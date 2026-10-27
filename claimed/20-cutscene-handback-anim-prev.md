## Your task: `cutscene-handback-anim-prev`, cut-scene hand-backs from an animated actor

`cutscene-handback-census` (on master, `crates/nf-game/tests/nis_handback_census.rs`) found that the game's event-6 hand-back admits a previous stream of kind 1 OR 2 and snapshots that object's +0x90 either way [M Script_EventHandler 0x1d94c0:73-84, :125-134], and an anim object's +0x90 is its live matrix [M AnimObjectUpdate 0x1befd8:290 -> Control_GetObjMatrix 0x134210 View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos)]. The port's event-6 arm reads only `Obj::Entity`, so four scenes hand Bond back where he was disabled instead: 07000002's 0x0600007f and 0x06000084, 07000003's 0x06000080, 0700004a's 0x060006d3 [D]. Carry the anim object's world matrix on `Obj::Anim`, let the event-6 arm snapshot it, and make those four census rows hard assertions (each on a floor [D]). The census stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-handback-anim-prev`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T21:21Z
