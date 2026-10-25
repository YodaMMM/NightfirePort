# level-14-lid-and-climb — the trench lid never opens: the E-open is a proven no-op [M]; the room is two parallel walkways and the descent is the east stairwell [D]; the corridor-side Q-Claw climb refuted; the leg-1a rewrite scoped

Branch `s/level-14-lid-and-climb` = `origin/master` (c528468b) + the WORKLOG entry + the seven
assertion-free probes. No port code changed; no test changed (the exploratory leg-A re-aim was
reverted — see "what's left").

## 1. The lid-open mechanism does not exist [M] — the mask-4 lid SP is a one-shot

Every access path was read off the decompile:

- The eight mask-4 lid SPs (kind 0xd9) all carry `scripts [0x06000A3B, 0]`, `modes [64, 0]`,
  out 0, gate 0, stay 1 [D probe14_spdump]. `SP_Update` 0x1dbda8 case 2 toggles to the other
  script slot only when the OTHER hash is non-zero; with scripts[1] == 0 the SP never reloads,
  and `SP_LoadScript` 0x1dbc08's `Script_Set2Start` — the ONLY stream/cursor rewind in the
  script system — never runs again after the load prime.
- Script 0x06000A3B's four keys are all identity (pos 0,0,0,1, scale 1) [D probe14_spdump]; its
  two streams hold no EntityEnd op, so `Script_KillStream` 0x1d9860 (the only killer, called
  from `Script_EntityEnd` 0x1dafe8 / `Script_RemoveObj` 0x1db6e0 / `Script_Free` 0x1da880) never
  runs: the created slab 0x02000750 (kind 0x31 → the `control_create_object(0,0,0,0)` fallthrough
  [M 0x1d8968]) and the ladder art persist closed forever. Event 0xC at t63 (SCRIPTINFO+0xaa8
  bit 0x10 [M 0x1d94c0 case 0xc]) is a keep-flag for a kill that never comes — the qclaw-climb
  report's "consumer not found" is resolved: the consumers are the KillStream arms, and on this
  script they are dead code.
- The E-press reaches the SP through the SLAB (class 0, obj+0xe4 = the owner ext [M
  `Player_Activate` 0x1a5170's class-0 arm → `SP_Activate` 0x1dc338]): it sets the activate word
  and resets SCRIPTINFO+0x9c (the pause counter [M `Script_Update` 0x1da998 head]), `SP_Update`
  case 1 replays the script — and nothing re-runs: the stream cursors rest after their End ops
  (`Script_Run` 0x1d8548's End arm returns; `StreamEnd` 0x1db420 only clears the playing bit once
  the stream clock reaches the script length), so the replay advances empty streams and then the
  SP kills itself: stay → state 5 → `obj+0xfe |= 1` [M 0x1dbda8 case 2/5], swept by
  `control_movement_object_handler` 0x1336f8's kill pass (`Effect_RemoveSObjs` +
  `control_delete_object`). One E-press and the SP is gone; the slab never moves.
- The slab is legitimately solid: map object 0x02000750 carries flags 0x0 (no never-pick 0x20, no
  bullet-pass 0x40) [D probe14_objflags], a created object's +0xf0 starts 0 [M
  `control_create_object` 0x1344e0 + `control_init_object` 0x134630 never write it], so
  `build_LinkToRoom` 0x1c9680's `+0xf0 & 0x20000000` test passes and the slab cel-links into the
  pick's chains. The qclaw-climb report's solidity chain stands; its "must go through an OPENED
  lid" inference does not.

## 2. The geometry: two walkways, not a slot [D]

Down-seg grids through the port's collision (static + dynamic), all [D] disc data via the port's
parser (probe14_slabgeo / probe14_slotgrid / probe14_room / probe14_stairs):

- The four duct lids are 5 cm flat panels at y 2.346..2.397 lying on the vent room's TWO floor
  pedestals — west x 40.06..41.54, east x 44.54..46.03, both z 62.22..63.66; the z 69.65 lid
  stands VERTICAL (y 0.94..2.37) — the open entry lid behind the spawn.
- From y 3.5 (Bond's height band) the room is TWO PARALLEL WALKWAYS: west x 40.5..41.5 and east
  x 45..46, floor 2.40, running continuously z 59..67. The "slot" is the 3 m gap BETWEEN them
  (open down to the corridor at -0.6); the west lid is simply the west walkway's floor patch.
  The slot-drop leg of the east route test drops between the walkways — it is not the game's
  route.
- The descent into the corridor is the EAST STAIRWELL: the east walkway steps down at x 44.5..46,
  z 58 (2.40) → 57.5 (2.30) → 56-57 (1.05, the (45.23, 1.07, 56.27) stairwell-slot approach [D])
  → 54.5-55.5 (0.60) → 51.5-54 (-0.91) into the x 48..50 corridor (-0.60, z 51..62) that runs
  east to nav node 4 and south to the mouths' lane.

## 3. The corridor-side Q-Claw climb is refuted [M + D]

`Bullet_Delete` 0x12e660:50-55 latches the claw bolt ONLY on a class-0x51 ('Q') grapple-point
object (`Player_SetGrapplePoint` 0x195da0); anything else and the bolt dies silently. Every
corridor aim ray at grapple #626/#657 (the port resolves them at (45.28/40.77, 5.10, 42.87) [D
probe14_climb]) hits the closed lid (y 2.37) or the mezzanine floor (y 2.40, continuous z
41..43.6) first — the claw can never reach the points from below while the lids are closed, and
nothing opens them (§1). The mouths' lids are flush with the mezzanine floor: the entry is the
DUCT FROM INSIDE — Bond walks out of the east duct's south mouth over its own flush floor lid —
so leg 1a's climb step must be rewritten onto the duct route, not a corridor grapple.

## What was done

- Setup (fresh sandbox): clone, work-slim + the onfoot ISO (369,823,203 bytes), the nf-queue
  worktree (recreated — the old one pointed at the dead previous-session path), Rust 1.99.1,
  ALSA dev + runtime libs into a user prefix, branch `s/level-14-lid-and-climb` pushed from the
  first minutes.
- The decompile pass over the whole script system: `Script_EventHandler` 0x1d94c0,
  `Script_KillStream` 0x1d9860, `Script_EntityStart` 0x1d8d78, `Script_CreateEntity` 0x1d8968,
  `Script_Interp` 0x1d9a08, `Script_Run` 0x1d8548, `Script_StreamEnd` 0x1db420, `Script_Update`
  0x1da998, `Script_Play`/`Script_Stop`, `SP_Update` 0x1dbda8, `SP_LoadScript` 0x1dbc08,
  `SP_Activate` 0x1dc338, `SP_Create(ScriptPlayer)` 0x1dc7e0/0x1db9a8, `Player_Activate`
  0x1a5170, `build_LinkToRoom` 0x1c9680, `control_create_object` 0x1344e0,
  `control_init_object` 0x134630, the kill sweep `control_movement_object_handler` 0x1336f8,
  `FuseBox_Update` 0x1adfe8, `Bullet_Delete` 0x12e660.
- Seven assertion-free probes committed (probe14_spdump/objflags/slabgeo/slotgrid/room/stairs/
  climb) — they ARE the [D] evidence tools for the next session.
- The kill-flag consumer question from the task brief: obj+0xfe bit 1 is consumed by the
  0x1336f8 sweep (deletion), and its setters are enumerated above.

## What's left (the next session's work, in order)

1. Rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto the EAST WALKWAY: crawl
   south x 45..46 from the start — the start nook (x 39..41.5, z 63..67) connects ONLY to the
   WEST walkway [L probe run: the east walk blocked at x ≈ 41.5, z 64.7; the nook→east crossing
   is an open question — see the live checks] — resolve the west→east crossing, descend the east
   stairwell (x 44.5..46, z 58→51), corridor east to x 49, south to the mouths' lane.
2. The mezzanine entry from INSIDE the east duct's south mouth (over the flush lid, no E);
   un-ignore `the_east_duct_route_reaches_the_stairwell_slot` (its drop-through-the-slot leg must
   become the stairwell descent) and then the infiltration walk when it walks green.
3. The full suite (three background starts this session were all killed by the sandbox between
   tool calls — 0 bytes each, the same failure the trench session logged; run it in a foreground
   slice on the final tree and report `tail -1 suite.out`).

## Live checks for the main agent

Fresh boot 07000014:

1. gdbwatch the west lid SP object (placement #266's obj) while pressing E at its slab
   (40.8, 2.37, 63.66): expect `SP_Update` 0x1dbda8 to run, no `Script_EntityStart` 0x1d8d78 hit
   after the load prime, then `obj+0xfe |= 1` (state 5) and `control_delete_object` 0x133178 —
   the one-shot SP death. This pins §1 end to end.
2. gdbwatch SCRIPTINFO+0xaa8 (bit 0x10) — expect NO reader during play (the KillStream arms are
   dead code for this script), closing the qclaw-climb report's open question.
3. Walk the game's Bond from the vent-room start (#666) south: WHICH walkway does the game's
   route take, and where does it cross from the west side to the east side (the start nook's
   east wall at x ≈ 41.5, z 64..65 blocks the port's walker)? A screenshot at the start + after
   the first south move decides the leg-1a rewrite's first waypoint.
4. At the east stairwell head (45.5, 2.3, 57.5): does the game's Bond walk down the steps into
   the corridor (the 2.40 → 2.30 → 1.05 → 0.60 staircase), and is the corridor at the stairs' foot
   the one that runs to x 49?

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).

## Correction (level-14-east-walkway, 2026-10-25)

§1's "the E-press replay re-runs NOTHING: the stream cursors rest after their End ops" is WRONG.
The mode-0x40 preload is Play + ONE `Script_Update` tick + Stop ([M] SP_LoadScript 0x1dbc08:46-54),
and `Script_Update` 0x1da998 only advances the streams while SCRIPTINFO+0xab2 bit 0 (playing) is
set — `Script_Stop` 0x1db668 clears exactly that bit, so the cursors FREEZE at t≈1, they do not
rest at their End ops. The state-1 trigger path ([M] SP_Update 0x1dbda8 case 1) calls only
`Script_Play` (no rewind — `Script_Set2Start` 0x1daf00 lives only in SP_LoadScript's already-loaded
arm), so the replay RESUMES from t≈1 and runs the script to its end: the key-61 ladder art
0x02000751 DOES spawn on the use (the stream's Entity op at key 61), and only then does the
stay-1 SP die (case 2's +0x35 arm → state 5 → obj+0xfe |= 1). The port's SP implementation
(crates/nf-game/src/nis.rs `load`/`update`: mode ∉ {1,8} → start+update+stop+state 1; state-1
trigger → `s.start(looping)` with no rewind) matches the decompile, and
panels_07000014_live::the_route_lid_use_spawns_the_ladder_art_beneath_the_opening (master, green)
pins it. §1's other conclusions stand unchanged: the slab never moves (identity keys [D
probe14_spdump]), the lids never open, the corridor-side Q-Claw climb stays refuted.
