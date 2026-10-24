# level-14-qclaw-climb — the rides restored on the full-glist reading [M]; the trench render-only lid premise refuted; the east route re-parked on the open lid mechanism; the climb leg untouched

Branch `s/level-14-qclaw-climb` = `origin/s/level-14-trench-landing` + `origin/master` merge
(62aa121a: the camera beam, the satchel draw) + this round's changes.

## What was asked

1. Find what the trench branch's drone.rs change does to riding/carried bodies (lifts, gondola,
   paternoster, the wire), fix it so every listed ride/carry test is green again while the east
   route stays green [M].
2. The Q-Claw climb leg (grapple #626/#657, reel, the mask-4 lid E-open, out onto the mezzanine)
   and the leg-1a rewrite of `walk_the_infiltration_to_the_delta_exit` onto the east route.
3. The full suite on the final tree; nothing red that is green on master.

## 1. The rides: the whitelist's premise is refuted at three independent [M] points

The trench change whitelisted the solid script-entity kinds to {0x20, 0x22, 0x29, 0xe1} (the four
`Script_CreateEntity` arms whose creators call `build_LinkToRoom`). That un-posed every other
kind — and **the ride platforms are fallthrough-kind script entities**: the paternoster cab is
`Obj::Entity { hash 0x2000367 }` keyed by script 0x06000048 [D lifts_ride_live + docs/systems/
lifts.md], the 0700004a lift platform is **obj_type 31** [D docs/levels/0700004a.md], the
gondola carriage likewise a keyed script entity. The game makes them solid regardless of kind:

- **The glist**: `Script_CreateEntity` 0x1d8968's fallthrough runs
  `hashtable_set_object_to_entity_gfx` 0x1ca6d8, whose hit lands in `Control_SetGList` (:38-40)
  — **the identical `t_hashtable` lookup the creator arms make** via
  `hashtable_hashcode_to_celglist` 0x1ca798 (same table, same value field). "gfx attach" IS the
  collision-glist attach; the creator/fallthrough split selects the wrapper behaviour
  (break/ladder/switch/destroy), not solidity.
- **The link**: `Script_EntityStart` 0x1d8d78:113-115 links the fresh object into the cel chain
  (`build_LinkToRoom(obj,1,glb_world)` when obj+0x20 == 0) — **any kind**; the second argument
  only gates `Controls_StraddleTest` [M build_LinkToRoom 0x1c9680:55-57].
- **The per-tick relink**: `Script_Interp` case 1 sets obj+0xfa bit 0x20 on every keyed move
  [M 0x1d9a08:94]; `control_movement_object_handler` 0x1336f8:147-236 rebuilds
  (`Control_BuildWorldSph`) and relinks (`control_handle_cel_change` 0x133288 — which clears the
  bit at :62 and moves the object between cel chains at :135-158) **every flagged object, any
  kind**, then `Collide_Update` scans them (:238).

**Fix**: restore the all-but-five rule in `script_parts`
(`crates/nf-game/src/drone.rs`, the Entity arm): every script Entity poses solid at its per-tick
matrix EXCEPT the five interactive kinds the port carries in its own systems — the triggers
0xea/0xeb (the port's `crate::triggers::Trigger` volumes), the pickup 0xF0 (its glist blocked its
own take ray — fix-safe08), the emitter 0xF2 (control-unlinked explicitly [M 0x1d8968:163-166])
and the AI volume 0xF5. The comment at the arm carries the full citations.

**Verified green** (all previously red on this tree): `lifts_ride_live` 2 (the paternoster carry
+ the 09 cab pair), `gondola_windows_live` 3, `zoe_gondola_realpath_live` 1,
`walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby`,
`walkreal_0700004a_satchel_live` 2, `fix_wire2_live` 3 (the fusebox crouch + both wire moves),
`interact_0700004a_live` 5 (the floor plates + the hint marker as the car passes; 2 pre-existing
documented ignores), `walkreal_07000016_live` 1, `kiko_button_real_walk_live` 1,
`campaign_flow_live` 5, `nf-game --lib` 547.

## 2. The east route: its greenness rode on the refuted premise — re-parked honestly

With the rides correct, `the_east_duct_route_reaches_the_stairwell_slot` fails again at its drop
assertion (:139): **the lid slabs stand solid over the slot**, exactly as the [M] chain above
predicts. The test's render-only reading was an inference from
`Script_CreateEntity`'s arm split — refuted above; its [L] game-side check (does the game's Bond
really drop through a closed lid?) was never run (the trench report's live check 2 is still
open). The [D] facts the port's own parser gives now (`probe14_lid`, committed):

- script 0x06000A3B: stream 0 creates the slab 0x02000750 (kind 0x31) at t1 and the ladder art
  0x02000751 (kind 0x19) at t61, `End(63)`; **no key animation at all**; stream 1 fires
  `Event 0xC` at t63 [M 0x1d94c0 case 0xc sets SCRIPTINFO+0xaa8 bit 0x10 — consumer not found];
- the lid SPs (mask 4, modes [64, 0]) prime at load and play once [M the port's mode-64 arm of
  `SP_LoadScript` 0x1dbc08], so the slabs exist closed from level load — the probe dumps them at
  (40.797, 2.372, 63.659) and (45.283, 2.372, 63.659), their 60 tris over the slot [D
  probe14_lid + the trench's probe section A].

So the game's drop must go through an **OPENED** lid, and the open mechanism (what moves or
kills the slab on the E-press) is the round's open point — the mask-4 SP re-plays the same
create-only script, which cannot open anything. The test is re-ignored with this exact
refutation in its reason (the route findings that do NOT depend on the lid — the crawl, the
trench floor landing shape, the corridor east, the nav lane — stand; only the drop-through-a-
closed-lid leg was pinned on the refuted model). Per the discipline: the test contradicted the
decompile; it was fixed by correcting its premise, not by weakening an assertion.

## 3. What's left

1. **The lid-open mechanism** (unblocks the east route AND the climb leg's "E-open the mask-4
   lid" step): find the consumer of SCRIPTINFO+0xaa8 bit 0x10 [M 0x1d94c0 case 0xc] and how the
   game's E-open at a lid removes/animes the kind-0x31 slab; port it; re-un-ignore the east
   test; the climb leg then chains on the same behaviour at the z 42.16 mouths.
2. **The climb leg**: aim the Q-Claw at grapple #626/#657 (45.30/40.78, 4.79, 42.86) [D], fire,
   reel, E-open the mouth lid, out onto the mezzanine; rewrite leg 1a of
   `walk_the_infiltration_to_the_delta_exit` onto the east route and un-ignore it.
3. The full `tools/test/suite.py` on the final tree: this sandbox kills background processes
   between tool calls (four `suite.out` starts, all 0 bytes — the same as the trench session
   saw), so the coverage above is the touched-crate batches in the foreground. The one batch
   that outgrew its 550 s cap (`campaign_flow_live` 514 s) passed inside a combined run; nothing
   red anywhere.

## Live checks for the main agent

Fresh boot 07000014:

1. At the drop-slot lid (x ≈ 41, z 62.2..63.64): WITHOUT any E-press, walk Bond over the slot —
   does he fall through (the closed lid non-solid in the game?) or stand on it? Then press E at
   the lid and repeat: does the drop work only after the press? This decides whether the
   E-open is real for the SLOT lid or the slot is open collision in the game despite the slab.
2. gdbwatch the kind-0x31 slab object (0x02000750's obj — `glb` via the script stream's +0x10)
   during the E-press: any `control_unlink_object` 0x134608 / `Script_KillObj` / matrix write
   names the open mechanism; also watch SCRIPTINFO+0xaa8 (bit 0x10 set by event 0xC) for its
   reader.
3. At the mezzanine mouths (z 42.16): press E at a mouth lid with Bond on the ground below —
   does the lid animate (the slab's obj+0x90 writes) or delete?

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
