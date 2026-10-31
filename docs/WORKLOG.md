# WORKLOG

## decals-follow-doors (branch s/decals-follow-doors) — start

Task: continue `origin/s/wip-decals-2` (built on `origin/s/fix-stairwell-decals`), merged `origin/master`
first. Three gaps to close, tests with sisters, run every bullet/decal/debris/door/interp binary plus
`walkreal_07000011_live`, quote totals.

State at start: the wip branch already carries two commits (63996cda, a92c64de):
- the decal pool models the game's 200-node DList (fresh nodes 199..0, returned nodes after them
  [M 0x1cc778, 0x1cca28, 0x1cccb8, 0x1cc868]); a full pool unlinks the oldest in-use node and reuses
  it in place (Effect_Bullet 0x1cea20..0x1cea58);
- decals carry a DecalHost (hit cel HITDATA+0x54 or hit object +0x58 with a node matrix made
  object-local [M 0x1ce8a8:160-200]) and follow the host's live matrix each tick
  (Effects::follow_hosts); hosts leaving the world free their holes (Effect_RemoveSObjs 0x1cf360);
- a cel-hosted hole draws only while its cel is drawn (Scene::decal_cell_entries /
  hide_unseen_decals [M View_DrawObjectGlist 0x1e6780, View_DrawCelGlist 0x1e6538]);
- debris keeps the draw slot it is handed at birth (Effects::assign_debris_slots, next-fit), so a
  full pool no longer slides later pieces between two debris on interpolated frames
  (Scene::debris_reused_entries);
- NF_AIM_SWEEP / NF_INTERP_SHOT / NF_INTERP_PLAIN render aids; tests added by the wip:
  stairwell_decal_interp_07000011_live (evicting burst, drawn slot must sit on a live hole),
  stairwell_decal_probe_07000011.

Plan from here:
1. Verify what remains: door-follow live test on 07000011's door holes #192/#213 with sister;
   room-visibility draw test with sister; debris-slot test with sister.
2. Fix whatever the tests expose.
3. Run every bullet/decal/debris/door/interp binary and walkreal_07000011_live; quote totals.
4. Full suite before each push (fast runner in background).

### progress 1 (20:16-21:16 UTC)

- Branch `s/decals-follow-doors` created from `origin/s/wip-decals-2` + `origin/master` merge (58aaf73e); WORKLOG
  start entry pushed (867473dc).
- The wip branch's three-gap implementation verified in place: DecalHost + Effects::follow_hosts (drone.rs:18603
  per-tick), Scene::decal_cell_entries/hide_unseen_decals, Effects::assign_debris_slots + debris_slot_map.
- NEW tests, all green (decal_door_follow_07000011_live, 4/4):
  - hole_on_the_stairwell_door_rides_the_leaf: hole shot into the stairwell door leaf (host Object(0x10000000000)
    [L], fresh hole 0.010 off the part's collision face [M Effect_Bullet hit+normal·0.01 0x1ce9f0]); E-open through
    the real input path moves the door matrix (delta 1.0) and the hole rides it (moved 0.816, still 0.010 off the
    moved leaf); the static floor hole next to the door moves 0.0000 (the fix must not drag static holes along).
  - the_follow_measure_flags_a_stuck_hole (sister): a hole left at p0 hangs off the moved leaf — the measure
    flags it, so the ride assert is not a blind zero.
  - a_wall_hole_draws_only_while_its_cel_is_in_the_frame: the stairwell shot's hole (cel 128, room of the landing)
    draws in the facing walk's set and collapses to FAR under the office-spawn eye's walk (cel out of set [L]);
    the all-visible mask never hides; the shot's hole has its own entry on it.
  - hiding_never_touches_visible_or_unmapped_cels (sister): hide_unseen_decals only touches mapped+unseen cels.
- Build env note: alsa-sys needed PKG_CONFIG_PATH + a resolvable libasound.so (fixed via the alsa-prefix + sysroot
  lib copy); recorded for the next session.
- Report's live checks: --bond spots for the door-follow render (below).
