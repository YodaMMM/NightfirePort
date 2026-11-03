
## level-12-copter-sound-distance
Started 2026-11-03 05:20 UTC. Branch s/level-12-copter-sound-distance cut from origin/master
6f16146e. Task: 07000012's helicopter sounds near full strength from behind and away — find the
game's 3D positioning for the copter's looping sounds (volume falloff, pan, rear attenuation) and
the copter sound records' radii [D], compare with the port's 3D sound path (nf-audio / sfx.rs),
fix the differences with cited numbers, and pin the copter emitter's volume/pan at distances and
angles to the game's law. Run nf-audio, the copter tests, then the full suite.
