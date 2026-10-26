# WORKLOG

## fusebox-inverter-2 (2026-10-26, sandbox expiring ~08:16 UTC)
Branch `s/fusebox-inverter-2` = origin/s/fusebox-inverter merged with origin/master (0b25f0d5).
Task: `InverterTrigger_Update` (0x1b30a8) triggers built from type-0xF9 placements (p0 input,
p1 output) go red with master on four live tests — fix_guard_misc_live::probe_sniper_direct_bullet
and ::the_0c_sniper_takes_damage_when_shot (0700000c), interactables_07000041_live::
the_searchlight_family_sweeps_as_one_light and ::the_wrapper_ends_into_the_space_station_after_
the_chain (07000041). Each is green on master. Plan: list the 0xF9 placements on 0c and 41 and
what the new inverters do to the channels those tests depend on; port the game's exact create
gate for 0xF9 [M parsemap_create_dynamic_objects 0x1d04f0's 0xF9 arm + InverterTrigger_Create]
and the update's exact rule [M 0x1b30a8]; where the game really inverts those channels, fix the
tests to the game's behaviour with evidence. Goal: all four green + the 14 inverter test; full
suite on the merged tree.

### fusebox-inverter-2 — the 0xF9/0x41 split resolved (2026-10-26 ~06:55 UTC)
The branch's premise was wrong on both ends, from the decompile + the disc: `parsemap_create_dynamic_
objects` 0x1d04f0 case **0xf9** calls `SoundTrigger_Create` 0x1b2628 (class 0x40) and case **0x41**
calls `InverterTrigger_Create` 0x1b3040 (class 0x53, p0→+0xF4, p1→+0xF6) [M 0x1d04f0:265-268/504-510];
`SoundTrigger_Update` 0x1b2790 stores NO switch_channels (p0 ON-gate, p1 one-shot flag, p2 sound id,
p4 OFF-gate, p6 difficulty) while `InverterTrigger_Update` 0x1b30a8 is `switch_channels[in] && out!=0 →
switch_channels[out]=0` (the raw OUT FIELD gate) [M]. Master's worldobj.rs already owns the 0x41
inverter with exactly that tick. The four reds were the branch's inverters built from 0xF9 rows whose
p1 (=1, the one-shot flag) became an output channel: on 07000041 the two voice rows #89/#90 (p0=1)
cleared ch1 while the chain was on (searchlight + wrapper-end red); on 0700000c all five voice rows
(#456/#541/#550/#562/#563, p1=1) built ch1 writers and the yard sniper #309 waits on ch1 [D param 3=1].
FIX: removed the 0xF9 trigger path + the INVERTER trigger kind; rewrote 07000014_inverter_trigger_live
to the game's data (18 0xF9 rows → sound triggers, no trigger from any of them; the four 0x41 rows
(0x0e→0xb8, 0x85→0x6f/0x22/0xd1) are the inverters; the sound-0xf1 voice rows incl. the 0x87/0x88
fusebox-cut gates never touch ch1 — the regression pin). All four reds + the 14 test green. The
fusebox cut's downstream events on 07000014 run through AND #391 [0x87,0x88] → 0x8C, Touch #456 →
0x0C, SEQ_AND #445 → 0x09 → SP #407 (script 0x06000848 writes 0x4A/0x11) — no inverter involved
[docs/levels/07000014.md §2]. Suite relaunched (the first run died on the ALSA pkg-config path).
