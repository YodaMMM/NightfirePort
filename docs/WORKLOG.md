# WORKLOG

## shallow-water-wade
Started 2026-11-06 12:57 UTC. Branch s/shallow-water-wade (from origin/master d6245281a).
Task: wading in shallow water — PCSX2-measured at 07000002 (-79, 4.68, 28) on a shallow 0xE
surface: the contact byte reads 0xE, anim+0x60 carries bit 0x10 alongside 8 while Bond stands
there, and steps play 0x28. Trace bit 0x10 of anim+0x60 to its writer and every reader; check
what the game changes while wading against the port (walk speed, ripples from Env_WaterRipple
on each step, footprint decals on 0xE, the eye height). 553 shallow cells on 02, 24 on 06 by
the port's plain ray (06's (24,-7) scan point is another surface in the game — confirm a spot
first). Fix each gap with a test on the real level, prove it red on the old code.
  Measured (2026-11-06 13:10 UTC): the four wade behaviours on the real 02 shallows all MATCH
  the game already — wade speed == dry speed (0.0953 vs 0.0929/tick; the game's walk-speed path
  reads no water state [M 0x192310/0x195a88]); Env_WaterRipple rings spawn on wade steps [M
  0x194458:92-95]; the disc's 0xE EffectRecord carries foot_decal_4/8 = 0 [D] and no decal sits
  on the water plane (the exit steps decal the dry surface they land on — both sides agree via
  the shared EffectInfo [M 0x1cec48:84-89]); the eye stays at the surfaced 0.7 target [M
  0x18f5d8:114-128]. Bit 0x10 of BLData+0x60 traced: cleared per tick with {8,0x10,0x20} at
  Player_CollisionHandler entry [M 0x1949a8:27-28], set by the body-query hit chain (the last
  hit with HITDATA+0x51 bit 2 clear; +0x20 when hit+0x40 y > BLData+0x760) [M :159]; the drone
  handler mirrors it [M 0x140968:71-72/99/107/226]. Readers: Player_Collision case 0xc zeroes
  the velocity and skips gravity while set (the swim-surface float) [M 0x194df0:158-178];
  NDrone2_DealWithObjHit negates drone+0x480 on a class-0x1d hit vs a bit-0x10 player [M
  0x13fff0:91]. The port models the float behaviourally and skips 0xE in the body query — no
  walk-substate reader exists, so no behaviour gap. 06: NO true wade spot — the pond's 0xE
  plane (Object_Pondwater cel #351 at 27.5, 38.6, -7 [D]) sits ~0.01 UNDER a 0xC cap the
  root-height ray reaches first everywhere (settle-scan of 225 grid points: zero feet-byte-0xE
  spots); the driver's 24 plain-ray cells are the coincident-surface trap. Live check requested.
  Tests: wade_real_live.rs (2 tests, green on master — behaviour pins, no fix needed);
  wade_probe.rs (assertion-free, the 06 scan tool).
  Wide 06 settle-scan (2026-11-06 13:20 UTC): 625 points over x 9.5-45.5 / z -25..11 — the
  settled feet bytes are ONLY 0xA and 0xC, never 0xE. Whether the GAME reads 0xE on the pond
  depends on its cel scoping (the port's buckets are coarser than the game's cels [S, documented
  in player.rs]) — live check requested in the report.
