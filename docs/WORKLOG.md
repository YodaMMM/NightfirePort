
## mp-score-lines

2026-11-07 11:18 UTC, branch `s/mp-score-lines` (from origin/master).

Task: the MP score panel shows the game's lines in every scenario — `MP_GetPlayerScore`
0x189730 prints three lines (score, kills, deaths) for Assassin and KOTH FFA, two otherwise
[M :13-27]; `HUD_MPUpdatePane` 0x19f0a0:50-51 formats it beside `MP_GetTeamScore`. The port's
`MpScoreRow::score_lines` (crates/nf-game/src/mp/hud.rs) ports the split but nothing calls
it; the live rows go through `display_score`. Compare the panel per scenario (values, order,
the :53 team-colour arm, number format), use the ported split on the live panel, delete what
it replaces, pin through real matches on Assassin, KOTH FFA and Deathmatch (score a kill,
read the panel text), list render frames for the main agent.

- [x] setup: the sandbox survived (repo/work/ISO from the mp-object-deleted session); branch
  cut from origin/master, WORKLOG started, pushing before code work.

- [x] 12:10 UTC: done — the live panel's player lines now go through `MpScoreRow::score_lines`
  (the 0x189730 split; the inline scenario branch deleted from `MpScorePane::build`); the
  pane's values/order/colour/format verified line-by-line against 0x19f0a0 + 0x189730 +
  0x18b338 (already faithful); the trio pin (Assassin "5\n1\n0" via the +5 target kill, KOTH
  FFA "0\n1\n0" — the bit30 gate blocks the kill score, DM "\n1\n0"); 1181 tests green across
  nf-game lib + every mp_*_live + mp_world_sync_live + the mp hud binaries (2 stress/net
  binaries SIGKILLed by the sandbox, see the report).
