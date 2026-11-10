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

- los-mask-repins-0c-2 started: branch cut from s/los-mask-repins-0c (974048c8a) and merged
  origin/master (0e1371451). This round: settle the landing-shot question at the landing with
  strip guard #35 (0x5000087) in AimStandFire (Bond's real fire ray vs the game's bullet test),
  then take the ledgers' next lever (lip-top west of the guard capsule, held slide east past the
  AimStandFire cycle) and un-ignore the 0d-handoff walk, its real_damage sister and the
  eastroute ground crossing. Work in progress.

- los-mask-repins-0c-2 MILESTONE: the landing-shot question SETTLED. The held weapon at the
  #284 landing is the MICRO-CAMERA 0x54 (no round spawns from it [M scene.rs:5002]) — the old
  "dry gun" telemetry read the camera's clip; the P2K (clip 7/41) and the 0x1e were loaded all
  along. The probe (aim_snap_traced + the flight trace) shows the torso-centre ray enters NO
  posed box (the AimStandFire pose displaces the body) and the snap steals onto the wall behind;
  the posed head/torso boxes are exposed instead. The fix (opt-in with_strip_p2k_shot, gated to
  the three tests being un-ignored): walk the ledger's proven west lane off the transient drop
  spot, then the yard stand's aimed shot (0x1e, still-gate) — guard #35 dies (hp -50 [L]) and
  walk_the_whole_mission_from_the_start_to_the_0d_handoff is GREEN under the game's sight
  (14499 ticks, stuck 0, falls 0, objective 3 complete on the 0d arrival).
