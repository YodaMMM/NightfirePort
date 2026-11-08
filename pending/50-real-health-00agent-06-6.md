## Your task: `real-health-00agent-06-6`, 07000006 can be walked to the party on 00 Agent with real health

Branch from origin/s/real-health-00agent-06-5, merge current origin/master, and read that branch's docs/reports/real-health-00agent-06-5.md plus the ignore message on `walk_the_castle_interior_to_the_party_real_damage_00agent`. The walk is still red by about 33 hp. Take the report's named lever first: the shift-free loot, a pre-guardroom resupply whose corridor search is pinned to the court box (30.6, -12.3), with the pair engagement re-measured under exactly that shift. Use its phase census for the ledge and dressing costs. Do not repeat its measured dead ends. Helpers stay level-local and gated on difficulty 3. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000006_live binary and interactables_07000006_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-06-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

