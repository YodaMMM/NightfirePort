## Your task: `gamepad-in-release`, the released PC build reads a game controller

Branch from origin/master. The pad backend exists (crates/nf-view/src/gamepad.rs, feature `gamepad` = gilrs) but tools/release/make-release.ps1 builds with `--features movies` only, so the shipped exe ignores controllers. Make the release build include `gamepad` (make-release.ps1 and notices.py's feature list, so gilrs and its deps land in the notices file), confirm a clean Windows release build, and add a test that drives the menus and a level start through the pad driver's input path (synthesised button/axis state into PadDriver, no hardware) so a release regression shows up in the suite. Do not change keyboard/mouse behaviour. Run nf-view --lib with and without --features gamepad, menus_e2e.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/99-gamepad-in-release`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
Main agent (2026-11-12): the release-script half is DONE on s/gamepad-in-release (0a8a18954: make-release.ps1 builds and notices with "movies,gamepad"; smoke OK, frame times unchanged) and HELD for a hands-on controller check. What remains for this task: the synthesized-pad test (PadDriver state into the menus and a level start, no hardware) - build it on top of that branch.
