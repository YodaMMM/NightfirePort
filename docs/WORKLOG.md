# WORKLOG

## level-09-guard-health-live (2026-11-01)

Task: do the 07000009 type-0xD lobby guards really have 1 health? With the placement stats-section
decode (`drone_stats` 0x31aa30, written only by `behaviour_util_get` 0x1c7a98) the 09 type-0xD guards
decode accuracy 80, health 1, reaction 80, recover 80. No live snapshot holds a type-0xD record.
Settle it from data: decode the 09 stats section by hand against the field widths (8,3,8,8,5,8,8,1)
and the bit order the decoder uses, check how health is consumed (DoModeSettingsNEW 0x14c600, the
damage path Drone_ModBulletDamage) — is the stat a multiplier/index rather than raw health? — and use
a fresh-boot EE snapshot at 09 load if one exists. Fix the decode or the consumer if wrong; pin the
09 guards' effective health with citations. Branch start: created s/level-09-guard-health-live from
origin/master.

Finish (2026-11-01, same session): the question is settled — YES, 1 health. The decoder was pinned
from disassembly (the decompile C misreads the width-table copy: it is four unaligned qword pairs =
8 words from 0x303330; the kind advance is (tag&0xffff)/3 words); the decode is live-anchored on
ee_start_07000001.bin's type-2/12 records; DoModeSettingsNEW reads kind 1 only [M 0x14c704] and
stores word 2 to +0xAC as (float)(int) [M 0x14c70c-30]; HitDamage 0x145558:154-156 is the raw-HP
gate; every 09 type-0xD placement runs NEW (param 5 = 0x64 >= 0x24 [M 0x14bccc-d4]). Kind 1 =
80/1/1/80/3/80/80/1: accuracy 80, health 1. Pins added to enemy_accuracy.rs (nf-game) and
walkreal_07000009_live.rs; report docs/reports/level-09-guard-health-live.md. nf-game 1016/0/9;
stealth_ai_live 7/0; walkreal_07000009_live 6/0; enemy_accuracy_live 5/0/4; suite cut at the
deadline with the wgpu-device + disc_golden environmental fails only (details in the report).

## rigid-body-cylinder-sweep (2026-11-01)

Task: port the rigid-body collision handler's unported cylinder contact check and torque step [M
RB_CollisionHandler 0x1d7a80 and the cylinder sweep it calls; RB_RestingContactRot 0x1d73a0's
non-bullet arms], so bodies react to Bond/guards/doors brushing them as in the game (07000004 lamps,
07000006 barrels, 07000001 lamps). Continue from origin/s/lamp-calib, merge origin/master first. Pin
with tests (a walk-through contact moves the body the game's way; sister tests), keep the bullet-hit
curve pinned on lamp-calib unchanged. Branch start: created s/rigid-body-cylinder-sweep from
origin/s/lamp-calib, merged origin/master.

Port (2026-11-01, same session, continued after a sandbox reset): the cylinder contact check and
its torque step are in. `rb_resting_contact_rot` now runs the game's THREE passes in order [M
0x1d73a0]: the cylinder block (up = Mat_GetUp row 1, HITTEST+0x50 base = obj_pos + up·(+0x13c −
+0x9c), start = obj_pos + up·(−+0x13c·1.8) [M 0x1d73f0-0x1d7490], the per-straddled-cel sweep with
the per-candidate DeltaP accumulation into rb+0xC0 (new `RbState.cyl_deltap`) [M 0x1ebaa8:353-394],
the pick's 0x10 flag skipping dynamic objects [M 0x1eb8f8]); the class-5 walk with the per-type
lane masks (type 2 keeps x, type 3 keeps y, type 4 keeps z, type 1 zeroes y — census lamps/barrels
— [M disasm 0x1d753c-0x1d757c], torque slot only, no +0x40 write); the grounding tail [M
0x1d75a0-0x1d7610]. The handler runs every tick (a resting body's cylinder digs the floor even
with an empty obj+0xD0 chain). 3 new lib pins + the lamp-calib bullet curve pins unchanged.

Audit fix (2026-11-01, same session): an independent disassembly audit of 0x1d75a0-0x1d7620
confirmed every mask and gate, and caught one bug the lib tests had missed — the grounding tail
zeroes the cross's Y lane before the add [M 0x1d7600 `swc1 f20, 0x24(sp)` stores the 0.0
tolerance register (f20 = mtc1 $zero, 0x1d75a8) over the stored cross's y lane; VADD 0x1d760c;
store rb+0x50 0x1d7610]. Fixed `rb_cyl_grounding_tail` (+0x50.y untouched) and added the oblique-
push sister case that pins it (nonzero raw cross-y must not reach +0x50). Doc nuance: obj+0xFA |=
0x20 is applied by RB_CollisionHandler itself after either branch [M 0x1d7a80:16-23], not the
caller.

Finish (2026-11-01, same session): the port verified and merged-clean on the final tree. An
independent audit (decompile + raw disassembly) verified all seven structural claims and caught
the grounding tail's missing y-lane zero [M 0x1d7600] — fixed and pinned. Named tests post-fix:
rigid_body_draw_live 3/0, rigid_body_wiring_live 2/0, interactables_07000001_live 9/0/1,
interactables_07000004_live 4/0, interactables_07000006_live 5/0; nf-game lib 644/0/5. Full
suite: build ok (1070 bins, 1546 s), run cut at the deadline at 288 bins — 1706/5/55, the 5
fails all environmental (3 disc_golden + identity_disc slim-ISO, probe_pylon_render no-GPU).
Report: docs/reports/rigid-body-cylinder-sweep.md (live checks for the main agent inside: rb+0xC0
lifetime, the tail's y lane on a tipped body, brush-reaction existence, the cylinder test points).
