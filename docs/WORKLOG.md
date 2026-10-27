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

## level-0d-gun-door-objective — the beam is the door's killer (2026-10-27 ~08:00 UTC)

- [M GunImp_Update 0x1320d8:372-384] the fire arm casts the 1000-unit `Collide_RayIntersect`
  along the barrel; [:385-387] the beam's far point becomes the hit; [:433-436] the 0x67 arm
  runs `Explode_Propagate(4.0, 5.0, the hit, cel, the imp obj)` at it. The old port called that
  sweep presentation — it is DAMAGE: case 0x21 [M 0x17c518:141-177] instantly smashes a
  destroyable the sweep reaches with ≥ 1.0 damage and `Destroy_Smash` 0x12bb60:40 writes
  `switch_channels[+0x12]` — C5's ch 0x1f, objective 6's complete channel.
- Ported: `gunimp_tick` casts the ray (the scoped world segment + the destroy/break volumes —
  the game's ray hits the door itself [D bigdoor02's collision block]), clamps the beam at the
  hit, and queues the sweep; `Drones::run_beam_explodes` drains it at the same tick's blast slot
  (the shared [M]-faithful `explode::propagate`): destroys smashed, breaks
  `Break_ApplyDamage`-ed, drones msg-9 EXPLOSION, Bond `Player_Hurt`. Attacker = the imp
  (usize::MAX−1 — never the player-credit mark).
- `Props::smash` added (the Destroy_Smash death half: dead mark, the channel write, the blast
  tuple); `bullet_hit` now routes its kill through it.
- The beam >900-tail pin in 0700000d_gunimp_beam_axis_live contradicted [M :385-387] for
  hitting rays — corrected to the ≤1000 clamp (the door test pins the hit clamp positively).
- New tests (0700000d_gunimp_door_live): the beam hit kills C5 in 4 fired ticks with health
  untouched (the instant smash — the 50-a-contact drain alone needs 201 ticks) + ch 0x1f +
  objective 6 completes; the beam stops at the door surface (the old raw far overshot to z 673).
- The interval comment fix: +0x14 counts down 1.0/tick — ticks, not seconds
  [M 0x1320d8:107-112,:324].
