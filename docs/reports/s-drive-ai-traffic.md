# s/drive-ai-traffic — the driving traffic AI checked against DRIVING.ELF

No change to the shared random stream: the junction picks draw from a new `DriveRng`, the
driving engine's own generator (FUN_0017dc28), which nothing on foot uses.

## What a player would notice

Civilian cars in the driving missions now drive the way the game's do: they cruise at the
traffic speed 7.5 (the old port used each road piece's speed field, which is 0 on more than
half of Paris's pieces, so most cars sat still), keep to their lane, steer toward a point a
little way down the road instead of sitting on the centre line, and turn at junctions by the
game's rules instead of stopping at the first node.

## What was wrong and is fixed (nf-game `driving/ai.rs`, `driving/road.rs`)

- FUN_00133ab0 (drive control): steering used the facing dot (full lock on a straight road);
  the game steers by the side row, locks fully when the aim is behind, and pushes the throttle
  when the car is stopped or rolling backwards.
- FUN_0012e808 (target speed): live traffic (profile 0xc) targets the constant 7.5; the
  segment's speed field is read only by profile 0xf.
- FUN_00224e00 / FUN_00130140 / FUN_00132f80: the game moves a carrot point along the road
  (lane-offset, leftover distance carried across nodes) and keeps it `max(|v|·0.35, box + 7.5)`
  ahead; the car is a free body. The old port moved the car itself on rails.
- FUN_00223f98: the kind-1 (traffic) next-piece chooser is ported whole; the old walk was the
  pedestrian-kind function's shape plus a car-group filter neither function has.
- Added: FUN_002257c0 join, FUN_002267d8 lane points (rs+0x20/+0x30 side rows), FUN_00221e98
  road class, the lane tables from .data 0x33a618/0x33a678 (checked against the disc's ELF),
  FUN_0013c5e8 spawn lane/direction, FUN_00130508's gated stop.

The full per-function table (function, address, verdict, fix) is docs/systems/driving-ai.md §9.

## Tests

- `nf-game --test driving_ai_traffic` (new, 16): drive-control numbers incl. the steer row, the
  aim-behind lock and the throttle-when-stopped branch; target speeds; lookahead; lane tables;
  spawn lanes; join; leftover carry; dead end; plain link with no draw; the 0.3 junction draws
  (count pinned); the 3-way slot rules; the generator; a car round an L corner (stays within 6
  units of its lane line, cruises 6.5–8.5); standby; ISO-gated: lane tables + generator constant
  read from the disc's DRIVING.ELF, node ids, unit side rows on all 7 road tracks, and a traffic
  car on every 7th Paris piece runs 10 s with no chooser error (84 of 110 move more than 20
  units). All pass.
- `nf-game --test driving_ai_follow` (5) and `--test driving_ai_physics` (2): pass. The old
  rails-model tests that pinned steer = 1 on the nose and target = the segment limit were
  removed; their pinned values are the ones the new tests contradict (that is the old/new proof).
- `nf-view --test driving_controls_live`: builds and passes (the viewer's `AiDriver::on_segment`
  / `step` calls are unchanged).

## Unproven / not done

- Physics stand-ins in `AiDriver` [S]: the port's `Car::step` has no tyre lateral force and lets
  a brake reverse the car, so the driver turns the velocity onto the heading and stops a
  brake-made reverse. The side-row sign is paired with `Car::step`'s steer sign; the game's own
  row-0 / SetSteer sign was not read.
- The viewer gives AI cars the player car's parameters (vanquish, MAXSTEERING 0.09); civilian
  cars ship their own `.atr` (hatchback/sedan2/suv1: MAXSTEERING 0.2, MAXACC 13.29). With the
  vanquish numbers a car overshoots tight corners; the fix is in nf-view (not touched here).
- Not ported: spawn pick FUN_0013ca60, respawn FUN_00132808, route bookkeeping FUN_00132d80,
  lane blend FUN_00227058, curved pieces (evaluated straight), the box extent (floor 0.001
  used), and all hostile behaviour: pursuit/ramming/avoidance (FUN_00131770, 0x133f60,
  0x135f70, 0x137310, 0x1377c8) and weapons (FUN_00130050 family). The port had no code for
  these before and has none now.
- The generator's stream position at a traffic pick is unproven (every driving system draws
  from it).
- Live check worth doing: in Paris, break on FUN_0012e808 and read +0x84 / +0x8c of a moving
  civilian car (expect 0xc / 7.5).
