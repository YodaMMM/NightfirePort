# agent-1b-panel49: the round converges on the crosshair's first hit

Shared random stream: no new draws; the fired round's direction changes for every first-person
weapon that starts at the head / view-datum-0 midpoint, so later hit outcomes (and the draws that
follow them) can move on any level.

## Fixed
`Player_WeaponInitBullet` 0x1a4bf0 traces the head -> aiming point ray (`Collide_RayIntersect`,
mask 0x20a, 0x208 for `+0x6C & 0x800`) before it moves the start to the head / datum-0 midpoint,
and a hit replaces the aiming point [M :154-163]; the round is then re-aimed from its start at
that point [M :226-245]. The mask has no 0x10, so objects in the straddled cels are picked too
[M Collide_Pick 0x1eb320:313-330]. The port aimed at head + aim*256 with no trace, so a round
leaving from below the eye flew level under the crosshair.

On 0700001b from (16.74, -1.84, 4.62) facing -x with the Beam, the port's round flew 0.1 below
the eye, met panel #49's far rim at x 7.37 and broke it. Now it rises toward the crosshair's hit
and first stops on the far wall at (6.958, -1.140, 4.620); the game's is (6.959, -1.136, 4.630)
on the world glist, no panel broken [L audits live-1108 section 5].

Port: `Drones::aim_trace` (world `segment_bullet_scoped`, the drones' posed boxes, the script
objects' glist triangles), used by the scene's fire path.

## Test
`solar_panel_hits_live::the_round_from_the_walkway_reaches_the_far_wall_and_breaks_no_panel`
(replaces the #49-breaks pin). Red on agent-lvl-1b: first hit (7.369, -1.235, 4.531).

## Unproven
- The round's spawn: port (16.198, -1.238, 4.528), game (16.187, -1.160, 4.497). The port's
  datum 0 on the fire tick sits 0.18 below its idle pose; the game's (from its spawn, if its head
  is pos + 0.7) about 0.02 below. Which view-model frame `AnimGetBoneWorldTrans` reads at the
  shot, and the game's head there, need a live read (head and datum 0 at the 0x1a4bf0 call).
- Glass panes and other breakables are not candidates of the port's aim trace.
