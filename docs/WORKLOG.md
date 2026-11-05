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

## broken-object-bark
Started 2026-11-05 04:18 UTC. Branch s/broken-object-bark cut from origin/master 17a784980,
with origin/s/audit-voice-triggers merged (2bc25c6c7 — not on master). Task: the port never
calls `Drone_BrokenObject` 0x138588 from `Break_Kill` 0x12b970 (breakable.rs only documents it):
level 0x7000001 only, a player-broken script-type-5 object, live type-0x0c drones, bark
0x40 + Rand(4) [M 0x138588 — reading the decompile in full next; audit-voice-triggers's
"Left as a table" row 1]. Plumb it from the Breaks kill path to Drones, game Rand order and
every gate; pin: Bond shoots a qualifying 07000001 object with a qualifying guard alive → the
line plays; another level or a non-player break → nothing.
