# saved-records-across-parts — records and carry through a whole campaign

Status at the sandbox cutoff (2026-11-05 ~14:15 UTC): **DONE. The journey test landed green whole
(`saved_records_across_parts_live` 1 passed / 0 failed, 2.41 s): "Night Shift" — the mission of
07000009 → 0700000a [D mission::LEVELS' parts] — walked through the App's own pieces from the
mission list to the results under a saved codename, every part keeps the saved table, the bonus
word and the game-complete flag, the failed part replayed from the fail menu keeps them, and
quitting from the fail menu leaves the saved file byte-identical. The failed path was checked
against the game's own decompile [M] and one wrong test-side assertion was corrected to the game's
shape. NO record drop exists anywhere on the walked paths — the App's architecture (the swap
seeding, the entry-checkpoint serve, the sync-before-save) holds, and the test now pins it.**

## What the test walks (the App's own pieces, real code paths)

1. **The codename**: `create_codename("SAVED01")` (the create chain [M Menu_UpdateMessageBox
   case 1's write]) on a temp card, then the file is overlaid with the prior-session stand-in a
   real save holds — rows 0..=5 beaten in the MSSN nightfire status, `game_complete = true`
   (the campaign finished once [M GameState[0x52], Menu_SetLevelBonus 0x1ffa10's tail]), the bonus
   word's saved bits (the top two bits, outside the reward space [M the reward descriptors' bit
   range]) and a prior mission's best in `table[0]` [M ScoringTable 0x2bc640]. The codename then
   LOADS the App's way: `load_codename(2)` — the MSSN status IS the unlock mask [M
   Menu_SetNightfireStatus 0x2023a0] — and every marker is asserted on the loaded record.
2. **The mission list**: the difficulty wheel confirms (the pick rides `requested_difficulty`
   [M C_SBNFDFCTY 0x216ca8:15]; the list opens through the delayed 0x44 — the input block holds
   until it lands [M 0x202190:7]), Night Shift's row confirms and the mission start rides
   `requested_level` [M C_SBNFMAP 0x20b8d8:18-41; the lock gate :21-23].
3. **The parts**: each part loads through `load_with_carry_medals` — the loader EVERY menu pick
   and part transition takes [M lib.rs menu_after] — with the bonus word riding it [M the
   pickup-medal-gate line: only LS_LoadBonus and the award path write bonus[]], then the App's
   swap seeding runs [M lib.rs swap_in_scene's four-field copy of table/bonus/nightfire_status/
   game_complete]. The entry checkpoint is captured per part [M Player_InitWeapon's unconditional
   `Player_RamSave` 0x1a7ef8:325]. The part's objectives fire through their own channels (the
   campaign runner's fallback [M 0x1827d8]) and the forced verdict ends it. **The record markers
   are asserted at EVERY part load and after EVERY part's play** — nothing dropped.
4. **The failed part + the fail menu**: part 1 takes the fail verdict (ch 99 [M 0x182c60:77-81]).
   The fail path was checked against the decompile: **[M 0x182c60 case 4, the MissionState != 6
   arm] — `MenuManager_Create(..., 0x40000042, ...)` + `GS_PauseGame(1)` + `InternalState = 5`:
   the manager is created DIRECTLY with the fail page and NO flow field is written** (the test's
   first draft asserted `flow.page == 0x40000036` — wrong; the game leaves GameState+0x1c alone
   on this path, and the test now asserts the game's shape: the manager's page up, the flow
   staying in the runner state, no level request, no fade).
5. **The REPLAY row** [L page 0x40000042: REPLAY = 0; M P_ENDMISSION_Handler 0x216618:19-27]:
   the current level reloads WARM on the entry checkpoint (`ResetMap_LevelToLoad(
   GameState._12_4_, 1, 1)` + `Player_RamLoad(Bond, 1)` [M 0x216618:19-21]) — the test serves it
   through the App's own `fail_restart_save(entry_checkpoint)` + `load_with_carry_medals` + the
   swap seeding. **The record keeps (every marker), the mission layer re-arms (internal 1
   [M Mission_Init 0x182270:20-28]), and the verdict channels are clean** — the entry
   checkpoint's channels predate the fail, so the reload cannot re-fail (the death-loop-07 fix's
   own guarantee).
6. **The QUIT row** [M 0x216618:24-26]: the front end reopens on the mission list. **THE PIN: the
   saved file is byte-identical** — the game's QUIT arm writes no save anywhere (the card writes
   ride only the results save chain [M 0x208e78:24-32 + case 1]), and the port's file compares
   equal byte-for-byte. The session's record also stands in memory.
7. **The results + the save**: the complete leg rides the front end (the outro request [M
   0x182c60:89-116] → phase 2 → the results page [M 0x1d6360:160-163] → `reopen_queued_page`),
   `results_enter` runs against the live layer the mission ran with [M 0x208e78:77 + :169], the
   award stamps Night Shift's row + the next unlock, **adds the run's medal bits to the bonus
   word [M 0x1ffa10:71-82] without dropping the saved bits**, leaves the game-complete flag alone
   (only 1b sets it [M 0x1ffa10's tail]), then `sync_profile_progress` merges the tally before
   the save prompt can run [M 0x1984b0:301-308 + Menu_UpdateMessageBox case 1]. The prompt stands
   (the commit improved — any completed run scores > 0 through the time row [M 0x1984b0:120-131]
   — and SAVED01 is a saved codename [M 0x208e78:24-32]), Yes → the card flow → **the file now
   holds the whole record: the award, the medal bits AND every prior-session marker**.

## What was fixed

- **Test-side only.** One wrong assertion (the fail arm's flow.page — corrected to the game's
  [M 0x182c60 case 4]) and the runner's forced-verdict channels riding the warm capture (cleared
  at the warm entry — a real warm entry never finds them ON [M 0x182c60:77-135; the campaign
  runner's own warm-entry note]). **No `crates/*/src` change was needed: nothing on the walked
  paths drops a record.** The keeps the task names are all real in the port already — the swap
  seeding carries the four fields into every fresh sim, the fail-restart serves the entry
  checkpoint, the QUIT writes nothing, and the save rides the merged record.

## How this was verified

- `cargo test --offline --release -j2 -p nf-view --test saved_records_across_parts_live`:
  the iterate loop (6 runs — the reproduce-then-fix of each wrong pin: the iris-gated list pick,
  the fail arm's flow.page, the re-arm timing, the forced-verdict carry), the final whole-binary
  run **1 passed; 0 failed; 0 ignored; finished in 2.41 s**.
- The neighbors that share the front-end/save machinery, whole:
  `memcard_save_flow` **7 passed; 0 failed**; `fix_save_profile_live` **4 passed; 0 failed**;
  `campaign_flow_live front_end_chain_01_to_05` **1 passed; 0 failed (136.08 s)** — the
  closest existing journey, green alongside the new test.

## What was NOT run (honest)

The full prescribed suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats
-p nf-audio -p nf-view -- --test-threads=1`) did not run this session: the window went to the
journey iterate loop and the neighbor binaries above. The changed surface is ONE new test file
(`crates/nf-view/tests/saved_records_across_parts_live.rs`) plus the WORKLOG and this report —
no `crates/*/src` change, no shared-test-helper change (`crates/nf-view/tests/common/` untouched),
so no other binary's behavior can differ; the neighbors that share the machinery ran green whole.

## What's left

Nothing for the task's own scope. A natural follow-up (not owed): the same walk with REAL
walker-driven parts (the campaign drivers) so the results score comes from real kills — the
forced-verdict fallback scores through the time row only.

## Live checks for the main agent

None required — the walk is Sim/front-end level and reproducible headless. One optional
confirmation: in PCSX2, fail a mission part under a saved codename, QUIT from the 0x40000042 menu
to the mission list, then check the memory card's save in the Codenames page — the game's own
QUIT must leave the save's timestamp/content untouched [M 0x216618:24-26 writes no LS_Save]; the
port pins the file byte-identical.

## Usage

not visible to me
