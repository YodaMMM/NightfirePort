## Your task: `level-03-interactables`, every interactable kind on 07000003 works like the game

07000003 has no per-kind interactables test (0700000b's `interactables_0700000b_live.rs` is the model: one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets - driven the way a player drives it (walk, use key, trigger pull, walk-over, gadget), each asserting the game's effect with a sister). Census every player-touchable placement on 07000003 [D placements + docs/levels/07000003.md], write `crates/nf-view/tests/interactables_07000003_live.rs` with one test per kind, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (no #[ignore] without a named blocker). Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

