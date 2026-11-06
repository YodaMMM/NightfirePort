# WORKLOG

## guard-repin-master
Started 2026-11-06 10:05 UTC. Branch s/guard-repin-master (from origin/s/guard-on-master
6f8ce18bb, the guard radii + glass work on current master, full suite 4417/5). Task: triage
the five reds (03 real_damage_agent death 51805; 13 warm-chain "the lobby west mid" 6000-cap
+ a 2627 death at (208.62,1.04,-229.64); 0d c5 door "the blind strip east 4 must walk"; 06
castle interior death 5217 at (20.27,41.13,-37.97); 08 whole-mission 00agent "the shoot-out
must kill every end pane") — for each, walk-tuned-to-old-positions vs guard-behaves-wrongly.
Route/timing levers only; no [L] loosening, no ignores, no game-code changes to suit a walk.
Finish battery: the five binaries whole + walkreal 01, 16, 1b + enemy_accuracy_live.
  Results (2026-11-06 11:45 UTC): five reds triaged on the measured ledgers. Verdicts: every
  red is walk-tuning to the OLD guard behaviour (the 2.0 stops, the glass shield) — no game
  code changed, no [L] loosened, no ignore added. FIXED: 06 (the file's d2 aimed-stand law
  extended to the d1 real-damage walk via a stands param; both sisters green; the binary
  6/1 with the 1 its pre-existing 00agent ignore) and 08's pane red (the cabin fights' ammo
  drift left the clip dry — pane 2's five pulls were clicks; w.reload_full() before the
  shoot-out → 0 end panes glazed). 13: the lobby re-anchor (2.5, 30.5) FIXED the 6000-cap
  (the walk passed the west-mid for the first time) but the ~5000-tick timeline shift moved
  #5's 28.0 into the budget and the room guard #8's point-blank 31.5 killed the walk at
  3006 — the re-anchor-end and doorway sweeps measured byte-identical (rays never open);
  the cap is fixed, the death remains. 0d: the strip-mouth #10 kill dwell 600→180 (the kill
  itself no longer lands — 600 converged ray-clear shots at d 63.6 scored zero, itself a
  look-item) moved the d1 red from the 2627 strip death to a 9300 drop death (the transit
  survives); the bay-sniper class (0x1e at d 18-61 through the bay glass) is unanswered.
  03: the far shooters (d 30-40) outside the plain 20 u window drain the servant/exit legs
  (#11 56.0 in 5 at d 40.2 through glass); the 06 stand_shooters_down law is the proposed
  lever, not landed. 08's station red remains (the branch's own diff predicted it: the
  #23/#19 top-station volleys at d 59-97; death 27596 to a hurt volume after the drain).
  The 2627 death at (208.62,1.04,-229.64) belongs to 0d's walk_arrival_to_the_c5_door_real_
  damage, not 13. Battery (--include-ignored, whole): 06 6/1 (33.3s), 13 2/1 (48.7s),
  0d 1/2 (45.6s), 08 5/1 (194.5s), 03 5/2 (752.2s), 01 8/0 (56.6s); 16/1b/enemy_accuracy
  were still running at report time — their counts are in the report tail if landed.
  Report: docs/reports/guard-repin-master.md.

## guard-repin-master-2
Started 2026-11-06 11:20 UTC. Branch s/guard-repin-master-2 (from origin/s/guard-repin-master,
already up to date with origin/master). Task: the last three reds — 03 real_damage_agent (land
the stand_shooters_down leg-end sweeps from 06 on the servant and exit legs, file-local, d-gated);
13 warm chain real_damage (restore the lobby budget; #8 at the security doorway met Bond at 21 hp);
0d c5 door real_damage and agent (change the "zero-hit while moving" promise to a measured budget,
explain the 600 zero ticks at d 63.6); 08 whole mission real_damage_00agent (station legs re-pin
the declared branch, else measured ignore for the 00 Agent sister only). No [L] loosening beyond
item 3's, no game-code changes to suit a walk. Finish binaries: those whole plus walkreal 01, 06,
16, 1b, enemy_accuracy_live.
