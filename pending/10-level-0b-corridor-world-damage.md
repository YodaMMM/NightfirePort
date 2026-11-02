# level-0b-corridor-world-damage — the exec-floor corridor hurts every few ticks

Branch from origin/master.

On 0700000b's real-health walk (walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage, ignored with this reason) Bond dies at (-1.36, 47.81, 5.30): "-2.40 from the world" every 3-5 ticks while crossing the terminal-room → ring corridor (x -1.0..-1.4, z -5.8..+5.3, floor y 47.18). The game's player pain from a hurt object is one drop per contact event [M Player_DealWithObjHit 0x194458:94-103 → Player_HandlePain(Hurt_GetDamage)]; probe_0b_hurts on master logs the contacts.

Do:
1. Name the objects that hurt there (placement, class, the hurt volume/fire/glass) [D] and the port's damage site that applies them.
2. Port the game's contact-event law (when a contact starts/repeats, any cooldown or state that stops a held contact re-hurting) [M with addresses]. Fix the port where it differs.
3. Un-ignore the 0b real-health walk; it must reach the mission end. Run walkreal_0700000b_live, every hurt_*/fire/blast test, every real-damage walk, then the full suite.
