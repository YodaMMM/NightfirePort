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
