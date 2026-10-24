# WORKLOG

## level-15-ladder-3 (branch s/level-15-ladder-3, started 2026-10-24 05:06 UTC)

Task: 07000015 ladder #382 — settle whether the game's pick (Collide_Pick 0x1eb320) ever admits the unlinked ladder. Master's interactables_07000015_live.rs ladder doc claims a crouch-walk-off-the-lip + strafe route reaches it; branch s/level-15-ladder-2 (unmerged) says the ladder links to no cel, is never on the pick's candidates, and can never be grabbed. Read Collide_Pick in full (disasm where unclear), decide, then either make master's route test pass with real input or replace it with the branch's pin + sister.

Work Log:
- 05:06 UTC: sandbox alive (repo, work/, ISO, ../nf-queue all present; suite is not started yet at this point — started right after branch push). Branch s/level-15-ladder-3 created from origin/master 5215090c. Baseline suite launched in background.

### 2026-10-24 05:04-06:55 UTC — the pick admission settled: master's empty-set reading quoted 0x1eb320 backwards

- Read Collide_Pick 0x1eb320 in full (C + capstone disasm): candidates = straddled cels' chains (gated on count != 0, :128; disasm 0x1eb410/0x1eb414 beql → 0x1eba38) ∪ ForcedList (LAB_001eba38, head = *(0x270590)). An EMPTY straddle set admits nothing. The ladder is in neither source: FindCel miss → obj+0x20 = 0 (build_LinkToRoom 0x1c9680 links only on hit; control_handle_cel_change 0x133288:63-67 retries the same miss), Controls_StraddleTest 0x134778 counts 1 < 2 from cel 0 [disasm 0x1347b8/0x1347d0/0x1347d8] and takes the clear arm, never LList_Add(0x270590).
- Master's ignored the_ladder_takes_the_climb replaced by s/level-15-ladder-2's pin + sister (re-landed on today's master, doc enriched with the beql quote): the_dead_ladder_never_grabs_at_point_blank + sister_the_dead_ladder_mesh_reaches_the_point_blank_stander. 11 passed / 0 failed ×2; the four 07000015 binaries green (walk 2/0, walkreal 1/0, census 2 pre-existing ignores). Census §7 row corrected. Added CollisionWorld::debug_scope_cells.
- Suite: this sandbox kills background jobs between tool calls (3 nohup/setsid starts died) → foreground chunks with exact suite.py per-binary semantics. nf-game 85 bins 881/0/6; nf-formats: only the slim-ISO disc-data failures (3 disc_golden allowances + identity_disc, which asserts the full disc's 1,191,072 sectors — this ISO is 1,061,754,880 bytes; crate untouched); nf-audio green; nf-view PARTIAL 275/769 bins (rootfs hit 100% "No space left on device (os error 28)" mid-build; after reclaiming, the 2-core rebuild left ~494 bins unrun at expiry). Cumulative 1705 passed / 4 failed (all slim-ISO environmental) / 57 ignored.
