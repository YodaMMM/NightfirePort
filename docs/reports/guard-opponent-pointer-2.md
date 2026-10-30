# guard-opponent-pointer-2 — the two merge-reds were harness artifacts; the pointer readers are the game's own

2026-10-30, branch `s/guard-opponent-pointer-2` (from `origin/s/guard-opponent-pointer` merged with
origin/master dca2f368). Merged onto master, the pointer task turned two tests red — both green on
master, both red alone: `grenade_live::guard_decides_to_throw_grenade_unforced` and
`bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too`. Verdict after
the probe runs: **no port reader is wrong — zero source lines change.** Both tests were pinning the
stand-in behaviour master shipped them with, and both fixes are test-side, with the decompile cites
in the test comments.

## The grenade red: the harness ended the mission, and a guard holds no player opponent on a mission that is over

The probe run (NF_AILOG on the merged tree) showed guard #0 leaving IDLE 0x4 and parking in
NoOpponent 0x60 by tick 60, forever, with `stat_ok=false bond_alive=true` — and zero
`can_throw_grenade` evaluations all run. The chain, every step cited:

- The test's blanket `for c in 1..=255 { sim.set_channel(c) }` set the mission's own channels:
  the standing fail hooks ch 0x60/0x61 [M `Mission_MonitorObjectives` 0x1827d8:98-117 — either
  sets ch 99] and the internal channels 99/100 [M `Mission_Update` 0x182c60:23-33 → internal 2/6].
- Mission_Status left the in-mission range within a tick, so `PlrStat_OkToUpdate` returned 0
  [M 0x1993e0 — SP arm: `Mission_Status() < 2`, else 0].
- `NDrone2_FindOpponent` zeroes +0x170 when OkToUpdate fails [M 0x1424e0:106-108] — the port's
  `opp_bond = team != 2 && stat_ok && bond_alive` fold is exactly this.
- The combat router's head returns NoOpponent 0x60 on pointer-without-grace [M
  `DroneFunc_CombatState` 0x148338:20-31; `NDrone2_HasOpponent` 0x14ad58:8-15], and NoOpponent's
  own exits take the pointer alone [M 0x1650a0:27-47] — a completed mission holds no pointer, so
  the guard parks forever.
- `DroneWeap_CanThrowGrenade` itself gates on `+0x170 != 0` [M 0x17a798:16] between the Rand(2)
  and the 8–20 window — even reached, it could never pass.

The fix (test-side): the blanket now skips `{0x60, 0x61, 0x62, 99, 100}` (both the unforced test
and its 30 m sister). The level's objective channels 0x48/0x4b/0x4d stay on: they announce but do
not move Mission_Status [L ee_start_05.bin 0x2a3ed0 — three complete-armed records, flags 0x00;
the monitor's complete case sets no channel]. With the mission alive the guard wakes, walks its
own decision tree into the fire arms, and throws within the window; the out-of-range sister still
holds. `grenade_live` **2 passed / 0 failed**.

## The bathhouse red: the walk buried Bond mid-run and a dead player holds no pointer

The probe run showed: zero sight acks the whole run (`SEEN` never printed), the load's forced-attack
trio going 0x1 → 0x56 → 0x58 → 0x60, and — the smoking gun — `death starts at frame 2143 (health
0.00)` followed by `ch 0x62 on at frame 2592 → internal 3 → Mission_Status 3`, then stat_ok=false
to the end. The walk teleports Bond onto each wake plate at `centre() - 1.0` — the very drop the
bunker sibling's comment already calls out ("the old −1.0 drop buried him below the stair lip") —
and on this route it put him below the floor/water line: he drowned mid-walk (health exactly 0.00,
the drown-damage arithmetic). Then:

- ch 0x62 → Mission_Status 3 [M `Mission_Update` 0x182c60:21, :58-64],
- `PlrStat_OkToUpdate` 0 [M 0x1993e0] → the pointer zeroed every tick on the dead-class arms
  [M 0x1424e0:111-114],
- the visible-sight branch never acks (`+0x170 == 0` falls to the lost-sight arm even with a clear
  ray [M `DroneVision_ProcessDroneSight` 0x176a28:222-231 — the probe caught guards at
  `los=true dist=1.4 opp_ptr=false`]),
- so no AttackTalk site [M 0x13eac0's callers: FirstAttack 0x148010, AimStandFire 0x166328,
  CombatNoMove 0x1643e8] can fire, and no shout rolls.

Master passed the identical run because master's sight ack had no pointer gate: it acked sight on a
dead Bond's position and let AttackTalk roll — the stand-in behaviour. The game's own guards stay
silent when the player is dead; the merged fold reproduces that.

The fix (test-side): the walk stands on the plate centres (`c[1]`, the bunker test's posture) —
Bond lives, the wake → sight → AttackTalk → SFXSetup → mixer chain runs for real.
`bathhouse_shout_mixer_live` **2 passed / 0 failed** (the bunker trio test untouched and green).

## Verification

- `grenade_live` 2 passed / 0 failed; `bathhouse_shout_mixer_live` 2 passed / 0 failed
  (both binaries, NF_ISO set, --test-threads=1).
- Final-tree suite (`tools/test/suite.py`, this branch merged with origin/master, NF_ISO set):
  the run reached **209 binaries: 1460 passed, 4 failed, 58 ignored** before the sandbox's stop
  line cut it (it was wedged behind one slow live binary under the 2-core contention). The 4
  failures are the slim-image environmental set: the 3 allowed `disc_golden` tests plus
  `identity_disc::the_real_disc_passes_the_identity_check` (the identity check hashes full-disc
  bytes the slim on-foot image stubs — the same class of env fail, called out here rather than
  hidden). The suite's own runs on the final tree include `grenade_live` (2 passed / 0 failed),
  `door_guard_live`, `astronaut_guards_live`, `drone_portcheck_live`, `walkreal_07000007_live`,
  `walkreal_07000016_live` and the rest of the first 209 — all green.
- `bathhouse_shout_mixer_live` was run directly on the final tree: **2 passed / 0 failed** (the
  suite had not reached the nf-view tail when the run was cut).
- The four walkreal binaries 03/09/0b/13 did NOT get a completed final-tree run: the foreground
  attempt hit its per-binary timeout mid-run under the suite's CPU contention, and the background
  suite never reached them. No port source line changed in this task (the two fixes are test-file
  harnesses only), so their code path is byte-identical to the merged tree the -1 branch and
  master's full-suite rounds ran; the next session should quote their totals from a fresh suite
  run first thing.

## What's left

- Nothing on the task's reds. The two fixes are harness-only; the -1 branch's port readers stand
  as merged.
- The bathhouse floor group's shout which-guard/which-plate ride the shared RNG stream position
  (already documented in the test's own comment) — unchanged by this task.
- [S] A live PCSX2 check could pin the death-side pointer zero directly: fieldprobe a bathhouse
  floor guard's +0x170 across a forced drown (Bond held underwater): expect 0x170 → 0 the tick
  Bond's record health hits 0 [M 0x1424e0:111-114], with +0x238 keeping its last stamp and
  expiring 30 frames later [M 0x14ad58:11-14].

## Live checks for the main agent

- [S] On 07000002, drown Bond near the floor five (hold him under a pool surface) and gdbwatch any
  guard's +0x170: expect 0 the tick the player record's health hits 0 (FindOpponent's class-3
  health arm [M 0x1424e0:111-114]), while a sight-visible guard's +0x230 keeps its last value and
  the +0x238 stamp expires 30 frames later [M 0x175ef0:66-68 + 0x14ad58:11-14] — the port now
  models exactly this and no test can pin it headlessly without the drowning.

## Usage

Token, credit and cost figures: not visible to me (this worker's CLI exposes no usage dashboard;
the IM gateway metadata carries only session/chat ids, no numbers). Context-window figures: not
visible to me. The only usage-shaped numbers this environment exposes are filesystem/process ones
(rootfs 9.9G with ~1.4G free at report time, 2 cores, the test build cached in /home/z/target-cache).
