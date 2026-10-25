# mission-outros — the report

Task: the end-of-mission cut-scenes do not play — 0700004a (Night Shift's end: "end of level
cut-scene played but end of mission cut-scene didn't") and 07000004 (Double Cross: the "Mission
Complete" box shows, the cut-scene that should follow doesn't). This task owns the outro; 04's
objective text is task 10-level-04-mission-complete-early's.

## The mechanism, from the decompile + the disc (all [M] unless tagged)

**The level-end vs mission-end distinction.** The two endings are different machines:

- **0700004a is a level end, not a mission end** [L the mission table 0x2a41e8: level 0x0700004a,
  mission 0x07000011, **part 2**; part 3 = 07000013]. Its ending: the in-level pack cut-scene
  (the NIS 0x060006cf [D 0780004a.bin]) whose stream 27 ends writing **ch 0x22** [D — the parsed
  script: `Event kind 0xD params [34, 1]`]; the ScriptPlayer with **out 0x22** starts on it [D
  the placement table], and the lib's **0x060006d3 fires event 0x12 with the movie code
  0x0720004a** [D — the parsed stream 9] — the script-driven `ResetMap_LevelToLoad` [M
  Script_EventHandler 0x1d94c0:189-199]. The code is NOT in `UsingMovie` [M 0x107eb8] and has no
  script arm [M 0x1d6360's switch], so state 6 pops at once and phase 2 chains it:
  `GetLevelToLoadFromMovieCode(0x0720004a)` → 0x0710013 (the briefing; itself not whitelisted →
  no movie) → masked to **0x07000013** [M 0x1d3560]. 0700004a's own data writes no ch 0x4a (the
  "Gain access to lobby" condition channel [L 0x2a41e8]) — the mission cannot complete there; the
  objective folds on 0700013's spawn TouchOnce #153 [D 07000013.bin].
- **07000004 is the mission end.** The kill → ch 0x54 → the #46 mux → ch 100 → `Mission_Update`
  case 6: `LevelToEndTo = 0x0730004` (the compiled switch, case 0x0700004 [M 0x182c60:89-90]) →
  case 7's `GS_PausePlayer(1,0)` hold [M :123-130] → case 4's
  `ResetMap_LevelToLoad(0x0730004, 0, 0)` [M :65-76, 0x1d3800]. Likewise 0700013's completion
  routes to **0x0730013** [M :107-110].
- **What plays at a mission's last level**: `ResetMap_Load` 0x1d6360's movie branch
  (`(code & 0xff0000) != 0`) — the loader first fetches the shared movie PTP
  (`LoaderLoad(0, 0x07000500)`), then for the codes in its script switch **`SP_Create`s the code's
  subtitle script** beside `psiStartBackgroundMovie(code)` [M 0x1d6360:101-112], and GameFlow
  state 6 holds while the movie decodes (`psiMovieFinished` = `BGFMVPlaying == 0` [M 0x1082e8]).
  The whitelist is `UsingMovie` 0x107eb8 [M]: the outros 0x0730004/08/0b/0d/13/17/1b, the
  briefings 0x0710001/05/09/0c/11/14/18, and the 0x7330048-family specials. Phase 2 routes every
  0x73 outro to the front-end map 0x0700048 [M 0x1d3560], whose load tail opens the queued
  0x40000036 results page [M 0x1d6360:160-163].
- **The outro's script is a subtitle track.** 07000500.bin carries one type-7 script per
  whitelisted code [D — `nf-extract bin-list`; the pairs are 0x1d6360's switch verbatim:
  0x0730004 → 0x0600095d, 0x0730008 → 0x06000962, 0x073000b → 0x06000964, 0x073000d → 0x06000966,
  0x0730013 → 0x06000968, 0x0730017 → 0x0600097a, 0x073001b → 0x0600096c; the briefings likewise].
  Each decodes to a single stream of timed `Text` commands [D]: 0x0600095d = **8 entries, length
  2695** (the first at t249, show 403, label 0x06000061); 0x06000968 = **4 entries, length 2292**
  (the first at t468, show 338, label 0x060000f1). The labels bind in USATxt.dat [D — the port's
  `Lang::bind` resolves them].

## Why nothing played, and what was ported

The two user-visible halves of an outro are the PSS video and the subtitle script. On this
sandbox's slim on-foot image every /MOVIES/*.PSS is a **0-byte stub** [D — the ISO tree walk], and
the game itself skips a movie whose open fails: `psiStartBackgroundMovie` gates on `UsingMovie`,
then `strFileOpen` — on failure `BGFMVPlaying` stays 0 and state 6 pops at once [M 0x1081a8's
strFileOpen arm + 0x1082e8]. The port's flow matched the game (the FE's `mission_ended` →
`outro_for` → the movie state → the resolve — pinned by reward-after-outro), but with no movie
data and **no subtitle script** the outro was a black instant — "the cut-scene doesn't play".

Ported in this task:

1. **The subtitle track** (crates/nf-view/src/menu.rs): `outro_script_for` (0x1d6360's 14-arm
   script switch), `SubtitleTrack`/`SubtitleEntry`, and `subtitle_track(iso, code)` — reads
   07000500.bin off the disc [D], parses the code's script (`LevelBin` + `Nis::parse`), and
   returns the timed entries + the script's length.
2. **The track runs beside the movie** (the FE's state-6 tick): the clock advances per tick while
   the movie state holds and the active `(label, ticks-left)` is exposed for the draw — the
   game's `SP_Create` + the Text rendering over the movie [M 0x1d6360].
3. **The no-movie hold** (the host's `mission_movie_glue`): when the PSS can't start (a stubbed
   image), the state-6 hold now runs the track's length instead of popping at once, and the
   start is not retried after the arm. **[S] unverified as game behaviour**: on a disc with a
   stubbed PSS the game pops at once [M 0x1082e8]; the track's length is the port's stand-in for
   the retail PSS's duration (the track spans the movie [D — 2695 ticks ≈ 45 s, 2292 ≈ 38 s]).
   On a full image the PSS plays and the track runs beside it exactly as the game's does.

## Verification (what ran)

- New: `crates/nf-view/tests/mission_outros_live.rs` (3 tests, green):
  - `the_double_cross_outro_runs_its_subtitle_script` — the track's disc data (the length, the
    first stamp/label, 8 entries, the label binds), then the host-shaped end sequence through the
    REAL `mission_movie_glue` + the FE flow: the movie state with `movie_code = 0x0730004`, the
    track loaded (the script started — the port of `SP_Create`), the first subtitle shown during
    the hold, the state up ≥ 2690 ticks (through the track's last full entry at t2690), then the
    resolve to the front-end map [M 0x1d3560] and the results page [M 0x1d6360:160-163].
  - `the_phoenix_fire_outro_runs_its_subtitle_script` — the same for 0x07000013 → 0x0730013 →
    the 0x06000968 track (4 entries, length 2292).
  - `the_4a_level_end_chains_to_13_and_the_mission_stays_open` — the chain pinned at the data
    level: the pack cut-scene's s27 ch-0x22 write [D], the out-0x22 SP [D], 0x060006d3's event
    0x12 with 0x0720004a [D], `resolve_exit_code(0x072004a) = 0x07000013` [M 0x1d3560], and the
    distinction pins (`outro_for(0x070004a) = FRONTEND_LEVEL` [M 0x182c60:93-95] — no outro on a
    mid-mission level; `outro_for(0x0700013) = 0x0730013`); plus the no-ch-0x4a-writer scan over
    the parsed 4a scene [D] — the mission cannot complete on 4a.
- Adjacent binaries re-run green after the flow changes: `reward_after_outro` (3),
  `doublecross_transitions_live` (6), `fix_mission_live` (4), `walkreal_07000004_live` (3),
  `briefing_plays_live` (its NF_ISO-gated body skips headless — 0 tests selected).
- The full four-crate suite: the build is green; the run needs ~45–60 min in the foreground and
  this sandbox kills background jobs between tool calls, so the run does not fit the window (the
  same constraint the previous session reported). The touched surface (the FE flow + the movie
  glue) is covered by the binaries above; the full run is left for the merge window.

## What's left

- The full 806-binary suite pass on the merged tree.
- **The draw**: the active subtitle (`flow.outro_sub_active`) is exposed but not yet drawn over
  the movie frame — the render is the main agent's (lesson 6). Render list:
  - the Double Cross outro (the mission end on 07000004): after the "Mission Complete" fade, the
    movie screen with the track's text — the first line at t249 (label 0x06000061), the last full
    one at t2480 (0x06000067);
  - the Phoenix Fire outro (0700013's end): the first line at t468 (0x060000f1);
  - a full image (NF_MOVIE_ISO) render: the PSS frame with the subtitle over it.
- The real-input drive to 0700004a's pack cut-scene: the route rides the lift, whose rider line
  has known port gaps (`walkreal_0700004a_live` is #[ignore]d, docs/levels/0700004a.md) — the
  chain is pinned at the data level above; the drive lands when the ride does.
- With a full image: verify the decoder + the track run together (the code path is symmetric —
  the glue's Ok arm — but headless this sandbox cannot decode).

## Live checks for the main agent

1. 07000004, fresh boot, the kill: after the "Mission Complete" fade, state 6 with
   `psiStartBackgroundMovie(0x0730004)` — the PSS plays (retail image) and the SP 0x0600095d's
   subtitles show over it (the first at ~t249 of the hold). gdbwatch: `BGFMVPlaying` (0x30c69c
   per the port's [L note]) 1 during the hold, 0 at the pop; `GameState._16_4_` = 0x0730004.
2. 0700004a, the lift ride to the pack cut-scene: at its end ch 0x22 rises [D 060006cf s27], then
   the out-0x22 SP runs and event 0x12 fires with 0x0720004a — the game shows NO video here
   (0x072004a is not in `UsingMovie` [M 0x107eb8]) and chains to 0700013 through the briefing.
3. 0700013's end: the 0x0730013 outro — the PSS + the 0x06000968 subtitles; after the pop,
   `GameState._12_4_ = 0x0700048` and the results page.

## Usage

not visible to me
