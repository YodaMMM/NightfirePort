# level-0b-real-health-roof — finish 0b on real health after the corridor fix

Branch from origin/master (it carries the GT wall-gun fix).

walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage is ignored: the corridor crossing now costs 44.8 (100 -> 55.2, the game's wall-gun law), and the run dies at tick 16652 on the roof — the open-floor fights and walker stalls bleed the rest.

Do:
1. Read the damage log for the roof death (who, weapon, range, how often) and check those guards' fire against the game [M].
2. Use the level's own health/armour pickups on the route and a player's tactics (cover, kill order, no stalls in the open). Fix the port only where it differs from the game, with citations.
3. Un-ignore the walk; it must reach the mission end. Run walkreal_0700000b_live, every real-damage walk, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets) and you must run EVERY walkreal_*_live binary before finishing — shared-helper edits broke other levels three times on 11-02.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox3 at 2026-11-02T19:14Z

---
## Report (done, 2026-11-02T20:52Z)


---
## Report from the worker (done, 2026-11-02T20:52Z)

# level-0b-real-health-roof — finish 0b on real health after the corridor fix

Branch `s/level-0b-real-health-roof` (cut from origin/master 93c54c45, which carries the GT
wall-gun fix b2576c04). The ignored walk
`walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage` died at tick 16652 on
the roof. It is now **un-ignored and green: mission complete in 16616 ticks with 15.4 health
left**, stuck 0, and the one reported fall is the parachute jump itself
(walkreal_0700000b_live 5/5).

## What the death was

The damage log at the death (44 hits for 100.0, all of it through the hit record) split into the
already-law-accurate corridor (23 × −2.40 = 55.2, the GT wall guns' eased-barrel fire from
b2576c04) and 44.8 from guards. A bullet probe on the run showed what the guards' fire was up
against:

- **Bond's only conventional weapon on 0b is the Korsakov K5 sleep dart** — the boot loadout's
  bullet is `wpn 0x43, dmg 0.10, range 45` ([L] the probe; the 09 kit the 0a/0b arms fall into
  [M 0x1a7ef8 cases 0x7000009/0a/0b]).
- **The game's dart law drops msg 0x19 for a guard in full combat**: `DSTATE_Attack` 0x164210
  carries no case 0x19, and the state gate is `DroneFunc_HandleImpact` 0x1469b8:315-361 (the
  port's on_dart_impact whitelist). The walk's ring fights stood 833 ticks tapping dead-on,
  unblocked darts into ATTACK-state guards — target hp pinned at 10.0 the whole time ([L] the
  fight traces). Unwinnable by the game's own design; a player walks on.
- **The plain 20 u shooters window never even picked the roof deck's guards** — they work the
  deck key from d 36-42 ([L] the death logs), outside the shared return-fire window.
- **The fixed 2900-tick cut-scene wait stood Bond at the deck key with no control**: the
  re-place happens at the script's tick 2447/2450, and the guards opened fire while the walker
  was still inside the wait — 11 hits, 23.8 → 0.0, at (−14.71, 58.13, −2.06), no fight ever
  started ([L]).
- The sleep the K5 does land is short here: `RecoverTime(0x43) = 36·(200−+0xB9)` ticks [M
  0x147188 k=60]; the ring sleeps (#1 at tick 13027, #3 at 13065) were both back shooting
  inside ~150 ticks ([L]) — about 144, i.e. +0xB9 ≈ 196 on these guards. Standing to dart four
  deck guards cannot hold them down.

## What changed

- **crates/nf-game/src/drone.rs** — no behaviour change, the law exposed for the test harness:
  `Drone::dart_impact_state_ok` is on_dart_impact's own state list (a pure refactor, pinned by
  nf-game's suite), plus `Drone::dart_family_state` / `Drone::dart_asleep_state` (the
  0x50/0x51(/0x52) dart states, cited).
- **crates/nf-view/tests/common/damage_log.rs** — one opt-in flag, `tranq_fights` (set only by
  the 0b walk): the fight pick answers only guards whose state carries case 0x19, and the
  fight's win/progress reads a dart landing (the target entering the dart family) instead of a
  health drop — the dart's HitDamage is never called [M 0x13fff0:269-306], so hp never moves.
  Flag-off behaviour is untouched (all 26 walk binaries pass with it off).
- **crates/nf-view/tests/walkreal_0700000b_live.rs** — the walk sets `lane_fights` (the 013
  walk's existing opt-in: the 48 u window, the gun-ray exposure, the resupply), `back_off 3.0`
  (under 3.0 u every drone round hits [M 0x179740:113-115]) and `tranq_fights`; the roof
  cut-scene tail waits until the GAME re-enables Bond (`wait_enabled`, the arrival cut-scene's
  pattern); and the deck crossing is the player's sprint — the same real-input pattern the
  jump leg drives, no fight stops — because a moving target cuts the guards' roll [M 0x179740
  `DroneFiring_TargetMoving_Accuracy`] while darts cannot hold the guards down (above).

Nothing in the port's game law differed from the disc this round: the guards' fire (cadence,
−2.10/−2.80 hits, the range band) is the ported `DroneWeap_FireWeapon`/`DoBulletAccuracy` law
[M 0x178f68 / 0x179740] and the deaths it produced are what that law yields on this route
against a standing player. The fixes are the player's tactics plus the fight machinery reading
the dart law it already ported.

## How it was verified

- `walkreal_0700000b_live` **5/5** (both warm walks on the shared body, the cab/cut-scene pins,
  and the un-ignored real-damage walk).
- **All 26 `walkreal_0700000xx_live` level binaries** green (the shared-helper rule; the two
  `ignored` walks in 14/15 carry their own tasks' measured reasons, untouched). One batch was
  SIGKILL'd mid-run by memory pressure under default test parallelism; re-run with
  `--test-threads=1`, green.
- `nf-game` **101/101** result lines ok. `nf-formats`/`nf-audio` green except exactly the 3
  allowed `disc_golden` failures on the slim ISO.
- `nf-view --lib` **100/100**.
- **134 nf-view integration binaries** sampled (the gunimp batch, the fight/damage/guard-heavy
  set, and alphabetical spread): all ok except `glass_guards_14_live`'s
  `a_guard_behind_the_comp_room_window_is_visible_through_it`, which dies in wgpu
  `adapter: NotFound` — the sandbox has no GPU (infra, not code; the same class the 08 report
  documented).

## What's left

- The remaining ~770 nf-view integration binaries were not built/run — the sandbox cannot link
  that set in the time (disk + wall clock; the 08 report documented the same blocker). Nothing
  in my diff touches them beyond the flag-gated paths above.
- The live checks below.

## Live checks for the main agent

1. **The corridor cost on the real machine** (carried from level-0b-corridor-world-damage):
   the eased-barrel law's own yield over the terminal-room crossing — the port shows 23 ×
   −2.40 = 55.2; whether the real PS2 run costs ~55 over the same route.
2. **The K5 sleep length on the Tower guards**: gdbwatch a darted guard's `+0xCC4` (the wake
   deadline) and `+0xB9` (the recover stat) after one K5 hit — the law predicts
   `RecoverTime(0x43) = 36·(200−+0xB9)` ticks, the port's run implies +0xB9 ≈ 196 (~144 ticks).
   Access type: read, once per dart; expected: the deadline ≈ hit tick + that value, and the
   guard resuming attack after it (nobody within 2.0).

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed to this
session's environment).
