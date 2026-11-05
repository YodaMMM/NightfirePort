## Your task: `local-match-bot-pickups`, bots take pickups in a match with no net session

In a multiplayer match with no net session, bots look unable to collect anything. The shared take walk `update_pickups_net` runs only when `Drones::net_bodies` is set, and scene.rs sets that only with a net row (around line 3641). `Sim::step_mp_pickups` (scene.rs:13688) is the pass written for this, humans then bots as in `Pickup_Update` 0x18e148, but nothing calls it. So `mp_shared_pickups` is never true in play, which also means lib.rs:6784's match detector picks the single-player Auto Aim row in such a match [M 0x1a7c98:39-45]. First confirm by test that a bot walks over a pickup in a local match and nothing happens; if a local match cannot be started from the real menus at all, stop and say so in the report. Read docs/reports/audit-pickup-calls.md on origin/s/audit-pickup-calls and merge that branch first if it is not on master: it edits `step_mp_pickups`. Call the pass from the per-tick MP driver at the point the game's order needs, for local matches only, leaving the online path as it is. Then wire the two dead helpers the report names: `mp::unregister_pickup` on delete [M 0x18e650:341] and lifetime expiry [M 0x18e148:257], and `reset_pickup_bot_visit_times` on a bot's respawn (0x171518). Pin through a real local match: a bot reaches a weapon, takes it, the sound plays at the pickup and the respawn timer starts, and with Auto Aim set differently on the two rows the MP row is read. Run mp_pickups_visible_live, mp_world_sync_live, every mp_*_live binary, pickup_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/local-match-bot-pickups`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

