# MP players — the characters (mp-character-data)

The MP body data: who the 29 playable characters are, which archive carries each mesh, and the
animation set a third-person body runs. Everything here is the game's own tables, read from
ACTION.ELF and the arena archives; the drawing itself is the app's (this task delivers data only).

Rust side: `nf_formats::mpanim` (the tables + the pure pose), `Scene::mp_bodies` (the parked
bodies), `nf_game::mp::MpSettings::players[slot].skin` (the per-slot character word).

## 1. `MP_skins` — the character table

`MP_skins` @0x26d488 (ELF symbol, 464 = 29 × 0x10 bytes) [D]. `Player_Init` 0x1910f8 reads it on
the MP path: `MPSettings+slot*0x30+0x24` (the character word) selects the record; disasm
0x1913c0-0x1913e0 loads record+0x00 into `AnimObjectNew(obj, skin)` and record+0x08 into
`BLData+0x965` (the sleeve) [M]. Record layout [D]:

| offset | meaning |
|---|---|
| +0x00 | the skin (anim object) hash, `0x0500xxxx` |
| +0x04 | the skin's DATA file hash, `0x0100xxxx` |
| +0x08 | the sleeve variant (4..7 for MP; `AnimObjectSetSleeveType`, the first-person arms) |
| +0x0c | runtime "selected this match" flag — 0 on the disc [D] |

The 29 characters, index = the MP character menu's list order (`Menu_GetMPSkins` 0x1ffc98 walks
the same indexes). Names [D — the menu's txt labels bound through the game's own arithmetic
(`Txt_BindLabel` 0x1cad60: `fixups[id>>24] + (id&0xffffff)`) against USATxt.dat; 14/28's labels
carry a high byte the port's `Lang::bind` resolves the same way]. Unlocks: `Menu_UnlockMPSkins`
0x1ff2f8 sets the per-row unlocked byte from mission rewards 0x26-0x37; 0-11 are unlocked by
default [M]. The `sleeve` column is the record's +0x08 [D].

| idx | name | label | skin | file | sleeve |
|---|---|---|---|---|---|
| 0 | Bond | 0x21a | 05000089 | 010001b4 | 4 |
| 1 | Drake | 0x21b | 050000a5 | 010001ec | 4 |
| 2 | Rook | 0x21c | 0500008e | 010001bc | 5 |
| 3 | Kiko | 0x21d | 05000092 | 010001c5 | 6 |
| 4 | Alura | 0x21e | 05000095 | 010001c9 | 6 |
| 5 | Dominique | 0x21f | 05000096 | 010001d5 | 6 |
| 6 | Snow Guard | 0x220 | 05000097 | 010001dd | 4 |
| 7 | Black Ops | 0x221 | 05000098 | 010001de | 4 |
| 8 | Yakuza | 0x222 | 0500009a | 010001e1 | 4 |
| 9 | Phoenix Commando | 0x223 | 050000c6 | 0100021b | 4 |
| 10 | Phoenix Soldier | 0x224 | 0500009e | 010001e5 | 5 |
| 11 | Ninja | 0x225 | 050000a0 | 010001e7 | 4 |
| 12 | Bond Tux | 0x39f | 050000a4 | 010001eb | 4 |
| 13 | Drake Suit | 0x3a0 | 0500008b | 010001b7 | 4 |
| 14 | Bond Spacesuit | 0x10001ee | 050000bb | 0100020b | 4 |
| 15 | Goldfinger | 0x226 | 0500001c | 0100012f | 4 |
| 16 | Renard | 0x227 | 0500001d | 0100012a | 4 |
| 17 | Scaramanga | 0x228 | 0500001f | 0100012d | 4 |
| 18 | Pussy Galore | 0x229 | 05000056 | 0100015a | 6 |
| 19 | Christmas Jones | 0x22a | 05000060 | 0100016a | 6 |
| 20 | Wai Lin | 0x22b | 05000063 | 01000170 | 6 |
| 21 | Xenia Onatopp | 0x22c | 05000086 | 010001af | 6 |
| 22 | May Day | 0x22d | 05000081 | 010001aa | 7 |
| 23 | Elektra King | 0x22e | 05000082 | 010001ab | 6 |
| 24 | Jaws | 0x22f | 05000084 | 010001ad | 4 |
| 25 | Baron Samedi | 0x230 | 0500006b | 0100017e | 5 |
| 26 | Oddjob | 0x231 | 05000085 | 010001ae | 4 |
| 27 | Nick Nack | 0x2ff | 0500007b | 010001a1 | 4 |
| 28 | Max Zorin | 0x10001ef | 050000b9 | 01000208 | 4 |

The per-slot word: humans get it from the character menu (`MPSettings+slot*0x30+0x24`, zeroed at
boot); bots get it from their record — `BOT_init` 0x1249e8:160 writes the record's character byte
(+0x10) into the same word and reads the body's skin from `MP_skins[rec[0x10]]` (:100) [M]. The
port's `MpMatch::create_bots` mirrors the write; the pre-build loader mirrors it too (the scene
build parks the bodies before the match starts).

## 2. Where the meshes live (there is no "MP character pack")

**Every MP arena's map pack carries all 29 skins.** Evidence:

- The MP arenas are hashes 0x07000021-0x07000029, 0x0700004b, 0x0700004c
  (`ResetMap_LevelToLoad` 0x1d3800's range check, which sets `GameState+0x84` = MP and calls
  `MP_setLoadingSkins`) [M]. 0x07000048 is the MP front-end and is explicitly excluded from
  `MP_Init`/`MP_PostLoad_Init` [M].
- Measured on the disc [D scan of FILES.BIN + the containers]: every MP arena present
  (07000023/24/25/26/27/28/29, 0700004b) carries **29/29** `0500xxxx` skins (type-3 sub-files)
  and **29/29** `0100xxxx` skin data files, plus the skeletons. 0x07000021/22/4c have no
  FILES.BIN entry (cut arenas).
- The game loads no other archive for MP characters: the load order is
  `ResetMap_Load`'s common pack `07000500.bin` (scripts + the universal skins 010001d8/0100010f),
  then the arena's animation pack `07F000XX.bin` (the kind-4 sequences), then the arena's map
  pack `070000XX.bin` (the skins + data files + kind-5 scripts) [M `LoaderLoad` 0x1d4a00 /
  `ResetMap_Load` 0x1d6360]. `MP_PostLoad_Init` 0x183718 post-inits the game-mode objects and
  loads nothing [M].
- The selection filter is load-time, per character: `MP_setLoadingSkins` 0x1261e8 clears all 29
  +0x0c flags and sets 1 for every human slot's character and every bot record's character [M];
  `MP_NeedSkin` 0x120c48 / `MP_NeedFile` 0x120ca0 return that flag (the SP path returns 1
  unconditionally) [M] — `AnimLoadFile` 0x1bdf40 gates skins on it and
  `parsemap_parsenextblock` 0x1d12e8 discards a skin file's 0x1c block when the file isn't
  needed. The port parses the whole arena archive (no filter): the data is a superset of what
  the game registers, and `MpSkins::set_loading_skins`/`need_skin`/`need_file` carry the
  mechanism for anything that needs the game's exact view.

## 3. The body's animation set

An MP body's movement is scripts (`0x06xxxxxx` ids) picked from **static 40-byte descriptors** in
ACTION.ELF `.data` @0x2b91b0..0x2b9430 (stride 0x28; `PunchSkin` follows at 0x2b9430) [D]. Layout
(from `AnimSetInit` 0x1c6a48 + `AnimSetUpdate` 0x1c0938's reads) [M]:
`u32 loco[countA], 0, u32 strafe[4], 0` — locomotion = `[idle, walk, run]` (crouch sets carry
walk == run), strafe = `[L_a, L_b, R_a, R_b]` (the pairs duplicate; `AnimSetUpdate` reads
word[countA+1] for dir 1 and word[countA+3] for dir 2).

**Set selection** — `PlayerAnimSetInitNormal` 0x18fcd8 / `PlayerAnimSetInitCrouch` 0x18fea0
(identical shape) [M]: the selector is `weapon_data[weapon]*0x10C + 0x84` (the weapon's anim-set
class byte) when `MPSettings._384_4_` (+0x180, MP on) is nonzero, else 999 (SP's one-set path;
`movz` @0x18fd18). The switch: 999→0x2b91b0, 1→0x2b92a0, 2→0x2b93e0, 3→0x2b9200, 4→0x2b9340,
5→0x2b92f0, 6→0x2b9390, default→0x2b9250; the crouch descriptor is always the normal one +0x28
(picked by `PlayerAnimSetInit` 0x195b90's `obj+0xf6 == 4`) [M]. The class bytes [L — the boot
initialiser's output, the run `WeaponData::from_action_elf` replays]: 0 = fists/grenades/gadgets,
1 = 0x02-0x0C, 2 = 0x12-0x15, 3 = 0x11/0x16-0x2B, 4 = 0x0E-0x10, 5 = 0x2C-0x31, 6 = 0x0D.

The full id table ([D] the blob bytes; see `nf_formats::mpanim::MpAnimSets` for the parse — the
test pins every one of these against the ELF):

| class | normal idle/walk/run | normal strafe L/R | crouch idle | crouch move | crouch strafe L/R |
|---|---|---|---|---|---|
| 999 (SP) | 06000a00 ×3 | 06000a00 | 060009ff | 060009ff | 060009ff |
| 1 | 060009dc / 060009de / 060009db | 060009d7 / 060009d8 | 060009d9 | 060009f5 | 060009f3 / 060009f4 |
| 2 | 060009b0 / 060009b2 / 060009af | 060009b3 / 060009b4 | 060009ad | 060009aa | 060009ab / 060009ac |
| 3 | 060001d4 / 060001db / 060001d9 | 0600097c / 0600097d | 060001d7 | 060008f9 | 06000914 / 06000915 |
| 4 | 060001ce / 060001cf / 060001cd | 060009fc / 060009fd | 060001cb | 060009d2 | 060009d1 / 060009d0 |
| 5 | 060001e6 / 060001e7 / 060001e5 | 060009df / 060009e0 | 060001e3 | 0600091f | 0600091e / 0600091d |
| 6 | 060009fe / 060009b5 / 060009b7 | 060009ba / 060009bb | 060009bc | 060009d3 | 060009d4 / 060009d5 |
| default (0, 7+) | 060001f1 / 060001f2 / 060001f0 | 060009e4 / 060009e5 | 060001ee | 06000931 | 06000930 / 0600092f |

**Movement-state → script** ([M] `AnimSetUpdate` 0x1c0938, called from `Player_SSWalk`
0x192310 / `Player_SSCrouch` 0x1924c0 with the maxima 0.1·FRM forward / 0.075·FRM strafe):
`t = clamp(|fwd|, 0, 0.1·FRM) / (0.1·FRM)`; `t ≤ 0.0002` → idle (arr[0]); else the walk script
plays with the run script crossfaded at weight t (the fraction goes to script B's +0x88; with
countA = 3 the slot is always 1 = walk). There is **no discrete walk/run threshold** and **no
jump/fire/pain slot** — jump freezes the phase as the speed → 0, firing plays on the invisible
weapon obj (below), pain (`Player_HandlePain` 0x1902c8) adds no body script. The walk scripts are
distance-driven: `ANIMOBJ+0xd4 = fwd × 0.5` accumulates into `ANIMOBJ+0xdc` (init 0.37) and the
frame comes from the sequence's distance table (`AnimDistanceTableDistanceToFrame2` 0x1c4270) —
feet lock to the ground speed. Crossfade rate on slot changes: blend-out 8.0/s (script+0xac,
≈0.125 s) with a 4-tick slot debounce [M].

**Strafe overlay** (MP only — `AnimSetUpdate` returns before it when `MPSettings._384_4_` == 0
[M LAB_001c0dc8]): `ratio = strafe/(0.075·FRM)`, `angle = atan2f(ratio, |fwd|/(0.1·FRM))`; when
|strafe| > 0.0002 and |angle·(2/π)| ≥ 0.5 (the ±30° cone), dir 1 (positive strafe) appends
word[countA+1] and dir 2 word[countA+3] as a blend overlay (`AnimScriptAppendBlend`, flags
|= 0x8d004000) with playback speed |ratio| (+0x98) and weight |angle·(2/π)| (+0xa8) [M].

**Stand↔crouch transitions** — `PlayerAnimCrouch2Stand` 0x190068 / `PlayerAnimStand2Crouch`
0x190198 [M]: type = the weapon class when MP, else 3; the script per type — Crouch2Stand:
1→060009da, 2→060009ae, 3→060001dd, 4→060001d1, 5→060001e9, 6→060009b6, default→060001f4;
Stand2Crouch: 1→060009dd, 2→060009b1, 3→060001dc, 4→060001d0, 5→060001e8, 6→060009b9,
default→060001f3. Played at ±4× speed (script+0x98 = ±4.0) with root-motion flags |= 0x0d000000.
`Player_ChangeSubState` 0x190630 sets the new substate first, rebuilds the set, fires the
transition when the previous substate was the other stance, and latches `BLData+0x960` to 60
ticks; `Player_SSWalk`/`SSCrouch` skip `AnimSetUpdate` while it counts down [M].

**Death** — substates 0xd (ground) / 0xe (drown; picked by `ANIMOBJ+0x60 & 0x100`): the set is
deleted and ONE script plays — 0x60008e0 when the water-level global `DAT_002d8894` == 0.0, else
0x6000151 (`Player_Init` initialises it to −9.9) [M 0x190630 cases 0xd/0xe]. MP respawn comes
300 ticks later via `MP_ReSpawn` 0x185368 (re-equip → set rebuild → substate 0) [M 0x190bb0:41-55].

**Spawn** — the MP branch of `Player_Init` plays 0x60001d4 (class 3's idle) after
`Player_StandAtNewPosition` [M 0x1910f8:155-158].

**The third-person weapon** — `Player_SetWeaponAnimObj` 0x1a2e38 is the weapon obj's action
state machine (draw/holster/reload/fire scripts from `weapon_data` on the INVISIBLE weapon obj
created by `Player_InitWeapon` 0x1a7ef8 — `View_SetDrawInNoViews`), not a body attachment. The
visible third-person weapon binds at draw time: `AnimObjectDraw` 0x1c4490:256-266 puts mesh
`weapon_data[w]+0x80` into the body skin's datum slot `weapon_data[w]+0x50` — the same datum
mechanism the port's drone held-weapons use. The muzzle offset for effects is `weapon_data[w]`
+0xE0 (SP) / +0xEC (MP) column + sway [M 0x1a2e38:569-596].

## 4. The port's loader API (nf-view, no drawing)

- `Scene::mp_bodies: Vec<MpBody>` — one per human slot (0..human_count) and per bot (4..4+bot_count),
  built by `Scene::build` on an MP load (`mp_settings` reaches build; the bot character words are
  written before it from the bot records, [M 0x1249e8:160]). Each `MpBody` carries the slot, the
  character word, the `MP_skins` row (skin/file/sleeve), the skeleton id, the parked character
  indices (one per main mesh) and the body's sequence set: every script id the descriptors,
  transitions, death and spawn can produce, resolved through the arena's kind-5 scripts into
  `(first frame, last frame, sequence index)` triples of `Scene::sequences[skeleton]`.
- The bodies are **parked hidden**: the characters sit at the sentinel translation (y = −1e6, the
  NPC-pool scale) and `char_hidden` reports true until the draw moves the Character's matrix to
  the slot's live position.
- The pose: `nf_formats::mpanim::mp_body_pose(&sets, class, &MpBodyState) -> MpBodyPose
  { script, time, blend }` and `mp_body_strafe(&sets, class, &state) -> Option<(script, speed,
  weight)>` — pure, [M] §3. `class = weapon_anim_class(&weapon_data, weapon)` (or 999 for SP).
  The returned id is the game's anim SCRIPT id (0x06…); `MpBody::scripts` maps it to sequences.
  `time` is the walk distance for the distance-driven locomotion/idle scripts, the one-shot clock
  for death/transitions (the draw owns both clocks: phase += fwd·0.5 per tick from 0.37; the
  death/transition script time advances ×speed).

### The drawn bodies (`Scene::mp_bodies`, mp-body-look)

- Held weapon: `Sim::mp_body_held(slot)` — a bot's brain `weapon_id` (BOT_vars+0x768), a human's
  session `held` (last nonzero while the remote is fed NEUTRAL). The attachment per weapon shows
  only for the held one.
- Light: the bodies ramp to their cel's ambient (`Sim::mp_body_chars` in `ramp_char_ambient`).
- Walk: distance-driven — `Sim::mp_walk[slot]` += |speed|/60 × 0.5 per tick (init 0.37), the
  walk frame from its z-only looping `DistanceTable`, the run synced and blended at t. Idle keeps
  the clock read (holds its last frame). Strafe overlay / backwards walk not ported.
- Render aids: `nf-view <iso> 07000023.bin --mp-match 2,2,0,9 --frame 60 --mp-look 1,2.6,1.3,0.6
  --shot out.png`.

## Live checks for the main agent

1. `MP_skins` bytes [D already — the ELF is the evidence]; the disc-vs-snapshot flag column:
   confirm a PCSX2 save-state mid-MP-match shows `MP_skins[i]+0x0c` (0x26d488 + i·0x10 + 0xc) =
   1 for the selected characters, 0 for the rest ([M] says `MP_setLoadingSkins` writes it).
2. Death variant: the branch at 0x190630 case 0xd/0xe reads `DAT_002d8894` (0x2d8894) == 0.0 →
   0x60008e0. Watch it during a water death vs a ground death; the semantic naming (drown) is
   [S] — the branch itself is [M].
3. The strafe overlay weight: watch `AnimSetUpdate`'s overlay script +0xa8 while strafing
   (expect |angle·2/π|, 0 outside the ±30° cone).
4. `BLData+0x954` (0xff on every substate change) has no decompiled reader — a gdbwatch on
   0x…+0x954 during play would say who consumes it.
