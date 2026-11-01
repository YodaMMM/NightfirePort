# WORKLOG

## exchange-briefing-load (branch s/exchange-briefing-load)

- start (master 71b7b6b9, 2026-11-01 ~15:12 UTC). Task: nf_view::headless_app_harness::
  boot_to_the_exchange_briefing_holds_then_the_level_loads fails on origin/master every run
  ("the briefing resolve never started the level load"); it passed mid-day. Bisect the last day
  of master merges, find the cause [M briefing page / MissionSelect handoff, Menu_RunMiniMission
  0x1ffb90 / the level-load request path], fix the port or prove the real path loads and make
  the test tick-based, confirm title -> The Exchange briefing -> 07000005 loads. Run nf-view
  unit tests, front_end_chain_live, campaign_flow_live, nf-game.
- Setup reused from prior session (repo, work/, ISO, nf-queue, toolchain all present).
