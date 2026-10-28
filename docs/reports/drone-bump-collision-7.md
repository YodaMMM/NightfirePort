# drone-bump-collision-7 — the east door's zone reactions and the depenetration question

Branch `s/drone-bump-collision-7` = `origin/s/drone-bump-collision-6` (988e9a49) merged with
`origin/master` (17aa36ab — already an ancestor, the merge was a no-op), plus one code change
(the zone-gate opponent model). The push was never disarmed; the Jointy push, the arming chain
and the door logic are untouched.

## What was ported (the game's answer to a player walking up)

**The 0x31 zone read now runs on the game's opponent model** (`crates/nf-game/src/drone.rs`,
`Drones::door_guard_pass`). The old gate was a [S] stand-in — "the guard currently sees Bond"
(`dr.los`) — which starved the zone ladder: a door guard whose sight never acked kept
`door_zone` 0 forever and the 0x31 UPDATE's zone reactions never fired on real input. The game's
gate is different, all [M]:

- `NDrone2_DoorGuardHandler` 0x1416d8:16-24 returns 0 only when the drone's opponent pointer
  `+0x170` is NULL (plus the |opp+0x34 − my+0x34| ≤ 2.0 height gate, disasm 0x14170c-0x141738).
- In SP `NDrone2_FindOpponent` 0x1424e0:100-120 — run EVERY tick from
  `NDrone2_ControlSTANDARD` 0x1490f0:167 — points `+0x170` at Bond's object
  UNCONDITIONALLY (no sight test), zeroing it only when `PlrStat_OkToUpdate` 0x1993e0 fails
  (the mission-status gate: status < 0 or ≥ 2), the player's class-3 health ≤ 0, or the
  class-0x12 body. The door guards' team byte is not 2 [M 0x14cbd8:48-61], so they always take
  this arm.
- The port already carried this exact fold as `bond_alive` (stamped every tick before the LOS
  gate, standins-drone-922) — the fix is the gate swap plus the comment chain. The
  reaction-acked sight `+0x228 & 4` (`DroneVision_HaveOpponentSight` 0x175ef0:41-46) gates
  `NearArmedPerson` — a DIFFERENT arm of the 0x31 UPDATE, and one the game itself cannot fire
  here for an unarmed player [M `DroneWeap_OpponentHasWeapon` 0x17a4b8 = the opponent's class
  byte 2/3; the tests walk the fists per the party's weapon lock] — not the zone read.

With the fix the zones fire on real input: zone 1 faces Bond, zone 2 plays the challenge
(anim 0x5f variant 1 + the speech), zone 3 is the mission-fail arm [M 0x160648:60-94] — the
port's UPDATE already consumed all three (that code was in place; it was fed by the starved
read).

**The depenetration half of the task is settled statically, and the answer is: the game has
nothing to depenetrate there.** The chain, all [M]/[D]:

- The guards' push capsules are ARMED in the game: `DefaultInit` 0x14b300:60 sets
  `+0x4f8 = (+0x4f8 & 0xffea05e2) | 0x801e2` — bit 0x20 (the DoCollision collide-enable) kept
  set, bit 0x200 (the disable) cleared; `NDrone2_Collision` 0x140e90:95-101 writes kind
  `+0x464 = 0x800` every control tick when `DoCollision` 0x149ec8 passes (the near arm inside
  2.0 [M 0x14b190]); no writer anywhere clears bit 0x20. So Bond IS pushed by both guards in
  the game too — the walk-through theory is dead.
- `Collide_Update` 0x1ea398 only builds/sorts the per-object hit lists (Collide_Pick →
  Intersect → the +0xd0 chain); the push consumers are `Collide_Jointy` 0x1ec180 — whose SP
  arm resolves the PLAYER against bots and returns 0 for a bot resolving against anything
  (:229-233) — and the nav-bounds push in `NDrone2_Collision`'s tail (the port's `push_out`).
  Bots never resolve against the world or the player in SP.
- The door leaf never touches the guards: it swings away from the activator [M
  `Door_SetupSwing`'s side·swing] into the corridor, while the guards stand on the hall side
  (z ≈ −48.0 vs the door plane z ≈ −47.25 [D boots]); the closed/open leaf poses can't sweep
  them. (The -6 report's live check 2 can still confirm the leaf arc on hardware, but the
  static geometry alone already excludes contact.)

## How it was verified

- Rebuilt nf-game + nf-view test binaries on this tree; direct runs (each binary
  `--test-threads=1`, NF_ISO set):
  - `door_guard_live` **3 passed** (pairing, the channel-armed pair, and
    `zone_bands_drive_the_challenge_and_the_mission_fail` — the zone test stays green on the
    new gate).
  - `probe_drone5_mouth` **ok** — dr22 still completes her ring idx15/15 (the -6 green holds).
  - `drone_bump_live` **2 passed**, `sleeper_push_live` **1 passed**, `alert_status_live`
    **1 passed**, `attack_reentry_live` **1 passed** (the Jointy/arming/status regressions).
  - `walkreal_07000007_live`: `the_group_plate_wakes_the_walkers_and_the_guards_stay_calm`
    **ok** (the zone fire does NOT wake the guards — matching the decomp: an unarmed player
    trips no NearArmedPerson arm and zone 1/2 never leave 0x31).
- The four reds re-measured on this tree, all at the SAME east-door wedge (unchanged by the
  zone fix, as the decomp predicts): `probe_07_route` FAILED (6000-tick cap,
  `bond=(95.43,44.35,−48.62) d=1.48`), `party_door_lock_07000007_live` ×2 FAILED
  (`(95.43,−48.62)` / `(95.26,−48.60)`), `cutscene_timer_pause_live` FAILED
  (`(95.43,−48.62)`), `walkreal_07000007_live`'s hall walk FAILED (`(95.26,−48.60)`).
- The `NF_DRONE_WATCH="10,19"` trace over the wedge: d10/d19 stay 0x31 the whole run and
  shuffle ±0.15 in place (d10 x 94.68..94.81, z −48.01..−48.13; d19 x 93.73..93.82,
  z −47.99..−48.05) — the zone reactions turn them, they never translate.

## The measured blocker (why the four reds stay red this round)

The two guards' capsules overlap EACH OTHER: boot centers (94.89,−48.00)/(93.66,−47.98) [D]
are 1.23 apart and the shuffle keeps their live centers ~0.97 apart — under 2×0.95 (Bond r
0.55 + bot r 0.4 [M 0x140e90:95-101 + the port's RADIUS]) — so the pair is a continuous wall
across the doorway. East of d10 the ring edge (x 95.7) is past the jamb (the pocket the steer
orbits into, `(95.43,−48.62)`); west of d19 is the -6 round's wedged arc. The leg's E-spot
(94.6,−47.4) is 0.65 from d10's live center — INSIDE the capsule — while the closest legal
standing point on d10's 0.95 contour, (94.54,−47.12), is only 0.29 from the E-spot: the leg
would arrive if Bond could ever round the wall, and nothing in the decomp moves the guards for
an unarmed player. So either the game's guards are NOT at their [D] boots when a player
arrives (they walked off, or a trigger moves them), or the game's E-spot is not where the
tests aim. That is an [L] question — the live checks below own it.

## What's left

1. The live checks (below) — the guards' real positions/behaviour at player-arrival on a fresh
   boot decide the last mechanism (a relocation, a wider shuffle, or an unreachable-by-design
   E-spot that needs the tests' leg re-derived from the disc's own play).
2. Porting that mechanism, then the four east legs.
3. A full-suite run that reaches every crate (this window's runs were foreground chunks; the
   numbers are under Suite).

## Live checks for the main agent

Fresh boot (a save-state resume skips the boot path), level 07000007, walk Bond (unarmed —
the party weapon lock) at the east door from the hall:

1. **Where do d10/d19 stand when Bond arrives?** gdbwatch d10/d19 (boot (94.89,−48.00) /
   (93.66,−47.98) [D]) while Bond closes from the hall east anchor (93.3,−50.4) to the door:
   record `+0x30..+0x38` (position), `+0x58` (state), `+0x2CC` (the zone) at 6/3/1.5/0.5 u.
   Expected if the boots hold: the zone steps 0 → 1 (face) → 2 (challenge) and the position
   never leaves ±0.15 of the boots. If EITHER guard is > 0.4 from its boot at any zone step,
   something relocates them and that writer is the missing port.
2. **The 30 s idle watch** (Bond far away): how far does the 0x24→0x25→0x26 idle cycle walk
   each guard (the root-motion range)? The port's ±0.15 comes from the same anims; a game-side
   ±0.5+ would mean the port's non-route root-motion fold is eating the step.
3. **Can a player reach the E-spot at all?** Fieldprobe/teleport Bond to (94.6,−47.4) exactly
   and confirm he stands there (no immediate push-out). If the game itself pushes him off,
   the E interaction point is elsewhere (the door's use radius, the plate) and the tests' leg
   target needs re-derivation from the disc's play, not from the port's door list.
4. **Does the challenge anim (0x5f, variant 1) carry root motion?** With Bond at zone 2
   (≤ 2 u), record d10's position across the challenge: any step-back/aside > 0.1 u per play
   is the port's missing non-route root motion (the port replays the anim in place).

## Suite

The sandbox kills background jobs between tool calls (setsid included), so the fast runner
could not be left running; the build (`cargo test --offline --release --no-run -p nf-game
-p nf-view`) and the runs above were foreground. Totals for the binaries run on the final
tree: door_guard_live `3 passed; 0 failed`, probe_drone5_mouth `1 passed; 0 failed`,
drone_bump_live `2 passed; 0 failed`, sleeper_push_live `1 passed; 0 failed`,
alert_status_live `1 passed; 0 failed`, attack_reentry_live `1 passed; 0 failed`,
walkreal_07000007_live `1 passed; 1 failed`, party_door_lock_07000007_live
`0 passed; 2 failed`, cutscene_timer_pause_live `0 passed; 1 failed`, probe_07_route
`0 passed; 1 failed`. The full four-crate sweep did not fit the window (the -6 round's window
did not reach nf-view either); the next message resumes from the branch with the runner.

## Usage

not visible to me
