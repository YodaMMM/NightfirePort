# WORKLOG

## 994-zoe-gondola-real-path (2026-10-23T10:24:08Z)
- Claimed; setup: clone done. Plan: real-path test on 07000008 gondola (Zoe targeting + riding), then fix port.
---
## 994-zoe-gondola-real-path (2026-10-23T12:02:16Z)
- Test + riding port landed; the team-2 opponent model port was attempted and REVERTED (the 07000001 matrix stream breaks; the evidence is in the report). (a) stays red pending 957b; (b) green.
---
## abseil-damage-repro (2026-10-23T23:18:00Z)
- Claimed; branch from origin/master 8b101249d. Task: 07000011 rappel guards take no damage; `abseil_damage_live` passes on both master and symptom code, so widen: all abseil placements, diffs 1/2/3, phases 0x8e/0x8f/0x90, real mouse-look aim, auto-aim on/off; log round end + guard health; fix from Collide_PickObj 0x1eae80 / abseil DSTATE impact cases 0x169610/0x169730/0x169940.
