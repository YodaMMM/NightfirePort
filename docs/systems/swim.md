# Swim — the substate-3 tick, the hittest capsule arms, and the pond settle

On the 02 pond (2026-10-18) the swimmer floated ~0.5 too high. PCSX2 truth [L, fresh
07000002 boot, Bond placed by PINE in the pond cel 0x1708870, stick forward held]: swimming into
the hut pocket under the hut ladder (centre (−74.84, ·, 22.22)) the PS2 root settles at y 3.17 at
(−76.27, 3.17, 22.14); in open water near (−71.1, ·, 27.9) it sits at y 3.48. The port rose to
3.70 / 3.60, its log showing the swim capsule touching the bed and pushing up (grounded at tick
12 of the swim-in). The root height `animobj+0xCC` matches both sides (PS2 1.0304, port 1.035),
so the capsule SIZE is not the cause.

## The settle is the bed push-out

Neither the swim move nor `Player_InWater` carries a buoyancy/sink term:

- `Player_Move` 0x191dd0:149-165 [M, substate-3 branch]: the forward speed rides
  `RotMatrixX(obj+0x50 − BL+0x8a8·π/2)` + `Mat_GetDir` — the look direction; with a level look
  the local move's Y lane is 0 and the stroke is planar.
- `Player_InWater` 0x195e50 [M]: on entry (`pos.y − 0.3 < cel+0x90`) it sets substate 3, zeroes
  `BL+0x14`, `obj+0x50` (pitch) and the anim move vectors, arms the 5-tick regrab — no vertical
  velocity, no drift.
- `Player_MonitorAir` 0x191a70 [M]: air drain/refill and the underwater fade (`BL+0x8C4`) — no
  position term.

The PS2 roots decode as the bed-capped rest: root = bed + `animobj+0xCC` — open bed
3.48 − 1.0304 = 2.4496, pocket bed 3.17 − 1.0304 = 2.1396. The swimmer strokes level and the
bed push-out (the hittest capsule against the pond floor) holds him there.

## The capsule switch decides the rest height

`Player_Collision` 0x194df0 builds one HITTEST per tick from a capsule switch **indexed on
substate−1** [M disasm 0x19506c-0x195098: `lhu 0xf6($s3)`, `addiu −1`, jump table @0x2fc0d0]:

| switch case | substate | radius `+0x7CC` | A `+0x760` | B `+0x770` |
|---|---|---|---|---|
| 0, 6 | 1 climb, 7 creep | 0x3e8ccccd (0.275) | pos + up·0x3f0ccccd (0.55) | pos + up·(0.275 − cc) |
| 3 | **4 crouch** | 0x3f0ccccd (0.55) | pos + up·(0.1 − (cc − 0.55)) | pos + up·(0.45 − cc) |
| 5 | 6 wire | 0x3f0ccccd (0.55) | pos + up·0.275 | pos + up·(2.1 − cc) |
| 0xb (no class-6 obj) | 12 | 0x3f0ccccd (0.55) | pos + up·0.275 | pos + up·(0.55 − cc) |
| **default** | **0 walk, 3 swim**, 5, 8, 9, … | 0x3f0ccccd (0.55) | pos + up·0x3e8ccccd (0.275) | pos + up·(0.55 − cc) |

The underwater path jumps **past the gravity straight into this switch** [M disasm
0x194f4c-0x194f70: `cel+0x90 > obj+0x34` → `bc1fl` skips `WldGravity` and the velocity
integration to the switch], so the swimmer's hittest capsule is the DEFAULT arm: r 0.55, spine
`pos + 0.275 … pos + (0.55 − cc)` — the lowest point exactly `cc` under the root. The game's bed
rest is therefore root = bed + cc — the [L] settles.

The gravity gate's own structure confirms the split [M 0x194df0 first switch @0x194e88]: the
default arm reaches the gravity block only through `cel+0x90 <= obj+0x34` (at/above the surface)
— a surfaced swimmer gets `WldGravity` and the `vel` integration; an underwater one gets neither.

**The port's bug:** `Bond::capsule()` (crates/nf-game/src/player.rs) handed the SWIM substate
the case-3 (crouch) LOW arm — the same misread `750a` already made for the swim grab pick and
fixed in fix-ladder-water2 ("750a mis-read case 3 as the swim substate; the swim substate 3
rides the switch's DEFAULT arm [M 0x195070 default]"). The LOW spine hangs `0.585` under the
root against the game's `0.485`: with r 0.55 the LOW capsule's lowest point sits `cc + 0.10`
below the root, so the same bed parks the port's root ~0.10 high (3.48 + 0.105 + the 0.0046 cc
gap = the observed 3.60 open-water settle). The port's own ladder pick already rode the default
arm in swim [M `ladder.rs::capsule_hits_default`] — the collision capsule disagreed with the
grab pick inside one struct.

## The swim tail (substate 3 of Player_Update 0x193600)

After `Player_Move(1.0, …)` [M 0x193600:181-182], with a cel and `AnimObj+0x60 & 8` [M
0x193600:186-222]:

- the local move is rotated to world, and the world-Y rise clamps at `surface − 0.3`:
  `pos.y < surface − 0.3` → `moveY = min(moveY, surface − 0.3 − pos.y)`; at/above the band any
  upward `moveY` zeroes — then back to local and integrated [M 0x193600:201-220];
- the exit: grounded and `cel+0x90 <= pos.y − 0.3` → `BL+0x130 = 2`, `ChangeSubState(0)` [M
  0x193600:189-194] — gated on the same `cel+0x90`, not on a "no water cel" shortcut.

## Tests

`crates/nf-view/tests/swim_height_02_live.rs` pins the two [L] settles through the real path
(`nf_view::load` + `Sim::step_bond`, the [L] PINE-placement protocol
`ladder_underwater_02_live` already uses): the pocket root 3.17 ± 0.1 at (−76.27, ·, 22.14) and
the open-water root 3.48 ± 0.1 at (−71.1, ·, 27.9), each with a sister (the substate must stay
SS_SWIM; the pocket must not latch the ladder). `fix_swim_live`, `ladder_02_live`,
`ladder_underwater_02_live` and `fix_ladder_water_live` must stay green.
