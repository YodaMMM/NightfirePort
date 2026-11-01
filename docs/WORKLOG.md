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
- The bisect: 82367ac7 (mission-outros-2, ~01:25 UTC) — before it the no-`movies` player reported
  `ended: true` on an OPENED movie (state 6 popped at once, so the test's 6 s wall budget always
  saw the resolve); after it the opened movie holds the PSS's real length in flow ticks [M
  0x1081a8 BGFMVPlaying=1 on the open; 0x108060 clears at the decode pump's EOF; 0x1082e8
  psiMovieFinished = BGFMVPlaying == 0; main 0x109080's per-vblank poll] — correct game behaviour,
  but the harness test's 6 s wall budget was never updated. On a movie-data image (the driver's
  env: NF_MOVIE_ISO with real PSSs) the briefing alone is 5550 ticks [D pss_movies.tsv
  07100005.PSS: 4 440 064 samples @48 kHz = 92.5 s] — "the briefing resolve never started the
  level load", every run, idle machine included. The slim on-foot ISO stubs the PSS (start fails,
  pop at once) — that's why the slim-image runs stayed green.
- Reproduced locally without retail movie bytes: a synthetic 40 s PCM PSS in a pycdlib image at
  NF_MOVIE_ISO — pre-fix the test fails with the exact reported message; post-fix it passes and
  07000005 loads. Fix: `HeadlessApp::pump_ticks` (the mission_outros_live 6000-tick shape) and the
  resolve wait in 7000 ticks (> 5550 [D]); the load-finish wait stays wall (the LOADING_HOLD is
  wall: Job::elapsed_ticks). All three image×feature shapes keep their pins (movies build's Enter
  skip unchanged; stub pop covered by the pred).
- Totals on the final tree (slim ISO): nf-view lib 100/0; front_end_chain_live 5/0;
  campaign_flow_live 2/0; nf-game 1021/0/9i (all 99 binaries); nf-formats 201/4/3i (the 4 known
  slim-ISO environmental: 3 disc_golden + identity_disc's full-disc hash); nf-audio 4/0. The
  `movies` feature build compiles. Report: docs/reports/exchange-briefing-load.md.
