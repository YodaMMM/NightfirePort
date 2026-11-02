# level-13-lobby-rifles — the 07000013 lobby rifle guards can be answered from the walked lane

Branch `s/level-13-lobby-rifles`. The task: `walkreal_07000013_live.rs`
`walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` was #[ignore]d — on real damage the
walk died under two rifle guards whose rounds reach Bond while the walker's `exposed_part` world
trace never saw their hit boxes. Decide port bug vs honest geometry, fix or route around, un-ignore,
quote the totals.

## Verdict: honest cover — and three real walker/port holes behind it

**The asymmetry itself is the game's own cover behaviour, not a port bug.** Evidence:

- The named 07000013 guards were never the first killer: on the current tree the real-damage walk
  died on **07000012** first (tick 78087, the south-east bend), under the same pattern. The NF_RF_GEOM
  ray dump (added to `damage_log.rs`, env `NF_RF_GEOM` + `NF_RF_TRACE`) shows the bend guards in
  **AIM_CROUCH_FIRE (state 0x70)** [M the port's state table, the game's AimCrouchFire 0x167060
  family] behind the east office block, with **every box-centre ray from the lane stopped** at
  `Office-C_Obj9_computer` — pushed instance 318, world bounds (14.99, 0.91, −2.10)..(15.73, 1.60,
  −1.33) [D disc mesh, named via the new `probe12_geom.rs` against `level.instances(true)` +
  `build_scoped`'s push filter], with the `Office-C` room mesh (instance 302) behind it. The guards'
  own spread pellets clear the desk lip; their 0.00-damage grazes in `player_hits` are the rest of
  the cone clipping Bond's cylinder for no health.
- The port's fire gate is wired and honest: `DroneVision_GunThroughWall` 0x178520 ([M]
  `drone.rs::gun_through_wall`) gates every combat fire request on `+0x3b = !+0x1e` [M
  0x164960/0x166328/0x167060 arms], and the guard's muzzle ray was clear — he shoots over the desk,
  exactly like the game. Weapon records [L ee_start_05.bin @ 0x2bf150 + id·0x10C]: 0x18/0x19 damage
  3.5, 0x1c 3.5 ×8 pellets — the observed −7/−14 hits are the ×2 pistol-class mul [M 0x14b300:267-293]
  and the TooClose ×2 [M 0x139fac-0x139fc8]; the one −75.5 was a guard **grenade blast** (throw state
  0x2D), not a bullet — no 0x19 blast flags exist on the disc [L +0x6c = 0x00040050].

So the fix branch of the task applies: give the walker the honest tactics a player uses against
cover, and fix what was genuinely broken in the port/walker. What was genuinely broken:

1. **`scoped_run_owners` misnamed every owner** (world.rs): it indexed `stat.owners` by the run's
   `tris.start` (a tri offset) instead of the run's own ordinal — every run past the first few read
   past the table and reported u32::MAX. Fixed to `owners[run]` (the per-cell buckets iterate the
   same table by run ordinal [M `attribute_scope`/`build_scoped`]), plus a new `scoped_run_bounds`
   diagnostic that names a blocking run's world-space bounds.
2. **`exposed_part` used the eye ray only.** The round leaves the view model low-right of the eye;
   at a wall edge the eye ray cleared while every round died on the jamb — the 07000013
   security-room stand held `aim Some`, error 0.000, clip draining, and the target's hp pinned at
   6.0 [L the fight#9 rf-trace]. `exposed_part` now requires the eye AND both low-lateral
   gun-offset rays clear (damage_log.rs).
3. **The resupply never armed anyone.** Three stacked holes [L the run logs]: the 1.8 arrive radius
   stopped outside a small drop's `radius + 1.1` take sphere [M 0x18e148:203-214]; the y filter
   chased placed boxes buried half a unit under the floor while the real guard drops waited farther
   out (the drops rest with their centre ON the floor ray [L drop-stand-height probe 2026-10-22]);
   and a taken gun's ammo lands in the inventory [M Pickup_Handler 0x18e650's equip path] while the
   pass's clip-only success test read the pass as dry — so it re-walked the same ghost 4 × 6000
   ticks, every pass. Fixed: on-floor y filter, walk-onto arrive 0.6, inventory-aware success test,
   a 12 u fetch cap (the tick-1102 death was a 14 u cross-lobby fetch that ate a frag blast for the
   remaining 75.5 [L NF_DMG_TRACE log]), and already-tried targets skipped.

The walker's new honest tactics (all in `damage_log.rs`, opt-in under the existing `lane_fights`):
the unseen-cover fallback fights the shooter that is actually hurting Bond (or the nearest firing
one — `shooters` already guarantees hit-in-240 or firing-at-him-now [M the shooters filter]) even
when no box is reachable, with a short 300 cap; and the stand **creeps toward the muzzle** while
unseen (never inside the back-off range) — a player pushing up to cover until his eye line opens
over the lip. The creep is what answers the crouch-fire desks.

Route fixes in the test itself (leg comments carry the run-log evidence): the armour-shelf stand
beside the unit instead of pushing into it, and the entrance-mouth arrive 0.9 (the gate post wedged
the last 0.75 while Bond was already inside touch #156's band [D x −8.61..−4.15, z 65.09..67.16]).

## The named 07000013 guards

Both named rifle guards die on the walked lane now: spawner #6 killed at tick 1153, mezzanine #4 at
1305 (the run log's "13: the lobby west mid: returned fire on drone #6/#4 (killed true)") — the
walker fights them from the lane with the same exposure rule; no port change to their hit volumes
was needed or made. The desk block at x −14.6..−12.9 / z 31.3..32.9 was not needed as a stand
position — the fight resolves from the lane once the exposure rule matches where the rounds
actually leave.

## Runs

- `walkreal_07000013_live`: **2 passed, 0 failed, 0 ignored** (`walk_the_warm_chain...` +
  `..._real_damage`, release, `--test-threads=1`, 23.79 s) — the real-damage variant un-ignored and
  green START TO FINISH (12 → 4A → 13 → mission success → the end sequence).
- Full suite (fast runner, final tree merged with origin/master 0ebfee9a): see the total line below
  (filled in from `suite.out`).

## Live checks for the main agent

1. The BodyArmourLight #414 take ray (07000013 security room, (−27.66, −368.51, 25.57)): from every
   spot the walker can reach (floor east of the unit; on top of the unit) the take fails — armour
   stays 0 while the distance test passes (1.27 < radius+1.1) — the LOS half of `Pickup_Update`
   0x18e148:203-214 ends inside the unit's own mesh. Watch (gdbwatch) the take-ray call while a
   fresh-boot player walks onto the shelf from the east: does the game take it, and from where?
2. The 0.00-damage guard pellet entries in `player_hits` (BL+0x910-family): a pellet reaching
   Bond's cylinder with 0 loss — confirm on a fresh boot that a crouch-firing guard behind a desk
   produces graze hits with no health loss (the port records (frame, owner, 0.0)).
3. The −75.5 frag blast on 07000012's east wing (a 0x19 guard's throw, state 0x2D): confirm the
   Agent-difficulty blast damage at ~1 u falloff lands near 75 on a fresh boot.

## What's left

- The #414 armour take-ray item above (the detour stays in the route, the take is explicitly not
  pinned — a warning line, not an assert).
- The probe/diagnostic tools committed for the next session: `probe12_geom.rs` (instance naming),
  `NF_RF_GEOM` (the per-shooter ray dump), `scoped_run_bounds` (world.rs).

## Usage

not visible to me
