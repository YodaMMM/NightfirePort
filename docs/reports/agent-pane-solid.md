# agent-pane-solid

The ride is on: live glass panes stop Bond and the guards, and a broken pane lets them through.
This moves the shared random stream: every level with kind-2 panes runs a new timeline.

## What the game does [M]

- The per-query mask is HITTEST+0x98. `Collide_PickObj` 0x1eae80:70-85 skips an object whose glist
  flags (obj+0xF0, copied from celglist+0x44 by `Control_SetGList` 0x134960) carry 0x20, plus
  0x1000 when the mask has 0x20, 0x40 when it has 0x8, and 0x10 when it has 0x4.
  `Collide_Intersect` 0x1ebaa8:300-319 then drops triangles by material, depending on the mask:
  0x800 drops surface bit 0x40, 0x1 drops 0xD/0xE, 0x2 drops 0x10, and 0x8 drops pass-flagged hits.
- Bond's walk query carries mask 0 (`Player_Collision` 0x194df0:88). The guard's body query is
  kind 0x800 with mask 0 (`NDrone2_Collision` 0x140e90:62). So no pane is ever filtered out of
  either query. `Drone_CollisionHandler` 0x140968 adds the query's push to obj+0x30.
- The 07000001 drain (`Break_Update` 0x12bfd8:23-34) reads (+0x30 − +0x40)²·20. The push lands
  after the walk copied +0x30 into +0x40 (0x1336f8:212), so a pressed guard drains the pane by
  the push, even when his step came from the animation.

## What changed

- world.rs: `pick_admits` and `surface_kept` hold the two halves of the mask. The dynamic set
  keeps each part's glist flags. `pane_push` is the guard's body query against the panes. The
  masked sight rays (`line_of_sight_masked`/`_hit`, `segment_masked`) apply the object half to
  pane parts. `move_probe_clear` is the guards' step probe and treats no pane as a wall: the
  guards press the glass and take the push.
- drone.rs: `PANE_RIDE = true`. A guard governed by `bounded` takes the pane push in
  `collision_handler` (radius 0.4, or 0.55 for type 0xC). On 07000001 the push drains the pane
  it pressed (`Breaks::drone_pressed`). The glass contact registers its world impact at the
  sheet for passing rounds as well as stopped ones (`Bullet_CollisionHandler` 0x12ce80:162-192).
- The 07000001 walks (test side only). At Agent and 00 Agent the ch-0xF ambush is fought at a
  4.2 radius, exposed guards first. A walled pocket guard is shot from the corridor band's west
  end when Bond has 40 health or more, and the walker never flanks into the pocket
  (`flank_hold_x`). At 00 Agent the walk resupplies before the ambush, and it opens the hall door
  from short of the plane, retreats, and lures #5 before the doorway step.

## Tests (NF_ISO set, --test-threads=1)

- pane_solid_06_live (new): pane #376 on 07000006, the hatch at z −16.33. A crouched Bond is
  stopped short of the plane while the pane is whole, and crawls through once it is killed. 2/0.
  With `PANE_RIDE = false` the first test fails: Bond reaches z 0.31, through the live pane.
- dead_pane_live: both ignores removed, 2/0.
- On the final code: walkreal 01 8/0, 02 3/0 (1 ignored), 06 6/0 (1 ignored), 07 6/0, 08 6/0,
  09 8/0, 0a 13/0 (2 ignored), 12 3/0, 13 3/0, 41 15/0. glass_pass_every_level_live 3/0
  (3 ignored, as before). interact_07000001_live 10/0. dead_pane_live 2/0. pane_solid_06_live 2/0.
- RED: walkreal_07000003_live is 5/1. The Agent real-damage walk dies at tick 106206 at
  (22.9, 12.5) to #5's shotgun. Master passes it, so the cause is the shifted timeline: there is
  no pane at the death spot. It is not fixed here.
- RED: walkreal_07000014_live is 4/5. Its west route steps from the computer room's machine floor
  down to the corridor at x 30.4, z 34.9, which is straight through CompRoom1Window (placement 97:
  kind 2, health 1, x 30.62..31.27, z 30.8..35.68, y 1.34..2.97). The window is solid now, so Bond
  is stopped. Its filter 0xffdf passes only weapon words 0 and 0x20, so the walk's guns cannot
  break it either. The route needs the game's own way round: the south loop the file's notes name
  (nav nodes 94-97/150/591-600), or a breaking weapon. Not done here.

## Unproven / open

- Guard sight through panes. With mask 0xa27 the game's [M] reading would let material-0x10
  glass pass and stop sight on material-0/2 panes such as 07000001's #130 and #136. That
  contradicts the earlier [L 01-doors.trace] note that guard sight crosses the drive's glass, and
  applying it stalled the contact's lead-wait on 07000001. So `line_of_sight` still skips pane
  parts, along with `segment`, the bullet rays and the flight rays. A live check should settle it:
  does a 07000001 guard see Bond through pane #130 (material 2)?
- `NDrone2_DoCollision` 0x149ec8's +0x570 bit 2 (no collision for some animations when the guard
  is not near Bond and not drawn) is unmodelled, so every bounded guard takes the push.
- The kind-0/1 prop rows (vases, crates) are solid in the game by the same `Collide_PickObj` read.
  They stay out of the collision set here, because making them solid re-routes the 07000001 walks.
- The 01 walk changes are walker tactics, not game behaviour.
