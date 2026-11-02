# level-03-real-health-servant — report

**Status: done.** `walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage`
is un-ignored and green, together with its whole binary (5/0). The failure was the walk's
assumption, not a port defect in the rescue path — plus one real port leak the diagnosis surfaced
(scream reactions for waiting/bound drones), fixed with decompile citations.

## Why the real-damage run differed

The route's servant lookup (`"the servant carrying 0x4f [D p6]"`) sampled the three servants
AFTER the courtyard legs. On the real-damage run those legs cross the **east 0x21 enable twin**
(measured: ch 0x21 on at tick 7165 [the port's channel read]), so servant #389 enters HOSTAGE and
binds a courtyard captor [M `NDrone2_SetupHostageKiller` 0x144030 — the nearest armed, alive guard
at the entering tick]. The real-damage fight then kills that captor: `NDrone2_DSTATE_Hostage` case
3's captor-death check [M 0x15cb78:24-36] fires HostageSaved → `DroneFunc_HostageSaved` writes
`switch_channels[+0x135]` [M 0x1441c0:11-18], and `NDrone2_PreDroneControl` consumes the p6 byte
once its channel is on [M 0x148db0:51-90]. The game's own designed save simply happened a leg
early — the sampled byte read 0x0 and the lookup panicked. The top-up walk passed because its
fights (no return fire) never reached the captor before the scripted rescue leg. **The game lets
this happen** — it is the rescue working as designed.

## Fixes

1. **The walk** (`crates/nf-view/tests/walkreal_07000003_live.rs`): the servants snapshot moved
   BEFORE any fight (the p6 bytes are the placements' own words [D p6]; the only consumer is
   PreDroneControl [M 0x148db0:51-90]). The rescue loop's live `channels.on(rescue_ch)` check
   already skips the captor shot for an early-rescued servant — the 0x4f leg now finds the byte
   in the snapshot and sees the channel already raised. The captor kill became a chase (the
   east-lane legs' own documented mode-4 dodge pattern: walk to the captor's live position,
   resupply, reshoot) — real input only. The `#[ignore]` is removed.
2. **The port** (`crates/nf-game/src/drone.rs`): the differential's per-drone log caught the
   saved servant knocked out of HostageHide through an Attack re-route (FirstAttack 0x148010
   clears the second switch) into CivilianHiding. The scream/shout receiver block ran its
   civilian/guard reaction for states whose game handlers have **no 0x11/0x12 case**:
   `WaitSwitch` [M 0x15b960 — only 0/1/3/0xE], the hostage family [M Hostage 0x15cb78 /
   HostageSaved 0x15ccf0 / HostageIdle 0x15ce60 / HostageHide 0x15cfe0 — no 0x11/0x12 in any case
   list], and the unhandled→GLOBAL fallback drops them too [M Global 0x15b830 — cases 0/1/10/0x1d
   only]. The block now returns false (the state handler's unhandled, which routes to the port's
   GLOBAL fallback and drops) for `WAIT_SWITCH | GLOBAL | HOSTAGE | HOSTAGE_SAVED | HOSTAGE_IDLE |
   HOSTAGE_HIDE`. Game behaviour: a waiting or bound/saved hostage cannot hear a death scream.

## Verification

- `walkreal_07000003_live` **5 passed / 0 failed** with the test un-ignored (top-up 78.6 s,
  real-damage 25.6 s, both through real input: nav walks, E, real fire).
- The full suite (`tools/test/suite.py`) on this tree: see the total line below.

## What's left

- Nothing on the task. The three sibling ignores (walkreal_0700000b/0c/0d) keep their own
  measured blockers (master 84e64088) — untouched here; the suite runs their binaries without
  their ignored tests.
- Live checks for the main agent: with a fresh 07000003 boot, watch the courtyard guard that
  binds as servant #389's captor (the port measured the east 0x21 twin firing at ~tick 7165 of
  the walkreal approach): confirm the game's HostageSaved path rescues #389 when that guard dies
  in the player's courtyard fight (ch 0x4f on, Drone+0x135 consumed [M 0x148db0:51-90]) — the
  port's differential log says it does; a PCSX2 watch would upgrade the timing to [L].

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(input/output tokens, context used, cost are all absent from the sandbox tooling I can see).
