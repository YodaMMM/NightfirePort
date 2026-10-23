# WORKLOG

## 982-civilian-first-sighting-alert
Started 2026-10-23 15:32 UTC. Branch s/982-civilian-first-sighting-alert (re-based onto the
scrubbed base 25d0d7932 after the driver's force-update; the work re-applied and re-verified).
Task: port the case-3 first-sighting alert of NDrone2_DSTATE_CivilianMissionWait 0x15dbb0 (the
part task 962 left out): the NearArmedPerson gate, +0x4F8 |= 0x8000000, the prop-0x33 +
once-only 0x200000-latch shout (+0x4FC |= 1, DroneAlertToObject(dc, 0x11, player, 20.0, 20.0,
0.76)), then the rest of case 3.

Work log:
- (15:32) branch start, WORKLOG entry (on the pre-scrub base efc6ed52).
- (16:10) ported the alert into CIVILIAN_MISSION_WAIT's UPDATE (drone.rs): own_sight for
  +0x4F8 0x8000000 (the old line mis-set `sighted` 0x4000000 — the +0x4F8 field doc settles the
  split), the prop-0x33 + scare_shouted-latch shout (alert_bits |= 1, pending_shout
  (msg::SHOUT, target.pos — the record carries the OBJECT's (player) +0x30 position), latch).
  Disasm-verified call order: DroneAlertToObject(dcv, 0x11, player, 20.0 f12, 20.0 f13,
  0.76 f14) [M 0x15dd3c call site + 0x14a538 callee prologue]; BroadcastMsg delivers msg 0x11
  [M 0x175aa8]; receivers: EnemyAlerts case 0x11, prop-0x28 gate, ShoutFromOtherDrone returns 0
  in this build [M 0x177c18:24-48].
- (16:30-17:35) tests: crates/nf-view/tests/civilian_first_sighting_alert_live.rs, 4 tests,
  ALL PASS on the pre-scrub tree; re-verified on the scrubbed base. Real-path finds:
  (a) 07000007's 35 dtype-10 guests are voice 0xD — NearArmedPerson returns 0 for voice 0xD
  outside 0x7000009/0xa [M 0x1787c8:24-31], so the party guests can NEVER raise this alert BY
  DESIGN; that carve-out hold is pinned as its own test. (b) The alert/latch tests drive the
  real Drone::send entries with the voice-0x6 civilian substituted [S, documented]: the disc
  places dtype-5 voice-0x6 p33+p3c staff on 0700000c/0d [D nf-extract drone-props]. A fully-real
  path needs a MissionWait-capable voice!=0xD civilian (next agent: verify the 0700000c/0d
  scientists reach 0x16/0x17).
- (17:45) re-based onto 25d0d7932 (the driver force-updated the branch base mid-session; the
  drone.rs patch re-applied clean), tests re-run.
