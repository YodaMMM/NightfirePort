# WORKLOG

## fx-npc-footsteps
Started 2026-11-05 00:45 UTC. Branch s/fx-npc-footsteps cut from origin/master 17a784980
(already carries the audit-effect-calls merge). Task: port row 2 of audit-effect-calls's
"Unported calls" table — the NPC footstep Effect_Create for NPCs (AnimObjectUpdate
0x1befd8:160-201, Script_Interp 0x1d9a08 case 2): the per-frame stride accumulator (stride
1.65, 0.4125 when obj+0xf6==1), `Effect_Create(1.0, obj+0xdc hitdata, obj, foot|0x42)` on wrap
(sound 0x40 + decal 0x2), suppressed while ScriptCam!=0, run branch = Sound_Play3D(0x286) only.
The stored hitdata at obj+0xdc the port does not keep per-NPC. Shell casings out of scope.
Progress 2026-11-05 04:12 UTC: pinned the raise semantics from disassembly — `ScriptFootstepType`
0x30ce10 is cleared at the TOP of every `AnimObjectUpdate` (0x1bf0c4 `sb zero`, dropped by the
decompiler), raised by the op-4 footstep markers (0x1c1b38:372-401) during the object's own script
tick, consumed once after (0x1befd8:191-208); the distance accumulator is the player's raise path
only (obj kind 3 [M 0x1910f8:76]). Ported the second row-2 call site: `Script_Interp` 0x1d9a08
case 2's trailing footstep — the NIS anim stream's own consumption (4-unit down-ray from obj+0xc0,
`Effect_Create(rayhit, obj, foot|0x42)`, NO ScriptCam gate [M disasm 0x1d9f30-0x1da028]) — as
`Request::NisFootstep` (nis.rs case 2 collects the stream script's markers via the new
`World::anim_ops4_footsteps`; the drain applies them to the one global alternator then runs the
full Effect_Player arm with `World::anim_foot_matrix` palette slots 55/51). Pinned by
`nis_footsteps_live` (07000005's ledge cut-scene 06000994 stream 14, Bond 06000995's real markers
ev4@8/ev5@56/ev4@90): three prints foot 4/8/4, y = ray hit + 0.05, sound starts, while the
cut-scene's script camera runs. Door/guard + nis/anim/mp binaries green; the 107 walkreal_*_live
binaries re-running.
