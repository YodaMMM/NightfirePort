# s/pickup-pushout-walks

Base: s/pickup-pushout (the dropped-gun push-out, `Pickup_Create` 0x18dda0:166-183) rebased onto
origin/master 2f9f65c57. The base branch moves outcomes beyond the pickup positions: on 15 the
drop of guard #3 carries 5 rounds on the branch and 14 on master, so the shared random stream
is not the same once a drop lands differently. That is the base branch's change. This branch
changes test scripts only (no game code, no draw code).

## Reds the base branch turns red

Every `walkreal_0*_live` binary was run on the rebased base (cargo test, NF_SILENT=1). Reds,
each checked against master 2f9f65c57 in a separate worktree:

| binary | test | master | base branch |
|---|---|---|---|
| 01 | following_mayhew_down_completes_the_level | ok (536 s) | contact killed (level reload) |
| 03 | walk_the_full_route_and_complete_every_objective_real_damage_00agent | ok | Bond dies t151432 |
| 06 | walk_the_castle_interior_to_the_party_real_damage_agent | ok | Bond dies t16413 |
| 06 | walk_the_castle_interior_to_the_party_real_damage_00agent | ok | Bond dies t6825 |
| 11 | walk_spawn_to_the_lower_floor_load_point_real_damage_00agent | ok | Bond dies t7307 |
| 15 | walk_spawn_to_the_omega_sector_exit_real_damage_00agent | ok | Bond dies t15214 |
| 13 | walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent | red, same death t4583 | red (not this branch) |

Every other walkreal binary passed on the base branch.

## What changed and why

- 01 contact follow (`following_mayhew_down_completes_the_level`): guard #8's dropped gun
  (weapon 0x12, the gun in hand) lands against a thin wall face west of the meet-room
  corridor and is pushed to the face's west side (centre (-8.42, 5.34, -1.26); master
  (-8.1, -1.3)). The shared resupply came at it from the east, slid south along the face and
  stalled 1.29 u off for the 3000-tick cap, and the walk unravelled (the contact died in the
  hall). The walk now steps round to the drop's open side, (-9.05, -2.44) then 0.6 u
  west-south-west of the drop, onto it, then waits 20 ticks. Wait sweep 0..200 step 20: 20 and
  80 complete; 0, 40, 60, 100-200 lose the contact.
- 03 00 Agent: the cluster funnel scoop walks a longer tour and Bond reaches the through-gate
  stand 0.02 u off the old spot; the aimed sweep then walked him into #15. A 20-tick wait
  at the stand before the sweep (sweep 10..80: 10, 20, 30, 50, 70, 80 complete).
- 06 Agent: the guardroom pre-clear resupply ends on a stand 0.08 u off; a 30-tick wait after
  the clear (sweep 15..450 step 15: 9 of 30 complete).
- 06 00 Agent: the 0.6 u step onto the #24 solo spot stood still in return fire while
  #24/#25 shot Bond to 0. Waits after the clear (15..450) and before the east strip (15..570)
  found no pass. The step now walks with return fire off (restored after), then the existing
  aimed stand fights from the spot.
- 11 00 Agent: Bond's legs match master tick for tick until the connector-1 descent, where his
  first hit comes 26 ticks earlier; he reached the 18.9 strip with 4 health instead of 14.5.
  A 285-tick wait at the 31.1 strip's east end (sweep 15..600 step 15: 225, 285, 330, 585
  complete).
- 15 00 Agent: not fixed, file unchanged. See below.

## 15 (00 Agent): still red

What was traced (pickup "gone" log, base branch vs master, same walk):
- Guard #1's 0x14 drop (8 rounds) lands at y 38.65 on the branch against 40.50 on master
  and is never in reach (master takes it at t1546). Guard #3's 0x14 drop carries 5 rounds
  on the branch against 14 on master and lies 1.4 u off the nav lane (master takes it at
  t7490). Bond reaches the west room with 0x14 carried rounds 0 against 21 on master, AR
  carried 30 against 38, and 55.1 health against 58.6.
- First death (t15214, -55.08 from a blast): the step-back after the first frag lob stood in
  return fire on #18 2 u east of the threshold. Turning return fire off for that step-back
  gets him past the lobs; he then dies at the west-room solo line or at the door[50]
  threshold.
- Tried, none passes: waits before the lobs (15..315), before the nav lane (10..400) and
  before the solo line (0..200, with return fire on and off); a scoop of the nav-lane drop;
  drawing the P2K for the solo line (the furthest: the room clears at 3.3 health, then #4 at
  door[50]); the two armour vests (both behind doors the walk cannot pass: (22.53, 35.53)
  sits behind the ch_lock 200 door at (25.1, 41.6, 36.6), (12.18, 0.18) wedges at
  (25.78, 0.58)).
- Open: this walk needs a new west-room plan on the branch's drops (more rounds for the solo
  line, or a different first target), not a timing tweak.

## Tests run

All with `CARGO_TARGET_DIR=target-o cargo test --offline --release -p nf-view --features
nf-view/movies,nf-formats/movies`, NF_SILENT=1:
- Base branch, every walkreal_0*_live binary: reds as listed above.
- Master 2f9f65c57: 01 mayhew ok, 03 7/0, 06 9/0, 11 4/0, 15 9/0, 13 3/1 (the same agent red).
- This branch: walkreal_07000001_live 5/0 (3 ignored), walkreal_07000003_live 7/0
  (3 ignored), walkreal_07000006_live 9/0, walkreal_07000011_live 4/0. 15 not re-run (file
  unchanged; red as on the base branch).

Unproven: the levers are measured timings and one route step each. They pass on this tree and
can shift again with any later change to the shared random stream.
