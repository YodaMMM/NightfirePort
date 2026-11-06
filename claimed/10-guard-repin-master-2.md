# Guard radii and glass: the last three reds

Branch from `origin/s/guard-repin-master`, merge current `origin/master`, push to `s/guard-repin-master-2`. Read docs/reports/guard-repin-master.md. 06 and the 13 cap and the 08 pane are fixed.

Measured in PCSX2 on 0700000d, Bond held at (208.62, 1.04, -229.64) for 45 s: 14 hits on Bond, and single rounds crossing class 0x1d objects at (163.9, 7.2), (173.9, 6.2), (189.5, 4.6), (200.1, 3.6) and flying on along one line. So guard rounds through glass is game-true, and the strip is not a zero-hit lane in the game. Those rounds came from the west (x < 163, z about -235), not from the bay posts your report named, so compare the port's shooters on that strip with that line.

Left:
1. 03 real_damage_agent: land the `stand_shooters_down` leg-end sweeps from 06 on the servant and exit legs, file-local, d-gated.
2. 13 warm chain real_damage: restore the lobby budget; #8 at the security doorway met Bond at 21 hp.
3. 0d c5 door real_damage and agent: a test promise of "zero-hit while moving" cannot stand, so change it to a real budget for that lane, with the measurement cited. Also explain "600 ticks of converged, ray-clear shots at d 63.6 scored zero"; if a guard's box or position is wrong that is a bug, not a walk problem.
4. 08 whole mission real_damage_00agent: the station legs re-pin the branch declared. If it cannot be done, an ignore with a measured reason is allowed for the 00 Agent sister only.

Do not loosen pins tagged [L] other than item 3's, and do not change game code to suit a walk. Finish with those binaries whole plus walkreal 01, 06, 16, 1b and enemy_accuracy_live.

---
claimed-by: sandbox5 at 2026-11-06T11:12Z
