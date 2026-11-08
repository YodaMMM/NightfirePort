# agent-recovery-3: the tests the out-of-cel recovery reddened

Base: origin/s/recovery-loop-07 merged onto origin/master (10956badb). The recovery
(`control_handle_cel_change` 0x133288 default arm) and the `place_bond` floor snap
(`Player_StandAtNewPosition` 0x190d88:104-116) are unchanged: every red below traced to
the test, not the port. No code change; no shared random stream change.

## Per test: what broke and what changed

Re-read 0x133288: the player is class 3 (`Player_Init` 0x1910f8:76), so it skips the
class-2 `Drone_FellOutMap` arm and its MP early return, and takes the default recovery in SP
and MP alike. The MP relink site's missing `find_cel` retry is not a gap:
`player_cel_relink` already ends in `find_cel(pos)`.

| test | cause (traced with `NF_RECOVERY_LOG` and a per-tick trace) | change |
|---|---|---|
| drone_block_swaps first_attack_alt_tail | 5 units in front of the guard is outside every room cel; each raw write was sent back (600 fires) | `place_bond` per tick |
| fix_searchlight_remainder a_courtyard_head | the +80 walk-away write lands outside every cel and is recovered beside the spot, so the head never loses Bond | `place_bond` |
| urn_break sky_bridge | the bridge root sits in the vault, no cel; the raw write was recovered | `fire_at` places; aim reads the snapped stand; bridge spot passed 1.2 lower (the miss arm adds 1.2) |
| fix_blast_scripts the_destroyable_smash | `place_bond` miss arm lifted Bond 1.2 above the box | placement y −1.6, so he stands at the old −0.4 |
| interact_07000006 the_well_shaft | the old band 39.4..40.4 was Bond sunk into the barrel by the raw write; the barrel top is 40.04 (flat ray), the snap stands him at 41.21 and he stays at 41.17..41.23 for 600 ticks | band = barrel top + 1.1 ± 0.2, top pinned 40.04 ± 0.1 |
| interactables_07000002 pickups_are_taken | with the snap Bond really touches P99 ammo #369 and takes it (41 → 70): 41 is below the type-1 cap. The old "refused at the cap" pin only held because the raw write left him under the deck | pool set to the cap: refused, pickup stays; pool back to 41: taken, pickup gone |
| interactables_0700000a qworm_pc_install | `teleport` raw write missed every cel at a candidate stand | `teleport` uses `place_bond` |
| interactables_07000011 plain_door / storeroom | (dx, dz+1.6) by door #13 is solid wall: the raw write put Bond inside the wall where the E still reached the door. With the snap he is pushed out of reach. The storeroom leaf swept into Bond standing in its arc | placements use `place_bond`; the plain door is used from the corridor floor west of the leaf, facing east; the storeroom wait stands on the plate, then places in front for the press |
| net_resync_live | 0 recovery fires; the 120 s wall-clock deadline ran out under machine load | unchanged; passes when run alone (50 s) |
| probe_07_route | no raw writes, a real walk. At tick 1708-1709 the bot capsule push (the airborne-only push in `Bond` collide, player.rs) shoves Bond 0.72 then 0.87 units, through the hall's north-west jamb, into a solid pocket with no floor. On master he then falls to y −115 and the probe still "arrives" by xz; with the recovery he is held under the floor at the cel's low edge (5488 fires) and the walker hits its cap | the legs after the E run lenient (the probe has no asserts) |

## The other raw `bond.pos` writes

295 test files write `bond.pos` raw. A scratch scan took every literal target (430 rows),
poked Bond there from the level start through the real step, and counted recovery fires over
40 ticks. Fires on every level the file names: 5 rows.
- void_recovery_07000003 ×2: on purpose (the poke tests).
- interact_07000009:279 (y −377 shutter): 1 fire in-test; the test passes and the assert
  does not depend on the stand.
- interactables_07000015:572/610 (ladder walkway): 8 fires in-test. The test passes, but
  Bond is under the walkway (y ≈ 39.3), not on it. Moved to `place_bond`, the snap stands him
  at 41.47 over the sloped floor (40.37), then something pushes him to 42.39 within 10 ticks
  and both ladder tests fail. Left raw: which surface lifts him is not settled.
354 computed targets (door/guard/pickup offsets) were not scanned.

## Results

- The ten binaries: drone_block_swaps 3/3, fix_blast_scripts 3/3, fix_searchlight_remainder
  3/3, interact_07000006 9/9, interactables_07000002 18/18, interactables_0700000a 13/13,
  interactables_07000011 16/16, urn_break 4/4, probe_07_route 1/1. net_resync_live passed
  3/3 once (50 s). On reruns, with the CPU at 100%, a different test missed its wall-clock
  pump deadline each time.
- Every walkreal_* binary (107) is green. With `NF_RECOVERY_LOG=1`, two have fires:
  walkreal_07000008 (7553): in `the_ally_never_shoots_bond_on_the_walk` the lenient walk
  steps off the walkway at (143.7, 46.1, −105.2) and drops to cel 12's low edge (y ≈ −33).
  On master he falls without end; now the recovery holds him there. The asserts do not depend
  on where he is. walkreal_probe03 (2): the 07000003 void edge, which is the measured
  recovery spot.
- interactables_07000002 `the_kitchen_camera_sees_bond_in_its_cone`: 11 fires, test green.
- Not run: the full suite.

## Unproven

- Whether the bot capsule push can shove the player through a wall in the game (the push
  is an [S] port choice: airborne only, no wall re-check). That is the 07 hall bug, and it
  happens on master too.
- The 07000015 ladder stand: what lifts Bond to 42.39.
- Whether the game kills or catches a player who drops off the 07000008 walkway into the
  space under it.
