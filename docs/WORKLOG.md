
## saved-records-across-parts
Started 2026-11-05 13:22 UTC. Branch s/saved-records-across-parts (from origin/master).
Task: walk a mission of several parts through the App from the mission list to results, saving a
codename — each part keeps the saved table, bonus word and game-complete flag, a failed part
replayed from the fail menu keeps them too, and quitting from the fail menu leaves the saved file
unchanged. Check the failed path against the game [M]. Fix anything that drops a record and pin it.
Progress 2026-11-05 13:22 UTC: branch cut, WORKLOG started, pushed.

Progress 2026-11-05 14:12 UTC: DONE. The journey test landed
(crates/nf-view/tests/saved_records_across_parts_live.rs): Night Shift (07000009 → 0700000a, the
mission's own parts [D mission::LEVELS]) walked through the App's own pieces — the codename created
and loaded (the MSSN status IS the unlock mask [M 0x2023a0]), the difficulty wheel, the mission
list pick [M C_SBNFMAP 0x20b8d8], every part load through load_with_carry_medals + the swap seeding
[M lib.rs swap_in_scene's four-field copy], the forced part verdicts (the campaign runner's
fallback). Every part load and play keeps the saved record (the table/bonus/game-complete markers).
The fail path checked against the game [M 0x182c60 case 4, MissionState != 6: MenuManager_Create
directly with 0x40000042 + GS_PauseGame(1) — NO flow field written; the test's original flow.page
assertion was wrong and fixed to the game's own shape]. The fail menu's REPLAY keeps the record and
re-arms the mission layer (internal 1; the entry checkpoint's channels predate the fail [M
0x216618:19-21]); the QUIT row leaves the saved file BYTE-IDENTICAL [M 0x216618:24-26]. The results
award adds the medal bits to the bonus word [M 0x1ffa10:71-82] without dropping the saved bits, and
the save rides the whole record into the file. NO record drop found anywhere — the architecture
(seeding at every swap, the entry-checkpoint serve, the sync-before-save) holds; the test pins it.
Whole-binary green: saved_records_across_parts_live 1/0 (2.41 s); neighbors green: memcard_save_flow
7/0, fix_save_profile_live 4/0, campaign_flow_live front_end_chain_01_to_05 1/0 (136.08 s). The full
prescribed suite did not run (the window went to the journey + the neighbors; the changed surface is
one new test file — no crates/*/src, no shared helpers).
