# WORKLOG

## c5-door-smash-explosion (started 2026-11-03 04:23 UTC)

Task: on 0700000d the deck laser destroys the C5 door and the port's door vanishes instantly —
no explosion, fireball or debris. The game plays an explosion when the door goes. Do: (1) find
what the game spawns when that door is destroyed [M the destroy path: explode script, blast/
particle emitters, debris, sound, camera shake; D the object record + the destroyed-model slot]
and the timing; (2) port it through the existing explode/blast/debris systems and list --bond
spots + frames for a render check; (3) a real-input test (the deck gun cutting the door from
the gun seat) pinning the spawned explosion and its timing; run the 0d tests,
walkreal_0700000d_live, then the full suite.

### c5-door-smash-explosion — 2026-11-03

The C5 door's explosion existed end-to-end in the port's blast machinery EXCEPT the smash gate:
props.rs `smash`'s <0.05-sphere None-guard (a one-slot-era draw workaround) suppressed
bigdoor01's same-tick twin blast [M 0x12bb60:LAB_0012bc50 gates only on the type byte]. Dropped
the guard; the real-input test (the_laser_kill_spawns_the_smash_blasts) pins both 0x6000052
fireballs, the sphere·3 scale [M 0x12bcdc], the camera shake [M 0x17cbd0:40-44] and the
resolvable fireball art; the C5 draw-keys pin updated to the game's two-blast shape. Report:
docs/reports/c5-door-smash-explosion.md.
