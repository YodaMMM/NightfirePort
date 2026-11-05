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

Progress 2026-11-05 10:18 UTC (audit-voice-triggers-2 continued, sandbox reset, rebuilt from scratch): smg_close_range_live three honest fixes, test still red. (1) The pistol (0x2) is semi-automatic — a held trigger fires once per pull, re-arms on release [M gun.rs Player_WeaponFiring — "the semi P2K held the same way fires ONCE"]; fire_at now presses 2 ticks/releases 4 through the same real input path, so a leg can actually spend its rounds (the old loop fired exactly one round per leg — master's pass hid this). (2) torso_aim's filter dropped parts 4 AND 5 as "head bones", but the port's damage law has one head part: hit_damage's match is 5 → DroneDamage_Head, 0x14/0x15/0x17/0x20/0x23/0x27 → Arms, 0x31..=0x38 → Legs, everything else (4 included) → Torso [M damage.rs 0x145558's table; pinned by nf-game hit_damage_per_part_and_difficulty h(4,2,1)=5.25 torso, head=false]; on the 0x5000039 rig part 4 IS the chest (half 0.2×0.25×0.15 [L the pistol-leg box dump]) — excluding it aimed at 0.07-half arm boxes the shifted cone draws miss. 1 u leg green: 6 rounds, 5 impacts, 26.25 drop. (3) The pistol legs take the SMG legs' clear-eye-to-torso gate plus the candidate retry. Remaining red: at 2 u d#11/d#6 sit behind the hall cover (0 impacts) and d#5 DIES to the escort mid-leg (hp 12→0 with 0 player impacts — the contact's volley steals the kill), exhausting the live roster. Next: re-route the 2 u/6 u legs to candidates off the escort's line, or assert the escort's interference explicitly. NOT STARTED: walkreal_07000001 walk_from_the_spawn_to_the_exit_with_the_contact_real_damage (Bond dies t3253 — drone #6 0x1c shotgun, 6 pellets one frame at 2.5 u after the RNG shift), walkreal_07000003 walk_the_full_route (east lane: Bond dry clip 1 spare 0, ally can't kill drone 11 exposed false, no pickup in corridor, 6000-tick cap).
