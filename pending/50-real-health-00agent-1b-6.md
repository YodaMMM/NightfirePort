## Your task: `real-health-00agent-1b-6`, 0700001b's 00 Agent real-damage walk survives the console phase

Branch from origin/s/real-health-00agent-1b-5 (its ledger is not on master yet) and read docs/reports/real-health-00agent-1b-5.md there, "What's left" items 1 and 2. First the cap math: the fight cap is deadline - 400, which lets a fight eat the slack a console chain still needs (a chain measured ~900 ticks, approach end 4374 -> latch 5574). Change it to deadline - CHAIN_RESERVE (~1000, measure it) and allow no fight once Bond is within ~4 u of the pending station. Then thin the fights: move the lane to the wave's approach side instead of |z| 1.6 behind the panel rows, and prefer the nearest shooter that has crossed. Judge each lever on its own real run and keep the ledger in the test's ignore message. Helpers stay gated to 0700001b and difficulty 3, so the other difficulties' walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the new ledger and the next lever (item 3, the Drake phase spawn band, comes after the console phase survives). Run the whole walkreal_0700001b_live binary and interactables_0700001b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-1b-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

