## level-05-cable-full-length (2026-11-01, s/level-05-cable-full-length)

Branch from origin/master (e79794da). Task: 05 — Bond stops short on the castle zip cable (07000005);
he should traverse more of it. Find the game's cable extent and stop rule [D the cable placement's
end points; M the cable/hang class's end test, Player state 0xB], fix the port so the hang-move
reaches the game's ends, and pin the reachable span both ways. Run the cable binaries,
walkreal_07000005_live and nf-game; quote the totals.

## Progress (2026-11-01, ~02:20 UTC)

- Branch created from origin/master (e79794da) and pushed immediately; fresh sandbox, cloning the
  game files next.

## Outcome (2026-11-01, ~03:20 UTC)

- THE RIDE EXTENT WAS NEVER SHORT — the premise dissolves at the data level: the game's own travel
  segment is the wire object's world-sphere span (radius − 0.5) [M Player_Wire 0x1976dc-0x197730;
  L obj+0x8C = 4.0390439 @ee_start_05.bin 016dc48c, obj base 016dc400] = z 2.724..9.802 on
  Wire_Cable #251, and the port already rides exactly that (stall 0.244/0.016 inside the stop band
  at the two ends, never beyond, both ways — cable_span_live through the real jump-grab path).
  The drawn cable is OBJECT_CableEnds [D #252: z −1.070..14.068, 15.14 u at the rope's height] —
  2.1× the rideable span, so the game itself stops Bond part-way along the VISIBLE cable; the
  stop rule [M 0x1977c0-0x197834: the strafe kill at squared-2D 0.5 of the travelled-to end + the
  beyond-end snap 0x1976f0-0x197708] leaves a reachable band (end − 0.707 … end] per end.
- New test crates/nf-view/tests/cable_span_live.rs (3 tests): the parsed geometry vs [L]
  (centre/yaw/radius/type/params), the disc cable-run placements [D], the both-ways span + the
  never-beyond clamps [M]. No sim change needed (the sway-lane [S] note went to the atlas only).
- Run: walkreal_07000005_live 11/0; the cable binaries wire_live 2/0/2i, fix_wire2_live 3/0,
  cable_span_live 3/0, walkreal_probe0c_wires_live 1/0, wire_repro 0/0/1i; nf-game 959/0/9i;
  nf-formats 108/3 (the 3 disc_golden slim-ISO allowed); nf-audio 28/0. The full fast-runner pass
  did NOT fit this sandbox (background jobs are reaped between tool calls and the 9.9G disk filled
  at ~86% of the nf-view test links — nf-view ran as the ~45 binaries that had linked, see the
  report).

## Continuation (2026-11-01, ~07:15 UTC)

- Merged origin/master (babf9247 → merge e53297fe; nf-game src +2093 lines) and re-ran the suite's
  run phase on the final tree with a resumable driver (same env/flags as tools/test/suite.py —
  NF_SILENT=1, --test-threads=1, 600 s/binary; the sandbox reaps even setsid'd background jobs, so
  the runner's single process can't survive a tool-call boundary). 454 of the 1039 test binaries
  ran in the window: 2112 passed, 6 failed, 74 ignored — nf-game 98 bins 993/0/9i; nf-formats 41
  bins 194/4/3i; nf-audio 4 bins 28/0; nf-view 311 bins 897/2/62i. Cable set all green
  (cable_span_live 3/0, wire_live 2/0/2i, fix_wire2_live 3/0, walkreal_probe0c_wires_live 1/0,
  wire_repro 0/0/1i, walkreal_07000005_live 11/0).
- The 6 failures are all environment, none gameplay: disc_golden 3 (the documented slim-ISO
  allowance), identity_disc 1 (asserts the full image's 1,191,072 sectors; the slim on-foot ISO is
  518,435 — the test is right for the retail disc), fix_0c_pylon_live 2 (wgpu adapter NotFound —
  this sandbox has no GPU; the render spots go to the main agent). 585 nf-view binaries did not
  fit the window; nothing gameplay-shaped was left unfixed because the task needed no sim change.
