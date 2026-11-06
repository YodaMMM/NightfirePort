# stream-batch-4 — the three suspects, the measured fixes, the list that fit

Branch `s/stream-batch-4` cut from `origin/s/stream-batch-3` (659429d32); merged
`origin/s/spawn-pick-walk-05` (7b807f30d), `origin/master` (d0587e2b5) and
`origin/s/void-recovery` (b5e916d84 — landed, merged per the task). Two merge conflicts,
both resolved by composition, not selection:
- `docs/WORKLOG.md` ×2 (add/add): both sides' entries kept, in order.
- `walkreal_07000003_live.rs` (content): stream-batch-3's self-gated door-8 mouth block and
  master's door-walk-03b east-lawn saddle block both stood at the same pre-servant-nav spot;
  both kept (door-8 first, then the saddle), the door-8 block's closing brace rebuilt. The
  03 walkreal binaries did NOT re-run this window (below).

## Suspect 1 — start_corridor d2 lost_sight latch: the decomp answer (does the game do that? NO)

The game never holds a firing guard blind at a VISIBLE Bond. The law, cited:
- The sight pass's unseen branch — `+0x41 = 1`, `+0x274 += 1`, `+0x270 = 0`, sight-flag bit 2
  cleared — runs ONLY when that tick's LOS ray failed [M DroneVision_ProcessDroneSight
  0x176a28:198-203]. The seen branch resets `+0x274` and accumulates `+0x270`, re-setting
  sight flag 4 once `+0x270 >= ReactionTime` [M 0x176a28:216-231; HaveOpponentSight
  0x175ef0:41-44]. `+0x41` has exactly three accessors in the whole decompile: set at
  0x176a28:203, cleared by EVERY shot in `DroneWeap_FireWeapon` [M 0x178f68:262], read as the
  beyond-Tooclose one-forced-miss gate in `DoBulletAccuracy` [M 0x179740:93-99]. The game's
  whole lost-sight fire penalty is ONE forced miss + the 2 s `+0x270` re-ramp — never a stop.
- The combat router: sight flag 4 clear AND `+0x274 > 0xf` → state 0x5c, from three arms
  [M DroneFunc_CombatState 0x148338:67-70/151/235]; 0x5c's UPDATE re-runs the router every
  tick [M NDrone2_DSTATE_CombatNoSight 0x164838:26-35] — with the ray back, the 0x5b/0x59
  arms re-engage and firing resumes.
- So the port's 14000+-tick `lost_sight=true/seen_ticks=0` hold (the unseen branch EVERY tick,
  port mirror drone.rs:18620-22) is the port's own `los` evaluation failing continuously at
  the guards' post-fight spots — the port's collision-world ray (segment_hit_cell /
  line_of_sight) vs the game's `RecurseBoxesSph` 0x1EDAE0 + `ASH_*` BSP walk — COMPOUNDED by
  the port's re-acquire exit parked behind `if false &&` in `drone.rs combat_state` (the
  known 07000001-hall-guard 56k-tick COMBAT_NO_SIGHT freeze class; that exit broke the
  abseil/door-chase/bathhouse guards on 06/09/16 when enabled — it is NOT re-enabled from
  here, per the no-game-code-for-a-walk rule). The decomp answer is recorded in the test's
  diag comment block.

Merged-tree re-run: `start_corridor_guard_07000011_live` **6 passed / 2 failed** — the d2
idle-Bond death still stalls as above, AND a NEW red: `at_difficulty_3_the_guards_walk_
their_disc_owner_points` — p126 stops at (0.07,19.08), 2.4 u short of its owner point
(1.40,16.70) [D; M 0x151b18] outside the 2.0 goal accuracy. The merged streams moved the
GoToGoal approach; the pin is [D]/[M], not [L], and was NOT touched.

## Suspect 2 — 13 real_damage: the "void fall" was a misattribution

`origin/s/void-recovery` merged (its fix is in the tree). Re-run
`walkreal_07000013_live`: plain 1 passed; real_damage + agent still red, but the failure
mode CHANGED: no void death — **y −368 is level 07000013's lobby floor, not a void**: the
walk's own resupply navs arrive at (0.82,−368.34,14.49) and (5.81,−368.34,28.42) and the
level-13 guards stand at y −362..−368 there [L this run's log]. Batch-3's "fall to −368"
was the normal 4A→13 shaft descent into the lobby. The current red is a real-damage
COMBAT death: tick 2110 at (3.26,−368.39,31.72), 23 hits for 72.5 from the lobby guards
(#3 weapon 0x18, #5 weapon 0x19; skins 0x5000006/0x5000004), after entering 13 at
72 hp + 50 armour (the 12 fights' bleed). Damage-budget class — the real-health-13 task's
lane; the d3 leg may carry a measured ignore (00-Agent-only allowance) if the next session
cannot recover the budget with route levers. Not ignored this window.

## Suspect 3 — 0c real_damage: the death-restart latch, named

`walkreal_0700000c_live` real_damage red, but the merged-tree mode is NOT the batch-3
death-at-t8216 and NOT a wire-grab bug. The NF_RF_TRACE read (diagnostics added this
window, opt-in): at the freeze, sim health **0.0**, mission state 3 / internal 5,
`level_exit Some(0x0700000c)`, `level_exit_restart true`, menu closed — i.e. **Bond DIED**
in the yard→ladder fight bleed (83 hits for 69.8 before the latch; the unarmed rushers
#24/#25/#27/#33 lead the ledger), the death flow opened the fail menu and the harness
auto-picked Restart — `fail_choose_restart` latches `level_exit = Some(level_current)`
with `level_exit_restart = true` [M death.rs:284 / 0x216618:20] — and `stop_on_exit`
bailed every route leg (+0 ticks), so the wire-grab loop ran on a held sim and "never
grabbed". The two sisters pass (plain d1, agent d2 — the agent leg is already IGNORED with
its measured ledger). The d1 Operative leg stays red and un-ignored; the levers are the
agent ledger's NEXT LEVERS list (d2-gated stair-head down-shot stand, the #31 display-
pocket stand), plus now the d1 rusher bleed.

## The measured fixes (all green)

- `hold_npcs_live` **2/2** — window 600 → 1800 (the driver's measurements: 0.003 u at 600,
  0.63 at 1200, 1.13 by 1800; held stays 0.0000; comment cites the same Gas_Init draw law).
- `fix_pickups_ammo_live` **4/4** — the box-take assert no longer `skip(sounds_before)`:
  the per-tick hearing list is DRAINED every tick (the file's own rocket comment), so with
  len 1 before and 1 after the skip walked past the new sound; it now searches the whole
  list scoped by the position band. The rocket sister's skip kept (it passes).
- `guard_drops_flashbang_live` **6/6** — the deaths add exactly ONE (6,6): a yard guard's
  stun expires during the frag fuse (batch-3's blast-spot search: recovery dominates every
  lethal offset), `StunGrenadeRecover` 0x163350's get-up → `DSTATE_Stunned_Recover` 0x1630b8
  re-arms ChangeWeapon(6) [M], he dies holding 6 (model ≠ 0 at +0x80) and `DropWeapon`
  0x17a2a0 drops it — the recovered-sidearm drop is game law; the count assert now requires
  exactly [(6,6)].

## Diagnostics added (opt-in, NF_RF_TRACE; no behaviour change with the env unset)

- `walker.rs` + `walker_b2.rs` exit-bail print: mission state/internal/fail_label/restart/
  health under the trace.
- `walkreal0c.rs` wire-grab loop: per-20-tick pos/substate/exit trace under the same env.

## What's left (stream-batch-3's list, unfitted this window)

1. The re-acquire exit re-enable (drone.rs `if false &&`) behind the game's own 0x5c shape,
   with the 06/09/16 walks re-proven — the highest-value fix, needs its own window.
2. The start_corridor d3 owner-point red (2.4 u short) — a route/pin review on the merged
   streams.
3. The 03 walkreal conflict composition re-verification (d1/d2/Agent) — the composed blocks
   compiled but did not run here.
4. The plain-walk re-run over the merged streams (01/06/08/0b/0c/11/12/13/14/15/16),
   mayhew_hold + fix_npc_doors re-runs, the enemy_accuracy re-pin (the harness is
   fixed-stream — the pin matrix may be re-measured against it), the 13 d3 budget levers.
5. The 0c d1 rusher-bleed levers.

## Live checks for the main agent

1. 07000011 d2, idle Bond at the spawn, the two start-corridor guards (#126/#127), past the
   6th hit: fieldprobe per guard `Drone+0x270` (seen_ticks), `+0x274` (unseen ticks), `+0x41`
   (lost_sight), `+0x228 & 4` (sight flag) for 5 s. EXPECTED on the real machine: `+0x274`
   returns to 0 and `+0x228 & 4` re-sets within ReactionTime, the guard re-fires — the port
   instead holds +0x274 climbing with +0x270 = 0. gdbwatch: record which arm of 0x176a28 the
   guards take (the unseen writer 0x176e4c vs the seen path) — if the real game takes the
   SEEN arm at poses where the port's ray fails, the port's collision-world LOS is the
   divergence to chase.
2. 07000013: none — the y −368 lobby attribution came from the port's own nav ledger; a
   screenshot of the 13 lobby after the shaft descent (Bond at ~(3,−368,31)) would confirm
   the floor visually.
3. 0700000c: none — the death-restart latch read is complete from the sim fields.

## Test binaries run this window (whole, --test-threads=1, real ISO)

| binary | result |
| --- | --- |
| nf-game lib (mp::bots + all unit tests) | built clean; guard_drops_flashbang_live 6/6 |
| guard_drops_flashbang_live | 6 passed / 0 failed |
| hold_npcs_live | 2 / 0 |
| fix_pickups_ammo_live | 4 / 0 |
| walkreal_07000013_live | 1 / 2 (real_damage + agent: the lobby combat death, above) |
| walkreal_0700000c_live | 2 / 1 (real_damage: the death-restart latch, above) |
| start_corridor_guard_07000011_live | 6 / 2 (the d2 stall + the NEW d3 owner-point red) |

The full four-crate suite did not fit this window (the merges rebuilt nf-game and the
nf-view binaries are mid-rebuild); every binary whose inputs this task touched ran whole
above. The diagnostics change no sim behaviour with the env unset.

## Usage

not visible to me
