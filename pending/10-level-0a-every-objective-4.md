## Your task: `level-0a-every-objective-4`, Tower B office floor: across the ring mouth to the hatch

Continue `level-0a-every-objective-3` (on master; read its report and the ignore reason on `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch`). The prologue (arrival, the real Q-Worm install, the keyboard) is green; the walk still wedges at the ring's mouth (the three wedge points in the ignore reason). Find the route the game's player takes through the mouth [D the collision around the mouth; M the player's capsule resolve; L if needed], fix any port collision gap, walk the ring legs and the hatch E on real input, and un-ignore the test. Keep it ignored only with the exact remaining blocker and evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-every-objective-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

