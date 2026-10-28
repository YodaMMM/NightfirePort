# WORKLOG

## level-02-skip-handback (2026-10-28, branch s/level-02-skip-handback)

Task: where the game puts Bond when a cut-scene is SKIPPED. Master hands Bond
back from an animated (kind-2) cut-scene at the anim object's matrix
[M Script_EventHandler 0x1d94c0:73-84,:125-134], matching the game when the
scene PLAYS (07000002 intro: (-27.504, 1.131, 32.348) [L]). When the player
SKIPS, the port fast-forwards and lands at the anim's END pose (-27.30, 1.23,
28.90) — unverified. Read the game's skip path [M Script_FFwd 0x1dac78,
SP_Update 0x1dbda8's skip arm, how event 6 and the anim's interp run (or not)
during the fast-forward] and make the port's skip hand-back exactly the
game's, for every SP with a kind-1/kind-2 hand-back. Test both paths on
07000002/03/14 through the real skip input. Full suite; report; finish steps.

Work log:
- Reused the sandbox repo (work/, ISO, nf-queue in place); fetched, created
  s/level-02-skip-handback from origin/master 8a966fa3; WORKLOG + first
  checkpoint pushed 17:36. Fast runner started 17:34 (setsid — plain nohup
  dies between tool calls here).
- Read the game's skip path [M]: SP_Update state 2's skip arm (Input_Action
  (-1, 0x19, 1) → Script_FFwd(si,1), death NISs excluded 0x1dae60); Script_FFwd
  0x1dac78 (arms only with a script camera live — +0xAB5 bit 4 from
  Script_CameraStart 0x1d9140:47; fade = min(length − clock, 45); Music_Event 5;
  Input_ClearAllActions + Text_FlushAllSubtitles on the ScriptCam arm; kind-2
  drones flagged 0x40000); Script_Update 0x1da998's do-while — ticks normally
  while the fade runs, then SPINS the streams/clocks (+1 per pass, events fire
  at their scripted ticks) until the script stops; Script_Interp case 2/case 4
  return while +0xAB5 & 0x20 (the anim/camera freeze at the fade-end pose),
  case 1 has no gate; Script_Run cmd 0x12 → Script_EventHandler case 6
  snapshots the kind-1/2 prev stream's obj+0x90 into +0x40 when it FIRES
  (player-matrix fallback without one); Script_Free 0x1da880:44-48
  Player_Enables Bond at +0x40.
- Verified the port against that chain (no code change needed — the mechanics
  were already exact: fix-dev's ffwd + fix-script-commands' freeze gates):
  new crates/nf-game/tests/nis_skip_handback.rs drives 07000002/03/14's every
  event-6 SP script through the REAL skip input (skip_request → begin_tick →
  SP_Update state 2 → Script::ffwd), play vs early skip (latched t3) vs mid
  skip (t = e6/2). Results: every row matches the [M] rule — an event 6 firing
  at/before freeze (K+fade_len) captures the play pose; after it, the anim's
  FROZEN frame-(K+fade_len−start_clock+start_frame) pose (clamped), which is
  NOT the anim's end pose unless the anim had already clamped. 0x06000084 (the
  intro [L] scene): play (-27.502, 1.162, 32.364) vs the game's
  (-27.504, 1.131, 32.348) [L]; skip@4 (-27.295, 1.218, 28.952) = the frozen
  frame-~50 pose — the task's "-27.30, 1.23, 28.90" symptom is that frozen
  pose, not the anim's end pose; skip@770 (-27.357, 1.128, 32.032).
  Press-invariant rows: kind-1 0x0600062f (-55.495, 1.546, 42.017) and the
  fallbacks. 07000014's 0x06000833 left unasserted ([S] vent-drop arm).
- docs/systems/script-player.md: new "Skip hand-back [M]" section.
- Full suite on the final tree: see the report for the total line
  (identity_disc + 3 disc_golden fail on the slim ISO — environmental, the
  identity test wants the 1,191,072-sector retail image).
