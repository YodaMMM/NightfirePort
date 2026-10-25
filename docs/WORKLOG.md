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
