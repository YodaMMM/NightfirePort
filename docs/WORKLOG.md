# WORKLOG

## head-bump-surface-sound

Branch cut from origin/master c4c00c7ec. Task: Player_HandleJump 0x1916b0:194-198 reads
*(obj+0xDC)+0x50 — the anim record's contact byte, not a level tag; the port's level_bump_5fb
comment and docs/systems/sound-coverage.md have this wrong and the flag is never set, so Bond
always plays 0xA3. Plan: read who writes the byte (Player_CollisionHandler 0x1949a8:29 clears it,
Player_DealWithObjHit 0x194458 and the collision path around it), confirm it is the contact's
surface material, pass the ceiling's material through, play 0x5FB on 0xD/0xE, fail-first test on
a real ceiling of each kind, fix the comment and the doc.
Progress: the byte's writer chain confirmed [M] — cleared per tick (Player_CollisionHandler
0x1949a8:29, disasm 0x194a18 sb zero,80), written by the collision path's 0x60-byte HITDATA
copies (Player_FeetOnPoint 0x194188 tail disasm 0x1943cc-0x19440c, gated on the anim+0x60 bit-8
gate + `surfaced` + not-already-0xD/0xE; Player_DealWithObjHit 0x194458:65-85 for object hits).
The port: the ceiling contact's raw surface byte now rides the downward push
(CollisionWorld::capsule_push_scoped's third return; the blanket 0xE skip became
swim/top_guard-or-not-down-facing so the pond undersides collide the walk capsule) and
Player_HandleJump's bump arm plays from `Bond::bump_surface - 0xd < 2` → 0x5FB else 0xA3;
`LadderCtx::level_bump_5fb` (the level-tag misreading) removed. Disc census [D over every
0700*.bin placement collision]: 0xD appears NOWHERE; 0xE down-facing only 07000002 map 51 obj 19
and 07000016 map 63 obj 94 (pond undersides) — and those sit under WATER cels (Bond swims, no
walk-jump), so the water arms ride unit pins; the real-level tests pin the 0xA3 else-arm on two
levels' dry ceilings (07000002 0x4 @8.16/6.32, 07000003 0x4 @8.87/6.93). Fail-first proven by
stash: on the un-fixed code the 0xE unit pin red `[163]` (no bump at all — the old water skip)
and the 0xD pin red `[163,163]` (bump on 0xA3). Tests 5/5 green.
