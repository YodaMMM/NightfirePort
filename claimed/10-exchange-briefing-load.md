## Your task: `exchange-briefing-load`, does The Exchange's briefing still hand off to the level?

`nf_view::headless_app_harness::boot_to_the_exchange_briefing_holds_then_the_level_loads` (a unit test in crates/nf-view) now fails on origin/master every run, idle machine included: "the briefing resolve never started the level load". It passed in mid-day suites, so a recent merge broke it — possibly a real player-facing stall (the mission-select -> briefing -> level path) rather than its 6 s wall-clock budget. Bisect it over the last day of origin/master merges, find the cause [M the briefing page / MissionSelect handoff, Menu_RunMiniMission 0x1ffb90 / the level-load request path], fix the port (or, if only the test's timer is wrong, prove the real path loads and make the test tick-based instead of wall-clock), and confirm the real app path from the title through The Exchange's briefing loads 07000005. Run the nf-view unit tests, front_end_chain_live, campaign_flow_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/exchange-briefing-load`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T15:08Z
