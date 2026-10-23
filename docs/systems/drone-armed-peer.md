# The drone armed-peer rules (NearArmedDrone / NotNearArmedDrone)

The "armed peer" scan behind the escort's cower (AllyLead's LeadHide family) and the hostage
states. The port's snapshot computes `Drone::armed_peer_dist` (drone.rs, the shared pass before
the per-drone walk) from it; every consumer branches on ≤ 10.0 (hide enter), > 14.0 (hide exit)
or ≤ 20.0 (the hostage radii), so the port caps the scan at 20.0 — a peer beyond 20 can never
flip any of those comparisons.

## The game's predicate

`NDrone2_NearArmedDrone(DCVars*, float radius, char flag)` 0x177420 walks the global obj list
and accepts a candidate only when ALL of these hold:

1. **The candidate's actor is a live started drone** [M 0x177420:74-89]: actor+0xe0 → drone data;
   its +0x4F8 flags need `(flags & 0x600) == 0` (not down/stunned), `(flags & 0x100) != 0`
   (started), health (+0xAC) > 0.0; obj+0xFF != 0x11 and obj+0xFE bit 0 clear (not deleted);
   obj+0x4F8 & 0x4200 == 0. The port's `team == 1 && started && !corpse && enabled && health >
   0.0` covers the shipped levels' cases (the 0x600 stun bits the port does not model are the
   known gap, tagged in the report).
2. **The weapon filter** [M 0x177420:89-104, disasm 0x177538-0x177578]: the candidate's weapon
   byte (obj+0xDC → +0x62) must be in **[2..0x46] ∪ [0x6a..0x71]** — the same predicate as
   `NDrone2_ObjectIsArmed` 0x178768 (`Drone::obj_armed`). 0 and 1 (unarmed/holstered-fists),
   0x47 and ≥ 0x72 are not peers. The decompiler C reads signed; the disassembly is `sltiu`
   on `(b − 0x6a) <u 8` ([0x6a..0x71] armed) + `slti (s8)b < 2` ([0..1] out) + the 0x47 branch
   at 0x177564 (out) — the C's `7 < bVar1 - 0x6a` is unsigned.
3. **The class byte** [M 0x177420 disasm 0x177580]: obj+0x44 == 1 (the team-1 guard class).
4. **The radius**: `Vec_Dist3D(drone_obj+0x30, candidate_obj+0x30) <= radius` [M 0x177420:111].
5. **The sight gate** [M 0x177420:113-128]: `NDrone2_GetHeadPos` on the seeker, then
   `DroneVision_CanSeeObjectFrom(drone, candidate, 5, head_pos, drone_cel, 0)` —
   0x176f18 builds the candidate's bone-5 (head) position and runs a head-to-head
   `Collide_LineOfSight` whose mask is the caller's flags | **0xa27** (NearArmedDrone passes
   flags 0 [M 0x177420:124]) behind an `AINetwork_TestRayCels` AI-cel pretest (the two cels'
   AI ray must be clear first; equal cels skip the pretest) [M 0x176f18:56-105].
   The port's stand-in is the head-to-head `world.segment_masked` (the 0xa27 mask — see the
   next section; world.rs). The AI-cel pretest is subsumed by the world segment test.
6. **The flag-1 prop gate** [M 0x177420:57-61]: with `flag != 0`, the SEEKER's own behaviour
   property 0x50 must be set (`behaviour_util_getProperty` 0x1c7d98 on Drone+0x4D8) — the
   callers pass flag=1 at the four AllyLead enter sites [M 0x15e698:30, 0x15eda8:30, 0x15ea28:30,
   0x15f120:26] and flag=0 at the hostage sites [M 0x15ce60:29, 0x15cfe0:24]. The port gates the
   four Lead hide-enters on `self.prop(d, 0x50)`; the hostage sites and the hide exit carry no
   prop gate.

`NDrone2_NotNearArmedDrone(DCVars*, float radius)` 0x177640 (the hide-exit read, radius 14.0
[M 0x15ec08:30]) runs the same 1-5 filters with NO prop gate, plus one hysteresis wrinkle: the
peer latched at Drone+0x2B8 by the last NearArmedDrone hit (0x177420:126 writes the found peer
there) blocks the exit by DISTANCE ALONE while within the radius, seen or not [M 0x177640:98-100].
The port does not model the +0x2B8 latch (each of its snapshots recomputes the min over visible
peers); the walkreal 01 walk does not hit the difference — the balcony guard who parks 13.7 from
the contact is never VISIBLE from the ground, so the sight gate alone clears the exit.

## The 0xa27 sight mask

Every drone-vision `Collide_LineOfSight` runs `Collide_RayIntersect` with HITTEST+0x98 = the
caller's flags | 0xa27 [M `DroneVision_CanSeeObjectFrom` 0x176f18 — the `ori t1, t3, 0xa27` /
`li t1, 0xa27` immediates at 0x176fec and 0x1770b0 sit directly before both calls;
`Collide_LineOfSight` 0x1edf50 forwards arg6 | 4 as `Collide_RayIntersect` arg7 → HITTEST+0x98
(0xa27 already carries bit 2), and 0x10 as arg8]. The mask's set bits exclude, before a hit is
reported:

- **bit 0x20**: the 0x1000-flagged nodes (the port's missile cage) [M `Collide_Pick` 0x1eb320:149-152];
- **bit 0x800**: any surface whose material byte carries 0x40 [M `Collide_Intersect` 0x1ebaa8:301];
- **bit 0x1**: materials 0xD and 0xE (`(mat & 0x3f) - 0xd < 2`) [M 0x1ebaa8:311];
- **bit 0x2**: material 0x10 [M 0x1ebaa8:314];

and still tests objects (bit 0x10 clear → `Collide_PickObj` runs [M 0x1eb320:314]). Bit 0x8 of
the mask (the NODE pass-flag drop the bullet queries ride, disasm 0x1ebff0) is NOT in 0xa27 —
the sight ray keeps node-pass-flagged hits and drops only the 0x40 MATERIAL byte. The port's
`CollisionWorld::segment_masked` (world.rs) carries exactly these exclusions; the raw `segment`
is strictly stricter, which is what blocked the 01 escort's stairwell peer.

## Why the sight gate matters on 07000001

The meet scene wakes four ch-0xE hall guards; one (skin 0x05000036) ends the walk parked on the
balcony above the west stairwell — head-to-head LOS from the escort's ground position is blocked
(the ray hits the balcony slab). Without the sight gate the port counted him forever: the escort
hiding at the stairwell head could not reach the > 14.0 exit (the guard sits at 3D ≈ 3.9-13.7
through the descent) and never finished his mission route to the LoadPointActivator plate. With
the gate he stops being a peer the moment the line blocks, exactly as the game reads it.

The other side of the same coin: while the escort descends the stairwell, a LIVING armed
guard on the landing above (guard 2, ≈ 4 from the contact) must STILL count — the hide's run-to
points walk the contact west to the AND plate (placement #180, Touch out 0x10, filter skin 0x4F
[D triggers]) only while a peer is within 10.0. The raw `segment` blocked that head-to-head ray
too (the stairwell geometry), so the hardened scan dropped EVERY candidate and the contact
parked 1.8 east of the plate. The 0xa27 mask is what separates the two cases: the balcony
guard's ray crosses real walls (mat 4 — still blocking for 0xa27), the stairwell peer's ray
crosses the masked classes.

## The port's consumers

- `ALLY_LEAD` / `ALLY_LEAD_WAIT` / `PLAYER_IN_WAY` / `LEAD_BOND_COMBAT` hide-enter at ≤ 10.0,
  gated on prop 0x50 [M the flag-1 call sites above] (drone.rs, the AllyLead family).
- `LEAD_HIDE` exits at > 14.0 with the 2-tick timer C latch [M 0x15ec08 case 3].
- The hostage save states (20.0, flag 0) ride the same snapshot.

## Live checks (open)

- [L wanted] 07000001, after the meet scene: watch Drone+0x2B8 (the armed-peer latch) of the
  skin-0x4F contact during the stairwell approach — write watch, expect 0 until a hide-enter,
  then the triggering guard's obj pointer.
- [L wanted] The balcony guard (skin 0x05000036) while the contact is at the stairwell head:
  confirm the game's contact does NOT hide (the sight gate blocks) — poke-bond to the stairwell
  head and screenshot the contact's state over ~5 s.
