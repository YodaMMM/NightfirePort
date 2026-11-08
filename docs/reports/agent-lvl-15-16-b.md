# agent-lvl-15-16-b: second audit pass on 07000015 and 07000016

Nothing on this branch moves the shared random stream: the commit is tests and a level-doc fix only.
It does not touch or repeat anything on s/agent-lvl-15-16.

## Level-gated code (every level-id compare in the decompile that reaches 15 or 16)

All of these were checked against the port. Every one is ported and matches:

- `Drone_PostLoad_Init` 0x136938:49-51 sets ch 0x96 = 0 on 16. Nothing on 16 reads or writes
  0x96: no placement param carries it and no script writes it [D the full placement dump +
  the script census]. So this is a no-op on the level, and the port has nothing to add.
- `NDrone2_HitTalkExtra`, `DroneFunc_InjuredTalk`, `DroneFunc_RunForCoverTalk` (drone.rs 4078/4265/4272).
- `DroneFunc_OnInitDeath`: the captain's frag drop 0x34 (drone.rs `drop_grenades`).
- `Mission_Update`: 16 → 0x7300017. `Mission_MonitorObjectives`: ch 0x7b fail.
- `Player_SetWatch` 1.44e6 (watch.rs). `Player_InitWeapon` RamLoad (player.rs).
- `ReadTuningVars` row 0x14-0x16, including the tail's unconditional `DroneDamage_Easy` (tuning.rs).
- `Sound_Ready` banks 0x28/0x2b. `DroneSpawner_Control` range splits (spawners.rs).
- The `<`-range compares that send 15/16 to an else arm (UnderCoverTalk/AimTalk say nothing;
  Drone_ModBulletDamage, DoBulletAccuracy, InitDrops, Door_Update's unlock note): ported.

## Objectives at all three difficulties

New walks reach each level's objective chain on the real path at Agent and 00 Agent, with
the walker's health allowance. Before this, only Operative had an allowance walk (and Agent
on 15 a real-damage walk); the 16 real-damage sisters at 2/3 are ignored because Bond dies.

- `walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_agent` / `_00agent`
- `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_agent` / `_00agent`

All four are green. Two test-side asserts were tied to difficulty 1, and the 00 Agent runs tripped them:
- 16's squad-phase hit cap was 50 × 0.4. It now uses the pick's pain row 0.4/0.6/1.0
  [M ReadTuningVars 0x1d3a80 case 0x7000014-16]. At difficulty 1 it is unchanged.
- 15's #447 fence: at 00 Agent Bond reaches the shot holding a dropped automatic, and its
  burst can take neighbour #449 by bullet before the blast reaches it. The bullet cause is
  allowed only at difficulty 3.

## Doc fix: 16's 0xca/0xcb/0xcc chain

The channel map said these channels were the deaths of #696-698. That was wrong:
- 0xca and 0xcb are the group-6 escorts' death channels (p12).
- They wake the gantry guards #696 and #697 (p3, y -7.63).
- 0xcb is also HurtPoly-LiftCrush #699's gate, so the second escort's death arms the
  lift-foot pad as a 1000-damage volume [D; M Hurt_Update 0x17e5d0:97-99].
- 0xcc comes from 0600090a stream 22. It wakes #698 and the blocking polys.

A probe on master ran this chain the game's way: the escort's kill raised 0xcb, woke #697 and
enabled the volume. No divergence.

## Tests run

walkreal_07000015_live: 8 passed, 1 ignored (the existing 00 Agent real-damage walk).
walkreal_07000016_live: 16 passed, 2 ignored (the existing Agent and 00 Agent real-damage walks).

## Unproven

- That the game kills Bond on the lift-foot pad after the escort dies. This is read from the code
  and the disc data, not watched live.
