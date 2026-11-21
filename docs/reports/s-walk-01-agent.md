# s/walk-01-agent: the 01 Agent real-damage walk

No game-code change on this branch (tests only). Nothing moves the shared random stream.

## The question

The ignored-walks census (s/ignored-walks-1121) left `walkreal_07000001_live`'s
`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` dying at t26967: pocket guard
#17 (SMG 0x12, at (-30.5, 6.3, 14.9)) landed 52.3 in 49 hits from 7.2 u while the walk printed
"no exposed part". Was that the port letting him hit through geometry the game blocks?

## Answer: no. The geometry is clear both ways; the walk's own friendly-fire gate held the trigger

Measured with a temporary probe in the drone bullet pass (removed before commit), on every round
that hit Bond in the fatal fight:

- **His line of fire is clear.** Each round flew from his weapon bone (about (-29.9, 6.41, 15.1))
  to Bond with no world contact: `segment_bullet`, `segment_flight`, the cel-scoped flight trace
  from the round's cel, and the reverse trace from Bond's cel all returned no hit. The round's
  cel and Bond's cel were the same (cel 76), so the cel scope plays no part here.
- **Bond can see him.** On the same ticks, the ray from Bond's eye (y 7.15-7.24) to the centre of
  every one of #17's 16 posed hit boxes entered the box with no world stop (scoped and flat), and
  so did the eye-to-muzzle segment.
- **What blocked the shot:** a trace in the shoot loop showed `exposed_aim` returning a part on
  every check while `shield_block` was true on every check. The escort contact had settled at
  (-30.0, 6.3, 13.6), about 1.3 u from #17's post. The walk's shield gate is a cylinder of
  0.9 u + 0.05 u per unit of range, widened to the bloom cap (0.2464 rad x range) on this walk.
  From the mouth (x -23) that is about 1.7 u, so it covered the contact on every line to the
  pocket, and the trigger never fired while #17's rounds reached Bond past him.

The game side of the guard's fire, as read: `DroneWeap_FireWeapon` 0x178f68 spawns real
`Bullet_init` rounds from the weapon bone toward the target point (+0x1f0) plus the
`DroneWeap_DoBulletAccuracy` 0x179740 miss offset (+0x200). The round then flies its own
collision (`Bullet_update` 0x12e7b0). The port's round does the same through
`segment_flight_contacts_scoped`. `AINetwork_TestRayCels` 0x15ab78 is a cel-reachability check
(`Collide_StraddleCels` from the target's cel must reach the seer's cel). It is not a geometry ray,
and with both sides in cel 76 it does not run [M 0x176f18:65]. Nothing here asks the port to pass
geometry the game blocks.

## What changed (tests only)

- `common/walker.rs`: a new opt-in `Walker::shield_boxes`, off by default, so no other walk changes.
  When it is set, the shield test casts the held gun's own cone against the contact's posed hit
  boxes: the centre ray plus 8 rays on 1.5x the one-tap cone edge, (+0x24 + one tap of +0x98
  bloom) x 0.0014 rad [M Bullet_init 0x12c5a0:139-180], each run to twice the target's range,
  through `damage::ray_box`, the same box test Bond's rounds use. The 0.9 u cylinder stays the
  default.
- `walkreal_07000001_live.rs`:
  - The Agent walk sets `shield_boxes`.
  - In the ch-0xF ambush, a pocket guard Bond can see is fought from the band's west end
    (-25.8, 14.9), about 4.7 u away. That is outside TooClose (3.0) and narrows the cone. Return
    fire is off for that short step.
  - The Agent test is un-ignored. The file also carries the census branch's 01 changes
    (ee2e1df5d: the descent legs' pocket rule and the Agent armour box), which are not on master
    yet. They merge cleanly with that branch.

Both levers are needed. With the shield change alone the walk dies at t17011; with the stance
alone it dies at t42850 (#4, with the contact standing 0.74 u off the line to him).

## Result

- The Agent walk is green at 53.8 health and 8.0 armour at the exit (t63240). It passed twice: alone,
  and again inside the binary's parallel run.
- `walkreal_07000001_live` without `following_mayhew_down_completes_the_level`: 6 passed, 1
  ignored (the 00 Agent walk, untouched).

## Every walkreal_* binary (NF_SILENT=1, -j 8)

See the section appended below once the run finishes.

## Unproven

- The shield's 1.5x cone margin and twice-range reach are a walk choice, not game data. The sim
  itself decides whether a round hits the contact, and the walk completing with the contact alive
  is the check.
- The 00 Agent sister (t2324 at the hall lane stand) was not worked on.
