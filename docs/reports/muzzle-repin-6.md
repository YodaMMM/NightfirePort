# muzzle-repin-6 — the last reds on the merged master

Branch `s/muzzle-repin-6`, cut from `origin/s/muzzle-repin-5` and merged current `origin/master`
(one conflict: `walkreal_07000003_live`'s keep_off — master's 2ea70f9d2 arms it at difficulty 1;
master's change taken, the pinned 4.0 stands, not loosened). Game code untouched: **every draw
stays** — the diff is two test files and docs. No pin tagged [L] was loosened. All fixes are
test-side, real input (walking, backpedaling, E, real trigger), no teleports, no health/ammo
writes.

## The three named reds

1. **walkreal_07000015_live — FIXED: 6 passed / 0 failed / 1 ignored (whole binary,
   --test-threads=1, slim ISO).** Both real-health reds:
   - The d1 red was **#462 (the silo beam), not #447**: `nav_until_clear` cleared the line, then
     the guard-kill fight (`w.shoot(0)`) moved Bond behind a ~0.5 u clutter hit (the eye→beam ray
     blocked at fraction 0.0376 for all 3480 ticks while the aim sat dead-on; 22 rounds + 261 dry
     pulls died in the near clutter). Fix: re-walk the same nav lane until the line is clear
     again after the fight.
   - The d2 red at **#447**: the mark walk turned Bond ~π and the aim turn outlived door[42]'s
     360-tick auto-close [M door.rs AUTO_CLOSE 0x13660c-0x136700] (`los None` at t0 →
     `los Some(0.271)` — 0.271 × 10.8 u ≈ the z 25.06 leaf plane [M world.rs segment fraction]),
     10 rounds dry, mine intact. Fix: a per-file `backpedal_face` — the game's own backward walk
     (forward −1) with the view held on the beam, so the trigger is on-target from the first gate
     tick, ~300 ticks inside the door's window. Real-health runs only; the master (top-up) walk's
     stream is byte-identical.
   - The deeper teacher: the P2K's pick-relevant record has **spread 0.0** [L ee_start_05.bin
     weapon_data 0x02 +0x24/+0x98, flags 0x1004 — no 0x400000, no 0x10000 → the shot rides the
     camera along the view], but the round **arcs under gravity** [M Bullet_update case 1] and
     #447's beam pick centre rides at the record y **44.217** while the census aim sat at y 42.3 —
     ~30 spread-free, dead-on-eye, clear-line rounds passed ~2.3 u under the pick and the fence
     never took one. `shoot_mine` now **brackets like a player**: each live round's closest
     approach to the pick centre is tracked; a round that dies without the boom on a clear line
     hands its miss vector to the aim (one correction lands the next round; ≤ 4). The other five
     mines pop on their first pull as before (the RNG draw count per shot is path-independent, so
     their streams are unchanged).
   - The d1 fence also went down on the game's own law — a corridor guard's contact
     (MINE_DRONE 4) set a neighbour off and the case-0x4B chain took #447 [M Mine_Update
     0x17c930's contact walk; Explode_Propagate 0x17c518] — so the neighbour assert now accepts
     any exploded cause (the fence's job: down before the crossing, never on BOND's body — that
     assert stays).
   - The d2 fights needed the 02/03 walks' **keep_off/back_off 4.0** (gated difficulty ≥ 2; the
     measured run carried 24 hits for 100.0 across 16 return-fire fights and died at door[51]).
2. **walkreal_07000008_live — the named red FIXED, one new downstream shape: 5 passed / 1
   failed.** The 00agent run now SURVIVES to the shoot-out (the merge's master levers already
   fixed the mid-walk death) and **all 10 end panes are shot out**: (a) the d3 loot rule — the 02
   walk's `scoop_drops` + `arm_fullest` (launcher excluded), gated difficulty ≥ 3 — re-arms Bond
   from the end-fight drops (the measured run reached the shoot-out with **clip 3 spare 0**, pane
   0 ate the last rounds and 6-8 panes walked dry); (b) the pane loop brackets the round's
   measured miss (the #447 lesson); (c) a round dying ≥ 1 u out marks the line BLOCKED (a
   mullion, not a near miss) and a sidestep + fresh bracket window re-takes the sight line.
   **What's left here:** after the panes the run dies later in the copter leg
   (walker.rs:283 real-damage death — the ledger is in the suite log; next session starts from
   the death's last-hits block).
3. **start_corridor_guard_07000011 d2 death assert — NOT touched this session** (the clock went
   to the 15/08 fixes and the suite batches). It stands as muzzle-repin-5 left it: guards hold
   0x5c at their corridor spots but `seen_ticks 0, lost_sight true` from the mid-fight on — the
   report's own read (a live check rather than a test lever) still stands, and its live-check
   request (below) carries forward.

## walkreal_07000003_live — re-run whole as ordered

The binary ran in the verification batch: the **full-route test
`walk_the_full_route_and_complete_every_objective_real_damage` passed ok**, the difficulty-2
sister passed earlier in that log, and the 00agent sister carries its pre-existing documented
ignore (the east captor-room cluster under the door routes — the ignore text lists its own next
levers). The tool's 10-minute cap killed the batch during 08's link, so the 03 binary's own
one-line summary did not print; the test lines above are quoted from the captured log
(/tmp/b_wcrit.txt, `walk_the_full_route_and_complete_every_objective_real_damage ... ok`).

## The suite (one total line, and exactly what ran)

The four-package command does not fit a sandbox tool call, so it ran as batches with
`--no-fail-fast --test-threads=1` on the slim ISO; one linker SIGBUS (disk 94% full) was fixed by
deleting 1116 stale pre-merge artifacts from `target/release/deps`. Everything that produced a
summary line this session:

| batch | result |
| --- | --- |
| nf-formats + nf-audio (53 targets) | **233 passed / 4 failed / 3 ignored** |
| nf-game (107 targets, lib + binaries) | **1095 passed / 0 failed / 8 ignored** |
| walkreal_07000015_live (whole) | **6 / 0 / 1 ignored** — fixed |
| walkreal_07000002_live (whole) | **3 / 0 / 1 ignored** — the 5's fix holds |
| walkreal_07000008_live (whole) | **5 / 1** — the pane red fixed, the new copter-leg death |
| walkreal_07000003_live (whole, killed at the cap mid-binary) | the full-route + d2 tests ok, the 00agent ignore pre-existing |

**Total over the batches that completed: 1342 passed / 5 failed / 13 ignored.** The 5 failures:
disc_golden's 3 (the slim ISO's allowed three) + `identity_disc`'s
`the_real_disc_passes_the_identity_check` (0.01 s — disc-identity on the slim ISO; needs the
full-disc check or a driver look) + walkreal_07000008's new copter-leg death.

**Not re-run this session:** the rest of nf-view's ~700 binaries (the sandbox tool's 10-minute
cap × fresh links after the stale-artifact cleanup, and the 11:29 stop). The last full
accounting for them is muzzle-repin-5's table plus what master merged since (06 walk levers, the
11 00 Agent walk on real health, the landing/steps water read — all on master before my branch
cut).

## What's left (for the next message, from this branch)

1. walkreal_07000008's 00agent copter-leg death (the panes are green; the death's ledger is in
   /tmp/b_wcrit.txt — copy it before the sandbox recycles if needed, else re-run the one test).
2. start_corridor d2 (the live-check path, below).
3. The full nf-view re-run (the batches above define the shape; alphabetical chunks of ~10
   binaries fit the tool cap once the links are warm).

## Live checks for the main agent

- Carried forward from muzzle-repin-5: gdbwatch on 07000011 d2, guard #126's `+0x1e`
  (GunThroughWall) and the sight flags during t1500–t3500 while Bond idles at the corridor start
  — which side moves (the port's latched lost_sight vs the game's keep-and-kill [L]) settles
  whether the d2 red is a port wedge or a test lever.
- New: none this session — the two fixed reds were settled by the port's own measurements (the
  fraction semantics, the pick centre, the auto-close timer), no disc probe needed.

## Usage

not visible to me
