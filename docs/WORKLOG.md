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

## guard-floor-keep — the bounds push input fixed, two walks shift

The driver's pushed-guard fall (blocked/10-guard-fell-through-floor.md) is a band-margin hole:
NDrone2_Collision 0x140e90:169-174 passes obj+0x30 (y = floor + hip_anim) to
GetBoundsPushVectorForSphere 0x157bd0 and the band is [Q.y - 0.4, Q.y + lerp(heights)] on THAT y
(0x157780:96-104); the port passed feet() = pos - 0.6, riding the hip drop — on 07000001's upper
west floor (walls #94-99 nodes y 9.65 over a 9.22 floor) the standing margin was 0.37, so any
crouch unbounded the wall and the guard's own dodge walked him into the void north of z 16.7.
Fix: push_out(dr.pos, 0.4) — the game's exact call (commit 402226e46). The shared walker grew
the guard-fall flag/trace (NF_GUARD_FALL_TRACE) and walkreal_07000001_live grew
no_live_guard_falls_through_the_floor_on_the_walk. 01/03/04/05/06/07/09/0a/0b + npc_doors +
fix_guard_misc all green; 07000002 00agent and 07000008 whole-mission shift (verified passing
pre-fix — the old input over-bounded walls 1.0-1.6 below the floor) and are owed re-tunes;
07000011 SIGKILLed twice (sandbox memory); the rest owed. Report:
docs/reports/guard-floor-keep-2.md.
