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

mission-results-page — the results page works (2026-10-27)

- The mission Results page (07000048, `P_NFRESULTS` 0x208e78) now behaves like the game's:
  Del opens the Mission Stats page, Tab opens the Dossier, Enter continues, Back returns,
  and the score/medal/reward values come from the finished mission's real tallies.
- Three root causes fixed, each cited in the commit:
  1. The sub-pages' 0x4c builds were never entered by the host: `results_subpages_enter`
     rides the mission-end Progress snapshot through `stats_enter`/`bonus_enter`
     [M P_NFSTATS 0x209938:31-38, P_NFBONUS 0x209670:14-18].
  2. Results' Tab/Del arms did not push the opener on the manager's back stack; they do
     now, like the game's 0x44 arg-0 arm [M 0x21c858 case 0x44 `(arg^1)&1` -> Stack_Push],
     and Stats' Back is the manager's default pop [M 0x21c858:592-600].
  3. `demo_results` built the page with `menu::Stats::default()`; `mission_end_stats`
     now reads the real PlrMissionStats (shots/hits/accuracy, elapsed, rows) and the
     elapsed clock accumulates per tick [M PlarStat_LogUpdateElapsedTime 0x1995a0
     <- Player_Update 0x193600:132] — the sim never wrote elapsed time before.
- Verified through the real end path in `walkreal_07000004_live.rs`:
  complete 07000004 -> outro -> Results page: score = last score, medal follows the
  threshold chain, reward bits follow the medal tier, new_reward set; Del -> Stats rows
  equal the tracked tallies, Back returns; Tab -> Dossier, Back returns; Enter -> Bonus
  page (new_reward) -> Mission Select. 5/5 green.
