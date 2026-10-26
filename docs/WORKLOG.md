# WORKLOG

## mission-outros (2026-10-25, sandbox expiring ~00:25 UTC)
Branch `s/mission-outros` off origin/master (b312d959). Task: the end-of-mission cut-scenes do not
play — 0700004a (Night Shift's end: "end of level cut-scene played but end of mission cut-scene
didn't") and 07000004 (Double Cross: the Mission Complete box shows, the outro doesn't; the 0x0400
0007 objective text is task 10-level-04-mission-complete-early's — this task owns the outro).
Prior mapping (my level-04 session, branch s/level-04-mission-complete-early @ aec52458): the
kill→outro path [M 0x182c60 case 6/7/4 → 0x1d3800 → 0x1d6360's movie branch (SP_Create(0x600095d)
+ state 6 + psiStartBackgroundMovie(0x0730004)) → 0x1d3560's resolve → the hub 0x0700048 → the
results page]; 07000004 has NO in-level outro SP [D]; the slim ISO's /MOVIES PSS files are 0-byte
stubs [D]; the port's FE end sequence (demo_results → mission_ended → outro_for → movie_pending →
ST_MOVIE → phase-2 resolve → results) is pinned headless for 07000013 and (on my branch) for
07000004. This task: the level-end vs mission-end distinction on BOTH levels, the movie table
(0x1d6360's script-id switch), the 0700004a pack cut-scene chain (0x072004a → 0x7100013 → 07000013
per 0x1d3560), and the port's gap.
Plan: reuse the sandbox (work/ + ISO + nf-queue); branch pushed at start; census c0700004a/11/12/13
mission records + the 070004a exit/movie triggers [D]; trace the port's 0x072004a resolve; port the
mission-end path so the outros play as the game's; real-input tests for both endings; suite in
foreground chunks (the sandbox kills background jobs); report + finish steps.

### mission-outros — the outro mechanism + the subtitle-track port (2026-10-25 ~23:55 UTC)
The level-end vs mission-end distinction, from the decompile + the disc: 0700004a (Phoenix Fire
part 3 of mission 0x07000011 [L the mission table]) ends at the LEVEL only — the pack cut-scene
0x060006cf's stream 27 writes ch 0x22 [D 0780004a.bin], the out-0x22 SP starts, and the lib's
0x060006d3 fires event 0x12 with the movie code 0x0720004a [D] — which has NO script arm and NO
UsingMovie entry [M 0x1d6360's switch + 0x107eb8] → state 6 pops instantly → phase 2 chains
0x072004a → 0x0710013 → 0x07000013 [M 0x1d3560]. The MISSION ends on 0700013: case 6 →
LevelToEndTo 0x0730013 [M 0x182c60:107-110]. 07000004 = the mission end directly (0x0730004 [M
:89-90]). Both outros' visible halves: the PSS video + the SP_Create'd SUBTITLE script [M
0x1d6360's script switch] — the scripts are timed Text tracks [D 07000500.bin: 0x0600095d =
8 entries, length 2695; 0x06000968 = 4 entries, 2292]. The root cause of "the cut-scene doesn't
play": the slim image's PSS are 0-byte stubs [D] and the game skips a failed open (BGFMVPlaying
stays 0 [M 0x1081a8]) — the port matched the game but played nothing. PORTED: the subtitle track
(outro_script_for = 0x1d6360's 14 arms; subtitle_track reads 07000500.bin's type-7 script), the
FE's ST_MOVIE tick runs the track beside the movie (the active label/time exposed for the draw),
and when the PSS can't start the hold runs the track's length [S — the stand-in; the game on a
stub pops at once]. Tests: mission_outros_live (3 — the track data, the script start, the hold,
the resolve → the results; the 4a chain + the no-0x4a-writer pin). Adjacent green:
reward_after_outro, doublecross_transitions_live, fix_mission_live, walkreal_07000004_live.

### mission-outros-2 — the outro hold on the REAL disc (2026-10-26 ~01:00 UTC)
Branch start: s/mission-outros-2 = origin/s/mission-outros + origin/master (already contained).
Goal: on the full ISO (07300004.PSS / 07300013.PSS present) the two outro tests end at 1 tick —
state 6's movie hold pops at once in the headless path. Find why the hold ends immediately with
a real movie (decoder not driven per tick in the harness? psiMovieFinished read as finished
before the first frame [M 0x1082e8]?) and make the hold last the movie's real length
(BGFMVPlaying == 0) with the subtitle track running beside it [M 0x1d6360:101-112]; tests assert
the real-movie hold with the disc present, the no-movie stand-in only when the file is absent.

### mission-outros-2 — the state-6 hold is the movie's real length (2026-10-26 ~02:30 UTC)
ROOT CAUSE of "the outro ends after 1 tick with a real movie" (headless, full ISO): the test
binaries build WITHOUT the `movies` feature (opt-in), and `Player::open`'s no-feature path
returned `Ok` with `ended: true` — finished before the first frame — for a file that OPENED fine.
The game: a successful open sets BGFMVPlaying = 1 [M 0x1081a8] and clears it only when the decode
pump finishes the file [M 0x108060]; psiMovieFinished is BGFMVPlaying == 0 [M 0x1082e8]. A stub
open failure is the only pop-at-once case [M 0x1081a8's strFileOpen arm]. FIX (movie.rs): the
player carries the movie's real length in 60 Hz ticks — the PSS's audio-track duration [D
golden/pss_movies.tsv: 07300004.PSS = 2 384 128 PCM samples at 48 kHz = 2980 ticks; 07300013.PSS
= 2 040 064 = 2550] — and when no decoder runs (no `movies` feature, declined worker) holds that
long (`headless_advance`, driven once per frame by the host's BGFMVPlaying polls: the glue, the
intro poll, the attract poll). A DECLINED start (not whitelisted / file absent / 0-byte stub)
still reports finished at once — the game's un-started movie. The no-movie stand-in hold (the
subtitle track's length [S]) now applies ONLY to the stub/absent case. Tests assert the real
hold when the image has the PSS (measured off the image at run time) and the stand-in when it
does not. VERIFIED: mission_outros_live 3/3 green (stub image here), adjacent green set
(reward_after_outro, doublecross_transitions_live, fix_mission_live, walkreal_07000004_live,
movie_playback_flow, gameflow_stack_sp_run, front_end_chain_live, menu_audio_flows) all ok;
full suite on the final tree: 308/737 binaries completed with results 940 passed / 2 failed —
the 2 were fix_0c_vent2_live under 4-way-parallel OOM pressure and pass in isolation; the rest
of the window was cut by the sandbox (OOM at -P4, wall time at -P2). The real-movie path itself
is not exercisable in this sandbox (the slim image's PSS are 0-byte stubs [D]) — the two new
hold assertions fire on an image with the retail movies.

### mission-outros-3 — the suite-runner failure is the unfixed tree; the fix proven on an injected real-movie image (2026-10-26 ~06:00 UTC)
Branch start: s/mission-outros-3 = origin/s/mission-outros-2 + origin/master (merge clean).
The driver's suite failure ("the outro stayed up through the track's entries ... got 1", both
outro tests, full disc) is the OLD `Player::open` state, not a runner-env difference: the tree
that fails carries mission_outros_live's tests WITHOUT the mission-outros-2 fix — only
s/mission-outros and s/mission-outros-2 contain the test file, and s/mission-outros's
`Player::open` still returns `Ok` with `ended: true` for an OPENED file in the no-`movies`
build ([M origin/s/mission-outros movie.rs:234]; master has neither test nor fix). With real
PSS bytes the start SUCCEEDS [M 0x1081a8's strFileOpen arm], the glue reports BGFMVPlaying = 1
for one poll, the next poll sees `finished()` and state 6 pops — movie_ticks = 1. NF_SILENT,
the cwd and the runner's wall-clock are not in the hold path at all: the no-`movies` player
reads only the ISO (NF_ISO) and its audio-track duration.
PROOF SETUP for the real-movie path (this sandbox's image stubs every PSS [D ISO tree]):
scripts/inject_movies.py builds minimal valid PSS files whose PCM audio tracks carry the
retail durations [D golden/pss_movies.tsv: 07300004 = 2 384 128 samples = 2980 ticks;
07300013 = 2 040 064 = 2550], appends them to an ISO copy and rewrites their ISO9660 directory
records → Nightfire-movies-test.iso. The outro tests then take the REAL-movie branch here.

### mission-outros-3 — results (2026-10-26 ~07:12 UTC)
The A/B proof on the injected real-movie image (scripts/inject_movies.py — minimal valid PSS
carrying the retail PCM durations [D golden/pss_movies.tsv], appended to an ISO copy, directory
records rewritten): OLD tree (s/mission-outros, no-fix player `ended: true` [M movie.rs:234])
under the suite shape (NF_SILENT=1, per-package cwd) fails BOTH outro tests with the driver's
exact "got 1"; THIS branch passes 3/3 with the holds matching the injected PSS lengths
(2980/2550 ± 2, no stub fallback). Both invocation shapes green on the injected image:
suite.py --only with NF_SILENT=1 and cargo test without NF_SILENT. No code change needed —
the mission-outros-2 hold already depends only on the movie's real length; the driver's suite
failure was the unfixed tree. Full suite on the final tree (master 2757d099 merged, 818
binaries, chunk runner — this sandbox kills background jobs): 81/818 recorded, 401 passed /
0 failed / 11 ignored, no FAILED/TIMEOUT; targeted 11-binary outro/movie sweep 34/34. Report:
docs/reports/mission-outros-3.md.

### mission-outros-4 — branch start (2026-10-26 ~08:47 UTC)
Branch start: s/mission-outros-4 = origin/s/mission-outros-3 + origin/master (merge clean).
Task: the outro hold must survive the WORKSPACE build, where the `movies` feature unifies ON
and the glue prints `movie: decoding 07300004 ahead of state 6`; both mission_outros_live
tests fail at the "the outro resolved" assert because the decoder thread finishes on
wall-clock time, not per poll. Plan: make the state-6 "finished" decision tick-driven (the
host's movie poll count, the PSS audio length in ticks, per round 2) on BOTH builds, keep the
decoded picture path for the real app. Evidence to cite: [M 0x108060, 0x1082e8].
