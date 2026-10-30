# level-05-interactables — the report

## What the task was

With `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable
placement, driven the way a player drives it, each asserting the game's effect with a sister),
read what the existing interact_/walk tests for 07000005 already cover, census every
player-touchable placement, add the missing kinds as
`crates/nf-view/tests/interactables_07000005_live.rs`, and fix every port gap the tests expose.

## What was already covered (read, not rewritten)

The census (`docs/levels/07000005.md` §7, the 363-placement main-map table) and the 11 green
tests of `interact_07000005_live.rs` (2026-10-17) plus the walk/walkreal/fix_* suites already
drove: the static creations census, the approach TouchOnce plates + the OR mux, the doors
(unlocked opens / locked refuse), the ladder, the pickups kinds 3 and 6, the searchlight sweep
(+ the [L] orphan light), the SC3 plate → the gated copter, the sound trigger 368, the ch-0x22
spawner, the grapple aim, the valley hurt fall, the wire grab + ride (`walkreal`), the creepwall
hug + its camera/prompt (`fix_wallhug_*`), the third icons / context hints, the Touch #318 +
mux #316 truck chain (`fix_truck_live`), and the exits (`level_carry_live`).

## What this task added (`crates/nf-view/tests/interactables_07000005_live.rs`, 3/3 green)

1. **Pickups 0xF0 kind 1** — the two 9mm boxes #322/#323 (49.24/48.06, 49.00, −60.52/−59.25
   [D audit rows: type 1, item 6, amount 0x1E = 30]). The walk-over take credits the type-1
   pool to its 70 cap and consumes the box (respawn 0 → deleted [M `Pickup_Handler` 0x18e650
   case 1 → `Player_EquipAmmo` 0x1a8d18; :338-348; L the weapon_data pool cap]). The full-pool
   walk-over is REFUSED and the box stays — the same case-1 arm returns nothing and the
   consumption tail runs only on a take [M 0x18e650:142-144, 338-348]. (A freed-pool re-take
   leg — fire until the reload frees a round — is the same case-1 arm and is left to a
   follow-up; the reload bookkeeping did not land in this session's window.)
2. **Touch 0xEB walk-in plates + MuxIn 0xEC (AND) + MuxOrIn 0xEF (OR)** — the ledge cut-scene
   chain. The west-wall guards #65/#68 wait ch 0x22 [D p3 = 0x22]; after the approach plate
   raises it (the real walk), their band patrol latches the mask-16 line (#57 → ch 84). The
   player's mask-8 line (#60, the window ledge) raises 85; mux #63 (AND 84, 85) closes ch 161
   and OR #270 raises ch 164 = 0xA4 (the alarm), and SoundTrigger #83 plays sample 329 [D the
   audit's trigger rows; M the mask gate + the multiplexors in triggers.rs]. Sister: a fresh
   load keeps the whole chain dark.
3. **The lock-gated TouchOnce 0xEA + the ScriptPlayer 0xD9 it starts** — #301
   "WallHug_touchplate" (out 0x43 = 67 = SP #298's watch [D p6], lock 0xB3 = 179 [D p7]) fires
   on the walk-over while the lock stands and starts the ledge NIS 0x06000994 [D the audit's SP
   row]; after TouchOnce #306 (out 0x44 = 68) arms OR #304 → ch 179, the same crossing is
   refused — `Trigger_Locked` 0x1b2fc8:24-29 (locked = the lock channel non-zero AND on) [M].
   Sister legs: fresh-load ch 67/179 off; the locked crossing keeps 67 off.

## Port gaps found

**None.** All three kinds behaved per the decompile on the first drive; the iterations above
were test-side (teleport heights, arrival radii, the refuse law). No source code changed — the
branch's diff is the new test file + docs.

## Transit stand-ins (documented per site, the operations are real input)

- The yard drop for the ammo boxes: the walkreal route ends at the zip door's E (the exit-chain
  test); no walked route to the yard east is established. The yard floor is
  Room_Room_Bott_I1's 48.88 under the resting boxes [D coll-scan].
- The window ledge drop: the walk there is the west-lane chain (the wire ride + the landing
  legs — walkreal's documented route) and the direct band walk is walkreal's [D]-backed seal
  negative. The ledge surface is the ledge icon object's plane at y 37.13 [D coll-scan].
- The #301 drop: the road's own south slope (floor_ny 0.80 [D]) slides a stand off the
  Room_Road_B flat quad (y 30.85-31.13, x 23.54-25.96, z 71.25-72.32 [D coll-scan]); the toucher
  pass runs every settle tick, so the crossing through the box IS the walk-over.
- The valley walk to #306: from the hurt test's documented valley floor east stand, the
  diagonal steer to the box's centre (the straight −x lane at z 57.5 is walled).

## How it was verified

- `interactables_07000005_live`: **3 passed, 0 failed** (6.5 s, `--test-threads=1`).
- The fast runner (`tools/test/suite.py`) batches on the final tree (this branch = master
  61c7e3ea + this task's commits):
  - batch A (`census_07000005, interact_07000005_live, interactables_07000005_live,
    walk_07000005_live`): **16 passed, 0 failed, 1 ignored** (3 min);
  - batch B (`fix_truck_live, mayhew_handover_live, mayhew_hold_live,
    mayhew_end_transition_live, gate_guard_truck_live`): **10 passed, 0 failed** (1 min);
  - batch C (`walkreal_07000005_live`): **10 passed, 0 failed** (1 min);
  - batch D (`interactables_07000004_live, interactables_0700000b_live,
    interactables_07000002_live, interactables_0700001b_live, interactables_07000041_live,
    walkreal_07000004_live, walkreal_0700000b_live`): **57 passed, 0 failed, 3 ignored** (1 min).
  - **Total: 93 passed, 0 failed, 4 ignored.** No FAILED/TIMEOUT/CRASH lines.
- **What did NOT run**: the remaining ~850 test binaries of the full four-crate suite did not
  fit the session window (the sandbox kills background jobs between tool calls, so the suite
  ran in foreground `--only` batches; all 947 binaries built — `build ok: 947 test binaries`).
  The touched crate is nf-view only (a test file) and no `src/` file changed, so the regression
  surface is the batches above. The next session should run the full suite.

## What's left

- The freed-pool ammo re-take leg (fire → reload → the standing take) — the same [M] case-1
  arm, a test-completeness item only.
- The full-suite run on this tree (the batches above cover the touched level family + the
  adjacent interactables families).
- The standing §7 open items are unchanged (the wall-hug window ledge's plunge-shell false kill
  — the port's plunge mesh catches the ledge walk at y ≈ 35.3; the TouchOnce #36 reveal plate's
  class-5 contact path note).

## Live checks for the main agent

- None required by this task (no visual or un-decompilable behaviour was pinned). If convenient:
  a fresh-boot walk of the ledge cut-scene corridor (the window ledge x −35, z −22..−28) with a
  watch on `switch_channels[84]/[85]/[161]/[164]` would confirm the guard-line latch order the
  test infers from the masks [D mask 0x18 vs 0x8 + M the mask gate].

## Usage

not visible to me — this environment exposes no token, credit or context-window figures
(input/output tokens, context used, cost) for the session; nothing to copy exactly.
