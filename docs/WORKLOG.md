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

## copter-death-debris — progress (2026-10-28 ~17:30 UTC)

- [M] the whole chain pinned: the death tail 0x17e390-0x17e3c8 (states 4/5 exempt, 6 on
  0x0700000b/0x07000012), case 6's Explode_Create at 0x17e120-0x17e168 (f12=f13=10.0,
  f14=0x4286cccd=67.4, a3=0x0600052, stack u16=6, t3=NULL), the callee's routing 0x17cbd0,
  Explode_Update's debris arm 0x17c350:34-67 (kind 6 -> MetalHash_132, 3 per passing tick).
- Ported: Copter::update returns the case-6 blast; the walk queues it into pending_blasts with
  the call-site values (attacker usize::MAX-1 for the game's NULL). The debris arm was already
  in (explode-debris-spawn).
- Tests: nf-game lib 584/0 (2 new: the call-site constants, the state-6 dispatch); the live
  tests on 0700000b (death -> kind-6 blast -> 3 metal particles in the 41-tick window, none
  after) and the 07000005 sister (state 4, no blast). Suite on the final tree: running.

## copter-death-debris — final (2026-10-28 17:55 UTC)

- The task body is complete and green: nf-game --lib 584/0, nf-view copter_death_debris_live
  2/0, the report committed. The full suite hit the sandbox cutoff at 17:55 UTC with 94/879
  binaries run (380 passed, 0 failed, one no-network MP crash) — the next message re-runs the
  runner on this unchanged tree for the total line.
