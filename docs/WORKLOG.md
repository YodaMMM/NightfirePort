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
Progress 2026-11-05 05:30 UTC: `Drone_BrokenObject` 0x138588 read in full (decompile + disasm
0x138588-0x1386d0): gates bDisableSystem → level 0x7000001 → hitdata+0x58 is a kind-5 BULLET
from the player's weapon system (+0xe0→+0x30 == glb_players) → the list walk (dtype +0xc5 ==
0x0c, health +0xac <= 0 ABORTS the walk [M 0x138628-0x138634 bc1tl], body-obj kinds 2/0x11),
one 0x40+Rand(4) per barking drone [M 0x1497e8], NDrone2_SpeechSFX(line, 0,0,0). The explode
path's hitdata +0x58 is the breakable itself (kind 0x20 [M 0x17c518:163-164]) and the
walked-through pane's is the drone (kind 2 [M 0x12bfd8:24-33]) — both fail kind-5 draw-free;
only the player-bullet kill (Break_Update's last contact, +0x58 = the bullet [M 0x1ea398:189])
barks. Port: `Breaks::kill_by_bullet` (the bark walk inside the kill, between the sound roll
and the debris draws, the game's order) + the caller turns the pairs into talks pushes; the
plain `kill` unchanged. Pins: broken_object_bark_live 4/4 (07000001 positive, another level
silent, explosion silent, the abort/type gates). nf-game FULL 1070/0;
interactables_07000001_live 10/0; walkreal_07000001_live 7/1 — the failure (and a second one)
exists on the merged base without my change (stash-verified ×2; plain master 8/0 in a clean
worktree): the voice-triggers merge's bark triggers (and now the game's own Break_Kill Rand(4)
draws) shift the walker's combat RNG. The full suite.py was killed by the sandbox's
background-job rule three times; the nf-view remainder did not fit. Report next.
