
## level-02-captors-real-approach

- (15:10 UTC, 10-30) Branch created from origin/master (f9d1971d). Task: replace the
  `clear_captors` teleport in walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit
  (the helper stands Bond 2.5 units beside each guard via bond.pos) with a real-input
  approach — walk the nav web to a firing spot with line of sight, aim and fire through the
  real path; the walk must stay green with NO bond.pos writes after the spawn; fix any port
  gap it exposes with [M/D] evidence; quote the touched binaries' totals. Sandbox kept
  repo/ISO/work/target/nf-queue from the level-0c-plates-reach-2 session.

- (15:25 UTC, 10-30) LANDED: clear_captors's bond.pos teleport is gone — the helper walks the
  nav web into the captor room (round 0 dest = the geisha's spot, later rounds the guard's
  current pos, arrive 3.5) and fires through the real aim loop; the shoot-side flank/reposition
  arms (walker.rs:640-700) were already real-input. walkreal_07000002_live 1 passed 0 failed
  (27124 ticks, falls []): geisha 1's captors approached via 3+1 nav nodes and killed at dist
  7.1 / 1.2, geisha 2's three captors via 9+2+1 nodes at 1.8 / 1.2 / 0.9 — no survivors, no
  flanks needed, no port gap exposed. Independent audit (subagent): 5/5 PASS — no bond.pos
  write reachable from the walk (walker.rs's 30 bond.pos uses are all reads), diff = 1 test
  file only, WORKLOG append-only, the ch asserts untouched. Suite started 15:15 UTC on this
  tree (the final tree: master + this change).
