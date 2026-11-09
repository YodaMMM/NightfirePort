# 07000007: party guests with voice 0x10 turn hostile near an armed Bond

CivilianPatrol 0x15d610:53-55 sets the team byte to 1 for voice 0x10, so 07000007's two voice-0x10 guests turn hostile when an armed Bond is near; the port now does this but no test reaches it through play (the guest never sees Bond on the current walk). Find where these two guests patrol, walk an armed Bond into their sight with real input, and pin what the game does (alert, attack, team), fixing the port if it differs.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-07-guest-hostile`, report in docs/reports/level-07-guest-hostile.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T06:23Z

---
## Report (done, 2026-11-09T07:33Z)

# level-07-guest-hostile — the two voice-0x10 party guests (07000007)

Task: CivilianPatrol 0x15d610:53-55 sets the team byte to 1 for voice 0x10, so 07000007's two
voice-0x10 guests turn hostile when an armed Bond is near; the port does this but no test reached
it through play. Find where the two guests patrol, walk an armed Bond into their sight with real
input, pin what the game does (alert, attack, team), fix the port if it differs.

## The answer, short

The task's premise — armed Bond meeting the patrolling voice-0x10 guests — is **unreachable through
the level's own data**: the weapon lock holds Bond on the fists until ch 0x48, and ch 0x48 is the
very plate that removes the last voice-0x10 guest. What the disc actually does to these guests is
a plate-timed wake/patrol/leave cycle, and the port does every piece of it exactly as the game's
decompile says. No port code changed: the deliverable is the play pin (a real-input walk) plus a
unit pin of the flip arm itself, with the story-dead conclusion documented.

## Where the two guests patrol (the census, [D] placements + nav, [L] sim probe)

- Voice 0x10 = skin 0x05000026; exactly two drones on the level:
  - **#22** (placement **#426**) at (80.71, −50.47): team 3, props 0x33/0x1D/0x3C, boots in
    WaitSwitch with switch channel **23** (TouchOnce #415) and flush pair (channel **21**, tag
    **100**). He sits 0.05 from kind-8 **path 3**'s node 7 → his patrol is path 3 (15 nodes: the
    hall west (79.75, −50.45) → the north row west (65–75, −44.5..−52)).
  - **#55** (placement **#501**) at (88.25, −55.63): team 3, same props, switch channel **21**
    (Drake's reveal) with flush pair (channel **0x48**, tag 100). He sits 0.03 from kind-8
    **path 7**'s node 0 → his patrol is path 7 (10 nodes: the hall (88.26, −55.60) → the north
    row east → (128.84, −55.89)).
- They are **CivilianMission 0x16 walkers** (prop 0x1D → `AssignAIPath(8)`), not CivilianPatrol
  0x14 — the party's non-walkers never reach 0x14. The flip arm lives in BOTH states with the
  same shape; for these two it is `NDrone2_DSTATE_CivilianMission` **0x15d860 case 3** that runs.
- Their **alive windows** are plate-timed: #22 patrols only between ch 23 (the north-row plate
  #415) and ch 21 (the reveal); #55 only between ch 21 and ch 0x48 (the hall walk-in #412).

## What the game does (the pin, [M])

1. **The wake**: `DSTATE_WaitSwitch` 0x15b960 case 3 → the gate channel on → `NDrone2_Enable(1)` →
   the start chain → CivilianInit/PartyGirlInit → prop 0x1D + `AssignAIPath(8)` → CivilianMission
   [M 0x172730:15-45].
2. **The patrol**: `UpdateMissionRoute` 0x154a80 walks the kind-8 path (the port's
   `update_mission_route`).
3. **The flip arm** (0x15d860 case 3, mirroring CivilianPatrol 0x15d610:53-55): on
   `NDrone2_NearArmedPerson` (the radius args 6.0/10.0 are DEAD [M 0x1787c8] — the real gates are
   prop 0x3c + the reaction-acked sight `+0x228 & 4` + `DroneWeap_OpponentHasWeapon`, and "armed"
   is the DRAWN weapon: kinds 2..0x46, 0x47 excluded, fists 1 unarmed [M 0x17a4b8 + 0x178768]):
   - `+0x4F8 |= 0x8000000`;
   - prop 0x33's once-shout: `+0x4FC |= 1`, `DroneAlertToObject(20, 20, 0.7625, msg 0x11, Bond)`,
     `+0x4F8 |= 0x200000` (the alert source is Bond);
   - **voice 0x10 → `+0x44 = 1` (the team byte) and SetState(0x56 Attack)**;
   - every other voice → SetState(0x12 CivilianScared).
   The port's CIVILIAN_MISSION UPDATE arm matches line for line (drone.rs, the arm above
   `(CIVILIAN_MISSION, msg::UPDATE)`).
4. **The departure**: `NDrone2_PreDroneControl` 0x148db0:53-99 — a drone with the second channel
   (`+0x135`) set whose channel is on consumes the pair; tag 0xc deletes outright, otherwise it
   reads the alternate state `+0x5A4` and SetStates to it (with the tag-100 behaviour-block swap).
   These drones' alternate state is **DeleteMe 0xb4** [L probe: alt_state 180 on drones
   22/55/50/9/28/29 — the disc's mode-0x15 row], so **each guest leaves at his flush plate, even
   mid-patrol** — the same mechanism that takes the library door guards #404/#417 off at ch 21
   (pinned by `drakes_reveal_07` on master).

## Why armed Bond can never meet them (the story deadlock)

- The weapon lock [M Player_Weapon 0x1a24c8:242-275]: the cycle is swallowed while ch 0x48 is off;
  the loadout's switch request is the fists. So Bond is unarmed through BOTH alive windows
  (#22's [ch 23 → ch 21] and #55's [ch 21 → ch 0x48]).
- The library door is guarded until ch 21 (the collision push [M 0x149ec8, 0x14b190]), so ch 0x48
  (#412) cannot fire before the reveal — the data forces the order.
- At ch 0x48's rise the P2K draw is the view loop's NEXT-tick rising edge, while the flush
  consumes #55's pair on the first drone pass — `PreDroneControl` runs before the state UPDATE in
  the same pass, so the deletion always precedes the first armed UPDATE. No ordering of real
  inputs reaches the flip.

## How it was verified

- **The play pin** — `crates/nf-view/tests/guests_hostile_07000007_live.rs`
  (`the_voice_0x10_guests_patrol_their_lanes_between_their_plates_and_never_meet_armed_bond`):
  the walkreal route with real input only (no teleport, no channel writes):
  1. the census asserts (two voice-0x10 drones, team 3, WaitSwitch boot);
  2. ch 6 (#476) and ch 23 (#415) fire on the walked route → **#22 patrols CivilianMission on
     route path 3** and crosses Bond unarmed at 8.3 units **without a flip** (the drawn-weapon
     gate [M 0x17a4b8 + 0x178768]);
  3. Drake's reveal (ch 21) → **#55 patrols CivilianMission on route path 7**; **#22 leaves here**
     (the flush → DeleteMe, mid-patrol);
  4. the hall walk-in (#412, ch 0x48) through the real door → the control-taking scene →
     **#55 and drone 50 (#50, the voice-0x12 path-6 walker) leave here**, at the same plate that
     lifts the lock;
  5. the player cycles to a gun (real `weapon_cycle` input through `switch_to_armed`) and walks
     BOTH lanes (path 3's and path 7's nodes) — **no voice-0x10 drone is hostile anywhere**.
  `walkreal_07000007_live` re-run whole after the change: **5 passed / 1 pre-existing ignore**.
- **The unit pin** — `crates/nf-game/src/drone.rs`
  (`civilian_mission_flips_voice_0x10_to_the_hostile_team_on_the_armed_sight`): drives the real
  CIVILIAN_MISSION UPDATE arm: voice 0x10 + prop 0x3c + acked sight + drawn weapon → **team 1,
  SetState(0x56), the prop-0x33 once-shout latched (`+0x4FC |= 1`, the 0x200000 latch, the
  Bond-sourced DroneAlertToObject queued)**; the voice-0x12 sister → CivilianScared with team 3;
  the unarmed sister → no flip, no shout. The `+0x44` byte, the shout and the split are now
  regression-pinned at the arm level.
- **The suites**: `nf-game --lib` whole: **749 passed / 0 failed / 5 pre-existing ignores**
  (includes the new test). `walkreal_07000007_live`: 5 passed / 1 pre-existing ignore. The
  scratch probes (`guest_dump07`, `guest_placements07`) are uncommitted examples, not tests.

## Why "test red on the old code" does not apply

The port already carried the fix (the flip arm, merged with party07-start-rules /
inert-notes-drone-b); nothing differed, so there was no code to revert — the red step is
structurally impossible. The pins added are green on master by construction and would go red on
any regression of the arm, the wake chain, or the departure flush.

## What's left / live checks for the main agent

1. **The departure order at ch 0x48** (gdbwatch): pre-arm Bond at the hall walk-in with
   guest 55 in sight; on the #412 touch, watch `switch_channels[0x48]`, then the drone's
   `obj+0x135`/`+0x13a` consumption and `obj+0xFE` bit 0. EXPECT: the flush consumes the pair and
   sets the delete bit on the first drone pass — BEFORE the first tick where the drawn weapon
   could arm `NearArmedPerson`.
2. **Guest 22's mid-patrol departure at ch 21** (fieldprobe): with #415 already crossed (ch 23
   on), take the reveal and watch drone 22's `+0x5A4` (EXPECT 0xb4) and the SetState(0xb4) →
   `NDrone2_Enable(0)` → gone. The port mirrors this; the game-side check settles the [L probe]
   row-table read.
3. **[S] The story-dead conclusion end to end**: confirm in PCSX2 that no non-story input order
   (e.g. squeezing past the library door guards, or a pre-reveal weapon source) can arm Bond
   while a voice-0x10 guest is enabled. The port's data says no; a live confirmation retires the
   [S].

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
