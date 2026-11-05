# WORKLOG

## guard-damage-since-1017
Started 2026-11-05 07:20 UTC. Branch s/guard-damage-since-1017 (cut from origin/master a89818235).
Task: standing 10 s on 07000006 costs 188 health and on 07000011 costs 180, against 6.6 and 10
on a 10-17 build; other stands barely moved. Bisect master from 10-17 for the commit(s) that
changed it, same stand at default difficulty. Read each against the decompile [M]: guard hit
roll, range falloff, weapon damage, difficulty scale. Fix anything that differs and pin the
numbers; if the current behaviour is right, say so with the addresses.
Outcome (2026-11-05 08:40 UTC): the current behaviour is RIGHT — verified against the decompile
today at the four asked areas: the hit roll (DroneWeap_DoBulletAccuracy 0x179740, including the
game's own double TooClose_Accuracy multiply), no range falloff (Drone_ModBulletDamage 0x139f50:
TooClose ×2 / +0x100 / backshot ×2 only, the hit site 0x19455c-0x194588 has none), the weapon
damage (weapon_data+0x0C, the pistol-class ×2 = DefaultInit 0x14b300's +0x100=0x40000000 switch)
and the difficulty scale (Player_HandlePain 0x1902c8 × the Plr_DMod per-level rows). The stands
jumped because the port closed onto the game: the ×2 class write landed 10-22 (3b3bd4eeb) and
the AI/vision/fire pipeline (74f68a479, 065d6e53f, 48edddf6e, adbcbe69e, 10-27..10-31) landed the
game's shot cadence; measured 07000011 d2 spawn stand 18.0 (3×6.0) at 10-17 → 60.0 (5×12.0) at
master, per-hit exactly the live 10/12/20 rows. Pinned by guard_stand_numbers_live 2/0; the
corridor binary 8/0. The driver's 188/180 spots are not the level spawns (06's spawn takes 0) —
the spots + a cadence trace are the report's live checks.
