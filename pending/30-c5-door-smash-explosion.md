# c5-door-smash-explosion — the C5 door blows without its explosion

Branch from origin/master.

On 0700000d, when the deck laser destroys the C5 door, the port's door just vanishes instantly — no explosion, fireball or debris. The game plays an explosion when the door goes (audits-side research: the C5 door rules are pinned in the port's 0d door tests).

Do:
1. Find what the game spawns when that door is destroyed [M the door/destroyable's destroy path: explode script, particle/blast emitters, debris, sound, camera shake; D the object's record and its destroyed-model slot], and the timing.
2. Port it through the existing explode/blast/debris systems. Visible work: list --bond spots + frames where the explosion should show, for a render check.
3. Add a real-input test (the deck gun cutting the door from the gun seat) pinning the spawned explosion and its timing; run the 0d tests, walkreal_0700000d_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.
