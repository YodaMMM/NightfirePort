## Your task: `real-health-agent-06`, The Exchange castle interior can be walked to the party on Agent with real health

Branch from origin/master. walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_agent is ignored: the pass ended at 3.7 hp; the balcony drop raises ch 0x34 five ticks sooner, the reveal and the courtyard guards shift, and the walker runs dry at the mouth (clip 0, spare 0) and dies to the strip post #25 at tick 17089. Restock before the mouth (the level's ammo pickups [D]) and take #25 from cover. Real input only, never lower damage or health, no top-ups; helpers gated to 07000006 and difficulty 2. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000006_live and interactables_07000006_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
