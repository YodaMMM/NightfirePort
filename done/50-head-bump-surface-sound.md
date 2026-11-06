# Head bump sound follows the surface

`Player_HandleJump` 0x1916b0:194-198 reads `*(obj+0xDC)+0x50`, the anim record's contact byte, not a level tag. The port's `level_bump_5fb` comment and docs/systems/sound-coverage.md have this wrong, and the flag is never set, so Bond always plays 0xA3. Read who writes the byte (`Player_CollisionHandler` 0x1949a8:29 clears it, `Player_DealWithObjHit` 0x194458 and the collision path around it), confirm it is the surface material of the contact, then pass the ceiling's material through and play 0x5FB on 0xD/0xE. Fail-first test on a real ceiling of each kind. Fix the comment and the doc.

---
claimed-by: sandbox1 at 2026-11-06T03:55Z

---
## Report (done, 2026-11-06T05:06Z)

