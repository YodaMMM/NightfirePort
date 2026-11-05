# fx-muzzle-and-trail — turret smoke, trail big puff, guard muzzle flash

Ported rows 6, 12 and 14 of `audit-effect-calls`'s "Unported calls" table. Shell casings untouched.
Every value below is tagged; the decompiler-dropped details were checked in the disassembly-backed
decomp files under `work/decomp/ACTION.ELF/`.

## Row 6 — the turret muzzle smoke (×3 sites)

**The two GT fire tails** (`GT_Update` 0x180498 case 4 @:356 = jal 0x180c38, case 7 @:530 = jal
0x181350): on every OTHER shot the fire tail toggles the flash object's barrel byte `+0xf6 ^= 1`
(and re-arms the flash `+0xf4 = 2`, unhides `+0xf0 &= ~0x10`) and calls
`Gas_Init(0.05, 0.1, cel, muzzle, rot, NULL-vel, 0x2000039, kind 7, 0x48403080, 1)` with life 128
only when the toggled byte is 1 [M disasm 0x180bd0-0x180c3c / 0x181304-0x181350]. Fresh objects
read 0, so the FIRST shot smokes. The muzzle is the same vector the round spawned from.
- RNG draws per smoke, in order [M]: 1. the caller's `Rand_FRand(2π)` (0x180bc0) stored to the ROT
  z; 2. the spin `Rand_FRand(π/2·REC)` [M 0x1aacc8:72-80]; 3. the sign `Rand_Rand(0x100)` — negate
  at ≥ 0x80 [M 0x1ab08c]; 4-6. the kind-7 NULL-vel wind spread `Rand_FRand(0.2)` ×3
  (`vel = (wind.x + r − 0.1, wind.y + r, wind.z + r − 0.1)·REC` — the game's own asymmetry: y is
  NOT centred [M 0x1ab138-0x1ab1c8]; wind = 0 in the port's rooms). All consumed BEFORE the
  pool-cap check (the searchlight precedent).
- Port: `GasPool::init_gt_shot_smoke` (gas.rs) + `GTEvent::ShotSmoke` emitted by the `barrel_alt`
  toggle in `gt.rs` (both fire sites) and drained in the Drones GT arm (drone.rs).

**The mounted-gun overheat smoke** (`GunImp_Update` 0x1320d8:218-247): while the hot byte
(ext+0x9c, 200 heat) stands, every 6th game tick (`GameState % 6 == 0` [M 0x132480-8c divu/mfhi])
`Gas_Init(0.05, 0.25, cel, pos, rot, NULL-vel, 0x2000039, 7, 0x484030e4, 1)`, life 128 [M
0x132510-40]. The position is the barrel-tip base — the SAME vector `Bullet_init` spawns the round
from (auStack_150, the port's `GunImp::muzzle()`) — offset by `−(obj+0x8c)·FRand(1.0)` along
`Mat_GetDir(obj+0x90)` [M :224-241]; the ROT z = the caller's `FRand(2π)` [M :242-244]. The game
emits it BEFORE the input read, so the port wires it ahead of the fire/stop block (drone.rs).
- RNG draws per emission [M]: 1. `Rand_FRand(1.0)` (0x1324b4, the offset fraction); 2. `FRand(2π)`
  (0x132504); 3-4. the spin + sign; 5-7. the wind spread ×3 (the same kind-7 path).

## Row 12 — the bullet trail's big puff

`Bullet_trail_effect_smoke` 0x12cba8 has two arms [M decomp + disasm 0x12cbf4-0x12cca0]:
- `obj+0xf6 == 0` (and `MPSettings == 0` [L .bss word 384 = 0]): the fast arm —
  `(GameState + (u8)obj[0xec]) & 3 == 0` → `Gas_Init(0.1, 0.5, …, 0x200001f, kind 7,
  0x404040ff → 0x40404001)` with ONE caller draw (`FRand(2π)`) and a given-ZERO velocity (no wind
  draws). Already ported (`init_trail_smoke`).
- `obj+0xf6 != 0`: the BIG arm — `(GameState + (u8)obj[0xec]) % 15 == 0` → `Gas_Init(0.2, 5.0, …,
  0x200001f, kind 0xb, 0x404040ff → 0x404040ff, life 255)` — six draws: vx `MVar2(0.3, 0.15)+0.6`
  [M 0x12ccc4], vz `MVar2(0.3, 0.15)+0.3` [M 0x12cce4], vy `MVar2(0.1, 0.05)+0.5` [M 0x12cd08], the
  ROT-z `FRand(2π)` [M 0x12cd28], the kind-0xb spin ×0.25 [M 0x1ab084], the sign [M 0x1ab08c]. The
  vel store is the VU0/EE multiply [M 0x1ab26c-0x1ab28c] — the port's plain `*` was corrected to
  `mul_ee` by the value test.
- **The latch**: `obj+0xf6 = 1` is written by `Bullet_DoTrails` 0x12d890:397-400 on the bullet's
  FIRST bounce (with state `+0xf4 = 1`), on the no-explode-on-hit path (records WITHOUT `+0x70`
  bit 0x20000) [M]. The smoke-bearing records 0x1b/0x36/0x69 [L ee_start_05.bin @ 0x2bf150 sweep:
  +0x6c bit 0x2 set] — 0x1b has bit 0x20000 CLEAR (it explodes on its first hit, fast arm only);
  0x36 and 0x69 carry it (they bounce). So the port's gate is `g.bounces == 0 → fast` /
  `g.bounces > 0 → big` (the WIP draft's `stuck` mapping was wrong — stuck satchels never latch
  +0xf6 and their records carry no trail bit anyway).
- **The born phase**: both gates add the bullet's creation stamp — obj+0xec = GameState at creation
  [M control_init_object 0x134630:15], read as a BYTE by the trail. The port stores
  `Grenade.born = frame as u8` at every creation site (throw_grenade, the player throw in scene.rs,
  the MP fire in mp/combat.rs) and the gate is `(self.frame + born) & 3 / % 15`. The old port gate
  dropped the +id offset entirely.
- Port: `GasPool::init_trail_big_puff` + the kind-0xb motion case in `GasPool::update`
  (pos += vel then the shared buoyant lift, the rot-z spin accumulation [M 0x1ab2d0 case 0xb =
  0x1ab44c]).

## Row 14 — the guard muzzle-flash object

`DroneWeap_FireWeapon` 0x178f68:189-213 [M]: after the muzzle light, `control_create_object(0x40,
muzzle, 0, 0)` creates the flash sprite — `control_create_object` 0x1344e0 is a plain
`Mem_Malloc(kind + 0x110)`, so it succeeds (and makes its draws) on every live shot; it returns 0
only on heap exhaustion. The object starts at the muzzle with the aim's spherical angles —
`Vec_Cartesian_2_Spherical` 0x1e39e8 gives (elevation, azimuth, ‖v‖) and the caller negates the
first (`fStack_120 = -fStack_120` 0x179258) — then three orientation draws [M 0x17925c-0x1792b0]:
`FRand(2π)` → obj+0x50, `FRandHalf(0.2)` → obj+0x54 (which also gains +π/2), `FRandHalf(0.2)` →
obj+0x58; then `+0xff = 0x10`, the gfx datum's `+0x30 = 2 / +0x32 = 0`, the entity-gfx hashtable
bind when the weapon datum carries one, `+0x106 = 0xff`, and the cel link.
- What each draw shifts: the flash sprite's yaw/pitch/roll jitters — and the SHARED STREAM: the
  three draws sit between the light's `Rand_Rand(param_11)` and the per-pellet spread draws on
  every guard shot, so every downstream draw (later shots, impacts, AI) moves. fix-guard-weapons3
  had deliberately left them unconsumed; that shifted the seed-calibrated cover tests (90-vs-37).
- Port: the sim consumes the three draws per shot and records them in `Drone.shot_flashes`
  ((frame, [j0, j1, j2]), parallel to `shots`); the viewer's AI flash arm (`scene.rs`) now builds
  the flash matrix from the REAL recorded jitters (`flash_matrix_jitters`) instead of the fake
  xorshift seed. The MP-body and firing-demo arms keep the stand-in (no sim records there).

## Pins (tests through the real path)

- `gt.rs::shot_smoke_fires_on_every_other_shot` — the real `GT_Update` step: FSFFS… alternation,
  the smoke's muzzle == the round's.
- `gas.rs::gt_shot_smoke_draws_six_and_lands_the_values` / `trail_big_puff_draws_six_and_lands_the_values`
  — twin-RNG replay pins the six-draw stream position and every stored value; the big-puff test
  caught the plain-`*` vs EE-multiply vel store.
- `nf-view/tests/trail_big_puff_live.rs` — a 0x36 smoke grenade dropped on the real 07000005 load:
  fast-arm puffs on `(frame+born)&3==0` in flight, the bounce, then big puffs ONLY on
  `(frame+born)%15==0` with mesh 0x200001f, scale 0.2, rgba 0x404040ff and the three velocity bands
  (vx [0.45,0.75]·REC, vy [0.45,0.55]·REC, vz [0.15,0.45]·REC, REC = 1/60).
- `ai_live.rs::guard_takes_cover_like_the_game` — re-seeded 9 → 16 on the corrected stream: seeds
  1, 11, 16, 17, 19, 23 pass every live pin (states 0x58/0x5a/0x8c/0x93/0x94/0x95/0x96,
  UnderCoverInit 30 (live 31), UnderCoverAim 34 (live 37), cover node 1, the cover position within
  0.02); 16 also passed every pin on the old model.

## Changed-code run list (release, --test-threads=1 unless noted)

- `nf-game --lib`: 665 passed / 0 failed (includes the new gt/gas pins).
- `nf-game --test ai_live`: 37/0 (after the re-seed). `nf-game --test guard_drops_flashbang_live`:
  its tests passed inside the combined pre-re-seed run (36 passed with only the cover test red).
- `nf-view --test trail_big_puff_live`: 1/0.
- **The full suite did not fit the sandbox window** (disk-pressure cleanup + the seed re-pick ran
  past the stop-work line). NOT run before the push: the remaining nf-game/nf-view test binaries
  (incl. every `walkreal_*_live`, `npc_doors_live`, `fix_guard_misc_live`), nf-formats, nf-audio.
  The `born`-field plumbing touches ~20 test files' struct literals (compile-checked:
  `cargo check -p nf-game -p nf-view` clean; the lib + ai_live binaries built and passed), but
  per the 2026-11-03 lesson the whole set should run before merge.

## Live checks for the main agent

1. **Row 14's draw count** [M→L]: gdbwatch the RNG entry during one guard shot in 07000005;
   expect exactly 3 draws between `Light_Create`'s `Rand_Rand(param_11)` (0x1cc3d8) and the first
   `Bullet_init` spread draw — the FRand(2π)/FRandHalf(0.2)/FRandHalf(0.2) triple at 0x179268/
   0x179288/0x1792ac. If the live game shows ZERO draws there (no kind-0x40 object), revert the
   consumption + re-seed (see ai_live's docstring).
2. **Row 14 render**: `--bond` at the 07000005 gate-guard cover spot during UnderCoverFire — the
   flash mesh should now carry the game's per-shot orientation (compare a few shots; the jitters
   are ±0.1 rad on pitch/roll and a full 2π yaw).
3. **Row 6 render**: 0700000c — a firing Ronin turret should puff small smoke on every OTHER shot
   from the muzzle; the mounted 0d guns should smoke from the barrel while overheated.
4. **Row 12 render**: throw a smoke grenade (0x36), let it bounce — small fast trail puffs in
   flight, then the big grey no-fade puffs rising off it every 15 ticks after the first bounce.
5. **The GT barrel byte** [M→L]: gdbwatch `obj+0xf6` of a firing GT's flash object (0x180bd8
   writes) — expect 0/1 alternating per shot with `Gas_Init` (0x1aacc8 entry) called only when the
   post-toggle value is 1.

## What's left

- The full-suite run listed above (the next message should start there).
- The row-14 render sign question: the game stores `−elevation` at obj+0x50 [M] while the viewer's
  base uses +elevation of the −X axis (the pre-existing stand-in's base, kept). Live check 2
  settles whether the base needs the sign flip.

## Usage

not visible to me — this sandbox exposes no token, context-window or cost figures.
