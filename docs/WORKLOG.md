# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## mp-object-deleted

2026-11-07 10:46 UTC, branch `s/mp-object-deleted` (from origin/master 10956badb).

Task: bots drop a goal whose objective object is gone. `control_delete_object`
0x133178:20-22 calls `MP_objectBeingDeleted` 0x18ac90 in a match, clearing every
mode-object row holding the deleted object and the 64 `MPObjects` slots, then routes
msg 0x3D to BotGlobal [M :66-278]. Port side: `object_being_deleted` and
`get_obj_ext_obj` exist in crates/nf-game/src/mp/mod.rs but nothing calls them outside
tests. List every mode row 0x18ac90 clears against the port's stores (flag and bases,
GoldenEye, Hill, Uplinks, Demolition, Protection); find which objects can really be
deleted during a match per scenario; wire the call from the port's delete path with
`BotGlobalOutcome::GoalCancel` in the game's order plus the 0x40 re-target from the
site getter; remove any stand-in. State equal on every machine online, no shared Rand
draws. Pin through a real match (bot leaves the goal the same tick), else say so and
keep a unit pin. Run every mp_*_live binary, mp_world_sync_live and nf-game.

- [x] setup: repo survived at /home/z/my-project/NightfirePort; ISO salvaged from the
  stale /home/z/my-project/nf copy (7.8 GB freed); branch cut from origin/master
  10956badb; WORKLOG started; pushing checkpoint before code work.
