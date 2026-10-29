
## level-14-invisible-guard (started 2026-10-29)

- Branch s/level-14-invisible-guard opened from origin/master ccc714a1. Task: 07000014
  (Countdown Part 1) has a guard that acts but draws nothing; load-log candidate g-021
  skin 050000b1 carried only by 07000016.bin/07000015.bin, no session container registers
  it, placements resolve through the Drone+0xe0 rewrite [M 0x14b300:82-95, :563-565] and
  a skin with no carrier misses in AnimObjectNew 0x1be910:44-46 and draws no character.
  Plan: (1) census every present/enabled drone on 07000014 without a drawn character
  (no skin resolved / held-back spawner mesh never released / palette slot FAR) with
  placement + skin hash + reason; (2) find what the game draws for each [M/D] and fix the
  port; pin with a test (resolved skin + non-FAR palette slot while present); full suite.

- level-14-invisible-guard: the three lanes censused on the real load path this session.
  (1) skin lane: 0 — the 11 raw 0x050000b1 placements (groups 10/12/14/15, p9 26 [D probe])
  resolve through the Drone+0xe0 rewrite to the own-archive 0x0500008c [M 0x14b300:82-95],
  captains to 0x05000093/0x050000b2 (both own-archive [D probe]); the 15 0x05000090
  placements substitute to 0x05000093 [M 0x1be910:33-43] and the session merge carries 90
  from 0700000d anyway; the g-021 load line about b1 is informational. (2) held-back lane:
  the port binds characters to drones by placement index at load; Action::Spawn mutates the
  template record in place (nf-game drone.rs), so every channel-fired wave (ch 1,2,3,9,10,
  11,13 probed; the mission chain 0x47/0x48/0x8C/0x09-walk-on/0x0D/0x54 driven) unhides its
  characters. (3) palette lane: dr.object_pose() Some at every stage for every alive drone.
  NO drone present/enabled/alive without a drawn character reproduced on any drive — pinned
  as crates/nf-view/tests/invisible_guard_07000014_live.rs (2 passed, 10.7 s): the whole
  mission chain asserts bound+unhidden+posed+resolved-skin per alive drone, and the raw
  placement census (11 b1 / 15 90) is pinned [D]. Suite on this branch tree: 191-binary
  background runner + chunks c00-c02 + the new test = 2092 passed / 5 failed / 171 ignored;
  the 5: disc-era identity sectors (slim ISO), walkreal_07000003 + campaign_flow at the
  260 s chunk cap (both pass at full length), 2 no-GPU pylon renders. ~500 small nf-view
  binaries remain unexecuted on THIS tree (the previous session's chunked full suite covered
  the level-03 tree; the branch carries master's ccc714a1 since). Report written.
