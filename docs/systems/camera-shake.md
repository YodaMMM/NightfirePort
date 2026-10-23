# Camera shake — `Camera_Shake` 0x1bc430 / `Camera_Update` 0x1bb6d8

The explosion camera shake, exactly as the game computes it. Everything here is [M] unless
tagged otherwise; the disassembly is quoted because the decompile MISREPRESENTS the store (see §1).
The port: `crates/nf-game/src/drone.rs` (`camera_shake_amount`, `Drones::viewer_pos`,
`Drones::shake`, `Drones::shake_off`, `Drones::camera_shake_tick`), the draw side
`crates/nf-view/src/lib.rs` (the play camera's eye). Branch s/camera-shake-exact (task 969).

## 0. The viewer's fields (viewer_tag, glb_viewer[4] @0x2d88b0)

- `+0x120` — the viewer's world MATRIX (16 f32); `+0x150/+0x154/+0x158` are its translation
  floats 12/13/14 (`0x120 + 0x30..0x38`), i.e. the camera's world position. [M
  `Mat_World2ViewMat` 0x1e3280 reads `param_1[0xc]..[0xe]` of the input matrix.]
- `+0x200` — the shake amount, a FLOAT (see §1). [M]
- `+0xc0..0xcc` — a copy of the (shaken) position `Camera_Update` writes every frame; the rest
  of the game reads the camera position there. [M 0x1bb6d8:47-50]

## 1. The set: `Camera_Shake(VECTOR* pos, float radius)` 0x1bc430

Per viewer (loops all 4 `glb_viewer` slots, `sltiu $s1,4` @0x1bc4a8/d8; the port has ONE
viewer — Bond's eye):

```
range = radius + radius            // add.s $f20,$f12,$f12      @0x1bc440
d     = Vec_Dist3D(pos, viewer+0x150)   // jal 0x1e2e48, $a1 = viewer+0x150 @0x1bc490-94
if (range < d)  skip               // c.olt.s $f20,$f0; bc1t    @0x1bc49c-a4  — NO store
inten = range·(1 − (d/range)²)     // div.s/mul.s/sub.s/mul.s   @0x1bc4ac-b8
if (inten <= 0) skip               // c.olt.s $f22(0),$f0; bc1f @0x1bc4bc-c4  — NO store
inten = min(inten, 10.0)           // min.s $f0,$f0,$f21        @0x1bc4d0 — word 0x46150029
viewer+0x200 = inten               // swc1 $f0,0x200($v0)       @0x1bc4d4 — a FLOAT store
```

`$f21` = 10.0f (`lui $at,0x4120; mtc1 $at,$f21` @0x1bc474-78), `$f23` = 1.0f, `$f22` = 0.0f.

**The store is a float, NOT the integer bit pattern.** The decompile prints
`(int)fVar3 * (fVar3 < 10) | (int)10 * (fVar3 >= 10)` because Ghidra's EE SLEIGH expands the
EE-specific `MIN.S` (COP1 fmt=S, funct 0x29 — capstone cannot decode it in any mode) into
INT_MULT/INT_OR p-code. Identification: the same unknown word appears 125×; at
`AINetwork_DoAStarPath` 0x157340/344/350 (words 0x46010029/0x46020029/0x46030029 — fd=f0,
fs=f0, ft=f1/f2/f3) the C shows a min chain over the four loaded floats, and 0x157360's funct
0x28 (MAX.S) shows the max arm — the multiply-by-boolean idiom IS the min/max semantic. So:
no `cvt.s.w` before the `swc1`, no denormal — **the PS2's shake is a real float, visible**,
e.g. radius 3 / d 1.5 → **5.625**, not the truncated 5 the old port formula produced.

Consequences the port must keep:
- No integer truncation (the old `camera_shake_amount` `(inten as i32).min(10)` was wrong).
- A skip leaves the field UNCHANGED (outside the range or ≤ 0 → no store) — the port's
  `Option<f32>`: `None` = do not touch `Drones::shake`.
- Overwrite, not max: a second blast in the same tick replaces the amount (last blast wins).
  [M the store has no compare-against-old]
- `Vec_Dist3D` 0x1e2e48 is scalar IEEE: `add.s (x²+y²)+z²` @0x1e2e74-78, `sqrt.s` @0x1e2e7c —
  the port's plain f32 dist is bit-exact.

Callers [M]: `Explode_Create` 0x17cbd0 (the blast path — gate `radius ≥ 0.5`, `bc1f`
@0x17cc28 → the call @0x17cc40; the port's `Drones::spawn_blast`), `Mine_Update` 0x17c930:105
(the port's mines reach the same value through `spawn_blast` → the same formula/inputs),
`Bullet_DoTrails` 0x12d890:291 (`Camera_Shake(2.0f, gas_pos)` — the gas arm; the port's
flashbang/gas site, radius constant 0x40000000 = 2.0).

## 2. The update: `Camera_Update(uint viewer)` 0x1bb6d8:35-43, once per Game_Run tick

`Camera_UpdateAll` 0x1bbd68 loops the viewers each Game_Run tick and runs
`Camera_Update(i)` + `Camera_CheckLocation(i)` + `Camera_UpdateGlbVars(i)` per non-null slot
[M 0x1bbd68:11-19]. Inside `Camera_Update`, after the cam-mode dispatch
(`Player_PositionCamera` 0x18f5d8 — which REWRITES viewer+0x150 every frame [M 0x18f5d8:275,
16-byte store] so the offset never accumulates) and the shear check:

```
if ((shake = *(f32*)(viewer+0x200)) > 0.0) {
    Rand_FRand_MVar2_Vec(shake·REC_FRAME_RATE, shake·REC_FRAME_RATE, viewer+0x150);
    shake -= Rand_FRand(shake·0.25);
    if (shake < 0.0) shake = 0.0;
    viewer+0x200 = shake;
}
```

**The RNG draws — exactly 4 per tick while shaking, 0 when shake == 0:**

1-3. `Rand_FRand_MVar2_Vec` 0x1e00d0: **three pair-draws of the shared stream** (one
   `SEED_X` step + one `SEED_Y` step each — the same shape as `Rand_FRand` 0x1e3780), in
   **X, Y, Z order** (draw 1 → axis 0 @0x1e0178-a4, draw 2 → axis 1, draw 3 → axis 2; each
   axis value = `uVar_x·2^16 + (uVar_y & 0xffff)` through the cvt.s.w trick, × 2⁻³²
   @0x1e014c-58). Per axis:
   `axis += param_1·u − param_2` — `mul.s $f0,$f12,$f1` @0x1e0170, `sub.s $f0,$f0,$f13`
   @0x1e0188, `add.s $f1,$f1,$f0` + `swc1` @0x1e0198-a4. With param_1 = param_2 =
   shake·REC_FRAME_RATE (REC_FRAME_RATE = 1/60 [M grenade.rs:148, the port's established
   constant]) each axis gets a **one-sided uniform offset in [−shake/60, 0)** on the WORLD
   axis — the shake biases the eye towards −X, −Y, −Z, up to 0.1667 per axis at the cap 10.
4. `Rand_FRand(shake·0.25)` @0x1bb6d8:38 — ONE draw; the argument re-reads the pre-decay
   +0x200. `shake -= draw` → uniform in (0.75·shake, shake] — expected ×0.875 per tick. From
   10.0 the offset amplitude (≤ 0.167/axis) fades below visibility in roughly 30 ticks. The
   decay never reaches exactly 0 (geometric; the `if < 0` floor @:41-43 only guards the float
   edge) — the port replays the same arithmetic so the shared stream keeps the same cadence.

The port's `Rng::frand` (drone.rs) already reproduces the pair-draw bit-exactly (the x/y
steps, the cvt truncation trick, the two `mul.s`), and `effect::frand_mvar2` is
`sub(rng.frand(a), a)` = the MVar2 axis step. The block consumes them in the game's order:
3 offset draws, then 1 decay draw, all on `Drones::rng` (the shared stream).

## 3. How the offset reaches the picture

`Mat_World2ViewMat(viewer+0x120, viewer+0x160)` @0x1bb6d8:45-46 rebuilds the view matrix
from the world matrix **whose translation IS viewer+0x150** [M 0x1e3280: the out translation
= `-(R·t)` over `param_1[0xc..0xe]`], then copies the shaken position to `+0xc0..0xcc`
@:47-50. So the offset is a **world-space translation of the eye** (units = world units,
axes = world X/Y/Z) — not a view-space roll/jitter — and everything that reads the camera
position (+0xc0) sees the shaken value that frame.

The port (fix-922 had unwired all of it; `Drones::shake` was written with the [S]
`max(radius)` / `×0.9 − 0.05` shape and read by nothing, and nf-view had no reader at all):

- `Drones::viewer_pos` = the +0x150 port: the Sim writes `bond.head() + shake_off` at the top
  of `Sim::step_bond`/`step` (the eye as of last tick's end + last tick's offset — the
  game's +0x150 between Camera_Updates); `spawn_blast` and the gas site measure d against it.
- `Drones::camera_shake_tick()` = the §2 block, run at the drones tick's tail (where the old
  [S] decay sat); it writes `Drones::shake_off` (zeroed when shake == 0).
- The draw side adds `sim.drones.shake_off` to Bond's eye before building the view
  (`Camera::pos`, lib.rs) — the first-person branch, world axes, no rotation change.
- The game shakes every viewer that exists (script cameras included); the port models one
  viewer, so the script-camera views are not shaken [S — the single-viewer stand-in].

## 4. Live checks for the main agent (only if the disassembly reading above needs
confirmation — the [M] record already settles all three task items)

1. FRESH boot (not a save-state resume), 07000005, throw a frag at a wall ~4 units away:
   watch viewer+0x200 (write watchpoint) — expect a write from `swc1` @0x1bc4d4 with a value
   in (0, 10], NOT the bit pattern of a small integer (e.g. 5.625 = 0x40B40000, not 5 =
   0x00000005); then per-frame reads: the value decays ×~0.875/frame to invisible in ~30-45
   frames.
2. Same pop: viewer+0x150 x/y/z right after the `swc1` writes and the MVar2_Vec block —
   expect ALL THREE axes ≤ their pre-pop PositionCamera values (the one-sided negative
   offset), each within 0.167 of it at shake ≈ 10.
3. The felt test (the merge gate): a grenade pop ~4-6 units away should read as a short
   (~0.5 s) dip/pull of the camera towards −X/−Y/−Z, not a centered jitter — if it reads as
   nothing, the draw-side eye add is the first suspect; if it reads as a slide, it is the
   game's own one-sided offset.
