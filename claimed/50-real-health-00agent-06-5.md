## Your task: `real-health-00agent-06-5`, 07000006 can be walked to the party on 00 Agent with real health

Branch from origin/s/real-health-00agent-06-4, merge current origin/master, and read that branch's docs/reports/real-health-00agent-06-4.md plus the ignore message on `walk_the_castle_interior_to_the_party_real_damage_00agent`. The walk now ends at the east-strip arrive (9.73, -35.26) t8893, about 33 hp short. Take the report's levers in order: first engage #24 before the east-strip legs, from a stand leg east of (14, -36.7) before the (10.5, -34.3) nav; then shave #4's doorway fight with the `aimed_fights` tight gate, which was not tried there. The pre-door `fight_through` and the kite are measured dead ends. The ledge and dressing phases have never been reached on 00 Agent, so print their costs as soon as the walk gets there. Helpers stay level-local and gated on difficulty 3, so the other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000006_live binary and interactables_07000006_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-06-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T13:32Z
