## Your task: `level-0b-roof-sprint-10`, the 0700000b roof sprint on Agent and 00 Agent under the game's guard sight

Branch from origin/master (823ff190b: the game's guard sight; the 0b Agent walk is ignored there pending its re-pin). Read docs/reports/level-0b-roof-sprint-9.md on origin/s/10-level-0b-roof-sprint-9 (built on an older sight tree: do NOT merge that branch; port its walk levers by hand: the early round chain-killing #7+#4, the take timing). Make walk_fresh_spawn_to_the_mission_end_real_damage_agent green on master and un-ignore it, then the 00 Agent sister. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and the difficulty. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T14:41Z

---
## Report (done, 2026-11-10T15:31Z)

