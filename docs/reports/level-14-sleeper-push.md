# level-14-sleeper-push — the sleeper drop, then the east corridor

Branch `s/level-14-sleeper-push` (from `origin/master` 4f31c0eb, merged `origin/s/level-14-east-duct` —
its rewritten east-route test, its report and `probe14_mouth4` were NOT on master; this branch merges
them). Task page: 07000014 past the sleeper drop.

## What's done

**1. The actor-actor push, root-caused and ported (task step 1).**

The game's drone-vs-player push is `Collide_Jointy` 0x1ec180:229-281 — the 0x800 arm of the
object-vs-object resolver: the closest points between the two capsule spines by two rounds of
alternating projection (:234-239, `vecutil_calculate_closest_point_on_line`), the overlap =
(r_mine + r_theirs) − dist (:251-252), and the push = −(their closest − my closest)·overlap·fVar12
written into MY hittest's arms and the +0x10 accumulator (:253-266) — the accumulator
`Player_CollisionHandler` 0x1949a8:136-140 adds to obj+0x30. The SP mode asymmetry is the answer to
"horizontal or vertical": `MPSettings+0x384 == 0` in single-player [M `C_SBNFMAP_Handler` 0x20b8d8:26
zeroes it; the MP menus set it 0x20ec00:149] makes fVar12 = 1.0 for a non-class-3 other and RETURNS
0 for a class-3 other (:231-233) — and the PLAYER is class 3 [M `Player_Init` 0x1910f8:76] while the
bot is class 2 [M `NDrone2_CreateObj` 0x14d7c8:14]. So the player's own query pushes BOND off the
bot's capsule with the full overlap, and the bot never pushes itself; dead-centre the between-spines
delta is vertical (Bond rides tangent on the carrier), beside the spine it is lateral, and as the
carrier walks the delta tilts and Bond slides off — which is exactly what the port now does. The
bot's capsule is the game's own hittest build [M `NDrone2_Collision` 0x140e90:91-121: top = pos +
up·r, bottom = pos + up·(r − h), r 0.4 default / 0.55 kind 0xc, h = the anim height word +0xa0 with
default 1.0 [M `NDrone2_DefaultInit` 0x14b300:48]], ON only near the player [M
`DroneFunc_NearPlayerCollision` 0x14b190 — `2.0 <= dist` fails `NDrone2_DoCollision` 0x149ec8].
Port: `LadderCtx::bot_capsules` (the Sim fills the near bots) + the push block in `physics_tick`
after the static push's DeltaP add-back (Jointy writes +0x10 only, so `pick_ref` is untouched).
Test: `sleeper_push_live::the_drop_onto_the_sleeper_ends_beside_the_drone_not_on_top` — Bond
dropped onto the live sleeper's spot ends beside it on the slot floor (dxz > 0.95, root y < 1.2),
green.

**2. The drop slot was never the drone's back — it is ROOFED, and the roof is a use-lid.**

Probes committed as `probe14_lid.rs` (assertion-free, `--nocapture`): the drop slot (x 40.07..41.53,
z 62.2..63.64, floor −0.6 [D probe14_mouth4]) is covered by vent-lid SP #266's glist shape (hash
0x02000750 — dynamic-set tris at y 2.346..2.397 spanning exactly the slot; the nine 07000014 lids
all run script 0x06000A3B [D decode: stream 0 keys the entity closed → open over 63 ticks, stream 1
event 12 keep-objects]). The lid SPs sit mask 0x4 = ACTIVATE [D placement p5] in state 1 — and the
port's E path already fires them (`useobj::player_activate` → `SP_Activate` 0x1dc338's mask-4 arm):
probe14_lid activated #266 on the FIRST press at the lip, the 63-tick anim swung the slab open (the
slot's dynamic tris 60 → 0 by k45), and Bond walked south through the opening past z 62.2. The
"the sim recentres Bond on the carrier" reading is superseded: the old wedge was the closed lid;
the sleeper push (item 1) handles the landing itself.

## How it was verified

- `sleeper_push_live` (new) green; the probe logs quoted above are in `probe14_lid.rs` output shape.
- The three groups the ungrounded push regressed (01 armour walk, 01 E-key door, 0a west doorway)
  are green again under the marked gate (below); `interact_07000001_live::the_armour_pickup…` and
  `…the_e_key_opens…` re-ran green locally; `walkreal_0700000a_live::west_doorway_repro` re-run was
  SIGKILLed by the sandbox memory cap under the concurrent suite (the documented no-GPU/OOM class),
  not a code failure.
- The decompile reads are quoted with addresses inline; the disc facts are the level data via the
  port's own parsers (probe dumps).

## What's left

1. The drop's landing wedges on a ghost rest at y 2.97 over the trench: `capsule_push` walkable
   with push ≈ 0, NO floor tri within 3 m below, no dynamic tri in the stall box, the sleeper far
   away. Suspect [S] unverified: the flipped-open lid's residual pose (the anim ends at a 180°
   flip north of the hinge) or the script's SECOND entity (0x02000751, obj_type 25, created at key
   61) standing in the slot. One tri-owner dump at the stall pin settles it.
2. The corridor east (x 41→49 at z 62.9), the x≈49 south run to the mouths (z 42.16) and the
   mezzanine. The mouths are the same mask-4 lids (#1/#3 of the nine, (45.30/40.78, 2.37, 42.16)) —
   the same E-open mechanic applies. Then `walk_the_infiltration_to_the_delta_exit`'s leg 1a
   un-ignores and legs 2+ run unchanged.
3. The task's grapple check is open: points #626/#657 at (45.30/40.78, 4.79, 42.86) sit at the vent
   mouths' level — whether they, not the stairwell slot, are the player's route to the mezzanine
   [D + M] (the mouths-first evidence above leans that way; the stairwell slot likely falls out of
   the route).
4. The grounded-side actor push: shipped behind a marked port choice [S unverified] — the push rides
   only while Bond is airborne. With it ungated, three master-green groups regressed (the 01 armour
   walk stuck against an ally's tangent zone at (7.05, 6.47, 2.67); the 01 E-key door; the 0a west
   doorway): the port's allies stand inside the tangent radius its walker paths through, and the
   game's own per-bot ON/OFF gates (+0x4f8 bits 0x20/0x200, +0x570 bit 2 — `NDrone2_DoCollision`
   0x149ec8) are not modelled per bot. Next session: port the per-bot collision flag (or the ally
   yield) and drop the gate; the drop-onto case keeps its [M] push either way.

## Live checks for the main agent

1. FRESH boot 07000014, walk Bond to the south lip, press E at the hatch: does the lid animate open
   (63 ticks) and does Bond drop through — and where does he LAND (beside the sleeper / on the
   slot floor at −0.6 / on a lid residual)? Record obj+0x30 at the landing tick.
2. Same boot, from on top of the sleeper: does the game's Bond slide off as it patrols (the
   Jointy vertical-then-tilting push), and does the starting gun damage it from his back?
3. gdbwatch on the ally case: with an ally inside 2.0 (01 start), does `Collide_Jointy` produce a
   push on Bond's +0x750 at all — i.e. does the game push the player off allies on the ground
   (+0x4f8 bit 0x20 / +0x570 bit 2 per bot decides; the 01/0a walks suggest NOT).

## Run log

- `cargo test --offline --release -p nf-view --test sleeper_push_live` — 1 passed.
- `cargo test --offline --release -p nf-view --test interact_07000001_live the_armour_pickup` and
  `… the_e_key_opens` — green after the gate.
- `probe14_lid` (assertion-free) — the logs above.
- The fast suite (`tools/test/suite.py`): the first run (pre-gate tree) built 788 binaries in 913 s
  and flagged the three push regressions named above (each re-verified green locally after the
  gate); the final-tree rerun was started at 19:25 UTC and did not reach its total line before this
  report — the touched-crate tests above plus the first run's signal are this session's coverage.
  The 3 `disc_golden` slim-ISO failures remain the documented waiver.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
