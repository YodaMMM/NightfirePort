# mp-hud-data — the multiplayer HUD's data model

Status: implemented (the snapshot struct + getters + the headless tests). Evidence tags: [M]
code (function + address), [D] disc, [L] EE snapshot, [S] suggested / port-only design.

## 1. What this is

In an MP match the HUD shows per-agent info (score / frags, the match timer, the scenario's
objective status, the radar's blips). The game reads all of it straight out of the MP globals
every frame: `HUD_Update` 0x19a088 walks the pane list and calls each pane's update fn pointer
[M 0x19a088:60-63; the MP set is `MPPaneList`, picked by `HUD_Init` when `MPSettings+0x384 != 0`
[M 0x199e88:36]], and the panes read `MPGame`/`MPSettings`/the scenario globals directly. This
task ports that read side as a **per-frame snapshot struct with getters** — `nf_game::mp::hud`
(crates/nf-game/src/mp/hud.rs) — that the main agent's draw code consumes. Nothing here renders.

## 2. The reads, and what the snapshot carries

### The score pane (`HUD_MPUpdatePane` 0x19f0a0)

The pane is **one per viewer** (keyed `BLData+0x94e` [M 0x19f0a0:31]) — there is NO in-match row
sort. The text is the team line over the player's own line [M 0x19f0a0:56-70; the Ghidra vararg
loss at :68 — the second `%s` is the player string], tinted `0xff0000ff` for team 0 [M :53-55].

- `MP_GetPlayerScore` 0x189730: scenario 0x400 (Assassin) and 0x40000800 (KOTH FFA) print THREE
  lines — score `MPGame+slot·0x30+0x18`, kills `+0x04`, deaths `+0x08` [M :13-19]; every other
  scenario prints two — kills, deaths [M :20-27]. → `MpScoreRow::score_lines(sc)`.
- `Menu_GetMPScore` 0x203af8 (the debrief's sort key, and the pane's score read for the
  kill-less scenarios): the slot's score for the scoring scenarios, KILLS for 0x1 (Arena) and
  0x20000002 (Team Arena) [M 0x203af8:44-54]. → `MpScoreRow::display_score(sc)`.
- `MP_GetTeamScore` 0x18b338: `"%d - %d"` from `MPGame + team·4 + 0x180` [M :20-34]. →
  `snapshot.team_scores`.
- The **sort**: `P_MPDEBRIEFING_Handler` 0x212198's bubble pass, descending by the display score
  [M 0x212198:112-130]. → `snapshot.sorted_rows()` (descending by `display_score`; equal scores
  keep slot order — the bubble's no-swap case). The in-match rows stay in slot order (humans
  0..human_count, bots 4..4+bot_count — the game's walk [M 0x212198:92-111, 0x204408:327-399]).

### The match timer (`MP_CheckForEndCondition` 0x184268 + `Timer_Seconds2String` 0x1deaa8)

- **The clock is `MPGame+0x190`** (the decomp's `MPGame._400_4_` is a DECIMAL print — hex
  +0x190; `MPGame` is 0x1d0 bytes so +0x400 cannot exist). `limit` is `MPGame+0x194`, copied
  from `MPSettings+0x1a0` [M 0x183438:39] — **seconds**; the menu's minutes are ×0x3c at the
  confirm [M P_MPCONFIRM 0x20ec00:146]. The port's menu path already stores seconds in
  `MpSettings.time_limit` (mp_confirm_start's ×60); the `deathmatch()` default (10) is the
  pre-menu number — the report's deviations note.
- The HUD string: `Timer_Seconds2String((int)((limit − clock)·100), 2)` → mode 2 rounds to whole
  seconds, `ss = secs % 60`, `mm = (secs / 60) % 60`, `"%02d:%02d"`; negatives clamp to 00:00;
  hours never show in mode 2 [M 0x1deaa8:17-33]. → `snapshot.time_left` (+ `timer_string()`).

### The radar (`MP_GetRadarObjects` 0x189840 → `HUD_RadarUpdate` 0x19f6e8)

`MP_GetRadarObjects` fills the global `RObj` @0x318e70 (0x20-stride entries: 4×f32 position,
packed RGBA at +0x10, u16 kind at +0x14) [M]; `HUD_RadarUpdate` is its only caller and the
port's `nf_view::hud::radar_project` already implements its transform (0x19f6e8:108-153 — the
inverse viewer matrix, the unit direction, the `len·0.02` rim clamp, the dead height cue) [M,
checked against the disassembly]. The projection takes `pos/argb/kind` records — **this task
builds those records**:

- **Gate**: the viewer's per-player radar toggle `MPSettings + viewer·0x30 + 0x28` — 0 returns
  an EMPTY list and hides the ring [M 0x189840:22-24, 0x19f6e8:102-105]. The port's compact
  `MpPlayer` skipped +0x28 (its `armor_bonus` is +0x2c); the new `MpPlayer::radar_on` carries it
  (own line, WORKLOG lesson 4).
- **Player blips** (kind 0): every live slot but the viewer [M 0x189840:37-40]; team games tint
  team 0 `0xd22d35ff` / team 1 `0x2d61d2ff`, FFA tints everyone `0x7f7f7fff` [M :54-62]. The
  game's dead/limbo gates ride the `obj+0xff` state byte + BLData reads [M :41-92;
  `MP_playerIsDead` 0x18a938 pins the states: 0x01/0x11/0x12 dead, 0x02 dead unless the
  BLData+0x4f8-bit-0x100 + health exception] — the port's `MpSlot::dead` stands in for them [S,
  the BLData pipeline isn't modelled per-player yet].
- **Scenario markers** [M 0x189840:97-232]: CTF 0x20000004 → the two `Flags` entries, kind 1,
  red then blue; Uplink 0x60000008 → the `Uplinks` entries, kind 2, the terminal's live RGB
  packed (team 0 `(0xd2,0x2d,0x35)`, team 1 `(0x2d,0x61,0xd2)` — what `MP_UplinkUpdate`
  0x186d98:41-47 mirrors via `SP_SetColour`); Demolition 0x20000040 / Protection 0x20000080 →
  the site, kind 3, white; GoldenEye 0x20000200 → the items not yet picked up (`ext+0xf0 & 0x10`
  skips a held one), kind 4, white; Blueprint 0x20000100 → the blueprint kind 6 white plus the
  two espionage bases kind 7, red then blue. Kind 5 exists in the UV sheet but is written by
  nothing [M — no kind-5 arm in 0x189840].
- → `snapshot.radar: Vec<MpRadarObj>` (world space; `nf_view::hud::radar_project` does the
  viewer-relative projection — the blips are relative to the viewer through it, `x = −view.x`,
  `z = −view.z`, `len·0.02 ≤ 1` [M 0x19f6e8:108-153]).

### The objective status

- **Flags held**: the pane's flag icon asks `MP_HasTeamFlag(obj, team)` 0x18b430 — the flag
  ext's `+0x1c` CARRIER pointer, not the slot's 0x26 bit0 (only bot AI reads that bit [M
  0x126b88:133-153]). → `snapshot.flags: [Option<carrier slot>; 2]` from the port's
  `MpObjState::carrier` (the pointer's stand-in).
- **Uplink progress**: there is no progress counter — ownership is the ext's `+0xf4` word
  (0/1/2, `MP_getUplinkStatus` 0x18ba80 returns it verbatim) and the "progress" is the score
  drip (+0.2/s to the controller's personal and the owning team's total [M 0x186d98:59-80]).
  → `snapshot.uplinks: Vec<MpUplinkHud>` (owner, controller, world pos, live RGB) — the pane
  colors its icons from exactly this [M 0x19f0a0:134-180: status 0 → `0xd22d35ff`, 1 →
  `0x2d61d2ff`, 2 → `0x7f7f7fff`].
- **The Top Agent / Assassin target** (scenario 0x400): the globals are inverted vs their names
  — `DAT_0030d774` holds the ASSASSIN, `Assasin` @0x30d770 holds the TARGET [M 0x18b538:6,
  0x18b558:6]; the target's `MPSettings+slot·0x30+0x20` word is overwritten as the is-target
  marker [M 0x18aa70:38-41]. The pane shows the assassin/target icons [M 0x19f0a0:102-113]. →
  `snapshot.assassin` / `snapshot.target` (the port's `MpMatch::assassin`/`assassin_target`,
  already mapped) + `MpScoreRow::is_target`.
- **KOTH**: no hill meter and no radar blip (kind 5 unused) — the HUD shows only the 3-line
  score (scenario 0x40000800/0x60001000) and the 0x10 on-hill status bits (bot voice [M
  0x186ae0:95-115]). The port's rules.rs already skips the bit as presentation; the snapshot
  needs nothing extra.
- **Name tags**: gated on `MPSettings+0x1c4` (the port's `settings.radar`) or scenario 0x400,
  within 100 units + LOS; FFA `0x1e504bff`, teams `0x5a1414ff`/`0x14145aff` [M 0x19f6e8:175-264].
  → `snapshot.names_on_radar` (the colors are constants in the doc — the draw reads them here).

## 3. The struct

`MpHudSnapshot::build(&MpMatch, viewer_slot, viewer_radar_on)` → one struct per frame:
`viewer_slot`, `scenario`, `time_left`, `rows: Vec<MpScoreRow>` (+ `sorted_rows()`), 
`team_scores`, `flags`, `uplinks`, `assassin`/`target`, `names_on_radar`, `radar: Vec<MpRadarObj>`.
Getters: `MpScoreRow::display_score` / `::score_lines`, `MpHudSnapshot::timer_string`,
`radar_team_color`. The draw side converts `MpRadarObj` → `nf_view::hud::RadarObj`
(pos/argb/kind — the same three fields) and feeds `radar_project`.

## 4. Tests

`crates/nf-game/tests/mp_hud_data.rs` — headless (no ISO), a match's snapshot matched against
the sim state, each with a sister assertion: the rows read the sim's own kill/death counters in
slot order and sort descending by display score (Arena = kills); the score-line count per
scenario family; the timer is `limit − clock` in seconds (`08:30` at 90 s of 600) with the
00:00 clamp and the minute wrap; the radar list excludes the viewer, drops dead slots, tints
teams red/blue and FFA white, and goes EMPTY with the toggle off; the flag carriers / uplink
owner+controller+RGB / the Assassin target marker ride the snapshot.

## 5. Deviations ([S], for the report)

- The per-player radar toggle is a NEW port field (`MpPlayer::radar_on`, the game's +0x28).
  Which menu row writes it (the Setup page's Radar row) is 611/611b's page state — the port's
  `FrontEnd.mp_radar` currently feeds `MpSettings.radar` (+0x1c4, the NAME-tag gate). A live
  check can settle which record the menu writes (below).
- Dead/limbo blip gates: the port's `MpSlot::dead` stands in for the `obj+0xff` state machine +
  BLData exception (0x189840:41-92); the port's players have no per-player BLData yet.
- `MpSettings.time_limit` semantics: the port's menu path stores SECONDS (the confirm's ×60
  folded in); the game stores minutes and converts at the confirm. The doc comment in mod.rs
  says minutes. Not changed here (shared struct) — flagged for the owners.

## 6. Live checks for the main agent

1. **The radar toggle's record**: on a 2-player split-screen MP match, poke
   `MPSettings + 1·0x30 + 0x28` (0x2a47a0 + 0x58 = **0x2a47f8**) to 0 with gdbwatch — expect
   player 2's radar ring to disappear while player 1's stays (access: write; the readers are
   MP_GetRadarObjects 0x189840:22 and HUD_RadarUpdate 0x19f6e8:102).
2. **The names gate**: poke `MPSettings+0x1c4` (**0x2a4964**) to 0 in an FFA match — expect the
   name tags to vanish while the blips stay (read at 0x19f6e8:177).
3. **The timer**: in a match with a 1-minute limit, watch `MPGame+0x190` (**0x2a4b10**)
   accumulate and the HUD string flip at 10 s left — confirms the seconds semantics and the
   mode-2 format (read at 0x184268:144-147).
4. **The assassin marker**: in scenario 0x400, check `MPSettings + target·0x30 + 0x20` = 1 after
   `MP_assassinReset` (write at 0x18aa70:38-41) — the port models the marker via
   `MpMatch::assassin_target` instead; the game's field is shared with the team word.

## 7. 16 participants and the in-match draw (mp-hud-16, port-only [S])

The port allows 16 participants (700v8 mp-16-slots); the game's layout is 8 (4 humans + 4 bots
[M `MP_Start` 0x183a50:116-160]). What scales with the count in the game's own HUD:

- **The score pane** is one per viewer (the team line over the viewer's own line [M
  0x19f0a0:56-70]) — it never lists participants, so the count does not reach it.
- **The radar** carries one blip per other live participant plus the scenario markers, capped at
  the pane's 24 runtime blips (`RadarPane` CREATE+0x16 [M 0x19f6e8:108,153]). The worst
  16-participant list is 15 players + 8 uplink terminals = 23 — inside the cap
  (`hud::RADAR_BLIP_CAP`, pinned by `tests/mp_hud_16.rs`).

Before this task the port drew NONE of the MP panes in a match: `nf_view::hud::overlay` is the
SP pane set and `radar_frame` had no caller; the radar textures were not loaded. The port's
in-match MP draw (`nf_view::hud::mp_panes`, called from `overlay` when `Sim::mp` is set):

- the radar via `radar_frame` (the game's projection) at the top-right — the viewer matrix rows
  are the render camera's axes with row 0 = −right [S the mapping onto viewer+0x120]; the ring
  hides with the viewer's `radar_on` [M 0x19f6e8:102-105]; textures 0x03000088 / 0x0300016f load
  for MP scenes only (`hud::MP_TEXTURES`, lib.rs after `Scene::build`);
- the clock `mm:ss` top-centre (`timer_string`, `Timer_Seconds2String` mode 2 [M]) and the team
  game's `"%d - %d"` (`MP_GetTeamScore` 0x18b338 [M]) under it;
- a standings list (rank, name, display score) at the left edge between y 150 and the health
  ring (341) [S port-only — the game's MPScoreInfo pane (7 sprites @0x2BE888) is not decoded
  yet]. `MpHudSnapshot::board(max_rows)`: the debrief's order and rank walk; when cut, the
  viewer's row replaces the last visible row (true rank kept) and a "+N more" line follows.
  `nf_view::hud::mp_board_layout`: ≤ 8 participants → the HUD font's size, one font line per
  row; 9–16 → constant on-screen size like the online lobby (`k = 720 / view_h`, 0.75 of the
  720p row; the window loop stores the height via `hud::set_view_h`), as many rows as fit.
- Render aid: `NF_MP_BOTS=n` adds n bots to the local `--mp` default match (headless `--shot`).

That self-chosen placement is superseded by §8 (mp-hud-real): the clock and the list now sit
where the game puts its own sprites, and the list draws only past 8 participants.

## 8. The game's MP HUD, decoded and measured (mp-hud-real, 2026-10-18)

How measured: the real game in PCSX2 driven only by PINE writes (no menus, no input) — `tools/trace/mp_goto.py`
does P_MPCONFIRM 0x20ec00's accept from outside: MPSettings players
/ globals (Quick Game values, +0x1ac = 1 human, bot records 0x2deeaa with `mpbots` = 1),
`GameState+0x54 = 1` (`ResetMap_GameInit` 0x1d5080 copies it into MPSettings+0x180 — writing
+0x180 alone is overwritten [M 0x1d5080:7]), the `MP_setLoadingSkins` flags
(`MP_skins+0xc`), then `goto_level.py`'s ResetMap push. EE snapshots of a 1-human Arena and a
Team Arena match on 07000023 were walked from `BLData+0x7e0` (HUD) → pane k at
`HUD+8+k·0x20` → its sprite array (runtime sprite: +0x10 colour, +0x14 colour2, +0x18 text,
+0x22 flags, +0x24/+0x26 x/y, +0x2c/+0x2e w/h, +0x38 layer) [L]. Every number below is that
runtime rect, re-derived from the create code [M].

**`HUD_FixMP` 0x19b110** clamps every MP pane sprite into the viewer rect; with one viewer
(MPSettings+0x1ac == 1) the rect is inset 16 on all sides → x 16..496, y 16..432 [M :27-31; L
viewer0 +0xf0..+0xfc = 0/512/0/448]. Split screen (2–4 viewers) insets per side — not used by
the port (one viewer per window).

**Score pane (MPPaneList[13], create 0x19ed10, update 0x19f0a0, `MPScoreInfo` 0x2be888)** —
`nf_game::mp::hud::MpScorePane`:

| sprite | what | tex | FFA rect [L] | team / bit30 / 0x400 [L Team Arena] |
|---|---|---|---|---|
| 0 | text, font 2 (style 0x30cc90 = FF 02), white, colour2 outline (flag 0x2000) | — | (40,13) "\n<kills>\n<deaths>" | (40,30) "<own> - <other>\n<kills>\n<deaths>" |
| 3 | team star | 0x03000188 | hidden (y −1) | (16,16) |
| 4 | gun = kills | 0x03000187 | (16,17) | (16,34) |
| 5 | skull = deaths | 0x03000186 | (16,35) | (16,52) |
| 1/2/6 | objective icons 32×32 | per scenario | (16,376)/(16,376)/(16,400) | Assassin (16,384)/(16,368); Uplink (16,352)/(32,352)/(48,352) tex 0x03000185 |

The text colour2 is black in FFA and the TEAM's colour with a team line — 0xff0000ff team 0,
0x0000ffff otherwise [M 0x19f0a0:52-70: the store is +0x14]; the capture shows white digits ringed
red. Scenario 0x400 / 0x40000800 print score/kills/deaths (3 lines) [M 0x189730]. Icons: CTF —
sprite 1 = 0x03000182 while carrying the other team's flag, tinted that flag's colour; Assassin —
1 = 0x03000180 (I am the assassin), 2 = 0x03000189 (I am the target); Uplink — 1/2/6 tinted by
`MP_getUplinkStatus(0..2)` (red/blue/grey); GoldenEye (0x0300018b/c) and Espionage (0x0300018a)
icons are decoded but NOT drawn yet (the snapshot carries no per-viewer GE/blueprint holder) —
open. Icons 3/4/5 are additive (flag 0x100) [L 0x2be888].

**Clock** — `TimeSpr` (`MPTimeInfo` 0x2a4710, a separate sprite made by `MP_Start` 0x183a50, not
a pane): font 1 (the header face, style 0x30cbd0 = FF 01 FE 03 → centred), (256,32) for one viewer
(MP_Start moves y to 0x20 [M :186-200]), white with a black outline; only when MPSettings+0x1a0 !=
−1 [M 0x184268:138]. **Status line** — `StatusSpr` (`MPStatusInfo` 0x2a4740): font 2 centred at
(256,48): "Game Over :" + name + " Won" / "Game Over :" + "A Draw" / "Time Up!" + "Press START" /
0x10's "Player %s Won" [M 0x1848d0, 0x184bd0 case 2] → `nf_game::mp::hud::status_line`.

**Radar ring** — RadarSprInfo (512,0) through FixMP → **(432,16)** 64×64 [L] (the port drew
(448,0)).

**MP health pane (MPPaneList[1], 0x19b390 / 0x19b630, `MPHealthSprInfo` 0x2be6b8)** — no SP swirl:
backing 0x03000172 (16,416) 128×16 layer 0x1d; bar 0x03000171 same rect, width = uv width =
health/(100 + handicap)·128, colour 0x52a88b / <0.5 0xc6984e / <0.2 0x9d1912, layer 0x1c; armour
strip 0x03000170 at (24,420) width armour·2.56, layer 0x1b, hidden at 0 [M :62-90; L]. Sprites
4–7 are the same damage-direction glow as the SP pane's 20–23.

**MP ammo pane (MPPaneList[0], `MPAmmoPane` 0x2be698, update 0x19a7f0, `MPAmmoSprInfo`)** — pane
origin (396,390); the count at (496,408), the fire mode / weapon name at (496,428), right-aligned
font 2; no rounds strip (the update gates the strip on the pane's sprite count) [L; M
0x19a7f0:247,275].

**Kill message** — `MP_PlayerKilled` 0x188f70:100-107 posts `Text_AddMsg(last shooter, 0, 1,
"Killed %s" (0x200004d) % victim name, 0, 0xb4)`; the MP info pane (MPPaneList[4],
`HUD_MPUpdateStatusPane` 0x19ca88, `MPInfoStatusSprInfo` 0x2be838) shows it as one white text at
(16,88), font 2, black shadow, no bar; alpha = time·17 under 15 ticks [M; L the sprite rect].
The port queues `MpMatch::kill_msgs` (recipient, victim) — presentation only, not hashed — and
the recipient is the credited killer, not the +0x20 last-shooter word [S: bot-vs-bot rounds do not
go through `register_bullet_hit`, so that word can be stale in the port].

Port: `nf_view::hud::mp_panes` (score pane, clock, status, radar, MP health),
`overlay`'s MP branches (no SP swirl/strip, MP ammo origin, MP info text). The compact standings
list stays only for > 8 participants [S]. Render aids: `NF_MP_SCENARIO=<hex>`, `NF_MP_TIME=<s>`,
`NF_MP_KILLMSG=<victim slot>` on a local `--mp` render. Tests: `nf-game/tests/mp_hud_pane.rs`
(layout, texts, kill queue, status line), `nf-view/tests/mp_hud_real_live.rs` (the drawn rects,
no swirl/strip in MP, the kill message reaches the INFO queue).

Side-by-sides (agent15 scratch, port left / PCSX2 right): `sbs_hud_ffa.png`, `sbs_hud_team.png`
(+ the port's kill message), `sbs_debrief2.png`. Layout and presence match; the port's 16:9 window
spreads the left/right anchored panes wider than PCSX2's 4:3. The PCSX2 CRT filter makes colours
non-comparable.

Open: the flag-0x2000 text outline is the port's reading of the captures (one-unit ring in
colour2) [S]; GoldenEye/Espionage pane icons; split-screen FixMP insets.
