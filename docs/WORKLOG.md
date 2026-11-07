# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## real-health-00agent-06-4

2026-11-07 06:10 UTC, branch `s/real-health-00agent-06-4` (from origin/master 22d5836b7).

Task: the 06 00 Agent walk keeps its sister ignored (`walk_the_castle_interior_to_the_party_
real_damage_00agent`, ignore text names this follow-up). Read `done/50-real-health-00agent-06-3.md`
(queue) and take its levers on current master, real input only, helpers level-local. Done =
green with the ignore removed, or ignored with a measured reason. If the report's open question
decides it (the deck posts leaving their posts), say what needs measured.

- [x] setup: repo/work/ISO/queue worktree reused from the sandbox, branch created from
  origin/master 22d5836b7 and pushed.
- [06:15 UTC] Baseline measured on current master: the sister dies at tick 6435 at the strip
  arrive (5.97, 42.03, −32.77) — 25 hits for 100.0: #11 (tunnel roof, (8.3, 55.5, −51.1))
  36.8 in 7, #4 (guardhouse court) 33.0 in 12, #19 (ch-55 pair, (17.7, 41.0, −37.2)) 27.5 in 5,
  #16 2.8. Same lane as 06-3's tick 5879. Lever 1 (the census probe) goes in next.
- [06:20-07:25 UTC] Ten measured runs (each on a rebuilt binary, frame-stamped censuses at
  every phase boundary). Findings: the pair (#19/#20) spawns dormant at (22.3,−39.1)/(24.1,−39.6),
  enables when ch55 rises, walks west, rays open only inside d ~13.5 and they pause at d 11.8-12.9
  of Bond's spot — INSIDE their own 12 u sight [D props]; the engagement at the pause cost 60.4
  (run 2), at the r1 edge 16.5 (run 4), mid-walk never tapped (06-3's own state). The deck posts
  die to the balcony stands directly (06-3's open question moot on this master — no chase-fall
  needed [L run 2's census]). The kite idea measured a NO-OP twice: [M player.rs:1006/1671] the
  aim stance forces forward to 0, and shoot_bounded is the walker's own flank loop that never
  reads DamageLog::back_off. The ch55 cone dodge measured NEGATIVE (the landing (4.4,−23.0) is
  swept by head ch63 at (7.13,47.57,−23.60) ~120-240 ticks after the drop — no lane avoids the
  catch, the heads' geometry dumped in the log). The pre-door fight_through measured WORSE (its
  resupply walks displaced the nook nav into a 6000-tick wedge at the hedge). The conditional
  "first r1 past d 12" watch measured WORSE (broke 84 ticks early, #20 charged to d 9.4 and
  unloaded 64.2). LANDINGS: the fixed 600 watch + patience re-picks (the posts' lean-outs) +
  the 600 stand cap at 00 + the strip patience hold: the pair dies for 16.5, #11 dies for 11.0,
  #24 dies to the ledge tier (run 8; run 10 he ends hp 7), and the walk's death moved from the
  strip arrive t5879 (06-3) to the east-strip arrive t8893 with 100.0 taken — the remaining
  leak is #4's guardroom melee (12 × 2.75 every run, no lever) and #24's east lane (33.0 in 6).
  The d2 Agent walk broke once (the 600 cap + the strip hold are shared-tier) and is restored
  byte-identical (13031 ticks, 3.7 end hp — the master ledger). Whole 06 binary: 6 passed /
  0 failed / 1 ignored (the ignore is the measured 00 sister). Test-only diff: the 06 test
  file + WORKLOG; the lesson-10 set (the 06 binary) is green; the full suite did not fit the
  window (same as 06-3).
