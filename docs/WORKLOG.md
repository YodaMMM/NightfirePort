
## level-12-copter-sound-distance
Started 2026-11-03 05:20 UTC. Branch s/level-12-copter-sound-distance cut from origin/master
6f16146e. Task: 07000012's helicopter sounds near full strength from behind and away — find the
game's 3D positioning for the copter's looping sounds (volume falloff, pan, rear attenuation) and
the copter sound records' radii [D], compare with the port's 3D sound path (nf-audio / sfx.rs),
fix the differences with cited numbers, and pin the copter emitter's volume/pan at distances and
angles to the game's law. Run nf-audio, the copter tests, then the full suite.

- (05:52) The copter's sound chain traced end to end and pinned: the engine 0x10b is a SCRIPT
  sound (script 060005bd stream 10 t=0, 3-D at the stream-4 object [D]; M 0x1d8820) that follows
  the copter per tick (M 0x1d9a08 case 5); the gun 0x5df is Copter_FireGun 0x17d828:85's Play3D.
  The [D] radii (engine inner 10/outer 100, gun 10/50) agree across SFXOutputData and the bank.
  The port's law (nf-audio levels() = M 0x1060c/0x10eb8/0x1300), the follow, the mode and the
  cull all match the game; measured on the real bank the engine is 34% at d20, 12% at d50, 9%
  front + 18% rear behind at d50, silent past d100 — the "full strength from behind and away"
  symptom does NOT reproduce in the sim/driver chain. No port change made; three pin tests added
  (copter_sound_12_live.rs) + two disc/live probes. Live checks for the main agent in the report
  (the PCSX2 volume probe at 0x10eb8, the render's sound_tick listener, the rear-pair downmix).
  Report: docs/reports/level-12-copter-sound-distance.md.
