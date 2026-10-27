# WORKLOG

One entry per task, appended at the end. Never rewrite or drop another task's entry.

## script-events-3-4

Task: the script player's Event 3 (spawn-transform bake, `Script_Event` 0x1d94c0:98-116) and
Event 4 (`Player_Disable`, :118-121) fall to the port's generic unused arm (nis.rs and
drone.rs). On 07000013 the intro 060006d7 fires Event 4 at key 153 and the shutter script
06000959 fires Event 3 twice. Port both events exactly, sweep every level's scripts for
Event 3/4 users [D dump], list them in the report. Tests: after key 153 player input does
not move Bond [M]; the 06000959 Entity spawns use the Event-3 transform [M + D].

Status: started 2026-10-27 ~01:21 UTC. Setup: fresh clone (sandbox reset), branch
`s/script-events-3-4` created from origin/master @ ec917093.
