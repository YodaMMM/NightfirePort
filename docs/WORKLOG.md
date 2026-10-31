

## level-02-bunker-stick, start (2026-10-31)

Task: Bond hangs in mid-air at (−45.42, 7.1, −22.15) between the two bunker doors on 07000002.
Reproduce on real input, find the cause (door leaf collision/pose, capsule push, the
feet-band rule, a ledge), fix with evidence, pin both arms. Branch cut from origin/master
81da6c47; the fix-02-real-health walkreal changes are the context.

## level-02-bunker-stick, outcome (2026-10-31)

The wedge reproduced on real input and pinned from both arms (bunker_stick_02_live 2/0): the
straight mid-gap steer into the bunker doorway sticks at (−45.42, 6.37, −22.22) — the
reported x/z to the centimetre — with the doors open and forward+strafe held dead (the
capsule push pins him; 900+ ticks). The cause sits in the disc's own doorway data [D]: the
two kind-2 doors #268/#330 (no channels — the E opens) part to TWO slots (north
z −23.93..−22.3, south −21.4..−19.37) with a STATIC PIER between (z −22.3..−21.4, east face
x −45.97, full height [L-port rays]); the threshold under the pier strip is a FLOOR HOLE
(the ray down from y 8 finds nothing above y 0 — the bunker level below is "the floor 5.33
below" the reported hang). The game's own crossing is the NORTH slot — the ambush sensor #389
stands at z −23.35 inside it [D], walkreal's node (−46.1, −23.2) crosses there — and the
fix pin walks exactly that crossing on real input: Bond crosses the plane and LANDS
grounded on the corridor floor band, staying grounded through the sampled tail. The reported
y 7.1 AIRBORNE variant did not reproduce this session (the sidestep phase held the same
grounded pin) — live checks filed. No port code changed: the pier and the hole are the
disc's own geometry [D]; the route is what walked into them. Totals: bunker_stick_02_live
2/0 (NEW), interactables_07000002_live 16/0, walkreal_07000002_live 1/0+1 ignored (the
pre-existing real-health blocker, untouched), walk_07000002_live 2/0, nf-game 588/0/5; the
door/collision binaries all green in the suite state (door_aim_ray_live 2/0, npc_doors_live
6/0+1, fix_npc_doors_live 1/0, door_portal_live 2/0, fix_door_react_path_live 1/0, door_pose_probe
2/0, …). Fast-runner chunk state on the identical crate tree: 490/996 recorded, 2083 passed,
2 failed (the GPU-less wgpu-adapter panics), 142 ignored.
