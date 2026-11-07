# agent-lvl-01-02: levels 07000001 and 07000002

Moves the shared random stream: guards and channels now wake later, so RNG-pinned digests on any level can shift.

## Method
Every placement on both levels dumped with `nf-extract placements-all`, then each channel traced to a writer and a reader.
Every `GameState == 0x7000001/2` site and every hard-coded `switch_channels[n]` in the decompile was read against the port.
State update arms were scanned: decompile `case 3` bodies with logic against the port's `msg::UPDATE` arms.

## Fixed
1. **07000001 had no level music machine.** `UpdateMFX_02MayhewA` 0x111ed8 was not ported; only the shared exit block ran, so the
   music never left its start section. Ported: section 3 on the intro's end or first alarm, the 500-frame hold, the 5 bridge,
   the 0xb/0xe and 7/9 round-robins. Row from the disc.
2. **Death channels fired on the first tick of the fall.** The port raised them from msg 3 (update); the game raises them from
   msg 2, the state exit [M 0x1637f8 case 2, 0x163a80 case 2, docs/systems/drone-ai.md]. They now land at the fall's end, all levels.
   07000002's own 90-tick cap is added [M 0x1637f8:55-61, 0x163a80:28-33 + case 0xC]. Measured: 213 ticks on 01, 90 on 02.
3. **ch 0x35 removal.** A guard still falling with death channel 0x88/0x29/0x2a raises it and is removed when the bathhouse scene
   starts, instead of standing frozen mid-fall through it [M 0x1637f8:76-90].
4. **Saved geishas cowered for good.** `HostageHide`/`HostageIdle` had no update arms. Now: armed guard in sight within 20 hides her,
   none stands her up facing Bond [M 0x15cfe0:23-29, 0x15ce60:17-33]. HostageIdle's impact resume is 0xe, not 0 [M 0x15ce60:36-46].

## Checked, no change
Triggers (Hit, Calc, SEQ windows, Bond moments), SP states incl. the fax (scripts 0x06000016/17; the level doc's 96/97 was wrong and
is corrected), SS, sound triggers, hints, Destroy_Smash, shooter, special death, air and drowning, tuning factors, Mayhew's route lines,
the corpse alert (`DroneVision_ShoutFromOtherDrone` returns 0 in this build, so msg 0x15 moves nobody).

## Recorded, not changed
- Visual: `NDrone2_DSTATE_Hostage` 0x15cb78 case 1 calls anim 0x21 and case 5 `SetDAnimNow(0xf4)`; the port plays COWER 0x19.
  A captive geisha's pose may differ. Needs a look against the game.
- Other levels: the per-level music machines for 04, 09, 0a, 0c and 0d are also absent from music.rs (`UpdateMFX_02MayhewD`, `03TowerA`,
  `03TowerB`, `PowerStationA1`, `PowerStationA2`). Not touched here.

## Tests
New: `death_channel_timing_live` (3 of 4 red on old code; the fourth is the sister), `hostage_save_07000002_live::the_saved_geisha_stands_when_no_armed_guard_is_near`
and `music_unported_consumers_live::the_01_intro_end_starts_the_level_sections` (both red on old code), the unit `the_07000001_machine_intro_alarm_and_calm_sections`.
All 26 level walkreal binaries were run. Re-pinned for the later channel: 01 and 02 (wait for the channel), 14 (Rook read), 04 (the return-fire kill ends the mission mid-leg).
nf-game lib 689/0. drake_fight_live, fix_finale_live, fix_npcs_live, explode_live, hostage_killer, walk_01/02 green.

**Red on this branch:** `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage`. Green on old code, where Bond ended on 9.6 health.
The shifted pacing kills him in the pit squad fight. It is a thin walker margin, not traced to a game fault; it needs the batch re-pin.
Not run: digest pins and other non-walk binaries that hold the stream.

## Unproven
The new timings are from the decompile, not a live capture.
