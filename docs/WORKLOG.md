# WORKLOG

### level-14-mezzanine-descent (start, 23:33 UTC, 2026-10-29)

Task: Countdown Part 1 (07000014) — the walk from the vent scene down into the facility. The
vent cut-scene 0x06000833 now hands Bond back where he stood [M 0x1d94c0 case 6; L PCSX2 fresh
07000014: touch #301 at (40.7, 3.1, 39.0), stands at (40.68, 2.76, 38.99)]. The old route of
`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` rode the removed teleport;
the mezzanine descent wedges at (32.06, 2.70, 39.4) — the wall at x 31.5, z 38.4..40.8, and the
floor edge south of z 38.4. Find the route a player takes from that spot into the facility
[D the collision and placements around the mezzanine: stairs, ladders, drops, doors, vents;
M any use/jump path], fix any port collision gap, walk it on real input, un-ignore the test
with the rest of the walk green. FULL suite; quote its total line.

Base: origin/master (48696227). No prior work on this branch (continuation message; the branch
had zero commits — this entry is the first). Suite runner launched at 23:31 UTC
(tools/test/suite.py, suite14.out).

Plan: read the level-14-skip-handback report + the walkreal ignore reason; map the mezzanine
collision [D] + any use/jump path [M]; port collision fixes with game addresses; walk on real
input; un-ignore; full suite on the final tree.

### level-14-mezzanine-descent (mid, 00:45 UTC)

The hand-back spot is on the mezzanine PAD (floor 2.15, x 32..40+ at z 38.5..39.5). Route
findings this window (all real-input walks through Walker unless marked [D]):

- The pad walks WEST to (34.63, 2.80, 37.94) and SOUTH to (34.57, 2.80, 37.18), then down a
  0.43 step to a landing at floor ~1.72 — (33.09, 2.37, 35.38) grounded. The mezzanine's west
  face wall at x 31.5 (z 38.4..40.8) is NOT the route; the pad's south-west protrusion
  (x 33.85..35.2, z 36.0..37.2 at 2.15 [D rays]) + the 1.72 landing are walkable.
- The door at the landing is plc #834 "OBJECT_Door" (32.01, 2.14, 35.81) [D] = sim door #53:
  kind 2, group 0, ch_open 0, ch_lock 0 — UNLOCKED [D params via Doors::create]. Leaf tri box
  x 32.00..32.65, y 1.76..2.52, z 35.72..35.82 [D]; it sits over the 1.72 landing next to the
  tall fusebox column #833 (x 31.93..32.65, y 0.60..4.60 [D]) and the "Fusebox_Sign(Cameras)"
  #835. The walker's E-press logged "opening door" but the through-walk still wedged — the
  doorway's usable opening needs a dedicated probe.
- The NOTCH west of the landing (x 31.2..33.75, z 36.0..38.3) is a BOTTOMLESS void in the
  port's static collision (flat `world.segment` rays NaN past −9) — but the GAME's own nav web
  has NO link crossing it either: path 0's corridor nodes 57/58/263/257/260 (x 27.3..28.3)
  never link east to the machine room's node 48 (38.72, 0.0, 36.14) / 49 / 47 [D nav links].
  The AI never walks the notch — the descent almost certainly does not cross it, and the
  "missing floor" there is likely correct void (a shaft), not a port gap.
- The EAST LANE is the live descent lead: the pad walks EAST to (44.05, 3.01, 39.13) and SOUTH
  to (45.13, 3.05, 43.58) — the lane (floor ~2.35..2.40) continues to z 43.6. [D rays] show the
  lane floor 2.38..2.40 at x 40.9..41.5 z 42.8 and x 39.5..41.0 z 44.0..44.5, with HOLES down
  to the machine room floor −0.60 at (42.5, 42.5), (44.5, 43.5), (43.0, 44.5). The grapple
  point #331 (40.78, 4.79, 42.86) + "3RD_Person_Icon" #330 (40.79, 0.41, 42.89) [D placements]
  mark the machine-room spot under the lane — the game's own climb-back-up marker. The walker
  did NOT find the drop (stuck at x 45.13 steering west-southwest, 3 × 6000-tick caps) — the
  exact walk-off edge/hole is the next session's first probe.
- Below: the machine room floor (−0.60..0.0) walks WEST only to x ~37.1 (root 0.05 at
  (37.15, 0.05, 42.48) grounded) — matching patrol path 1 node 0 (37.44, 0.0, 42.81), the
  patrol's west turn [D nav]. The machine-room → corridor connection is still unfound; next
  probes: the floor UNDER the pad (z 37..42, x 31..40 — never rayed from below) and the lane's
  south end (z 44.5+).

Suite: relaunched 00:44 UTC with the full env (the 23:31 launch died at the alsa-sys build —
the relaunch shell lacked PKG_CONFIG_PATH; my error). No repo code changed this window, so the
tree is origin/master's merged state (48696227).
