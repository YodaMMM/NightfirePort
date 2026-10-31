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
