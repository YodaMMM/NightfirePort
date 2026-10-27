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

## script-events-3-4

Task: port `Script_EventHandler` (0x1d94c0) Event 3 (the delay + spawn-transform bake) and
Event 4 (`Player_Disable`) exactly; sweep every level's scripts for Event 3/4 users [D dump].

Status: done 2026-10-27 ~02:56 UTC. Event 3's delay write is now skipped while a skip is armed
(+0xAB5 bit 0x1 [M 0x1dac78:56]; the port's fade_flags bit 1); the bake is structural in the
port's single-matrix Obj model and the full [M] chain is cited in the arm. Event 4 verified
end-to-end with a real-input walk test (the intro 060006d7 disables Bond from its first tick —
[D] the stream-7 dump corrects the brief's "key 153"; 06000959's keys are all identity, so the
[S] cel-vs-baked hypothesis makes no difference there — its Event 3 is the −1 hold). [D] sweep:
339 sites (303 Event 3 / 36 Event 4) across every 07-archvive, probe committed as
`script_events_dump.rs`; tests in `script_events_3_4.rs`; report
`docs/reports/script-events-3-4.md`.
