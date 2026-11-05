# WORKLOG

## audit-voice-triggers
Started 2026-11-04 23:29 UTC. Branch s/audit-voice-triggers (cut from origin/master f503c15c7).
Task: every caller of the drone voice path in ACTION.ELF (Drone_SfxInfo lines, speech and bark
functions): the trigger, the condition, the repeat delay, the line; whether the port plays it
on that trigger. Include allies, civilians, bosses and the player. Fix every missing or wrong
trigger with a clear port site, pin each with a test on real level data; report the rest as a
table. Merge origin/master first.
Progress 2026-11-05 00:33 UTC: caller inventory complete (36 SpeechSFX + 1 SpeechSFXd + 10
PlaySFX sites in 33 caller files); 4 missing triggers fixed in drone.rs (RunAwayFromObject
ENTER death bark 0x168aa8:21; UnderCoverFire 0x16a3e0:33 / UnderCoverSniperFire 0x16a610:20
ENTER attack barks; PlayScript 0x6000168/0x6000169 ENTER 0x15ba68:72 / msg-3 re-bark :120 /
msg-5 :157 — new field play_script_talk_gate). Pins: 3 lib tests + voice_triggers_live
(2 tests: real Drone_SfxInfo rows; the frag-flee drives the ENTER bark on real 07000005 data).
nf-game release suite 1066 passed / 0 failed; nf-formats 92+11 with the 3 allowed disc_golden
failures; nf-audio 28. Report next.

## audit-voice-triggers-2
Started 2026-11-05 05:31 UTC. Branch s/audit-voice-triggers-2 (continued from origin/s/audit-voice-triggers, merging origin/master).
Task: the four added triggers read correct against the decompile, so no draw is removed to make a test pass unless a line is proved wrong. Merged with master these four are red, green without the branch: walkreal_07000001 following_mayhew_down (AND-touch never fires, walkreal_07000001_live.rs:1522), walk_from_the_spawn_to_the_exit_with_the_contact_real_damage (Bond dies tick 3253), walkreal_07000003 walk_the_full_route (east lane cap), smg_close_range_live (pistol control at 1 u). For each, say which draw moved what, then re-route or re-pin the test, keeping every pin honest. Run every walkreal binary and the full suite, quote the total line.
Progress 2026-11-05 05:31 UTC: branch cut, WORKLOG started.
