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
