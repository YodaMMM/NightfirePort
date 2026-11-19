# Drive suspension — the per-wheel spring–damper (DRIVING.ELF vehicle cluster)

Status: first port pass. Evidence: [M] code (function + address, IDA database of
`work/elfs/DRIVING.ELF`), [D] disc data, [S] suggested/unpinned. Complements
`drive-physics.md` (§1's step table) and `driving-world-bounds.md` (G-056). No PCSX2: the
remaining `[S]` scalars are marked for static resolution only (U-note at the bottom).

## 1. Where the params come from (corrects drive-physics.md §1)

The per-car controller `sub_188200` (0x188200, vtable+0x2c of the physics tree walk) **swaps the
pointer at `instance+0x23C`** to the current-params global `dword_3D0D10` before calling the
vehicle passes, and restores it after [M 0x188b48-0x188c54: save `v70`, write `v69`, call,
restore]. `dword_3D0D10` is the flat `.atr` block (the doc's "PVehicleParams"); the swap is the
tuning-DB indirection (mission/difficulty overrides). So "params at actor+0x23c" is a
**swap-slot**, not a fixed sub-object — the older note's wording is corrected here.

The `.atr` schema (drive-physics.md §2) registers via `sub_16E590(db=0x32E638, class,
name, getter=sub_16C1D0, offset, type, 0, 0)` [M 0x1975dc-0x1977c4 for the SPRING_* rows]:
`SPRING_STIFFNESS_FRONT/REAR` +80/+84, `SPRING_DAMPING_FRONT/REAR` +88/+92,
`SPRING_REST_LENGTH` +96, `SPRING_COMPRESSION_LIMIT` +100 — matching §2's +0x50/54/58/5c/60/64.

## 2. The suspension pass — `sub_18FC00` (0x18fc00)

Called per vehicle-kind (the cluster `sub_18BB90`/`sub_18C1B0`/`sub_18E548`/`sub_18ECB8`/
`sub_18FC00`/`sub_190510` are the kind implementations; FC00's skid-sound lookup names
`"small_snowmobi…"` [M 0x190338] — the kinds carry their own audio sets). All params reads go
through `*(a1 + 0x23C)` (the swap slot) [M; the only readers of `dword_3D0D10` are these
functions + the controller [M xref scan]].

Per wheel `i` ∈ 0..4 [M the `v76 < 4` loop, 0x1902e8]:

1. **Wheel position**: 16 bytes from the caller's array (`a5`, two quads per wheel [M 0x18fcb8]) →
   scratch `+0x250`.
2. **Rest**: `scratch+0x2EC = axle[3] + SPRING_REST_LENGTH` [M 0x18fce4; `axle` = arg `a4`
   (`_$S5`), floats at +4/+12 read — the per-axle descriptor]. 
3. **Ground query**: the world's `vtbl+532` raycast on the instance's collider adj [M
   0x18fcfc-0x18fd18, called with the wheel index byte; the doc's pass-B "quad/ray tests"].
4. **Compression**: `min(rest_term, SPRING_COMPRESSION_LIMIT)`, floored at 0 [M 0x18fd1c-0x18fd34:
   `min.s` against `+100`, then `max.s … 0.0`]. The exact operand the raycast contributes to the
   `min` is register-obscured [S: distance-vs-fraction; the two candidate forms noted in §5].
5. **Contact gate**: `if (Physics:Physical[0] >= *(hit-thing) * axle[1]) skip` [M 0x18fd80;
   `flt_331970` = the namespace base 0x331970 = **`PAD_DEAD_ZONE` 0.2** per drive-physics.md §2
   [M registration 0x185668]; operand roles [S]].
6. **Spring force** (compression > 0) [M 0x18fd9c-0x18fe08]:
   - `Δc = compression − prev[i]` (`prev` = the per-wheel array at `a1+528`, `v37`).
   - axle split by wheel index: `i < 2` → `STIFFNESS_FRONT`/`DAMPING_FRONT` (float-index
     `params[20]`/`params[22]` = bytes +80/+88), else `…_REAR` (`[21]`/`[23]` = +84/+92)
     [M 0x18fdd4-0x18fdcc].
   - **`F = compression · STIFFNESS + Δc · DAMPING`** [M 0x18fde8: `scratch+0x30C = F3·k + Δc·c`].
   - contact counter `scratch+0x330++` [M 0x18fda4].
7. **Force application**: the per-wheel world-space force vectors are built in the VU0 block
   (cross/dot chain over the wheel position, the axle up-vector, and the accumulators at
   scratch +0x280/+0x2A0/+0x260 [M 0x18fe30-0x18fe8x]) and applied after the loop through the
   physics force API (`S3+20 += scratch+804` [M 0x190314]; the accumulator twins
   `sub_1F84D0`/`sub_1F8888`). The VU0 macro lane bodies are not lifted (as in §6 of
   drive-physics.md) [S].
8. **Extra rear damping** [M 0x190320-0x19035c]: `max(SPRING_DAMPING_REAR, *(a1+576)·0.45)`
   × `−150.0` → a per-tick scalar (`scratch+0x274`) — the drift/handbrake wheel-damping term;
   operand roles [S].

**`SPRING_STIFFNESS_*` is read ONLY here** (as `v42[20]`/`v42[21]` — array-index syntax that
plain offset greps miss; the `driving-input`-era greps concluded "not lifted" — corrected). The
earlier drive-physics.md §4 bullet ("spring-damper per 0x331a60 collider") conflated this
pass's region with the boundary sheets — both corrections now on file.

## 3. The kind cluster

| function | size | params offsets read | role |
|---|---|---|---|
| `sub_18BB90` | 135 ln | +184 (×10) | small helper — the +0xB8 attr (`WS_SPEED`?) consumer; roles [S] |
| `sub_18C1B0` | 795 ln | +16, +56, +76, +96, +100, +140, +144, +164 | a kind's suspension variant (same loop shape) [M] |
| `sub_18CEF8` | 1093 ln | the engine/gear/steer set (§3 of drive-physics.md) | engine pass |
| `sub_18E548` | 444 ln | +16, +76, +96, +100, +204 | a kind variant; **+204 (0xCC) is an attr the port's schema table lacks** [M] |
| `sub_18ECB8` | 748 ln | +32, +108, +152, +204 | **wheelspin/tyre state**: `ω = |wheel v| / (WHEEL_RADIUS·6.32·dt)` [M 0x18f0f8], capped by the spin gate at 0.01 [M 0x18f110], handbrake lock-up `HANDBRAKE_FORCE·(speed·0.075+1.75)` [M 0x18f094] — feeds the skid/rpm systems |
| `sub_18FC00` | 499 ln | +16, +76, +92, +96, +100, +[80/84/88/92 as v42[20..23]] | the decoded suspension (§2) |
| `sub_190510` | 434 ln | +20, +28, +32, +152 | small kind variant (MAXACC/MAXBRAKE/MODSTEERSPEED/HANDBRAKE_FORCE) [S roles] |

New attrs the port's schema table lacks: **+0x10** (read ×4 across the cluster; sits between
`SPEED_CUTOFF_RATE_REVERSE` +0x0c and `MAXACC` +0x14), **+0x38** (×1), **+0xCC/204** (×3),
**`WHEEL_FORCE_APP_SCALE`**, **`CAR_WHEEL_X_OFFSET`/`ZF_OFFSET`/`ZR_OFFSET`** (registered via
`nullsub_6` on "pvehicle" [M 0x198218-0x198270]; the port's `CarParams` gains them as unresolved
fields when a consumer needs them).

## 4. The port

`crates/nf-game/src/driving/suspension.rs`: `SusWheel { compression, prev }` +
`SusParams { stiffness_f/r, damping_f/r, rest_length, compression_limit }` (the `.atr` values
ride `CarParams`), `step()` implementing §2.6's `F = k·x + c·Δx` with the clamp/gate, and the
axle split (wheels 0-1 front). `FlatWorld`'s ground makes the raycast analytic, so the `[S]`
raycast operand does not bite the flat-surface tests; the general raycast consumer is the
mission-runner work. Tests pin: the axle split, the k·x + c·Δx form with vanquish's `.atr`
values (80/98 · 400/350), the COMPRESSION_LIMIT clamp, the no-contact floor, and sisters that
prove each comparison can fail.

## 5. Static resolutions & remaining notes

- **Raycast return semantic** [M `sub_18E548`]: `*(hit+0x0C)` is added to `params+0x60` (rest length),
  then clamped to `[0.0, params+0x64]` (`max_travel`) and `max.s` into `+0x324` (`804`), confirming
  `hit+0x0C` is the signed hit height/compression offset relative to rest length.
- **`PAD_DEAD_ZONE` rescale** [M `sub_18CEF8`]: `(v - 0.3) * 1.4285715` (`1.0 / 0.7`) rescales `[0.3, 1.0]`
  to `[0.0, 1.0]`.
- **Wheelspin / surface smoothing** [M `sub_18BB90`]: averages the ground frame with `params+0xB8`
  (`WS_SPEED` sample count `n`): `((n - 1) * prev + cur) / n`.
- Remaining `[S]`: `axle[1]`/`axle[3]` descriptor layout and the kind cluster's per-vehicle split.
