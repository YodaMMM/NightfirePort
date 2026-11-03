# real-health-agent-00agent-11 — round 1 report

Branch `s/real-health-agent-00agent-11` (from origin/master 68298698; `origin/s/real-health-agent-difficulty-01-07`
was already merged into master — no merge needed).

## What's done

**The Agent (difficulty 2) sister is GREEN.** `walkreal_07000011_live` gained the parameterized body
`walk_spawn_to_the_lower_floor_load_point_at(difficulty)` with:

- `walk_spawn_to_the_lower_floor_load_point` — the plain Operative walk (replay parity, untouched path).
- `walk_spawn_to_the_lower_floor_load_point_real_damage` — the existing Operative real-damage walk (unchanged wrapper trio).
- `walk_spawn_to_the_lower_floor_load_point_real_damage_agent` — **new**, difficulty 2, loaded the menu way with
  `load_with_carry_difficulty(&iso, "07000011.bin", false, None, 2)`, asserting `Drones::difficulty == 2` at load
  [M GameState+0x28 0x2a3790 → Drones::difficulty].
- `walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` — **new**, difficulty 3, same load path, **still red** (below).

The difficulty rows the sisters ride are the game's own: Bond's pain scale `Player_HandlePain` 0x1902c8:29-41
(Easy 0.5 → Normal 0.7 → Hard 1.2 on this level group), the guards' hit chance `DroneWeap_DoBulletAccuracy`
0x179740:74-88 (Easy 0.5 → Normal 0.7 → Hard 1.0; under 3.0 u every round hits, :113-115), the guards' round
damage `DroneDamage_*` (damage.rs) and the min-difficulty spawn gate `Drone_Create` 0x136be0:49 (a record's +0x34
mindiff spawns only when GameState+0x40 ≥ it). All cited in the test.

**The keypad #177 gate at Agent/00 Agent.** `Lock_Create` 0x1ae1f8:67-69 calls `Lock_Unlock` at create ONLY while
GameState+0x40 == 1 — on Agent/00 Agent the stairwell keypad #177 ("Lock - KeypadLocked", p1 = code 1, p5 = out
ch 0x12, p6 = the tag [D]) is created LOCKED and ch 0x12 is cold (the interactables_07000011 sister already pins
the cold load). The sisters solve it the player's way, all real input: the aim + one use-key pulse arms the
mini-game (state 3 [M Player_Activate 0x1a5170:158-166 → Lock_Activate 0x1aed00's kind-0 no-key arm]), the four
digit presses run C_KEYPAD_Handler 0x216810 msg 0x4b, the case-4 match fires `Lock_Unlock` [M 0x1ae3d8:262-296]
and the next case-1 tick holds ch 0x12 [M 0x1ae3d8:192-196] — then the stairwell door #192 answers E. The walk
asserts ch 0x12 off at load (d≠1) and on after the solve. The d2 sister walks the whole chain (07000011 spawn →
the descent → the exit → the real 07000012 load at the carried difficulty → tower2 → the shaft exit → the real
0700004A load) green with 0 stuck, 0 falls.

## What's left: the 00 Agent (d3) sister

Red at the east connector (31.1→27.1), death at tick 14455 at (21.64, 32.27, −2.67), 15 hits for 100.0. The chain
is the ammo economy, not a port divergence:

1. The d3 min-difficulty placements grow the load to **37 drones** [L the run census; the gate M 0x136be0:49] and
   the fights answer them with the fresh **7-round pistol** [M 0x1a7ef8:275-281].
2. The 0x0500004-skin stairwell guards land **7.0 a round** at Agent-1.0 accuracy (DroneDamage_Hard included) —
   drone #14 dealt 56.0 in 8 hits (duelled at d 11.9), #15 28.0 in 4 — 84 health gone before the first resupply.
3. The fights' own kill drops rest with their centre ON the floor ray [L drop-stand-height probe 2026-10-22],
   1.09 under Bond's standing pos y — the lane resupply's 0.5 band could not see them. **Fixed this round** with
   the opt-in `lane_floor_drops` band (damage_log.rs + walker_b2.rs resupply, default off; only this walk sets it):
   the NE-landing pass then walked onto a fresh kill drop and armed **clip 30** [L the run log].
4. Still short: the pistol was dry again by the connector and the 0x0500004 pair held the strip. Levers tried and
   reverted this round (each wedged the leg's 6000-tick cap via resupply recursion): the BodyArmour take at
   (23.12, 47.62, 22.88) [D] and `dry_taps = Some(3)`. Untouched levers for the next round: take the armour
   BEFORE the first fight (a pre-leg, not mid-leg), the Spas12 #267 on the lower floor (13.60, 15.55, 5.52 [D] —
   after the crux), and a dry-fight disengage so a dry Bond keeps walking instead of standing in the volley.

No port divergence was found on this walk: every death is the game's own difficulty rows firing through the
faithful pipeline; no health/ammo/channel writes, no teleports; the loads are the game's exit → next-level path.

## Verification (what ran)

| Binary | Result |
| --- | --- |
| `walkreal_07000011_live` (4 tests, incl. the two new sisters) | **3 passed, 1 failed** (the d3 sister, above) |
| `start_corridor_guard_07000011_live` | **8 passed, 0 failed** |
| `nf_game` lib | **656 passed, 0 failed** (5 ignored) |

Shared-helper note (rule 7): `crates/nf-view/tests/common/damage_log.rs` + `walker_b2.rs` changed — the
`lane_floor_drops` flag, default FALSE, read only in the lane resupply band where the false arm reproduces the
old constant exactly (`if lane { floor_band } else { 1.5 }`, floor_band = 0.5 when off). Only the d3 sister sets
it. The full every-`walkreal_*_live` sweep did **not** fit this sandbox window (the level's own three walks
through the changed code ran green with the flag off, plus the corridor-guard battery); run the sweep first on
the next round.

The full 4-crate suite was not rebuilt here (the predecessor branch measured it does not fit this sandbox's
disk); the per-binary runs above are the coverage.

## Live checks for the main agent

1. On a fresh-boot 00 Agent load of 07000011: confirm the stairwell keypad #177 is LOCKED at load (the E at the
   keypad opens the code page) and the code entry writes ch 0x12 — the port's whole d2/d3 keypad path rests on
   [M 0x1ae1f8/0x1aed00/0x1ae3d8] plus this one observable.
2. The 0x0500004-skin stairwell guards' unarmed-class damage at 00 Agent: confirm 7.0 a hit lands from ~3-12 u
   (weapon byte 0) — the death ledger's biggest block rests on the port reproducing that row faithfully.

## Usage

not visible to me (no token, context-window or cost figures are exposed to this session).
