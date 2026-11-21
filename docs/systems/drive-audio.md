# Driving missions: sound logic

Code: `crates/nf-game/src/driving/audio.rs` (unit tests there), disc test
`crates/nf-game/tests/drive_audio_logic_disc.rs` (golden `crates/nf-game/tests/drive_audio_logic.tsv`).
Data formats: `docs/formats/driving-audio.md`. Tags as there.

## Startup [M `FUN_00176658`, `FUN_002eb7a8`]

- Speech/music base per track: `uw_*` → mis11, `snow1a*` → mis3, `snow2a*` (incl. `_race`) → mis4,
  `paris_*` → mis01, `junglea/b/c*` → mis13a/b/c, else mis11. Archive `driving\<base><lang>.spe`
  (`en fr ge sp` at 0x376bb0), English when the localized one is missing.
- Menu volumes PTP+0x968 (music) / +0x964 (effects) over 100 → `1:Music` / `1:Effects`.
- Bank slots (`PTR_DAT_003562a0[slot]`): 0 Generic, 1 Collisions, 2 Smackables, 3 CarWeapons,
  4 Weapons, 5 Ambience, 6 Misc, 7 Actors, 8 `g_common.bnk`, 9 Engine.

## Mix groups [M `FUN_002f3058`, `FUN_002f2d08`, `FUN_002f34d0`, `FUN_002eb7a8`]

The ini sets the volumes and the trailing int (meaning not traced). Then `0:Fader2/1/0` and `Speech`
are forced to 1.0 and the tree is linked (`MIX_PARENTS`: most groups → `0:Fader2` → `0:Fader1` →
`0:Fader0` → `1:Effects`; `Landings`, `Player Grunts`, `Player Wind`, `Skids` → `0:Fader1`; `Music`
→ `1:Music`). A linked group the ini lacks keeps the ctor volume 0, so it is silent. Per track that
is `Friendly` on six tracks, `Player Car` on junglea, `Player Wind` on junglec, and six groups on
uw_mis11 [D, golden].
Effective volume = own × parent chain; a running fade adds its step per read and stops at the target.

## Vehicle voices

- Voice gate `FUN_002ddfd0`: ≤ 1/127 → 0, else ≤ 1.
- Ticks = sim ticks since the last audio update (`DAT_00355e60`, int).
- Base step `FUN_002e1768`: rpm wobble +0x120 ±150 at 5/tick. Three scrapes: a counter > 3 after
  `-= ticks` aims the level at min(speed/15, 1), slewed 0.04/tick, × `Scrapes`. Front/rear skid
  levels are slewed 0.04/tick toward the bridge's targets, × `Skids` × `SFX_SKIDMULTIPLE`. The voice
  per surface pair (front wheels d4/d8, rear dc/e0): paved/cobble `SkidFront1`/`SkidBack1` (`Squeal*`
  for non-0xE classes), grass/paved-rough `GrassSkid`, dirt/gravel/metal `OffRoadSkid`, ice/snow
  `iceskid*`. The rear grass and off-road voices play at 0.75 pitch. The rest of the function
  (after the ice skids) is not ported.
- Speed `FUN_002df7c0`: |v|; with a horn voice, (5,14) reads 14 and slews 0.1/tick.
- Player car `FUN_002f0900`: rpm floor 500; road noise = ramp((speed−15)/35) × `Asphalt`. On a wheel
  surface change the voice switches (1 Paved, 2 Gravel, 4 Cobble, 8 Ice, 9 Snow, 10/15 Off; others
  none). The ctor starts with `PavedRN`.
- Engine `FUN_002fa8f0` + `FUN_002ff320`: target = rpm + wobble − rear-skid·1000 (10000/0 by throttle
  when a rear wheel has no contact). rpm += 10 % of the gap, or −75/tick on a drop beyond 150/tick
  (then the load step ×3 toward throttle/8). The load slews 0.05/tick. Pitch = (rpm + 2000)/4500.
  Low/high split sqrt(1 − rpm/3000) / sqrt(rpm/3000) below 2000 rpm, else 0/1. Interior/exterior by
  the listener distance (the 3-D model's [5] slot, ≤1 / ≥2, sqrt blend between). Layers: idle = low·sqrt(1−load), pitch
  ×1.1; lowload = sqrt(load); mid/hiload = high·sqrt(load); cruz = high·sqrt(1−load). Vacuum
  one-shot when the load first passes 0.2; above 0.5 load the pitch gets sin(2π·t/10)·0.3·2/(t+2).
- Banks [D]: the engine keys resolve only on the Vanquish tracks (MIS01, MIS4, RACE), and
  `vanquish.h` has no `midload` keys. The other tracks' vehicles use other classes, not ported.
- Sniper bolt `FUN_00124650`: weapon state 3, anim time in (13.9, 14.1) → `SFX_sniperbolt`, slot 4,
  `POV Weapons`. It resolves only on MIS01 [D].

## 3-D [M `FUN_00147e30`, `FUN_002f4e00`, `FUN_002ec238`, `FUN_002f1f40`] (`sound_events.rs`)

- Listener: the camera matrix rows (right, up, forward), position = the camera record's +0x10,
  velocity +0x20, index 2, height band -100000 / 0 (inert unless an ambience region sets it).
- Gate: a sound with max² (+0x64) 0 or d² below it gets an update; else none.
- Per sound (ctor min 3, max 350): gain = 1 inside min, (max−d)/((max−min) + 8(d−min)) out to max,
  0 past it. Pan = atan2(local.x, local.z)/2π, 0 inside min/2, × (2d−min)/min inside min.
  Pitch = 400/((dir·v_src + 400) − dir·v_listener). Cone = dir · the sound's facing. Send =
  (d−25)/400 past 25. Slot [5] = d. Listeners other than index 2 get a fixed ∓0.25 pan.
- Pan law: azimuth = pan × 65536 + the patch's own (pan byte 64 → 0, 0 → 0xc000, 127 → 0x3fff).
  Stereo speakers at 0x4000 / 0xc000 (`FUN_0030cf90`), curve 2x − x²/127 (`FUN_0030c640`), 256 rows
  (`FUN_0030c668`). Speaker 0x4000 goes to SPU VOLR, 0xc000 to VOLL (`FUN_0030ec40` bytes +0x27 /
  +0x26, SNDDRV `SNDIOP_spusetvol` params 0x100 / 0x000). So + pan = right, local +x = right.
  Centre front = 95/127 each side. With the driving rule right = up × forward
  (driving-handedness.md) a source on the listener's right plays in the right channel (tests).

## Events [M] (`sound_events.rs`)

- Contacts `FUN_00149ed8`: |impulse| (+0x48); the other party's material 0xe (or types 4/7) → the
  landing call, else (type < 4) the impact call with the material code (jump table 0x36f0b0 [D]).
- Impacts `FUN_002e0890`: code 0xd → metal counter, 5 → sand, else rock (+3 while < 10; these are
  the scrape counters the base step runs down). Any counter > 3: no impact sound. Code 1 with a wheel
  down: `TiresHeavy` (Landings). Metal adds 0.35 (not on boats). Smackable vehicles (+0x168, every
  non-0xE ctor) clang `MetalGate2`/`3` alternately (slot 2, vol min(speed/5, 1)). Then a table
  (A: Cobble1..Heavy2, 12 rows; boats B: Light1..Heavy2, 10; boats on sand C: sandbump1..3; sleds
  none): the first row at or above the impulse (from row 1), skipping rows still cooling down, played
  if its threshold < impulse + 0.3; cooldown = now + 5. Volume min(speed/2, 1), slot 1.
- Landings `FUN_002dfa00`: only when a wheel was airborne last frame; volume min(strength + 0.1, 1).
- Horn `FUN_002dea48`: honks of 5..64 updates in bursts (count 3..5, decremented each held update
  without a honk); gap 5..34 between honks, 120..359 after a burst; all from the driving random
  generator `FUN_0025a5ac` (ported; it is shared with two other callers).
- Traffic `FUN_002e5598`: `SFX_<car>LdEn` / `LdEx` (slot 0) with rpm slew 200/tick, load 16/127 per
  tick, pitch (rpm + 4000)/7200 + a per-car offset, En/Ex × (1 ∓ cone/2) × 0.66; road
  `SFX_TrafficRN` (tank: `SFX_tanktreads`) by speed. Gear model `FUN_002df8a0` for horn cars.
- Car weapons `FUN_0020de20`: boost, smoke screen, two-wheel (slot 4, Weapons Fire), upload /
  download code (slot 0, Menu Sounds), dry fire.
- Disc [D, `drive_sound_events_disc`]: `SFX_CarAlarm` (ctor `FUN_002debc8`) is in no bank on any
  track; sandbump1..3 only on MIS11; the boat and snowmobile engine keys sit in slot 9.

## Streams [M] (`stream_events.rs`, disc test `drive_streams_disc`)

- `EStreamEvent` (0x14) words: 1 channel, 2 name, 3 fade seconds (int, -1 = wait for the current
  stream to end), 4 loop, 5 only while the player has health, 6 prebuffer. Handler `FUN_0015d000`:
  `speech` → the fader (skipped while the manager's +0x3cc and +0x3d0 are both set), `nis` → the
  fader as NIS (fade 0), `music` → the Music channel. Compares are case-insensitive.
- Channels (`FUN_002eb7a8`): Music reads `driving\<base>.mus` (the speech base minus its language),
  Speech reads `driving\<base><lang>.spe`. Entry = the name lower-cased + `.asf`. On the disc all
  121 (track, stream) pairs the eight tracks' scripts use are in the archive their channel reads,
  and `bondmove` is in every `.mus` and `.spe` [D].
- Channel update `FUN_002e8c70`: a waiting record with fade >= 0 fades the current stream out
  linearly over that many seconds, then the next starts in the same update; fade -1 waits. A
  prebuffered record opens held until `EReleaseStream`, or is purged after +0xb4 clock units.
- Fader `SpeechVsAmbience` (`FUN_002f8e00` / `FUN_002f9180`): a speech line sets `0:Fader2` (the
  parent of almost every effects group) to the ini's `0:Fade Effects` and the Music channel to
  `0:Fade Music` at once; when the channel is free again the level climbs 2/s and both follow
  level × (1 − duck) + duck. NIS lines do not duck and are dropped while a plain line plays. The
  free channel loops the ambience stream of the region the car is in (`FUN_00147e30`). Duck
  volumes per track are pinned (MIS01 0.3 / 0.4, MIS4 0.25 / 0.3 ...).
- Stinger: every `EAddScoreAction` plays `bondmove` on the stinger channel (default Speech → NIS
  path; `ESwitchStingerChannel` `music` → the Music channel, fade 0).
- Mix events: `ESetAudioMix` target = clamp((pct/100 + 1) × ini volume) over N reads,
  `ERevertAudioMix` back to the previous volume, `EResetAudioMix` every group to its ini volume
  (0 for groups the ini lacks). The group map is case-insensitive (`FUN_002f37e0`).
- `EPlaySound` (2-D, `Power Ups`) / `EPlaySoundPos` (3-D, `World Sounds`, words 5 / 6 = min / max)
  name the bank by its `banks.ini` path; every key resolves except `SFX_timerbeep` in
  `snox/g3_snowmb.bnk` on MIS3 [D].

## Vehicles, weapons, listener [M] (`vehicle_sounds.rs`)

- Class updates (vtable +0x24): tank `FUN_002f1000` (treads = min(speed/8, 1), 0 without rear
  contact, pitch level/2 + 1/2), heli `FUN_002f16b8` (rotor blend 0.9·b (+0.1 when moving);
  inside (< 2) three voices each at pans 0.25 / 0.75 / 0), ultralight `FUN_002e3c90` (dive / low /
  high layers by the climb rate, pitch 0.7..1.7). `SFX_TankTread` is in no bank [D].
- Listener `FUN_00147e30`: the main render view's camera matrix (and the velocity after it) while
  the view set exists, else the player body's matrix at the car position.
- Landing strength = max over counted wheels of compression × 5, ≤ 1 (`FUN_0018c1b0` /
  `FUN_0018cef8`, copied by the bridge). Horn request = the traffic driver's avoidance
  `FUN_001377c8` finding a blocker in its way.
- Fire `FUN_0014ca60` by weapon: ten 2-D `POV Weapons` `SFX_<stem>Fire`; rocket / missile / prox
  mine / cannon / grenade / torpedo 3-D (min 50, max 200; `Weapons Fire` for the player,
  `Henchmen Fire` for others; AI torpedoes `SFX_htorpedofire`); the gun only for AI cars
  (`SFX_hmntmachfire`, flying ones `SFX_HumntmachFire` at 0.75) and elements (`SFX_HMachGun1/2`
  alternating; `SFX_HMachGun2` is in no bank [D]); on MIS3 every other gun shot is skipped.
- Flight `FUN_00180bc8` (`SFX_RocketTrans`, `SFX_MissleTrans`, `SFX_EMPtrans`, torpedo keys,
  `SFX_UMissileTrans`; none for the MIS3 rocket and the snowmobile rocket), Q-Pulse hit
  `SFX_EMPdetonate`, lock warning `SFX_Htargetlock`, explosions `FUN_0015a098` (four keys round
  robin with a 30-unit cooldown each, plus `SFX_DistExp`; ranges ×5 on jungleb).

## Boat, snowmobile, turret, tank [M] (`class_sounds.rs`, disc test `drive_class_sounds_disc`)

- Boat `FUN_002e7358`: base step, then `SFX_load` (pitch speed/20, × 0.85 with the brake; volume
  own × group × (rpm + wobble)/3300 × a 10-step ramp up while gas or brake is down, down after
  release), the steer-sound servo one-shots (`SFX_servon2/1` on GAMEACTION_SOUND_STEER* 0xc..0xf,
  `SFX_servoff2/1` on the releases 0x10/0x11), `SFX_jetcav1` while moving with the brake down
  (pitch (rpm + wobble) × 0.0005556 + 0.2), `SFX_rudcav1` while moving and steering or on any
  vertical steer, both fading 0.06 a step after release; the rudder's silent end plays
  `SFX_rudcav3/2` (slot 9, `One Shots`). Creaks: `ESetCreakLevel` (action 0xff) sets level / interval;
  every interval ± N(0,1)·10 % steps a 2-D `SFX_creak<level>` (slot 0, `Ambience`). On MIS11 the
  script uses levels 0..3 with interval 240 [D]; every boat key resolves there [D].
- Snowmobile `FUN_002edd88`: the `SFX_windy` wind (slot 4, `Player Wind`) drifts level 0.4..1 and
  pitch 1..2.8 at rates redrawn from the driving generator, × min((speed − 5)/5, 1) × the forward
  speed; the road / carve pair (`OffRoad` × speed/20, no listener gain, silent unless the listener
  distance is below 2): front `SFX_roadcarve` on paved / `SFX_carve` on 3, 8, 9; rear
  `SFX_roadRN` / `SFX_snowRN` from the rear-right wheel and last frame's front-left one (the
  game's offsets +0xe0 / +0xe4). Landing thumps `SFX_heavy1` / `SFX_med2` / `SFX_tiressnow` by the
  front compression against 2.5 / 1.67 / 1.25 × the rest length. Engine: rpm 0.9/0.1 smoothing,
  pitch (rpm + 2000)/7500, idle × (1 − min(rpm/1000, 1)), cruise × the rest; `SFX_rev2` / `SFX_rev1`
  on take-off with the throttle open, silencing idle and cruise until it ends; `SFX_freewheel`
  when the rear loads again under throttle. Every snowmobile key resolves on MIS3 [D].
- Turret `FUN_002e4980` (from the base step when a vehicle has one, and from the tank): per axis
  the loop `SFX_TurretLR` / `SFX_TurretUD` while moving, the `...Stop` sample on stopping
  (`Weapons Init` × 0.5); the servo `SFX_Kachung` + `SFX_servo` on start, `SFX_servodown` on stop
  (slot 4). The LR / UD keys are in slot 0 on MIS13A/B/C; `SFX_servo`, `SFX_servodown` and
  `SFX_Kachung` are in no slot-4 bank on any track [D], so those are silent in the game too.
- Tank `FUN_002f1000`: base step, then the same engine object as the player car (max 4500, no 500
  rpm floor), treads, turret. `SFX_TankTread` is in no bank; the engine layer keys only on MIS01,
  MIS4, RACE [D].
- Landing strength: the wheel query `FUN_001931f8` is car +0x210[wheel] != 0, last pass's
  compression, so only wheels that were in the air count.
- Horn blocker `FUN_00135530`: within 0.9 (dot) of the heading to the driver's target, and either
  within the range of the target or nearer the car than the target.
- Game clock `FUN_00261038`: timer-1 interrupts at the video mode's rate, 60 a second on an NTSC
  boot (`FUN_00260d38`, table 0x3c1a38 [D] 60/60/50/60): impact cooldown 5 = 1/12 s, explosion 30 =
  0.5 s.
- Ambience level: the disassembly confirms (y + top + depth) × 0.1 inside the band (all float adds).
  Every `Envi` region has top -1000, depth 10, except `nearvilla` (top 0) on MIS4 / RACE [D]; every
  road node is above -1000, so the level is 1 everywhere except `nearvilla`'s 0..1 ramp over
  y -10..0. Every region's ambience stream is in the track's `.spe` archive [D].

## Not traced / not ported

The sled's +0x50 vector (from `FUN_001bcca0(1.0, view set +4)`), the writers of the turret
inputs (+0xb8 / +0xbc) and the meaning of the bridge's +0xb0, the voice "finished" test (taken as
the sample ended), what a channel's +0x48 does, and the writers of `DAT_00355e6c` / `DAT_003c82c8` /
`DAT_00355e88`. The viewer does not play any of this yet.
