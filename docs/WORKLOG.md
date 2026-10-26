
## level-0a-band-settle

- (14:07 UTC, 10-26) Branch created from origin/master (7e35bd77). Task: un-ignore the three
  walkreal_0700000a_live tests that pin the doorway base climb's +0.0256 band settle [L 1.1147],
  the corner walk-through (y 1.127 -> 1.120 [L]) where the port wedges at the band edge 1.1347,
  and the ring walk (:227) wedge at 'the ai lane to the N-S corridor' — find the game's
  step/ground-settle rule on band edges (Player_Update ground probe + step-up [M]), port it,
  keep every other walkreal test green. Sandbox kept repo/ISO/work/target from the previous
  session; branch pushed as checkpoint.
