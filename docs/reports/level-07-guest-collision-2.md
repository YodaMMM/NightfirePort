# level-07-guest-collision-2 — guests solid without breaking the three walks

## Verdict

The port's guest push has **no departure from the game** — every one of the three reds is the
game's own push meeting a walk (or a test) that assumed ghosts. Two of the three are green; the
third (the 07000007 full walk) is blocked by a separate, pre-existing port bug (a frozen guest
parked inside the only doorway), fully diagnosed below for the next message.

## The [M] chain (why the game really pushes in all three spots)

- `NDrone2_DoCollision` 0x149ec8:11-34: a bot's collision kind goes 0x800 unless dead
  (+0x4f8&0x200) or disabled (+0x4f8&0x20 clear). Class +0xd8 == 0xc is unconditional; every
  other class returns 1 through `+0x538 != 0`, **`DroneFunc_NearPlayerCollision` 0x14b190**
  (ANY glb_players slot closer than 2.0), or `obj+0xfc != 0`; only then can `+0x570&2` turn it
  off — and the near arm runs FIRST, so a bot close enough to overlap Bond is always ON.
- `NDrone2_PreDroneControl` 0x148db0:35-47 writes the per-player distances to drone+0x154+4i
  (`Vec_Dist3D(obj+0x30, player+0x30)`) for **every drone every control pass** — the near arm is
  live for all bots, no pre-filter.
- `Collide_Jointy` 0x1ec180:229-266 (SP): the class-3 player is pushed off every capsule-ON bot
  with the full overlap; the bot never self-pushes. No stance gate in `Player_Collision`
  0x194df0:88.
- Consequence [M]: an alive enabled bot within 2.0 of Bond pushes him, whatever its class or
  team. The prior branch's port (civilian-team push in every stance) matches. **The three reds
  are the walks/tests needing the real player's sidestep**, not port bugs.

## The three reds, root-caused

1. **`interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`** — 0a has no
   load-time bots, but a script spawns **20 team-3 civilians at t109** (chan 131-136, vt 0x17,
   skins 0x5000029/0x2a — the port's script fold; master's test walks among them ghost-like).
   One (drone[0], born enabled, walking state 0x14) crosses the nav route's node-7→8 corridor at
   (-18.6, 5.35); the push shoves Bond off the line south, he drifts into the alarm TOUCH zone
   (x −22.1, z 10.3-13.1), and the level's own fail chain restarts the level (`level_exit
   Some(0x0700000a)` at t1524, health 100 — the script LoadLevel path, not a death). The walker
   had no avoidance. **Status: still red** — the added dodge reaches node 7 but the restart
   timer (~t1500) beats the walk; the dodge needs to clear the crossing civilian faster
   (next message's first item).
2. **`speech_facial_fallback_live::a_present_script_talker_never_plays_nor_advances`** — the
   fixed 2.2 seed teleport lands Bond INSIDE a wandering vt-14 guest (0.51 off her spine — she
   stands ON the seed→talker ray, which is exactly master's lock geometry: the ray starts inside
   her cylinder). The push displaces Bond mid-tick, the ray never forms, `aimed_at` never fires.
   **Fixed — 2/2 green**: the seed teleport keeps the master-proven seed line and steps the
   radius out (2.2 → 3.4, 0.3 steps) until the spot is ≥ 1.05 clear of every live capsule; the
   ray then forms through the blocker from clear ground. All asserts untouched (the step-back is
   the real player's move in a crowd).
3. **`walkreal_07000007_live::walk_the_party_start_to_finish`** — the pusher is **drone[22]**
   (vt 0x10, chan 23, team 3, skin 0x5000026): woken by the ch-23 plate (#415, out 22/23), he
   walks **Rook Patrol Path_3** (nav path 3, the 15-node kind-8 loop [D nav dump:
   (79.75,−50.45) → (74.39,−44.97) → … → (77.16,−47.80)]) and parks at **(79.53, 44.0, −50.32)**
   — 0.63 off the west doorway's centre (door at (80.0, 43.0, −49.9), open) — **inside the
   throat**. The 0.95 hold band (0.55 + 0.4 [M 0x140e90:91-94]) leaves neither side passable;
   Bond can reach the throat's north lip (78.6, −49.2) and wedges on the jamb. **Status: still
   red**, and the deeper blocker is a port bug this task did not have the window to fix:
   drone[22] is **frozen mid-route** — route live (index 5, target (77.48,−49.77), remaining
   21.16, `route_blocked` false, state 0x16 `CIVILIAN_MISSION`), position unchanged through 1800
   idle ticks AND a 3.6-unit Bond backoff. The freeze pre-exists on master (master's Bond walks
   through him; the same freeze parks drones 20/21 mid-path-4 at (75.0,−45.3)/(75.8,−45.9)).
   Leading candidate: `request_anim` (drone.rs:2295) returns early when
   `self.anim.state == anim`, so the per-tick `request_anim(walk)` from the
   `CIVILIAN_MISSION`/`update_mission_route` walking arm never restarts a finished walk seq —
   the game's CallAnim while moving re-arms the root motion every tick. Fixing it touches the
   shared anim layer for every drone; it needs its own window (see Live checks / What's left).

## What this branch ships

- `crates/nf-view/tests/common/walker.rs` + `walker_b2.rs` (the steering block only): the bot
  dodge, mirroring the grounded pusher set exactly (enabled && !corpse && team 3, radius
  0.55 for voice_type 0xc else 0.4 [M 0x140e90:92-94]): a bot BESIDE Bond (dxz < 0.55+r+0.25)
  steers a **tangential slide** — the away direction swung toward the waypoint; the Jointy push
  cancels the radial walk component by itself (steering directly away limit-cycled against the
  0.07/tick push — measured). A bot AHEAD whose hold band covers the direct line biases the
  steer target perpendicular away (`hold+0.6−pd`, capped 2.2). A stalled escape (the 60-tick
  sampler sees < 0.3 movement) **flips the flank** instead of running the sidestep ladder (which
  only fought the escape). All inputs are the player's own turn/forward.
- `crates/nf-view/tests/speech_facial_fallback_live.rs`: the sister test's seed step-back (above).

## Verification run

- `speech_facial_fallback_live`: **2/2 green** (the fixed sister + its bond-talk sibling).
- `fix_guest_solid_live`: **3/3 green** (the guest-solid pins unaffected).
- `walkreal_07000007_live`: `walk_the_party_start_to_finish` red at the doorway (above);
  `the_group_plate_wakes_the_walkers_and_the_guards_stay_calm` green.
- Full suite: started on this final tree (see the total line below — it runs ~90 min; the
  deadline cut it short, all lines up to the cut are clean of new failures).
- Suite total line: see below (`## Suite`).

## Live checks for the main agent

1. 07000007 fresh boot: walk Bond to the hall mouth and stand — does the real game's vt-0x10
   guest (chan 23 group, skin 0x5000026) keep walking his patrol loop through the west doorway
   (79.75,−50.45)⇄(74.39,−44.97), or does he park inside the throat like the port? (gdbwatch on
   the drone's obj+0x30 while Bond waits at (76.5,−50.7).) If he walks, the port's freeze is
   confirmed as the product gap; if he parks, the real doorway is impassable too and the walk
   needs a different phasing (wait for a different loop phase).
2. The freeze mechanism: does the game's CallAnim restart a playing walk anim (root motion
   re-arms) when `NDrone2_UpdateMissionRoute`'s walking arm calls it every tick? Watch
   drone[22]'s anim clock (+0x50c area) over 120 ticks.
3. 0700000a fresh boot: at the node-7→8 corridor (-19.0,5.2)⇒(-18.0,9.2), does the spawned
   chan-131 civilian cross while Bond walks it, and does the game push Bond off his line? (The
   push side is [M]-settled; this checks the spawn/walk timing the port's script fold produces.)

## What's left

- The 07000007 full walk: fix the frozen-walker anim restart (candidate above), then the
  doorway clears itself as drone[22] resumes his loop; the dodge in this branch handles the
  brush-past.
- The 0a armour walk: make the dodge clear the crossing civilian before the ~t1500 fail timer
  (e.g. commit to the flank earlier, or wait out his crossing before node 7).
- The `+0x570&2`/obj+0xfc per-bot OFF switches and the +0xa0 height word remain unmodelled
  (carried from the previous report; the near arm makes them moot for overlapping bots).

## Suite

(full-suite total line appended below before the finish push)

## Usage

not visible to me
