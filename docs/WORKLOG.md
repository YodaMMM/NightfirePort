
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
