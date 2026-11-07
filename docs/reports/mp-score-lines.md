# mp-score-lines — the score panel shows the game's lines in every scenario

Branch `s/mp-score-lines` (from origin/master 10956badb), 2026-11-07.

## The game's rule, read in full

- `MP_GetPlayerScore` 0x189730 [M 00189730 decompile]: scenario 0x400 (Assassin) and
  0x40000800 (KOTH FFA) sprintf `"%d\n%d\n%d"` — `(int)`-cast score float, kills, deaths
  [M :13-19]; everything else prints kills and deaths [M :20-27], and the two sprintf formats
  differ: FFA `"\n%d\n%d"` ([L] ee_snapshot 0x30cc18 = `b'\n%d\n%d'` — the leading newline is
  a blank first line where the 3-line arm prints the score) and team games `"%d\n%d"`
  ([L] ee_snapshot 0x30cc10).
- `HUD_MPUpdatePane` 0x19f0a0 [M 0019f0a0 decompile, read in full]: :44-48 fetch
  `MP_GetTeamScore(my_team)` and `MP_GetPlayerScore(my slot)`; the text arms — team string
  present and player string present: `"%s\n%s"` ([L] 0x30ccd0, team line first) and the
  sprite's colour2 (+0x14) := `0xff0000ff` when the viewer's team is 0 else `0xffff`
  [M :53-59]; FFA (team string NULL) -> `"%s"` with the player string only, +0x14 := 0xff
  [M :60-63]; the player-string-NULL arm writes the empty constant ([L] 0x30ccd8 = `b''`) —
  unreachable in practice (`MP_GetPlayerScore` always returns a heap string).
- `MP_GetTeamScore` 0x18b338 [M 0018b338 decompile]: FFA -> NULL; team games -> `"%d - %d"`,
  own team's score first ([L] ee_snapshot 0x30cc08 = `b'%d - %d'`; the score words are
  `MPGame+team·4+0x180`).

## What the port did and what changed

- `MpScorePane::build` (nf-game/src/mp/hud.rs) already printed all of this faithfully —
  verified line-by-line against the decompiles above: values, order (team line first, then
  the player lines), the FFA blank first line, the `(int)` float cast, and the :53 colour arm
  (`0xff0000ff` team 0 / `0x0000ffff` otherwise / `0x000000ff` FFA). The existing
  `mp_hud_pane` pins (layout, Team Arena colour per viewer, Assassin's star + three lines)
  already covered the rest.
- **Changed:** the player lines now come through `MpScoreRow::score_lines` — the ported
  0x189730 split, which had no caller — and the inline duplicate branch on the scenario word
  is deleted from `MpScorePane::build`. `display_score` keeps its own job (the board sort /
  `Menu_GetMPScore` 0x203af8 semantics) and is untouched.

## Verification — the trio pin through real matches

`mp_hud_pane::the_panel_text_after_a_kill_in_the_three_pinned_scenarios` scores a kill
through `MpMatch::player_killed` (the port of `MP_PlayerKilled` 0x188f70) and reads the panel
text:
- Assassin: the target kill as the assassin — score +5 [M 0x188f70:229-233], kills +1
  [M :157-163] -> the pane reads `"5\n1\n0"`; the victim's pane `"0\n0\n1"`.
- KOTH FFA: a plain kill — kills +1 [M :157-163] but the score word does not move: the
  personal gate needs `MPSettings+0x400 == 0` [M :178-182] and 0x40000800 has bit30 set ->
  `"0\n1\n0"`.
- Deathmatch: the FFA two-line form — the text is the blank line, kills, deaths
  (`"\n1\n0"`; the score word +1 [M :178-182] is NOT printed; [L] 0x30cc18).

## What ran (all release, --test-threads=1, per binary)

- nf-game lib: **688 passed / 0 failed** (5 ignored).
- nf-game mp binaries: mp_arenas_live 1, mp_debriefing 10, mp_hud_16 7, mp_hud_data 4,
  mp_hud_pane 6, mp_lockstep 0 (the binary has no tests), mp_map_objects_live 4,
  mp_match_rules 18, mp_never_mentioned 6, mp_status_bits 3, mp_teams_flow 4 — all 0 failed.
- nf-view mp binaries (every `mp_*_live` + `mp_world_sync_live`): mp_16_slots_live 3,
  mp_bodies_live 3, mp_body_anim_live 2, mp_bot_armour_online_live 1, mp_bot_body_sounds_live 1,
  mp_bot_objectives_live 10, mp_bot_voices_live 3, mp_bot_weapons_live 2, mp_bots_live 3,
  mp_bots_moving_live 3, mp_clock_gate_live 1, mp_combat_weapons_live 11, mp_core_live 5,
  mp_death_effects_live 3, mp_death_hands_online 2, mp_host_match_live 2, mp_hud_16_live 2,
  mp_hud_real_live 4, mp_world_sync_live 8, mp_match_end_live 2, mp_net_slots_post_move_live 2,
  mp_perf_16_live 2, mp_pickup_take_live 2, mp_pickups_visible_live 2, mp_remote_fx_live 2,
  mp_respawn_ammo_live 3, mp_scenarios_live 19, mp_shot_effects_live 2, mp_shot_gate_live 5,
  mp_site_damage_offline_live 4, mp_site_destruction_blast_live 4, mp_status_strings_live 1 —
  all 0 failed.
- nf-view other mp binaries: mp_debriefing_flow 6, mp_defaults_setup 12, mp_menu_lists_live 5,
  mp_net_lobby 11, mp_screens 5, mp_lobby_one_screen 7, mp_menu_flow 11 — all 0 failed.
- The rest of Rule 5: nf-formats 24, nf-audio 92, nf-view lib 138 — all 0 failed.
- **Total: 1181 passed, 0 failed.**
- **Not runnable in this sandbox:** `mp_stress_explosives_live` and `mp_online_failures` —
  the test process is SIGKILLed 10–30 s in, reproducibly, on this 4 GB box (no assertion
  output, no crash trace; the direct binary run dies the same way). Neither reads the score
  pane; the kill is environmental (memory/net sandbox), not this change. The suite fast
  runner could not stay alive in the background here either (killed between tool calls three
  times), so the above per-binary foreground runs are the suite's equivalent.

## What's left

- Nothing for the split itself. If the driver wants the fast-runner total line, re-run
  `tools/test/suite.py` on the merged tree in an environment where background processes
  survive.

## Live checks for the main agent (render frames)

1. Assassin frame: any arena in Assassin (0x400), 1 human, fresh boot — render the spawn view
   (e.g. `--bond <spawn x>,<floor y>+1.6,<spawn z>,<facing yaw>,0`). Expected top-left score
   pane: THREE text lines (score on line 1, kills line 2, deaths line 3 — all `0` at the
   start), the star icon shown at (16, 16), the gun at (16, 34), the skull at (16, 52), text
   pen (40, 30).
2. Two-line frame: the same arena in Deathmatch (FFA). Expected: NO star icon, text pen
   (40, 13), the first text line BLANK with kills on line 2 and deaths on line 3, gun at
   (16, 17), skull at (16, 35).
3. Optional team check (Team Arena): the first line reads `own - other` (e.g. `0 - 0`) with
   the red outline for a team-0 viewer, blue for team 1.

## Usage

not visible to me — this session's driver shows no token, credit or context-window figures.
