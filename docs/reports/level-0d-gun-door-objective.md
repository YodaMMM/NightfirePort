# level-0d-gun-door-objective — the Chain Reaction deck laser kills its objective door

Branch `s/level-0d-gun-door-objective` (from `origin/master` @ 1d2651c0). Task: on Chain
Reaction Part 2 (0700000d) the deck-mounted laser gun (GunImp, placement #225) aims, draws and
sounds right, but its effect on the objective door was missing — find what the game's gun shots
do to that door/target and port it, so firing the gun at the target does what it does on the
PS2 (opens/breaks it, raises its channel, advances the objective).

## What the game does (the mechanism, all [M] unless tagged)

The fire tick of the 0x67 laser (`TurretGuns[0xb]`, the SP entry [M GunImp_Create 0x131f20:52-63])
does two damaging things in `GunImp_Update` 0x1320d8:

1. **The round** — `Bullet_init(weapon_data + 0x67·0x10c, muzzle, dir, 0xffff)` [:327-331]. The
   port already flew it: the player-bullet flight's class-5 contacts drain the C5 `Destroy`
   leaves through `Destroy_CollisionHandler` 0x12bd48:31-35 (health −= HITDATA+8 per contact;
   the 0x67 check at :32-33 only dims the leaf's +0x104/+0x105 colour bytes by 5) and the death
   writes the leaf's channel. The laser round deals 50 a contact [L ee_start_05.bin
   weapon_data + 0x67·0x10c + 0x0c = 50], so bigdoor02 #96 (health 10000 [D placements p3])
   needs 201 contact ticks.
2. **The beam hit** — the same tick the arm casts the 1000-unit
   `Collide_RayIntersect(muzzle, muzzle + dir·1000, cel, the imp obj, 0, &list, 0, 8)`
   [:372-384], the beam's far point **becomes the hit** [:385-387 — the hitdata position
   replaces the raw tail; the miss keeps the tail], and for the 0x67 arm the hit position feeds
   `Explode_Propagate(4.0, 5.0, hit, cel, the imp obj)` [:433-436 — args 0x40800000/0x40a00000,
   the hitdata+0x40 position, the attacker is the imp object]. `Explode_Propagate` 0x17c518's
   case 0x21 (the `Destroy` class — `Destroy_Create` 0x12c318:22 sets obj+0xff = 0x21)
   **instantly smashes** a destroyable the sweep reaches with ≥ 1.0 damage [:141-177 — the
   falloff `5.0·(1 − (int)(dist − obj+0x8c)/4.0)`, the +0x14 & 0x20 mask escape] and
   `Destroy_Smash` 0x12bb60:40 writes `switch_channels[+0x12] = 1` — C5's **ch 0x1f**, the
   objective-6 record's complete channel [D placements p0; M 0x1827d8:78-85]. One beam contact
   on the door kills it, health regardless.

The old port called that whole beam arm "presentation" (the `gunimp.rs` doc said so) and drew
the beam to the raw muzzle+dir·1000 tail: firing the mount at C5 left the objective to the
201-contact bullet drain — effectively unreachable in play. **The missing effect was the beam's
`Explode_Propagate` smash.**

## What was ported

- `crates/nf-game/src/drone.rs` — `gunimp_tick` now casts the fire arm's ray (the scoped world
  segment via `segment_bullet_scoped`, plus the destroy leaves' and breakables' own volumes —
  the game's ray hits the door itself; bigdoor02 carries a collision block [D], and the port's
  destroy box is its collision model — nearest stop wins), clamps the beam quad at the hit
  [M :385-387], and for weapon 0x67 queues `(hit, 4.0, 5.0, the imp)` into
  `pending_beam_blasts`. `Drones::run_beam_explodes` drains the queue at the same tick's blast
  slot (the drone arm needs the tick's `Data`), running the shared [M]-faithful
  `explode::propagate` (the integer-truncated falloff, the ≥ 0.0002 gate, the ≥ 1.0 smash gate)
  and applying the arms the port models: case 0x21 the destroyable smash, case 0x20 the
  breakable `Break_ApplyDamage` [M 0x17c518:163-167], case 2 the drone `Drone_ExplosiveHit`
  msg 9 [M :112-123, the spawn_blast shape], case 3 `Player_Hurt` [M :125-140, the
  pain_generic arm]. The attacker is the imp (`usize::MAX − 1` — the port's `usize::MAX` is the
  player-credit mark; a guard hit by the beam does not credit Bond [M :435 the attacker is
  GunImp_Update's own obj]). The game's remaining cases (0 copter, 5 model destructibles, 0x28
  vehicle, 0x35 MP, 0x36 GT, 0x37 sensor, 0x38 monitor, 0x3d sub, 0x4b hit-list) stay unwired
  in this path — none is reachable from the 0d mount's beam [D census: no copter/GT/sensor/
  monitor/vehicle on the beam's line; MP is SP-only] — listed as left work.
- `crates/nf-game/src/props.rs` — `Props::smash` added: the `Destroy_Smash` 0x12bb60 death half
  (the dead mark obj+0xff = 1 [M :33/:41], the channel write [M :40], the type-script blast
  tuple [M :58-76 — the port keeps its established radius/damage stand-ins, documented at the
  old bullet_hit note]). `bullet_hit`'s kill now routes through it (one code path for the
  channel write + blast).
- `crates/nf-view/src/scene.rs` — the `gunimp_tick` call passes the world (the ray's input).
- `crates/nf-game/src/gunimp.rs` — doc corrections only: the beam ray is damage, not
  presentation; the `TurretGuns` interval field counts down 1.0/tick — ticks, not seconds
  [M 0x1320d8:107-112,:324].
- `crates/nf-view/tests/0700000d_gunimp_beam_axis_live.rs` — the old `len > 900` pin
  ("the beam reaches the 1000-unit ray tail") contradicted [M :385-387] for every ray that
  hits: the far point is the HIT there, the tail only on a miss. Corrected to the ≤ 1000
  clamp (the muzzle-origin and barrel-axis pins untouched). **The test was wrong, not the
  decompile.** The clamp is pinned positively by the new door test.

## Verification

- New `crates/nf-view/tests/0700000d_gunimp_door_live.rs` (real input: the ST-3 plate walk for
  the objective's reveal channel 0x1d [D #378], the mount, the aim driven by the player's own
  `Input` deltas, the trigger held):
  - `the_beam_hit_smashes_the_c5_door_and_raises_ch_1f` — bigdoor02 dies in **4 fired ticks**
    with health untouched at 10000 (the instant smash; the pre-fix tree needed 201 contact
    ticks), ch 0x1f on, `mission.states[12][2] == 4` (objective 6 completes).
  - `the_beam_stops_at_the_surface_it_hits` — the beam far point lands on the door
    (z −269.95 vs the door plane −269.5); the old raw tail overshot to z 673.
- Regression families run (all green): the gunimp live family
  (`beam_axis` 3, `beam_lifetime` 1, `fire_sound` 1, `rate` 2, `aim` 1, `gunimp_mounted` 2,
  `gunimp_gun_draw` 1), `cargo test -p nf-game --lib` 578/0, and the new door pair. The full
  fast suite (`tools/test/suite.py`) runs on this branch — the total line is at the bottom.
- The fail-first reproduction: on the pre-fix tree the same harness needs ~212 ticks for the
  door (the bullet drain) and the beam overshot the door — both new tests fail there.

## Left

- The unwired `Explode_Propagate` cases in the beam path (copter/model/vehicle/MP/GT/sensor/
  monitor/sub/hit-list) — [M]-listed, unreachable from the 0d beam; wire them when a level
  needs them.
- bigdoor01 #93 carries no gfx bounds and an empty collision block in the game's own data
  [D; docs/levels/0700000d.md §6] — neither the game's ray nor the port's can hit it; the
  objective completes through #96's death (both leaves carry ch 0x1f [D]).
- The 0x200026e impact-flare draw at the hit (hit − dir·0.2 [M :407-432]) — presentation, not
  wired (the beam quad itself is).
- The smash blast's radius/damage stand-ins (the port doesn't parse the placed obj's +0x8c
  sphere; p5·0.01 vs the game's kind·10) are pre-existing, documented at `props.rs`, open live
  check — unchanged by this task.

## Live checks for the main agent

1. **The beam kill on a fresh boot** (0700000d, after the ST-3 yard): mount #225, aim at the
   C5 door, hold fire. Watch `Destroy_Smash` 0x12bb60 (or a write watch on
   `switch_channels[0x1f]`): expect the write the tick the beam first touches the door
   (the fire rate is ~a round a tick [M :324 + :107-112], heat 2.0/shot → the overheat byte at
   ~100 continuous shots [M :292-298]) — NOT after ~200 separate contacts.
2. **The beam length**: `PositionBeam` 0x1320d8:394-407's second arg — expect the ray hit
   (on the door plane z ≈ −269.4 from the mount at (137.27, 9.33, −324.66)), not
   muzzle + dir·1000.
3. **The sweep's reach** (only if convenient): a watch on `Destroy_Smash`'s caller from
   `Explode_Propagate` 0x17c518 case 0x21 (:171) while the beam holds on the door — confirms
   the propagate path (vs only the collision-handler path) writes ch 0x1f.

## Render spots for the main agent (the beam clamp is visible)

- `--bond 137.27,9.33,-323.0,3.14159,0.10` mounted at #225 firing at C5: the beam quad should
  END at the C5 door surface (~55 units out, z ≈ −269.5) — not poke through into the corridor.
- `--bond 137.27,9.33,-323.0,0.0,0.0` the mount's default aim (+z, into the station): the beam
  ends at the first station geometry, wherever that is — never a 1000-unit tail through walls.

## Usage

not visible to me — this session exposes no token, cost or context-window figures.

## Suite total (the fast runner, this branch)

(see the line below — filled in from `tail -1 suite.out` before the finish steps)
