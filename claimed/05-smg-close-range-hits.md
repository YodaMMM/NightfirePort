# smg-close-range-hits — Bond's SMG rounds do no damage at close range

Measured on 07000001 (docs/reports/level-01-agent-hall-fight.md on origin/s/level-01-agent-hall-fight, "Remaining" item 3): 32 SMG rounds fired at a guard 0.5-2.1 u away, aim settled, rounds spawned (`last_shot` advances), zero damage. The pistol (weapon 0x2) hits the same guards at 3.5-9 u.

Find why the player's full-auto rounds miss guard hit boxes at close range. Candidates: the first-segment spawn origin for full-auto records (`fire_player`'s `shot_from` in crates/nf-view/src/scene.rs, the muzzle/rocket-origin midpoint path) starting the segment past the target, or the drone hit-box test's near-field gate. Port the game's rule with addresses [M Bullet_init 0x12c5a0, Bullet_update 0x12e7b0, the drone collision handler].

Done = a real-input test: Bond fires the SMG at a guard 1 u and 2 u away and the guard takes the record's damage per round; the same for the pistol as a sister; every walkreal_*_live green; the full suite total line.

---
claimed-by: sandbox6 at 2026-11-03T17:22Z
