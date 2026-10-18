# mp-teams — team selection for the online lobby's team modes

Task 700v2 ("optional team selection depending on the mode"). The state lives in
`crates/nf-net/src/teams.rs` — its own module the lobby page (700v1, unmerged when this landed)
calls; nothing here draws (the main agent owns every page).

## 1. What the game does (the evidence)

**Which modes are team-based**: the scenario word's bit 29 (0x2000_0000). `MP_Init` 0x183438
derives `MPSettings+0x18C = (scenario >> 29) & 1` — the team-game flag every rule reads — and
`+0x190 = (scenario >> 30) & 1` [M 0x183438:41-42]. The couch menus test the bit raw
(`scenario & 0x20000000`) when offering the team pick [M C_RBMPSETUP 0x20de58:62,130-139;
C_SBMPOPTIONS 0x20e818:84; P_MPDEBRIEFING 0x20ec00:177]. The scenario wheel rows [L
`MP_SCENARIO_ROWS` in mp_menu_lists_live.rs; the `mp_scenario` table @0x2df508]:

- team rows (bit 29 set): Team Arena 0x2000_0002, CTF 0x2000_0004, Uplink 0x6000_0008,
  Demolition 0x2000_0040, Protection 0x2000_0080, Industrial Espionage 0x2000_0100, GoldenEye
  Strike 0x2000_0200, Team KOTH 0x6000_1000.
- FFA rows: Quick Game/Deathmatch 0x1, Top Agent 0x10, Assassin 0x400, KOTH 0x4000_0800.

Assassin is an FFA whose team word is a rules-internal role marker (the assassin reassignment)
and whose spawn table still splits [M 0x184f20:54-67, 0x188f70's assassin bypass] — the lobby
shows NO team column for it: nothing about it is picked.

**The teams are 2**: Phoenix (0, label 0x1c7, red 0xd22d35ff) and MI6 (1, 0x1c8, blue
0x2d61d2ff) [M 0x20de58:130-139 (the pick list), 0x20ec00:195-207 (the tints); L
mp-hud-data.md]. There is no third team anywhere; the spawn table has two halves.

**Storage and flow**: the per-slot record `MPSettings + slot·0x30 + 0x20` (u32 team) [M
0x1857c0 and every rules read]. Humans pick on the Join page into `mpjoin+4` (state 2, offered
only when the scenario has the bit) [M 0x20de58:130-139]; the character pick filters
`Menu_GetMPSkins` to the chosen side [M 0x1ffc98:41-58]; the confirm compacts the joined blocks
and writes `mpjoin+4 → MPSettings+0x20+i·0x30` [M P_MPCONFIRM 0x20ec00:109-144]. Bots carry the
team byte in their record (+0x0F, "1 = MI6" [M C_SBMPBTCHOOSE 0x210f08:26-33]) and `MP_Start`
copies it into the bot slots 4.. [M 0x183a50:140-141].

**Start gates** [M C_SBMPOPTIONS 0x20e818:50-130]: a team game rejects when either team has 0
members (the "%s team" error with the 0x1c7/0x1c8 labels); an FFA rejects more than ONE
MI6-side participant (0x1000311); any game needs ≥ 2 participants. **There is no auto-balance in
the game** — the only start rule is validation.

**What the rules do with the teams** (all ported in `crates/nf-game/src/mp/`; cited here because
the lobby's picks must reach exactly these reads):
- kill credit (kills/streak) only when FFA or the teams differ [M MP_PlayerKilled 0x188f70:157-163];
  the personal score add sits OUTSIDE that gate — bit30==0 ∧ scenario ∈ {1, 0x20000002,
  0x40000800, 0x60001000} [M 0x188f70:179-184];
- TEAM_DEATHMATCH (0x20000002 only) team scores: an enemy kill pays the killer's team |Δ|, a
  same-team kill (or an uncredited death) pays the victim's team [M 0x188f70:246-274];
- the friendly-fire gate blocks same-team bullet hits while `friendly_fire == 0` (the +0x198
  option), with the 3 s warn timer on the victim [M MP_RegisterBulletHit 0x1857c0, disasm
  0x185864-0x1858a0];
- the 64-point spawn table splits [0,0x20)/[0x20,0x40) per team in team games (and Assassin)
  [M 0x184f20:54-67, 0x185098:85-98];
- every objective scores `MPGame + team·4 + 0x180/0x184` [M 0x1861a8 CTF capture, 0x186d98
  Uplink, 0x186ae0 KOTH, 0x1870a0 Demo/Prot, 0x187568 GoldenEye strike targets].

## 2. The port module (`crates/nf-net/src/teams.rs`)

- `scenario_has_teams(scenario)` / `SCENARIO_TEAM_BIT` — the lobby's column gate [M above].
- `TEAM_PHOENIX`/`TEAM_MI6`/`team_label` — the pick values and the row's label ids.
- `TeamSelection` — the humans' picks (slots 0..4; bots don't pick [M 0x183a50:140-141]):
  `team_of`/`set_team`/`toggle` (rejections leave the state unchanged), `auto_balance(humans)`
  ([S] unverified, port-only — deterministic Phoenix/MI6 alternation; the game has none),
  `apply(&mut MatchSettings, humans)` (writes the human records' team; bot slots untouched) and
  `effective_teams(&settings, humans)` (the humans' picks + the settings' slot records — the
  array the start gate scores).
- `start_check(scenario, &effective_teams, humans, bots)` — the couch gates over the
  participants (humans at 0.., bots at 4..) [M 0x20e818:50-130]: `TooFewPlayers`,
  `EmptyTeam(team)` (the "%s team" error's label id), `TooManyMi6`.
- `TeamEdit` — the client→host pick over the session's DATA tunnel (`Session::send_data` →
  `Event::Data`), framed like the lobby codec's one-row edit (magic "NFLE", version 1,
  `{slot, field=1 (EDIT_TEAM), value}`) so whichever codec lands first decodes both [S — the
  framing mirrors docs/systems/net-online-lobby.md §6; online play is a port feature, the PS2
  has none [D]].

The default picks ([S] port default): Phoenix/MI6 alternating, so a 2-human team lobby passes
the both-teams gate with no edits; an FFA default of all-Phoenix keeps the ≤1-MI6 gate true.

## 3. The flow (what the lobby page does with this)

1. The page shows the team column only when `scenario_has_teams(settings.scenario)`; rows are
   the human slots, labels via `team_label`.
2. A row tap = `TeamSelection::toggle(slot)` → the client encodes `TeamEdit` → the DATA tunnel →
   the host merges (`set_team`) and republishes its lobby state (700v1's publish path).
3. Start: the host composes `effective_teams` (the picks + the bot rows' teams), runs
   `start_check`, then `apply` into the `MatchSettings` it already hands out through the
   handshake — the settings' player records cross the wire (protocol 2) and the app-side
   converter (`nf-view/src/mpnet.rs` `match_to_mp`) lands them in
   `MpSettings.players[slot].team`, where every ported rule reads them [M the citations in §1].
   The host then runs the match exactly as the couch confirm does [M mp-menus.md §4].

## 4. Tests (numbers/ids only; the main agent draws)

- `crates/nf-net/src/teams.rs` (unit): the column gate over all 12 wheel ids [L], the labels,
  the start gates against the couch errors, the pick rejections, the deterministic auto-balance
  [S], apply-into-settings (humans written, bot records untouched), the strict frame decode.
- `crates/nf-net/tests/teams_loopback.rs`: the column gate + start check through the public
  re-exports; a client's `TeamEdit` travels the real loopback DATA tunnel (`Event::Data`) and the
  handshake settings the client received carry the same team records.
- `crates/nf-game/tests/mp_teams_flow.rs`: the picked teams drive the ported rules — a cross-team
  kill scores the killer's team and the killer [M 0x188f70:246-274, 179-184], a same-team kill
  adds no kill but pays the victim's team [M :157-163, 246-274] (the personal add is outside the
  gate [M :179-184]), the friendly-fire gate blocks the same-team hit with the 3 s warn [M
  0x1857c0], and the spawn table splits per the picked teams [M 0x184f20:54-67].

## 5. Live checks for the main agent

- **PCSX2, a team scenario (Team Arena or CTF), the Join page with 2 pads**: the team pick state
  offers Phoenix/MI6 [M 0x20de58:130-139]; pick opposite sides, start, confirm the radar tints
  (red vs blue) and that a same-team shot warns instead of hurting with the Player-Mods
  friendly-fire row off [M 0x1857c0].
- **PCSX2, Team Arena, one side empty at the options page**: Start shows the "%s team" error with
  the empty side's name [M 0x20e818:84-103].
- **Render spots (port, once the lobby page lands)**: the team column hidden on an FFA row
  (Deathmatch), visible with Phoenix/MI6 rows on Team Arena; the Start button's error line for
  `EmptyTeam(1)` reads "MI6 team".
