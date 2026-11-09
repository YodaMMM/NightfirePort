# 07000001: a guard falls through the floor near (-28.7, 27.6)

With solid guard bodies (master 0a9d39435), in one 01 walk variant guard #4 fell through the floor near (-28.7, 27.6) and read y about -1e6. Reproduce through real input around that spot (guards pushed by Bond's body, crowding against the hall walls), log the guard's position, floor contact and state per tick, and find why he loses the floor (a push into geometry, a missing floor snap, a cel change). Compare with the game's drone floor handling (Drone_FeetOnPoint, the bounds push) and fix the port.

Reproduce on the real level first through real input, not a teleport. Branch from origin/master, push to `s/guard-fell-through-floor`, report in docs/reports/guard-fell-through-floor.md. Run every walkreal_* binary if the guard movement changes. No draw changes.

---
claimed-by: sandbox1 at 2026-11-09T19:57Z

---
## Report (blocked, 2026-11-09T20:48Z)


---
## Report from the worker (blocked, 2026-11-09T20:48Z)

# guard-fell-through-floor — report (window 1, BLOCKED: the repro did not fire yet)

Task 07000001: with solid guard bodies (master 0a9d39435), one 01 walk variant had guard #4
fall through the floor near (-28.7, 27.6) and read y about -1e6. This window was spent on setup
(a fresh sandbox: clone, work-slim, ISO, rustup + ALSA user prefix, nf-queue) and recon. The
repro did not reproduce on the two walk variants that fit the window; the tooling, the pinned
facts and the candidate mechanisms are below so the next window can go straight at it.

## Pinned facts

- **Guard #4** is d#4, skin 0x500003a, team 1, spawn (-27.23, **10.22**, 2.28), switch channel
  0xF — the ch-0xF stairwell-ambush pair with d#12 (0x5000036, ground, (-31.31, 6.34, 11.51))
  [L the d2 roster probe after the intro, tick 2400]. He lives on the **upper west floor**
  (y ≈ 9.22-10.22) above the exit corridor.
- **The reported spot (-28.7, 27.6) is VOID at every height** on 07000001 — a down ray from
  y 40 to −20 hits nothing, `find_cel` None [L the probe scan]. The upper west floor spans
  z 0..17; the AI boundary walls close its north edge at z ≈ 16.7-16.75 (walls #94-101 in the
  port's `Network::walls`), and z ≥ 18 has no floor at any x from −34 to −19 [L the push-field
  + floor scan]. (-27.2, 29.8) — 07000002's start — is void on this map too, as expected.
  So a guard who ends up near (-28.7, 27.6) has left the walkable world north of the upper
  floor and falls forever: `Drone_FeetOnPoint`'s ray misses, `Drone_CollisionHandler`
  accumulates `WldGravity` unbounded, y → −1e6.
- `nav.bounds` is true for this level [L probe].

## Mechanisms (game vs port)

- `Drone_FeetOnPoint` 0x140660 [M]: down-ray from obj+0x30 by (height+0.6) with mask 0xC;
  contact when `pos.y − height − hit.y < 0.2`; on contact the game memcpys the 0x60-byte floor
  HITDATA into the AnimObject and sets `anim+0x60 & 8`. The port's `feet_on_point`
  (crates/nf-game/src/drone.rs:6506) mirrors it and already carries the 4-ray 2 cm seam bridge.
- `Drone_CollisionHandler` 0x140968 [M]: the per-tick feet ray + gravity + the 0.125 settle
  (`fVar10 < −0.1` band through the second probe 0x140e90). The port's `collision_handler`
  (drone.rs:6555) ports this; a missed ray → `in_air`, gravity per tick — the −1e6 path.
- The only horizontal constraint on a game guard's move is the **bounds push**
  `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0 [M; the port's `Network::push_out`,
  nav.rs:146]: XZ-only, the y band [Q.y − 0.4, Q.y + lerp(post heights)], pushes the feet OUT
  to r = 0.4 — **away from the wall, on whichever side P already is**: a guard who CROSSES a
  boundary wall in one tick is never pulled back. Walls flagged 0x800 (door) / 0x1000 (kick) /
  0x400 (passable) are skipped by the 0x1c00 mask [M 0x157bd0:94-97, drone+0xba4 = 0x1c00].
  The port applies it per tick after the body pass (drone.rs:22753-22761), gated
  `enabled && bounded && nav.bounds`.
- The bump path [M `NDrone2_DealWithObjHit` 0x13fff0 case 3]: the player's body overlap sets
  drone+0x26, stores the delta, sends msg 0x22 — it does NOT shove the guard; the guard's own
  root motion loses its into-contact component while the latch stands (0x13b490:104-116, the
  port's `objhit_turn_step`). The port's body pass (drone.rs:22730-22751) matches.
- Consequence: for the guard to reach the void he must CROSS the upper floor's north boundary
  (z ≈ 16.7) or leave through a 0x1c00-skipped wall (a door/kick/passable edge) in a single
  tick — the push cannot recover him afterwards. A walking guard (≤ 0.06 u/tick) cannot tunnel
  the 0.4 push; a single-tick displacement (a pane push, a ride, a script move) could.

## Tooling added (this branch)

- `NF_GUARD_FALL_TRACE=1` — additive env-gated walker trace: prints `[guard-fall]` for every
  live team-1 guard seen through the world (y < −20 or a sub-zero fast drop) with tick, skin,
  state, pos, on_surface, bounded; sets `walker::guard_fell()` (the NF_WALK_TRACE pattern, no
  behaviour change).
- `walkreal_07000001_live::no_live_guard_falls_through_the_floor_on_the_walk` — the whole real
  route (no teleport after the load) asserting no live guard ever loses the floor. **This is
  the red test for the fix**: green on today's master runs (below), red the moment the driver's
  variant reproduces.
- A scratch probe (NOT committed; rebuildable in 30 lines per the report): loads 07000001 d2,
  prints the roster, floor scans, the boundary walls and the push field.

## Runs (each its own tool call)

- `walkreal_07000001_live no_live_guard_falls_through_the_floor_on_the_walk` (d1): **pass,
  12.18 s** — no guard fall on the default walk.
- `walkreal_07000001_live walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`
  (d2, real damage): **pass, 8.22 s** — no guard fall.
- Full suite: **not run** — the window went to setup + recon; the full test-binary build only
  finished at ~21:35 (two 10-minute build chunks on 2 vCPUs; ALSA dev symlink had to be
  user-prefixed). Owed to the next push.

## What's left

1. **Reproduce.** The driver's "one 01 walk variant" is most plausibly the IGNORED d3 00agent
   run (`..._real_damage_00agent`, the door-phase crowd runs at tick 22k+) or a longer crowding
   session than the two quick variants. Also worth a direct attempt: walk Bond INTO d#4 on the
   upper west floor and crowd him against the north wall (z → 16.7) while he dodges, watching
   `[guard-fall]` + `NF_DRONE_WATCH=4`.
2. Pin the crossing tick (position + state per tick), then pick the fix:
   - if the guard leaves through a door/kick/passable-flagged edge — check the game's mask
     skip against the port's wall flags on this level;
   - if the game's per-cel BLINK bounds close the north edge differently than the port's flat
     wall list — port the per-cel bounds (the documented one-cel deviation, nav.rs:70-73);
   - a single-tick displacement source (pane push / ride / script move) crossing the wall —
     gate that displacement the way the game does.
   No fix before the red test is red (rule 4).

## Live checks for the main agent

1. Watch the ch-0xF pair (the port's d#4/d#12 — the game's drone indices for the two ch-0xF
   spawns) with a position+state dump while Bond crowds them at the west stairwell and on the
   upper west floor: confirm the game NEVER lets a guard cross z ≈ 16.7 north on the upper
   floor (y ≈ 9.2-10.2), and record the closest approach.
2. Dump the game's upper-west-floor cel boundary (BLINK) wall list for the cels at
   x −34..−20, z 0..17: does the game's per-cel bounds close the north edge the same way as the
   port's flat walls (#94-101)? Any edge the port lacks (or carries with different flags)?
3. If the driver's variant was the d3 00agent run: the tick and the guard's last state/position
   before the y drop, from that run's log.

## Usage

not visible to me — this environment shows no token, context-window or cost figures for the
session (no usage readout is exposed to the worker).
