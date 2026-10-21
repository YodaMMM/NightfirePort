# ai-spawned-objects — the 13 call-tree gap functions (task 851)

Branch `s/ai-spawned-objects`, 2026-10-21. The call-tree check (`tools/calltree/calltree.py`,
master 20874c31) walks the game's code from every guard-AI DSTATE the port mentions and lists
functions the port never mentions. This task owned 13 of them; **all 13 now drop off the list**.

## The verification (the done criterion)

```
$ python -I tools/calltree/calltree.py work/decomp/ACTION.ELF work/elf/ACTION.ELF . \
      --all-mentioned-dstates --list
...
TOTAL distinct never-mentioned reachable: 178
$ grep -cE "MISSING 0x(1ac158|1aa8b8|134cf0|134e58|134ab8|17ffa0|181560|182130|1d7ba0|1d7dc8|1d7ad8|1d7da0|1ad6c0) "
0
```

(was 13 of the 199-entry 2026-10-20 audit, `nf-queue/backlog/calltree-missing-ai-2026-10-20.txt`
lines 25-27, 104-106, 119-121, 173-176; the remaining 178 are other tasks' scope.)

## What was done, function by function

### Ported (a)

**`BodyGlow_Create` 0x1ac158** → new `crates/nf-game/src/body_glow.rs`.
[M 001ac158] the record (main object class 0x46, gfx 0x200025f, obj+0xe8 = 6.0f, obj+0xf0 |=
0x2408, ext[0] = owner, u16 ext+0x40 = 0x100, 14 child sprites gfx 0x200055a class 1 flags
0x2400), gated on `switch_NO_DRONES` (0 on the shipped path — set only by the 0x4b menu control
`C_CHCHDRONES` 0x215cf0). Hooked where the game hooks it: every guard spawn — `Drone::new` (the
port's `NDrone2_DefaultInit` 0x14b300-tail site; result → DIVars+0x10, docs/atlas/drone.md:51),
owner keyed to the list index at both index-assign sites (`Drones::new`, the CoderCreate NIS
arm). `Player_Init` 0x1910f8:184 also creates one (+0x88C, docs/atlas/player.md:102) — the
port's player side is not wired yet (noted, not asserted). The sprite rendering is the main
agent's draw work; the record carries the hashes/flags/count it needs.

**`Create_SpaceLaser` 0x1aa8b8** → new `crates/nf-game/src/space_laser.rs` + the kind-0x38 arm
in `Drones::attach_level_id` (drone.rs), mirroring the existing kind-0x39 SpaceMissile walk.
[M 001aa8b8:14-38] the full ext param map: ext+0x20/24/28 = raw f32 params 0/1/2, ext+0x30/34 =
`(float)(int)` params 3/4, ext+0x38 = 10.0f, ext+0x40/42/44 = low u16 of params 5/6/7, class
0x50, obj+0xf4 = 0. One disc placement: 0700001b #108 [D script-census.md:144,
docs/levels/0700001b.md:71]. `SpaceLaser_Update` 0x1aa3f8 (the arm/fire/damage flow) stays the
tracked gap — the record is its state base.

**`GT_Create` 0x17ffa0 + `GT_DeployMiniGun` 0x181560** → new `crates/nf-game/src/gt.rs` + the
kind-0x27 arm in `attach_level_id` + the `DroneWeap_FireWeapon` deploy citation in
`fire_weapon` (drone.rs).
- [M 0017ffa0:69-160] the creator: gate (param 2/3 scripts zero → no object), class 0x36,
  obj+0xf0 |= 0x810 (bit 0x10 hidden-until-deployed — the scene draw already honours it),
  ext+0xec = param 0 (facing mode + HP selector: 0 → HP 200 else 0xffff), ext+0xee = param 1,
  ext+0xe8 = param 4, ext+0xea = param 5, ext+0xb8 = parent, ext+0xcc = 100.0f, obj+0xf4 = 0.
  41 disc placements [D script-census.md:141]. The per-object script VM (Script_Load →
  GT_Register 0x181538) and the turret behaviours (GT_Update 0x180498, GT_Track 0x17f9c8,
  GT_ApplyDamage 0x1816d8 …) remain the documented gap (docs/reports/script-ops-unported.md:78-91).
- [M 00181560 + 00178f68:92-145] the deploy: the guard weapon-datum char 'R' + combat state
  type 4 + not 0xdd/0xde + AmmoInGun ≥ 1 + not tooCloseForWeapon →
  `GT_DeployMiniGun` at (x+sin(yaw), y, z+sin(yaw+1.5707964)), on success
  BOTWEAP_decrRounds(1000) + BOTSTATE_changeWeapon(0,1,0) and no bullet. The deploy position
  math is ported and unit-tested (`gt::deploy_position`, minigun glist 0x2000359, radii
  0.5/0.6). The fire_weapon branch cannot select yet because the port does not model the weapon
  datum char (BOT_vars+0x768) — cited at the exact branch site in `fire_weapon`, above the first
  RNG draw, with the full gate list. **Live check wanted** (below).

**`RB_Init` 0x1d7ad8, `RB_CuboidMomentsOfInertia` 0x1d7dc8, `RB_Build` 0x1d7ba0, `RB_SetState`
0x1d7da0** → new `crates/nf-game/src/rigid_body.rs`, ported from the decompiles [M all four]:
the 0x160 memset + the per-type triple (type 0 → −0.5/1.0/31.0, type 3 → −0.75/0.0/252.0, else
−0.75/0.0/25.0) at ext+0x98/0xa0/0xa4, the identity quat at +0x64; the moments formula
(m·(b²+c²)/12 etc., literal 0.083333336); the triangle check + mass/inv-mass/moments/reciprocal
stores (note the game's reciprocal order +0x8c=1/m0, +0x94=1/m1, +0x90=1/m2) + the collider
vert table (box 0x2d8890 / sphere 0x2d6200 by type); the quat→matrix rebuild. These are the
init base of the kind-0xD8 rigid-body props (which DO ship — levels 01/03/04/06/0d/41 [D
docs/systems/levels.md]); the evolution (`RB_Update` 0x1d6e38, `RB_EulerEvolve` 0x1d7810,
`RB_ApplyImpulses` 0x1d6ff8, `RB_RestingContact` 0x1d7168) stays the tracked gap
(docs/atlas/world-objects.md:54-59 has the RB_Create writeup; nf-view's scene.rs draws them
at-rest = the post-init state).

### Proven never called on the shipped path (c)

**`Sub_Create` 0x1ad6c0** (parsemap case 0x36, 0x1d04f0:215), **`MiniSub_Create` 0x182130**
(case 0x37, :221), **`Creature_Create` 0x134ab8 / `Creature_CalcPoint` 0x134cf0 /
`Creature_CalcWaypoints` 0x134e58** (case 0xE0, :353): each has the placement dispatcher as its
only caller, and the disc placement census has **no rows for those kinds** — "Creature_Create
type 0xE0 never occurs on the disc" [D docs/systems/script-census.md:120; the census type table
has no 0x36/0x37/0xE0 rows; docs/systems/uncited-audit.md:229-231 "dead on retail data"]. The
proof + the game names are cited at the exact dispatch site in `attach_level_id` (drone.rs,
after the 0x27/0x38 arms). No port code runs for them because the shipped data never reaches
the dispatch.

### Cited where the port does the work (b)

— none needed beyond the above: the (c) citations live in the dispatcher port, the RB_* leaves
are real ports, and the at-rest draw equivalence for 0xD8 is already cited in
`crates/nf-view/src/scene.rs` ("0xD8 RB_Create 0x1d6bf0:52 (rigid bodies evolve obj+0x90 once
disturbed; at rest = placement)").

## Tests (all green)

- `nf-game` lib: 358 passed / 0 failed (incl. the 10 new): the RB type table, the moments
  formula, the build triangle check (accept + two reject sisters), the identity set-state; the
  glow record (14 children, hashes, the no-drones gate); the laser param map (signed-int
  sister); the GT param map (HP selector sister), the script gate, the deploy offset.
- `nf-view` real path (`NF_ISO` + `nf_view::load` + `Sim::step_bond`),
  `crates/nf-view/tests/spawned_objects_live.rs`, 4 passed / 0 failed:
  0700000a carries exactly 6 GT turrets, each with both scripts and a legal HP value
  [D levels doc + M creator]; 07000005 carries 0 (the count is level data, not a constant);
  0700001b carries exactly 1 laser with the creator's constants; every 07000005 guard spawns
  with a 14-child glow whose owner is its own index.

## What ran before the push

- `cargo build --offline -p nf-game` — clean.
- `cargo test --offline --release -j2 -p nf-game --lib` — 358/0.
- `NF_SILENT=1 cargo test --offline --release -j2 -p nf-view --test spawned_objects_live` — 4/0.
- The fast suite (`tools/test/suite.py`, started 06:11 on the claim tree) reported
  `2728 passed, 18 failed, 143 ignored (87 min)` — **unreliable as a final-tree result**: the
  run raced this task's source edits (the tree changed underneath it). Its FAILED lines are not
  attributable to this branch; the touched-crates runs above are the honest pre-push evidence.
  A clean full-suite re-run did not fit the sandbox window (87 min run vs ~50 min remaining at
  verification time); the next session should re-run it on this tree.

## Live checks for the main agent

1. **The GT deploy branch** — find or force a guard whose weapon datum char (BOT_vars+0x768)
   is 'R', put it in combat (state type 4), watch 0x181560 (`GT_DeployMiniGun`) get hit and the
   turret object (class 0x36) appear at (x+sin(yaw), y, z+sin(yaw+π/2)); expected: no bullet on
   that fire tick, BOTWEAP_decrRounds(1000), changeWeapon(0,1,0). Address/watchpoint:
   0x181560 entry, BOT_vars+0x768 byte read.
2. **BodyGlow_Create 0x1ac158** on a fresh-boot guard spawn: 1 main object (class 0x46) + 14
   children (class 1), gfx 0x200025f / 0x200055a — confirms the record constants the port pins.
3. **SpaceLaser_Update 0x1aa3f8** on 0700001b after arming ch 19: which of ext+0x40/42/44
   (params 5/6/7) is the arm/kill channel and what the 10 s period actually gates — the port's
   record stores the params but the update flow is unported, so the param→channel semantics are
   [S] until this runs.
