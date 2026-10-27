# WORKLOG

### mission-results-page — start (2026-10-27 ~09:30 UTC)

Branch `s/mission-results-page` created from `origin/master` @ 83490999.
Task: after Double Cross (07000004, ninja kill, outro movie) the game lands on
the mission Results page (07000048, page `P_NFRESULTS` 0x208e78). There,
"Del Stats" and "Tab Dossier" do nothing, and the rewards shown are not
accurate. Port the page's input arms and data exactly: the Stats and Dossier
sub-pages its 0x50 handler opens [M P_NFRESULTS 0x208e78 + key/action arms],
and the reward values it commits and draws (`GetScore`, `Menu_SetLevelBonus`,
the medal/bonus fields the page reads [M :77, :169, RewardsTable 0x2bc920
path]) from the finished mission's real stats (time, accuracy, kills,
bonuses) — the port's mission-end currently builds the page with default
stats (`menu::Stats::default()` in lib.rs `demo_results`). Drive it through
the real end path (complete 07000004, the outro, the hub) in a test: Del opens
Stats, Tab opens Dossier, Enter continues, and the reward/medal values follow
the mission's tracked stats [M].

Plan:
- Read the P_NFRESULTS 0x208e78 decompile: the 0x50 (key) handler arms — Del
  opens Stats, Tab opens Dossier, Enter continues — and the reward commit
  path (:77, :169, RewardsTable 0x2bc920, GetScore, Menu_SetLevelBonus).
- Find where the port builds the Results page with `menu::Stats::default()`
  (lib.rs `demo_results`) and wire the real mission-end stats through the
  game's commit path.
- Reproduce through the real end path (07000004 complete -> outro -> hub) in
  a test; pin Del/Tab/Enter arms and reward values with [M] cites.
