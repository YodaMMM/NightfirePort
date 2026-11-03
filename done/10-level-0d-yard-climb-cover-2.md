# level-0d-yard-climb-cover-2 — finish 0d on real health

Continue from origin/master (it carries level-0d-yard-climb-cover: the perch kill on drone #31 is proven free; probe_0d_climb_cover; the report on origin/s/level-0d-yard-climb-cover lists the next levers — kill #33 from the platform before the drop, avoid the standing fights whose noise turns #31).

Do: apply those levers with real input (no health writes), un-ignore walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage, run walkreal_0700000d_live, every walkreal_*_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.

---
claimed-by: sandbox7 at 2026-11-02T23:38Z

---
## Report (done, 2026-11-03T00:08Z)

# level-0d-yard-climb-cover-2 — the platform-kill lever mapped; the real-damage walk stays `#[ignore]`d

Branch `s/level-0d-yard-climb-cover-2` from origin/master (0ea03862) + a merge of
`origin/s/level-0d-yard-climb-cover` (its code — the perch kill, `shoot_far`,
`probe_0d_climb_cover` — was NOT on master yet; only its report was linked from the queue).
Task: apply the report's levers with real input, then un-ignore
`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`.

## Status: the lever is half-landed and the blocker is now pinned to a pitch law — the walk stays ignored

- The non-damage twin PASSES the full mission with the platform kills wired in (22991 ticks,
  pool 100 → 100 through the kills, 0 stuck, 0 falls) — committed, green.
- The real-damage twin is still `#[ignore]`d: the platform kills do not LAND yet, and the
  reason is measured to the decimal (below). The next message can finish from this branch: the
  fix is one aim-bias term in the local `shoot_far`, no shared helper touched.

## What the probes measured (both new, assertion-free `--ignored`)

### probe_0d_platform — the lever's geometry is REAL

- The warehouse main floor is y **12.19** (the walk's y 11.2 legs ride a lip slope); its
  walkable edge at x ~178 is z ~−207.6. Standing at the dock-edge leg's arrive point
  (178.44, 11.45, −207.40) SLIDES Bond off to a y ~7.9 ledge — and from that ledge every #33
  round dies on the dock face (measured: the first fight burned clip 5→1, health 10 → 10).
- From probe-verified stands on the main floor (the armour arrive point (180.46, 12.19, −207.93)
  among them, settles at y 12.19) the raw rays to BOTH targets are CLEAR:
  #33 (d 28.4) and #31 (torso y 7.15, d 34.0), while BOTH cones read every stand
  x 176-184, z −205..−222 blind (**vis 0.00 → no LOS probe at all** [M
  `DroneVision_ProcessDroneSight` 0x176a28]). The west roof stays ASLEEP through the kills
  (it wakes only when Bond stands at the drop point [probe_0d_climb_cover part E]).
- The walk now kills from the armour arrive point BEFORE the dock-edge leg, #33 first, then
  #31; the post-drop kill calls remain as log-only fallbacks. Real fire path end to end
  (`shoot_far` holds the stand), no writes.

### probe_0d_bullet — the fire law that actually blocks the kills

- **The held 0x1e is a 0x400000 record**: [L ee_start_05.bin weapon_data 0x1e] spread +0x24 =
  **100.0**, +0x68 = 0x408050 (the exact-aim bit), speed 13.2/tick, no gravity bit, dmg 40.
- **Hip-fire keeps the full 100-unit cone** [M `Bullet_init` 0x12c5a0:139-170's
  `player_spread` — mul 0 only with the aim bit]: the offset is ±0.14 per component on a unit
  direction (~4-8°), so a 0.3 u torso box is unhittable beyond ~5 u. Measured: 4-5
  dead-centre hip rounds at d 21.8-33.3 (err/perr 0.000) landed NOTHING — including a repeat
  of the previous branch's "free" d 21 perch kill, which did not reproduce (its two +19-tick
  landings were first-bullet cone luck, not a law).
- `shoot_far` now fires AIMED (`aiming: true`) — the exact-cone path the C5 door kill already
  uses at d 25. Still no damage: `probe_0d_bullet` traces the aimed round and pins the REAL
  blocker — **the bullet spawns at the camera and flies ~0.016 rad FLATTER than the settled
  bond.pitch** (at bond.pitch 0.0256 normalized the bullet's dir[1] says 0.0155): at d 27 that
  is 0.44 u LOW — under a ~0.3-0.5 u torso box. The camera-vs-bond pitch mapping is the one
  unknown left; one calibration (fire at three settled pitches, read the bullet dirs —
  `probe_0d_bullet` already has the harness) gives the bias term, and the platform kills land.

## The next message's steps

1. Fix `probe_0d_bullet`'s calibration loop to call `w.after_step()` each step (the shot
   spawn gate is view-drained; without it only the first bullet spawns — measured), run it,
   and fit the pitch mapping.
2. Bake the bias into the local `shoot_far`'s `want_pitch` (real aim input, no writes), rerun
   the non-damage twin: expect "#33 down true #31 down true pool 100.0 -> 100.0".
3. Un-ignore `walk_arrival_to_the_c5_door_real_damage`; run it, then `walkreal_0700000d_live`
   (both twins), EVERY `walkreal_*_live`, then the full suite (none of the suite ran this
   session — the deadline; the diff touches only `walkreal_0700000d_live.rs` + the two new
   ignored probes, no shared helper).
4. Budget after the kills land: the roof's ~2 hits (32) + #10's wire hit (16) + #29's late
   shot (16) + the deck fight (≤16) + the bay drop (24 [M 0x1949a8:117-126]) ≈ 88-104 against
   the flat 100 pool — tight but winnable with both snipers down; if the deck fight eats
   more, the next levers are the walk-off strip's cover and #16's wake.

## Live checks for the main agent

1. **The camera pitch law.** PCSX2, 0700000d, scoped 0x1e: aim the crosshair at a wall mark
   with a known pitch and fire — does the impact sit where the crosshair says, or does the
   scope camera introduce the ~0.016 rad flat offset the port's trace shows? If the REAL game
   zeroes it (the scope re-references the aim), the port's camera pitch is the bug and the
   fix belongs in the port's camera, not the walk.
2. **The hip cone.** Same spot unscoped (hip): the 100-unit cone should spray ~4-8° —
   confirm no hits on a man-sized target past ~10 u.

## Usage

not visible to me
