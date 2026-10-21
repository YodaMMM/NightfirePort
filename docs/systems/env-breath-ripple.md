# Env_Breath and Env_WaterRipple — cold-breath puffs and hit water rings

Task `unported-misc-3`. Both functions live in the `Env_*` weather/effects family
(`crates/nf-game/src/env.rs`, next to the snow/rain `Drops`). The gas pool API is
`crate::gas::GasPool` (`Gas_Init` 0x1aacc8's port); drawing the puffs is the main
agent's (the pool records are the getters: `pos`, `vel`, `mesh`, `scale`, `scale_rate`,
`rgba`, `rgba_rate`, `life`, `kind`).

## Env_Breath 0x17be00 (584 B) — ported `env::env_breath`

Called from `Player_Update` 0x193600 (≈0x193eb0; gated `BLData+0x94b == 0` and
`BLData+0x254 != 0`, pos = the head bone `AnimGetBoneWorldTrans(obj, −5, 0, …)`, dir =
`Mat_GetDir(obj+0x90)`, phase = `obj+0xEC` — the documented breath phase,
docs/fields/obj-tag-player.md:57) and `NDrone2_ControlSTANDARD` 0x1490f0:196-198 (phase =
`obj+0xEC + (obj & 0xfff)·0x80`) [M].

Gates [M disasm 0x17be1c-0x17bec8]:
- `cel != 0` and the cel's breath-zone byte `cel+0xa0` != 0 [M 0x17be1c-0x17be28];
- `(GameState+0x52 − phase) % 6 == 0` — the game divides the global FRAME_RATE_INT by 10
  (folded to the literal 10-divide at 0x17be30-0x17be40) [M];
- `(float)((GameState+0x52 − phase) % 120) <= (FRAME_RATE+FRAME_RATE)·0.33333334` = 40.0
  [M 0x17be60-0x17bec8].

Spawn at `pos + dir·0.05` (0x3dcccccd, the shared COP2 prologue), mesh **0x0200_021d**,
kind 0, life **40** (`sw $v0(0x28), ($sp)` at 0x17c010/0x17bf48 — the decompiler drops
this arg), velocity = the `dir` argument · REC_FRAME_RATE (`Gas_Init`'s non-kind-4
`param_6 != 0` arm [M 0x1aacc8 tail]) [M]:

- **Variant A** — env gate true AND cam mode 2 (`glb_viewer+0x236 == 2` [M 0x17bf7c]):
  p1 0.05 → p2 0.2, quaternion z = `Rand_FRand(6.2831855)` [M 0x17bf8c-0x17bfa0], rgba
  0xffffffc8 = (255,255,255,α 200) → 0x0000ff01 = (0,0,255,α 1) [M 0x17c018/0x17c020],
  and the gas object's `+0xf0 |= 0x400` draw flag [M 0x17c024-0x17c02c].
- **Variant B** — env gate false (any cam mode): p1 = p2 = 0.05 (static size [M
  0x17bf00-0x17bf08: f13 = f12 = 0x3dcccccd]), null quaternion (no yaw), rgba 0xffffff80
  (α 128) → the same 0xff01 end [M 0x17bf60].

The env gate is `GlobalEnv == 0 || GlobalEnv+0x4b != 0 || cel+0x3c & 0x10000000` [M
0x17bed0-0x17bef8]. The port models no `GlobalEnv`: `env_breath`'s `env_gate: Option<bool>`
takes `None` for the unported case, which takes the `GlobalEnv == 0` arm (true). The
cam-mode read and the `+0xf0 |= 0x400` flag are wiring points for the main agent (the
flag is the same class of draw-flag the port's `Gas` does not model — the blood puff's
`+0xf0 |= 8` deviation, gas.rs).

Wiring points: `Player_Update`'s call needs the port's bond tick (nf-view `step_bond`
owns Bond) with the cel's `+0xa0` zone byte and `BLData+0x254`; the guard call needs the
AI pass's cel + `obj+0xEC + (obj & 0xfff)·0x80` phase. Neither tick currently carries the
breath inputs — recorded as the follow-up with this function ready.

## Env_WaterRipple 0x17c118 (148 B) — ported `env::env_water_ripple`

Called from `Player_DealWithObjHit` 0x194458 (0x1944e4; after copying the 0x60-byte
HITDATA, `BL+0x50` material 0x0d/0x0e, `BL+0x60 & 4`) and `NDrone2_DealWithObjHit`
0x13fff0 (0x140060, same gate) [M].

Gate [M disasm 0x17c120-0x17c138]: hit non-null and `(HITDATA+0x50 & 0x3f) − 0xd < 2`
unsigned — the water surface-material pair 0x0d/0x0e. Attach-cel resolution
(HITDATA+0x54, else HITDATA+0x58 → +0x20, else abort [M 0x17c140-0x17c158]) is the
caller's — the port's `Gas` record carries no room/cel link (`Gas_Init`'s
`build_LinkToRoom` attach; [S] documented).

Spawn [M disasm 0x17c160-0x17c19c]: `Gas_Init(0.5, 1.01, cel, HITDATA+0x40, quat 0,
vel 0, 0x0200_0129 (RIPPLE_MESH), kind 1, rgba 0x7f7f7f7f, rgba_end 2, life 63)` — the
life the decompiler drops is `sw $v0(0x3f), ($sp)` at 0x17c180. White ring, α 0x7f → 2,
growing 0.5 → 1.01 over 63 ticks.

Wiring point: the port's actor-hit path (the per-hit dispatch the ledger records as a
standin, docs/verify/ledger.md:342 — `NDrone2_DealWithObjHit`/`Player_DealWithObjHit`)
does not yet carry water-surface HITDATAs; the function is ready for that dispatch.

## Related findings from this task (not this task's functions)

- The ambient `Ripples_Update` 0x1ac028 site (ported before this task) maps
  `Gas_Init`'s args differently: the game passes kind = t1 = **1**, rgba 0x7f7f7fff,
  rgba_end = t3 = **6**, life = **42** (`sw $v0(0x2a), ($sp)` at 0x1ac114), while
  world_fx.rs's ambient ripple spawns kind 6, no end colour, life = the placement's
  period. Behaviourally close (kinds 1 and 6 are both no-ops in `gases_smoke`'s motion
  switch), listed here so a render pass can reconcile the fade/size if the rings differ
  in PCSX2.
- `DAStateFunc_AimStand` 0x13e518 is the DAState-table's only entry ({id 0x0a → empty
  `return`}; table `DroneData_DAStateFunc` 0x2707f0) — nothing to port; cited at
  drone.rs `set_danim`.
- `Emitter_Kill` 0x17b960 is `control_funcs[0x32]`'s kill slot, an empty `return` —
  cited at world_fx.rs `EmitterFx`.
