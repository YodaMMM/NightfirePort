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

## Unproven
The new timings are from the decompile, not a live capture.
