# WORKLOG

## playscript-deaf-0x168
Started 2026-11-05 05:45 UTC. Branch s/playscript-deaf-0x168 (cut from origin/master a89818235).
Task: scripted guards on 0x6000168 ignore gunfire while their script plays. NDrone2_DSTATE_PlayScript
0x15ba68's ENTER sets +0x4F8 |= 0x400000 on three scripts — the shared tail of 0x6000168 and 0x6000169
(decomp line 73) and the 0x60007ac arm (line 84) — but the port's (PLAY_SCRIPT, msg::SOUND) arm in
drone.rs (~line 7393) tests only 0x6000169 and 0x60007ac, so a 0x6000168 guard breaks out of the script
on gunfire where the game's stays deaf [M 0x177800:19-21]. Confirm against the disassembly at
0x15bb50-0x15bba0; read the three clears (decomp lines 128, 135, 153) so the deaf window ends on the
same messages for all three scripts; fix the comment citing :36-44. List which levels place a guard on
0x6000168 [D]; pin it on one of those guards with real gunfire and pin the guard hears once the script
ends (unit pin if no disc placement). Run cover_nodes_live, fix_06_alarm_live, the ai binaries and
nf-game; suite via tools/test/suite.py in the background + once more on the final tree merged with
origin/master. Sandbox expires ~07:45 UTC.
Done 2026-11-05 ~09:40 UTC (continued across gateway restarts; sandbox window moved to ~10:35 UTC). The
(PLAY_SCRIPT, msg::SOUND) deaf window covers 0x6000168 now (e7c3c595f): the 0x168/0x169 arms share the
ENTER tail SetScript -> AttackTalk -> +0x4F8 |= 0x400000 [M 0x15ba68:73; disasm lw/lui 0x40/or/sw at
0x15bbf4-0x15bc08], 0x60007ac sets its own OR [M :84; disasm 0x15bc40-0x15bc54], the other arms have none
[M 0x15bb50-0x15bba4]; AlertSound 0x177800:19-21 returns 0 under the bit; all three clears (&= 0xffbfffff)
sit at case 5 [M :128/:135/:153] and the port zeroes start_script unconditionally in its ANIM_DONE arm, so
the window ends on msg 5 for all three. The :36-44 citation fixed. Census [D]: NO disc placement resolves
to 0x06000168 (dead arm on retail data; DefaultInit 0x14b300:611-627 the only +0x554 writer); exactly two
deaf-script placements: 07000001 -> 0x06000169, 07000012 -> 0x060007ac (drone_start_script_census.rs, green
on the disc). Unit pin drives the real ENTER and pins deaf-through-gunfire then hears-after-msg-5 (mode 9
alert 1.0 -> Attack [M 0x177800:51-68]); pre-fix red run reproduced the symptom (70dfa5dbd). Runs: nf_game
667/0, nf_formats 92/0, nf_audio 23/0, nf_view 100/0, census 1/0, cover_nodes_live 3/0, fix_06_alarm_live
2/0, all 11 ai binaries green, npc_doors_live 6/0, fix_guard_misc_live 6/0, fix_guard_weapons_live 3/0,
guard_first_shot_01_live 2/0, corridor_guards_07000012_live 1/0, start_corridor_guard_07000011_live 8/0,
abseil_elevator_live 3/0, nis_coder_break_live 3/0, vision_bit_census 1/0. Suite sweep (resumable
foreground runner; background processes are killed in this sandbox): 2955 passed, 9 failed, 106 ignored
across 631/1114 binaries; the 9 failures are all environmental (the 3 allowed disc_golden + 4 GPU-adapter
panics in fix_0c_pylon_live/glass_guards_14_live/missing_scenery_live — no GPU here — + identity_disc,
which needs the full 2.44 GB image, the sandbox has the 1.06 GB onfoot cut; nf-formats/src untouched by the
branch and the same failure shows in the pre-fix session log). mp_online_failures killed/hangs in-sandbox
(real sockets), standalone too. Report: docs/reports/playscript-deaf-0x168.md.
