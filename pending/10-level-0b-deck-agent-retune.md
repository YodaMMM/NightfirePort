## Your task: `level-0b-deck-agent-retune`, 0700000b's Agent and 00 Agent real-damage walks are green after the files-plate fix

Branch from current origin/master (level-0b-files-plate-hazard is merged) and read docs/reports/level-0b-files-plate-hazard.md on origin/s/level-0b-files-plate-hazard, "What's left" items 1 and 2, plus the ignore messages on the 0700000b Agent (d2) and 00 Agent (d3) walks. The plate fix shifted the deck arrival about 700 ticks. First the Agent walk: it now dies in the deck band near tick 17366 (23 hits, 1 return-fire fight); the d3 six-round window extension did not clear it, so give d2 its own pass over the shifted phase using the Operative (d1) lever list as the template, judging each lever over several real runs. Then the 00 Agent walk: the same pass on its shifted phase (the last measured death is roof guard #4 at 17096). Helpers stay gated to 0700000b and the difficulty, so the Operative walk and its committed replay stay byte-identical (replay_parity_0700000b must stay green). Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000b_live binary, replay_parity_0700000b and interactables_0700000b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-deck-agent-retune`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

