## Your task: `level-4a-interactables`, every interactable kind on 0700004a works like the game

Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, gadget targets - driven the way a player drives it, each asserting the game's effect with a sister), read what the existing interact_/walk tests for 0700004a already cover, census every player-touchable placement on 0700004a [D placements + docs/levels/0700004a.md], add the missing kinds as `crates/nf-view/tests/interactables_0700004a_live.rs`, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (an ignore only with a named blocker). No comment or doc may name who runs a live check. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T22:59Z
