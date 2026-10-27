# Report: mission-results-page

Branch `s/mission-results-page`. The mission Results page (07000048, `P_NFRESULTS` 0x208e78) after Double Cross (07000004) now works end to end: Del opens the Mission Stats page, Tab opens the Dossier, Enter continues, Back returns, and the score / medal / reward values on the page are committed and drawn from the finished mission's real tallies instead of zeros.

## What was wrong, and the fixes

1. **The sub-pages were never entered by the host.** The port built `menu::Stats` and `menu::Bonus` pages, but the only callers of `stats_enter`/`bonus_enter` were tests. The game's Stats page build rides the 0x4c handler of `P_NFSTATS` 0x209938 [M :31-38: `GetScore(0)` plus the row refill from `PlrStat_GetScore` 0x1984b0:66-84] and the Bonus page rides `P_NFBONUS` 0x209670 [M :14-18: `Menu_GetLevelBonuses`, the `DAT_002e1c2c` new-reward text gate]. Fix: the frontend keeps a snapshot of the mission-end `Progress` (`results_progress`, set when a mission completes in `demo_results`), and the `menu_after` hook calls the new `results_subpages_enter` whenever the flow lands on the Stats or Bonus page [S: port wiring choice — the game reaches these builds through its own message pump; the snapshot carries the same data the game's globals would].
2. **Tab/Del did not push the opener on the manager's back stack, so Back did not return to Results.** The game's page-manager case 0x44 with arg 0 pushes the current page and cursor (`+0x1bc`/`+0xac`) via `Stack_Push` [M 0x21c858 case 0x44: `uStack_b0=(arg^1)&1`; default pop path :592-600 re-opens the pushed page and restores the cursor]. Fix: the Results page's `Action::Side`/`Action::Extra` arms push `(Page::Results, cursor)` before entering Dossier/Stats, and the Stats page's Back arm pops that stack (restoring `flow.page`) instead of hard-wiring Results.
3. **The page was built from `menu::Stats::default()`** (lib.rs `demo_results`), so every value was zero. Fix: `mission_end_stats` reads the real `PlrMissionStats` fields the game's row refill reads — shots fired/hit `+0x04/+0x08`, accuracy (full 100 when no shots, [M PlrStat_GetScore 0x1984b0:74-78]), elapsed `+0x30` in 1/100 s, bond moves `+0x20`, dispatched `+0x0C`, subdued/disabled `+0x10`, surrendered `+0x14`, bonus pickups `+0x24` [M 0x1984b0:66-84] — and feeds the page through the game's commit path: score from `GetScore(0)` [M P_NFRESULTS 0x208e78:77], medal from the threshold chain [M :96-142], reward tier bits via `Menu_SetLevelBonus` [M :169, 0x1ffa10:70-82].
4. **Bonus root cause found on the way:** the sim never wrote elapsed time at all (`PlarStat_LogUpdateElapsedTime` had no caller). Fix: the drone tally fold logs it every tick [M 0x1995a0: `+0x30 += clock - +0x2C`, gated by `+0x34` ← Player_Update 0x193600:132; [S] the port derives the 1/100 s clock from the fixed 60 Hz frame counter].

## Verification

- `crates/nf-view/tests/walkreal_07000004_live.rs` `the_results_page_opens_stats_and_dossier_and_scores_the_real_tallies`: drives the real input path — pick up the Uzi and armour, shoot the target (skin 0x0500_001e), channel 0x54, mission end, outro walk — then on the Results page asserts score == last score, medal follows the threshold chain (580486 vs [125000, 250000, 500000, 1000000] → medal 3 for 07000001-base), reward bits == tiers ≤ medal, `new_reward` set; Del opens Stats whose rows equal the tracked tallies (29 fired / 29 hit / 100% / 26 s / 1 dispatched in that run), Back returns to Results; Tab opens Dossier, Back returns; Enter opens the Bonus page (new_reward) and continues to Mission Select. 5/5 green.
- `mission_stats_flow` 5/5, `reward_after_outro` 2/2, `menu_graph_walk` 2/2, `front_end_chain_live` 2/2.

SUITE: (filled below after the run)

## What's left

Nothing known for this task. The page draw itself (layout/label rendering) is the existing page renderer's job and unchanged by this task.

## Live checks for the main agent

The following would confirm the port against the real game; none block the merge:

1. After finishing any mission with shots fired, on the Results page the TIME row should show `mm:ss.d`-style time that matches the playthrough length, not 00:00.
2. Del on the Results page opens the Stats page listing rows BOND MOVES / DISPATCHED / DISABLED / SURRENDERED / SHOTS FIRED / SHOTS HIT / ACCURACY / TIME / BONUS PICKUPS with the run's values; Back returns to Results with the cursor on Del Stats.
3. Tab on the Results page opens the Dossier page; Back returns to Results.
4. On a first-time medal (any tier), the Bonus page shows the new-reward text (the `DAT_002e1c2c` gate [M 0x1ffa10:76-79]), and after returning to the hub the reward persists on the mission select entry.

## Usage

not visible to me
