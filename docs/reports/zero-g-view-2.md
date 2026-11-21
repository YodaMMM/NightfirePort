# zero-g-view-2

Follow-up to zero-g-view (the zero-g camera and body-frame look on 0700001b).

## The auto-aim basis in zero-g

`Bond::view_pitch_ps2` returns the forward row's elevation in the zero-g substates and the
walk's `ps2float::mul(pitch, pi/2)` otherwise (walking stays bit-exact). The three scene
readers that built the aim basis from the dead scalar pitch now use it: the per-tick shot
direction (`autoaim::view_basis` before `aim_dir` / `auto_aim_view`), `cursor_for` and
`aim_offset_angles`. The game builds that frame as RotX(-pitch*pi/2) composed onto obj+0x90 with
BL+0x8a8 zeroed every zero-g tick [M Player_GetAimingPoint 0x1a4a98:63-65, Player_ZeroG
0x192ab0:74], so in zero-g it is the object matrix itself. Pinned by
`zero_g_aim_basis_follows_the_forward_row` (the sister reads the old scalar-pitch basis tilted
against a pitched aim).

Unproven: the port's basis is still yaw plus pitch with a horizontal right row; the game's uses
the object matrix rows, which can carry a roll after a pitched turn. They agree on an upright body.

## The 00 Agent walk on 0700001b

The decompile agrees with the branch's composition: Concat(A, B) writes each row of B as
sum_j B[i][j] * A[j] [M 0x1e4a90], so Concat(obj+0x90, R) then Mat_CopyRot back is obj := R*obj.

The walk's aim helpers are not frame-dependent. The console and fight code turns with yaw input
only (`level_turn`) and never feeds `look`; a trace of every `level_turn` call on the failing run
saw no tilt of the forward or up rows (thresholds 0.003 / 1e-4). With yaw only on an upright body
R*obj and obj*R are the same turn and differ only in rounding.

A/B on one binary with an env toggle: the world-frame product passes the walk, the body-frame
fails it, and the camera-pitch change makes no difference either way (identical latch frames).
The runs drift apart by a few hundredths of a unit by frame ~2900 and the real-health fight then
re-rolls: on the branch Bond reaches console 3 at 6378 (past its 6065 read, missile 3 launches)
and dies at 6538. A 60-value pause re-roll sweep (NF_WALK_PAUSE, stops 100-5500, 1-200 ticks)
found no stop that completes the walk. The walk is marked ignored with that reason, like its
Agent sister.

## Tests

- nf-game `zero_g` unit tests: 9 passed.
- walkreal_0700001b_live (whole binary): 17 passed, 2 ignored (Agent and 00 Agent walks).
- zerog_0700001b_live 1 passed, auto_aim_live 3 passed, fix_autoaim_elevator_live 1 passed.
- drake_fight_live 3, fix_world_gravity_live 5, interactables_0700001b_live 9, solar_panel_hits_live 5: all passed.
