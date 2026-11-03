# level-14-cascade-servers-2 — the servers vanish when the generator hall blows

Branch from origin/master; the previous attempt's research is on origin/s/level-14-cascade-servers-and-door-guards (its report pins the block-0x27 LOD-next / parsemap fallback slot for destroyed models).

On 07000014, when the generator-hall cascade blows the server racks, the racks disappear instead of showing their destroyed models, and guards keep trying to open a door outside the room.

Do:
1. Finish the destroyed-model swap for the cascade's objects [M parsemap_block_map_data_dynamic 0x1d1b68 / parsemap_block_LOD 0x1d1f10 and the destroy path that switches the drawn model]; the destroyed racks must draw their destroyed model at their spots. List --bond spots for a render check.
2. Find why the guards spam the door outside the room (their goal/door target after the cascade) against the game [M]; fix where it differs.
3. Add real-input tests on 07000014 (the cascade from the switch E, then: destroyed models present, no door spam); run interactables_07000014_live, walkreal_07000014_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.

---
claimed-by: sandbox3 at 2026-11-03T00:14Z

---
## Report (blocked, 2026-11-03T01:10Z)

# level-14-cascade-servers-2 — report

Branch `s/level-14-cascade-servers-2` (from origin/master c0450ef6). Task: on 07000014, the
generator-hall cascade's racks must not vanish, and the guards must not spam the door after the
cascade. The previous attempt's branch (origin/s/level-14-cascade-servers-and-door-guards) is fully
merged into master (part 1's wreck side, the two OpenDoor dispatch fixes, the wait pin, the probe
harness) — this session worked on top of it.

## Part 1: the racks' NO-SWAP case — DONE, pinned, green everywhere

The old census's "the 19 satellite boxes stay vanish-only" verdict was wrong, and that verdict IS
the user's symptom (the racks disappear).

**The game's chain, corrected** ([M] unless noted):

- `Destroy_Create` 0x12c318:25-30 stores tag[0] = `*(glist+0x40)` (the block-0x27 LOD-next) or the
  parsemap fallback `pCelList + idx*0x50 + 0x50` [M 0x1d1b68]; **but 0x12c318:57-59 then ZEROES
  tag[0] when the leaf's OWN glist flags carry 0x10** (obj+0xF0 = glist+0x44, copied by the
  SetGList at :25 [M Control_SetGList 0x134960]).
- `Destroy_Smash` 0x12bb60 takes its `*piVar1 == 0` arm and **skips the `Control_SetGList(obj,
  tag[0])` swap entirely** — the intact glist stays attached; the leaf keeps drawing exactly what
  it drew before the smash. (`Control_SetGList(obj, 0)` is a no-op anyway [M 0x134960's `param_2
  != 0` guard] — the swap never lands either way.)
- 07000014's racks are that case: **[D] obj#273 DESTROY_BoxSatellite glist flags = 0x30 — bit 0x10
  set** (the port's parse of the disc's object table; the dumped census: 6 non-hash placements
  plc#922/923/925/1009/1010/1011 + the by-hash rest of the audit's 19, obj#274
  3RD_Person_Icon flags 0x50 batches 1 — the fallback record that never swaps in).

The old read applied `View_AddObjects` 0x1e62d8:33's `(obj+0xF0 & 0x8010) == 0` gate to the swap
TARGET (the icon) — but the target never swaps in when tag[0] is zeroed; the gate is irrelevant.

**The fix** (`crates/nf-view/src/scene.rs`, the 0xe1 leaf block): a leaf whose own object flags
carry 0x10 joins NEITHER the per-frame dead gate (`destroy_centres`) NOR the wreck bakes — its
draws stand unfiltered, exactly as the game's no-swap leaves the glist attached. The swap-case
leaves (barrels — obj#290 flags 0, generators — obj#291 flags 0 [D]) are unchanged.

**Tests** (`crates/nf-view/tests/destroy_wreck_07000014_live.rs`, 2/2 green):

- `the_destroyed_glists_resolve_per_the_game` — the [D] flags pin (obj#273 flags & 0x10 == 0x10),
  the fallback adjacency [D], no wreck baked for any rack leaf, and the new sister: **the rack
  leaves are NOT in the dead gate** (their intact draws survive) while the swap-case leaves stay
  in it.
- `the_cascade_swaps_the_smashed_leaves_to_their_wrecks` — the real E press → cascade → 0x85 →
  the smashed swap-case leaves' wreck draws live, AND cascade-killed no-swap leaves (the racks)
  keep their intact draws.

**For the main agent's render** (the racks now stay visible through the blast):

- `--bond -55.0,1.5,42.0,-1.2,0.05` AFTER the cascade: the wrecked barrel meshes
  (object_oilbarrel01_destroyed) AND the three satellite-box racks (#1335-1337's row at
  x −55.9..−55.2, z 40.9..43.6) still standing in their intact models; the same spot BEFORE the
  cascade shows the intact racks + barrels.
- `--bond -48.0,2.0,44.0,2.6,0.1`: GENERATORS_Destroyednew where the generators stood.

## Part 2: the post-cascade door spam — REPRODUCED, root cause [M]-pinned, fix drafted but NOT landed

**Reproduced** (a real-input test drafted on this session's tree, then reverted with the fix): the
E press → cascade → 0x85 → doors 45/46 (Door_Slide_11A/B, ch_lock 0x85) unlock and fully open
(pos01 1.0) — and the released clones then log **96 OpenDoor 0x81 ticks in a 1200-tick window** at
the open door. The probe (examples/probe14_guards.rs, committed on master) also shows three of the
six templates crossing west of x −52.4 (ticks 60/1055/1468 after E) — the old attempt's
zero-crossing is gone on this tree; the spam is what remains.

**Root cause** [M `NDrone2_ReachedDestNode` 0x153358:151-155, read this session]: when a route
re-arrives at a door node whose door **already stands in +0xBA8** (the previous OpenDoor's
binding), the game returns immediately — `if (iVar11 == *(int *)(iVar2 + 0xba8)) return 1;` — no
re-bind, no 0x81 re-entry, no +0x5A6 clobber. +0xBA8 stands until the tail clears it on a later
non-door node with the bound door open. The port's `route_reached_special`
(drone.rs, the `flags & 2`/`next_door` arm) re-binds and re-enters OpenDoor on EVERY arrival —
enter → case 3 exits on DoorIsOpen [M 0x1681f8 case 3] → the route re-arrives → re-enter — the
spam cycle.

**The drafted fix (NOT on the branch — it did not verify within the session):** a persistent
`door_bound: Option<u16>` field on Drone (the +0xBA8 stand-in, added to both literals), set at the
park site next to `door_node`; the same-door no-op `if self.door_bound == Some(node) { return
true; }` before the bind [M 0x153358:151-155]; cleared in the `hit_door_open` tail arm next to the
`hit_door` clear [M the tail's +0xBA8 clear]. With it the spam did not drop below the ≤8 bound in
the one run that fit — either the cycle's entries come through a second path (the `croute`
door-park or OpenDoor's case 0xC re-arm [M 0x1681f8 case 0xC]) or the exit path needs the same
treatment. The next session should re-apply the draft (this report + the reverted diff shape) and
bisect where the 96 ticks actually enter — the test file's second test
(`no_guard_spams_the_opendoor_after_the_cascade`, the door-open precondition now works: hoist the
door index out of the wait loop, because the unlock CLEARS ch_lock to 0x0) pins the bound.

## What ran (this session's tree, before the door revert)

- `destroy_wreck_07000014_live` 2/2 · `door_wait_07000014_live` 1/1 (the locked-door pin) ·
  `walkreal_07000014_live` **8/8** · `interactables_07000014_live` **10/10** ·
  `destroy_draw_keys_0700000d_live` 3/3 · `destroy_channels_02_live` 1/1 ·
  `blast_debris_draw_live` 1/1 · `fix_blast_scripts_live` 3/3. All green.
- The FULL suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p
  nf-view -- --test-threads=1`) did NOT reach its total line this session — the sandbox cutoff hit
  first. The tree differs from master only by the scene.rs no-swap gate + the test updates; the
  next session must run the full suite AND every `walkreal_*_live` (the scene.rs change touches
  every level's destroy draw path — only 07000014's racks carry self-0x10 flags among the census'd
  levels, but the gate is universal).

## What's left

1. Re-apply + verify the door-spam fix (the drafted `door_bound`), with the
   `no_guard_spams_the_opendoor_after_the_cascade` test landing green.
2. The full-suite total line + every `walkreal_*_live` on the final tree.

## Live checks for the main agent (PCSX2, fresh boot)

1. The racks after the cascade: at the generator hall, do the satellite-box racks still draw after
   the blast? Expected [M 0x12c318:57-59 + 0x12bb60's skip]: yes — the intact models stay. If the
   game instead shows destroyed/vanished racks, the no-swap reading is wrong and the gate belongs
   back on the swap target.
2. A door-waiting guard after the doors open: gdbwatch its state (Drone_tag+0x10C) for ~10 s —
   expected: 0x81 entered at most briefly (one enter+exit per door of the pair [M 0x153358:151-155
   + 0x1681f8 case 3]), no sustained OpenDoor at the open door; then combat-move arms walking west.

## Usage

- GitHub token: the driver message's `x-access-token:` PAT (HTTPS password for clone/push only;
  never committed).
- Context window / token / cost figures: not visible to me.
# level-14-cascade-servers-2 — report

Branch `s/level-14-cascade-servers-2` (from origin/master c0450ef6). Task: on 07000014, the
generator-hall cascade's racks must not vanish, and the guards must not spam the door after the
cascade. The previous attempt's branch (origin/s/level-14-cascade-servers-and-door-guards) is fully
merged into master (part 1's wreck side, the two OpenDoor dispatch fixes, the wait pin, the probe
harness) — this session worked on top of it.

## Part 1: the racks' NO-SWAP case — DONE, pinned, green everywhere

The old census's "the 19 satellite boxes stay vanish-only" verdict was wrong, and that verdict IS
the user's symptom (the racks disappear).

**The game's chain, corrected** ([M] unless noted):

- `Destroy_Create` 0x12c318:25-30 stores tag[0] = `*(glist+0x40)` (the block-0x27 LOD-next) or the
  parsemap fallback `pCelList + idx*0x50 + 0x50` [M 0x1d1b68]; **but 0x12c318:57-59 then ZEROES
  tag[0] when the leaf's OWN glist flags carry 0x10** (obj+0xF0 = glist+0x44, copied by the
  SetGList at :25 [M Control_SetGList 0x134960]).
- `Destroy_Smash` 0x12bb60 takes its `*piVar1 == 0` arm and **skips the `Control_SetGList(obj,
  tag[0])` swap entirely** — the intact glist stays attached; the leaf keeps drawing exactly what
  it drew before the smash. (`Control_SetGList(obj, 0)` is a no-op anyway [M 0x134960's `param_2
  != 0` guard] — the swap never lands either way.)
- 07000014's racks are that case: **[D] obj#273 DESTROY_BoxSatellite glist flags = 0x30 — bit 0x10
  set** (the port's parse of the disc's object table; the dumped census: 6 non-hash placements
  plc#922/923/925/1009/1010/1011 + the by-hash rest of the audit's 19, obj#274
  3RD_Person_Icon flags 0x50 batches 1 — the fallback record that never swaps in).

The old read applied `View_AddObjects` 0x1e62d8:33's `(obj+0xF0 & 0x8010) == 0` gate to the swap
TARGET (the icon) — but the target never swaps in when tag[0] is zeroed; the gate is irrelevant.

**The fix** (`crates/nf-view/src/scene.rs`, the 0xe1 leaf block): a leaf whose own object flags
carry 0x10 joins NEITHER the per-frame dead gate (`destroy_centres`) NOR the wreck bakes — its
draws stand unfiltered, exactly as the game's no-swap leaves the glist attached. The swap-case
leaves (barrels — obj#290 flags 0, generators — obj#291 flags 0 [D]) are unchanged.

**Tests** (`crates/nf-view/tests/destroy_wreck_07000014_live.rs`, 2/2 green):

- `the_destroyed_glists_resolve_per_the_game` — the [D] flags pin (obj#273 flags & 0x10 == 0x10),
  the fallback adjacency [D], no wreck baked for any rack leaf, and the new sister: **the rack
  leaves are NOT in the dead gate** (their intact draws survive) while the swap-case leaves stay
  in it.
- `the_cascade_swaps_the_smashed_leaves_to_their_wrecks` — the real E press → cascade → 0x85 →
  the smashed swap-case leaves' wreck draws live, AND cascade-killed no-swap leaves (the racks)
  keep their intact draws.

**For the main agent's render** (the racks now stay visible through the blast):

- `--bond -55.0,1.5,42.0,-1.2,0.05` AFTER the cascade: the wrecked barrel meshes
  (object_oilbarrel01_destroyed) AND the three satellite-box racks (#1335-1337's row at
  x −55.9..−55.2, z 40.9..43.6) still standing in their intact models; the same spot BEFORE the
  cascade shows the intact racks + barrels.
- `--bond -48.0,2.0,44.0,2.6,0.1`: GENERATORS_Destroyednew where the generators stood.

## Part 2: the post-cascade door spam — REPRODUCED, root cause [M]-pinned, fix drafted but NOT landed

**Reproduced** (a real-input test drafted on this session's tree, then reverted with the fix): the
E press → cascade → 0x85 → doors 45/46 (Door_Slide_11A/B, ch_lock 0x85) unlock and fully open
(pos01 1.0) — and the released clones then log **96 OpenDoor 0x81 ticks in a 1200-tick window** at
the open door. The probe (examples/probe14_guards.rs, committed on master) also shows three of the
six templates crossing west of x −52.4 (ticks 60/1055/1468 after E) — the old attempt's
zero-crossing is gone on this tree; the spam is what remains.

**Root cause** [M `NDrone2_ReachedDestNode` 0x153358:151-155, read this session]: when a route
re-arrives at a door node whose door **already stands in +0xBA8** (the previous OpenDoor's
binding), the game returns immediately — `if (iVar11 == *(int *)(iVar2 + 0xba8)) return 1;` — no
re-bind, no 0x81 re-entry, no +0x5A6 clobber. +0xBA8 stands until the tail clears it on a later
non-door node with the bound door open. The port's `route_reached_special`
(drone.rs, the `flags & 2`/`next_door` arm) re-binds and re-enters OpenDoor on EVERY arrival —
enter → case 3 exits on DoorIsOpen [M 0x1681f8 case 3] → the route re-arrives → re-enter — the
spam cycle.

**The drafted fix (NOT on the branch — it did not verify within the session):** a persistent
`door_bound: Option<u16>` field on Drone (the +0xBA8 stand-in, added to both literals), set at the
park site next to `door_node`; the same-door no-op `if self.door_bound == Some(node) { return
true; }` before the bind [M 0x153358:151-155]; cleared in the `hit_door_open` tail arm next to the
`hit_door` clear [M the tail's +0xBA8 clear]. With it the spam did not drop below the ≤8 bound in
the one run that fit — either the cycle's entries come through a second path (the `croute`
door-park or OpenDoor's case 0xC re-arm [M 0x1681f8 case 0xC]) or the exit path needs the same
treatment. The next session should re-apply the draft (this report + the reverted diff shape) and
bisect where the 96 ticks actually enter — the test file's second test
(`no_guard_spams_the_opendoor_after_the_cascade`, the door-open precondition now works: hoist the
door index out of the wait loop, because the unlock CLEARS ch_lock to 0x0) pins the bound.

## What ran (this session's tree, before the door revert)

- `destroy_wreck_07000014_live` 2/2 · `door_wait_07000014_live` 1/1 (the locked-door pin) ·
  `walkreal_07000014_live` **8/8** · `interactables_07000014_live` **10/10** ·
  `destroy_draw_keys_0700000d_live` 3/3 · `destroy_channels_02_live` 1/1 ·
  `blast_debris_draw_live` 1/1 · `fix_blast_scripts_live` 3/3. All green.
- The FULL suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p
  nf-view -- --test-threads=1`) did NOT reach its total line this session — the sandbox cutoff hit
  first. The tree differs from master only by the scene.rs no-swap gate + the test updates; the
  next session must run the full suite AND every `walkreal_*_live` (the scene.rs change touches
  every level's destroy draw path — only 07000014's racks carry self-0x10 flags among the census'd
  levels, but the gate is universal).

## What's left

1. Re-apply + verify the door-spam fix (the drafted `door_bound`), with the
   `no_guard_spams_the_opendoor_after_the_cascade` test landing green.
2. The full-suite total line + every `walkreal_*_live` on the final tree.

## Live checks for the main agent (PCSX2, fresh boot)

1. The racks after the cascade: at the generator hall, do the satellite-box racks still draw after
   the blast? Expected [M 0x12c318:57-59 + 0x12bb60's skip]: yes — the intact models stay. If the
   game instead shows destroyed/vanished racks, the no-swap reading is wrong and the gate belongs
   back on the swap target.
2. A door-waiting guard after the doors open: gdbwatch its state (Drone_tag+0x10C) for ~10 s —
   expected: 0x81 entered at most briefly (one enter+exit per door of the pair [M 0x153358:151-155
   + 0x1681f8 case 3]), no sustained OpenDoor at the open door; then combat-move arms walking west.

## Usage

- GitHub token: the driver message's `x-access-token:` PAT (HTTPS password for clone/push only;
  never committed).
- Context window / token / cost figures: not visible to me.

## ntfy failed

All three fallbacks failed (2026-11-03 ~02:15 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(...)"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>` (the first attempt
   also hit a SyntaxError from a non-ASCII dash in the message byte literal; the retry is the
   ASCII-only text above)
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → no output, killed at the
   30 s tool timeout (no banner, no transfer)
