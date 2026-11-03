# satchel-laser-scramble — lasering the satchel: sparks and scrambled digits

Branch from origin/master.

On 0700004a (and anywhere the satchel charge is placed), cutting the placed satchel with the laser watch should show the laser's spark particles where the beam hits it, and after a short time under the laser its countdown digits should scramble (jumble) the way the game's do. The port shows neither: no sparks, and the digits keep counting normally.

Do:
1. Find the game's satchel-vs-laser path [M: the satchel object's update and its laser-hit handling, the digit display's scramble state and its timer, the spark emitter it spawns] and the data it reads [D].
2. Port both (the scramble timing and the digit pattern exactly; the spark burst through the existing particle system). Visible work: also list --bond x,y,z,yaw,pitch spots where the sparks and the scrambled digits should show, for a render check.
3. Add a real-input test on 0700004a (place the satchel, laser it, assert the scramble state and the spark emitter); run satchel/laser tests, interactables_0700004a_live, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets, and the code must check it) and you must run EVERY walkreal_*_live binary before finishing.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox5 at 2026-11-02T23:19Z

---
## Report (done, 2026-11-03T01:16Z)

