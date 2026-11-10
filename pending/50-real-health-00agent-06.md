## Your task: `real-health-00agent-06`, The Exchange castle interior can be walked to the party on 00 Agent with real health

Branch from origin/master. walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_00agent is ignored: the ch-55 pair (#19/#20) charges the landing engagement from inside its 12 u sight, and the tunnel-roof guard #11 (y 55.5) holds the whole mouth-strip band in a full-damage lane. Read the full ignore message and its tried levers. Take #11 first from a spot outside his lane (his roof's own cover edges [D]), then meet the ch-55 pair from outside 12 u. Real input only, no top-ups, never lower damage or health; helpers gated to 07000006 and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000006_live and interactables_07000006_live. (50-real-health-agent-06 covers the Agent sister: if it is claimed or done, read its branch first.)

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
