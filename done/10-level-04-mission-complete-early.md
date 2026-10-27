## Your task: `level-04-mission-complete-early`, 07000004: killing the ninja completes the mission, but the outro cut-scene never plays and the objective jumps to "Destroy Mayhew's computer"

Real play on 07000004 (Double Cross, the courtyard; Bond near (-9.45, 5.47, 36.49)): killing the ninja is what completes the mission (the "Mission Complete" box) - that part is right - but the cut-scene that should follow does NOT play, and the HUD shows "NEW OBJECTIVE / Destroy Mayhew's computer" at the same moment, during normal gameplay (HUD and gun still up).

1. Drive the real level with real input to the ninja kill and log, per tick: the mission records' state, every objective write and HUD message (text id, requester), the script players that should start (the outro / next scene: their trigger channels, states), and the level-end path [M - the mission complete path, the objective writers (0x1827d8 family), the end-of-level cut-scene start (SP/script triggers on the completion channel), GameFlow's level-end state]. Find why the outro does not start and why the next objective is shown in gameplay.
2. Fix it so the kill leads into the game's outro exactly as the game does (the cut-scene plays; the next objective appears where the game shows it, not over gameplay); real-input test from the kill: the outro script runs, the gameplay HUD is down during it, the level ends/transitions as the game's.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-04-mission-complete-early`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T20:43Z

---
## Report (done, 2026-10-25T22:19Z)


# level-04-mission-complete-early — the report

Task: 07000004 (Double Cross, the courtyard) — killing the ninja completes the mission (the
"Mission Complete" box, right), but the outro cut-scene never played and the HUD showed
"NEW OBJECTIVE / Destroy Mayhew's computer" over gameplay at the same moment.

## The game's path, from the kill to the hub (all [M] unless tagged)

1. The target's death raises ch 0x54 (`DroneFunc_SetDeathChannel` 0x13e660; the raise fires on the
   death-anim exit); OUT trigger #46 "MultiplexOut" (out 0x54, list [0x64] [D placement]) mirrors it
   into **ch 100 = the mission-complete channel** every tick with no touch check
   (`Trigger_Update` case 5 [M 0x1b1a78:147-160]).
2. `Mission_Update` 0x182c60 case 1 sees ch 100 → internal 6 → **case 6** [M :85-122]:
   `MissionState = 6`, `LevelToEndTo = 0x0730004` (the compiled per-level switch, case 0x0700004),
   the type-3 "Mission Complete" banner (0x2000006, life `VIDEO_FRAME_RATE << 2` = 240 ticks).
3. **Case 7** [M :123-130] runs `GS_PausePlayer(1, 0)` EVERY tick of the hold and counts the 240
   down; the player's input is dead while the world and the HUD keep running.
4. **Case 4** (success arm, MissionState 6) [M :65-76]: `GameState+0x28 = 0x40000036`,
   `ResetMap_LevelToLoad(0x0730004, 0, 0)` [M 0x1d3800] + `GameFlow_PushState(60.0, 7, FadeClr)`.
   `Menu_IsDrivingLevel(0x0730004)` is **0** [M 0x202750 — only 0x0900001/2/3/5/6 are driving
   levels], so the on-foot loader takes it; param_3 = 0 and 0x0730004 is not in the 0x71 wrap
   switch, so `GameState+8 = 0x0730004` unwrapped, GameFlow state 3 pushed, then the fade (state 8)
   on top.
5. **The outro** [M `ResetMap_Load` 0x1d6360 phase 0]: `(0x0730004 & 0xff0000) != 0` → the movie
   branch — `hashtable_getitem(0x600095d)` → `SP_Create(...)` (the outro NIS object, loaded with
   the shared movie PTP `LoaderLoad(0x07000500)`), `GameFlow_PushState(0, 6, 0xff)` (the movie
   state), `psiStartBackgroundMovie(0x0730004, 0, SFXGetVolume())` — which builds
   `cdrom0:\MOVIES\30_FPS\07300004.PSS` [M 0x1081a8]. State 6 pops on `psiMovieFinished`
   (`BGFMVPlaying == 0` [M 0x1082e8]) or the skip action (0x19).
6. **Phase 2** [M 0x1d6360 LoadState 2]: `GetLevelToLoadFromMovieCode(0x0730004)` returns
   **0x0700048 — the mission hub** [M 0x1d3560; every 0x73 outro returns it]; the full hub load
   runs (phase 3) and its tail `MenuManager_Create` opens the queued 0x40000036 **results page**
   [M :160-163].

The level itself has NO outro script player: 07000004's placements carry exactly two ScriptPlayers
(#2 = the story scene 0x06000082, #3 = the intro 0x06000083 [D]), and both scripts' streams are
fully decoded in this task — no stream watches ch 0x54/0x64; every stream ends at the script's
830-tick length. The outro lives entirely on the movie-code path above. (The task brief guessed
"SP/script triggers on the completion channel" — the decompile says otherwise; the completion
channel's only reader is `Mission_Update` [M 0x182c60:24] and the PTP/drive path is NOT taken:
`Menu_IsDrivingLevel(0x0730004) = 0` [M 0x202750] and `GetLevelToLoadFromMovieCode` routes to the
hub [M 0x1d3560].)

## What the port did and what was fixed

The port's sim already latched `mission_end = Some(0x0730004)` at case 4 (walkreal's
`walk_the_full_mission_start_to_finish`), and the host already ran the end sequence
(`demo_results` → `FrontEnd::mission_ended` → `outro_for(0x07000004) = 0x0730004` →
`reset_map_level_to_load` → `movie_pending` → the state-6 movie → phase-2 resolve to
`FRONTEND_LEVEL` → the queued results page — the reward-after-outro machinery, previously pinned
only for 07000013). What was missing/broken:

- **The case-7 hold freeze was not modelled.** `Mission_Update` case 7's `GS_PausePlayer(1, 0)`
  [M 0x182c60:124] had no port counterpart, so Bond kept walking and shooting under the "Mission
  Complete" box — the user's "during normal gameplay (HUD and gun still up)". **Fixed**: the sim's
  input gate now folds `mission.internal == 7` in with the death/menu/net gates
  (crates/nf-view/src/scene.rs, the `mission_hold` arm, [M 0x182c60:123-130] — the case body is
  unconditional in internal 7, so the fail/death holds freeze too, as the game's does).
- **The kill→outro chain was unproven for THIS level.** The new real-input test
  (`walkreal_07000004_live::the_kill_leads_into_the_games_outro`) walks the level with the
  player's own movement, kills the ninja through the real fire path, and pins, in order: case 6
  (MissionState 6, the banner text "Mission Complete"), the case-7 hold (Bond's position does not
  drift under a held forward input, the banner's life ticks down, `hud_frozen` stays false — no
  script camera gate), case 4 (`mission_end = Some(0x0730004)`, internal 5), then the host's end
  sequence through the real `FrontEnd` flow: `movie_pending`, the queued 0x40000036, the movie
  state up with `movie_code = 0x0730004` (the port's stand-in for SP_Create(0x600095d) +
  `psiStartBackgroundMovie`), phase 2 resolving to `FRONTEND_LEVEL` [M 0x1d3560], and the results
  page opening at the load tail. The disc pin: `/MOVIES/30_FPS/07300004.PSS` exists in the image
  tree [D, read through the port's IsoImage::find].

## "Destroy Mayhew's computer" is the game's own cold-boot announce queue — not a port bug

The disc ships every objective record STATE 0 ([D] ACTION.ELF `.data`, all 24 rows' 55 records);
`Mission_Init`'s mode-1 arm never writes a prior-part record [M 0x182270:92-95]; the monitor's
case 0 then announces each earlier part's record once `record.level <= current` [M 0x1827d8:41-54],
one type-2 message per 300-tick slot. The game does the same on a fresh load ([L] the pinned
PCSX2 fresh 0700000b boot: part-0/1 records announcing). On a cold 07000004 boot the queue is the
seven earlier-part records (0x04000006 "Escort Mayhew to bunker", 0x04000007 "Destroy Mayhew's
computer", 0x04000008, 0x04000009, 0x0400000a, 0x0400000b, 0x0400000c — LEVELS order, confirmed by
the port's per-tick drain in this task's probe) plus the story scene's writes — about 2 700 ticks
≈ 45 s of banners after the intro settles. The playtest killed the ninja early (Bond
(−9.45, 5.47, 36.49), mid-courtyard — the story scene places Bond right beside the target), so a
stale boot banner was still on screen at the Mission Complete moment. On the campaign path (warm
arrivals; the mission snapshot restores the prior parts' states [L ee_start_0700000a.bin
@ 0x2a3f78, states 2/3/4]) no such banner exists. The new test pins the kill path itself: the
objective queue at the hold contains only 0x0400000d + 0x02000053 — **no 0x04000007 write happens
on the kill path** (its record left state 0 at boot via case 0, never re-armed).

## The outro movie data on this sandbox's image

The sandbox ISO is the slim on-foot image: `/MOVIES/30_FPS/07300004.PSS` exists as a **0-byte
stub** [D — a full ISO9660 walk of the image; 186 movie entries, the PSS data stripped]. The
port's `Movie::load` bails on a stub ("the slim on-foot ISO has no movie data; point NF_MOVIE_ISO
at an image with movies") and the flow's state-6 gate pops at once — exactly the game's own
`BGFMVPlaying == 0` arm [M 0x1082e8]: the game on this disc would skip the outro the same way.
With a full image (NF_MOVIE_ISO/NF_ISO) the decoder starts and the outro plays. The new test pins
the flow with `movie_playing = true` (the decoder-started stand-in), so the code path is covered
either way.

## Notes

- The `ptp_handover` dev stand-in (drive-ptp): the sim's LevelEnd arm still runs it for 0x0730
  codes (the fabricated drive return, pinned [S] by `walk_07000004_ptp_live`). The game's own path
  never touches the PTP/drive machinery here [M 0x1d3800 + 0x1d3560]; the stand-in is the port's
  documented dev auto-continue (ptp.rs), left as pinned — its only observable effect is the
  stashed driving return the hub's dev path reads.
- The kill's own banner sequence on the real path (this task's probe, the skipped-cut-scene
  timeline): at the kill tick ch 0x54 and ch 100 rise together; obj0 folds 3 → 4 with the
  "OBJECTIVE COMPLETE" (0x02000053) notice; the "Mission Complete" banner posts 2 ticks later.

## Verification (what ran)

- New/updated, green: `walkreal_07000004_live` (4 tests, incl. the new
  `the_kill_leads_into_the_games_outro`), `fix_objective_banner_live`,
  `fix_objective_messages_live`, `objectives_0b_start_live`, `objective_banner_timing_live`,
  `objective_hints_flow`, `level_carry_live`, `mission_stats_carry_live`, `fix_level_chain_live`,
  `campaign_flow_live`, `fix_cutscenes_live`, `fix_cutscenes2_live`, `walk_07000004_live` (52
  tests), `fix_death_loop_07_live` (3), `fix_mission_live` (4), `doublecross_transitions_live`
  (6), `reward_after_outro` (3), `walk_07000004_ptp_live` (4), `level_end_live` (6).
- Crate runs (`NF_SILENT=1 cargo test --offline --release --no-fail-fast -- --test-threads=1`):
  **nf-game + nf-formats + nf-audio: 1 091 passed, 4 failed, 9 ignored** — the 4 are the
  disc-image golden set: `disc_golden`'s 3 (bigf header/tables/directory) and
  `identity_disc::the_real_disc_passes_the_identity_check` (the image measures 518 435 sectors vs
  the retail 1 191 072 [D identity check output] — the slim image, the documented cause).
- The full `tools/test/suite.py` run (806 binaries): the build is green (254 s) and the run phase
  was started three times, but this sandbox kills background processes between tool calls and the
  foreground tool cap (≈10 min) covers only a fraction of the ≈45–60 min run (26 binaries through
  the shared log before the stop, all green, `doublecross_transitions_live` included). The bins
  listed above cover the mission/objective/carry/chain/death surface this task touches; the rest
  of the suite needs a longer window (the next message can run it on the merged tree).

## What's left

- The full 806-binary suite pass on the merged tree (the constraint above).
- A render pass on a full image: the outro state (the movie) after the fade, and the results page
  after it — the port's flow is pinned headless; the picture is the main agent's render.
- The 240-tick hold's visual: Bond frozen under the banner (the freeze is pinned positionally;
  a render of the moment would confirm the pose).

## Live checks for the main agent

1. Fresh 07000004 boot (goto_level.py, warm reset 0): watch the type-2 `Text_AddMsg` messages for
   the first ~2 500 frames. Expected ([M 0x1827d8:41-54] + [L the 0700000b precedent]): the seven
   earlier-part labels announce in LEVELS order (0x04000006, 0x04000007, …), one per ~300 ticks,
   starting after the intro/story scenes release the HUD (+0x2a) gate. If the real game does NOT
   queue them here, the port's case-0 reading needs a re-look (the record-state scan would need a
   gate the decompile doesn't show).
2. At the ninja kill: `GS_PausePlayer(1,0)` — Bond frozen under the "Mission Complete" box for
   240 ticks (gdbwatch on the player's input handlers or a poke-and-watch on `InternalState`
   0x30cb98: 1 → 6 → 7 → 4).
3. After the fade: GameFlow state 6 with `psiStartBackgroundMovie(0x0730004)` — the PSS plays
   (full image); skip action 0x19 ends it; `GameState._16_4_` holds 0x0730004 until phase 2.
4. After the movie: `GameState._12_4_ = 0x0700048` (the hub) and the 0x40000036 results page up at
   the load tail.

## Usage

not visible to me
