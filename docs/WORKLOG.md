
## level-0b-corridor-world-damage (started 2026-11-02 15:03 UTC)

Branch cut from origin/master 84e64088. Task: on 0700000b's real-health walk Bond dies at
(-1.36, 47.81, 5.30) to "-2.40 from the world" every 3-5 ticks crossing the terminal-room → ring
corridor (x -1.0..-1.4, z -5.8..+5.3, floor y 47.18); the game deals pain one drop per contact
event [M Player_DealWithObjHit 0x194458:94-103 → Player_HandlePain(Hurt_GetDamage)];
probe_0b_hurts on master logs the contacts. Plan: (1) name the hurting objects [D] and the port's
damage site, (2) port the contact-event law [M with addresses], (3) un-ignore the 0b real-health
walk; run walkreal_0700000b_live, every hurt_*/fire/blast test, every real-damage walk, full suite.
- 15:03-15:41 UTC: the damage site found and fixed. Markers at every Bond-pain site (one build) pinned the -2.40s to the BULLET path: raw 4.0, owner usize::MAX, weapon 0x60 = the GT turret record 96 (TURRET_WEAPON_RECORD, 0x2c55d0 [L ee_start_05.bin @0x2c55d0]) — NOT a hurt/contact class (0b has zero 0xE4 hurts, zero fuseboxes [D probe dump]). The shooters are the TERMINAL-ROOM wall guns #174/#175 (p0=1 facing-1, p4=0xe enable [D placement dump]; ch 0xe rises 3x in the walk), firing north down the return corridor; the exec-floor trio #291-293 (p4=0x1c, p5=0x4e) never activates (ch 0x1c never rises [D NF_LEVELLOG run]) and their p5=0x4e IS the files channel. Root cause: the port's case-4 fire pushed the DEAD-ON dir to the target; the game eases the barrel first (Quat_Slerp_Acc(REC_FRAME_RATE*4.0, ...) [M 0x180498:295-330; the slerp's alpha = t [M 0x1e4188]) and fires along Mat_GetDir(ext+0xc4 obj+0x90) [M 0x180498:344-352] — a lagging barrel that sweeps behind a runner. Fix: ease before the fire block, fire along the pose forward; new pin gt::combat_fire_rides_the_eased_barrel_not_the_dead_on_line (runner = lag + behind, still target = converges dead-on). The 0b real-health walk un-ignored; cadence/hp/re-holster pins untouched and green (nf-game gt:: 19/19).
