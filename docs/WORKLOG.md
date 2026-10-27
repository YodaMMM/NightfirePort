### level-0d-gun-door-objective — start (2026-10-27 ~07:05 UTC)

Branch `s/level-0d-gun-door-objective` created from `origin/master` (e5289b21 is the
branch tip of master at creation; master = origin/master @ 925bee6c+). Task: on Chain
Reaction Part 2 (0700000d) the deck-mounted laser gun (GunImp, placement #225) aims,
draws and sounds right, but its effect on the objective door it is meant for is
missing. Find what the game's gun shots do to that door/target [M GunImp_Update
0x1320d8's round spawn :327-331 and the round's hit path, the target's
damage/breakable/channel handling, the objective it advances] and port it, so firing
the gun at the target does what it does on the PS2 (opens/breaks it, raises its
channel, advances the objective). Real-input test: mount, aim, fire, the target
responds and the objective advances [M].

Plan:
- Start the fast suite runner in the background in the first minutes.
- Read the decompile: GunImp_Update 0x1320d8 (round spawn :327-331), the round's hit
  path, the target's damage/breakable/channel handling, the objective consumer.
- Reproduce through the real input path (mount, aim, fire) on 0700000d.
- Port what the game does; cite addresses for every pinned behaviour.
