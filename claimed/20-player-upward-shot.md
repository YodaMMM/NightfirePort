# player-upward-shot — Bond's rounds die on upward shots at range

Branch from origin/master.

level-0c-yard-climb-real-health (blocked; its report is on origin/s/level-0c-yard-climb-real-health) found that on 0700000c Bond's aimed rounds at the deck snipers (weapon 0x1e drones ~33 u away and above him) never land: three clips dry, target hp unchanged, strict gun rays true, while level shots at the same range work.

Do:
1. Trace one upward shot through the port's bullet path (flight segment, the per-tick HITTEST, Collide_Pick straddle cels, the lip geometry) and log where the round stops.
2. Compare each step with the game [M Bullet_update 0x12e7b0, Collide_Pick 0x1eb320, the cel straddle 0x1eac98]: a round that the game lets through but the port stops (wrong cel scope, a wrong tri, a range cap) is the bug — fix it and cite it. If the game's geometry stops it too, say so with the tri/placement that does.
3. Add a test that fires up at a deck sniper from the yard and pins the outcome; run every bullet/decal/stairwell test, walkreal_0700000c_live, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets, and the code must check it) and you must run EVERY walkreal_*_live binary before finishing.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox4 at 2026-11-02T23:00Z
