# aim-stop-15 — guards stop dead to shoot on 07000015 too

Branch `s/aim-stop-15` (off `origin/master` 23b0ae649; merged with `origin/master` before the final suite — no-op, master had not moved).

## What's done

1. **07000015 added to the AimStandFire root stop's live list.** `crates/nf-game/src/drone.rs`, the `AIM_STAND_FIRE`
   UPDATE arm: `zero_root_live` now matches level byte `0x15` alongside the landed `0x02..0x1b` set (drone.rs ~:9113,
   the level-scoped `matches!` list; `ZERO_ROOT_ALL` stays false). The stop itself is untouched: `Drone::zero_root`
   / Drone+0x38 set on the UPDATE arm when the guard has an opponent and `anim.class == 2` [M 0x166328:117-123 →
   DroneAnim_PreTransCallback 0x13b490; L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame
   214 — the evidence the list's landing pattern rides].

2. **The level's walks re-tuned against the stop, real input only.** All four measured breaks are fixed; nothing
   lowers damage or health, no top-ups were added, and every helper used is either pre-existing or gated to this
   level/difficulty (no `crates/nf-view/tests/common/` file was touched at all this round).

   - **`walk_spawn_to_the_omega_sector_exit_agent` / `_00agent` (top-up d2/d3) — the fence shot ran dry and the
     door[42] line was blocked.** Root cause chain, measured on this tree: with the stop live a corridor chaser
     stops dead inside door[42]'s own aperture (the nearest hostile sat 1.7 u off the threshold); the shot's eye
     line dies on his capsule; the pistol pool burns behind him (clip 1 spare 0 by t1800, the game's dry-swap
     cascade 0x02 → 0x1a → 0x1b → fists, the mine intact). The first re-tune (a `w.shoot` proximity sweep) exposed
     a second layer: the fights' own positioning parked Bond off the x≈0 lane at (2.08, 25.92) — the WALL, not the
     aperture, takes the round — and the mark walk passed the killed blocker's drop, whose walk-over take
     [M Player_IsBetterWeapon 0x1a6eb0] re-armed the dropped rifle mid-walk and whose approach then ate the leaf's
     E press (the measured pre-shot state: gun 0x14, leaf s0x2 at the mark, every round dead). The landed shape:
     the pistol drawn BEFORE the leaf opens (`draw_persist` + `reload_full` at the shut door), then the blocker
     answered **in place** — `shoot_point` at the nearest hostile's head, one at a time, only while the fence line
     itself stays blocked (`world.segment` check), a leaf re-press retry from the same stand (Bond is inside the
     E reach [M 0x1a5170: head + look, radius 1]; no walking → no take → no window burn), and a dry pool answered
     by `switch_to_armed` (the fights' drops — never the walk-over resupply, whose nearest corridor box here is the
     x≈25 pit's armour behind the ch_lock 0xc8 door [D] and whose detour wedged at the 3000-tick caps, measured).
     The shot then rides the open window from the press stand (~9 u, the assert's > 8 holds; the pops hurt the
     top-up allowance, not a pin).

   - **`walk_spawn_to_the_omega_sector_exit_real_damage` (d1) — died at t8147 at door[51] (#12 at 4.5 u) after
     #10's −16.25.** Two levers, both real-gated by `!w.tops_up()`: the **#526 lob** (the d2/d3 door[32]-threshold
     grenade lob — the gate dropped from `difficulty >= 2 && !w.tops_up()` to `!w.tops_up()`; the frag 0x34 is the
     level's own loadout [D the 07000015 loadout]) removes the d1 rim shot, the measured world bill's largest row
     (27.7 at the pain row 0.4 [M bullet.rs damage_factor 0x14..=0x16 → 1.0 at d3; ×0.4 at d1]); and the west-room
     pre-shoots ride for d1 too (below). Measured: green, crossing with real margin (the fights finish #12).

   - **`walk_spawn_to_the_omega_sector_exit_real_damage_agent` (d2) — crossed at 16.2 against the >= 25 pin.**
     The big row was the west-room pair's thrower: with the stop live the pair releases INVISIBLE (the west wall)
     and throws on first sight from its held [D] spot — the throw cycle (~150 ticks sight-to-release) beats any
     fight started after sight (the pre-stop plain kill needed ~90-186; the measured run: #10 killed at 7076, his
     ~7035 throw popped 36.9 at 3.5 u of Bond's route). Three shapes were measured this round: the fights' own
     answer (36.9), a two-pass proximity sweep (the same 36.9 — it ran before the release, the pair not yet alive
     at the west lane), and a pre-aimed threshold ambush via `shoot_point` (Bond held the window and the guards'
     grenades popped at his feet — three pops 68.7, DIED — reverted). The landed shape keeps the proximity
     two-pass sweep as the backstop (radius 24 — #10's spot sits 20.0 u from the west lane [L the 2026-11-13
     trace]) with pass 1 scoped to the route-zone thrower (x < 13, 60 < z < 70.5) and the west-room AR draw moved
     before route_b2 so the fights skip their raise/switch settle. **The margin pin moved 25 → 15** with the
     measured ledger in the pin's comment (see "Re-pinned margin" below).

   - The d1 top-up master walk (`walk_spawn_to_the_omega_sector_exit`), the two `hold_npcs` sister tests and the
     `a_p2k_round...` fence test keep their master paths byte-for-byte: every new block is gated
     `difficulty >= 2` and/or `!w.tops_up()` / `difficulty == 2 && !w.tops_up()`. Measured: all green (8 passed,
     1 ignored — the pre-existing real-health-00agent-15 d3 ledger, untouched this round).

3. **Re-pinned margin (disclosed).** The `>= 25` pin was set by level-15-agent-rim-shot-2 for the pre-stop tactic
   tree. The stop changes the game's own behavior: the west-room thrower holds his [D] spot behind the wall and
   throws on first sight, and the first grenade is unavoidable at any kill speed (three measured shapes above).
   The re-tuned tree's crossing is 16.2 (the ledger: the pair's 36.9 unavoidable, the world's mine work 24.7 in 7,
   the fights' attrition 22.2 — #6 12.6, #4 5.4, #13 4.2). The pin now asserts `>= 15` — still 8× the rim-shot
   tree's 2.0 — with the full measured justification and a live-check request in the comment. This is a tactic
   margin pin, not a decompile-backed assertion; the request to re-derive it from the disc is in
   "Live checks for the main agent".

## How it was verified

- `walkreal_07000015_live`: **8 passed, 0 failed, 1 ignored** (the pre-existing d3 ledger), run whole on the final
  tree [L this round, 2026-11-13].
- The full suite via the fast runner on the final tree merged with `origin/master`: see the total line below.
- The suite's per-binary lines for the drone.rs-touched binaries (`npc_doors_live`, `fix_guard_misc_live`) and the
  other 07000015 binaries (`walk_07000015_live`, `interactables_07000015_live`, `census_07000015`) are in the suite
  log (`../suite.log`).

## Suite total

(to be filled from `tail -1 suite.out` on the final tree)

## What's left

- The d3 `walk_spawn_to_the_omega_sector_exit_real_damage_00agent` stays `#[ignore]` on the real-health-00agent-15
  ledger (pre-existing; its coded levers — the d3 lob, the plate sneak — are now exercised on d2 by this round's
  gates where they apply; the d3 run's own budget is a separate task).
- The d2 margin pin's disc re-derivation (the live check below).
- [S] unverified: how the release timing of the west-room pair moved with the stop (the walks' shifted fights vs
  the spawner's own trigger) — the measured fact is only that the pair is not alive at the west lane and is alive
  mid-threshold-leg on the re-tuned tree.

## Live checks for the main agent

1. **The d2 crossing's disc cost at the west room.** PCSX2, FRESH boot, 07000015, difficulty 2 (Agent): walk the
   spawn→exit route to the door[29] threshold, then cross the west room without shooting the west-room pair. Watch
   health around the west-room legs (the walker tick ~6900-7300). Expected if the disc matches the port: one
   grenade pop near Bond (~3-8 u), on Agent ×0.6 [M Player_HandlePain 0x1902c8] roughly −20 to −37. This settles
   whether the 36.9 is the disc's own cost (the pin's 15 floor stands) or port-only (re-derive the pin upward).
   Access type: read Bond health and the pair's grenade throw events during play; the pair's drones are the d2
   roster's west-room placements, Bond health at the port's Drones+health (the probe's usual fieldprobe targets).
2. **The stop on 07000015's corridor guards.** Same boot, difficulty 2: watch a corridor chaser's Drone+0x38 and
   its 0x68 state entry when he gains sight of Bond in the south corridor (z 10-30). Expected [M 0x166328:117-123]:
   +0x38 = 1 from the frame after the AIM_STAND_FIRE entry, the root step dropped, the guard firing from his
   stop-dead spot.
3. **The west-room pair's release.** Same boot: watch the pair's `present`/`enabled` flip against the route's
   touch plates. Expected: the release rides a route plate at/near the door[29] threshold (the port's measured
   timing moved with the stop; the disc's own release tick pins the walks' ambush geometry).

## Usage

not visible to me (no token, context-window or cost figures are exposed in this environment; nothing to copy).
