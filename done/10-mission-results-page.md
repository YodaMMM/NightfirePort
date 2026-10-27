## Your task: `mission-results-page`, the Results screen after a mission works

After completing Double Cross (07000004, the ninja kill, then the outro movie) the game lands on the mission Results page (07000048, page `P_NFRESULTS` 0x208e78). On it, "Del Stats" and "Tab Dossier" do nothing, and the rewards shown are not accurate. Port the page's input arms and data exactly: the Stats and Dossier sub-pages its 0x50 handler opens [M P_NFRESULTS 0x208e78 + its key/action arms], and the reward values it commits and draws (`GetScore`, `Menu_SetLevelBonus`, the medal/bonus fields the page reads [M :77, :169, the RewardsTable 0x2bc920 path]) from the finished mission's real stats (time, accuracy, kills, bonuses) — the port's mission-end currently builds the page with default stats (`menu::Stats::default()` in lib.rs `demo_results`). Drive it through the real end path (complete 07000004, the outro, the hub) in a test: Del opens Stats, Tab opens Dossier, Enter continues, and the reward/medal values follow the mission's tracked stats [M].

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-results-page`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T09:24Z

---
## Report (done, 2026-10-27T13:29Z)

Closed by the driver: the report is on `s/mission-results-page` (docs/reports/mission-results-page.md); the worker didn't run finish.sh.
