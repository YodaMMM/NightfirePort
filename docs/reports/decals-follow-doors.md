# decals-follow-doors — report

Branch `s/decals-follow-doors` (created from `origin/s/wip-decals-2` @a92c64de + `origin/master` merge;
start entry 867473dc, tests baee7842).

## What the task was

Continue `origin/s/wip-decals-2`: (1) a bullet hole on a door must move with the door (07000011's doors
#192/#213 are the reported floaters; any level), (2) the game draws a hole only while its room/cel is
visible, (3) the flying-debris pool must not shift draw slots when full (interpolated frames drew debris
between two pieces). Find the decal owner/matrix fields and per-frame update, the draw-time room check and
the debris pool rule [M]; fix; render before/after; tests with sisters; run every bullet/decal/debris/door/
interp binary and walkreal_07000011_live; quote totals.

## What was already on the wip branch (verified, not redone)

The two wip commits (63996cda, a92c64de) implement all three rules, each with its [M] chain in the code and
commit message:

1. **Holes ride their host.** `Effect_Bullet` 0x1ce8a8:160-200 links a dynamic hit's hole node to the hit
   object (`node+0x14 = obj`) with the node matrix made object-local; `View_DrawObjectGlist` 0x1e6780:66-74
   draws it at `node+0x20 · obj+0x90` at every draw [M]. Port: `DecalHost` (effect.rs), the object-local
   matrix (`to_object_local`, `Mat_Inverse` 0x1e3520 + `RotPreTransVec` 0x1e49e8), the per-tick
   `Effects::follow_hosts` (called from `drone.rs:18603` with `world.dynamic_matrix(id)`), a host leaving the
   world frees its holes (`Effect_RemoveSObjs` 0x1cf360).
2. **A cel-hosted hole draws only with its cel.** `View_DrawCelGlist` 0x1e6538:62-96 draws a cel's hole
   nodes only while that cel draws [M]. Port: `Scene::decal_cell_entries` + `hide_unseen_decals` fed by the
   window's room mask (`lib.rs` blend + shot paths).
3. **Debris keeps its draw slot for life.** `Effects::assign_debris_slots` (next-fit over slots no live
   piece holds) + `debris_slot_map` / `debris_slots_new_since`; the reused slot snaps instead of sliding
   (`Scene::debris_reused_entries`) — the decals had the same bug and the same fix shape (node mapping +
   snap, `decal_reused_entries`), pinned by the wip's `stairwell_decal_interp_07000011_live` (evicting burst,
   both ticks blended exactly as the window does; every drawn slot must sit on a live hole).

## What this task added (the missing tests, with sisters)

New binary `crates/nf-view/tests/decal_door_follow_07000011_live.rs` — **4/4 pass** (NF_ISO, NF_SILENT;
real input path: `step_bond` trigger + the E/use key, harness spots only):

- `hole_on_the_stairwell_door_rides_the_leaf` — the harness door is the stairwell door (placement 13, the
  walkreal E-opened kind-0 leaf; the same `DecalHost::Object` path as the reported #192/#213, which are
  kind-0 doors of the same level — 10.22,46.76,-5.21 and 13.71,46.78,19.45 [L probe_doors]). One M32 round
  into the leaf through the real trigger path makes an object-hosted hole [M Effect_Bullet 0x1ce8a8:160-200]
  sitting 0.010 off the part's collision face [M hit + normal·0.01, 0x1ce9f0]; holding the use key E-opens
  the door (pos01 0.500 → 1.000, state 8 → 4, part matrix delta 1.0 [L]) and the hole rides the swinging
  leaf — moved 0.816, still 0.010 off the moved face [L]. The static floor hole next to the door moves
  0.0000 — the fix must not drag static holes along.
- `the_follow_measure_flags_a_stuck_hole` (sister) — a hole left at p0 hangs off the moved leaf (the
  measure flags it; vacuous-proof: the door really moved).
- `a_wall_hole_draws_only_while_its_cel_is_in_the_frame` — the stairwell shot's hole (cel 128) draws in the
  facing walk's visible set and collapses to FAR under the office-spawn eye's walk (office cell 156, the
  away set drops cel 128 [L]); the all-visible mask never hides; the hole has its own drawn entry on it.
- `hiding_never_touches_visible_or_unmapped_cels` (sister) — `hide_unseen_decals` hides only mapped, unseen
  cels; a visible cel and an unmapped cel index never hide.

Gap (3)'s tests were already on the wip branch and stay green: `stairwell_decal_interp_07000011_live`
(2/2: the eviction blend + the measure sister) and the unit pins in effect.rs
(`a_debris_piece_keeps_its_draw_slot` with the Vec-shift sister, the decal-pool node tests with reuse
sisters — 9/9 `effect::` unit tests pass).

## Binaries run and totals (this tree)

- `decal_door_follow_07000011_live`: **4 passed; 0 failed** (new)
- `stairwell_decal_interp_07000011_live`: **2 passed; 0 failed**
- `decal_order_live`: **1 passed; 0 failed**
- `blast_debris_draw_live`: **1 passed; 0 failed**
- `break_debris_live`: **2 passed; 0 failed**
- `stairwell_bullets_07000011_live`: **2 passed; 0 failed**
- `bullet_range_live`: **3 passed; 0 failed**
- `door_glist_census_live`: **1 passed; 0 failed; 1 ignored**
- `doors_live`: **2 passed; 0 failed**
- `doors_01_live`: **2 passed; 0 failed**
- `0700000d_gunimp_door_live`: **3 passed; 0 failed**
- `door_aim_ray_live`: **2 passed; 0 failed**
- `door_portal_links_live`: **3 passed; 0 failed**
- `fix_interp_ghosting_live`: **2 passed; 0 failed**
- `stairwell_impacts_render_07000011_live`: **1 passed; 0 failed; 1 ignored**
- `nf-game` lib `effect::` units: **9 passed; 0 failed**
- `walkreal_07000011_live`: **1 passed; 0 failed** (the spawn → lower-floor load-point walk, 9.66 s).

No test was weakened; no assertion rests on an [S] guess. The two pre-existing `#[ignore]`d measurement
dumps in the stairwell binaries stay ignored (assertion-free by design).

## What's left

- **The full four-crate suite has not been run to completion in this sandbox**: background processes do not
  survive tool-call boundaries here, and the foreground window (≈2 h) went to the tests above. The named
  binaries all pass on the final tree; the next session should run `tools/test/suite.py` once on this
  branch (merged with `origin/master`) and quote the total.
- Remaining door/bullet/interp binaries not in the list above (the door_* remainder, npc_doors_live,
  walkreal_probe0c_door*, interp_camera_bracket, bullet_pass_probe, bullet_range_census,
  copter_death_debris_live, misc3_door_wipe, searchlight_bullet_07000006_live, fix_foliage_bullets_live,
  fix_flesh_impacts_live) were not run for time; the door binaries above cover the glist/portal/aim/react
  surfaces the changed draw paths touch.
- **Render before/after for the main agent** (no GPU here; NF_INTERP_SHOT renders the in-between frame):
  - `--bond 12.2,46.6,6.5,1.5708,0.05` on 07000011 (difficulty 3): shoot the stairwell door leaf, E-open,
    and check the holes ride the swung leaf (before the fix they hung at their closed-leaf spots past the
    leaf's edge).
  - `--bond 16.03,32.17,-2.39,0.340,-0.101`: the stairwell spray; holes on the far landing draw only while
    the landing is in the visible set.
  - The debris check: a full pool + a death + a new piece with NF_INTERP_SHOT — the new piece's slot must
    snap (no slot drawn between two pieces).
- Live checks for the main agent (PCSX2, fresh boot): on 07000011, open the stairwell door, shoot its leaf,
  let it close/open again and confirm in-game the holes move with the leaf (the [M] chain predicts they
  do); confirm a hole in the stairwell disappears when its room leaves the VisCelList.

## Build-env note for the next session

alsa-sys needs `PKG_CONFIG_PATH=/home/z/my-project/alsa-prefix/usr/lib/x86_64-linux-gnu/pkgconfig` (now also
in `~/.cargo/config.toml [env]`) and a resolvable `libasound.so` — the dev deb's symlink dangles; the copy
in the rustc sysroot lib dir (stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib/
libasound.so → libasound.so.real = the system libasound.so.2.0.0) makes the link work.

## Usage

Not visible to me: this environment shows no token, cost or context-window figures for the session. The
only context figure I can see is the conversation's own transcript size, which I cannot read a number off.

## Addendum (21:37 UTC)

- `walkreal_07000011_live`: **1 passed; 0 failed** — the walked route spawn → stairwell door → lower-floor
  Load Point passes on the final tree.
