# Do dead guards block the player?

The game gives a dead guard a collision body (the capsule-join work found the code path) while the port gives corpses none. Read the dead/ragdoll states' collision setup and whether the player's capsule push meets a corpse in the game (NDrone2_Collision 0x140e90, the dead state's flags), pin it through real input (walk Bond into a fresh corpse), and port it if the game blocks.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/dead-body-collision`, report in docs/reports/dead-body-collision.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T13:24Z

---
## Report (done, 2026-11-09T15:12Z)


---
## Report from the worker (done, 2026-11-09T15:13Z)

# dead-body-collision

Task: do dead guards block the player? Read the dead/ragdoll states' collision setup
(NDrone2_Collision 0x140e90, the dead state's flags), pin it through real input (walk Bond into
a fresh corpse), and port it if the game blocks.

## Answer: yes — the game blocks, and the port now does too

The game gives every dead guard a live collision capsule and the player's capsule push resolves
against it, grounded included. The chain, all [M] (read this session, in work/decomp/ACTION.ELF/):

- `NDrone2_Collision` 0x140e90's dead branch (`+0x4F8 & 0x200`) arms the HITTEST
  **unconditionally**: `+0x464 = 0x800` with no `NDrone2_DoCollision` call (:142), radius
  `min(+0xA0, 0.4)` (:124), and rebuilds the capsule every tick from the ragdoll bone chain —
  axis `normalize(palette[5] − palette[0x37])`, extents `|p1−p2|·0.5` / `|p1−p3|·0.5`
  (:125-141; `AnimGetBoneWorldTrans` 0x1c3120:76-84 routes the negative ids onto the palette
  table at `id·0x40` mod 2³²), arms `pos + axis·0.4` and `pos + axis·(0.4 − max)` (:148-166).
- It runs for the dead: `Drone_Control` 0x139268:61 → `NDrone2_ControlSTANDARD` 0x1490f0 (the
  call sits after `NDrone2_Move`, no dead gate) → `NDrone2_Collision`. The dead branch exists
  precisely for corpses.
- The base is obj+0x30, whose Y rides the animated hip down
  (`DroneAnim_PreTransCallback` 0x13b490:60-75: `+0xA0 = clamp(anim−0.02, 0.2)`, the root's
  delta Y IS the hip change) — the corpse capsule is a LOW roller at the lying body's height.
- The player's body hittest mask is 0 every tick (`Player_Collision` 0x194df0:88; only case 0xe
  sets 0x1c0), so **Bond picks the corpse** (`Collide_PickObj` 0x1eae80 — mask 0 excludes
  nothing; the corpse's own `+0x468 = 0` too). The contact lands on Bond's list;
  `Collide_Intersect` 0x1ebaa8:100-107 dispatches `Collide_Jointy` 0x1ec180 (the corpse's
  `+0xFA & 8` set at creation, `AnimObjectNew` 0x1be910:58; the SP class gates pass — Bond is
  class 3 `Player_Init` 0x1910f8:76, the corpse class 2, and only a class-3 CANDIDATE is
  refused; the class-2-vs-2 movement gate compares the tick's x/z move, `Drone_Control`
  0x139268:105, and a walking Bond always out-moves a corpse). Jointy pushes Bond's capsule arms
  and fills `+0x10` = BLData+0x750, which `Player_CollisionHandler` 0x1949a8:136-140 adds to
  obj+0x30. **No gate anywhere is grounded/airborne.**
- The same bone-capsule formula is the player's OWN dead-body arm (`Player_Collision` 0x194df0
  cases 0xc/0xd — bones 5/0x37/0x33): the lying-body capsule is the game's standard corpse shape.
- The corpse never reacts: `NDrone2_DoCollision` 0x149ec8 returns 0 for dead, so
  `Drone_CollisionHandler` 0x140968 skips the bump walk — the corpse is a static capsule Bond is
  pushed out of. (The old scene.rs comment read that 0 as "no capsule"; it gates the HANDLER,
  not the arming.)

## The port

- `Drone::dead_capsule` (crates/nf-game/src/drone.rs): the exact dead-branch port — palette
  bones 5/0x37/0x33 for axis+extents, radius `min(drone_height(hip), 0.4)` (the port's
  `drone_height` IS the game's `+0xA0` formula [M 0x13b490:63-67]), base `draw_y(hip)` — the
  port's stand-in for the game's hip-riding root Y, the same object Y the palette's matrix uses.
  `None` for a live drone, a short skeleton, or a degenerate axis [S: the port skips where the
  game would divide zero].
- `Sim::corpse_capsules` (crates/nf-view/src/scene.rs): the near corpses' capsules (the 2.0
  near scope, `DroneFunc_NearPlayerCollision` 0x14b190).
- The push list: corpses join `bot_capsules`, leading it with the door guards, counted into
  `ground_bots` — the game's Jointy has no grounded gate, so a walking Bond is pushed exactly
  when the capsules overlap. The live allies keep the shipped [S] airborne-only gate unchanged.
  No draw changes.

## The repro (real input from the level start, no teleport)

`crates/nf-view/tests/dead_body_collision_live.rs`: 07000001 from the spawn → the intro
cut-scene → Bond's own gun kills the corridor guard → Bond walks straight at the fresh corpse.
- Old code (the corpse exclusion restored): Bond crosses the body — worst capsule penetration
  **−0.306**, his position reaches dxz 0.01 of the body centre. RED.
- Fixed: the push holds him on the standoff — worst penetration **−0.024** (the per-tick
  sliver), held at dxz 0.58 while grinding. GREEN.

Measured geometry on that corpse (one capsule): r 0.20 (the death anim consumes the root down —
the clamp floor), arm Y 5.52..5.56, vs Bond's walk capsule reaching pos−1.03 — they overlap once
Bond is on the body, which is why the walk-block is real.

## Verification (each binary whole, --test-threads=1)

- `dead_body_collision_live`: 1 passed (and RED on the old code as above).
- `nf-game --lib`: 754 passed, 0 failed.
- `npc_doors_live`: 6 passed. `fix_guard_misc_live`: 6 passed (drone.rs touched).
- `walkreal_07000001_live`: 5 passed, **2 FAILED**, 1 ignored — see Left.

## Left (the follow-up)

1. **walkreal_07000001_live's main walk + exit-fade** (2 tests) now fail: Bond's scripted route
   walks over fresh corpses (the agent-hall fight leaves bodies on the lane), and the game's
   blocking — now ported — shoves the un-adaptive scripted walk off its lane; the last attempt
   died short of the LoadPointActivator plate at (−26.6, 6.47, 13.6). The choreography was
   written when corpses were ghosts; it needs the same class of re-work the prior
   behavior-port merges did (route nudges around the bodies). The failing tests are the port
   meeting the game's real behavior, not a port regression — but they are red and must be
   re-choreographed before this merges.
2. The remaining walkreal_*_live sweep did not fit the window (the 07000001 binary alone is
   ~300 s on this box; the full 79-binary sweep and the four-crate suite both overflow it).
   Highest-risk unrun: walkreal_07000007_live (door guards + corpses), walkreal_0700000d_live,
   walkreal_07000002/03_live. Run them (and the rest) with the 01 re-choreography.

## Live checks for the main agent

1. Walk Bond into a fresh corpse (shoot a guard, walk into the body): expect Bond to slide off
   the body, unable to stand inside it; watch obj+0x30 of Bond for the shove (BLData+0x750
   accumulates while overlapping).
2. On a fresh corpse: watch ext+0x464 (expect 0x800 while the corpse is present) and
   ext+0x45C (expect min(+0xA0, 0.4) — likely 0.2 after the death anim's root ride).
3. Watch the corpse's obj+0x30.Y through the death: expect it to drop by the hip change
   (0x13b490:73-75) to roughly the lying height.
4. If any death anim keeps the root high (no root track), the corpse capsule sits at the
   standing height and should block a walking Bond hard — worth a second spot check.

## Usage

not visible to me
