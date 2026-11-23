# s-bond-hitboxes: Bond's hit boxes follow his stance transitions

Moves the shared random stream: a guard round that crosses Bond during the 12 ticks after a crouch
or stand press can now hit where it missed (or miss where it hit), and the hit's pain draws shift
the stream from there on. No draw is added or removed by construction. None of the binaries run
below went red.

## What the game does [M unless tagged]

- `AnimGetCollData` 0x1c4ac8 calls `psiBuildMatrixPalette` on the player's anim object, then builds
  the 16 `HumanCollisionBones` boxes from that palette. So the boxes come from his current frame.
- In single player every word of the stance sets is one script (0x2b91b0 all 0x6000a00, 0x2b91d8
  all 0x60009ff [D]). Both are 2 frames, non-looping [D]. The idle, walk and run layers
  `AnimSetUpdate` 0x1c0938 appends therefore all hold the stance pose: moving = standing.
- A stance change (`Player_ChangeSubState` 0x190630, case 0 / 4) builds the new set, then
  `PlayerAnimStand2Crouch` 0x190198 / `PlayerAnimCrouch2Stand` 0x190068 add the class-3 transition
  (0x60001dc down, 0x60001dd up) at speed 4. Both are 50 frames [D], so the body is between stances
  for 12 ticks.

## What changed in the port

- `drone::bond_hit_boxes_posed`: the boxes from a given local pose. `Drones::bond_pose` (new) is
  set each tick from `Sim::bond_local_pose`, the resolved `bond_anim` script list that already
  drives the drones' sight bones. The round pass uses it. `None` (MP, dev paths, unposed substates)
  keeps the old stance-script frame 1.
- What a player notices: right after crouching, rounds aimed over a crouched head can still hit the
  body going down. Right after standing up, rounds just under the standing head height miss the
  body still rising.
- Tests (bond_hit_boxes_live): `stance_changes_take_the_transition_frames` fails on the old
  behaviour (checked by handing the round pass no pose) and passes now.
  `walking_keeps_the_stance_boxes` pins the moving case.

## Tests run (all green)

bond_hit_boxes_live 6/6, enemy_accuracy_live 5, start_rounds_07000012_live 1,
start_pair_07000012_live 1, walkreal 01 (7), 05 (17), 0c (6), 12 (4), 14 (9).

## Unproven / not modelled

- Climb, wire, zipline and swim: `Player_ChangeSubState` deletes every script and adds none, so the
  palette keeps the last frame the anim object resolved. The port still gives these the stand pose.
  A live capture would settle what the boxes are there.
- Tick order: the pose is the post-update frame of the same tick (as the sight bones use).
  `Player_Update` 0x193600 runs `AnimObjectUpdate` after the substate handler. Whether drone rounds
  move before or after the player object within a tick is not traced.
- Found, not ported: `AnimSetInit` 0x1c6a48 draws `Rand_Random` once per set init.
  `AnimSetUpdate` draws `Rand_Random` every tick its tagged script count is not 1 (walking, and
  the fades). The port's Bond anim set makes none of these shared-stream draws.
