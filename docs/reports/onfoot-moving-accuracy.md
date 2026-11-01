# onfoot-moving-accuracy — the "Bond is moving" accuracy term on foot

Branch `s/onfoot-moving-accuracy`, from `origin/s/1b-real-health` (b08e6d84) merged with
`origin/master` (bd00a727).

## What the game does (evidence)

`DroneWeap_DoBulletAccuracy` 0x179740 multiplies the hit chance by `DroneFiring_TargetMoving_Accuracy`
(x0.75) when the guard's moving byte `DL+0x1DC` is set (the read at LAB_00179830/00179850).
`DroneWeap_DoOpponentTargetting` 0x179a48:33-41 sets that byte whenever the TARGET object's class
byte `obj+0xFF == 3` — and `Player_Init` 0x1910f8:76 gives the player object class 3, so the test
applies ON FOOT exactly as in zero-g:

```
moving = |(BL+0x10, 0, BL+0x18)| != 0        // the target's move quad, y forced out
```

The quad's lanes are the walk/strafe input vector the player code stores:
- `Player_Update` 0x193600:131-133 zeroes `BL+0x10` every active tick (`obj+0xF4 == 1` → `Vec_Zero`);
- `Player_Move` 0x191dd0 (called by `Player_SSWalk` 0x192310:37 with 1.0, `Player_SSCrouch`
  0x1924c0:58 with 0.75) stores the eased lanes: `BL+0x10 = strafe`, `BL+0x18 = forward · mult`
  [M 0x191dd0:166-170], each eased 0.2/tick toward the stick's 0.075/0.1·a² lanes with a dead-zone
  that snaps released axes to EXACTLY 0.0 [M 0x191dd0:96-99] and a ±0.1 clamp [M :111-141];
- swim overwrites x after the look-dir add [M 0x191dd0:149-165]; zero-g adds the smoothers
  [M Player_ZeroG 0x192ab0:113,135] with the lift riding `+0x14` [M :156] — the lane the magnitude
  never reads (0x179a48:38 forces the y slot 0), so a pure vertical thrust is NOT "moving";
- the climb dismounts push `+0x18` directly [M Player_Climb 0x196310:56,90] and the creep exit adds
  0.05 once [M Player_Creep 0x1969d0:151]; the climb and creep BODIES store nothing, so creeping
  reads as still to the guards.

## What the port did

`crates/nf-game/src/drone.rs` read a change of position (`prev_player` vs `player.pos`) — a port
invention; the game's only position-delta targetting is `BOT_opponentTargetting` 0x125b30, the
multiplayer bots' own `+0x1DD` path (threshold 0.025), never the guards' `+0x1DC`. That read called
a wall-pinned walk still where the game reads MOVING (held input keeps the lanes non-zero).

The fix (commit 93306529): `Bond::move_local: [f32; 2]` mirrors the `BL+0x10` quad's x/z lanes per
substate — zeroed at the `tick_ctx` top [M 0x193600:131-133], stored by the walk/crouch lanes [M
0x191dd0:166-170], the swim look-dir z [M :149-165], the zero-g thruster pair [M 0x192ab0:113,135],
the climb dismount pushes [M 0x196310:56,90] and the grapple hang drift. The snapshot `Player`
struct (drone.rs) carries it to the sim; scene.rs feeds Bond's real quad (the only non-default
site). The guards' moving test is now `player.move_local[0] != 0.0 || player.move_local[1] != 0.0`
— the game's magnitude test on the same lanes. The `+0x1DD` "first moved" 2 s window and its clear
are unchanged (they matched the decompile already). The zero-g thruster-pair read that
origin/s/1b-real-health pinned (1faf0953) and master's merge later dropped (with the zero-g
body-frame look revert, d55c10a3) is restored through the same game-shaped read.

## Tests (all pinned to decompile addresses)

- `player::tests::move_quad_stays_set_pushing_into_a_wall_and_snaps_to_zero_at_rest` — held forward
  into the wall keeps `move_local[1] > 0` while the position pins (the symptom); the dead-zone parks
  the released lanes at exactly [0, 0]; the strafe lane rides x; the crouch walk eases toward
  0.1·0.75.
- `player::tests::zero_g_move_quad_is_the_thruster_pair_and_lift_is_not_read` — forward thrust reads
  moving, the released drift keeps the lanes set (the 0.04 ease has no dead-zone [M 0x192ab0:92-135]),
  and pure lift (riding +0x14) does not.
- `drone::tests::target_moving_reads_the_move_quad_not_a_position_delta` — through `tick_inner` with
  a minimal Data: a non-zero quad reads moving + first-moved EVEN with the position identical to the
  previous tick's (the old read's blind spot); the first-moved window is 120 ticks; a zeroed quad
  clears both bytes.
- Mechanical: every `drone::Player` struct literal gained `move_local: [0.0; 2]` (the still-target
  default), commit c2f7f710; the test-order fix 7312bb13.

## Runs (all quoted totals)

`NF_ISO=<slim ISO> NF_SILENT=1`, release, `--test-threads=1`:

- nf_game lib: **638 passed; 0 failed; 5 ignored**
- walkreal_07000001_live (the fighting courtyard walk): **5 passed; 0 failed**
- walkreal_07000005_live: **11 passed; 0 failed**
- walkreal_07000008_live: **3 passed; 0 failed**
- walkreal_0700000a_live: **7 passed; 0 failed; 2 ignored**
- walkreal_07000011_live: **1 passed; 0 failed**
- walkreal_07000012_live: **1 passed; 0 failed**
- walkreal_0700001b_live: **10 passed; 0 failed; 1 ignored**
- stuck_guard_combat_05_live: **2 passed; 0 failed**
- mp_combat_weapons_live: **11 passed; 0 failed**
- drone_portcheck_live: **5 passed; 0 failed** (fast-suite run)
- drone_weapon_drop_live: **3 passed; 0 failed** (fast-suite run)
- dronefunc_rest_live: **2 passed; 0 failed**
- fix_drone3_live: **5 passed; 0 failed**
- fix_drone4_live: **2 passed; 0 failed**
- census_07000002_drones: **0 passed; 0 failed; 1 ignored**
- walkreal_07000007_live: **2 passed; 0 failed** (fast-suite run)

No test binary name matches `*accuracy*` (checked the full 1054-binary list) — the accuracy term's
pins are the three tests above plus the fighting walks.

The fast suite (`tools/test/suite.py`) built all 1054 binaries and ran 194 of them before it was
stopped for the window; per-binary totals above quote its log where marked. Known lines from its
output:
- `FAILED nf_game::player::tests::move_quad_stays_set_pushing_into_a_wall_and_snaps_to_zero_at_rest`
  — the suite built BEFORE the test-order fix 7312bb13 (the strict strafe sister asserted the
  forward lane 0 while it was still decaying); the committed test passes (the lib run above).
- `CRASH mp_online_failures (exit -9)` — OOM (the sandbox has 3 GB; two big ISO-loading binaries
  ran concurrently). It produced no result line when retried alone either. Not re-verified.
- `FAILED escort_hide_findrun_live::the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes`
  — deterministic alone (2/1 on the other two). This test is a known timing-sensitive flipper (the
  nf-queue record has it failing as "route index 2, want 1" under an unrelated cutscene change
  before). Mechanism: the walk wedges at geometry with input held — under the game's read [M
  0x179a48] that is MOVING (×0.75), under the old position-delta read still (×1.0) — so the hit-roll
  stream differs and the guard-death/hide timing shifts. The re-anchor rule itself is not touched by
  this task.

## What's left

1. Re-run the full fast suite on this final tree (it did not fit the window; 194/1054 ran).
2. The escort_hide_findrun_live re-anchor expectation: re-derive the test's own anchor mirror under
   the game-correct moving read (or re-anchor the walk's wedges), per the mechanism above.
3. mp_online_failures needs a bigger-RAM box to verify (OOM at 3 GB, both in-suite and alone).

## Live checks for the main agent

1. gdbwatch on the player's `BLData+0x10` and `+0x18` (the move quad's x/z lanes) during the 07000001
   courtyard walk: expected non-zero while the stick is held, decaying to exactly 0.0 within ~11
   ticks of release (the dead-zone snap).
2. gdbwatch on a firing guard's `DCVars+0x1DC` (moving) and `+0x1DD` (first moved) during the same
   walk: expected 1 while Bond walks or pushes a wall, and +0x1DD held 2 s from the first moving
   tick; 0 on both after the stand decay.
3. fieldprobe write test: park Bond against a wall holding forward (input held, position pinned) —
   expected `BLData+0x10/+0x18` non-zero and `+0x1DC == 1` (the old port read the position pin as
   still).

## Usage

not visible to me
