# drone-bump-collision-5 — the 07 party hall's west mouth with the push on

Branch `s/drone-bump-collision-5` = `origin/master` (ae4d2894) merged with `origin/s/drone-bump-collision-4`.
Task: find what the game's bot does at the west mouth (the kind-8 route + the door's swing/collision, the
shared-RNG phase the merge moved) and give the tests the player's real answer — wait the bot through, step
aside, take the measured timing, with cites. The push was never disarmed.

## What the mouth is (the anatomy)

- The west mouth is path3's ring. Disc nav [D the MAPNODE/MAPLINK dump, `probe_drone5_mouth.rs`]:
  node0 (79.75,−50.45) and node6 (80.25,−50.44) carry flags 0x2 (the door pair) and straddle the free
  door at (80.0,−49.9); links 11→0, 0→6 (len 0.50), 6→7, and 10→14→13→12→11 close the ring
  (node11 (78.27,−50.50), 12 (77.83,−50.27), 13 (77.48,−49.77), 14 (77.16,−47.80)).
- dr22 boots east of the door at (80.71,−50.47); its kind-8 route is the 15-node ring
  [M InitAIPath 0x153720 chain — the kind&8 open-chain walk]. The door-flagged pair stands her in
  DSTATE_OpenDoor [M ReachedDestNode 0x153358:149-163]; the state opens door3 herself and waits
  DoorIsOpen [M OpenDoor 0x1681f8:18-67, cases 1/0xc]; the ring continues west [D the links].
- Port measurement (deterministic sim, `probe_drone5_mouth.rs`): she wakes (state 0x16), opens the door
  (0x02→0x04) and crosses west to (79.48,−50.32) within ~100 ticks of the wake — then PARKS there while
  `Route::follow`'s creep races ahead to the leg 13→14 (creep a=(77.48,−49.77) b=(77.16,−47.80)). Her
  target node 13 has CLEAR world LOS from the parked spot; the port's `probe_move` stand-in
  [S drone.rs `move_by`/`probe_move`] rejects the body's step, so the body never follows the creep.
  The game's root motion has NO world-collision gate — `DroneAnim_PreTransCallback` 0x13b490 applies it
  and the only horizontal constraint is `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0 [M, the
  walls-990 note] — so in the game she walks through and clears the mouth.

## What the player can actually do (the measured geometry)

`probe_drone5_gap.rs` walks Bond to the mouth and radial-scans the world (24 rays × 3 m) while stepping
east. From the pin (79.20,−51.25):

- Only the 45..75° rays reach the hall (through the door's south side); the north arc (0..30°) is
  blocked by the open leaf / north jamb; pure east (90°) hits the south-wall return at the door plane.
- dr22's capsule at (79.51,−50.32) covers z −50.82..−49.82 of the opening; the clear band south of her
  is ~0.1 m. **Nothing body-wide passes while she is parked.**
- The Jointy push [M Collide_Jointy 0x1ec180:229-266] pins Bond at the 0.95 contact: the direct east
  aim sits 0.49 INTO the capsule's radial half-space and the mouth's wall absorbs the tangential —
  400 ticks of forward 1.0 produce zero displacement (the trap trace, this round).

## What the walker now does (the player's beats, committed ba0ac28f)

The doorway anti-trap (opt-in `crowd_aware`) plays the three beats with cites in the code:

1. **Wait the bot through** — 300 ticks, break early when she clears the 2.6 band or walks 1.2 from her
   wait-start spot (her measured wake→crossing is ~100 ticks in the sim). This is the game-true answer:
   in the game she walks through and the player follows.
2. **Break the contact** — walk radially out of her capsule (the wall absorbs the blocked component).
3. **Cross on a vision fan** — 3 m world rays around the goal bearing every tick; among the clear ones
   take the best mix of goal progress and capsule clearance; past = 0.8 behind her along the crossing
   (the plain behind-0 form fires while Bond is still at the door plane and the main loop's crowd slide
   re-pins him). Also fixed: `self.pos` was stale inside the trap loops — the aims computed from a
   frozen origin; it now refreshes from the sim each tick.

The fan cannot succeed **in the current port** because of the 0.1 m band — the pass is honest and
correct, and it will start passing the moment the bot walks. No push was disarmed: the trap only steers
Bond; the Jointy push, the arming chain and the door logic are untouched.

## The blocker, plainly

The four red tests (party_door_lock_07000007_live ×2, cutscene_timer_pause_live, probe_07_route) wedge
because the port parks dr22 mid-mouth; walkreal_07000007_live's second test now wedges there too
(it does not opt into crowd_aware — the moved shared-RNG phase parks her on its route as well). The
root cause is the `probe_move` stand-in contradicting the game's ungated root motion at exactly this
spot: her route target has clear world LOS from the parked position, the creep advances, the body
doesn't. The walls-990 note documents the stand-in as a deliberate port choice for missing nav fidelity;
the mouth is where it diverges from observable game behaviour.

**Next round's fix (proposed, not implemented — [S] until the live checks land):** in the route-walk
move, replace the probe_move reject with the game's model — apply the root-motion step, then the
nav-bounds constraint (the port already has `bounds_node_test` for the follow); or, if the live checks
show the leaf's swing cell is the rejector, exempt the door's own passage band from the probe. Either
way the change is bot-wide and needs the full suite, so it did not fit this round's window.

## How this was verified

- `probe_drone5_mouth.rs` (committed e26e0330, rerun this round): the ring anatomy, the wake→crossing,
  the park + creep race, the door 0x02→0x04.
- `probe_drone5_gap.rs` (new, committed): the radial scan at each eastward step; the 45..75° band, the
  blocked north arc, the 0.1 m passable band.
- `walkreal_07000007_live` rerun after the walker change: 1 passed, 1 failed at the same mouth (not a
  regression of this branch's change — the test doesn't use `crowd_aware`; see above).
- The trap instrumentation run (temporary prints, removed): fire positions, the fan's chosen ray, the
  zero-displacement pin, the stale-`self.pos` find.

## What's left

1. The live checks below (they settle [S]→[M]/[L] for the bot-side fix).
2. The bot-side move change + the full suite on the final tree.
3. The four (five with walkreal) red tests turning green behind the walker's wait-beat.

## Live checks for the main agent

Fresh boot (a save-state resume skips the boot path), level 07000007, no input, Bond left at the spawn:

1. **Does she walk?** gdbwatch dr22 (the armed party guest booting at (80.71,−50.47)) — record her
   position (Drone+0x30..+0x38) and route target (Drone+0x670) for ~30 s from the group plate. Expected
   if the game model is right: she crosses the door plane x≈80 westward and keeps walking toward
   (77.48,−49.77) — NOT parked at (79.5,−50.3). This one check decides the whole fix.
2. **The open leaf's collider band.** fieldprobe the door at (80.0,−49.9) at pos01=1.00: poke nothing;
   read the leaf quad / swing angle. Expected: whether the open leaf lies across the opening's north
   side (my scan's blocked 0..30° arc) or swings into the hall.
3. **The mouth's mid-band geometry.** A render at `--bond 76.6,44.1,-50.7` facing east (yaw ≈ 1.38 rad
   in the port's atan2(x,z) convention): what stands between the mouth and node 13 (77.48,−49.77) — a
   table, the leaf, or nothing? My scan says knee-height clear, waist-height blocked on the direct line.
4. **The squeeze.** `--bond 79.2,44.1,-51.2` facing east: with dr22 standing at (79.51,−50.32), walk
   forward — does the game's push slide Bond along her capsule past the mouth, or pin him the way the
   port's does? (The player-facing truth of the 0.95 equilibrium.)

## Suite

Run per the driver's command (the fast runner, `tools/test/suite.py`) on this tree; the total line is
quoted at the bottom of this report (appended after the run finished).

## Usage

not visible to me
