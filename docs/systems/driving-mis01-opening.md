# MIS01 (Paris Prelude): the opening sniper section

What the original shows (PCSX2, reached through the real menus: Play without saving, Operative,
Paris Prelude) [L 2026-11-20]: the mission does not start behind the wheel. Bond holds the
**Sniper Rifle** (HUD "24 / Sniper Rifle") on a moving vehicle above the riverside road and
shoots at pursuing cars below (black saloons and a truck), at night with lamp-lit facades. The
driving executable is already loaded at this point, so the section runs in DRIVING.ELF.

Evidence tags: [M] DRIVING.ELF decompile (EE addresses), [D] disc / ELF data bytes, [L] live
capture, [S] suggested (not proven). Port: `nf_formats::drive_script` (the script decoders) and
`nf_game::driving::pov` (weapon table, loadout, gunner, mode flag); disc test
`crates/nf-game/tests/driving_sniper_live.rs` (needs `NF_ISO`).

## 1. The vehicle is the helicopter

- The `MSet` name `"helicopter"` [D] is the player vehicle name: `FUN_00204d18` stores it at
  mission +0x1b44 (the `lotus`/`playervan`/… fallbacks are player cars) [M].
- `data\sim\attrib\pvehicle\helicopter.atr` [D]: `RENDER_FILENAME.s=helicopter.crp`,
  **`SECONDARY_TYPE.s=vanquish`** — the Vanquish is the helicopter's secondary vehicle
  (the boot loads `SECONDARY_TYPE` as the secondary, driving-load-path.md §3.6).
- Bond is not in the car model: he is a separate actor (`AICharacter`, 0x170 bytes, vtable
  0x3696e0) built by `FUN_00123950` and mounted on the vehicle's `AIC_HSNIPE_POS` /
  `AIC_HSNIPE_ROT` points, model `sniper`, anim set `6.a_snipe.b_snipe.70` [M+D 0x3275d0..].
  `paradis_car` / `c_domm1` are an AI element (AIEl record 23, §4), not the player.

## 2. The `MSet` is the weapon loadout (correction)

drive-mission-scripts.md §3 read `MSet` +0x2c/+0x4c as "objective channels". They are weapon
slots: `FUN_00205498` calls `FUN_0020d378(DAT_0033a530, i, count, …)`, and `FUN_0020bbe0` fills
`DAT_0033a530`+0x10 by copying the 32-entry weapon table `FUN_0020f2b8` builds at 0x4165c0
(stride 0x54, 0x4165c0..0x417040) [M]. Entry fields used: +0x04 name, +0x14 flags (bit 0
owned, bit 2 clip dry), +0x18 rounds carried, +0x1c clip, +0x20 clip size, +0x24 last shot
tick, +0x34 (the MSet third array; consumer not traced), +0x38 shot counter on, +0x3c refire
ticks = `(int)(seconds[i] * fps)` from 0x33a030 [D], +0x44 sound stem.

MIS01 enables weapon **2 Missile, 30 rounds** and **26 (0x1a) Sniper Rifle, 24 rounds**
(clip sizes 4 and 30 from MSet +0xcc) [D]. The load selects the first enabled index past 0
with a count: the Missile [M 0x205498]. `FUN_0020d378` overrides the clip size and caps the
clip at the count only while the POV mode is on (manager +0x350) [M].

## 3. The POV mode flag and the weapon swap

`DAT_0033a024`+0x350 (the "auto-drive camera" / POV mode) is written only by `FUN_00206398`
[M]; its callers are scripted events:

| event (kind) | handler | effect |
|---|---|---|
| `ESetAutoDriveCamOn` (77) | 0x1576e8 | when off: camera start `(angle/360, pitch/360)`, mode on, gunner shown |
| `ESetAutoDriveCamOnMovie` (234) | 0x157838 | the same, but `FUN_00206398`'s third argument 0 skips its `FUN_00167d80`/`FUN_00166cb0` calls (not traced) |
| `ESetAutoDriveCamOff` (76) | 0x157460 | mode off, gunner hidden |
| `ESetAutoDriveCamOffMovie` (233) | 0x157540 | gunner hidden, mode off |

Each frame the weapon manager `FUN_0020c078` swaps: mode on with a non-POV weapon current →
stash it (+0x0c) and select the first owned POV weapon (ids 0x17..0x1f) with rounds; mode off
with a POV weapon current → restore the stash (or weapon 0, or the first owned) [M]. So the
Missile is stashed and the **Sniper Rifle** becomes current — the HUD's "Sniper Rifle" [L].
A POV kind is `id - 0x17` (sniper = 3) [M 0x123950, 0x20c078].

## 4. The script that runs the section

The `Map ` record's window holds `el` (event list) and `sr` (string) instance records besides
the chunks; `Trgr` +0x18 and `Rule` +0x0c are `el` references, event parameters use `sr`
references (layouts in `nf_formats::drive_script`) [M walkers 0x22c890 / 0x2095f8, D refs].
Event kinds index the factory table 0x36fda0 (`FUN_001757b8`); names come from each class's
typeinfo accessor [M]. MIS01 [D]: 868 lists, 2787 strings, 754 triggers, 114 rules; every
trigger/rule reference resolves; four `EPlaySystemAnim` strings are absent from the archive.

1. **Start.** World init `FUN_0022cd30` fires every trigger whose list holds `EStartMission`
   (0x16) [M]: only trigger **684** (list 685) [D].
2. **Sniper intro.** Trigger **685**, radius 2.5, 0.9 units from 684 at (778.3, −7.6, 207.4),
   fires list 686 [D]: `EControlToSpline` (the player vehicle onto spline `AISp` offset 0),
   **`ESetAutoDriveCamOn(-60°, -20°)`** (the only one in the mission), throttle 0.71,
   `EChangeStage(19)`, music stream `snipeintro`, objectives 2800–2803 (current 2800),
   letterbox on, four power-ups disabled, `ESetReflectiveObject(AIEl 23 = paradis_car)`.
3. **The phase.** `EChangeStage(n)` stores the next rule group (`FUN_00149828`: mgr+0x10);
   `FUN_00206340` runs the default group and the current one each update, then makes the next
   one current; `ECallStage(n)` runs group n once (`FUN_00206310`) [M]. Stage 19 holds rules
   3, 45, 46, 79, 80, 85, 105, 107, 113 [D] (AI commands to the pursuing elements, timers,
   stage calls). Trigger 712 (544.9, −1.9, 458.7) changes to stage 5; trigger 727
   (−320.6, −10, 117.9, radius 74) to **stage 13** [D].
4. **Hand-over.** Stage 13's rule **34** (`SRuleSpeed`) fires list 789 [D]: letterbox off,
   audio mixes, `ESetAutoDriveCamOnMovie(0,0)`, `EEndControlToSpline`,
   **`ECustomTransition("paris")`**, `ESetAutoDriveCamOff`, **`ESetVideo(paris2.mad,
   paris2.sub)`**, `EResetPlayerCarPos(trigger 683 at (−321.2, −0.1, 66.3))`, AI elements
   off. A second route, trigger 741 (radius 80 near the start) → list 742, carries the same
   transition, video (no subtitle file), spline end, mode off and reset [D]; which route a
   normal run takes is not proven.
5. **`ECustomTransition("paris")`** `FUN_0014b020` [M]: the player car swaps through its vtable
   +0x294 / +0x8c (not decomposed — the swap target is the `.atr` secondary, the Vanquish
   [S]); the MSet copy loses every POV weapon (enabled 0x17..0x1f cleared), then weapon 0
   ("Gun", stem `mntmach`) gets 500 rounds — or, when `DAT_0032e690` is set, the Missile count
   is zeroed — and `FUN_00205498` re-registers from the copy (inventory reset first).
   `DAT_0032e690` is set only on `paris_mis01` with PTP+0x618 == 1, +0x984 == 0, +0xa18 == 0
   [M 0x174bf0:269-276]; what that state is has not been traced.

The ported run over the real data (`the_phase_runs_on_the_real_script`): load → Missile
current; list 686 → mode on, Sniper Rifle current with 24 rounds; list 789 → mode off, rifle
gone, Gun 500 + Missile 30, Missile current, video `paris2.mad`, car reset to trigger 683.

## 5. Bond's gunner (fire, reload, scope)

The update `FUN_00118af0` calls vtable slot `0x44 + 8*state` (state = +0x4c) [M]:

| state | handler | behaviour |
|---|---|---|
| 8 | 0x1243f0 | spawn/ready anim; at its end → 2, then a pending switch |
| 1 | 0x124240 | switching: waits on 8 (ready) or 0xd (put-away, then the ready anim) |
| 2 | 0x124348 | idle loop; a pending reload starts; a changed scope wish toggles |
| 9 | 0x124440 | scope toggle (scope-up anim 7 / scope-down anim 6 for the sniper) |
| 10 | 0x124570 | scoping; at anim time (4.9, 5.3) going up: model hidden, scope view on |
| 11 | 0x124650 | fire anim; sniper bolt sound `SFX_sniperbolt` at time (13.9, 14.1) |
| 12 | 0x124770 | reload anim |
| 13 | 0x124800 | put-away; at its end → 0 with the new kind |

Anim indices per kind (sniper column): ready 4, idle 1, put-away 3, reload 5, fire 0/0, scope
up 7, scope down 6, spawn 2, bank 0xe [D 0x3274b0..0x327650].

Fire (`FUN_0020de20`, POV branch) [M]: no rounds → nothing; Sniper Rifle with manager +0x5ec
set and the scope object's +0x2c clear → `SFX_DryFire`, no shot; held during the switch
cooldown (+0x50), before `last + refire` (sniper 5 ticks at 60 fps), or while the gunner is
not idle; a shot spends one round from the carried count and the clip; an emptied clip sets
flag bit 2. The empty-clip loop in `FUN_0020c078` starts the gunner's reload once
`last + refire − 5 < now` and clears the bit; `FUN_0020e978` refills the clip to
`min(carried, clip size)` once the gunner is idle again. A reload request (`FUN_0020e868`)
needs an idle gunner, a clip below its size and below the carried rounds.

Scope input (`FUN_001be038` cases 0x38 press / 0x3a release) [M]: press raises the scope
wish; with PTP+0xa44 set (toggle) a press while up lowers it, otherwise the release lowers it.

## 6. Not proven / not ported

- The hand-over rule's condition: rule 34 is `SRuleSpeed` with payload `[5, 0, 1.0]`; the rule
  evaluators are not decomposed (drive-mission-scripts.md §5), so what speed/progress fires it
  is open. Trigger contact semantics (what makes 685/712/727/741 fire) are not traced.
- The event queue order (`FUN_00175730` allocates, the class destructor runs the effect): the
  port runs a list in its stored order [S].
- The `el`/`sr` references → runtime pointers loader is not traced (the references resolve
  against the `Map ` window's instance records on every MIS01 record [D]).
- Not ported: the aim/camera matrix and scope sway `FUN_00124848`, bullet hit tests, the
  pursuer AI commands (`EAICommand` → `AICo`), the spline follow, the movie, the vehicle swap
  itself, the zoom-fraction blend (+0x160), the scope object's own guards. The port's driving
  view (`crates/nf-view/src/driving.rs`) still starts in the Vanquish and has none of this.
- Live checks that would settle the open parts: break on `FUN_00206398` (expect arg 1 from
  list 686, 0 from list 789/742); break on `FUN_0014b020` and read which list ran; watch
  `DAT_0033a530`+0x10 + 26*0x54 + 0x18/0x1c while firing (expect 24 → 23, clip the same).
