# Driving: particle effects, wheel smoke / spray / sparks, glare

Port: `crates/nf-formats/src/drivefx.rs` and `crates/nf-formats/src/trackfx.rs` (data),
`crates/nf-game/src/driving/effects.rs`, `crates/nf-game/src/driving/trackfx.rs` and
`crates/nf-game/src/driving/smashfx.rs` (logic).
Disc checks: `crates/nf-view/tests/driving_effects_live.rs`, `crates/nf-formats/tests/trackfx_disc.rs`
(every placed effect per track with its kind), `crates/nf-game/tests/drive_trackfx_disc.rs`,
`crates/nf-game/tests/drive_smashfx_disc.rs`.

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

`gg`, `gd`, `gs` and the event-started effects are in sections 9-12 (`driving::smashfx`).
**Not ported**: the point light (type 5, none on the disc); how the particle manager ticks the
Giotto emitters (`FUN_001e65c0` is reached through a vtable; the port ticks each once per call);
the frame counter's rate; the quaternion handedness (only identity orientations were checked).

**Frame rule found later** [M `FUN_00232af8` -> `FUN_00234d80` stack args, `FUN_002345d8`]:
the instance's bit 43 is the effect's `Flgs` bit 0. Without it the run frame `+0x5c` is reset
to 0 at every tick, so the elements always see frame 0 and LIFE never ends the effect (a forever
emitter is switched on again every tick). The load-started effects of `TrackFx` (Wd_Smoke,
waterfall, ...) are of this kind; `TrackFx::tick_particles` still stops calling them after LIFE
(the emitters keep ticking, so the difference is only the repeated switch-on).

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
  1) corners + four words. The add `FUN_001da6c8(width, height, manager, centre, p5, p6,
  record)` [M] builds two triangles (rows 0,1,2 and 1,2,3) on a 96-slot ring: a = (p6 x p5) *
  width, b = ((p6 x p5) x p5) * height, corner 0 = centre - a/2 - b/2 (both edges are
  perpendicular to p5: p5 is the decal's normal, p6 the "up" that turns it). Word `+0x44`
  = the number of decal draws (`FUN_001da978`, one decrement each) before the decal gets its
  alpha (0 means 1); `+0x48` = that alpha (0x3f, 0x20 or 0x28 on the disc); until then the
  colour is 0x007f7f7f (alpha 0). Words `+0x40` and `+0x4c` are not read by either (record 7's
  `+0x40` holds the bytes "data", leftover text). `nf_formats::trackfx::decal_style`. The
  callers (`FUN_001ea8e8`, `FUN_001eb078`) are impact code, not track placements: no track
  places skid marks or oil from `decal.dat`. The ring is ported in section 12.

## 9. Giotto elements: `gg`, `gd`, `gs` (`driving::smashfx`) [M + D]

Data: `nf_formats::trackfx::{GiottoGlareKey, GiottoBurst, GiottoSound, GiottoElement}`;
`GiottoEffect::elements` keeps every keyed element in the order `FUN_00234818` runs them.
Census [D, `drive_smashfx_disc.rs`] (gp / gg / gd / gs elements with keys, left over):
gall1 330 / 18 / 38 / 21 / 0, sav_gall1 208 / 15 / 39 / 13 / 0, ears_gall1 297 / 17 / 26 / 6 / 11.

**Instance frames** (`FUN_002345d8`): the key frame (`+0x68` low word) counts every tick; the
run frame `+0x5c` counts only for `Flgs` bit 0 (section 7). The elements run while the run frame
is <= LIFE; the run at LIFE first sets the end bit (bit 44), which `gg`, `gd` and `gs` test, so
that run is the `gp` emitters only. A `Flgs` bit 0 effect keeps no handle (`FUN_00232af8`
returns 0): it is fire-and-forget.

**Random stream**: `gg` jitter and `gd` bursts draw from the Giotto stream at 0x33ba74 (same
generator as the engine's, seeded `FUN_0024b328(0x642f2b)` = 175 steps, from `FUN_0024afe0`);
the debris spawn draws from the engine stream 0x333e60 (`DriveRng`). Who calls `FUN_0024afe0`
(once at boot or per mission) is not traced.

**`gg`** (`FUN_00233df8`, `FUN_00235ea0`, `FUN_00235d50`, `FUN_001f1f88`): key `+0x00` offset,
`+0x0c` frame (int), `+0x30` glare type, `+0x34` size, `+0x38` jitter. The pair for the key
frame: frame 0 gives key 0 twice; a frame equal to a key frame stays on the earlier pair; past
the last key, the last key twice. t = (frame - fa)/(fb - fa) (0 when equal); size = |a.size +
(b.size - a.size) t|; type and jitter from the earlier key; position = instance position + the
earlier key's offset (not rotated). Jitter: three glares, each moved by 2r/65536 - 1 per axis.
The glare is added straight to the glare manager's list (flags 0, kind 2, pos.w 1; at most 128
a frame, shared with the placed glares; the order against them is not traced): no pulse, no
distance cull, alpha 0x20 (`trackfx::glare_sprites` draws it). `Wd_Sprk`'s glare is type 43
growing 0 -> 0.5 over frames 0-5, then types 0 and 1 [D].

**`gd`** (`FUN_00235810`, once per instance; the element's first key): `+0x10` quaternion,
`+0x30` size, `+0x34` / `+0x38` / `+0x3c` upward speed, vertical spread, horizontal spread
(each x size), `+0x40` ARGB, `+0x44` kind, `+0x4c` / `+0x50` / `+0x54` position spread x / z /
y, `+0x58` piece size, `+0x5c` gate (0 = no spawn). Kind 0: one piece of debris type 4, size
0.0049, velocity = (instance x key quaternion) applied to (0, side spread x size, 0) + (0, 0.018,
0), colour converted to GS twice. Other kinds: count = trunc(2 x size), at most 7 when size <= 15
(no cap above 15); per piece three draws for the velocity, three more for the offset when a
spread is set (turned by the instance orientation); types: kind 2 -> 0, 1, 2 in turn, 3 -> 5, 6
in turn, 1 and 4..19 -> the table at 0x39c980 (3, then 7..22). Examples [D]: `Wd_Glass` 7 + 7
glass pieces (types 8, 7) + 7 damage pieces (5, 6); `Sm_Wsplinter` 7 + 7 wood splinters (17,
18); `Sm_Debris` 6 each of types 11, 12, 13 (mail, rubbish).

**`gs`** (`FUN_002363b0`; the element's first key): `+0x30` volume %, `+0x38` bank slot,
`+0x3c` name (to `+0x5c`), `+0x5c` -> voice `+0x70` (x 0.01), `+0x60` / `+0x64` -> voice
`+0x5c` / `+0x60` (and its square at `+0x64`). Group `Giotto`. The voice is made on the first
run when the name is not `(null)`, at most 8 sound elements; later runs move it to the instance
position. `Wd_Glass` plays `SFX_GlassShatter3` from bank 2 at volume 1.0, 10 / 100 [D]. The
readers of voice `+0x5c/+0x60/+0x70` are not traced (by their use: near / far distance, pitch).

## 10. Debris (`FUN_001d94a8` family) [M]

23 types (table 0x338e28, stride 0x14: model `dmge dmge pipe dgls shel dmge dmge dgls dgls coin
mail mail grbg grbg frut frut dmge wdsp wdsp dgls wdsp wdsp dmge`), 64 pieces each. Spawn
`FUN_001d9bb8`: only while the count is below 64 and the gate is non-zero; pos, velocity, birth =
the sim tick (as a float in the velocity's w), axis = three engine draws 2r/65536 - 1 normalised
(the VU `VMULq` has dest xyz: the spin in w is not scaled), spin = r/65536 x pi/35, the colour
through `argb_to_gs`; every spawn sets the type's size (`+0xd10`) and life 70 (`+0xd14`).
Per frame `FUN_001d9560` -> `FUN_001d9920` for every type but 4: with t = now - 70 (now =
`DAT_00334000`, the tick as a float), a piece born before t is dropped (the last piece takes its
slot and is looked at next); any other gets alpha = round(alpha x t / birth), read from and
written to its own colour each frame (so the fade compounds and depends on the absolute tick).

**Motion** [M disassembly of the VU1 program, D ELF bytes; `DebrisPiece::pose`]. `FUN_001d9920`
draws through `FUN_00285388` with the render list at 0x348c60, named `BondDebris` (parameters
`Parameters`, `viewProjection`, `Texture`, `ControlData`, `Coordinates`, `UV`, `State`,
`SimTick`, `SplitReg1`, `Offsets`, `Quats`, `InstCols`, `Velocity`). Its op 1 loads the
microprogram at 0x349020 (209 instructions; DMA chain 0x3496e0: MPG to micro 0, REF 0x68 qwords).
`SimTick` is the qword at 0x334000: x = the tick, y = 0x334004 = -0.002 (ELF .data, no writer).
Per piece (micro 0x019-0x036): t = now - birth; a = spin x t - 3.95; q.xyz = axis x ESIN(a), q.w =
ESIN(a + pi/2) (the VU sine series, used outside -pi/2..pi/2 as is). Then (micro 0x045-0x06a):
pos = p0 + v t, pos.y += -0.002 t^2 (no 1/2), the standard rotation matrix of q, and the model
(table 0x338e28: `+0x00` coordinates, `+0x04` UVs, `+0x0c` vertex count, all in the ELF) is drawn
by it. There is no ground test, bounce or drag anywhere in the program or in `FUN_001d9920`: a
piece falls through the road until its 70 ticks end (9.8 below its straight line by then). The
size word `+0xd10` is uploaded as `SimTick.w` and not read by the program. A fresh piece is
already turned (q.w = -0.69 at t = 0), and q is not unit length (the series is not exact).
Type 4 (kind-0 bursts) is never faded, expired or drawn: no other pass handles it.

## 11. Event-started effects: blocks, hits, explosions [M]

**The block** (`FUN_001a1f58`, `RSceneObj +0x28`, scene objects and car models alike): an entry
is kept only when `FUN_001a1bc8` passes: flags & 0x109e and, for a sub-item whose byte `+0x0c`
has bit 7, flag 0x200 (`PlacedEffect::scene_kept`). Masks: `+0x28` flag 8 (damage-driven),
`+0x38` flags & 0x96 (event-startable), `+0x30` type 6; per entry the event mask `+0x12` (the
flags' high half) and the id `+0x17`.

**Start / stop** (`FUN_001ca198`): on, not running: a non-Giotto entry (glare, light) is marked
running; a Giotto entry with no handle starts its effect (`FUN_002330b0`) and is marked only
when a handle comes back. Off, running: unmarked, and a Giotto handle is stopped
(`FUN_00233160`). Build (`FUN_001ca048`): non-damage entries with flag 0x10, or flag 4 when the
build's reset argument is set, start.

**Events** (`FUN_001caba8`, object vtable +0x84; the car's `FUN_001d3c28` passes the others
through): 0 / 2 start / stop the event entries outside the damage mask whose mask meets the
argument's high half (or both are zero), 1 / 3 the entry with that id; then the keep-alive
`+0x20` = tick + 600 while anything runs. 4 / 5 call `FUN_002331c0` on running Giotto entries
(not traced). 9: new damage bits restart the damage entries: on when their mask meets the
accumulated bits, off otherwise; a Giotto one with no handle is marked anyway. 10: model part
animations only. Out of range (`FUN_001ca340`, when the keep-alive has passed): within 170 of
the camera, 600 more ticks; else running Giotto entries without flags 0x14 stop.

**Who sends what** [M]: a smackable hit (`FUN_0019e8c0`, `Smackable` vtable 0x37cec4 slot 3):
damage (>= 0) comes off the hit points; still standing, with f = damage taken / start: event 10
above 0.15, event 9 (all bits) above 0.75, part anims 0x13..0x16 by f and 4/5/6/0x19 by the
hit's size; knocked down (or hit again at 0): part anims 2, 6/7, 0x13..0x16, 4..6, 0x19 and
event 10 only. A sentry hit (`FUN_0019b5a8`, `Sentry` vtable 0x37c260 slot 3; earlier listed
here as the smackable) reads health left instead: event 10 above 0.15, event 9 above 0.75. A
car panel hit (`FUN_00189598`): per panel the larger of its
two damage words; event 10 bit above 0.35, event 9 bit above 0.75. The car step
(`FUN_00188200`) sends event 0 / 4 with its `+0x6c` / `+0x70` bits (x 0x10000) and with 0x800000
by speed; the writer of `+0x6c` is not traced. On the disc's car models the panel bits start
`Ex_Glass` and `Cr_Dmg2` entries (e.g. `coupe`: panel 1 starts one `Cr_Dmg2` and four
`Ex_Glass`) [D].

**The paris smash effects** [D through the M filter]: every `P_streetlight` holds its lamp glare
(flags 0x14, the only block entry, lit at build) and `Wd_Sprk` x 2 and `Wd_Glass` with flags 0;
the bench `Sm_Wsplinter`, newsbox `Sm_Debris`, the mall glass and the university arch debris are
flags 0 too (only the hydrant's spray, flags 0x00020002, and its sound, flag 4, are block
entries). Flags-0 entries never enter a block, so no object event starts them. The calls that
start a Giotto effect (every caller of `FUN_00232af8`'s wrappers `FUN_002330b0`, `FUN_00233108`,
`FUN_00232f70`): the block (`FUN_001ca198`), world load (`FUN_0022faf8`, `FUN_00230730`),
`EPlayEffect` (`FUN_00153f80`, an entry named by the action), `EPlayActorEffect`
(`FUN_00153560`), `EPlayGiottoEffect` (`FUN_001543e8`), `EPowerUp` (`Pr_Dstry`),
`ESpawnForceEffect` (`FUN_0015b2f8`: an entry outside the filter by its own name, `Ex_Snow` on
surface 9 / `Ex_Small` without one, else event 1 with the id), the explosion handler, the
collision event `ECollision` (`FUN_00149ed8`: `bigsplash`, `Cr_Dirt`, `CR_Wasm`, `CR_WasmS` and
`Un_SandCar` and `Un_DebrisW` on underwater tracks), the window hit `EHitWindow` (`FUN_0014f480`: `Wd_Glass` and `Wd_Wsmk` by
name at the pane, for shots and missiles through `FUN_002153b0`, not for cars) and the bullet
impact `FUN_0019d5a8`. `FUN_0015b2f8` was listed here as `ESpawnExplosionStatic`: it is
`ESpawnForceEffect` (vtable 0x370468), action 0x8d, which no script on the eight tracks uses;
`ESpawnExplosionStatic` is 0x9a (`FUN_0015b010`, spawns an `ESpawnExplosion`) [D]. On the disc
the 14 `EPlayEffect` actions all name hydrant entries, and no action argument names a street
light [D `no_script_action_starts_a_street_light_effect`]. So a lamp a car knocks over plays its
part animations and never starts its `Wd_Sprk` / `Wd_Glass` entries: no path in DRIVING.ELF
reaches them (only a live capture can say whether anything outside these paths does).

**Explosions** (`ESpawnExplosion`, fields stored by `FUN_00159fb8`, handler `FUN_0015a098`): the
effect = the explicit name; weapon 0x1e `FODHitEffect`; surface 9 `snowspecificexplosion`;
surface 6 `rocketsplash`; an underwater track `Un_ExplL/M/S` (damage > 600, > 200); surfaces
2..5, 10, 0x50, 0x60 `Ex_SmallD`, `2`, `3` by tick % 3; else `Ex_SmB00..02` when the player's
car was hit, `Ex_Small..3` otherwise, by a shared 0, 1, 2 counter. Then the scorch decal
(`FUN_001eb078(8, .., 5)`), the light, and two voices from bank 4: one of `SFX_HugeExp`,
`SFX_LrgExp`, `SFX_MedExp`, `SFX_SmallExp` (rotating; a name is skipped while its clock time +
30 has not passed; none free = no sound) in `Explosions`, near / far 100 / 400 (x5 on
`jungleb_mis13b`), and `SFX_DistExp` in `ExplosionBass` (3 / 350). A missile or rocket ground
hit (`FUN_00182978`): weapon 0xe makes nothing; else the hit mark `FUN_0019ddb8(damage, damage
x 0.008, .., 5)` (section 12) and `FUN_00200088` -> `ESpawnExplosion` with the weapon, the
damage and the ground's surface byte, no car.

**A destroyed car** [M]: no code path spawns an explosion when a car's hit points reach 0. The
code callers of `FUN_00200088` are weapon and projectile paths (`FUN_00155ea8`, `FUN_0017e8b8`,
`FUN_0017ffc8`, `FUN_001802e0`, `FUN_00180620`, `FUN_00182978`, `FUN_00199f88`,
`FUN_0019a710`, `FUN_0019d080`, `FUN_0020ea60`), and `FUN_00159fb8` is reached only from
`FUN_00200088` and the script actions `ESpawnExplosion` (`FUN_0015ae90`) and
`ESpawnExplosionStatic` (`FUN_0015b010`). The car step `FUN_00188200` at hit points <= 0 calls
vtable +0x7c (`FUN_0018adc0`: clears its group's `+0x40`) and, for a category-2 car (`+0x2c8`) outside the
`tunnel` / `openheli` / `redheli` models with no part animation running, credits the kill
(vtable +0x16c = `FUN_00193140`). The damage virtual's wreck branch sends part animations and
model event 10. What a player sees of a wreck is the panel damage entries (`Ex_Glass`, `Cr_Dmg*`
by event 9) and event 0 bits (`Cr_Dmg4` on bit 31); the writer of the car's `+0x6c` bits is still
not traced. Scripted wrecks use `ESpawnExplosionStatic` (MIS01 names `cr_Dmg3big` and
`parisendexplosion` [D]). **Not traced**: the radius damage `FUN_00200200`, the light
`FUN_001afa88`, the unit of the sound clock `FUN_00261038`.

## 12. The decal ring [M]

`FUN_001da6c8` (section 8 for the record words): 96 slots, overwritten in turn; a decal never
fades out, only its slot is reused 96 adds later. `FUN_001da978` per decal draw: the delay
counts down and on 0 the six vertex colours become alpha << 24 | 0x7f7f7f. The flat ground
placement `FUN_001eb078(radius, ground, pos, record)`: refused on surface 6, with the ground's
`+0x2d` bit 1, or on an object whose `+0x1c -> +0x0c` byte is set; r = min(radius / 2, 6);
normal = (v1 - v0) x (v0 - v2) normalised, y made positive and capped at 0.9999 (not
renormalised), (0, 1, 0) without a triangle; refused below y 0.95 or without a triangle hit;
except for record 4, the points x + r, x - r, then x - r with z + r and z - r must all hit
triangles within 0.15 of the height, the last not surface 9; size = r - (pos.y - height) / 2
must be positive; the decal sits at height + 0.025, 2 size square, up (0, 0, 1). Callers:
explosions (radius 8, record 5: delay 5, alpha 0x28 [D]), Q-Slick (`FUN_0020de20` case 10:
radius 4.2, record 4).

**Hit marks** (`FUN_0019ddb8` -> `FUN_001ea8e8`, `smashfx::hit_decal`) [M + disassembly]: a
mark only when the hit's `+0x52` byte is 0, its surface is not 0x70 (glass) and its `+0x51` bit
1 is clear; the struck scene object gets part anims 0x16 / 0x15 / 0x14 / 0x13 at damage >= 500 /
250 / 50 / 10. Callers: `FUN_00199f88` radius 0.0625 (0.3125 for weapon 0x1d), record 2;
`FUN_0019d080` its `+0x70` x 0.005, record 5; the missile ground hit damage x 0.008, record 5.
Placement: refused on surface 6 or the ground's `+0x2d` bit 1; r = min(radius, 6); reference
axis (0, 1, 0), or (0, 0, -1) when |normal.y| > 0.85; side = normal x axis. Above r 0.33 three
segments through the surface (pos +/- normal / 4) offset by + side r, - side r, - side r + axis
r must hit something other than surface 6; on a miss r halves, two tries. Centre = pos + normal
x (|(1 - normal.y) 0.05| + 0.035); nothing on a flagged object; the mark is `2r` square
(normal, up = side) unless the last segment hit had flag bit 1. A wall hit (|normal.y| <= 0.85)
within r above a ground triangle also gets a `2r` x `8r` streak at the wall's foot (moved back
4 x the drop, at the plane height + the lift), when the ground normal has |y| >= 0.98.

**Bullet impact** (`FUN_0019d5a8`, `smashfx::bullet_impact`) [M]: the Giotto effect `Henbult`
(surface or material 0x80), `bullethe` (the player's car, unless its `+0xc0` flag), `bulletcar`
(cars of kind 1 / 10, surface 0xd), `snowbullethit` (9, 8, 0x30, 0x40), `bulletsplash` (6),
`bulletHut` (0xc), by weapon with no car `bulletrockH` (0x1b), `bulletrock` (0x1d), `Bullgat`
(0x1a), else `bullet`; two `BKPS` spark particles unless snow or the flagged player car; a voice:
one draw under 0.25 picks `SFX_Ricochet1..3` in turn (`Bullet Ricochet`), else metal (5 by a
second draw), wood, dirt, glass, flesh or concrete 1 / 2 alternating by the surface nibbles
(`Bullet Impact`).

## 13. Not covered

Muzzle flashes: not traced. `RParticulate` (weather), lightning, water and shadows are not
ported. Nothing draws the items of sections 7-12 yet.
