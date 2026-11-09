# Solid guard bodies: the last reds

Branch `s/capsule-join-2` (on `s/batch-1109g`: master + the game's guard standing body within 2.0 of the player, 0x149ec8 / NDrone2_Collision 0x140e90, and the bump contact fix through Collide_Jointy 0x1ec180) still fails these tests that pass on master; read docs/reports/capsule-join.md on it (the capsule-join-2 section):
- walkreal_07000001: following_mayhew_down_completes_the_level and both real_damage contact walks
- walkreal_07000005: the 00 Agent door-band exit
- walkreal_07000008: walk_the_whole_mission_on_real_input
- walkreal_07000011: the Agent real-damage walk
- upward_shot_0700000c (2): the deck sentries' spawner never re-creates them on the moved stream - check the spawner's respawn pick against the game (a port bug if the game always refills)
- guest_faces_live, speech_facial_fallback_live: Bond placed in the party crowd is pushed off his mark by guests' bodies - check whether party guests (civilians) carry the standing body in the game at all
For each: port bug vs lost margin with evidence; fix the port (test red on old code) or re-route as a player would. Never lower damage or health, no loosened pins, no new ignores. Branch from origin/s/capsule-join-2, push to `s/capsule-join-3`, report in docs/reports/capsule-join-3.md. Run every walkreal_* binary one per call at the end.

---
claimed-by: sandbox4 at 2026-11-09T14:16Z
