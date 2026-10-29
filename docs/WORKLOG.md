# WORKLOG

## level-16-escape-real (started 23:09 UTC 2026-10-29)

Task: Countdown Part 3 (07000016) — find what the shipped game really does after the cinema
re-enables Bond at the pit floor (3.57, -23.18, -179.55) inside HurtPoly-ShuttleLaunch #309's
stack: the jetpack/escape mechanism a player uses [M substates/scripts/triggers; D placements;
L fresh-boot PCSX2 if needed]. Port it and walk the escape on real input in
walkreal_07000016_live / fix_finale_live / kiko_button_real_walk_live — no scan-mode toggle,
no debug fly control. FULL suite + total line. Branch `s/level-16-escape-real`, from
origin/master 48696227.

Progress log:
- 23:07 setup checked (repo, work/, ISO, nf-queue all present; level-05 scratch stashed);
  branch created and pushed with this entry.
- 23:10 fast suite runner launched in the background (suite.out; the sandbox killed it between
  tool calls again — the known behaviour; no chunk window survived the research round).
- 23:12-00:15 the mechanism hunt, all negative on the fly family: the substate dispatch [M
  Player_Update 0x193600] enumerates 0 walk/1 climb/2 grapple/3 ride-swim/4 crouch/5 scan-fly/
  6 wire/7 creep/8-9 zero-g/0xf zipline; SS 5's only entries are the action-39 toggle and the
  CHCHFLY debug control mode, SS 8/9's the CHCHZEROG debug mode [M 0x00215c70/0x00216388] — no
  player-reachable fly. [D probes wires16/sps16/pitgeo16/lent16]: NO wire/grapple placement on
  the whole level, ONE ladder at (−0.4,−6.2,−86.2) mid-level, the pit holds the lift SP#347
  (out 80, mask 1, at (2.48,0,−159.93), script 060009e9 keys bay 1.60 → pit −21.22 over 549
  ticks), the blast doors 02000994-7 (keys open at 1350), the dynobj blockers #532 gated on
  204/205 (@2249/2803), and the launch script's absolute-wait timeline: pulse 1 = ch 243
  1611..2170, ch 80 (the lift summon) @2240 — post-pulse (the old ignore text's "630 into the
  pulse" was right for the wrong clock).
- 00:17 THE ESCAPE FOUND IN THE PORT'S OWN PHYSICS (example escape16): the pit's WEST rim
  structure (the blast-room shoulder, x −8..−16, z −167..−173, top ~−21.7) is climbable on real
  input (run + jump pulses), and standing at (−15.2,−21.85,−167.0) survives the WHOLE pulse with
  hp 100 — the hurt contact is TRI-based, not a volume y-test, so the rim shoulder is outside
  every band's reach. The east/no-jump attempt died at the pulse (the control).
- 00:41 walkreal_07000016_live::the_pit_escape_climbs_the_west_rim_and_survives_the_pulse_on_real_input
  GREEN (6.5 s): the full walked approach (the arrival test's 42 legs) → the slab → the cinema
  plays out → the hand-back (2.85,−23.18,−177.83) → the real-input climb (atan2(dx,dz) steer +
  jump pulses; the port's yaw forward is (sin,cos)) → the ledge (−15.19,−21.95,−167.01) → the
  559-tick pulse survived (hp 100 at rise and fall, top_up_health off) → the chaser spawns
  (2.39,−22.04,−149.17). No teleport/poke/scan-mode/fly anywhere in the leg.
- 00:47 the report + the finish steps; the FULL suite did not run this window (the runner was
  sandbox-killed twice; the touched binary is green — the next session resumes the chunks).
