# Driving: particle effects, wheel smoke / spray / sparks, glare

Port: `crates/nf-formats/src/drivefx.rs` and `crates/nf-formats/src/trackfx.rs` (data),
`crates/nf-game/src/driving/effects.rs` and `crates/nf-game/src/driving/trackfx.rs` (logic).
Disc checks: `crates/nf-view/tests/driving_effects_live.rs`, `crates/nf-formats/tests/trackfx_disc.rs`
(every placed effect per track with its kind), `crates/nf-game/tests/drive_trackfx_disc.rs`.

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
named entries (`smk5`, `puff`, `sprk`, `strk`, `exp1`…) whose role is not traced: no traced
reader looks a sheet entry up by a particle name (types carry their own `+0x60` cell, the
Giotto types included), and `FUN_001f34d8` binds `MAIN` from the particle sheet and `sprk` from
the common one. Unproven: that the 1x1 entries are unused.

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
512-chunk: default bucket, additive bucket, streaks. GS ALPHA per bucket [M `FUN_001f34d8`]:
default 0x8000000044 = (Cs - Cd)*As + Cd, plain alpha blending (`FUN_001a0f80` on 0x3e3f70);
additive 0x8000000048 = Cs*As + Cd; streaks 0x8000000068 = Cs*FIX + Cd with FIX 0x80
(`FUN_001a0fb8(0x3e8070, 0x80)`). The VU1 quad corner expansion is **not decoded** (the
microcode is not disassembled).

## 7. Track effects: `ef` lists, glares, Giotto effects

Code: `nf_formats::trackfx` (`placed_effects`, `giotto_effect`, `giotto_gallery`),
`nf_game::driving::trackfx::TrackFx` (per tick: glare sprites, load-started particles).

**Where the lists are** [M `FUN_0022faf8` + D]. An `as` part payload is `{'Base', 'ef'#n,
'an'#n, ...}`: `+4` names the part's `ef` list for model parts as well as effect-only parts (the
traffic light's lamp model carries its glare). Entries are 0x40 bytes, ended by a 0 type byte.
- Static items (no in#0 flag 0x10) use the list of the part they name. Init World flags the
  entries with bit 0x10 or type 2; type 6 with bit 0x10 starts its effect at load. The static
  draw `FUN_001ad1a0` runs flagged type 2 as a glare (if DAT_00333784: 1 in `.data`, no writer
  found) and leaves type 6 where it started (`FUN_00233190` is empty) [M].
- Scene objects are built from their article's own `in#(ps +7)` list (`in#0` when missing,
  `FUN_001c6fb8`); their effects are the lists of those sub-items' parts (`FUN_001a1f58`), and
  `+0x15` names the sub-item whose matrix places the entry [M]. `FUN_001ca930` runs type 2 as a
  glare with distance 0, type 5 as a point light, 6 and 8 as nothing; type 6 starts on object
  events (`FUN_001ca198`), e.g. smash sparks [M]. `FUN_001a1f58`'s entry filter `FUN_001a1bc8`
  is **not traced**; the census lists every entry. The world position composes sub-item then
  item matrix; `FUN_001c9368`'s exact composition is not traced.

**Entry fields** [M `FUN_001f1d18`, `FUN_001f1b60`, `FUN_00232af8`, `FUN_00233460`]: `+0x00`
position, `+0x10` flags (0x10 load, 0x20 one-sided, 0x400 pulse, 0x800 triangle, 0x4000
shaped, 0x2000 ignore the part bit), `+0x14` type, `+0x15` sub-item, `+0x16` part-mask bit,
`+0x18` glare type or `sr` -> `GIOT::<name>`, `+0x1c` size, `+0x20` facing (glare) / quaternion
(type 6), `+0x2c` phase, `+0x30..0x3c` rate, duty, base, amplitude. Types on the disc: only 2
and 6 [D].

**Census** [D, `trackfx_disc.rs`]:

| track | static glare | static type 6 at load / not | scene glare | scene type 6 | Giotto names missing from the gallery |
|---|---|---|---|---|---|
| MIS01 | 351 | 10 / 1 | 252 | 708 | none |
| MIS11 | 26 | 0 / 0 | 208 | 76 | Un_SandR |
| MIS13A | 23 | 13 / 0 | 84 | 90 | Wd_steambig, dummy |
| MIS13B | 12 | 16 / 6 | 85 | 171 | Ex_Oil, dummy |
| MIS13C | 0 | 9 / 0 | 49 | 115 | Ex_Oil, snowbridgeexplosion |
| MIS3 | 79 | 2 / 0 | 16 | 158 | snowlandingdeep |
| MIS4 | 16 | 0 / 0 | 86 | 121 | sm_splinter |
| RACE | 16 | 0 / 0 | 84 | 121 | Sm_Fruit, sm_splinter |

A missing name logs "Could not locate Giotto effect" and that effect does not appear
(`FUN_00232e18`) [M]. Paris examples: `P_walllamp`, `bridge_lamp_*`, `EiffelLight`,
`University` static glares (types 69, 70, 52), `P_streetlight` scene glares (type 68) plus
`Wd_Sprk` / `Wd_Glass` on a smash, `sawhorse` beacons (type 36, pulse 0.02 / 0.5 / 1 / 3.5),
`P_Building` chimney smoke `Wd_Smoke` started at load. Started at load elsewhere: MIS13A
`Wd_Fnt2`, `watermist`; MIS13B/C `waterfall`, `watermistB1..B3`; MIS3 `fireplacebig`,
`fireplacesmall`. No track places a type-6 effect at load on MIS11, MIS4 or RACE.

**Traffic lights** (paris, 5 scene objects) [D through the M readers]: one glare each, type 49
(halo "255,96,0,0" red, spike "255,200,20,5"), flags 0xc34: one-sided, pulse rate 0.02 per
tick, duty 2/3, base 0.5, amplitude 1, triangle. The light flashes red with a 50-tick period: it
ramps 0.5 -> 1.5 -> 0.5 over ticks 0-33 and is dark for ticks 34-49. The article's three `in#`
variants (`as#0`/`#2`/`#3` -> `Model[0].00/.01/.02`) all carry the same glare list; nothing in
the traced code switches between them over time, so this data holds no red/amber/green cycle.
Whether the three models differ in colour was not checked.

**Glare instance** (`FUN_001f1d18`) [M]: size x pulse (flag 0x400; floorf fraction,
`FUN_002bfb50`); kept when `dist / (size*pulse) <= 1.6`, where the static path passes the
squared LOD distance `(max(0, |camera - item origin| - r) * view[+0x30])^2` (`FUN_001ad0b8`;
r = `(w15 & 0x3ff) * {0.25, 16}[bit 30]`, `FUN_001ad128`) and scene objects pass 0; one-sided:
size * -dot(camera +0x20 row, facing) - 0.5, dropped at <= 0, then doubled; at most 128 a
frame, counted in item draw order. **Not traced**: the view word at +0x30, which way the camera
row points, and the view cull that decides which items are drawn (the port takes a `drawn`
predicate).

**Glare draw** (`FUN_001f2058`) [M]: per instance a halo then a spike from the type entry
(manager + 0x2038 + 0x40*id: cells, spins, colour words, sizes, ZBias); size_L = type size x
instance size; centre = pos - ZBias*size_L*normalize(pos - camera); w = size_L * 1.4; colour =
type colour & 0xffffff | alpha << 24, alpha 0x20, or trunc(size * 32) when one-sided; rotation
= frac(spin * distance / 2pi) * 2pi - 3.93 (truncating fraction). Colour words follow
`FUN_00144740` ("a,b,c,d" -> a<<24 | d<<16 | c<<8 | b; the colour tuner `FUN_001439a8` calls
it). GS state: ALPHA 0x8000000048 (additive), alpha test >= 0x70 with AFAIL = RGB only
(`FUN_001a0f30(..., 0x70)`). ZBias defaults to 2.0 (0.3 for ids 33, 34, 52) and the disc's
tuning sets it for all 80 ids.

**Giotto effects** [M]: `GIOT::<name>` is a `GANM` of the effects gallery `FUN_0024b7d0`
picks by the track file name: `sav_gall1.gal` for snow1a_mis3 / snow2a_mis4, `ears_gall1.gal`
for junglea/b/c_mis13x, else `gall1.gal` (RACE's `snow2a_race` is not in the snow list). The
match is case-insensitive (`FUN_002339e8` -> `FUN_002a9198`). The instance (`FUN_00234d80`)
sits at item origin + entry offset with identity orientation (static path, `FUN_00233460`) and
runs its elements for frames 0..=LIFE (`FUN_002345d8`): `gp` emitter (`FUN_002360c0`), `gs`
sound, `gg` glare, `gd` debris/decal burst (`FUN_00235810`, once).

A `gp` key [M `FUN_002360c0`, `FUN_00235ed0`]: `+0x00` offset, `+0x10` quaternion (axes =
rotate (0,1,0) and (1,0,0)), `+0x30` a 0xb0 particle record, copied with gravity and its
variance x 127 and spin x 360 and added once per element under key `n + 5`; `+0x2e > 0` writes
the previous slot into `+0x2d`; `+0xd8` start frame, honoured when `+0xdc` is 0. The emitter
distance is |frame - camera|; a forever type is switched on at the first frame. Ported for the
static load-started effects.

**Not ported**: `gg`, `gd`, `gs`; event-started scene-object effects; the point light (type 5,
none on the disc); how the particle manager ticks the Giotto emitters (`FUN_001e65c0` is reached
through a vtable; the port ticks each once per call); the frame counter's rate; the
quaternion handedness (only identity orientations were checked).

## 8. Tunings [D]

- `Render\FX\<track>.tun` (MIS01 `paris_mis01`, MIS11 `uw_mis11`): `Doglow`,
  `Enablevehiclesparks` (= DAT_00335418, the sparks gate), `EnableParticulate`, water scale /
  scroll / sun direction, `Lightning*`, `Shadowwidth/length{car}`, `BulletStreak*{class}`
  (colour, texture, width, length, speed, count for Henchmen, Vanquish, SuperSnowmobile, SUV,
  Captain, Ultralight, Sentinel). Only the sparks gate's reader is traced; the rest are data.
- `Render\Glare\default.tun` (identical in all eight archives): 80 glare types, halo + spike
  (tex, spin, colour, size) and `ZBias` [M names 0x394ee0-0x394f60]. IDs 64, 76-79 hold the
  0xcbcbcbcb fill. Cell per tex index `FUN_001f19f0` [M]: 0/1 = top halves, 2.. = quarter cells.
  Pulse `FUN_001f1b60`, the instance `FUN_001f1d18` and the halo/spike build `FUN_001f2058` are
  ported (section 7); the instances come from the track `ef` lists.
- `Render\SwitchEffects\default.tun` (identical everywhere): 16 channels (Booster, Headlights,
  Brakelights, …) x `VScroll`, `Glow`, `Illuminate`; data only.
- `data\render\decal.dat` (identical on all 8 tracks [D]): the loader copies 0x280 bytes into
  the decal manager + 0x10 (`FUN_001da3b0`, `FUN_001da030`); 8 x 0x50 records of four (u, v, 1,
  1) corners + four words. The add `FUN_001da6c8(width, height, manager, centre, dir, normal,
  record)` [M] builds two triangles (rows 0,1,2 and 1,2,3) on a 96-slot ring: a = (normal x
  dir) * width, b = ((normal x dir) x dir) * height, corner 0 = centre - a/2 - b/2. Word `+0x44`
  = the number of decal draws (`FUN_001da978`, one decrement each) before the decal gets its
  alpha (0 means 1); `+0x48` = that alpha (0x3f, 0x20 or 0x28 on the disc); until then the
  colour is 0x007f7f7f (alpha 0). Words `+0x40` and `+0x4c` are not read by either (record 7's
  `+0x40` holds the bytes "data", leftover text). `nf_formats::trackfx::decal_style`. The
  callers (`FUN_001ea8e8`, `FUN_001eb078`) are impact code, not track placements: no track
  places skid marks or oil from `decal.dat`. The decal ring itself is not ported.

## 9. Not covered

Explosions: the track's explosion and fire effects are Giotto `GANM`s (`Ex_*`, `shipfire`,
`fireplacebig`, ..., section 7); the event-started ones are not ported. Muzzle flashes: not
traced. `RParticulate` (weather), lightning, water and shadows are not ported.
