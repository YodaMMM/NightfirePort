## Your task: `50-hard-walks-1127-a`, the ignored real-damage mission walks below green on current master

Branch from current origin/master (it carries the 11-26 stream batch: guards and Bond's body on the game's shared random stream, the route/sight cel fixes). Each test below is #[ignore]d with its measured death or wedge in the ignore message. For each one: reproduce the recorded failure on master first, read the ignore message, then find the fight or route that stops Bond and re-tune the walk (waits, stops, route points, which guard to engage first). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change game code. Judge each lever on its own real run. Remove each ignore when green; if still red, write the measured ledger and the next lever into the ignore message. Run each whole walkreal binary you touch.

- crates/nf-view/tests/walkreal_07000001_live.rs (the 00 Agent walk at line ~4367)
- crates/nf-view/tests/walkreal_07000006_live.rs (line ~1426)
- crates/nf-view/tests/walkreal_07000008_live.rs (line ~2935)
- crates/nf-view/tests/walkreal_0700000d_live.rs (both c5-door real-damage walks, lines ~2498/2533)

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-hard-walks-1127-a`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
