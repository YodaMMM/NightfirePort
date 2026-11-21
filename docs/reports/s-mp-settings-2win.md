# s-mp-settings-2win: every multiplayer setting, host + joiner windows

Two real `nf-view` windows on one PC (host: Host Online, joiner: Join Online at 127.0.0.1),
LAN only, silent, a throwaway config dir per window. Every setting was changed through the real
one-screen lobby page (menu-script presses, no desktop input), the host pressed Start, and both
windows' logs (`[lobby]`, `[mp-settings]`, `[mp-equip]`, `[mp-props]`, `[mpcombat]`, the match
end lines) and one or more captures per window were read. Settings that exist only on the local
(split-screen) pages were checked in one window through those pages.

Two settings did not do what the game does. Both are fixed here with tests.

## Fixes

1. **Bots switched on in the online lobby played for the wrong side.** In the game a bot record
   is only ever switched on by the Bot Choose confirm, which writes the side from the character
   in the same step (`+0x0F = Menu_IsBotGood(char)` [M C_SBMPBTCHOOSE 0x210f08:26-43]). The
   lobby's Enabled toggle (Confirm, or a step on an Off row) skipped that write, and the four
   default records started on MI6 (`MpBot::fresh` set `team_mi6 = !side`): Snow Guard and Black
   Ops, both Phoenix characters, joined MI6 in every team game (all ten team-game runs logged
   `t1` for them in both windows). The records ship zeroed [D ACTION.ELF .data
   0x2deeaa..0x2def01]; the hub's first-open init writes only the characters [M C_SBMPOPTIONS
   0x20e818:26-31]; the start check reads `+0x0F != 0` as MI6 [M :72-80]. Now: fresh records are
   Phoenix, and switching a bot on in the lobby takes its character's side. After the fix both
   windows log `c6/.../t0 c7/.../t0` (run r4).
2. **The Health Handicap never reached a match.** The Setup page's handicap list (and the
   profile's Multiplayer Options row) write `MPSettings + slot·0x30 + 0x2c` [M C_RBMPSETUP
   0x20de58:112, P_CNMPOPTIONS 0x2144d8:14]; the confirm copies the record whole [M P_MPCONFIRM
   0x20ec00:113]; `Player_Init` and `MP_ReSpawn` spawn at `(float)(int)+0x2c + 100` health [M
   0x1910f8:128-131, 0x185368:71-72]. The port's confirm wrote a hard 0, and the respawn read the
   word unsigned (a negative handicap would have been 4.29e9 health). Now the pick rides
   `MpPlayer::armor_bonus` (agent 1 = slot 0, the port's one local agent), `MpMatch::human_health`
   reads it signed, and the match start, every respawn, a late join's spawn and the HUD bar use
   it. One window, local pages, handicap -50: `loadout: ... health 50` (was 100).

## The table

Runs and captures: `NightfirePort-work/audits/mp-settings-2win/<run>/{host,join}/` (not in the
repo). "Same" = the host's and the joiner's lines are identical.

| Setting | Value | Host | Joiner | Game docs |
|---|---|---|---|---|
| Scenario | Arena `0x1` (r0, r2, r3) | FFA panes, kills/deaths | same | y |
| Scenario | Team Arena `0x20000002` (r1) | team score pane, "Hurt by teammate" | same word, team pane | y |
| Scenario | CTF, Uplink, Top Agent, Demolition, Protection, Industrial Espionage, GoldenEye Strike, Assassination, KOTH, Team KOTH (s-row3..12) | each word in `[mp-settings]`, the scenario's HUD pane (uplink icons, team scores, three-line assassination pane) | same | y (start + HUD only, see unproven) |
| Map | Skyrail / Fort Knox / Atlantis | that arena loaded | same | y |
| Weapon Set | Normal | spawn P99 (0x6), 16 rounds | same | y |
| Weapon Set | Pistols | spawn P2K (0x2), 7; Fort Knox mounts take the set's turret | same | y |
| Weapon Set | Automatic | spawn 0x12, 32 | same | y |
| Points limit | 1 (r2) | match over on the first kill, frame 189 | same frame | y |
| Duration | 1 min (r1) | clock 00:56 at tick 240; debriefing at tick 3604, boards agree | same | y (×60 [M P_MPCONFIRM]) |
| Duration | 10 min default | `time 600`, clock 09:35 | same | y |
| Friendly Fire | On, Team Arena (r1) | hurt by a same-team bot | same | y |
| Professional Mode | On (r2) vs Off (r3) | Automatic head hit, raw 1.5: dealt 72 vs 6 | same lines | y (×3 [M Player_HandlePain 0x1902c8:79-81]) |
| Location Damage | On (r2) vs Off (r3) | head ×4 is in the 72; 6 = 1.5 × Multi 4 only | same | y ([M :52-77]; not varied apart from Pro Mode, the arithmetic splits it) |
| Team ID (lobby row "Radar", label 0x1e5) | Off (r1) / On (r3) | no name over the other player / "Player 2" over him | Off: same | y ([M HUD_RadarUpdate 0x19f6e8:177, 238]) |
| Enviro: Respawn (+0x1C0) | 0 (r1), 2 (r0) | word carried | same | word only |
| Enviro: Gun Emplacements | On, Fort Knox | `gunimps 2 turrets [1, 1]` | same | y |
| Enviro: Explosive Scenery | On, Atlantis | `destroy 10` (Off: 0) | same | y |
| Enviro: Grapple | On | owned 0x50 at spawn and respawn | same | y ([M MP_EquipPlayer 0x1855f8:27-34]) |
| Enviro: Mini Vehicles | Tanks (r1), Helicopters (r2) | `vehicles 12` drawn (Off: 0) | same | word + count only |
| Bots | 2 on | `bots created: 2` | same | y |
| Bot character | Yakuza c8, Black Ops c7 | records same | same | y |
| Bot skill | stepped (s8 / s3) | records same | same | y |
| Bot side | switched on without a character step | **was MI6 for Phoenix characters**, fixed: t0 | same | **n → y** |
| Player character | joiner Kiko (r1), Drake (r2) | host sees the Kiko body | joiner's own row echoed | y |
| Player team | joiner MI6 | `Player 2:t1` | same | y |
| Health Handicap (local pages, one window) | -50 | **was 100 health**, fixed: 50 | n/a | **n → y** |
| Bot Setup hit points (local pages, one window) | 100 → 125 | `hp125` in the match | n/a | y |
| Bot Choose (local pages, one window) | Snow Guard | Phoenix (t0) | n/a | y |

No run printed a desync, a resync or a mismatch.

## Tests

- `crates/nf-view/tests/mp_settings_2win.rs` (headless, 4): the handicap through the local
  pages into the settings and `human_health` (-50 → 50, +100 → 200, 0 → 100), the handicap
  across the wire round trip, fresh bot records on Phoenix and a lobby-enabled bot on its
  character's side, and the lobby start's team words. All four fail on the old code (checked:
  2 fail with the confirm's hard 0, 2 with the old lobby/record code) and pass now.
- Every `mp_*` and `net_*` nf-view test binary (58 binaries): 230 passed, 1 not settled.
  `net_soak_faults_live`'s 10-game-minute soak failed at its first line with AddrInUse: its
  fixed port 27980 was held by another checkout's run of the same test on this PC. Re-run on a
  free port it did not finish inside this check's 10-minute command limit, so it is not proven
  here. It builds its match from `MatchSettings::default()` (no lobby, handicap 0), which these
  changes leave as they were. After the comment-only edits: `mp_settings_2win`,
  `mp_lobby_one_screen`, `mp_menu_flow` and `mp_net_lobby` re-run green.

## Unproven

- Scenario rules past the start and the HUD (scoring, flags, uplinks, the bomb, the strike)
  were not played out in these short runs; earlier flag-path two-window runs covered CTF, KOTH,
  Demolition and Assassination.
- The Respawn policy's effect on where players come back, and which vehicle type Tanks /
  Helicopters put in the arena (only the count was read).
- Online clients have no handicap row; their slots keep 0. The host's own slot takes its
  profile handicap.
- Bot Setup's aim, speed, trait and recovery rows in a match (only hit points was read).
- Friendly Fire Off was not re-run through the lobby (the flag-path check covers it).
- Ravine (bots refused) was not run in two windows.
- Nothing here was compared against the game running live.
