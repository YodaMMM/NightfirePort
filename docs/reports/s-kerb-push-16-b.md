# s/kerb-push-16-b: the walks on the toward-zero cylinder push

Trajectory-shifting branch (it carries s/kerb-push-16's push rounding); the random stream itself is untouched.
NOT ready to merge: the rounding makes Bond slide off the 07000014 pad landing and ramp edges where
master holds him (below), and two walks have no working lever yet.

## The 0c deck-south stall: not a step-up fault
The Agent walk's deck-sniper stands each open with the stand's 60-tick strafe. On this tree the first
stand left #19 alive, the second strafe walked Bond off the deck's north edge (z -150.2) and he fell to
the corridor floor (y 1.9) [L the per-tick position trace]. "The deck south" then wedged 3.9 below the
deck. Same class as the deck-south wedges already ledgered on master for other stops.

## Real regression found: Bond slides off faceted edges at rest
Posed standing still on the 07000014 pad landing (x 32.96..33.12, z 35.0..35.7), on its east edge
(33.88, 35.36) and on the armoury ramp edge (-39.95..-39.31, z 31.7..32.1), master holds Bond at every
spot for 1500 ticks; this branch slides him off at 13 of 20 spots crouched and 9 of 20 standing
(falls to y 1.45 / 0.65) [L headless pose probe, both builds, same spots]. Mechanism (cap log): Bond
rests on rounded-edge facets (normals (-0.227, 0.974), (-0.776, 0.631), (0.632, 0.775)); after a
contact tick's push the next tick reads |v| >= 0.55 on every face, so the contact drops, a gravity
tick falls, the next push returns him with a -x component: a 0.0004-per-2-ticks creep down the facet.
Master rests at |v| = 0.5500 with zero push on two opposing facets. Making the capsule ends
(pos + up x arm, VU `vadd` [M 0x194df0:216-218, :391-408]) round toward zero as well changed nothing.
This is what breaks both 14 walks: the drop-band guard is shot from the landing while Bond sinks on it
(arrival y 2.22 vs 2.33), and the tripbomb route slides along the ramp instead of mounting it.

## Walks, per test (cause, fix)
- 16 plain chaser walk: the route now crosses a dropped 0x1a. Fix: counts drops taken, as the arrival test does. Green.
- 16 punch record damage: fight death t4916. Lever 2000/30 -> 500/30. Green.
- 16 chaser real damage: death t16228 (#15 at 2.3 u). Lever 2000/30 -> 3000/60. Green.
- 01 real damage: pocket guard #12 death t9020. Lever 250/30 -> 1000/60. Green.
- 01 real damage Agent: pocket guard #12 death t11252. Lever 1500/45 -> 250/30. Green.
- 01 following Mayhew down: contact off the plate. Lever 2000/30 -> 1500/45. Green.
- 0c real damage Agent: the deck fall above. Lever 2000/60 -> 500/30. Green.
- 08 real damage Agent (new break, master green): cabin fight death after four rockets. Lever 1000/60 -> 1000/60 + 20000/30. Green.
- 0c real damage 00 Agent: NOT fixed. 46 stop variants all die (display ground t3200-4500, deck, wire).
- 14 real damage and 14 tripbomb route (new breaks, master green): NOT re-tuned. Their cause is the slide above.
- 05, 0d: green, no change.

## Open question for the real game
Stand Bond still on the 07000014 pad landing at about (33.04, 2.37, 35.38) for 30 s: does he stay, or
creep west and drop off? Master's port holds him; this branch drops him in about 10 s.
