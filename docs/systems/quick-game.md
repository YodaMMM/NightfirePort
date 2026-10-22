# Quick Game: the scenario wheel's row 0 and the match it starts

quick-game-scenario (2026-10-21, branch `s/quick-game-scenario`) — GAPS G-053 item 6. The
`mp_scenario` wheel's row 0 carries scenario word 0 (a sentinel, not a gametype [D
MP_SCENARIO_ROWS — see `docs/systems/mp-menus.md` §2]) and its confirm runs the game's own
"no choices" preset. Everything below is first-hand: [M] = read in this task
(`work/decomp/ACTION.ELF/0020c600_C_SBMPSCEN_Handler__FUcP9M_CONTROLUiUiii.c` + a capstone
disassembly of 0x20c600..0x20c870, `work/elf/ACTION.ELF`, LOAD vaddr−0x106100 = file offset);
line numbers refer to that decompile. This page only adds the Quick Game detail;
`docs/systems/mp-menus.md` §1–§4 holds the surrounding page graph and the confirm's handover.

## 1. The trigger

`C_SBMPSCEN_Handler` 0x20c600 msg 0x4b (the wheel's Confirm): the row index comes from the
control's msg-0x40 (:32-33); the lock check reads `mp_scenario[row·0x18+0x10]` and refuses when
the byte is 0 (:35-38, `menu_unlock_everything` bypasses) — row 0 ships unlocked. Row 0 takes the
quick-start arm (:48+); any other row stores `MPSettings+0x1A4 = mp_scenario[row·0x18+0xC]`
(:41-43, the disasm's `sw` at 0x20c9c8) and opens the map page — that arm is the CUSTOM path and
never touches the quick fields.

## 2. The quick-start writes (the fixed config)

`Menu_PrepareBots` 0x1f94a0 first (:48), then these stores, in disasm order — `$s0` is MPSettings
0x2a47a0, `$v0`/`$v1`/`$a0` hold 1/2/10 from the preceding `addiu`s (0x20c6f0..0x20c728):

| disasm | MPSettings offset | value | meaning |
|---|---|---|---|
| 0x20c72c `sw $a0, 0x1a0` | +0x1A0 | 10 | time limit, minutes |
| 0x20c734 `sw $v1, 0x1cc` | +0x1CC | 2 | vehicles = Helicopters (Car_Create 0x1305b0:29-40) |
| 0x20c738 `sw $a0, 0x19c` | +0x19C | 10 | frag/score limit |
| 0x20c73c `sw $v0, 0x180` | +0x180 | 1 | the MP master enable |
| 0x20c740 `sw $v0, 0x1a4` | +0x1A4 | 1 | scenario = Deathmatch |
| 0x20c744 `sw $zero, 0x190` | +0x190 | 0 | |
| 0x20c748 `sw $zero, 0x1d4` | +0x1D4 | 0 | explosive scenery off |
| 0x20c74c `sw $zero, 0x18c` | +0x18C | 0 | FFA |
| 0x20c750 `sw $v0, 0x1c4` | +0x1C4 | 1 | radar on |
| 0x20c754 `sw $zero, 0x198` | +0x198 | 0 | friendly fire off |
| 0x20c758 `sw $zero, 0x1d0` | +0x1D0 | 0 | grapple off |
| 0x20c760 `sw $zero, 0x1b4` | +0x1B4 | 0 | weapon set Normal (delay slot of the jal) |

**Not written** — whatever the struct already holds persists:
- +0x1B8 (gun emplacements), +0x1BC (professional mode), +0x1C0 (view distance), +0x1C8 (location
  damage) are absent from the store list [M 0x20c72c-0x20c768]. Boot sets +0x1BC = 0, +0x1C8 = 1
  (Location Damage ON), +0x1C0 = 2 (Random), +0x1B8 = 0 [M bootup_bootup 0x1c7e78:38,45,46,28 —
  stores 0x1c7ff0..0x1c7ffc] — so a fresh boot's Quick Game plays with Location Damage ON, and a
  custom visit's changes to those four survive a later Quick Game.

## 3. The arena: one shared-RNG draw, rows 0..6

0x20c75c `jal 0x1e36f0` (Rand_Random), then `divu …, 7` / `mfhi` (0x20c76c-0x20c7a4): the draw
mod 7 indexes `mp_level` (0x2df400, `lui/addiu` at 0x20c770-0x20c778), stride 0x18
(`mult $a1, $a2` 0x20c7a4), and the row's +0x0C level code lands in BOTH GameState+0x8
(`sw $v0, 0x3770($v1)` 0x20c7b0) and MPSettings+0x1A8 (`sw $v0, 0x1a8($s0)` 0x20c7b4) [M].
Rows 0..6 = Skyrail, Fort Knox, Snow Blind, Phoenix Base, Atlantis, Missile Silo, Sub Pen —
**never Ravine** (row 7) and never the bots-forbidden-map question [M the modulo 7].
The exact draw value depends on the whole boot path's shared-stream position — not reproducible
headlessly — so the port samples row 0 (Skyrail) and records the deviation [S — the game's sample
is random; the port's rule "never add draws" bars inventing a stream position].

## 4. The joined players and the bots

- The joined loop (mpjoin, 0x2dfee68, stride 0x10; 0x20c7b8-0x20c84c) counts the joined slots
  into MPSettings+0x1AC (0x20c858) and fixes teams/skins per slot [M]: slot 0 → team 1 (MI6),
  skin 0 (Bond) (`sw $v1, 4($a3)` / `sw $zero, 8($a3)` 0x20c810/18); slots 1-3 → team 0 (Phoenix),
  skins 6/7/8 (`sw $t2, 0x18` = 6, `sw $t1, 0x28` = 7, `sw $t0, 0x38` = 8 with `$zero` to the
  +4 teams, 0x20c7f8-0x20c834). The placeholder skins for UNJOINED slots 1-3 are what the bots
  never use — the match's human slots are the joined ones only.
- Bot count: the byte 0x2deea9 = 3 (`sb $a1, 1($v0)` 0x20c864, `$a1` = 3) and MPSettings+0x1B0 = 3
  (`sw $a2, 0x1b0` 0x20c86c) [M].
- The three bot records (0x2deeaa, stride 0x12; the loop 0x20c870..0x20c950): +0x10 character —
  1 (Drake), 3 (Kiko), 2 (Rook) (DAT_002deeba/cc/de stores); +0x0E enabled = 1, +0x0F team = 0
  (Phoenix), +0x11 dirty = 0 (the `(&DAT_002deeb8/9/bb)[i·0x12]` stores); +0x00..0x0D = the 14
  preset stat bytes amemcpy'd from `BOT_getDefaultStats(char)` 0x2027a8 (0x20c890-0x20c8b8); the
  name goes to MPSettings+0xC0+slot·0x30 from the wheel's label (0x20c8c4-0x20c8dc) [M].
- Then `Menu_ChangePageCloseIris(0x40000049)` (0x20c950-0x20c95c) — straight to the match
  summary; the confirm page's Confirm is THE start ([M] §4 of mp-menus.md: ×0x3c on +0x1A0, the
  compaction, GameState+0x08 = +0x1A8, the push).

## 5. The port

- `FrontEnd::mp_quick_start` (crates/nf-view/src/menu.rs) writes exactly the fields of §2 —
  including NOT touching `mp_player_mods` (+0x1BC/+0x1C8) or `mp_envmods[0]/[1]` (+0x1C0/+0x1B8),
  whose boot values [0,1]/[2,0] (FrontEnd::new, [M bootup 0x1c7e78]) and any custom visit's
  values persist. (Fixed this task: the previous model re-stamped those fields, which would have
  played a fresh Quick Game with Location Damage OFF and stomped a custom visit's View Distance —
  the store list is the proof neither happens in the game.)
- `mp_confirm_start` carries every field into `MpSettings` (the mapping is in mp-menus.md §4);
  the quick config reaches the match through it, so the sim's MpMatch gets scenario 1, the
  sampled arena, 10/10 limits, 3 bots, radar on, friendly fire off, weapon set 0, vehicles 2.
- The arena sample: row 0 [S §3]. `mp_menu_lists_live.rs` pins the table bytes from the disc;
  the headless lists test pins the rows-0..6-never-Ravine invariant over `MP_MAP_ROWS`.

## 6. Tests (crates/nf-view/tests/mp_menu_flow.rs)

- `quick_game_starts_the_games_fixed_match` — the real path (Main → Multiplayer → Local → Join →
  ready → row 0 → the summary → Confirm), every MpSettings field asserted with its citation,
  plus the loader reaching the arena. Sister A inside: Back from the summary re-opens the
  scenario page on row 1 (the quick start's +0x1A4 = 1 drives the cursor re-derivation).
- `custom_game_keeps_its_chosen_scenario` — sister B: from that same page a custom pick (row 2,
  Team Arena) flows through the setup pages and the match carries scenario 0x20000002
  (team_game 1) — the quick start's Deathmatch stomp is confined to the quick path.
- `quick_game_keeps_a_custom_visits_mods` — sister C: the Player Mods / Enviro-Mods pages set
  +0x1BC = 1 and +0x1C0 = 0 through the real rows, then a Quick Game's match still carries them
  (the store list writes neither).
