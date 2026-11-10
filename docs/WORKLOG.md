
## mp-bot-backoff
Started 2026-11-10 23:21 UTC. Branch s/mp-bot-backoff cut from origin/master 0ee8478fd.
Task: check the landed CanBackoff port (4c2203cc6, crates/nf-game/src/mp/bots.rs) against the
decompile; add the two halves it lacks — the DroneAnim_CanDoAnimState(anim 4) gate and the
behind-room NDrone2_MoveTest (-0.75 local-Z probe, cel-checked) — in the game's order
[M 0x1501a0:31-77]; pin via a real bot match; make mp_bot_objectives_live::
demolition_bots_shoot_the_site green again with the game's 4.0 too-close band; keep MP state
equal on every machine, no shared Rand draws; run every mp_*_live binary and mp_world_sync_live.

Work log:
- (23:21) branch created from origin/master 0ee8478fd, WORKLOG entry, checkpoint pushed.
  Next: fetch work-slim.7z for the decompile (work/decomp missing in sandbox), read
  NDrone2_CanBackoff 0x1501a0 in full, audit the landed port.
