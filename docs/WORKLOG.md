# WORKLOG

## guard-floor-keep — branch cut

Branch cut from origin/master 6a4fcf4ef. Task: continue blocked/10-guard-fell-through-floor.md —
07000001 (01), with solid guard bodies (0a9d39435) guard #4 (the ch-0xF stairwell-ambush pair,
upper west floor y ≈ 9.2-10.2) was pushed off and fell through to y ≈ -1e6 near (-28.7, 27.6),
which is void at every height (upper west floor spans z 0..17; its north boundary walls #94-101
close at z ≈ 16.7). The earlier attempt (s/guard-floor-keep) never pushed and is lost, including
its NF_GUARD_FALL_TRACE walker trace and the
walkreal_07000001_live::no_live_guard_falls_through_the_floor_on_the_walk red test — both rebuilt
here. Plan: reproduce through real input from the level load (no teleport; the driver's variant is
most plausibly the d3 00agent door-phase crowd run), pin the tick where the guard crosses the
north boundary, then fix the port so a pushed guard keeps the floor the way the game's bounds
push (AINetwork_GetBoundsPushVectorForSphere 0x157bd0) and floor handling
(Drone_FeetOnPoint 0x140660, Drone_CollisionHandler 0x140968) keep him. No lowering damage or
health, no loosened pins, no new ignores, no draw changes. Every walkreal_* binary run one per
call if guard movement changes.
