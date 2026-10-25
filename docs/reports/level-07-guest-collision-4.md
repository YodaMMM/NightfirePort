# level-07-guest-collision-4 — the walks that still break with guests solid

Branch `s/level-07-guest-collision-4` = `origin/s/level-07-guest-collision-3` (62bdbc99) merged with
`origin/master` (34ac31a5), clean. WORKLOG entry `level-07-guest-collision-4` in `docs/WORKLOG.md`.

## What was done

Setup was done from a fresh sandbox (repo clone, work-slim 192M, ISO verified 369823203, rust
1.99.1, ALSA via a user prefix — note `libasound2t64` had to be extracted too for the runtime
`libasound.so.2`, plus `RUSTFLAGS="-L <prefix>"` and `LD_LIBRARY_PATH`). The background
`tools/test/suite.py` dies between tool calls in this sandbox, so the suite was run in foreground
slices per test binary with `--test-threads=1`.

**All six reds were reproduced on the merged tree** (each green on master per the task):

| test | symptom observed |
|---|---|
| `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` | 10 rounds identical: the contact parks at (-18.3, 6.3, 13.6) state 0x13, `armed_peer` f32::MAX |
| `npc_doors_live::the_01_escort_driven_through_the_shoji_opens_it_and_crosses` | the driven Mayhew crosses the CLOSED shoji (closest 1.33, crossed, door state 0x2 pos01 0.00) |
| `elevator_buttons_09_live::walk_operate_the_button_and_ride_to_0700000a` | STUCK "into the cab": d=1.00, **no world segment** — an invisible blocker at the cab mouth |
| `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it` | `level_exit reached at +0` — the alarm restart beats the walk-over; armour 0 |
| `speech_facial_fallback_live::a_present_script_talker_plays_through_the_facial_and_advances` | the present-script line never sounded through the facial op-1 |
| `probe_07_route::probe_the_route` | STUCK at the hall NW corner (94.15, 44.13, -46.98) → (92.60, 43.60, -47.50), d=1.64, world-segment hit 0.34 |

## Root causes found (evidence-tagged)

1. **npc_doors (the shoji crossing)** — the round-3 undrawn raw slide
   [M Drone_Control 0x139268:75-96, ported at drone.rs `update_anim`] has **no boundary consumer**.
   The game's slide arm clamps the slide speed against the boundary bits before the add
   [M 0x139268:76-83: `+0x4f8 & 1` and `+0xba8 == 0` → bit 2 (+z blocked) zeroes a positive slide,
   bit 4 a negative one], and those bits are `DroneMove_SetBoundryFlags`' probes
   [M 0x14f140: feet → feet ±0.5 along the local axes via `NDrone2_MoveTest(feet, probe, 0, obj,
   0,0,0)`; `MoveTest != 1` sets the bit; docs/systems/boundary-flags-fold.md]. The clamp also gates
   the anim root motion [M DroneAnim_PreTransCallback 0x13b490:120-135, bits 8/0x10]. The deeper
   difference: the game's route stall swaps the walk danim for the stall anim, so the game's slide
   stops with the route; the port's stall keeps the walk danim looping (the round-2 trace), so the
   port's raw slide carries a route-stalled undrawn Mayhew through the closed door. The game's
   `NDrone2_MoveTest` is the AINetwork bounds test only, no collision LOS [M 0x155330 — the
   deviation note on `Network::move_test`].
2. **walkreal_01 (the contact wedged in 0x13)** — a trace with `NF_AILOG` shows the contact
   (drone 1) runs the AllyLead family correctly for the whole escort, then: msg 0x21 (explosive)
   → `RUN_AWAY_FROM_OBJECT` 0x86 → deadline → `RECOVER_FROM_SCARY_OBJECT` 0x88 → its ENTER always
   attacks → `ATTACK` 0x56 → the ENTER's unarmed arm → `DroneFunc_AttackNoWeapon` [M 0x14af80]
   → `CIVILIAN_SCARED` 0x12 → `CIVILIAN_HIDING` 0x13 — which has no self-exit in the game either
   [M 0x15d520 — no update/timer arm; only bullets/impacts resume 0x12]. The trigger was
   Bond's shot on guard 14 (dist 1.8) right before the scare. The divergence candidate is the
   explosive broadcast: the port sends msg 0x21 on any `GrenadeEvent::Bounce` (including a rested
   grenade's micro-collisions) with a **bare segment LOS**, while the game requires the fear props,
   the radius, and `DroneVision_ObjectVisibility ≥ 0.7` (the ±e4/≤e8/<2 cone + CanSeeObject(5,0))
   [M 0x1435f0:30-44 + 0x1781a8 — the port's own comment calls this "a noted deviation"].
   No grenade appears anywhere in the test log, so the bounced object's identity is still open.
3. **probe_07_route (the hall NW corner)** — the bots at the wedge are **two TEAM-1
   CivilianDoorGuards**: drones 10/19, skins 0x5000014/0x5000013, state 0x31, vt 0x11, at
   (94.7, 44.0, -48.06) and (93.7, 44.0, -48.04) (`NF_PUSH_DEBUG` + a scratch dump). They sit inside
   the 2.0 capsule fill, but the port's push skips them for grounded Bond (`guest_team` = team==3
   only — the round-1 [S] port choice), and the walker's dodge mirrors that pusher set, so neither
   applies. There is ALSO a real world-segment hit at 0.34 along the line, so the leg is walk-timing
   plus geometry, not just a push. Per the round-2 [M] reading the game really pushes Bond off every
   capsule-ON bot here (`NDrone2_DoCollision` 0x149ec8:11-34 — non-0xc classes ride the near arm
   [M NearPlayerCollision 0x14b190]; `Collide_Jointy` 0x1ec180:229-266 pushes the class-3 player
   with the full overlap) — the faithful fix is the full push plus a dodge that mirrors it, and the
   walk taking the player's way around.
4. **elevator_09 (the cab mouth)** — blocked with d=1.00 and **no world segment**, so the blocker
   is a bot capsule or a trigger interaction, not world geometry; the pusher census at the stall
   was not captured yet (`NF_PUSH_DEBUG` prints the exact capsule set — one run needed).
5. **armour_0a** — the level-restart alarm fired at +0 of the final leg (the round-3 diagnosis
   stands: g8 opens corridor door d4 and the open panel forms the pocket at (-20.23, 3.93)); the
   walk-over never reaches the armour.
6. **speech_facial** — the seed teleport is cleared to ≥1.05 of every capsule at seed time
   (round-2), but a wandering guest can reach and push Bond afterwards (the new grounded push),
   breaking the seed→talker ray before the facial fires.

## Fix attempted and reverted

The slide boundary clamp was implemented three ways and each was verified against BOTH the shoji
drive and the 07 doorway-leg test: LOS-probe clamp (shoji green, 07 leg red — the 8|4 mask grazes
the throat jambs), bounds-only clamp (shoji green, 07 leg red — the throat IS a nav boundary), and
a `route_blocked`-scoped clamp (07 leg green, shoji red — `route_blocked` is the mission-route
field; the 0x63 mover does not set it). No scope fixed its target without breaking the other side,
so all three were reverted; `crates/nf-game/src/drone.rs` is back at the merged-tree raw add, and
`walkreal_07000007_live` was re-verified 3/3 GREEN after the revert. No test was un-ignored; no
test was weakened.

## What is left (the next round's plan)

1. Port the slide consumer behind the *state* the game really uses: find what stops the game's
   undrawn route-stalled walker (the 0x63 `GoToGoalPosition` blocked arm — the game swaps to the
   stall anim there; the port's blocked arm in the 0x63 UPDATE is the place to look), or request the
   live check below for the `+0x4f8 & 1` setter. Then re-land the clamp on that gate.
2. walkreal_01: port the broadcast's ObjectVisibility cone [M 0x1781a8] (or gate msg 0x21 on the
   drone's facing), identify the bounced object (log `(p, wt)` in the `explosive_queue` push under
   NF_AILOG), and give CivilianScared's scared-run path its game exit for the escort.
3. probe_07_route: make the push faithful (all capsule-ON bots push grounded Bond [M 0x149ec8 +
   0x1ec180]) and extend the walker dodge to the same pusher set; then re-author the corner leg the
   player's way.
4. elevator_09: one `NF_PUSH_DEBUG` run at the cab stall to name the blocker, then either the
   faithful-push route or the player's way around.
5. armour_0a: the round-3 pocket — the E-toggle attempts failed to move the door state; the next
   candidate is walking the pocket escape with the dodge active or the door's own use path.
6. speech_facial: re-clear the seed ray dynamically (a small orbit or a later re-seed) so a late
   guest push cannot break the alignment.

## Live checks for the main agent

1. `gdbwatch` drone+0x4f8 bit 0x1 during a guard pushed against a wall and during the 07000001
   driven-escort walk — does the boundary-flag bit ever arm on the shipped path (the doc's open
   item), and what writes it?
2. `gdbwatch` drone+0x58c (slide speed) and obj+0x30/0x38 for the driven Mayhew walking at the
   closed courtyard shoji while undrawn — expect the slide to stop (the clamp) or the danim to
   swap to the stall anim (the state machine) at the door.
3. `fieldprobe` the 07000001 contact (skin 0x500004f) after Bond fires near guard 14 at ~(-9.5,
   6.4, 17.9): read his state (+0x10?) through 0x86/0x88/0x56 — does the real game send him to
   CivilianScared, and does he resume the lead afterwards?
4. A fresh-boot 07000007 measurement at the hall NW corner: with two door guards at
   (94.7/93.7, 44.0, -48.0), does Bond walking (94.15, -46.98) → (92.60, -47.50) get pushed, and
   which way does the real player's route go?

## Verification performed this session

- Reproduced all six reds on the merged tree (exact binaries, `--test-threads=1`).
- `NF_AILOG` trace of the full walkreal_01 run; `NF_PUSH_DEBUG` + a scratch drone dump at the
  probe_07 corner; the three slide-clamp variants each verified against both affected suites.
- After the revert: `walkreal_07000007_live` 3/3 GREEN, `npc_doors_live` 5 passed / 1 FAILED / 1
  ignored (the pre-existing state), `fix_guest_solid_live` not re-run after the revert (drone.rs
  is byte-identical to the merged tree, which the earlier runs covered).
- The full four-crate suite was NOT completed in the window (the sandbox kills background jobs; the
  foreground slices covered the touched tests). The total suite line is therefore
  **not available this session** — the next round should run `tools/test/suite.py` on the merged
  tree before merging.

## Usage

not visible to me — this session's interface exposes no token, context-window or cost figures.
