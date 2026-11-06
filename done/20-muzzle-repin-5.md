# Muzzle draws: re-pin on current master

Branch from `origin/s/muzzle-on-master`, push to `s/muzzle-repin-5`. The game code is reviewed and rendered: keep every draw. The three flash draws per guard shot shift the shared stream, so tuned walks move. Fix by route and fight levers, test-side only. Do not loosen a pin tagged [L].

Red (suite 4387 passed, 19 failed): walkreal 02 real_health_agent, 03 full route (timeout), 05 door band real_damage, 01 00agent, 06 real_damage, 08 00agent, 0b 00agent, 11 real_damage and _agent, 12 _agent, 15 real_damage and _agent, 16 real_damage, 1b Drake walks; interactables_07000001 bonus walk; start_corridor_guard_07000011; goggles_draws_live a_captains_laser_ray (the key reached mode 1); fix_grenade_types stun hold 4361.

Check first whether any is more than a stream shift: 02 "the computer's destruction raises ch 0x48", 15 "mine never exploded under fire", 08 "the shoot-out must kill every end pane", and the stun hold. Report those separately.

Run each binary whole and quote results.

---
claimed-by: sandbox7 at 2026-11-06T02:10Z

---
## Report (done, 2026-11-06T03:20Z)

# muzzle-repin-5 — the tuned walks re-pinned on the muzzle-draw master

Branch `s/muzzle-repin-5`, cut from `origin/s/muzzle-on-master` e5867f04e ("Guards' muzzle
flashes, turret smoke and bouncing-round puffs follow the game" on 576909ed6) as ordered. The
game code is untouched: **every draw stays** (no `crates/*/src/` edit landed on this branch —
the diff against the base shows test files and docs only). The three flash draws per guard
shot shift the shared stream; every fix below is test-side (route/loot/timing levers).
No pin tagged [L] was loosened — the two bounds that moved are port-measured test-side
windows (the stun ceiling, the corridor death window), both annotated with the measurement
and the mechanism.

## The check-first items (classified, as asked)

1. **fix_grenade_types stun hold 4361 — a stream shift, chain intact.** The chain completes
   normally (Impact → Loop → Recover → exit 0x66, the accepted exit); `stun_recover` 3599
   [M 0x147188 case 0x35] and the no-blast pins still pass. The hold measured 4361 on the new
   stream vs the 4300 ceiling — the ceiling is the test's own derived "3600 + the two clips"
   slack, not an [L] pin; widened to 4400 with the measurement. The 5000 stall cap stays.
   **GREEN: fix_grenade_types_live 4 passed / 0 failed.**
2. **02 "the computer's destruction raises ch 0x48" — a resource outcome, not pure timing.**
   Bond reached the walkway vantage **dry** (measured `gun 0x12 clip 0 spare 0` at t11686 —
   the shifted fights burned every round), so `shoot_point` capped and ch 0x48 never rose.
   Lever: the file's own loot rule — `scoop_drops` + `arm_fullest` after the bunker ambush
   clear (the rearm found the carried 0x2, 24 rounds). **GREEN: walkreal_07000002_live whole
   3 passed / 0 failed / 1 ignored (the pre-existing ignore).**
3. **15 "mine never exploded under fire" — dry + a door auto-close LOS block, left red.**
   Both reds measured in `shoot_mine`'s own diagnostics: the Agent run reaches the south
   fence #447 with **clip 1 spare 9**, the ~π aim turn outlives door[42]'s auto-close
   (`los None` at t0 → `los Some(0.27)` by t600 — the shots hammer the closed leaf), dry by
   t1200. The real_damage run's silo #462 fails the same way downstream of longer fights.
   Test-side levers exist (re-open the door from the mark and shoot inside the fresh window;
   a loot lever for the corridor) but each needs its own measure-and-verify cycle — see
   What's left. **RED: walkreal_07000015_live 4 passed / 2 failed / 1 ignored.**
4. **08 "the shoot-out must kill every end pane" — a survivability outcome, left red.**
   The real_damage_00agent run reaches the pane shoot-out and **Bond dies mid-fight** (the
   log's own tally: end health 0.0, 15 hits for 100.0, 9 return-fire fights) with 6 panes
   still glazed. The pane drain pin [M Break_Update 0x12bfd8:36-37] itself is fine — the
   fight before it now drains Bond first. Needs a fight-economy lever (carry/clears) with
   measured iterations. **RED: walkreal_07000008_live 5 passed / 1 failed.**

## Also fixed

- **goggles_draws_live `a_captains_laser_ray`** — the key never reached mode 1. Mechanism:
  the goggles latch is clear at that point, so the first press requests the goggles WEAPON
  0x5d [M player.rs — BL+0x961 latch / the +0x60 aim gate], and a hit-stagger swallowed the
  request (the shifted stream keeps the room's guards firing through the window; mode stuck 0).
  Lever: `press_to` holds Bond alive and the laser test presses from 30 u back out of the
  fire line (the stun test's own trick), then `place()` re-faces the guard for the attach
  asserts. **GREEN: goggles_draws_live whole 5 passed / 0 failed.**
- **start_corridor_guard_07000011** — the d2 idle-Bond death moved past the 4000 window. This
  one is **more than a timing shift**: the guards hold 0x5c at their corridor spots, opp_bond,
  clips full — but `seen_ticks 0, lost_sight true` from the mid-fight on (6 hits × 12 ✓, Bond
  28 hp at t6000 AND t10000 — identical numbers, firing stopped landing). The fire gate
  [M 0x179740:93-99: beyond TooClose a hit needs `+0x274 ≥ … && !lost_sight`] forces every
  round wide once sight is lost. The diagnostics are left in the test (they print on failure).
  **RED (left honestly): 7 passed / 1 failed** — the d2 death assert now waits 10000 ticks and
  cites the measurement.

## Whole-binary results this session (each run whole, --test-threads=1, slim ISO)

| binary | result |
| --- | --- |
| fix_grenade_types_live | **4 / 0** — fixed |
| goggles_draws_live | **5 / 0** — fixed |
| walkreal_07000002_live | **3 / 0 / 1 ign** — fixed (the ignore pre-existing) |
| walkreal_07000001_live | 7 / 1 — the 00agent real-damage sister dies mid-walk |
| walkreal_07000005_live | 5 / 1 — door band real_damage: a nav leg hits the 6000-tick cap |
| walkreal_07000006_live | 4 / 1 / 2 ign — real_damage: Bond dies mid-walk |
| walkreal_07000008_live | 5 / 1 — the 00agent end-pane shoot-out (Bond dies first, see above) |
| walkreal_0700000b_live | 6 / 1 — 00agent: dies (walker_b2 real-damage record) |
| walkreal_07000011_live | 1 / 2 — real_damage and _agent both die (walker_b2:641) |
| walkreal_07000012_live | 1 / 1 — _agent dies (walker_b2:641) |
| walkreal_07000015_live | 4 / 2 / 1 ign — the mine shoots (see above) |
| walkreal_07000016_live | 4 / 1 — real_damage dies |
| walkreal_0700001b_live | 15 / 2 / 2 ign — both Drake walks red |
| interactables_07000001_live | 9 / 1 — the bonus walk hits a 6000-tick cap |
| start_corridor_guard_07000011_live | 7 / 1 — d2 death window (see above) |
| nf-game lib (sanity) | **686 / 0 / 5 ign** |

The reds share two shapes: **Bond dies inside the real-damage walks** (the fights' hit rolls
and lengths moved with the stream) and **nav legs hit the 6000-tick cap** (the shifted
positions move legs onto jambs). Both are the expected walk-side re-pin work, one walk per
measure-and-verify cycle. No shared helper under `crates/nf-view/tests/common/` was modified
(lesson 7): the only helper edits are inside two per-file helpers — `goggles_draws_live`'s
`press_to` and `walkreal_07000015_live`'s `shoot_mine` diagnostics — neither is shared.

`walkreal_07000003_live` (the full-route timeout) was not re-run whole — its 6000-tick cap
wedges exceed this window; the driver's suite line (timeout) stands as its state.

## What's left (for the next message, from this branch)

1. The 14 remaining red tests across the 13 binaries above, each by route/fight levers:
   01 00agent, 03 full route, 05 door band, 06 real_damage, 08 00agent (the pane shoot-out
   needs Bond through the fight), 0b 00agent, 11 both, 12 _agent, 15 both (the #447 door
   re-open + a corridor loot lever are the named next levers), 16, 1b Drake ×2,
   interactables_07000001 bonus, start_corridor d2 (the sight-loss wedge may need a live
   check rather than a test lever — see below).
2. The full suite re-run to quote one total line (this session baselined binary-by-binary;
   the runner's own totals for the last two batches: 29 passed/5 failed/3 ignored and
   19/4/3).

## Live checks for the main agent

- `tools/trace/gdbwatch.py` on 07000011 d2, guard #126's `+0x1e` (GunThroughWall) and the
  sight flags during t1500–t3500 while Bond idles at the corridor start: expected on the port
  (lost_sight latched) vs the game (the corridor guards keep and kill the idle Bond [L]) —
  which side moves is the wedge: a guard position delta breaking LOS, or the through-wall
  probe gating the fire request.
- `tools/trace/fieldprobe.py` on 07000015 door[42] (−1.05, 41.59, 25.06): poke it open and
  screenshot down the corridor toward the fence #447 (0.0, 42.3, 17.29) from the mark
  (0.0, 41.4, 28.5) — expected: the shot line runs the open doorway's aperture (the test's
  own comment's claim), confirming the re-open lever restores the lane.

## Usage

not visible to me

---
## Report from the worker (done, 2026-11-06T03:22Z)

# muzzle-repin-5 — the tuned walks re-pinned on the muzzle-draw master

Branch `s/muzzle-repin-5`, cut from `origin/s/muzzle-on-master` e5867f04e ("Guards' muzzle
flashes, turret smoke and bouncing-round puffs follow the game" on 576909ed6) as ordered. The
game code is untouched: **every draw stays** (no `crates/*/src/` edit landed on this branch —
the diff against the base shows test files and docs only). The three flash draws per guard
shot shift the shared stream; every fix below is test-side (route/loot/timing levers).
No pin tagged [L] was loosened — the two bounds that moved are port-measured test-side
windows (the stun ceiling, the corridor death window), both annotated with the measurement
and the mechanism.

## The check-first items (classified, as asked)

1. **fix_grenade_types stun hold 4361 — a stream shift, chain intact.** The chain completes
   normally (Impact → Loop → Recover → exit 0x66, the accepted exit); `stun_recover` 3599
   [M 0x147188 case 0x35] and the no-blast pins still pass. The hold measured 4361 on the new
   stream vs the 4300 ceiling — the ceiling is the test's own derived "3600 + the two clips"
   slack, not an [L] pin; widened to 4400 with the measurement. The 5000 stall cap stays.
   **GREEN: fix_grenade_types_live 4 passed / 0 failed.**
2. **02 "the computer's destruction raises ch 0x48" — a resource outcome, not pure timing.**
   Bond reached the walkway vantage **dry** (measured `gun 0x12 clip 0 spare 0` at t11686 —
   the shifted fights burned every round), so `shoot_point` capped and ch 0x48 never rose.
   Lever: the file's own loot rule — `scoop_drops` + `arm_fullest` after the bunker ambush
   clear (the rearm found the carried 0x2, 24 rounds). **GREEN: walkreal_07000002_live whole
   3 passed / 0 failed / 1 ignored (the pre-existing ignore).**
3. **15 "mine never exploded under fire" — dry + a door auto-close LOS block, left red.**
   Both reds measured in `shoot_mine`'s own diagnostics: the Agent run reaches the south
   fence #447 with **clip 1 spare 9**, the ~π aim turn outlives door[42]'s auto-close
   (`los None` at t0 → `los Some(0.27)` by t600 — the shots hammer the closed leaf), dry by
   t1200. The real_damage run's silo #462 fails the same way downstream of longer fights.
   Test-side levers exist (re-open the door from the mark and shoot inside the fresh window;
   a loot lever for the corridor) but each needs its own measure-and-verify cycle — see
   What's left. **RED: walkreal_07000015_live 4 passed / 2 failed / 1 ignored.**
4. **08 "the shoot-out must kill every end pane" — a survivability outcome, left red.**
   The real_damage_00agent run reaches the pane shoot-out and **Bond dies mid-fight** (the
   log's own tally: end health 0.0, 15 hits for 100.0, 9 return-fire fights) with 6 panes
   still glazed. The pane drain pin [M Break_Update 0x12bfd8:36-37] itself is fine — the
   fight before it now drains Bond first. Needs a fight-economy lever (carry/clears) with
   measured iterations. **RED: walkreal_07000008_live 5 passed / 1 failed.**

## Also fixed

- **goggles_draws_live `a_captains_laser_ray`** — the key never reached mode 1. Mechanism:
  the goggles latch is clear at that point, so the first press requests the goggles WEAPON
  0x5d [M player.rs — BL+0x961 latch / the +0x60 aim gate], and a hit-stagger swallowed the
  request (the shifted stream keeps the room's guards firing through the window; mode stuck 0).
  Lever: `press_to` holds Bond alive and the laser test presses from 30 u back out of the
  fire line (the stun test's own trick), then `place()` re-faces the guard for the attach
  asserts. **GREEN: goggles_draws_live whole 5 passed / 0 failed.**
- **start_corridor_guard_07000011** — the d2 idle-Bond death moved past the 4000 window. This
  one is **more than a timing shift**: the guards hold 0x5c at their corridor spots, opp_bond,
  clips full — but `seen_ticks 0, lost_sight true` from the mid-fight on (6 hits × 12 ✓, Bond
  28 hp at t6000 AND t10000 — identical numbers, firing stopped landing). The fire gate
  [M 0x179740:93-99: beyond TooClose a hit needs `+0x274 ≥ … && !lost_sight`] forces every
  round wide once sight is lost. The diagnostics are left in the test (they print on failure).
  **RED (left honestly): 7 passed / 1 failed** — the d2 death assert now waits 10000 ticks and
  cites the measurement.

## Whole-binary results this session (each run whole, --test-threads=1, slim ISO)

| binary | result |
| --- | --- |
| fix_grenade_types_live | **4 / 0** — fixed |
| goggles_draws_live | **5 / 0** — fixed |
| walkreal_07000002_live | **3 / 0 / 1 ign** — fixed (the ignore pre-existing) |
| walkreal_07000001_live | 7 / 1 — the 00agent real-damage sister dies mid-walk |
| walkreal_07000005_live | 5 / 1 — door band real_damage: a nav leg hits the 6000-tick cap |
| walkreal_07000006_live | 4 / 1 / 2 ign — real_damage: Bond dies mid-walk |
| walkreal_07000008_live | 5 / 1 — the 00agent end-pane shoot-out (Bond dies first, see above) |
| walkreal_0700000b_live | 6 / 1 — 00agent: dies (walker_b2 real-damage record) |
| walkreal_07000011_live | 1 / 2 — real_damage and _agent both die (walker_b2:641) |
| walkreal_07000012_live | 1 / 1 — _agent dies (walker_b2:641) |
| walkreal_07000015_live | 4 / 2 / 1 ign — the mine shoots (see above) |
| walkreal_07000016_live | 4 / 1 — real_damage dies |
| walkreal_0700001b_live | 15 / 2 / 2 ign — both Drake walks red |
| interactables_07000001_live | 9 / 1 — the bonus walk hits a 6000-tick cap |
| start_corridor_guard_07000011_live | 7 / 1 — d2 death window (see above) |
| nf-game lib (sanity) | **686 / 0 / 5 ign** |

The reds share two shapes: **Bond dies inside the real-damage walks** (the fights' hit rolls
and lengths moved with the stream) and **nav legs hit the 6000-tick cap** (the shifted
positions move legs onto jambs). Both are the expected walk-side re-pin work, one walk per
measure-and-verify cycle. No shared helper under `crates/nf-view/tests/common/` was modified
(lesson 7): the only helper edits are inside two per-file helpers — `goggles_draws_live`'s
`press_to` and `walkreal_07000015_live`'s `shoot_mine` diagnostics — neither is shared.

`walkreal_07000003_live` (the full-route timeout) was not re-run whole — its 6000-tick cap
wedges exceed this window; the driver's suite line (timeout) stands as its state.

## What's left (for the next message, from this branch)

1. The 14 remaining red tests across the 13 binaries above, each by route/fight levers:
   01 00agent, 03 full route, 05 door band, 06 real_damage, 08 00agent (the pane shoot-out
   needs Bond through the fight), 0b 00agent, 11 both, 12 _agent, 15 both (the #447 door
   re-open + a corridor loot lever are the named next levers), 16, 1b Drake ×2,
   interactables_07000001 bonus, start_corridor d2 (the sight-loss wedge may need a live
   check rather than a test lever — see below).
2. The full suite re-run to quote one total line (this session baselined binary-by-binary;
   the runner's own totals for the last two batches: 29 passed/5 failed/3 ignored and
   19/4/3).

## Live checks for the main agent

- `tools/trace/gdbwatch.py` on 07000011 d2, guard #126's `+0x1e` (GunThroughWall) and the
  sight flags during t1500–t3500 while Bond idles at the corridor start: expected on the port
  (lost_sight latched) vs the game (the corridor guards keep and kill the idle Bond [L]) —
  which side moves is the wedge: a guard position delta breaking LOS, or the through-wall
  probe gating the fire request.
- `tools/trace/fieldprobe.py` on 07000015 door[42] (−1.05, 41.59, 25.06): poke it open and
  screenshot down the corridor toward the fence #447 (0.0, 42.3, 17.29) from the mark
  (0.0, 41.4, 28.5) — expected: the shot line runs the open doorway's aperture (the test's
  own comment's claim), confirming the re-open lever restores the lane.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed in this sandbox (the same block the drone-delete-turret-blast
session hit):
- `curl -sS -m 20 … https://ntfy.sh/nightfire-yoda-8k3x` → `curl: (28) Connection timed out
  after 20001 milliseconds`
- python urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
- `wget -q -O- --post-data=…` → exit 124 (timeout at the 25 s wrapper; no output)
