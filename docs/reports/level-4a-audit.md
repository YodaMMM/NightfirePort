# Report — `level-4a-audit` (the 0700004a behaviour audit)

Branch `s/level-4a-audit`, 2026-10-26. The deliverable is `docs/levels/0700004a-audit.md`;
this report summarises what it covers, how it was verified, and what is left.

## What's done

**The full placement audit.** All 294 placements of the main map (map 41) are tabled one
kind per row in `docs/levels/0700004a-audit.md`: 0x0F drones ×38, 0x19 static cels ×7,
0x24 start ×1, 0x31 lift cab ×1, 0x33 hint ×1, 0x47 room volumes ×57, 0xD2 shooters ×2,
0xD9 script-players ×4, 0xDB doors ×162, 0xE9 AI points ×2, 0xEA touch triggers ×13,
0xF1 spawners ×6 [D census]. Every row names the creator in `parsemap_create_dynamic_objects`
0x1d04f0 (or the static path for the two caseless kinds 0x19/0x47), the update and
use/damage/event handlers with addresses [M], and the port's answer, with a GAP column.
Every kind the dispatcher handles but the census shows zero of on 4a is listed under the
table (switches, monitors, sensors, pickups, ladders, grapples, destroyables, …) so the
"nothing there" claim is checked, not assumed.

**The door mechanism, decoded for this level** (audit doc §1): `Door_Create` 0x136100's
param map, `Door_Update` 0x1363c8:101–107's one-shot unlock on the p3 rising edge (with the
0x7000014/0x700004a message gate), `Door_DoMsg` 0x135f18's E-refusal (every 4a door has
class word 2 → remote-only), and `Doors_Calc` 0x135fe0's twin sync. The unlock map: exactly
6 of the 81 pairs carry p3 — #88/89 ch 0x3d, #110/111 ch 0x40 (the lift NIS's t1610 event-0xd
write, not a plate), #128/129 ch 0x3f, #140/141 ch 0x45, #204/205 ch 0x3e, #220/221 ch 0x4f —
the other 75 pairs are permanently sealed and are exactly the floors the ride passes.

**The scripts** (audit doc §2): all 9 in-archive type-7 NISs dumped and decoded per stream
(the lift ride 06000571 with its keep-objects, negative-delay hold and t1610 ch-0x40 write;
the rest are unreferenced deck dressing). The two story cut-scenes live in the companion
archives and are decoded op-by-op: 060006cf (0780004a.bin, the −52 landing scene — camera
spline, `Player_Disable`/`Drone_EnableAll`, the s27 ch-0x22 write at t575) and 060006d3
(0790004a.bin, the shaft-bottom exit scene — camera spline, event 6 hand-back p0 = 0, and
the `0x12 [0x0720004a]` → movie-code → 0x07100013 → 07000013 chain the port already
implements at triggers.rs:23–70). This reconciles `0700004a.md` §2's "11 scripts" = the
9 in-archive NISs + the 2 companion cut-scenes.

**The mission record** (audit doc §3): one objective ("Gain access to lobby", text
0x04000039), complete ch 0x4a — written cross-level by 07000013's spawn touch #153, nothing
on 4a writes it [D scan]; enable ch 0x4d = reveal plate #277; no in-level fail writer (the
shared alarm 0x61 / civilians 0x60 monitor has no writer on 4a).

**Small gaps fixed with tests.** `crates/nf-game/tests/levels_0700004a_census_live.rs` pins
the two behaviours the audit found present but untested at placement level:
- `the_shaft_shooters_fire_once_on_their_arm_channel` — ch 0x22's rising tick fires exactly
  one weapon-61 bullet from #286's position, #287 (arm 0x4f) stays silent on it, and the
  fired shooter never fires again [D census placements; M `Shooter_Update` 0x17d0c0 +
  the 0x1336f8 delete pass].
- `the_door_unlock_map_opens_exactly_the_six_fight_floor_pairs` — 162 doors; exactly the 12
  doors of the unlock table carry p3, the pair and its twin open on the channel's edge,
  data+0x68 is cleared after the unlock, a sealed pair (#106/#107) ignores the same channel,
  and a spawner channel (0x17) opens nothing [D census + lift NIS event; M 0x1363c8 / 0x135fe0].

**Draw-only gaps: none.** Mesh resolution is 0 unresolved across all 42 maps [D census] and
the static path draws every 0x19/0x47 cel.

## How it was verified

- Census: `nf-extract placements-all` / `bin-list` / `nis` against the slim ISO [D], dumped
  to scratch TSVs (not committed). Cross-checked against the decomp per kind: the 0x1d04f0
  switch re-read, doors 0x136100/0x1363c8/0x135f18/0x135fe0, shooters 0x17d030/0x17d0c0,
  spawners 0x1378d0/0x137aa0/0x137cb0, script events 0x1d94c0, script-players 0x1db9a8/0x1dbda8 [M].
- The two new tests: green on the final tree, run directly
  (`cargo test --release -p nf-game --test levels_0700004a_census_live` → both `ok`).
- Suite (`tools/test/suite.py --only`, foreground chunks — the sandbox kills background
  jobs): **nf-game chunk1 198 passed / 0 failed; chunk2 906 passed / 0 failed / 6 ignored**
  (the 6 are the pre-existing ignores: world.rs cull gates, anim_events, door_glist census,
  nav_kick — none added by this task). The chunk logs were scratch and the sandbox reset
  wiped them; the totals are as recorded in-session.
- nf-formats: 4 FAILED, all `disc_golden`/identity — the known slim-ISO gap; this task
  touches no nf-formats code.
- nf-view: not run to completion — its sweep renders ~700 bins at ~80 s each in software;
  it did not fit the window. This task adds no nf-view code (its own 4a live tests already
  exist there from earlier tasks).
- nf-audio: not run in the window; no nf-audio code touched.
- The tested tree is exactly the pushed branch: origin/master (7e35bd77) is the branch's
  direct parent and no merge happened after the runs.

## What's left (pre-existing, own tasks)

- **GAP-1 — the rider spawner release (large).** For Bond on the ride line the floor
  garrisons never spawn: the game's NOT-visible spawn gate is answered by real wall
  occlusion (`Drone_PositionVisible` 0x13a7a0), while the port's stand-in
  (`drone_position_visible`, drone.rs:7743) is not blocked by the lift car's interior walls,
  so the gate never opens. Chain: `DroneSpawner_Init` 0x137aa0 arm →
  `DroneSpawner_SpawnDrone` 0x137cb0:43–47 screen → 0x13a7a0. Parked in
  `crates/nf-view/tests/interact_0700004a_live.rs`: `the_floor_spawner_releases_its_drones_on_the_floor_plate`
  and `a_spawned_drone_takes_the_riders_fire_across_the_shaft`, both `#[ignore]`d with the
  harness aside-park workaround. Fix the LOS stand-in, un-ignore both.
- **GAP-2 — the hand-back POSE after the landing scene (large).** The event-6 arm copies the
  script matrix into Bond's (+0x90 ← SCRIPTINFO+0x40 [M 0x1d94c0 case 6]) and the matrix
  part is pinned (`crates/nf-view/tests/fix_landing_matrix_live.rs`), but the base-pose
  composition — what Bond's pose actually is at hand-back before the walk resumes — is not
  verified against the game.
- A full nf-view suite sweep (see above).

## Live checks for the main agent

1. **The hint text (placement #293).** `Hint Message ` at (3.07, −13.80, −3.36) with
   p0 = 0x0100026B [D census]; the game shows the label's string once the player enters the
   marker's sphere — `Text_AddMsg(label, type 1, life 0xB4)`, no sound, no stage gate
   [M `Hint_Update` 0x1b3380]. Check (read-only): ride the lift down and screenshot the
   message the car passes at that marker — what is the string behind 0x0100026B, and does it
   show once (not every frame)?
2. **The landing hand-back pose (GAP-2).** When 060006cf ends (event 5 at t595 releases
   input; s27 wrote ch 0x22 at t575 [D dump]), screenshot the frame the player regains
   control and, if the debugger is handy, record Bond's object matrix +0x90 write. Expected
   [M]: +0x90 = SCRIPTINFO+0x40 (the scene's final script matrix — already pinned by
   `fix_landing_matrix_live`); what is not pinned is the composed base pose on top of it.
3. **The exit scene chain (060006d3, 0790004a.bin) — blocked by GAP-1 until the spawner LOS
   fix.** Once Bond can reach the shaft bottom (plate #275 writes ch 0x84), play the scene
   and check: the camera spline (s0) drives the view; Bond is frozen at the scene start
   (event 4) and released at its end (event 5); the hand-back matrix (event 6, p0 = 0) lands
   in Bond's +0x90; then the `0x12 [0x0720004a]` event saves and the movie code maps
   0x0720004a → 0x07100013 → level 07000013 [M `GetLevelToLoadFromMovieCode` 0x1d3560;
   triggers.rs:23–70]. Screenshots: the scene's last frame and the 07000013 load.

## Usage

Token, credit, context-window and cost figures for this session: not visible to me. The
environment I run in (a CLI worker behind an IM gateway) exposes no token/credit/context or
cost readout anywhere I can read — there is no UI panel in the sandbox and no such fields in
the message metadata. Nothing is estimated here.
