# WORKLOG

## fx-npc-footsteps
Started 2026-11-05 00:45 UTC. Branch s/fx-npc-footsteps cut from origin/master 17a784980
(already carries the audit-effect-calls merge). Task: port row 2 of audit-effect-calls's
"Unported calls" table — the NPC footstep Effect_Create for NPCs (AnimObjectUpdate
0x1befd8:160-201, Script_Interp 0x1d9a08 case 2): the per-frame stride accumulator (stride
1.65, 0.4125 when obj+0xf6==1), `Effect_Create(1.0, obj+0xdc hitdata, obj, foot|0x42)` on wrap
(sound 0x40 + decal 0x2), suppressed while ScriptCam!=0, run branch = Sound_Play3D(0x286) only.
The stored hitdata at obj+0xdc the port does not keep per-NPC. Shell casings out of scope.
