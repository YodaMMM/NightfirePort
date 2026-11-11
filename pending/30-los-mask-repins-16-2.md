## Your task: `los-mask-repins-16-2`, 07000016's three defeat walks are green with the game's guard sight

Branch from origin/s/los-mask-repins-16, merge current origin/master, and read its docs/reports/los-mask-repins-16.md, "What's left", items 1 to 3. First the Agent d2 walk: extend the gallery-rifleman stand ("the ring 1") to the station_armour defeat walks and/or clear #5 from the ring's west legs. Then the 00 Agent d3 walk, on the same levers. Then the Operative d1 walk: walk into the pit room toward the squad member and head-tap each one on arrival, with the dealer stands between, and tighten the pit-room armour #648 arrival. One walk per tool call. Helpers stay gated to 07000016 and the difficulty, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000016_live binary and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-repins-16-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

