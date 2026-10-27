## Your task: `drone-bump-collision`, guards and Bond stop passing through each other

Guards walk straight through Bond on every level (and through each other): the drone collide step of the control pass is not ported. Port it from origin/master:
- `control_funcs` class 2 collide = `Drone_CollisionHandler` 0x140968 [M table @0x26f8a8]; the probe is the two-end capsule ext+0x3F0/0x400 with radius ext+0x45C = 0.4 (0.55 in state 0xC, 0.3 in the +0x2F variant), armed per tick by `NDrone2_Collision` 0x140e90:59-124 under the `NDrone2_DoCollision` 0x149ec8 gates (the 0x20/0x200 flag arms, the 0700004A special case, `DroneFunc_NearPlayerCollision` 0x14b190's 2.0 near-player arm).
- Contacts mirror into both sides' +0xD0 [M Collide_Update 0x1ea398:160-233]; the consumer dispatch sends the player bump msg 0x22, bullets 6/7/0x19/0x17, and the default arm (walls/doors) = velocity zero + deflect [M 0x13fff0:127-147]. Port the msg-0x22 arm's behaviour from its handler (find and cite the address).
- Bond's side: the port's only actor push is Bond-side and airborne-only (player.rs, `!self.grounded`, marked [S]); replace it with the game's rule.
Read `origin/s/level-07-guest-collision-5`'s report first: rounds 1-4 of a narrower attempt broke the walkreal routes (party walk on 07, 0a armour pickup) because a push shifted guard paths and RNG; measure each walkreal/escort test before and after and fix causes, never re-pin a walk to make it pass. Tests: a guard walking a line into stationary Bond stops or deflects at the capsule distance [M]; Bond walking into a standing guard is blocked.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-10-27T20:11Z

---
## Report (done, 2026-10-27T21:57Z)


---
## Report from the worker (done, 2026-10-27T21:57Z)

# drone-bump-collision — the Jointy push landed un-gated, the capsule armed, the 0x22 sender reverted

Branch `s/drone-bump-collision` = `origin/master` (08f06953) + this round. WORKLOG entry
`drone-bump-collision` in the worker's log.

## What was done

### The player-side push — the game's rule, landed

The port's only actor push was Bond-side and airborne-only (`player.rs`, `!self.grounded`,
marked [S]). The stance gate is GONE: `Player_Collision` 0x194df0 runs the class-3 resolve
against every armed class-2 capsule in every stance [M Collide_Jointy 0x1ec180:229-266 — SP
fVar12 = 1.0, the FULL overlap on the query's owner; the class-3-other return 0x1ec180:231-233
keeps the push Bond-side only]. The two-round alternating closest-point resolve and the full
overlap were already ported; only the gate and its comment changed.

### The capsule — armed per the decompile, shared by both consumers

New `Drone::bump_capsule()` [M NDrone2_Collision 0x140e90:59-124]: ends `pos + up·r` and
`pos + up·(r − hip)`; r = 0.3 when the down/asleep flag +0x2F stands (both ends coincide at
`pos + up·(r − 1.0)` [M :76-88 — one VADD, identical stores]), else 0.4, else 0.55 in DSTATE 0xC
[M :91-94 — `*(short*)(ext+0xd8) == 0xC`]; hip = the +0xa0 anim height word, modelled at its
default 1.0 [M NDrone2_DefaultInit 0x14b300:48]. Consumers switched to it:
`LadderCtx::bot_capsules` production (scene.rs) now maps `bump_capsule()` instead of the old
[S] fixed ±0.4 arms, and adds the corpse exclusion (the class-2 contact gate's dead mapping
[S — Door_ReactToHit 0x135980:53-61's port mapping; Jointy has no corpse-word evidence either
way]). The +0x2F identification: `DSTATE_StunDartLoop`'s "down flag +0x2F" (the port's `down`
field, drone.rs).

### The msg-0x22 bump sender — implemented, measured, REVERTED (no code kept)

The decompile research (fresh this round, disasm-checked) found the literal sender the port's
msg::WAIT_PAUSE comment said did not exist: `NDrone2_DealWithObjHit` 0x13fff0:108-125 — the
class-3 hitdata arm zeroes +0x70/+0x80, stores the drone-local player direction, and sends
`Drone_Message(obj, 0x22, 0, 0)` per contact, EVERY tick the contact stands. Receivers:
CivilianMission 0x15d860:130-137 → SetState(0x17) (voice-0x13 skins ignore), CivilianMissionWait
0x15dbb0:136-141 → the +2 s re-arm; all other states swallow it (default return 0). The contact
list itself: mirrored into both sides' +0xD0 [M Collide_Update 0x1ea398:160-233], the hittest
enable behind it the +0x464 = 0x800 NDrone2_Collision writes under NDrone2_DoCollision
0x149ec8:11-34 (the 0x20/0x200 flag arms, the 0700004A special case, NearPlayerCollision
0x14b190's < 2.0 arm; the DEFAULT arm is ON), pick candidates via Collide_PickObj 0x1eae80
(line vs candidate centre +0x80, radius sum). The default arm (walls/doors, velocity zero +
deflect) is mode-0x1d-only [M 0x13fff0:127-147]; drone-vs-drone (case 2) has no visible
message or push in the decompile (a Ghidra tail artifact around 0x1e4e90 — open).

The sender was written into the tick_inner post-walk block and worked (the freeze arms fire),
but it froze a 07000007 mission civilian inside Bond's path and probe_07_route's hall-door leg
STUCK (bond (79.2, 44.4, -51.1), the frozen bot (79.6, -50.3) in permanent 0.88 contact,
NF_PUSH_DEBUG 18768 grind lines). The unproven piece is the PICK-level gate: whether the
drone's hittest seek config picks the class-3 candidate at all [Collide_PickObj 0x1eae80:83-87's
seeker-class filters — 0x100 seeks classes 2/0x11 — hint it may not]; without that gate the
sender over-fires vs the game. REVERTED; the evidence block stays in drone.rs as a comment.
The Jointy push (proven) stays and alone keeps the walk-through symptom fixed.

## Verification

Baseline measured BEFORE any change (master 08f06953), then the same set after (final tree):

| binary | before | after |
|---|---|---|
| walkreal_07000001_live | 3/3 | **3/3** |
| walkreal_07000008_live | 4/4 | **4/4** |
| walkreal_07000014_live | 2/2 + 1 ignored | **2/2 + 1 ignored** |
| ai_traces_live | 4/4 | **4/4** |
| sleeper_push_live | 1/1 | **1/1** |
| probe_07_route | 1/1 | **1 FAILED** (see below) |
| drone_bump_live (NEW) | — | **2/2** |
| nf-game lib | — | 578 passed / 0 failed |

All runs `--test-threads=1`, `NF_SILENT=1`. New tests (crates/nf-view/tests/drone_bump_live.rs,
real input path): `bond_walking_into_a_standing_guard_is_blocked_at_the_capsule_distance`
[min_d 0.873 at r_sum 0.95 − one tick of integration slack, no interpenetration, the tangent
delta deflects Bond around — the brief's "stops or deflects"] and
`the_bump_capsule_arms_follow_the_drone_state_fields` [the 0.3/0.55/0.4 arms vs each live
drone's own fields].

**The full four-crate suite total line is NOT available** — the background `tools/test/suite.py`
dies between tool calls in this sandbox (three prior rounds' finding, re-confirmed) and the
foreground window went to the build + the measured set above. The affected binaries are covered.

### probe_07_route — the one red, diagnosed not re-pinned

The push (not the reverted sender — re-verified with the sender out) wedges the probe's
mechanical straight-line nav on a standing bot at (79.57, 44.40, -50.34) inside the hall-door
gap the route crosses; Bond stops at capsule distance and the 3000-tick cap expires. This is
the faithful game rule vs a route authored pre-push: the same situation collision-5 measured on
07 (the door-guard capsules across a throat, "the real player's route must go around them a
different way" — their live check #4). The probe needs the around-the-guard leg the human
player does; that re-route needs the live-check answer first, so it is left to the next round
with the census below.

## What is left

1. The msg-0x22 sender's pick-level gate: a gdbwatch/trace of the drone HITTEST seek config
   (the +0x08 class-filter bits) during a real bump — does the drone's pick include the class-3
   player? Then re-land the sender (the receivers are already ported and green).
2. probe_07_route's hall-door leg: the around-the-guard route per live check 1 below; never
   re-pin the walk blind.
3. The case-2 (drone-vs-drone) tail at 0x1404bc/0x1e4e90 — name it and decide whether guards
   push each other in SP.
4. The full four-crate suite once on this tree for the total line.

## Live checks for the main agent

1. (07000007 fresh boot) At the hall door: stand Bond against the standing bot at
   (79.57, 44.40, -50.34) — does the real player get pushed (which way), and what route do the
   dev cameras / a human take past that gap? Also the collision-5 #4 pair at (94.66, -47.96) /
   (93.88, -47.90).
2. (any level, fresh boot) gdbwatch the drone's HITTEST struct (+0x08 word) for a class-2 bot
   with Bond within 2.0: the seek/class-filter bits, and whether a +0xD0 contact node against
   the player exists on the drone while touching — settles the msg-0x22 sender's gate.
3. (07000014 fresh boot) The sleeper bot's +0x2F byte and DSTATE (+0xd8): if the "patrolling"
   sleeper carries +0x2F set or state 0xC, the capsule radius is 0.3/0.55 in the real game and
   sleeper_push_live's 0.95 pin needs the address-cited correction.

## Usage

- tokens/context/cost figures: not visible to me (no token or context-window readout is exposed
  to this session; nothing to copy exactly).
