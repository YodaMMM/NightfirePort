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
  the listener's camera value (≤1 / ≥2, sqrt blend between). Layers: idle = low·sqrt(1−load), pitch
  ×1.1; lowload = sqrt(load); mid/hiload = high·sqrt(load); cruz = high·sqrt(1−load). Vacuum
  one-shot when the load first passes 0.2; above 0.5 load the pitch gets sin(2π·t/10)·0.3·2/(t+2).
- Banks [D]: the engine keys resolve only on the Vanquish tracks (MIS01, MIS4, RACE), and
  `vanquish.h` has no `midload` keys. The other tracks' vehicles use other classes, not ported.
- Sniper bolt `FUN_00124650`: weapon state 3, anim time in (13.9, 14.1) → `SFX_sniperbolt`, slot 4,
  `POV Weapons`. It resolves only on MIS01 [D].

## Not traced / not ported

Which script event starts which stream: the stream names sit in the track `.crp` as `CHAR::`
strings. `FUN_0015d000` (the stream event: `speech` / `nis` / mix-group channel) is described but
its records are not decoded. Also not traced: the scrape counter writer, horn timing, collision
impact sounds (the `IMPACTSOUND*` attribute schema `FUN_00194e18` only), 3-D panning, traffic/AI
vehicle classes, and other weapon sounds. The viewer does not play any of this yet.
