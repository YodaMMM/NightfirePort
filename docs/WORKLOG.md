
## level-0b-corridor-world-damage (started 2026-11-02 15:03 UTC)

Branch cut from origin/master 84e64088. Task: on 0700000b's real-health walk Bond dies at
(-1.36, 47.81, 5.30) to "-2.40 from the world" every 3-5 ticks crossing the terminal-room → ring
corridor (x -1.0..-1.4, z -5.8..+5.3, floor y 47.18); the game deals pain one drop per contact
event [M Player_DealWithObjHit 0x194458:94-103 → Player_HandlePain(Hurt_GetDamage)];
probe_0b_hurts on master logs the contacts. Plan: (1) name the hurting objects [D] and the port's
damage site, (2) port the contact-event law [M with addresses], (3) un-ignore the 0b real-health
walk; run walkreal_0700000b_live, every hurt_*/fire/blast test, every real-damage walk, full suite.
