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
