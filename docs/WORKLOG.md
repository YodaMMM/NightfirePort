# WORKLOG

## level-41-exit-event-2
Started 2026-11-03 06:20 UTC. Branch s/level-41-exit-event-2 cut from origin/master 6f16146e.
Task: on master, `interactables_07000041_live::the_wrapper_ends_into_the_space_station_after_the_chain`
is red since level-41-exit-event landed (the 241-tick wrapper-end stand-in removed, the 0x18→0x1b
resolve added). Find how the game's 41 exit event and the wrapper's end-into-the-space-station
chain coexist [D the 41 scripts/channels; M the level-exit / LOAD_LEVEL path], fix the port, keep
the wrapper test green without weakening an assertion. Run every *07000041* binary, the
campaign/hand-off binaries and nf-game.

Work log:
- (06:20) branch start, WORKLOG entry. Next: fast runner in the background, then the red test.
- (06:25) the reds confirmed on the re-land: interactables 16/2 — exactly the two wrapper-end pins
  (line 502 + the chain-trigger leg-1 line 569). The history found: c691219e merged the exit-event
  work, b5780dc3 reverted it (stand-in back, resolve fix out, the disc-data test deleted) — the
  re-land = revert of the revert, clean.
- (06:40) the research: Trigger_Activate 0x1b2318 read whole — kind 0xa's exit tail (Music_Event(3,1),
  GS_PauseGame(1), ResetMap_LevelToLoad(ext+0x20,1,0), PushState(300,7), kind:=99, RamSave) and kind
  0xd's movie arm (GameState+0x10 := movie code, push 6+7, no load). GetLevelToLoadFromMovieCode
  0x1d3560's fallback = GameState+8 & 0xff00ffff; ResetMap_Load 0x1d6360 phase 0's 0x18→0x1b special
  case re-read. GameFlow_Main 0x1c98c8: no case 7 in the C (the fade arm the decomp hides). Mission
  07000041: no row → inert [D 0x2a4350]. The 41 exit carriers are ALL inert on this disc — the
  finding: the chain plays in-level, the level never self-terminates, the hop is the story layer's.
- (06:44) the fix: the wrapper-end story hand-off re-landed in drone.rs (wrapper_end_timer field +
  the tick_inner arm) with the doc rewritten to the honest story — the window (241 ticks) is a port
  choice [S] bounded by the chain's longest script's length [D 06000027], the script never runs [D
  the SP pins], the target 0x0700001b the story's next base map [L sp_level 0x2df2e0 row 23 —
  re-verified this session; M the Equinox hop 0x1d6360:99-102]. The interactables wrapper-test doc
  corrected the same way (no assertion touched). interactables 18/0, walkreal 7/0, walk 1/0, census
  0/0/1i, probe 1/0, nf-game lib 656/0/5i, wrapper_07000041_end_live 5/0.
- (06:52) campaign/hand-off: campaign_flow_live 5/0 (294 s), probe_campaign06 1/0,
  walkreal_probe03handoff 3/0, handback_sp_path_live 3/0, fix_scores_handoff_live 1/0,
  briefing_plays_live 0/0 (opt-in), mp_debriefing_flow 6/0. Level doc §2/§5 updated (the port models
  the hand-off). Fast runner restarted (the first start died: nohup env had no cargo on PATH).
