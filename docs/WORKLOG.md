# WORKLOG

## los-mask-repins-0c (2026-11-10)

- Branch from origin/master 114737d6d. Scope: the three 0700000c walks the game-correct guard
  sight moved red (walk_the_whole_mission_from_the_start_to_the_0d_handoff,
  walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage,
  the_ground_route_crosses_to_the_east_side) — read each ignore, check the round-3 note
  (the west-ledge strip guard: falls into the gap going round him; landing shots do not hit
  him — verify game-correct before re-routing), re-route with real input, remove each ignore.
  Helpers gated to 0700000c + difficulty. Worklog started, checkpoint 1 pushed.

## los-mask-repins-0c (2026-11-10, round 2)

- The 0c-diag dump landed (the guard's in-sim position/state + the deep map). Repairs tried
  and reverted with measurements: the -4 lip route's slide now falls to the yard (sight-era
  guard footwork), the 11.24 lattice south is thin wall tops. Ledgers updated on all three
  walks with the mechanism + the next lever. Required binaries green (handoff 1/0/3,
  barrelrow 1/0, interactables 7/0, the three walkreal0c consumers 2/0 1/0 2/0). Report:
  docs/reports/los-mask-repins-0c.md.

## los-mask-repins-0c (2026-11-10, round 2)

- The 0c-diag dump landed (the guard's in-sim position/state + the deep map). Repairs tried
  and reverted with measurements: the -4 lip route's slide now falls to the yard (sight-era
  guard footwork), the 11.24 lattice south is thin wall tops. Ledgers updated on all three
  walks with the mechanism + the next lever. Required binaries green (handoff 1/0/3,
  barrelrow 1/0, interactables 7/0, the three walkreal0c consumers 2/0 1/0 2/0). Report:
  docs/reports/los-mask-repins-0c.md.

## los-mask-repins-0c (2026-11-10, round 2 addendum)

- The round-3 note ANSWERED: the landing shots were fired DRY (clip 1 off-target, then clip 0
  spare 0 [L the telemetry run]); the world rays (eye + chest, every offset) are ALL clear
  [L the 0c-diag run]. The miss was the ammo, not the world. Next lever: resupply before the
  #284 ride, the on-target shot kills the 10-hp guard, the straight diagonals walk clean.
  Ledgers + report updated; all 0c binaries green (handoff 1/0/3, eastroute 1 ignored,
  barrelrow 1/0, interactables 7/0, guard1 2/0, comms 1/0, upward 2/0), nf-game lib 782/0/4.

- The fetch attempt measured: the west pickup (0x0003/0x0006/50 rds at (-21.53,-218.54)) was
  reached but the wheel stayed 0x54/1/0 after the cycle [L the run]; the re-anchor walk
  landed a lower perch. The ledgers carry the equip-path question. All 0c binaries green
  (1/0/3, 1 ign, 1/0, 7/0, 2/0, 1/0, 2/0), nf-game lib 782/0/4 on the final tree.
