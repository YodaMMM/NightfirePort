# drone-bump-collision-6 — the bot-side move model at the 07 west mouth

Branch `s/drone-bump-collision-6` = `origin/s/drone-bump-collision-5` (cbd1c5c1) merged with
`origin/master` (17aa36ab), plus the route-walk move change below (964ec440). The push was never
disarmed; the Jointy push, the arming chain and the door logic are untouched.

## What was ported (the game's route-walk move)

Round-5's verdict, now implemented in `crates/nf-game/src/drone.rs`: the port's `probe_move`
world-collision stand-in has no counterpart on the game's route-walk path. The game's model, all
[M]:

- **Root motion is applied unconditionally** — `AnimFrameResolve` 0x1c35f8:425-432 adds the
  PreTransCallback's vector to `obj+0x30` with no world test of any kind.
- **The only horizontal constraint is the boundary-flag per-axis clamp** — `DroneMove_SetBoundryFlags`
  0x14f140 (runs in `NDrone2_ControlSTANDARD` 0x1490f0:202-206) probes the four object-local axis
  directions (0,0,±0.5)/(±0.5,0,0) from the feet with `NDrone2_MoveTest` 0x155330 — the distance/slope
  gates + `AINetwork_BoundsTest`, **nav boundaries only, no collision-world LOS** — and sets +0x320
  bits 2/4/8/0x10; `DroneAnim_PreTransCallback` 0x13b490:120-136 then transforms the root step to
  object-local, zeroes each component pointing into a blocked direction (the differential slide along
  a wall) and transforms back.
- **The whole clamp is skipped while the door target +0xBA8 stands** [M 0x13b490:120
  `&& *(int *)(iVar1 + 0xba8) == 0`; the same skip in `Drone_Control` 0x139268:76 and
  `LinkCreep_Handler` 0x15a2c8:67] — the bot walks the door band.
- **The post-hoc corrections already ported**: the nav-bounds push (`NDrone2_Collision` 0x140e90:170
  → `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0; the port's `push_out` fold, drone.rs tick) and
  the bot-yielding rules (`Collide_Jointy` 0x1ec180:229-233 — SP returns 0 for a class-2 mover vs a
  class-3 other: the bot never resolves against the player; bot-vs-bot push is MP-only).

The port change (drone.rs, `root_motion` + new `boundary_clamp`): when the drone is following a route
(`route.is_some()`) in a move anim (RUN 0x1e / ALERT_WALK 0x35 / WALK_AIM 0x0f / RUN_AIM 0x08 /
0x37..=0x39), the walk step now goes through the game's own constraint — the four nav-only probes
(the port's game-true shape is `nav::Network::node_scan_ok`, which is exactly `NDrone2_MoveTest`
without the [S] LOS deviation), the blocked per-axis components zeroed on the OBJECT-LOCAL step, the
skips while `hit_door` (+0xBA8) stands or the drone is unbounded — and the remaining step applies
ungated; the post-tick `push_out` keeps the nav bounds. `blocked_reach` keeps move_by's two
observables (body put = the full step absorbed, for the pane drain). Combat moves (strafe/step anims)
and non-route anims keep `probe_move` — the documented walls-990 fold — so the stuck-guard-combat
consistency (choice probe = mover test) is preserved.

[S] marks carried in the code: the 0.5 probe is used unscaled (0x14f140 scales it by the mode-0x1E
bot's `BOT_getMovementSpeedMul`, not measured for these bots; 1.0 assumed).

## How it was verified

- `probe_drone5_mouth` (the round-5 anatomy probe, rerun on this tree): dr22 crosses the west mouth —
  `[aside t100] dr22 (79.34,-50.27)` — past the old park (79.48,−50.32) within ~100 ticks of the wake
  exactly as round-5 measured the game's crossing, door3 0x02→0x04 on the way, and **completes her
  15-node ring** (`[aside t1000] route path3 idx15/15`, settled at (66.27,−50.42), state 0x11). The
  body now follows the creep the whole way; the round-5 "park + creep race" is gone.
- The four red tests' west-mouth legs, previously lenient give-ups or wedges, now all **arrive**:
  `the hall west mouth` (+75 ticks), `the hall floor`, `the hall east` (+125) in probe_07_route's
  trace; walkreal_07000007_live `the_group_plate_wakes_the_walkers_and_the_guards_stay_calm` passes.
- Build: `cargo build --release -p nf-game` clean; the touched crate's tests were run via the four
  test binaries + the two probes directly (quoted below); the full suite result is under Suite.

## The remaining red: the east-door wedge (every red accounted)

After the mouth fix, all four still-red tests wedge at the **same new spot** — `E the hall door`
(the east door (94.9,−47.2)), which round-5's runs also show as their failure label once the mouth
legs gave up leniently. Diagnosis, all measured on this tree:

- Two bots stand in the doorway's south band: d10 boot (94.89,−48.00), d19 boot (93.66,−47.98)
  [D the placement dump]. Both are **CIVILIAN_DOOR_GUARD (state 0x31)** [M DroneTypeSettings row mode
  0x16 start 49, [D] the ACTION.ELF type table dumped this round] and shuffle ±0.15 around
  (94.75,−48.05)/(93.78,−48.00) for the whole run (the `NF_DRONE_WATCH` trace) — they never leave.
- The game-true Jointy push [M 0x1ec180:229-233] pins Bond at the 0.95 r-sum contact. With d10 at
  (94.75,−48.05) her contact ring covers the door approach disk (the leg's target (94.6,−47.4) is
  0.65 from her); the gap between the two guards is 0.97 < 2×0.95, and east of d10 the ring edge
  (x 95.7) is past the jamb — **no passage exists while they stand there**. `segment_to_target=None`
  in the stuck dumps: the blocker is purely the two capsules, not world geometry.
- The door is opened by the walker ("opening door … state 0x2" in the nav leg) but the leaf swings
  away from the activator [M `Door_SetupSwing`'s side·swing, door.rs `swing_from_side`] — into the
  corridor — so no swing-cell jostle clears d10 either; and the port's bots have no generic
  cylinder-vs-world depenetration (the game's `Collide_Update` 0x1ea398 push) that would move a bot
  the leaf does touch.
- The door guards' own reaction system never fires in the port: d10/d19 stay in 0x31 with
  `near_armed_person` (the UPDATE gate [M 0x160648:39-57], which would send them to ATTACK/zone
  behaviour) never triggering at 0.9 u — the port's gate needs the reaction-acked sight
  (`+0x228 & 4`, `DroneFunc_ReactionTime`) and the guards' vision never acks Bond's approach arc in
  these runs. In the game, a zone-1/2/3 reaction (face / challenge / the 0xc fail + Attack) is the
  observable answer to an armed player at the door — which also moves the guard.

A test-side approach fix (an intermediate nav waypoint north of the guards, mirroring the tests' own
manual "hug x ≤ 92.8" legs) was tried and **reverted unwound**: the nav steer wedges at the same pin
before it can reach the north arc, and the change was not evidence-backed enough to ship. The five
reds therefore stay red at the east door, with the mouth half of each walk now genuinely green.

## What's left

1. The live checks below — they settle what actually clears the east doorway in the game (the
   guards' reaction, a bigger idle shuffle, a different passage, or a depenetration push).
2. Porting that mechanism, then the four tests' east legs.
3. A full-suite run that reaches nf-view (this window's run gets through the earlier crates only).

## Live checks for the main agent

Fresh boot (a save-state resume skips the boot path), level 07000007, walk Bond at the east door
(94.9,−47.2) from the hall:

1. **Do the door guards react?** gdbwatch d10/d19 (boot (94.89,−48.00)/(93.66,−47.98)) — record
   state (+0x58), the zone step, position (+0x30..+0x38) and `+0x228` while Bond closes from 3 u to
   contact. Expected if the zone system is the answer: 0x31 → zone 1/2 (face/challenge, anim 0x5f)
   → zone 3 (Attack) — i.e. they leave the doorway. If nothing changes even at contact, the game's
   player is physically pinned too and the route probe's premise (this door as a walkable shortcut)
   needs the disc's own answer.
2. **The doorway's real gap.** fieldprobe the east door at pos01=0 and pos01=1: the leaf quad's
   closed/open poses and the jamb boxes. My scan says d10's contact ring overlaps the door plane, so
   I need the actual leaf arc — does the open leaf lie across Bond's approach at all?
3. **A 30 s idle watch** of d10 with Bond far away: how far does the 0x24→0x25→0x26 idle cycle walk
   her (root motion range)? If the game's shuffle reaches ±0.5+, the doorway opens by itself on a
   timer and the port's ±0.15 is the port's probe/boundary interplay shrinking it.
4. A render at `--bond 95.4,44.1,-48.6` facing north-west (yaw ≈ 2.4 rad in the port's atan2(x,z)
   convention): what stands between that spot and the door — walls, the leaf, the two guards?

## Suite

The fast runner (`tools/test/suite.py`, background, this tree merged with origin/master) was
restarted after the first background job was killed between tool calls; the window ends before it
reaches nf-view. Direct runs on this tree (cargo test --release --test-threads=1, nf-view):
probe_drone5_mouth **ok**, probe_07_route **FAILED** (the east wedge), party_door_lock_07000007_live
**FAILED ×2** (the east wedge), cutscene_timer_pause_live **FAILED** (the east wedge),
walkreal_07000007_live **1 passed / 1 FAILED** (the east wedge). Final suite numbers will be filled
from the runner's total line if it lands in the window.

## Usage

not visible to me
