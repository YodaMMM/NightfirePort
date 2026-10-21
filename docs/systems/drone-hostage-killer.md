# HostageKiller 0x15c728 / HostageKillerAttack 0x15c940 (states 8/9)

The DTYPE-4 executioner and its attack beat. Handler addresses are `NDrone2_StateFuncs[8]` /
`[9]` [L 0x29b300 table read]. Evidence tags: [M] decompile function+address, [D] disc data,
[L] ELF/snapshot read, [S] suggested.

## Reachability [M/D]

- Entry: Global state's ENTER dispatches the drone's start state `+0x5a2` [M 0x15b830:39-42]; mode 4
  maps to start state 8 (`GetDroneTypeAttackTypeFriend` 0x14cbd8 case 4 [M]); HostageKillerAttack 9
  is entered from Global's timer case 10 on a DTYPE-4 killer with flags 0x100 [M 0x15b830:50-57].
- **Five DTYPE-4 placements exist on the disc, all in 07000012 map 38** (object `oDrone`, skin
  0x05000006, parameter 5 = 0x64) [D census over all 99 level archives / 555 drone placements]. The
  shipped game runs these states on 07000012.

## Behaviour ported (drone.rs)

- `HOSTAGE_KILLER` 8 ENTER: vision bits on, aim anim 9 [M 0x15c728:19-26]; the pairing
  (`SetupHostageKiller` 0x144030 — nearest unpaired hostage, links both) runs once in the `Drones`
  tick; no hostage → Idle 4 [M :27-29]. Tick 3: sight → combat [M :33-39]. Msg 5 (ANIM_DONE): combat
  0x56 (the marker shout rides the shared shout path) [M :40-46]. Timer 10: with a live hostage
  link, the execution anim 0x58 [M :63-68]. Impacts: case 8 (0, allow 0) and the 7/9/0x17/0x18/0x19
  group (allow 1) via `impact_args` + the `on_explosion` raise [M :47-62].
- `HOSTAGE_KILLER_ATTACK` 9 ENTER: with a hostage link, the 1 s execute timer (FRAME_RATE) + aim
  anim 9; without, combat 0x56 [M 0x15c940:22-51]. Timer 10: the execute anim 0x58 [M :86-96].
  Tick/impacts as above [M :55-100].
- Port fields: `Drone.hostage_target` (the killer-side link) + `Drone.killer_searched`, bound in the
  `Drones` tick beside the existing hostage-side `captor` machinery. **[S] deviation:** the game
  binds both halves of +0x2b0 in the one `SetupHostageKiller` call; the port binds the killer side
  here and leaves the tested hostage-side `captor` search unchanged, so the two searches can pick
  different partners.

## Tests / live checks

The port is compile- and regression-covered (nf-game lib 352/0 at the time of writing). A 07000012
real-path live test (walk to map 38's executioners, assert they pair and take state 8, then 9 on the
Global beat) is future work — see the report's live checks.
