# level-06-rook-door — the real-path doorway stall found, cause proven, fix validated and held

Branch `s/level-06-rook-door` (from origin/master 3a0cb26e). Changed: `crates/nf-view/tests/
common/walker.rs` (an env-gated `NF_DRONE_WATCH` per-tick drone logger, additive), new
`crates/nf-view/tests/rook06_realpath_live.rs` (the evidence walk), `docs/systems/rook-door-06.md`
(the findings section). **No port code changed on this branch** — the cause is proven and the fix
validated, but it conflicts with a master-green 01 pin (below), so the change rides the report and
the next round lands it after a 01 re-derivation.

## What was done

**Step 1 (drive, log, find) — done.** The player's real route on 07000006 was walked with real
inputs only (the proven walkreal legs; no teleports after the spawn), logging every drone's
state, position, route and door binding per tick. The stall reproduced exactly: the **switch-16
guard pair beside door #47** (the courtyard→castle E-door at (36.50, 40.00, −16.25)) wakes on the
interior step triggers; one of them, after a 5000-tick normal-block fight, takes RUN_FOR_COVER
0x8c whose cover route crosses the path 7 door-node pair — his own DSTATE_OpenDoor 0x81 opens
door #47 (the ported drone door path works) — and he completes the cover run INSIDE the doorway.
There his ATTACK re-entry hit the port's ATTACK-ENTER `ChangeToAttackMode` flush: his mode
flipped 0 → 25 (0x19, the alert sniper — stationary, sight/combat range 1000) and he parked in
SniperAim 0x2a at (35.47, 41.00, −16.93), bound to the door, the door held at 0x4, for the whole
1800-tick watch. An NPC parked in the player's doorway — the playtest symptom. The driver's
path-10 walk and the ch-185 escort's path-12 walk are clean end-to-end (no door involvement),
confirming the earlier rounds' finding.

**Step 2 (the cause, [M]) — proven.** `NDrone2_ChangeToAttackMode` 0x14a648 has exactly five call
sites (the `jal` census of ACTION.ELF): 0x14ce68 (GetDroneTypeAttackTypeFriend's kind-0xb arm),
0x14d45c (DoTypeSettingsOLD — 06's guards are NEW placements), 0x15da54 (CivilianMission's
mission arrival), 0x15e35c (EnemyMission's arrival — gated OFF on 0x0700006 itself) and 0x165948
(GoToGoalPosition). `NDrone2_DSTATE_Attack` 0x164210 and `DroneFunc_FirstAttack` 0x148010 are
NOT callers — the port's ATTACK-ENTER arm is the stall's whole cause. The same read exposed the
missing +0x5a4 lifecycle: GetDroneTypeAttackTypeFriend's tail (0x14cea8) writes the attack-fold
result INTO +0x5a4, so a combat guard carries no flushable pending alt at all; the port never
writes the fold back, which is why the wrong arm could fire with a creation-time alt pending.

**The fix — validated, held.** Removing the ATTACK-ENTER arm (flush moved into a helper at its
real EnemyMission site; the CivilianMission arrival arm also ported, then reverted as unproven
for this round) was measured on the exact walk: the guard keeps mode 0, fights through the
doorway and pursues Bond, door #47 ends at the shut idle 0x2. Green: nf-game 887 unit tests,
rook_door_06_live/walk_live, fix_06_driver/fire/alarm, escort_follow_07000001, fix_mission,
rook06_realpath (the new test's assertions all pass on the fixed tree). Red:
`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` (master green — the
contact dies at round 10, state 0x47, with the fix): the 01 walk's balance depends on the
non-decompile flush parking the corridor guards as stationary snipers. Per the never-weaken rule
the fix is NOT shipped red; the evidence walk is committed `#[ignore]`d with the full chain in
the ignore reason. Landing order for the next round: identify 01's contact-killer guard (his
dtype/kind/alt_state), check whether the +0x5a4 fold-write alone resolves both walks, then remove
the arm; run the 01 walkreal binary and this test un-ignored together.

## Verification

- `rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway` — the full real
  path with the three decompile-backed pins (mode stays 0; the guard crosses to the courtyard
  side of the door plane; the doorway ends clear and the door at shut-idle 0x2): green on the
  fixed tree, and every pin demonstrably fails on master's tree (the pre-fix watch logs: mode 25,
  state 0x2a at the door plane, door 0x4, 1800 ticks).
- `cargo test --offline --release -p nf-game`: 887 passed, 0 failed (the AI unit surface).
- The neighbor binaries, all green: rook_door_06_live, rook_door_06_walk_live, rook_absent_06_live,
  fix_06_driver_live, fix_06_fire_live, fix_06_alarm_live, escort_follow_07000001_live,
  fix_mission_live, fix_npc_doors_live (its one test is #[ignore]d on master — harness note from
  escort-door-harness; unchanged here), rook_sp_06_live.
- The fast runner could not be used as designed: this sandbox kills every background process
  between tool calls (verified twice with setsid), so the suite ran as foreground slices of
  directly-invoked test binaries instead (the list above). The full four-crate suite did not fit
  the window on this round's tree; the touched binaries are all green as listed, and the only
  known red anywhere is walkreal_07000001_live **with the unlanded fix applied** — master itself
  is untouched by this branch.

## What's left

1. The 01 re-derivation (the landing gate): which guard kills the contact, his dtype/kind and
   alt_state, and whether writing the +0x5a4 fold back in `start_state` ([M 0x14cbd8 caseD_3])
   resolves both walks at once; then remove the ATTACK-ENTER arm and un-ignore
   rook06_realpath_live.
2. The flush's remaining game sites (GoToGoalPosition 0x165948, the kind-0xb start arm) are still
   unported; CivilianMission's arrival arm was ported and reverted here as unproven-by-test.
3. The full four-crate suite on the landed tree.

## Usage

Task-system token/context figures are not visible to me this session (no token, cost or
context-window readouts are exposed in this environment). Session wall-clock: started 00:17 UTC;
sandbox expires ~02:16 UTC. The repo, work/ and the ISO were already present; one branch create +
push, two full nf-game/nf-view rebuilds plus the bisection runs, no downloads.
