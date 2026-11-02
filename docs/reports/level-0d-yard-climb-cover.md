# level-0d-yard-climb-cover — the 0d climb under the sniper

Branch `s/level-0d-yard-climb-cover` from origin/master (0ea03862). Task: pick the climb timing
and approach a player would against drone #31 (placed sniper, weapon 0x1e, 16 per hit on Easy,
the 180-tick cadence) — climb between shots, take the covered side, or take the sniper out from
cover — then un-ignore `walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage` on real
input only.

## Status: not finished — the perch kill is proven free, the walk stays `#[ignore]`d

The real-damage twin is still ignored: after the free perch kill, the climb/wire crossfire (the
west roof #28/#29, the dock patroller #33, the south heavy #10, and #16 once the gunfire turns
it) still outguns the 100-health pool. The non-damage twin passes the FULL mission with the kill
legs in (green, 20.78 s). The full suite has NOT run (deadline) — the changes touch only
`walkreal_0700000d_live.rs` (no shared helper touched) and the new assertion-free probe.

## What the probe mapped (probe_0d_climb_cover, new, assertion-free `--ignored`)

- **The shooter map.** At the intro only #16/#31/#33 are enabled. The PLATFORM DROP wakes
  #27/#28 (the west roof, (145.6-145.8, 22.7, −237/−257)), #10 (skin 0x5000093,
  (214.2, 12.09, −289.44)) and #14/#15/#18 (the south roof) within a tick of Bond standing at
  the drop point. #29 wakes even later (its only hit in the death run was tick 2988, mid-ride).
- **#31 owns the whole climb**: visibility 1.68 with a clear ray at every ladder point
  (base/low/mid/high/top) and on the early wire; its blind wedge covers only the strip itself
  (vis 0.53-0.58 at the strip stands). The climb cannot be dodged through the wedge.
- **The reverse-ray matrix.** From the strip stands the raw rays to #31 (d 21-44), #33 (13-22)
  and #10 (59-64) are CLEAR. To the west roof (#27/#28/#29) EVERY ground ray is blocked by the
  canopy (fractions 0.79-0.90 from all six stands) — the roof cannot be killed from cover;
  #14/#15 are blocked from everywhere too.
- **Bond's kit**: the held 0x1e (clip 5 + spare 30); owned 0x1/0x2/0x1e/0x4a/0x4e/0x50/0x54 —
  no grenades, no sniper (the level's PICKUP_300AP (Sniper) #523/#524 stand right beside the
  ladder base at (221.5, 0.49, −232/−233) but only feed a rifle carried from 0c, which the walk's
  level-start loadout does not have).

## Two mechanics the fights measured (both now in the walk's comments)

1. **The walker's ±0.02 rad aim gate is wider than a guard's ~0.3 u torso box beyond ~15 u**
   (0.02 rad ≈ 0.66 u at 33 u): a 900-tick `shoot_bounded` fight at d 33 with 50+ trigger taps
   never landed a round. The new local `shoot_far` (kept local — the file's `shoot_at` precedent,
   "kept local to keep walker.rs additive") scales the gates to the range (±0.25 u lateral,
   ±0.6 u vertical) and holds the stand (the walker's strafe repositioning slid Bond 5 u out of
   his own cover once — measured).
2. **The aim-mode fire never lands steep shots.** With `aiming: true` 8 dead-centre rounds at
   d 4.4-17 (err/perr ≈ 0.000) all missed a patrolling guard; the walker's hip-fire (no aiming
   flag) lands. `shoot_far` hip-fires.
3. Also pinned: **the armour cannot absorb bullets** — the pain types 1..7 carry
   armourFraction 0 [M `Player_HandlePain` 0x1902c8:84-104], so the pool is the 100 health flat
   (the death runs confirm: 100 damage killed from 100 hp + 50 armour). The per-guard cadence is
   the ~180-tick interval [D weapon 0x1e data; M `DroneWeap_NextBulletTime` 0x179d40 — the
   trace's #28 hits 1997→2177].

## The tactic as it stands (all real input, no health/armour/ammo writes)

1. **THE PERCH KILL — free and proven.** From the drop point (181, −226) the reverse ray to #31
   is clear at d 21 while its idle cone still reads the spot blind (vis < 0.7 → no LOS probe at
   all [M `DroneVision_ProcessDroneSight` 0x176a28]). One tap: **down at +19 ticks, zero
   incoming, pool 100.0 → 100.0** (measured twice; the mission's FIRST gunshot, fired before any
   guard has a reason to face the strip).
2. **#33's clear window has closed by the drop.** The patroller walks east (~0.003 u/tick) and
   east of x ≈ 192 the dock's south face eats every steep shot from the west (measured: 8 + 13
   aimed rounds at d 4-18 from x 180.8-194.3 with dead-centre aim all died on the face; the
   probe's "clear" rays were against the census position x 191.26). The attempt stays in the
   walk as a 600-tick log-only fight so the next message inherits the measured state.
3. **The sound-turn is real**: #31 fired at Bond during the #33 fight (its wedge rotated east
   after the first shots) — every standing fight buys ~180-tick cadence shots from every guard
   the noise reaches.

## What is left (the next message's levers)

- **Kill #33 from the PLATFORM, before the drop**: from the platform edge (y ≈ 11.2) the shot
  at #33 (y ≈ 14.0) is nearly FLAT (+2.8 over dxz 13-17) — the face cannot block a flat ray
  above the lip, and the patrol is still in its west window early. This is the one untried
  free kill.
- Then climb immediately (the roof's ~2 hits ≈ 32 are the honest cost — no ground ray exists to
  remove them), eat #10's one wire hit (16) and #29's late shot, win the deck fight (≤ 16), take
  the bay drop (24 [M 0x1949a8:117-126]): budget ≈ 88-104 against 100 — tight but only if #33
  dies free.
- Un-ignore the walk; run `walkreal_0700000d_live`, EVERY `walkreal_*_live`, the corridor/grenade
  tests and the full suite (none of that ran here — the deadline).
- The kill legs keep the pool assert (`> before − 48`) and log-only outcomes; they run in BOTH
  twins (real fire, no writes), so the non-damage twin stays green.

## Live checks for the main agent

1. **The sound-turn.** PCSX2, 0700000d, from the drop point fire one shot at the perch #31 from
   (181, 1.0, −226.1) with gdbwatch on its yaw: the port's guard turns toward the shot it cannot
   see (its wedge then covers the strip). Does the REAL game's guard turn on heard gunfire with
   no LOS? If it does not, the blind-wedge kills extend to #33 and #10 from the strip and the
   tactic closes the pool easily.
2. **The flat platform shot.** From the platform south edge (179, 11.2, −227.5), can a player
   hit the dock-edge patroller #33 at (191-198, 13.7, −232.2)? Render
   `--bond 179,11.2,-227.5,<yaw toward (196,14,-232)>` — the expected view: the dock edge
   line-of-sight unobstructed.
3. **The canopy.** Render `--bond 196,1.0,-228.5,<yaw toward (145.8,22.7,-237.3)>`: the expected
   view is the canopy/pillar blocking the west-roof sight line (the no-ground-ray verdict,
   visible).

## Usage

not visible to me
