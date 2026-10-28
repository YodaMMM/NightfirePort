# WORKLOG

## copter-death-debris (2026-10-28, branch s/copter-death-debris)

Task: the copter's death blast throws its metal debris. explode-debris-spawn
ported Explode_Update 0x17c350's debris arm but the car/copter death blasts
(debris kind 6 [M 0x131e70, 0x17e128]) are not plumbed — no copter death blast
call site in the port. On The Exchange (07000005 Part 1 / 07000008 Part 4) the
copter can be destroyed. Port the copter's death blast as the game creates it
(Explode_Create with the copter's kind-6 debris arg, its script, radius,
position [M 0x17e128 and its caller]) so Explode_Update throws the
MetalHash_132 debris. Tests: destroying the copter creates the blast with kind
6 [M]; debris objects follow the game's cadence [M 0x17c350:34-67].

Plan:
- Fast runner started in the background in the first minutes.
- [M] read Explode_Create/Explode_Update (0x17c350) and the 0x17e128 call
  site; find the copter's death-blast caller (0x131e70 family) and its args.
- [D] level 07000005/07000008 copter object and its kill path.
- Port the blast call site; tests through the real input path (destroying the
  copter -> blast kind 6 -> debris cadence).
- Full suite on the final tree; push; report; finish steps.
