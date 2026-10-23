# WORKLOG

(Started fresh from the rewritten master a67f1a27e; older entries live on the pre-rewrite branches.)

## breakable-damage-unit — start

- Branch `s/breakable-damage-unit` created from origin/master a67f1a27e.
- Task: how much damage a player bullet does to a 0x20 breakable. `Break_Create` stores param 1 as int->float [M 0x12c230:22-27] so 0700000a vending glass #379 has health 9. `Break_Update` subtracts each class-5 contact's HITDATA+8 [M 0x12bfd8:37]; the port passes the bullet's raw weapon `damage` (+0x0C = 0.1 for gun 0x43), so ~90 contacts. Find what the collide pass writes to HITDATA+8 for a bullet contact on scenery, and the one-contact-per-bullet-per-object rule. Un-ignore `interactables_0700000a_live::vending_glass_breaks_under_fire` with the proven count as pin; sister: far glass #378 stays alive.
- Plan: read the decompile for the HITDATA fill on class-5 hitters; check the lobby-roof-glass-09 branch for prior art; fix the port; prove count; un-ignore; suite; push.
