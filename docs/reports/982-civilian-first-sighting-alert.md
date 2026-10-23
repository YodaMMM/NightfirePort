# 982-civilian-first-sighting-alert — the CivilianMissionWait case-3 first-sighting alert

Branch `s/982-civilian-first-sighting-alert` (final commit 1acbefc33 on the scrubbed base
25d0d7932 — the driver force-updated the branch base mid-session; the work was re-applied and
re-verified on it).

## What's done

**The port** (`crates/nf-game/src/drone.rs`, `CIVILIAN_MISSION_WAIT` msg::UPDATE — case 3 of
`NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0):

1. `+0x4F8 |= 0x8000000` now maps to `own_sight` (the own-sighting bit). The pre-existing line
   mis-set `sighted` (0x4000000) with a comment claiming 0x8000000 — the `+0x4F8` field doc in
   the same file (sighted 0x4000000 / own_sight 0x8000000) settles the split [M 0x15dbb0:45].
2. The prop-0x33 armed-intruder shout, once per drone life: `alert_bits |= 1` (+0x4FC bit 0
   [M 0x15dbb0:54]), `pending_shout = Some((msg::SHOUT, self.target.pos))` — the port's existing
   alert path (the PartyGirl/Patrol/DoorGuard shout: the consume pass broadcasts msg::SHOUT within
   20 of the source position) — and `scare_shouted = true` (the 0x200000 once-latch [M
   0x15dbb0:53-58]), gated on `self.prop(d, 0x33)` [M 0x15dbb0:46-47].
3. The shout position is the player's (`self.target.pos`): the alert record carries the OBJECT's
   (the player's) +0x30 [M 0x14a538:18-21 + the callee prologue `sw $a2, 0x290(s0)`].
4. The rest of case 3 was already ported (voice `+0xD8 == 0x10` → +0x44 = 1 + Attack 0x56, else
   CivilianScared 0x12) — unchanged.

**The disasm check** (task-mandated): the call is exactly
`NDrone2_DroneAlertToObject(dcv, 0x11, NDrone2_Player(), 20.0, 20.0, 0.76)` — a0=dcv, a1=0x11,
a2=Player, f12/f13=0x41a00000, f14=0x3f428f5c, no stack args [M 0x15dd3c call site +
0x14a538 callee prologue; every arg-carrying word hand-verified against the ELF bytes]. Ghidra's
float-first display in the decompile is its register-model artifact, not the call's order. The
game delivers msg 0x11 (the alert type IS the broadcast message) with the record pointer and
range `*(dcv->[4] + 0x108)` [M 0x175aa8:24-27 + the 0x14a538 tail], and `NDrone2_SetupAlertTarget`
points the shared alert target at the record position [M 0x155d48:6-11].

**How the 0x11 alert type is handled by the receivers** (the task's citation request):
`DroneVision_EnemyAlerts` case 0x11 does the +0x5a0/+0x5b0/+0x5c0 bookkeeping, then gates on the
RECEIVER's prop 0x28 → `DroneVision_ShoutFromOtherDrone`, which returns 0 in this shipped build —
no state switch from the shout itself [M 0x177c18:24-48]. The port's pre-dispatch SHOUT arm
(drone.rs ~6534-6552) is the reaction model: prop 0x28 → guard `enemy_alerts`, civilian →
CivilianScared; my test's delivery witness is a prop-0x28 guard's `resume_state` bookkeeping
flipping 0 → her state id.

## The real-path finding that shaped the tests

**07000007's party guests can never raise this alert — by the game's own design.** All 35
dtype-10 guests carry props 0x33 and 0x3c [D `nf-extract drone-props 07000007.bin`, 2026-10-23]
but they are voice 0xD, and `NDrone2_NearArmedPerson` returns 0 for voice 0xD on every level
outside 0x7000009/0xa [M 0x1787c8:24-31]. The live sim confirmed it end-to-end: the guest reaches
visibility 1.0, LOS true, seen true with armed Bond 1.5 ahead — and `near_armed_person` still
returns false through the voice gate. That carve-out hold is pinned as its own test (it is also
the regression guard: the new alert code must not fire through it).

The alert/latch tests therefore drive the REAL state-machine entries (`Drone::send` for msg 0x22
and the UPDATE, the real per-tick `bond_armed` stamp, the real consume-pass broadcast) with two
documented [S] harness substitutions: the voice-0x6 civilian skin (the disc places dtype-5
voice-0x6 p33+p3c staff on 0700000c/0d [D drone-props]) and a raw `state = 0x16` re-arm between
sightings (the port models no scare→mission resume; the real chain ends in the scared-run 0x13).

## Tests (`crates/nf-view/tests/civilian_first_sighting_alert_live.rs`, opt-in NF_ISO)

4 tests, ALL PASS, with the fail-first check on the pre-fix tree:

- `the_first_sighting_shouts_the_armed_intruder_once` — FAILS pre-fix (own_sight never set): the
  sighting stamps [M :45], `+0x4FC` bit 0 [M :54], the latch [M :58], the shout queued
  (DroneAlertToObject's 0x11/20/20/0.76 call through the port's path) [M :55-57], SetState(0x12)
  runs per her voice [M :63-70]; one real tick delivers the shout to a prop-0x28 guard (the
  EnemyAlerts bookkeeping witness) [M 0x177c18:24-48]; a second sighting re-stamps but does NOT
  re-shout (the latch [M :53-58]); the control (latch poked clear) re-queues — the silence was the
  latch's doing.
- `without_property_33_the_sighting_never_shouts` — FAILS pre-fix: the sighting still stamps (the
  own-sighting bit is unconditional [M :45]) but no +0x4FC bit, no latch, no queued shout
  [M :46-54].
- `the_voice_0xd_guests_never_raise_the_alert` — passes on both trees (pre-existing behavior):
  the carve-out hold with armed Bond in sight [M 0x1787c8:29-31 + 0x15dbb0:39-44], durable past
  the 2 s clock (the TIMER_C re-arm [M :87-106]).
- `an_unarmed_bond_gets_bondtalk_and_no_alert` — passes on both trees: Bond's fists (weapon 1)
  are outside the armed set [M 0x17a4b8 via the port's fold], she holds the wait, nothing stamps.

## What was run

- `cargo test -p nf-view --test civilian_first_sighting_alert_live -- --test-threads=1` — 4
  passed, 0 failed (on the final tree; re-run after the rebase). Fail-first re-verified: with the
  pre-fix `drone.rs` checked out, 2 passed / 2 failed (the two alert tests fail at their own_sight
  asserts).
- `cargo test -p nf-view --test civilian_mission_wait_live` — 2 passed (task 962's tests still
  green with my change).
- nf-game neighbors of the change: `drone_portcheck_live` 5 passed, `door_guard_live` 3 passed,
  `ai_live` 36 passed, 0 failed. All 85 nf-game test binaries compile clean.
- **The full four-crate suite (`tools/test/suite.py`) did NOT run this session** — the sandbox
  lost most of a disk to the stale 983 tree (freed 3.1G), the ALSA dev setup had to be rebuilt,
  and the driver's mid-session branch-base force-update consumed the re-verify window. What ran
  is the focused set above; the next runner should start `suite.py` on this branch.

## What's left

1. The full suite on this tree (see above).
2. A fully-real first-sighting test needs a MissionWait-capable civilian with voice != 0xD:
   the 0700000c/0d dtype-5 voice-0x6 staff carry 0x33+0x3c [D drone-props] but their route into
   CivilianMission 0x16 / msg-0x22 reachability is not verified yet (my probe printed their
   placements; the next agent should check whether they walk mission paths and park).
3. Observations not touched (other tasks' states, flagged not fixed): CIVILIAN_GUARD 0x1603a8 and
   CIVILIAN_DOOR_GUARD 0x160648 pass `self.pos` as the shout position while their decompiles call
   `DroneAlertToObject(…, player)` — the record position is the OBJECT's (the player's) [M
   0x14a538:18-21], so those two may want `self.target.pos` (the Patrol/PartyGirl/MissionWait
   pattern). The PATROL UPDATE sets both `sighted` and `own_sight` where the decompile writes only
   0x8000000 [M 0x15d610 case 3].

## Live checks for the main agent

1. 07000007, party guest (e.g. the walker near (122.8, 43.8, -83.2)): give Bond the P2K, stand 1.5
   ahead of her facing; expect her to hold the wait indefinitely (voice 0xD carve-out), and when
   Bond is REMOVED, the expired TIMER_C resumes 0x16 [M 0x15dbb0:87-110]. Trace `+0x4F8` at
   drone+…: expect NO 0x8000000/0x200000 write while she waits.
2. Any level with a MissionWait civilian of voice != 0xD: armed Bond in sight → watch ONE
   `jal 0x14a538` with a1=0x11, a2=player, f12=f13=0x41a00000, f14=0x3f428f5c, then
   `+0x4F8 |= 0x200000` [M 0x15dd3c-0x15dd50]; a second sighting must not re-jal.
3. The shout's receivers: with the record at 0x2990a0, confirm a prop-0x28 guard within 20 of the
   player's position writes its +0x5a0/+0x5b0/+0x5c0 [M 0x177c18:24-30] and does NOT switch state
   (ShoutFromOtherDrone returns 0 [M 0x177a30]).

## Usage

`not visible to me` — this sandbox exposes no token/credit/context-window figures. (Subagent
usage: one research agent reported prompt 76321 / completion 7247 / total 83568 tokens.)
