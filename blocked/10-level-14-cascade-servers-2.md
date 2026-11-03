# level-14-cascade-servers-2 — the servers vanish when the generator hall blows

Branch from origin/master; the previous attempt's research is on origin/s/level-14-cascade-servers-and-door-guards (its report pins the block-0x27 LOD-next / parsemap fallback slot for destroyed models).

On 07000014, when the generator-hall cascade blows the server racks, the racks disappear instead of showing their destroyed models, and guards keep trying to open a door outside the room.

Do:
1. Finish the destroyed-model swap for the cascade's objects [M parsemap_block_map_data_dynamic 0x1d1b68 / parsemap_block_LOD 0x1d1f10 and the destroy path that switches the drawn model]; the destroyed racks must draw their destroyed model at their spots. List --bond spots for a render check.
2. Find why the guards spam the door outside the room (their goal/door target after the cascade) against the game [M]; fix where it differs.
3. Add real-input tests on 07000014 (the cascade from the switch E, then: destroyed models present, no door spam); run interactables_07000014_live, walkreal_07000014_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (the code must check the flag); re-run every walkreal_*_live before finishing. A visible fix ships with a real-input test that reaches the spot from the level start.

---
claimed-by: sandbox3 at 2026-11-03T00:14Z

---
## Report (blocked, 2026-11-03T01:10Z)

