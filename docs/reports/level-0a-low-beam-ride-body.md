# level-0a-low-beam-ride-body — one contact body per player state

Branch `s/level-0a-low-beam-ride-body` = origin/master 88d87c9c + `origin/s/level-0a-low-beam-standing`
(merged clean: `cylgeom_hits_tris` in triggers.rs, its report, its WORKLOG row). Status: **not finished —
`low_beam_621_fires_standing` stays `#[ignore]`d**; the session ported the missing collision arm, ran the
per-substate toucher swap to a decisive negative, and named the real gap. The next session's shape is at the end.

## What the task asked

Take Player_Collision's per-substate arms literally [M 0x194df0 cases 0/3/6/default]: the truck ride is not
the walking substate; find which arm the player is in while riding/attached on 07000005 and which on the 0a
corridor; build the trigger toucher from the arm for the player's CURRENT substate; all three must hold
(#621 fires standing on 0a, the 05 park plate silent standing / fires crouched, the truck chain runs).

## What was done

1. **The arms table, read off the decompile** (`00194df0_Player_Collision` — the switch is on
   `(obj+0xf6) − 1`, so decompile case N = substate N+1):
   - substate 1 (climb) / 7 (creep) — case 0/6: r 0.275, A = pos+up·0.55, B = pos+up·(0.275−cc) [M :202-222];
   - substate 4 (crouch) — case 3: r 0.55, A = pos+up·(0.65−cc), B = pos+up·(0.45−cc) [M :228-248];
   - substate 6 (wire) — case 5: r 0.55, A = pos+up·0.275, B = pos+up·(2.1−cc), attach obj at +0x7b8 [M :249-267];
   - substate 0xf (zipline) — case 0xe: the same tall arm + +0x7d8=0x1c0 [M :383-410];
   - substate 0xc (mount) — case 0xb: r 0.55, A = pos+up·0.275, B = pos+up·(0.55−cc), +0x7b8 = the +0x878
     mount, +0x7d4 = 0 (a THREE-point build; falls through to default when the mount's class byte is '6')
     [M :335-380];
   - substate 0xe (the bone capsule — the missile ride): radius min(cc, 0.55) between bones 0x80000005 /
     0x37 / 0x33 [M :340-381];
   - everything else (the walk 0, and **substate 0x10 — `GT_TakeControl` 0x181740:15's
     `Player_ChangeSubState(obj, 0x10)`, the vehicle take-control**) → the default arm: r 0.55,
     A = pos+up·0.275, B = pos+up·(0.55−cc) [M :315-333].
2. **The ride identified [M/D]**: the 05 hand-off is the SP #38 cut-scene (0x060008C8, 143 ticks, event 4
   disables Bond, `Script_Free` places him on the bed) [D docs/levels/07000005.md §3; the port's
   mayhew_handover_live], after which Bond rides ENABLED on the bed via
   `Control_InheritVelocity` 0x133e88 (the port's `ground_motion`, scene.rs:3055-3071) — NOT a special
   substate. The trace's ride is CROUCHED (pos−floor 0.607 at the park [D fix-truck's trace]); a standing
   ride would be substate 0.
3. **The missing arm ported**: the body build (scene.rs, the `drone::Player` site) had climb/creep, crouch
   and default; it now carries the wire/zipline attach arm (A pos+0.275, B pos+2.1−cc, r 0.55) for
   SS_WIRE | SS_ZIPLINE [M case 5/case 0xe] — committed 859c3c8f. `player.body` feeds the collide-pass
   HITTEST (drone.rs:17591) and the crouched trigger toucher (drone.rs:18262).
4. **The per-substate trigger-toucher swap — tried, measured, REVERTED**: standing toucher = `player.body`
   (default arm). `fix_truck_live` went **0/2**: neither test's chain even started — Bond never left
   (100, 25.88, 108), ch 0x22/0x72/0x70/0x65 never fired (only 0x73 at t5413, after the truck's scripted
   park). The gatehouse TouchOnce #305 (ch 0x5C, the chain's opener) sits at y 27.05 while Bond stands at
   y 26.0 [D the trigger's placement; the test's `--bond 100,26,108` setup] — **1.05 ABOVE Bond's position,
   i.e. above the top of EVERY collision arm** (the default tops at pos+0.825 [M :315-333]; the tallest
   attach arm reaches pos+1.665 but only in the wire substates). The old standing box
   (`Toucher::player(pos−0.6)`: axis pos−0.05..pos+1.26, r 0.55) touched it because its TOP is pos+1.26 —
   not because a game capsule reaches there. The game's own trigger contact is not the capsule-triangle
   test the port models: `Trigger_Hit` 0x1b1fc8 does NO geometry — it classifies the CONTACT LIST the
   collide pass built, and the collide pass's contact gate is `Collide_PickObj` 0x1eae80:93-107: the
   closest point on the toucher's segment against the trigger object's WORLD SPHERE (centre obj+0x80,
   radius obj+0x8C), with `Intersect_CylGeom` 0x109da0 refining the bullet path (ht mode) only. A world
   sphere around the gatehouse touch easily contains a capsule 1.05 below its centre.
5. **The three constraints on one standing reach are unsatisfiable — which is the point**: the opening
   plates sit 0.557-0.575 below the position reference [L the boot plate, docs/levels/07000005.md §3],
   the ch-0x70 park plate at pos−0.624 at the standing park [D script stream 6 key 8 + the trace's
   standing offset 1.086 [L]], and 0a's #621 band at pos−0.74..pos−1.0 [D 0700000a triggers dump]. A
   reach ≥ 0.74 (for #621) always contains the park plate (0.624); the measured box's 0.6 sits between
   the first two and fails #621. With the game's actual contact gate (the world sphere), the plate/
   beam distinction comes from each trigger's own sphere and the arms can go in per-substate as the
   game builds them — the port's single capsule-triangle model is what forces the knife-edge box.
6. **Parity verified on the revert**: `fix_truck_live` **2 passed / 0 failed**, `walkreal_0700000a_live`
   **10 passed / 0 failed / 3 ignored** (the standing report's parity), `walkreal_0700000d_live` (the
   mount family's level) **1 passed / 0 failed**.

## What is left (the next session's shape)

1. **Port the trigger contact gate the game uses**: a toucher contact = the toucher segment vs the
   trigger's world sphere [M Collide_PickObj 0x1eae80:93-107] — the port's Trigger needs the sphere
   (centre/radius of its glist, the obj+0x80/0x8C pair `Trigger_Create` 0x1b1720 stores) surfaced next to
   the AABB; `touched_by`'s first gate becomes segment-vs-sphere (the AABB stays as the broad phase).
   This is the change that lets the ABOVE-THE-HEAD touch fire without a taller box.
2. With that in, re-apply the per-substate toucher (the one-line swap at drone.rs player_touchers, kept in
   this report's history), un-ignore `low_beam_621_fires_standing`, and re-run fix_truck_live +
   walkreal_0700000a_live + the interactables/walkreal sweep. The park-plate standing question settles
   with the sphere model in: the arms predict a standing ride DOES reach the plate (bottom pos−1.035 vs
   the plate pos−0.624) — if the game still shows silent, the ride's contact runs through the cut-scene's
   disabled window and the sister stays as the disabled-ride pin.
3. The full suite (this sandbox kills background processes between tool calls; the four-crate build was
   not started this session — the parity runs above are the tree's coverage).

## Live checks for the main agent

1. **The gatehouse touch's world sphere (07000005, trigger #305)**: `gdbwatch.py` on the trigger object's
   obj+0x80 (centre) / obj+0x8C (radius) during the level start, Bond at the spawn — expected: the sphere
   reaches down to Bond's capsule top (pos+0.825) so the contact registers without any geometry hit.
2. **The 05 park plate under a STANDING ride**: fresh boot, ride the truck without crouching, watch ch
   0x70 — expected per the arms: it FIRES (the default arm bottoms pos−1.035 below the plate at
   pos−0.624); if the game stays silent, the contact is gated elsewhere (the disabled window) and the
   port's sister test keeps its current shape.

## What I ran (this branch, release, NF_ISO the slim on-foot image)

- `fix_truck_live`: **0 passed / 2 failed** with the per-substate swap (the evidence for §4), **2 passed /
  0 failed** after the revert.
- `walkreal_0700000a_live`: **10 passed / 0 failed / 3 ignored** (incl. `low_beam_621_fires_standing`
  ignored); the `low_beam` filter run: 1 passed / 1 ignored.
- `walkreal_0700000d_live`: **1 passed / 0 failed** (the mount level, with the new attach arm in).
- nf-game, nf-formats, nf-audio and the full suite: **not run** — time. The tree's net change vs the
  merge is the one body-build arm (scene.rs) + this report.

## Usage

not visible to me
