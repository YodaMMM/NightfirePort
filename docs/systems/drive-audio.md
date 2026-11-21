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

## Not traced / not ported

Which script event starts which stream: the stream names sit in the track `.crp` as `CHAR::`
strings. `FUN_0015d000` (the stream event: `speech` / `nis` / mix-group channel) is described but
its records are not decoded. Also not traced: the landing strength (+0x150) writer, the clock unit of
the impact cooldowns, which matrix the listener's camera record points at (the port uses the chase
camera's rows), the boat / snowmobile / helicopter / tank / ultralight updates (ctor keys only), and
the projectile weapons' fire jump table (`FUN_0014ca60`, key `SFX_<weapon>Fire`). The viewer does
not play any of this yet.
