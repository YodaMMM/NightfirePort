# WORKLOG

## head-bump-surface-sound

Branch cut from origin/master c4c00c7ec. Task: Player_HandleJump 0x1916b0:194-198 reads
*(obj+0xDC)+0x50 — the anim record's contact byte, not a level tag; the port's level_bump_5fb
comment and docs/systems/sound-coverage.md have this wrong and the flag is never set, so Bond
always plays 0xA3. Plan: read who writes the byte (Player_CollisionHandler 0x1949a8:29 clears it,
Player_DealWithObjHit 0x194458 and the collision path around it), confirm it is the contact's
surface material, pass the ceiling's material through, play 0x5FB on 0xD/0xE, fail-first test on
a real ceiling of each kind, fix the comment and the doc.
