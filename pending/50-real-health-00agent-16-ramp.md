## Your task: `real-health-00agent-16-ramp`, 07000016's 00 Agent chasers walk survives the corridor approach

Branch from current origin/master and read the ignore message on walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent in crates/nf-view/tests/walkreal_07000016_live.rs, plus docs/reports/real-health-00agent-16-1b-4.md on origin/s/real-health-00agent-16-1b-4, "What's left" item 2. Bond dies in the corridor approach cluster near tick 1030 at (12.12, 0.93, -88.14). The cheap levers are measured there; take the next one: a cover-shaped approach from the ring that meets the ramp-foot cluster a few at a time, or an armour pickup the level really has before the ramp. Judge each on its own real run. Helpers stay gated to 07000016 and difficulty 3, so the other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary and interactables_07000016_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-16-ramp`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

