# The HT-level music updaters (`UpdateMFX_HT_Level_*`)

Seven never-mentioned ACTION.ELF functions, all reached through one dispatcher [M]:

| Function | Address | Size | Music id (the `switch(*(u32*)MapMusicData)` case) |
|---|---|---|---|
| `UpdateMFX_HT_Level_Tower2A` | 0x114250 | 244 | 0x7000011 |
| `UpdateMFX_HT_Level_Tower2B` | 0x114348 | 476 | 0x7000012 |
| `UpdateMFX_HT_Level_Tower2C` | 0x114640 | 232 | 0x7000013 |
| `UpdateMFX_HT_Level_EvilBase` | 0x114728 | 780 | 0x7000014 |
| `UpdateMFX_HT_Level_EvilSilo` | 0x114a38 | 256 | 0x7000015 |
| `UpdateMFX_HT_Level_EvilBaseC` | 0x114b38 | 544 | 0x7000016 |
| `UpdateMFX_HT_Level_Tower2Elevator` | 0x114528 | 276 | 0x700004a |

## Reachability [M]

`UpdateMusicalEvents` 0x114e70 (called from `SFXUpdate` 0x1156c0 every frame) ends with a switch on
`*(undefined4 *)MapMusicData` — the music id of the track now playing — under a `MusicWaitForEnd == 0`
gate [M disasm 0x1153c4-0x1153c8, cited at `crates/nf-game/src/music.rs`]. The cases `0x7000011`,
`0x7000012`, `0x7000013`, `0x7000014`, `0x7000015`, `0x7000016` and `0x700004a` call the seven
functions above [M 0x114e70:261-283]. The ids are the level ids (`wire_music` raises
`Music_Event(0, level)`; the game does the same at track start [M 0x114e70:43-54]); the disc's
`MapMusic` table carries rows for them (the port parses them with `nf_formats::music::tables` [M],
and levels 07000011/12 are played by the port's own live tests, e.g. `fix_actors_live.rs`,
`pistol_sound_12_live.rs`). So: reached on those on-foot levels whenever their music plays and no
end-wait stands.

## Shared shape

Every one of the seven starts with the same **exit block** [M; the identical block in the other 14
updaters]: if `DAT_002d3518` (= event slot e[8], mission success — `0x182c60:31`) or `DAT_002d3504`
(= e[3], the `Trigger_Activate` LOAD_LEVEL exit — `0x1b2318:109`):

1. scan the 5 u32s at `MapMusicData+0x24` (the combat set) for `MusicLastJump` (stop at the
   `_heap_size` sentinel) — found → jump `MapMusicData+0x14` (`end_combat`), else `+0x18`
   (`end_calm`); the jump is INSTANT (`SFXJumpToMusicMarker(s, 1)`);
2. `SFXFadeDown(0)` unless `MapMusicData[0] == 0x7000006`;
3. `MusicWaitForEnd = 1`.

The port already runs this shared block for exactly these ids in `Music::exit_block`
(`crates/nf-game/src/music.rs`), inside the dispatcher's `!wait_for_end` gate — that block IS the
whole body of `UpdateMFX_HT_Level_Tower2A` [M 0x114250:9-34: nothing else].

The event-slot map used below (base `DAT_002d34fc` = e[1], `EventList` = e[0] — the map the port's
`Music::update` already pins): `DAT_002d3504` = e[3], `DAT_002d3518` = e[8], `DAT_002d3524` = e[0xb]
(NIS-playing slot), `DAT_002d3528` = e[0xc] (the stream-kill slot).

## The per-level halves (`MusicWaitForEnd == 0`)

**Tower2B 0x114348:36-76** — a 3-state alarm machine on `MusicTemp1` [M]:
- state 0: `MusicUnderAttack` → `SFXJumpToMusicMarker(0xf, instant=1)`, state 1;
- state 1: `!MusicUnderAttack` → `jump(0x11, instant=0)`, state 2;
- state 2: `!MusicUnderAttack` → if `MusicLastJump` is in the combat set, `jump(0x16, instant=0)`;
  `MusicUnderAttack` → `PickRandomCombat(0)`.

**Tower2C 0x114640:9-12** — BEFORE its exit block [M 0x114640:9-12 vs :14-39]:
`MusicUnderAttack` → `MusicLastJumpRequest = 0; SFXJumpToMusicMarker(0, 0)`. (The order matters: on a
frame where both the alarm and the exit events land, the game queues jump(0) first and the exit
marker jump second — the IOP's last request wins.)

**EvilBase 0x114728:36-134** — the busiest machine [M]; latches `MusicTemp3/6/2/4` and the counter
`MusicTemp1` (reset by the track-start restart, 0x114e70:52-54):
- `e[0xc] == 0x6000848` or `MusicTemp3 != 0`: `MusicTemp3 = 1`, request `&o_right`, then the shared
  tail `SFXJumpToMusicMarker(0x18, 1)` — re-issued EVERY frame the state holds;
- else: `e[0xb] == 0x6000848` or `MusicTemp6 != 0` → `MusicTemp6 = 1`, `jump(0x22, 1)`;
  `e[0xb] == 0x600062f` or `MusicTemp2 != 0` → latch (`MusicTemp2 != 0` returns), `MusicTemp2 = 1`,
  `MusicTemp1 = 0`, calm `jump(5, 1)` / attack `jump(0x12, 0)`;
- `e[0xc] == 0x6000833` or `MusicTemp1 != 0` → `MusicTemp1 += 1`; calm: `MusicLastJump` in the combat
  set → `jump(0xf, 0)` (return); request already `0xf` → return; else request `&o_left` and tail
  `jump(0x14, 1)`. Attack: `MusicTemp1 > 0x37` → `PickRandomCombat(1)` (return); else request
  `&o_left`, tail `jump(0x14, 1)`;
- else if `e[0xb] == 0x6000833` or `MusicTemp4 != 0` → latch; `MusicLastJump == 0x2e` →
  `jump(0x34, 0)` (return); else `jump(0x16, 1)`;
- else: calm — `MusicLastJump == 5` → return; `jump(0x32, 0)` (return). Attack: `jump(0x2e, 1)`.

`MusicLastJumpRequest` is EE-side bookkeeping only — `SFXJumpToMusicMarker(s, instant)` queues batch
op 0x13 with its ARGUMENT [M 0x116950:3-15]; the request is read by `PickRandomStealth` /
`PickRandomCombat` dedupe and the machines' own `== 0xf` check [M 0x111c70:20, 0x111dc8:20]. The
`&o_right`/`&o_left`/`&o_fx_int` stashes are pointer-as-id marks that no reader ever compares
against a set entry; the port's `jump()` records the section jumped, which is exact for every
reader.

**EvilSilo 0x114a38:41-47** — after the exit block [M]:
`MusicUnderAttack ? PickRandomCombat(0) : PickRandomStealth()` every frame.

**EvilBaseC 0x114b38:36-77** — a 4-state cut-scene machine on `MusicTemp1` [M]:
- every frame: `e[0xb] == 0x6000631` → `jump(0x24, 1)`, state 1;
- state 0: `PickRandomCombat(0)` / `PickRandomStealth()` by the alarm;
- state 1: `e[0xc] == 0x6000631` → `jump(0x27, 1)`, state 2;
- state 2: `e[0xb] == 0x6000632` → `jump(0x29, 0)`, state 3;
- state 3: `e[3] || e[8]` → `jump(0x2b, 0)`, `MusicWaitForEnd = 1`.

**Tower2Elevator 0x114528:36-40** — after the exit block [M]:
`e[0xb] == 0x60006d3` (the elevator ride's NIS stream) → request `&o_fx_int`, `SFXJumpToMusicMarker(4, 0)`.

## Where the port takes it

`crates/nf-game/src/music.rs`: the shared exit block was already dispatched for these ids; the port
adds the per-level halves as `tower2_b` / `tower2_c` (its pre-exit attack jump) / `evil_base` /
`evil_silo` / `evil_base_c` / `tower2_elevator`, each cited with its address, dispatched from
`Music::update` under the same `!wait_for_end` gate, with the machine state as new fields
(`tower2b_temp1`, `evil_base_*`, `evil_base_c_t1`) zeroed by the track-start restart like the game's
`MusicTemp1..9` [M 0x114e70:52-54].
