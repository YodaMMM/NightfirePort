# audit-sprite-set-text — on-screen strings outside the message queue

The sweep walked every `Sprite_SetText` 0x1de698 / `Sprite_SetTextNFmt` 0x1de718 caller in
ACTION.ELF (18 files), kept the on-foot + multiplayer set (status sprites, timers, counters, pane
labels), and checked each against the port's draw sites. Driving skipped per the task. The
message queue itself (`Text_Update` 0x1b1510 / `Text_Update2Line` 0x1b09f0 — the subtitles and
`Text_AddMsg` playback), the loading screen (`ResetMap_GenLoadScreen` 0x1d5de0) and the front-end
menu labels (`Label_Init` 0x218638 / `Label_Update` 0x218cb8) are out of the task's scope and are
only listed in the caller census below. Origin/master merged by branching from it (df172883b).

## The caller census

| caller | scope | verdict |
|---|---|---|
| `MP_Start` 0x183a50:202-205 | MP | in scope — the boot clears (below) |
| `MP_CheckForEndCondition` 0x184268:61-64,144-147 | MP | in scope — the clock (2 fixes) |
| `MP_SortOutWhoWon` 0x1848d0:53,85 | MP | in scope — ported ✓ |
| `MP_Update` 0x184bd0 case 2 / case 6 | MP | in scope — 1 fix (case 2) |
| `MP_DemolitionProtectionUpdate` 0x1870a0:104,123,158 | MP | in scope — ported ✓ |
| `HUD_UpdateAmmoPane` 0x19a7f0:165-245 | SP+MP | in scope — ported ✓ |
| `HUD_UpdateMPHealthPane` 0x19b630:102 / `HUD_UpdateHealthPane` 0x19b9c8:270 | SP+MP | debug-only (`switch_SHOW_COORDS`, the "%.2f %.2f %.2f\nFPS %d\n" overlay) — retail-invisible, listed not ported |
| `HUD_UpdateStatusPane` 0x19c310:170,173 | SP | in scope — ported ✓ (the objective header/body + the mission fail label) |
| `HUD_MPUpdateStatusPane` 0x19ca88:24 | MP | in scope — ported ✓ (the kill message) |
| `HUD_CreateOICWPane` 0x19cc90:47-59 | SP | in scope — ported ✓ (the five boot lines, `OicwBoot`) |
| `HUD_MPUpdatePane` 0x19f0a0:71 | MP | in scope — ported ✓ (the score pane) |
| `HUD_RadarUpdate` 0x19f6e8:241 | MP | in scope — ported ✓ (the name tags, `MPSettings+slot·0x30`) |
| `Text_Update` 0x1b1510 / `Text_Update2Line` 0x1b09f0 | SP | out of scope (the message queue / subtitles; the queue the task excludes) |
| `ResetMap_GenLoadScreen` 0x1d5de0:129-156 | boot | out of scope (the loading screen, not play) |
| `Label_Init` 0x218638:69-73 / `Label_Update` 0x218cb8:218 | front end | out of scope (the menus) |

## The in-scope table: string, when, template, port site

`StatusSpr` = `MPStatusInfo` @0x2a4740 [L ee_start_05.bin]: colour 0xffffffff, shadow
0x000000ff, style 0x30cbd8 (font 2, centred), texture 0x20000001; the runtime rect (256,48) [L
mp-hud-data.md §8]. `TimeSpr` = `MPTimeInfo` @0x2a4710: colour 0xffffffff, shadow 0x000000ff,
style 0x30cbd0 (font 1, centred), texture 0x20000001; the runtime rect (256,32) [L, MP_Start's
y=0x20 for one human]. The fmt constants [L ee_start_05.bin]: "" @0x30cbe8, "%s" @0x30cbf0,
"%s%s" @0x30cbf8, "%s%s %s" @0x30cc00, "%d - %d" @0x30cc08. The strings [L] are bound off the
disc's USATxt.dat by `mp_status_strings_live` (the port's own FILES.BIN reader).

| state / arm [M] | text written (or cleared) | port site | status |
|---|---|---|---|
| `MP_Start` 0x183a50:202-205 | `TimeSpr` + `StatusSpr` ← "" (the boot clear) | empty default draw | ✓ |
| `MP_CheckForEndCondition` :61-64 (the 0x20000040/0x20000080 arm) | `TimeSpr` ← `Timer_Seconds2String((round_limit − round_clock)·100, 2)` every playing tick while `MPGame+0x194 > 0` and ch 0xfe clear | `hud::mp_panes` clock | **FIXED** (the gate, below) |
| `MP_CheckForEndCondition` :144-147 (every other scenario) | `TimeSpr` ← the match clock, only while `MPSettings+0x1a0 != -1` and ch 0xfe clear (0xfe latches time-up) | `hud::mp_panes` clock (`settings.time_limit != -1`) | ✓ |
| `MP_Update` case 0 → `MP_CheckForEndCondition` | (the writes above; state 0 only) | ✓ same window | ✓ |
| `MP_Update` case 2 [M :42-50] | `StatusSpr` ← `sprintf("%s", bind(0x2000028)="Time Up!", bind(0x1c9)="Press START")`, the second argument unread | `status_line`'s `MpEndStatus::None` arm | **FIXED** (the port concatenated "Press START") |
| `MP_Update` case 2 with scenario 0x10 | falls through to case 1 (`MP_SortOutWhoWon`) | the same fall-through in the port's state machine | ✓ |
| `MP_Update` case 1 → `MP_SortOutWhoWon` 0x1848d0 | 0x10: "Player %s Won" 0x2000022 / "A Draw" 0x2000023 (the tie quirk [M :23-52]); FFA: exactly one qualifier → "Game Over : %s Won" = "%s%s %s" of (0x2000024, codename, 0x2000027) [M :55-85]; team games: "Game Over :A Draw" = "%s%s" of (0x2000024, 0x2000023), the third bind (0x1c9) again a dead arg [M :87-92] | `MpMatch::sort_out_who_won` rules.rs:584-648 + `status_line` | ✓ all four arms |
| `MP_Update` case 6 at ≥2.5 s [M :82-84] | `StatusSpr` ← bind(0x2000049) = "Restarting" | `status_line`'s state-6 arm | ✓ |
| `MP_Update` case 6 at ≥5 s [M :88-90] | `StatusSpr` ← "" (the clear, fmt 0x30cbe8) + `MP_RestartScenario`, state 0 | state 0 → `site_status` None → empty | ✓ |
| `MP_DemolitionProtectionUpdate` :102-104 | a defender's hit → `Text_AddMsg` bind(0x200002b) "You are supposed to protect the target!", once per 1.5 s | `rules.rs:1094-1098` (`slot_msgs`) | ✓ |
| `MP_DemolitionProtectionUpdate` :117-123 | round timeout → `StatusSpr` ← 0x200002c "MI6 failed to destroy target" / 0x200002d "Phoenix failed to destroy target" (site index 0/1) | `rules.rs:1037` (`site_status`) | ✓ |
| `MP_DemolitionProtectionUpdate` :147-158 | site destroyed → 0x200002f "Phoenix destroyed target" (owner team 0) / 0x200002e "MI6 destroyed target", state 6 | `rules.rs:1118-1119` | ✓ |
| `HUD_UpdateAmmoPane` :165-245 | the count "%d-%d"/percent/heat + the fire-mode label (`weapon+0x30+mode·4`) + the weapon name (`weapon+0x38`), the `BL+0x95E` switch window | `hud.rs` ammo pane (count, `fire_mode_label`/`name_label`) | ✓ |
| `HUD_UpdateStatusPane` case 2 [M :113-176] | the objective header rebind 0x2000051 "NEW OBJECTIVE" / 0x2000053 "OBJECTIVE COMPLETE" (the strcmp arm), the body ← the record's label, the mission fail row ← `Mission_FailLabel()`; the record+0x18 < 0 arm zeroes both colours | `hud.rs` objective/mission banners (`objective_rows`, the fail label rebind) | ✓ |
| `HUD_UpdateStatusPane` case 3 [M :191+] | the banner flash (the gold blink, the body alpha) | `objective_rows` (fix-hud-parity) | ✓ |
| `HUD_CreateOICWPane` :47-59 | the five boot lines ("COMMAND.COM" … "OK") | `nf_game::hud_pane::OicwBoot` | ✓ |
| `HUD_MPUpdateStatusPane` :24 | the kill message text ← the message record's label | `mp_panes` MP info (the `kill_msgs` queue) | ✓ |
| `HUD_MPUpdatePane` :71 | the score pane text (the team line over the viewer's own, the 3-line 0x400/0x40000800 arms) | `MpScorePane::build` | ✓ |
| `HUD_RadarUpdate` :240-241 | the name tags ← `MPSettings+slot·0x30` (the style 0x30cc90 = font 2), gated on `MPSettings+0x1c4` / scenario 0x400 | `mp_panes` `names_on_radar` branch | ✓ |
| `HUD_UpdateHealthPane` :262-270 / `HUD_UpdateMPHealthPane` :94-102 | the debug coords/FPS overlay behind `switch_SHOW_COORDS` — a debug switch, off in retail | not ported (retail-invisible) | listed |

## The fixes (each pinned by a test through the real path)

1. **The time-up status read "Time Up!Press START"** — `MP_Update` case 2's format is the plain
   "%s" at 0x30cbf0, with `Txt_BindLabel(0x2000028)` ("Time Up!") its argument and
   `Txt_BindLabel(0x1c9)` ("Press START") a spare one it never reads [M disasm 0x184da8-0x184de0:
   a1 = s0, a2 = s1 in the jal's delay slot, a3 = the second bind; L ee_start_05.bin @0x30cbf0].
   The port's `MpEndStatus::None` arm concatenated both labels.
   `status_line` now returns the game's single label [M]. Test: `mp_hud_pane` 5/5 (the None arm
   re-pinned), `mp_status_strings_live` 1/1 (the fifteen labels bound off the disc, including the
   case-2 pair).
2. **The round clock was hidden on unlimited demolition/protection matches** —
   `MP_CheckForEndCondition` has two `TimeSpr` write gates: the round arm (scenarios
   0x20000040/0x20000080) writes while `MPGame+0x194 > 0` [M :57, no −1 check] and every other
   scenario only while `MPSettings+0x1a0 != -1` [M :142]. The port keyed the whole draw gate on
   the settings limit, so a 60-second demolition round [M 0x183438:53] with an unlimited match
   limit showed no clock. The draw gate now follows the game's split (`game.limit > 0` for the
   two round scenarios). Test: `mp_clock_gate_live` 1/1 — unlimited demolition draws the clock,
   unlimited deathmatch does not (TimeSpr keeps `MP_Start`'s "" clear [M 0x183a50:202-203]), a
   limited deathmatch does.

Doc corrections carrying the evidence: `docs/systems/mp-debriefing.md` case-2 line (it said
"Game Over :" 0x2000024 — that label belongs to the case-1 arms; the disasm binds 0x2000028) and
`docs/systems/mp-hud-data.md` §8 (the status-line list and the clock gate).

## What ran (per-binary pass counts, --test-threads=1, release, NF_ISO set)

| binary | result |
|---|---|
| mp_hud_pane (nf-game, the status line) | 5 passed |
| mp_status_strings_live (new) | 1 passed |
| mp_clock_gate_live (new) | 1 passed |
| mp_hud_real_live | 2 passed |
| mp_hud_data | 4 passed |
| mp_hud_16 | 7 passed |
| walkreal_07000001_live | 8 passed |
| walkreal_07000005_live | 14 passed |
| walkreal_07000014_live | 9 passed, 1 ignored |
| walkreal_07000011_live | 3 passed, 1 ignored |

The full suite did not fit this sandbox's window (the release build of the ~950 test binaries
filled the 10 GB disk twice; the build completed for the set above). The changes touch only
`nf-game/src/mp/hud.rs` (MP-only) and the `mp_panes` branch of `nf-view/src/hud.rs` (called only
with `Sim::mp` set), so the SP walks above are the regression surface; the remaining walkreal
levels are the next session's first item if a fuller run is wanted.

## Live checks for the main agent

1. The time-up line: in a PCSX2 deathmatch with a 1-minute limit, at 00:00 the status line
   should read "Time Up!" with NO "Press START" (`StatusSpr` text at 0x2a4740's runtime
   buffer; the writes are at 0x184da8-0x184df0). If the real machine shows a trailing
   "Press START", my fmt reading is wrong and the port's concat was right.
2. The round clock: an unlimited-limit demolition match (`MPSettings+0x1a0` @0x2a47a0+0x1a0 =
   −1) should still show the 60-second round clock (`MPGame+0x194` @0x2a4b14 = 60) — the write
   at 0x184734-0x184748.
3. The name-tag style: `watch 0x30cc90 4` during a match with tags up — the sprite's +0x1c
   should hold the font-2 style pointer for the tagged name sprite (read at 0x19f6e8:238-241).

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).
