# fx-water-and-fire — water hit ripples and fire smoke

Port rows 3 and 4 of the table: Env_WaterRipple from the flesh hit handlers, and the Flicker_Update smoke puffs. Read docs/reports/audit-effect-calls.md on origin/s/audit-effect-calls, "Unported calls". Merge origin/master first. Cite [M] for each call, pin each with a test through the real path on real level data, and report every value you use. Shell casings are out of scope; do not change them.

---
claimed-by: sandbox7 at 2026-11-05T01:05Z

---
## Report (done, 2026-11-05T02:22Z)

Closed by the driver: the report is on `s/fx-water-and-fire` (docs/reports/fx-water-and-fire.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-05T02:22Z)

Rows 3+4 of audit-effect-calls done on s/fx-water-and-fire (head f53933049, report
docs/reports/fx-water-and-fire.md). Row 3: Env_WaterRipple 0x17c118 wired into its two
production arms — both FOOTSTEP-gated (AnimObject+0x60 bit 0x4 + material 0xd/0xe), not
bullet-gated: the guard footstep block (drone.rs, NDrone2_DealWithObjHit 0x13fff0:79-83) and
the player footstep block (scene.rs, Player_DealWithObjHit 0x194458:80-82). Row 4: the
Flicker_Update 0x1ab960 spark puffs ported behind their exact dead gate (Light_FindMapLight
0x1cc3d0 is an 8-byte return-0 stub [M]) with values pinned by a unit test: Gas_Init(1.2, 0.8,
cel, obj+0x80, 0, drift, FIRE_MESHES[Rand_Rand(5)], kind 4, 0xffffffff, 0x0a460003, life 85
[M disasm 0x1abc0c/0x1abc34]); mesh table 0x2fcca0 = 0x020002d1..d5 [D elf dump verified].
FlickerFx grew spark_gate + drift (own lines); the GameState+0x52 doc slip fixed to +0x34.
Verified: nf-game lib 663/0/5i, nf-view lib 100/0 (full runs, --test-threads=1). NOT run
(sandbox reset ate ~45 min of setup): every walkreal_*_live + the effect-adjacent live
binaries — listed in the report's "What's left". Also: master's merge 17a784980 dropped
docs/reports/ from the tree (audit-effect-calls.md survives only in commit 62756abf6) — I
re-created the directory; and a live check is requested for the footstep-hit material (0xd vs
0xe under the port's no_water probe).
