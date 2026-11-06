# level-14-ignored-walks — report

Task: `walkreal_07000014_live`'s ignored tests — read each reason, fix what can be fixed
test-side on current master, remove the ignore when green. Real input, no teleports; no [L] pin
loosened. Branch `s/level-14-ignored-walks` (from origin/master c2c4a6efd).

## What master actually carried

The driver's "3 ignored tests" is **one real `#[ignore]`** on this master:
`walk_the_infiltration_to_the_delta_exit_real_damage_00agent` (the 00 Agent wedge at the 1.72
landing resupply leg). The other two matches in the file are comment lines quoting `#[ignore]`
inside the stale-note block at the top — both tests that share that helper were already
un-ignored and green by the merged work. No other 07000014 walk file carries an active ignore
relevant to this task.

## Baseline on current master (measured, 2026-11-06)

Running the ignored test on this tree does **not** reproduce the recorded wedge — it dies much
earlier, by damage:

```
[real-damage] Bond died at tick 4587 (frame 4587) at (30.62,0.55,33.28) armour 0.0; last hits:
  frame 4537: -5.00 from drone #6 skin 0x5000093 weapon 0x14 at (32.3,1.8,33.4) d 2.1 hp 28
...
end health 0.0 armour 0.0, lowest 0.0, 20 hits for 100.0, other 0.0, 7 return-fire fights
  65.0 in 13 hits: drone #0 skin 0x5000090 weapon 0x0 at (49.0,0.4,63.1) d 35.0 hp 0
  35.0 in 7 hits: drone #6 skin 0x5000093 weapon 0x14 at (32.3,1.8,33.4) d 2.1 hp 28
test result: FAILED. 0 passed; 1 failed; ... finished in 7.19s
```

The trench guard (placement #270) closed to d 1.2 before the first torso-gated tap landed and
13 of his rounds hit for 65.0 (every round lands inside 3.0 [M DroneWeap_DoBulletAccuracy
0x179740:113-115]); the P2K's clip 3 spare 9 could not out-race his bursts. Everything
downstream then fought dry (clip 0 spare 0 by tick 3295) and the death came when the dry
resupply fetch walked Bond point-blank past live drone #6. The recorded ignore's wedge (tick
26340, the hover dither at the resupply leg) sits behind this fight phase and was never reached
on this tree before the levers below.

## The levers that landed (test-side only, all d3-gated, real input)

All three are gated behind thread-locals in the level's own test file
(`crates/nf-view/tests/walkreal_07000014_live.rs`) — the difficulty-1/2 and real-damage call
sites are byte-identical by construction (libtest isolates threads; the flags default false and
only the 00agent body sets them). No shared file under `crates/nf-view/tests/common/` was
touched; no game bytes, no engine change.

1. **The trench fight answers with the return-fire stand** (`TRENCH_STAND` +
   `with_trench_stand`; `kill_the_trench_guard_270`'s d3 branch): `switch_to_gun` +
   `stand_and_fire` instead of the torso-gated `shoot_loose`. The stand is the walk's own
   proven fight: the exposed-part aim takes the clear HEAD box first (part 5 within 6 u — the
   head-hunt wrap is on), the `back_off` 4.0 band is held by the stand's own backpedal, and the
   reload runs the real R path. Measured: the trench cost drops from **65.0 in 13 hits** to
   **15.0 in 3 hits** (health 85.0 after the fight).

2. **The guard's own drop is scooped** (the bounded test-local loot pass, the
   `clear_hostiles` pattern widened to 9 u / |dy| < 1.0 for the stand's backpedal): the shared
   lane resupply EXCLUDED this drop on the measured run (its 0.5 floor band; "resupply done
   (false)" with the drop untouched), and the first fetch arrive of 1.2 stopped 1.17 short of
   the small-model drop and the take never fired — the 0.6 arrive is the lane resupply's own
   law (the take test is the pickup's sphere within `bounds.radius + 1.1` of player+0x30 [M
   Pickup_Update 0x18e148:203-214], a small-model gun drop stops taking at ≈ 1.4). Measured
   take: `[take] tick 643 weapon 0x14 kind 0 rounds 11 loaded true → Some(Weapon { weapon: 20 })`.

3. **The taken gun is DRAWN with the wheel** (a bounded `weapon_cycle` press + the 60-tick
   draw wait, the same pattern `switch_to_gun` runs): the fights' `switch_to_gun` returns true
   while the P2K still holds rounds and never cycles to the richer taken weapon, so the pads'
   15 return-fire fights ran on the drying P2K ("killed false" ×15) while the taken SMG's
   rounds sat owned. The direct `switch_to` press is the GADGET draw in this port (the
   crossing's 0x50 Q-Claw, the tripbomb's 0x3a) and was a measured no-op on a gun (40 held
   presses left gun 0x11 in hand). Measured: `the wheel stops on 0x14 (want 0x14): clip 21
   spare 0` — 21 rounds in hand at tick ~700.

## The run's shape after the levers (measured)

```
[014] the trench stand: drone #0 killed true, health 85.0
[014] the wheel stops on 0x14 (want 0x14): clip 21 spare 0
...
walk_to the 1.72 landing: 6000-tick cap
STUCK at the 1.72 landing: bond=(32.13,2.70,39.83) target=(33.09,2.37,35.38) d=4.56 grounded=true
 ... segment_to_target=Some(0.3635162) ... health=85.0
[real-damage] at tick 15300: end health 85.0 armour 0.0, lowest 85.0, 3 hits for 15.0, other 0.0, 48 return-fire fights
test result: FAILED. 0 passed; 1 failed; ... finished in 24.19s
```

From 100.0/20 hits dead at tick 4587 to **15.0/3 hits through 48 return-fire fights (all
free — the AIM guards never landed a round), health 85.0 held to tick 15300**. The new blocker
is a displacement wedge, not a death: the pads'/landing's fights stand up every leg (the
proactive-fight gate answers every seen guard on a 240-tick cooldown) and each failed stand's
`back_off` 4.0 crowd-backpedal carries Bond a little off the leg's line; the walk_to "the 1.72
landing" then capped at 6000 ticks from (32.13, 2.70, 39.83) with the direct steer's segment
cut 0.36 along by the pad structure.

## What is left (the next window's levers, in order)

1. **The AIM guards' stands fail "unseen" on the gun-ray pick while the plain eye-ray is
   clear.** The d3 walk sets `lane_fights`, so the stands aim through `exposed_part_gun_rays`
   (the muzzle stand-ins at ±0.25 right); on the mezzanine pads that ray is clipped by the pad
   edges, the stand gives up after the unseen cap without firing, and the fight repeats on its
   cooldown. These guards are AIM placements — they see Bond (they are the ones the sight chain
   fires on), so the plain eye-ray pick (`exposed_part`) is clear where the gun-ray pick is
   not. A `DamageLog` opt-in flag (set by this level's walk only, checked at the aim-pick
   branch, damage_log.rs ~line 799) is the shape; lesson 7 applies: shared helper, so the whole
   `walkreal_*_live` set re-runs before finishing.
2. **The proactive-fight gate fights guards that never dealt damage.** 48 fights against
   guards that landed 0 rounds is pure displacement cost on a real-health walk. A d3-gated
   "answer only after first damage (or first muzzle event) from that drone" flag would keep the
   corridor fights (where the guards do trade) and delete the pads' 48 free stands. Same
   shared-file opt-in shape.
3. **The recorded hover pocket** (the 26340 resupply-leg wedge — the port's fallback radial
   push holding the capsule where the game's dead push-tail [M 0x109da0:431-447] would give
   none) was never reached this window. The recorded lever 1 (the scoped sim-side fix) plus the
   d2 `with_real_damage` one-liner remain the recorded path once the fight phase holds.

## Verification

- The whole `walkreal_07000014_live` binary, before the levers (the master tree, my branch's
  base): `test result: ok. 9 passed; 0 failed; 1 ignored; ... finished in 152.20s`.
- The whole binary after the levers (this branch's tree):
  `test result: ok. 9 passed; 0 failed; 1 ignored; ... finished in 151.22s` — every active
  test stays green; the changes are unreachable outside the 00agent body.
- The ignored test's measured runs are quoted above (7.19 s baseline → 24.19 s after, with the
  health/damage/fight-count record in each).
- The full four-crate `cargo test` suite did not fit this window (fresh `target/` after a
  disk-full rebuild consumed the first ~40 minutes; the remaining budget went to the measured
  iterations). No file outside this one test file changed, so no other binary's inputs moved;
  the walkreal binary above is the affected surface, run whole, twice.

## Live checks for the main agent

None required this window — no visual work, no [L] pin moved.

## Usage

not visible to me
