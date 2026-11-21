# Driving: particle effects, wheel smoke / spray / sparks, glare

Port: `crates/nf-formats/src/drivefx.rs` (data), `crates/nf-game/src/driving/effects.rs` (logic).
Disc check: `crates/nf-view/tests/driving_effects_live.rs`.

Tags: [M] DRIVING.ELF decompile/disassembly (EE = file offset + 0x107080), [D] disc bytes.
Nothing here is a live capture. The port produces draw items; nothing draws them yet.

## 1. Where the particle types come from

- The library loader `FUN_001e7f60` builds `"%sparticle.dat"` but never opens it; it adds the
  `.data` record at 0x3351c0 under the key `'dflt'` [M]. No `particle.dat` is on the disc [D].
- The vehicle-effect init `FUN_001edf38` adds nine more `.data` records [M]:

| key | EE | name (+0x24) | life | count | flags | notes [D] |
|---|---|---|---|---|---|---|
| dflt | 0x3351c0 | smk5 | 80 | 200 | 8 | |
| SMRT | 0x335440 | smk5 | 60 | 30 | 8 | tyre smoke: speed 0.015, drag 0.96, size 0.2 → 0.7 |
| RDDF | 0x3354f0 | puff | 20 | 20 | 8 | dust |
| RDDR | 0x3355a0 | puff | 15 | 20 | 8 | dirt |
| WATR | 0x335650 | grvl | 80 | 30 | 8 | gravity 4 |
| RDGR | 0x335700 | gras | 60 | 30 | 8 | gravity 4 |
| SNOW | 0x3357b0 | puff | 15 | 40 | 8 | |
| SPRK | 0x335910 | smk5 | 24 | 40 | 0x22 | streak, gravity 30, window 3+1 |
| SPKB | 0x3359c0 | smk5 | 24 | 60 | 0x22 | |
| SPRL | 0x335a70 | smk5 | 24 | 200 | 0x22 | |

  (0x335860, a `snw1` record, has no registering call.) Track effect parts add theirs through
  `FUN_00235ed0` under keys `n + 5` (§7).
- Library: 256 slots of 0xb0 bytes, a tree map key → slot (`FUN_001e7cf0`); an unknown key
  returns the smallest key's slot (`FUN_001e7db0`) [M].

### The 0xb0 record [M — the readers named in drivefx.rs]

+0x00/+0x08 spawn spread x/z, +0x04 speed, +0x14 speed variance, +0x10/+0x18 side/third-axis
variance, +0x0c count, +0x1c flags, +0x20 drag, +0x24 name, +0x2c a byte for the particle,
+0x30 1/life, +0x34/+0x38 sizes, +0x3c life (steps, capped 255), +0x40/+0x44 emission window,
+0x48 pre-warm, +0x4c/+0x50 gravity and variance, +0x54 spin variance (degrees), +0x5c max size,
+0x60 atlas cell (u, v, size), +0x70/+0x80/+0x90 colour keys (A, R, G, B; 512 = 1.0),
+0xa0/+0xa4 size/alpha randomisers, +0xac library slot.

Flags: 0x02 emit forever, 0x08 rotate, 0x10 fixed spin, 0x20 streak, 0x40 alpha nibble, 0x80 size
nibble, 0x100 particle bit 0x40, 0x200 additive bucket.

### Preparation when added (`FUN_001e80b0` + `FUN_001f3188`) [M]

max size = max(size0, size1); flag 2 → window length 99999; life capped at 255, 1/life stored;
count halved (truncated); unless streak: size0' = size1*1.4142, size1' = (size0 - size1)*1.4142,
so size = size0' + t*size1' runs from the birth size at t = 1 to the death size at t = 0; spin →
radians, flag 8 when spin != 0 or flag 0x10; colour keys /512, alpha halved, key 0 ↔ key 2 (the age
counts down).

### Atlas cells (`FUN_001ede48`) [M]

The particle buckets bind the 256x256 `MAIN` texture of the track's particle sheet [M
`FUN_001f34d8`: `'MAIN'`; D: every sheet has a 256x256 `MAIN`]. The vehicle types pick their cell
by comparing names: smoke (and WATR) (96, 160, 32), puff (128, 64, 64), gras (160, 224, 32), in
texels; u = (x + 0.5)/256, size = (n - 1)/256.

Sheets [D]: MIS01/RACE `partic01.ssh`, MIS11 `partic02.ssh`, MIS13A/B/C `ears_partic01.ssh`,
MIS3/MIS4 `sav_partic01.ssh` (names also at EE 0x3ac750-0x3ac798). Besides `MAIN` they hold 1x1
named entries (`smk5`, `puff`, `sprk`, `strk`, `exp1`…) whose role is not traced.

## 2. The pool and the step [M]

Two buffers of 1024 x 32-byte particles (`RParticleSystemManager` + 0x40). New particles go into
the other buffer (`FUN_001f3bb8`, current + pending < 0x400). A step (`FUN_001f3ca8`) per
particle: pos.xyz += vel (vadd.xyz, 0x1f3d4c); vel.xyz *= drag (vmulx.xyz, 0x1f3d58); vel.y +=
gravity_byte * -1/4096; age -= 1; survivors are appended to the other buffer; then swap.

Frame (`FUN_001f47f8`): steps = tick - last (unsigned), 3 if that is >= 4; then the draw build.
The tick is DAT_00334010 (copied from DAT_00339f54); **its rate is not traced**.

Particle slot: +0x00 pos, +0x0c age, +0x0d flags, +0x0e gravity (i8), +0x0f type, +0x10 vel,
+0x1c nibbles, +0x1d drag*255, +0x1e attached object (0xff none), +0x1f spin*40.7.

## 3. Spawn (`FUN_001f29d8`) [M]

Random draws from the engine's shared generator (seed 0x3ade68b1, multiplier 0x75bcd15,
`FUN_001c08c0`; r = bits 8..23 of state*mul, state = low 16 bits), in this order: x spread, z
spread, speed, side, third axis, gravity, spin (then the two randomisers when enabled). Velocity =
A*speed + B*side + C*third (+ carrier/60). Age = life - round(frac*frames), 0 → 1. Then the
position is moved by delta*frac + vel*frames*(frac - 1) and gravity*(frac*frames)^2, so particles
owed earlier in the interval sit where they would have been.

Type flag 0x10's screen-space spin (camera matrix + atan2) is **not ported**; no traced type
sets it.

## 4. Emitters (`FUN_001e5e10`, `FUN_001e5fa0`, `FUN_001e5cd8`, `FUN_001e65c0`) [M]

Rate per step = count/life, or count/delay inside a window (see `emit_rate`). Each tick: owed +=
rate*scale*frames; a type whose max size / distance < 0.004 emits nothing; else one spawn per
whole particle owed (emitted starts at 1.0). Window: once age > delay, nothing is emitted; past
delay + length the age wraps, and a flag-2 type's emitter stops.

## 5. Vehicle effects (`FUN_001ee238`, `FUN_001ee4a0`, `FUN_001eed18`) [M]

Fourteen slots per vehicle from the table at EE 0x393870 [D]: 0-3 tyre smoke, 4-7 surface spray,
9-12 sparks. Per wheel:

- Surface (vtable +0x1fc & 0xf) → spray type from EE 0x3938a8 [D]: 2 and 5 RDDR, 3 RDGR, 6 WATR,
  9 SNOW, others none.
- No spray type: tyre smoke when slip (vtable +0x21c) >= 0.2; scale = (min(slip, 1) - 0.2)*4 +
  0.1; main axis = (vel.x, vel.y + 0.8, vel.z).
- Spray type: value = slip, or speed/10 clamped 0..1 for SNOW, speed/14 clamped 0..2 for RDDR
  (both when vtable +0x214 is true); emits when >= 0.3 (capped at 1 except RDDR); main axis =
  vel - car_axis*3.66 in x/z, vel.y + 0.8.
- Sparks on surfaces 1, 2, 4, 10, 12 when both wheel flags are true and |vel|^2 > 5, and
  `Enable vehicle sparks` is on: direction d = -2*vel (the 2.0s at EE 0x335420 [D]), y +=
  0.2*max(|d.x|, |d.z|); stored relative to the car (attached), restarted each time.

Which quantity each vtable slot returns (slip, speed, the two flags) is inferred from use and
**not traced**; the port's physics does not feed these yet.

## 6. Draw build (`FUN_001f3dc8`) [M]

t = age/life. Colour = key 0→1 over t <= 0.5, 1→2 above; packed *255 truncated, R G B A.
Quad: centre (+ attached object position), size = size0' + t*size1' (or nibble/1.5), rotation =
-pi/4 without flag 8, t*spin/40.7, or spin/40.7 with flag 0x10, cell from +0x60. Streak (flag
0x20): head = pos + vel*size1, triangle (head + (0, size0, 0), head, pos) on `sprk`. Order per
512-chunk: default bucket, additive bucket (GS ALPHA 0x48, FIX 0x80), streaks. The default
bucket's blend and the VU1 quad expansion are **not decoded**.

## 7. Track effect parts (`ef#`)

`FUN_002360c0` drives a track effect: it registers the part's particle record (the definition at
+0x30) through `FUN_00235ed0`, which copies 0xa8 bytes, scales three fields by 127/360/127 and adds
it under key `count + 5`; the instance is an emitter whose distance is the camera distance. The
`ef#` payload layout on disc is **not decoded** (docs/formats/geoprim.md §3 still lists the tails
as unresolved).

## 8. Tunings [D]

- `Render\FX\<track>.tun` (MIS01 `paris_mis01`, MIS11 `uw_mis11`): `Doglow`,
  `Enablevehiclesparks` (= DAT_00335418, the sparks gate), `EnableParticulate`, water scale /
  scroll / sun direction, `Lightning*`, `Shadowwidth/length{car}`, `BulletStreak*{class}`
  (colour, texture, width, length, speed, count for Henchmen, Vanquish, SuperSnowmobile, SUV,
  Captain, Ultralight, Sentinel). Only the sparks gate's reader is traced; the rest are data.
- `Render\Glare\default.tun` (identical in all eight archives): 80 glare types, halo + spike
  (tex, spin, colour, size) and `ZBias` [M names 0x394ee0-0x394f60]. IDs 64, 76-79 hold the
  0xcbcbcbcb fill. Cell per tex index `FUN_001f19f0` [M]: 0/1 = top halves, 2.. = quarter cells.
  Pulse `FUN_001f1b60` and the 1.6 size cull `FUN_001f1d18` are ported; the glare instances come
  from track data that is **not decoded**, and the halo/spike draw is **not ported**.
- `Render\SwitchEffects\default.tun` (identical everywhere): 16 channels (Booster, Headlights,
  Brakelights, …) x `VScroll`, `Glow`, `Illuminate`; data only.
- `data\render\decal.dat`: the loader copies 0x280 bytes (`FUN_001da3b0`); 8 x 0x50 records of
  four (u, v, 1, 1) corners + four words. The words' meaning is **not traced**.

## 9. Not covered

Explosions and muzzle flashes: no explosion or muzzle-flash emitter was found among the traced
callers; the sheet's `exp*` / `fir*` entries are presumably used by track effects or weapons,
**not traced**. `RParticulate` (weather), lightning, water and shadows are not ported.
