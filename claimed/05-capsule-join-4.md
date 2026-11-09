# Solid guard bodies and corpses: one join on current master

Merge s/capsule-join-3 (guard standing body within 2.0 of the player: 0x140e90:91-121 radius 0.4 / 0.55 kind 0xc, 0x149ec8:16-31 with 0x14b190; bump contact through Collide_Jointy 0x1ec180) and s/dead-body-collision (a corpse always has a body, radius min(hip, 0.4) [0x140e90:124,142]; the player collides with it [0x194df0:88]) onto origin/master as one branch (both edit scene.rs's bot_capsules block; combine them). The triage (audits notes) found on the merged tree:
- guest_faces_live (no talking line attached), speech_facial_fallback_live (crosshair never reaches the talker): check whether party guests carry the standing body in the game; if they do, re-stage the tests' Bond spot clear of the crowd.
- walkreal_07000001 (Mayhew plate never fires; real damage: the contact dies in crossfire; Agent: Bond dies to drone #17), walkreal_07000005 00 Agent (E at the vent leaves Bond in substate 0 - check the vent's use with a guard body near), walkreal_07000011 Agent (6000-tick cap at the east connector).
Decide port bug vs lost margin for each; fix or re-route as a player would. Then run every walkreal_* binary one per call. Never lower damage or health, no loosened pins, no new ignores.

Branch from origin/master, push to `s/capsule-join-4`, report in docs/reports/capsule-join-4.md.

---
claimed-by: sandbox1 at 2026-11-09T16:16Z
